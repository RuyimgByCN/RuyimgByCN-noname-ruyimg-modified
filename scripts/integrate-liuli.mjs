/**
 * noname-liuli 整合脚本 (M4)
 *
 * 把 Noname-Liuli-Modified 的独有武将包 + 全部扩展，
 * 用 ESM 包装层整合进 Noname-ruyimg-Modified/apps/core。
 *
 * 用法: node scripts/integrate-liuli.mjs
 * (从仓库根目录 Noname-ruyimg-Modified/ 运行)
 */
import { readFileSync, writeFileSync, existsSync, readdirSync, statSync, mkdirSync, cpSync } from "node:fs";
import { join, basename, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = join(__dirname, ".."); // Noname-ruyimg-Modified/
const APPS_CORE = join(REPO_ROOT, "apps", "core");
const LIULI_ROOT = join(REPO_ROOT, "..", "upstream", "noname-liuli", "noname");
const LIULI_CHARS = join(LIULI_ROOT, "character");
const LIULI_EXTS = join(LIULI_ROOT, "extension");

// 已存在于 apps/core 的武将包(不覆盖)
const EXISTING_CHARS = new Set(readdirSync(join(APPS_CORE, "character")).filter(f => {
	const p = join(APPS_CORE, "character", f);
	return statSync(p).isDirectory() || f.endsWith(".js");
}).map(f => f.replace(/\.js$/, "")));

// 不可覆盖的原版扩展(本仓库自带的 stockextension，非 Liuli 来源)
// Liuli 同名扩展会加 _liuli 后缀避免覆盖；其余 Liuli 扩展重跑时直接覆盖更新
const STOCK_EXTENSIONS = new Set(["boss", "cardpile", "coin", "wuxing"]);

const log = [];
function logln(s) { console.log(s); log.push(s); }

// Liuli 武将包中文名映射(从其 translate 或常见名推断，不确定的用拼音)
const CHAR_DISPLAY_NAMES = {
	ddd: "点将DDD", gujian: "古剑奇谭", gwent: "巫师牌", hearth: "炉石传说",
	jiange: "守卫剑阁", mtg: "万智牌", ow: "守望先锋", swd: "仙剑奇侠传",
	xianjian: "仙剑奇侠", xiake: "侠客风云", xinghuoliaoyuan: "星火燎原",
	yxs: "英雄杀", zhuogui: "捉鬼",
};

/**
 * 花括号配对扫描器(感知字符串/模板字面量/注释)。
 * 从 src[startOpen] 处的 '{' 开始,找到与之匹配的 '}' 的下标。
 * 返回 '}' 的下标,找不到返回 -1。
 */
function matchBrace(src, startOpen) {
	let depth = 0;
	let inStr = false;
	let strCh = "";
	let escaped = false;
	for (let i = startOpen; i < src.length; i++) {
		const c = src[i];
		if (escaped) { escaped = false; continue; }
		if (inStr) {
			if (c === "\\") { escaped = true; continue; }
			if (c === strCh) inStr = false;
			continue;
		}
		if (c === '"' || c === "'" || c === "`") { inStr = true; strCh = c; continue; }
		if (c === "/" && src[i + 1] === "/") {
			while (i < src.length && src[i] !== "\n") i++;
			continue;
		}
		if (c === "/" && src[i + 1] === "*") {
			i += 2;
			while (i < src.length - 1 && !(src[i] === "*" && src[i + 1] === "/")) i++;
			i++;
			continue;
		}
		if (c === "{") depth++;
		else if (c === "}") {
			depth--;
			if (depth === 0) return i;
		}
	}
	return -1;
}

/**
 * 从 Liuli IIFE 文件提取 return 对象的字面量文本。
 * 用于武将包(结构为 game.import("character", function(){ return {...} }))。
 * 策略: 找 "return {" 的位置，用花括号配对找到匹配的 "}"，
 * 截取这段(含 return 关键字到对象结束)。
 */
function extractReturnObject(src) {
	// 定位第一个 "return {"
	const retMatch = src.match(/\breturn\s*\{/);
	if (!retMatch) return null;
	const start = retMatch.index;
	const openBrace = src.indexOf("{", start);
	if (openBrace === -1) return null;
	const end = matchBrace(src, openBrace);
	if (end === -1) return null;
	return src.slice(start, end + 1).trim();
}

/**
 * 修复 ES Module 严格模式不兼容的代码。
 *
 * ESM 默认严格模式,而 Liuli 扩展原运行在非严格脚本环境,存在两类不兼容:
 *   1. delete <标识符>;  —— 严格模式禁止 delete 变量(只允许 delete 属性)
 *      例: 手杀ui "delete div2;", 祖安武将 "delete ssr_guidao_zhou;"
 *      修复: 改为 <标识符> = undefined;(语义等价,变量已声明,解除引用)
 *
 * 注:仅处理「delete 纯标识符」(非属性访问),不误伤 delete obj.prop。
 */
/**
 * 收集代码中的「隐式全局赋值」变量名(未用 var/let/const 声明)。
 *
 * 严格模式(ESM 默认)下,给未声明变量赋值会 ReferenceError。
 * Liuli 扩展原运行在非严格脚本环境,常见隐式全局如:
 *   十周年UI 的 SSEquip、AI优化 的 minusatt、标记补充 的 radiansToDegrees、
 *   雷霆万钧 的 _ThAnim、龙血玄黄 的 _0x1a60(混淆器)
 *
 * 策略:
 * 1. 扫描形如「行首空白 + 标识符 =」的赋值(不限大小写),排除已声明的和 JS 关键字。
 * 2. 额外扫描混淆器变量名(_0x 开头),这类变量常嵌在 IIFE 内部赋值,行首正则漏掉。
 */
function collectImplicitGlobals(code) {
	const implicit = new Set();
	// JS 关键字与保留字白名单(行首裸赋值这些通常不是隐式全局,而是属性或解构)
	const KEYWORDS = new Set([
		// 声明/控制流
		"break", "case", "catch", "class", "const", "continue", "debugger", "default",
		"delete", "do", "else", "export", "extends", "finally", "for", "function",
		"if", "import", "in", "instanceof", "let", "new", "of", "return", "super",
		"switch", "this", "throw", "try", "typeof", "var", "void", "while", "with", "yield",
		// 字面量/全局
		"true", "false", "null", "undefined", "NaN", "Infinity",
		// 常见对象属性名(行首裸赋值罕见但出现时是属性,非隐式全局)
		"name", "value", "length", "size", "type", "key",
	]);
	// 先提取所有 var/let/const 声明过的标识符(支持逗号分隔多声明:let A = 1, B, C = 2)
	const declared = new Set();
	const declStatementRe = /\b(var|let|const)\s+([a-zA-Z_$][\w$]*)/g;
	let dm;
		while ((dm = declStatementRe.exec(code)) !== null) {
			declared.add(dm[2]);
			// 继续找同一声明里逗号后的标识符(到分号/换行/{ 结束)
			// 注意:遇到 { 必须停止,否则会跨进函数体/对象字面量,误判内部赋值为声明
			// (如龙血玄黄混淆器: var _0xod3='x',_0xod3_=function(){...,_0x1a60=[...]})
			let j = dm.index + dm[0].length;
			while (j < code.length) {
				// 跳过赋值部分(= ... 到下一个逗号/分号/换行/{ 结束)
				while (j < code.length && code[j] !== "," && code[j] !== ";" && code[j] !== "\n" && code[j] !== "{") j++;
				if (j >= code.length || code[j] !== ",") break;
				j++;
				while (j < code.length && /[\s]/.test(code[j])) j++;
				const idMatch = code.slice(j).match(/^([a-zA-Z_$][\w$]*)/);
				if (idMatch) { declared.add(idMatch[1]); j += idMatch[1].length; }
			}
		}
	// 行首(缩进)+ 任意标识符 = 赋值(放宽:不限大写开头)
	const assignRe = /^(\s+)([a-zA-Z_$][\w$]*)\s*=/gm;
	let m;
	while ((m = assignRe.exec(code)) !== null) {
		const varName = m[2];
		if (!declared.has(varName) && !KEYWORDS.has(varName)) {
			implicit.add(varName);
		}
	}
	// 混淆器变量名(_0x 开头)特殊扫描:这类变量常嵌在 IIFE 内部赋值(如龙血玄黄),
	// 行首正则无法捕获。扫描所有 _0x 标识符的赋值,未声明的加入预声明列表。
	const obfRe = /([a-zA-Z_$][\w$]*)\s*=/g;
	while ((m = obfRe.exec(code)) !== null) {
		const varName = m[1];
		if (varName.startsWith("_0x") && !declared.has(varName)) {
			implicit.add(varName);
		}
	}
	return [...implicit];
}

/**
 * 修复 ES Module 严格模式不兼容的代码。
 *
 * 1. delete <标识符>; → <标识符> = undefined;
 *    严格模式禁止 delete 变量(只允许 delete 属性)。
 *    例: 手杀ui "delete div2;", 祖安武将 "delete ssr_guidao_zhou;"
 *
 * 2. 隐式全局赋值预声明:在函数体顶部插入 var 声明。
 *    例: 十周年UI "SSEquip = {...}"(未声明)→ 顶部加 "var SSEquip;"
 *
 * 注:仅作用于函数体(由 generateExtensionESM 传入 mainBody / extra body)。
 */
function fixStrictMode(code) {
	let fixed = code.replace(
		/^(\s*)delete\s+([a-zA-Z_$][\w$]*)\s*;\s*$/gm,
		(_, indent, name) => `${indent}${name} = undefined;`
	);
	// 收集隐式全局并在顶部预声明
	const implicit = collectImplicitGlobals(fixed);
	if (implicit.length > 0) {
		const decl = `\t// strict-mode 补丁:预声明隐式全局变量\n\tvar ${implicit.join(", ")};\n`;
		fixed = decl + fixed;
	}
	return fixed;
}

/**
 * 提取所有 game.import("extension", function(...){...}) 回调的【完整函数体】。
 *
 * 返回数组,每个元素是一个函数体文本(含 return 前的前置语句 + return 对象)。
 *
 * 为什么不只取 return 对象:
 *   Liuli 扩展结构常为
 *     game.import("extension", function(lib,game,ui,get,ai,_status){
 *       window.xxx = {...};        // 前置全局定义(选将美化的 xiguaxuanjiang)
 *       var helper = function(){}; // 前置辅助
 *       return { name, content, ... };
 *     });
 *   只取 return 对象会丢失 window.xxx 定义,导致对象字面量内裸引用 xiguaxuanjiang 时 ReferenceError。
 *   保留整个函数体,用 IIFE 执行,前置语句会先执行,return 结果作为导出。
 *
 * 多个 game.import(如祖安武将含 "祖安武将" + "祖安设置" 两个):
 *   全部提取,在 ESM 里依次执行,最后一个 import 的 return 作为 default 导出。
 */
function extractImportCallbackBodies(src) {
	const bodies = [];
	// 匹配 game.import( 然后跳过空白和字符串参数 "extension" 和逗号,定位到 function 后的 '{'
	// 用正则找所有 game.import( 调用点,再逐个解析参数
	const importRe = /\bgame\s*\.\s*import\s*\(\s*(['"])extension\1\s*,/g;
	let m;
	let firstImportStart = -1; // 第一个 game.import 的起始下标(用于切 preamble)
	while ((m = importRe.exec(src)) !== null) {
		if (firstImportStart === -1) firstImportStart = m.index;
		// 从 importRe.lastIndex 开始,跳过空白,期望 function( 或 函数表达式
		let i = m.index + m[0].length;
		// 跳过空白
		while (i < src.length && /\s/.test(src[i])) i++;
		// 期望 'function' 关键字(具名/匿名均可)
		const funcRest = src.slice(i).match(/^function\b\s*/);
		if (!funcRest) continue;
		i += funcRest[0].length;
		// 跳过函数参数列表 (...) —— 用括号配对
		if (src[i] !== "(") continue;
		let parenDepth = 0;
		let parenEnd = -1;
		for (; i < src.length; i++) {
			if (src[i] === "(") parenDepth++;
			else if (src[i] === ")") { parenDepth--; if (parenDepth === 0) { parenEnd = i; break; } }
		}
		if (parenEnd === -1) continue;
		i = parenEnd + 1;
		// 跳过空白,期望 '{' 函数体开始
		while (i < src.length && /\s/.test(src[i])) i++;
		if (src[i] !== "{") continue;
		const openBrace = i;
		const closeBrace = matchBrace(src, openBrace);
		if (closeBrace === -1) continue;
		// 函数体 = openBrace+1 .. closeBrace-1 (不含最外层花括号)
		const body = src.slice(openBrace + 1, closeBrace);
		bodies.push(body);
		// 推进 importRe.lastIndex 到函数体之后,继续找下一个 game.import
		importRe.lastIndex = closeBrace + 1;
	}
	// preamble = 第一个 game.import 之前的代码(可能含 window.xxx 全局定义,如史诗卡牌的 window.JK)
	const preamble = firstImportStart > 0 ? src.slice(0, firstImportStart).trim() : "";
	return { preamble, bodies };
}

/**
 * 生成 ESM 武将包文件。
 * 格式:
 *   import { lib, game, ui, get, ai, _status } from "noname";
 *   export let type = "character";
 *   export default function() {
 *     <原 return 对象>
 *   }
 */
function generateCharacterESM(src, packName) {
	const retObj = extractReturnObject(src);
	if (!retObj) return null;
	// retObj 形如 "return { name:'ddd', ... }"
	const objLiteral = retObj.replace(/^return\s*/, "").trim();
	return `import { lib, game, ui, get, ai, _status } from "noname";

// 整合自 Noname-Liuli-Modified (M4)，原文件 character/${packName}.js
// ESM 包装层：保留原 IIFE 的 return 对象，适配新版 import.ts 加载机制
export let type = "character";
export default function () {
	return ${objLiteral};
}
`;
}

/**
 * 生成 ESM 扩展文件。
 * Liuli 扩展: game.import('extension', function(){ window.xxx=...; return {...} })
 *
 * 机制: game.loadExtension 支持 function 参数——会执行 fn(lib,game,ui,get,ai,_status),
 * 拿其返回值作为扩展对象。import.ts 第 77 行调用 game.import("extension", module.default),
 * 所以 module.default 可以直接是【那个回调函数】。
 *
 * 因此 ESM 包装只需: export default 那个函数。前置 window 定义会在 loadExtension 执行时跑。
 *
 * 多个 game.import(祖安武将含 "祖安武将" + "祖安设置"):
 *   无法用一个 default 导出两个扩展。策略:第一个以后的回调,在模块顶层直接重建
 *   game.import 调用注册(其前置语句 + 注册副作用都会发生);default 仍取第一个回调
 *   (主扩展)。注:多 import 极少见,目前仅祖安武将/手杀ui 两个。
 */
function generateExtensionESM(src, extName) {
	const { preamble, bodies } = extractImportCallbackBodies(src);
	if (bodies.length === 0) return null;
	// 第一个回调:作为 default 导出(loadExtension 会执行它);修复严格模式不兼容
	const mainBody = fixStrictMode(bodies[0].trim());
	// 额外的回调:在模块顶层重建 game.import 调用注册(同样需修复严格模式)
	const extraImports = bodies.slice(1).map((body, idx) => {
		const fixed = fixStrictMode(body.trim());
		return `// --- 附加 game.import #${idx + 2} (${extName}) ---
game.import("extension", function (lib, game, ui, get, ai, _status) {
${fixed}
});
`;
	}).join("\n");
	// preamble:第一个 game.import 之前的代码(可能含 window.xxx 全局定义,如史诗卡牌的 window.JK)
	// 放到模块顶层执行,修复严格模式后保留
	const preambleBlock = preamble ? `// --- preamble (game.import 之前的全局定义) ---
${fixStrictMode(preamble)}
` : "";
	return `import { lib, game, ui, get, ai, _status } from "noname";

// 整合自 Noname-Liuli-Modified (M4)，原扩展 ${extName}/extension.js
// ESM 包装层：default 导出原 game.import 回调函数(game.loadExtension 会执行它)
export let type = "extension";
${preambleBlock}${extraImports}
export default function (lib, game, ui, get, ai, _status) {
${mainBody}
};
`;
}

/**
 * 对特定扩展做针对性 patch(修复 content 阶段运行时错误)。
 * 在 M4 ESM 生成 + 素材拷贝之后调用,幂等(已 patch 过不重复)。
 *
 * 这些 patch 修复的是扩展依赖上游运行时状态/作用域/eval 锚点的问题,
 * 无法通过 ESM 包装层统一解决,需逐个 patch。
 */
function applyPatches() {
	const PATCHED_MARKER = "// [liuli-patch] applied";
	const patchFile = (relPath, find, replace, label) => {
		const fullPath = join(APPS_CORE, relPath);
		if (!existsSync(fullPath)) { logln(`[patch] 跳过 ${label} (文件不存在)`); return; }
		let src = readFileSync(fullPath, "utf-8");
		// 幂等:已 patch 过(含 marker 或已含 replace 内容)则跳过
		if (src.includes(PATCHED_MARKER) && src.includes(replace.split("\n")[0])) {
			logln(`[patch] 跳过 ${label} (已 patch)`);
			return;
		}
		if (!src.includes(find)) {
			logln(`[patch] 警告 ${label} (未找到匹配文本,可能源已更新)`);
			return;
		}
		src = src.split(find).join(replace);
		writeFileSync(fullPath, src, "utf-8");
		logln(`[patch] 完成 ${label}`);
	};

	// --- 更新保护: window.extTraList 可能 undefined ---
	// 根因:上游运行时注入的全局对象,本环境没有。加 || {} 守卫。
	patchFile(
		"extension/更新保护/extension.js",
		"var tralist=window.extTraList[tratype];",
		"var tralist=(window.extTraList||{})[tratype];",
		"更新保护 extTraList 守卫"
	);

	// --- 选将美化: lib.skill.jieyue2 可能不存在 ---
	// 根因:技能 ID jieyue2 来自未加载的武将包。加 if 守卫。
	patchFile(
		"extension/选将美化/extension.js",
		"lib.skill.jieyue2.intro = {",
		"if(lib.skill.jieyue2) lib.skill.jieyue2.intro = {",
		"选将美化 jieyue2 守卫"
	);

	// --- 末日乱斗: window.brawlRename 定义在 if(lib.brawl) 块内,使用在块外 ---
	// 根因:lib.brawl(祖安武将注入)为 falsy 时 brawlRename 从未定义。
	// 修复:在 content 函数开头无条件插入兜底定义(不依赖 lib.brawl)。
	patchFile(
		"extension/末日乱斗/extension.js",
		"return {name:\"末日乱斗\",content:function (config,pack){\n    delete lib.characterPack[\"mode_extension_末日乱斗\"];",
		"return {name:\"末日乱斗\",content:function (config,pack){\n    window.brawlRename=window.brawlRename||function(name,rename){return name;};\n    if(!lib.brawl){lib.brawl={};}\n    delete lib.characterPack[\"mode_extension_末日乱斗\"];",
		"末日乱斗 brawlRename 兜底(content 开头)"
	);

	// --- 如真似幻: lib.init.layout('nova') 在 content 阶段过早调用 ---
	// 根因:ui.window 未就绪导致 .hide() 报错。加 try-catch 包裹。
	patchFile(
		"extension/如真似幻/extension.js",
		"lib.init.layout('nova');",
		"try{lib.init.layout('nova');}catch(e){console.warn('[如真似幻] layout 切换失败(可能 ui 未就绪)',e);}",
		"如真似幻 layout try-catch"
	);

	// --- 十周年UI splash.js: eval 锚点失效 ---
	// 根因:$showCharacter.toString() 不含 'this.checkConflict();' 锚点,
	// SC_idx=-1 导致 slice 出错位字符串,eval 抛 SyntaxError。
	// 修复:SC_idx<0 时跳过 eval(保留原函数,yinni_character 注入降级)。
	patchFile(
		"extension/十周年UI/splash.js",
		"lib.element.player.$showCharacter=eval(SC_new);",
		"if(SC_idx>=0){try{lib.element.player.$showCharacter=eval(SC_new);}catch(e){console.warn('[十周年UI] splash yinni_character 注入跳过',e);}}",
		"十周年UI splash eval 锚点守卫"
	);

	// --- 千幻聆音: new Function 创建的函数体含 await(async 函数重写) ---
	// 根因:新内核 useSkill/useCard 是 async 函数(含 await),千幻聆音用 new Function()
	// 重建会创建普通函数(非 async),不支持 await → SyntaxError。
	// 修复:改用 AsyncFunction 构造器(支持 await)。
	patchFile(
		"extension/千幻聆音/extension.js",
		"lib.element.content.useSkill = new Function(qhly_changeUseSkill(lib.element.content.useSkill.toString()));\n      lib.element.content.useCard = new Function(qhly_changeUseCard(lib.element.content.useCard.toString()));",
		"var AsyncFunction = Object.getPrototypeOf(async function(){}).constructor;\n      try { lib.element.content.useSkill = new AsyncFunction(qhly_changeUseSkill(lib.element.content.useSkill.toString())); } catch(e) { console.warn('[千幻聆音] useSkill 重写失败', e); }\n      try { lib.element.content.useCard = new AsyncFunction(qhly_changeUseCard(lib.element.content.useCard.toString())); } catch(e) { console.warn('[千幻聆音] useCard 重写失败', e); }",
		"千幻聆音 AsyncFunction 重写"
	);

	// --- 千幻聆音: decadeUI.animation.loadSpine 可能 undefined ---
	// 根因:decadeUI.animation 是十周年UI 注入的对象,可能未初始化或结构变了。
	// 修复:加 decadeUI.animation 守卫(loadSpine 失败只是动皮不加载,非致命)。
	patchFile(
		"extension/千幻聆音/extension.js",
		"decadeUI.animation.loadSpine(window.qhlyUI.assets.huanpifu.name, \"skel\");\n          decadeUI.animation.loadSpine(window.qhlyUI.assets.pinzhi.name, \"skel\");\n          decadeUI.animation.loadSpine(window.qhlyUI.assets.huanfu.name, \"skel\");",
		"if(decadeUI&&decadeUI.animation&&decadeUI.animation.loadSpine){decadeUI.animation.loadSpine(window.qhlyUI.assets.huanpifu.name, \"skel\");decadeUI.animation.loadSpine(window.qhlyUI.assets.pinzhi.name, \"skel\");decadeUI.animation.loadSpine(window.qhlyUI.assets.huanfu.name, \"skel\");}",
		"千幻聆音 loadSpine 守卫"
	);
	// --- 十周年UI tryContent: event.content 可能 undefined ---
	patchFile(
		"extension/十周年UI/extension.js",
		"event.content(event, step, source, player, target, targets, card, cards, skill, forced, num, trigger, result, _status, lib, game, ui, get, ai);",
		"if(typeof event.content==='function'){event.content(event, step, source, player, target, targets, card, cards, skill, forced, num, trigger, result, _status, lib, game, ui, get, ai);}",
		"十周年UI tryContent 守卫"
	);

	// --- 十周年UI meihua.js: setContent 重写访问 undefined content ---
	// 根因:十周年UI 重写了 lib.element.event.setContent,用 lib.init.parsex 解析 content。
	// 新版内核某些事件名的 lib.element.content[name] 为 undefined,访问 ._parsed 报错。
	patchFile(
		"extension/十周年UI/meihua.js",
		"if (!lib.element.content[name]._parsed) {",
		"if (lib.element.content[name] && !lib.element.content[name]._parsed) {",
		"十周年UI meihua setContent 守卫"
	);

	// --- 手杀全扩: lib.translate 值可能非字符串,trim() 报错 ---
	// 根因:lib.translate[a+'_info'] 可能是 number/object 等,只检查了 falsy 没检查类型。
	patchFile(
		"extension/手杀全扩/extension.js",
		"var last=info.trim().slice(-1);",
		"var last=(typeof info==='string')?info.trim().slice(-1):'';",
		"手杀全扩 info.trim 类型守卫"
	);

	// --- AI优化: 天候换牌 eval 在新版内核失败 ---
	// 根因:旧版 oltianhou.content 是 ()=>-1 简单箭头,AI优化用 eval+replace 覆写它。
	// 新版 content 是 async content(){...} 复杂函数,不含 ()=>-1,replace 匹配不上,
	// eval 执行拼接的完整函数字符串时报 SyntaxError(async content 标识符冲突)。
	// 修复:只在 content 含旧签名 ()=>-1 时才执行 eval 覆写,否则跳过(旧优化对新版不适用)。
	patchFile(
		"extension/AI优化/extension.js",
		"if(game.aiyh_skillOptEnabled('oltianhou','优化〖天候〗换牌ai','oltianhou_replace')) try{",
		"if(game.aiyh_skillOptEnabled('oltianhou','优化〖天候〗换牌ai','oltianhou_replace')&&content.indexOf('()=>-1')!==-1) try{",
		"AI优化 天候换牌 旧签名检查"
	);

	// 替换两处旧式 17 参数调用为新版单参数 event.content(event)
	// tryContent 两处:① 17参数→单参数 event.content(event);② 加 typeof 检查(else分支)
	patchFile(
		"extension/十周年UI/extension.js",
		"event.content(event, step, source, player, target, targets, card, cards, skill, forced, num, trigger, result, _status, lib, game, ui, get, ai)",
		"event.content(event)",
		"十周年UI tryContent 新签名(17参数→单参数)"
	);
	patchFile(
		"extension/十周年UI/extension.js",
		"\t\t\t\tevent.content(event);\n\t\t\t}",
		"\t\t\t\tif(typeof event.content==='function'){event.content(event);}\n\t\t\t}",
		"十周年UI tryContent else typeof 检查"
	);

	// --- 十周年UI tryContent: 适配新版内核 content 调用签名 ---
	// 根因:新版内核 ContentCompiler 编译后的 event.content 是 async function(event),
	// 只接受 1 个参数。十周年UI tryContent 用旧式 17 参数调用,
	// 且某些事件的 content 为 undefined(未编译)导致 not a function。
	// 修复:改为 event.content(event) 单参数调用,与内核 ArrayCompiler 签名一致。
	patchFile(
		"extension/十周年UI/extension.js",
		"event.content(event, step, source, player, target, targets, card, cards, skill, forced, num, trigger, result, _status, lib, game, ui, get, ai);\n\t\t\t} catch(e) {",
		"event.content(event);\n\t\t\t} catch(e) {",
		"十周年UI tryContent 新签名(try分支)"
	);
	patchFile(
		"extension/十周年UI/extension.js",
		"event.content(event, step, source, player, target, targets, card, cards, skill, forced, num, trigger, result, _status, lib, game, ui, get, ai);\n\t\t}\n\t\t}",
		"event.content(event);\n\t\t}\n\t\t}",
		"十周年UI tryContent 新签名(else分支)"
	);

	// --- 十周年UI Create.dialog: for-in 复制 lib.element.dialog 触发 Illegal invocation ---
	// 根因:新版内核 lib.element.dialog 是 getter,返回 class Dialog.prototype(extends HTMLDivElement)。
	// 原扩展 `for (i in lib.element.dialog) dialog[i] = lib.element.dialog[i];` 用 for-in 遍历原型链,
	// 把原生 DOM 方法(getter/setter/绑定函数)拷到普通 div 上,后续访问触发 "Illegal invocation"。
	// 修复:只拷贝 Dialog.prototype 自身的(非继承的、用户定义的)方法,跳过原生继承成员。
	patchFile(
		"extension/十周年UI/extension.js",
		"for (i in lib.element.dialog) dialog[i] = lib.element.dialog[i];",
		"for (i in lib.element.dialog) { try { dialog[i] = lib.element.dialog[i]; } catch(e) {} }",
		"十周年UI Create.dialog for-in 拷贝 try-catch 守卫"
	);

	// --- 更新保护: navigator.notification.confirm 是 Cordova/PhoneGap 专属 API,桌面/Electron 环境不存在 ---
	// 根因:扩展原为移动端 Cordova 设计,navigator.notification 在桌面/Node 环境为 undefined。
	// 修复:用 try-catch 包裹,捕获失败时回退到 alert + 直接 reload(语义等价,单按钮"载入游戏")。
	// 只替换稳定的单行锚点 `tips();`(调用点),前面插入守卫调用,避免脆弱的多行空白匹配。
	patchFile(
		"extension/更新保护/extension.js",
		"    			tips();",
		"    			try { tips(); } catch(e) { alert(message); game.reload3(true); }",
		"更新保护 navigator.notification try-catch 守卫"
	);

	// --- 祖安武将 precontent eval 解包产生 `alive = function(){}` 隐式全局(严格模式 ReferenceError) ---
	// 根因:第 15436 行 eval(function(p,a,c,k,e,d){...}()) 是 packer 混淆器,
	// 解包后执行 `alive = function(c,time,arena){...}`。ESM 默认严格模式下给未声明变量赋值抛
	// ReferenceError。collectImplicitGlobals 是构建期静态扫描,看不到 eval 运行时生成的标识符。
	// 修复:在 precontent 函数体顶部预声明 `var alive;`(eval 与 precontent 共享同一函数作用域)。
	patchFile(
		"extension/祖安武将/extension.js",
		"precontent: function() {\n            // ---------------------------------------启动代码·预加载------------------------------------------//",
		"precontent: function() {\n            // [liuli-patch] 预声明 eval 解包产生的隐式全局(严格模式)\n            var alive;\n            // ---------------------------------------启动代码·预加载------------------------------------------//",
		"祖安武将 precontent eval alive 预声明"
	);

	// --- 祖安武将 content: `if (!datehappen) happen = 0;` 行中隐式全局(严格模式 ReferenceError) ---
	// 根因:collectImplicitGlobals 的正则只匹配「行首缩进+标识符=」,漏掉行中的赋值。
	// `if (!datehappen) happen = 0;` 的 happen 在行中,未被预声明 → ReferenceError。
	// 修复:就地改为 `var happen`,显式声明(原意是局部标记变量)。
	patchFile(
		"extension/祖安武将/extension.js",
		"if (!datehappen) happen = 0;",
		"if (!datehappen) { var happen = 0; }",
		"祖安武将 content happen 隐式全局→显式声明"
	);
}

// ===== 1. 整合武将包 =====
logln("===== 整合武将包 =====");
let charCount = 0;
const charFailed = [];
const allLiuliChars = readdirSync(LIULI_CHARS).filter(f => f.endsWith(".js")).map(f => f.replace(/\.js$/, ""));
for (const pack of allLiuliChars) {
	if (EXISTING_CHARS.has(pack)) {
		logln(`[跳过] ${pack} (apps/core 已存在，不覆盖)`);
		continue;
	}
	const srcPath = join(LIULI_CHARS, pack + ".js");
	const src = readFileSync(srcPath, "utf-8");
	const esm = generateCharacterESM(src, pack);
	if (!esm) { charFailed.push(pack); logln(`[失败] ${pack} (return 对象提取失败)`); continue; }
	const destDir = join(APPS_CORE, "character", pack);
	if (!existsSync(destDir)) mkdirSync(destDir, { recursive: true });
	writeFileSync(join(destDir, "index.js"), esm, "utf-8");
	logln(`[完成] character/${pack}/index.js`);
	charCount++;
}

// ===== 2. 整合扩展 =====
logln("\n===== 整合扩展 =====");
let extCount = 0;
const extFailed = [];
const allLiuliExts = readdirSync(LIULI_EXTS).filter(f => statSync(join(LIULI_EXTS, f)).isDirectory());
for (const ext of allLiuliExts) {
	const liuliExtDir = join(LIULI_EXTS, ext);
	const extJsPath = join(liuliExtDir, "extension.js");
	if (!existsSync(extJsPath)) {
		logln(`[跳过] ${ext} (无 extension.js)`);
		continue;
	}
	// 确定目标名:仅原版 stockextension 加 _liuli 后缀避免覆盖;Liuli 扩展直接用原名(重跑覆盖更新)
	let destName = ext;
	if (STOCK_EXTENSIONS.has(ext)) {
		destName = ext + "_liuli";
		logln(`[改名] ${ext} -> ${destName} (原版 stockextension)`);
	}
	const destDir = join(APPS_CORE, "extension", destName);
	// 复制整个扩展目录(含素材)
	cpSync(liuliExtDir, destDir, { recursive: true });
	// ESM 包装 extension.js
	const src = readFileSync(extJsPath, "utf-8");
	const esm = generateExtensionESM(src, ext);
	if (!esm) { extFailed.push(ext); logln(`[失败] ${ext} (return 对象提取失败，保留原文件)`); continue; }
	writeFileSync(join(destDir, "extension.js"), esm, "utf-8");
	// 补 info.json(若缺)
	const infoPath = join(destDir, "info.json");
	if (!existsSync(infoPath)) {
		const info = { name: destName, author: "Liuli", intro: `整合自琉璃修改版的扩展《${ext}》`, version: "1.0", diskURL: "", forumURL: "" };
		writeFileSync(infoPath, JSON.stringify(info, null, "\t"), "utf-8");
	}
	logln(`[完成] extension/${destName}/`);
	extCount++;
}

// ===== 2.5 针对性 patch(修复 content 阶段运行时错误) =====
logln("\n===== 应用针对性 patch =====");
applyPatches();

// ===== 3. 注册武将包到 game/package.js =====
logln("\n===== 注册武将包到 game/package.js =====");
const packagePath = join(APPS_CORE, "game", "package.js");
let packageSrc = readFileSync(packagePath, "utf-8");

// 检查是否已注册过(M4 标记)
const MARKER = "// ===== Liuli 整合 (M4) =====";
if (packageSrc.includes(MARKER)) {
	logln("[package.js] 已含 Liuli 注册标记，跳过(如需重跑请先手动移除)");
} else {
	// 新增的武将包条目
	const newEntries = allLiuliChars
		.filter(p => !EXISTING_CHARS.has(p) && !charFailed.includes(p))
		.map(p => `\t\t${p}: "${CHAR_DISPLAY_NAMES[p] || p}",`)
		.join("\n");
	// 在 character 对象的闭合前插入(找 "key:" 行作为锚点，character 块以 "key: \"Key\"," 结尾)
	const anchor = "\t\tkey: \"Key\",";
	if (packageSrc.includes(anchor)) {
		const insertBlock = `\n\t\t${MARKER}\n${newEntries}\n`;
		packageSrc = packageSrc.replace(anchor, anchor + insertBlock);
		writeFileSync(packagePath, packageSrc, "utf-8");
		logln(`[package.js] 注册 ${charCount} 个 Liuli 武将包`);
	} else {
		logln("[package.js] 警告: 未找到锚点 'key: \"Key\",'，请手动注册");
	}
}

// ===== 汇总 =====
logln("\n===== 汇总 =====");
logln(`武将包: 成功 ${charCount}，失败 ${charFailed.length}${charFailed.length ? " (" + charFailed.join(", ") + ")" : ""}`);
logln(`扩展:   成功 ${extCount}，失败 ${extFailed.length}${extFailed.length ? " (" + extFailed.join(", ") + ")" : ""}`);

// 写日志
writeFileSync(join(REPO_ROOT, "scripts", "integrate-liuli.log"), log.join("\n"), "utf-8");
logln(`\n日志已写入 scripts/integrate-liuli.log`);
