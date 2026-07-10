/**
 * 运行时错误修复应用器 (一次性,幂等)
 *
 * 对应 integrate-liuli.mjs applyPatches() 中新增的 4 个 patch,
 * 直接应用到运行副本 (dist/ 与 apps/core/),无需重跑整个整合流程。
 *
 * 修复的错误:
 *   1. 十周年UI extension.js:5665 Illegal invocation (Create.dialog for-in 拷贝)
 *   2. 更新保护 extension.js:43 navigator.notification.confirm undefined
 *   3. 祖安武将 extension.js:15436 precontent eval "alive is not defined"
 *   4. 祖安武将 extension.js:828 content "happen is not defined"
 *
 * 用法: node scripts/apply-runtime-fixes.mjs   (从 Noname-ruyimg-Modified/ 运行)
 */
import { readFileSync, writeFileSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";

const __dirname = dirname(fileURLToPath(import.meta.url));
const REPO_ROOT = join(__dirname, "..");

// 4 个需要修复的运行副本根目录(dist 是 http 服务实际加载的,apps/core 是同源副本)
const TARGETS = [
	join(REPO_ROOT, "dist"),
	join(REPO_ROOT, "apps", "core"),
];

// (relPath, find, replace, label) 三元组,与 integrate-liuli.mjs applyPatches() 保持一致
const PATCHES = [
	[
		"extension/十周年UI/extension.js",
		"for (i in lib.element.dialog) dialog[i] = lib.element.dialog[i];",
		"for (i in lib.element.dialog) { try { dialog[i] = lib.element.dialog[i]; } catch(e) {} }",
		"十周年UI Create.dialog for-in 拷贝 try-catch 守卫",
	],
	[
		"extension/更新保护/extension.js",
		"    \t\t\ttips();",
		"    \t\t\ttry { tips(); } catch(e) { alert(message); game.reload3(true); }",
		"更新保护 navigator.notification try-catch 守卫",
	],
	[
		"extension/祖安武将/extension.js",
		"precontent: function() {\n            // ---------------------------------------启动代码·预加载------------------------------------------//",
		"precontent: function() {\n            // [liuli-patch] 预声明 eval 解包产生的隐式全局(严格模式)\n            var alive;\n            // ---------------------------------------启动代码·预加载------------------------------------------//",
		"祖安武将 precontent eval alive 预声明",
	],
	[
		"extension/祖安武将/extension.js",
		"if (!datehappen) happen = 0;",
		"if (!datehappen) { var happen = 0; }",
		"祖安武将 content happen 隐式全局→显式声明",
	],
];

for (const target of TARGETS) {
	console.log(`\n===== 应用到 ${target} =====`);
	for (const [relPath, find, replace, label] of PATCHES) {
		const fullPath = join(target, relPath);
		if (!existsSync(fullPath)) {
			console.log(`  [跳过] ${label} (文件不存在: ${fullPath})`);
			continue;
		}
		let src = readFileSync(fullPath, "utf-8");
		// 幂等:find 仍存在则未 patch;若 find 已消失且 replace 标记存在则已 patch,跳过
		// (不能只看 replace 首行 —— replace 首行常与 find 重叠,导致误判"已 patch")
		const replaceMarker = "[liuli-patch]";
		if (!src.includes(find)) {
			if (src.includes(replaceMarker) || src.includes(replace)) {
				console.log(`  [跳过] ${label} (已 patch)`);
			} else {
				console.log(`  [警告] ${label} (未找到匹配文本,可能源已更新)`);
			}
			continue;
		}
		src = src.split(find).join(replace);
		writeFileSync(fullPath, src, "utf-8");
		console.log(`  [完成] ${label}`);
	}
}
console.log("\n全部完成。");
