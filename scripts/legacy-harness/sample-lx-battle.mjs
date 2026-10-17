#!/usr/bin/env node
// 龙血玄黄战斗触发轨迹采样（D5）：注册期门控 + 技能 trigger 面静态摘录 +
// 无头启动中对 mbqianlong / _CMeffects / Boss 诸葛亮四技 / offline ylyg* 的动态钩挂记录。
// 用法：
//   node sample-lx-battle.mjs [liuli根] [证据输出目录]
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { bootLiuli } from "./lib/boot.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const REPO = path.resolve(HERE, "../..");
const root = path.resolve(process.argv[2] || path.join(REPO, "../scratch/legacy-harness/liuli"));
const outDir = path.resolve(
	process.argv[3] || path.join(REPO, "docs/operations/evidence/2026-10-17-t9-lx-content-hooks")
);
fs.mkdirSync(outDir, { recursive: true });

const INTEREST =
	/^(mbqianlong|_CMeffects|xiangxing|yueyin|fengqi|gaiming|ylyg)/;

function summarizeSkill(name, skill) {
	if (!skill || typeof skill !== "object") return { name, present: false };
	const trigger = skill.trigger || null;
	return {
		name,
		present: true,
		forced: !!skill.forced,
		locked: !!skill.locked,
		charlotte: !!skill.charlotte,
		limited: !!skill.limited,
		unique: !!skill.unique,
		priority: skill.priority,
		audio: skill.audio,
		derivation: skill.derivation || null,
		trigger: trigger
			? Object.fromEntries(
					Object.entries(trigger).map(([k, v]) => [k, Array.isArray(v) ? v : [v]])
				)
			: null,
		hasContent: typeof skill.content === "function" || typeof skill.content === "string",
		hasInit: typeof skill.init === "function",
		keys: Object.keys(skill).slice(0, 40),
	};
}

function extractStaticTriggers(filePath, names) {
	if (!fs.existsSync(filePath)) return { file: filePath, missing: true, hits: [] };
	const text = fs.readFileSync(filePath, "utf8");
	const hits = [];
	for (const name of names) {
		const re = new RegExp(
			String.raw`(?:^|\n)\s*${name.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")}\s*:\s*\{`,
			"g"
		);
		let m;
		while ((m = re.exec(text))) {
			const slice = text.slice(m.index, m.index + 1200);
			const trig = /trigger\s*:\s*\{([\s\S]*?)\}/.exec(slice);
			hits.push({
				name,
				offset: m.index,
				line: text.slice(0, m.index).split("\n").length,
				triggerSnippet: trig ? trig[0].replace(/\s+/g, " ").slice(0, 300) : null,
				head: slice.split("\n").slice(0, 12).join("\n"),
			});
			break;
		}
	}
	return { file: filePath, missing: false, hits };
}

const COMMANDS = [
	"damage",
	"draw",
	"loseHp",
	"recover",
	"gainMaxHp",
	"loseMaxHp",
	"chooseButton",
	"chooseCardButton",
	"chooseTarget",
	"chooseControl",
	"chooseToUse",
	"addSkill",
	"removeSkill",
	"awakenSkill",
	"discard",
	"gain",
	"useCard",
	"addMark",
	"logSkill",
];

function hookSetContent(lib, trace) {
	const eventProto = lib.element && lib.element.event;
	if (!eventProto || typeof eventProto.setContent !== "function" || eventProto.setContent.__lx) return false;
	const orig = eventProto.setContent;
	const wrapped = function (item) {
		const ret = orig.apply(this, arguments);
		const skillName = this && this.name;
		if (
			skillName &&
			INTEREST.test(skillName) &&
			typeof this.content === "function" &&
			!this.content.__lxHooked
		) {
			const parsed = this.content;
			const run = function (...args) {
				if (trace.length < 200) {
					trace.push({
						kind: "skill.content",
						skill: skillName,
						event: this?.name,
						step: this?.step,
						player: this?.player?.name || this?.player?.name1 || null,
						rngDraws: trace.rngDraws ? trace.rngDraws() : null,
					});
				} else {
					trace.overflow = (trace.overflow || 0) + 1;
				}
				trace._stack = trace._stack || [];
				trace._stack.push(skillName);
				try {
					return parsed.apply(this, args);
				} finally {
					trace._stack.pop();
				}
			};
			run.__lxHooked = true;
			this.content = run;
		}
		return ret;
	};
	wrapped.__lx = true;
	eventProto.setContent = wrapped;
	return true;
}

function hookPlayerCommands(lib, trace) {
	const proto = lib.element && lib.element.player;
	if (!proto || proto.__lxCmd) return;
	proto.__lxCmd = true;
	for (const cmd of COMMANDS) {
		const orig = proto[cmd];
		if (typeof orig !== "function" || orig.__lxCmd) continue;
		const wrapped = function (...args) {
			const stack = trace._stack || [];
			if (stack.length) {
				trace.commands = trace.commands || [];
				if (trace.commands.length < 400) {
					trace.commands.push({
						skill: stack[stack.length - 1],
						cmd,
						player: this?.name || this?.name1 || null,
						rngDraws: trace.rngDraws ? trace.rngDraws() : null,
					});
				}
			}
			return orig.apply(this, args);
		};
		wrapped.__lxCmd = true;
		proto[cmd] = wrapped;
	}
}

const lines = [];
const log = msg => {
	lines.push(msg);
	console.log(msg);
};

process.on("uncaughtException", e => {
	log(`uncaughtException: ${e && (e.stack || e.message || e)}`);
});
process.on("unhandledRejection", e => {
	log(`unhandledRejection: ${e && (e.stack || e.message || e)}`);
});

log(`# 龙血玄黄 / Boss 诸葛亮 / ylyg* 轨迹采样`);
log(`date: 2026-10-17`);
log(`liuli_root: ${root}`);
log(`seed: 20261016`);
log(`mode: boss`);
log(`boss: boss_zhugeliang`);
log(`fighters: mb_caomao,caocao,liubei`);

// —— 静态摘录（只读上游副本路径：scratch 或 archive 旁路均可；此处用 scratch noname + liuli）——
const nonameRoot = path.resolve(root, "../noname");
const staticTargets = [
	{
		label: "boss.js Zhuge skills",
		file: path.join(nonameRoot, "apps/core/mode/boss.js"),
		names: ["xiangxing", "yueyin", "fengqi", "gaiming"],
	},
	{
		label: "offline_yanling ylyg*",
		file: path.join(nonameRoot, "apps/core/character/offline/skill/offline_yanling.js"),
		names: [
			"ylygxiangxing",
			"ylyglianhuan",
			"ylygniepan",
			"ylyg_jiang",
			"ylygguhuo",
			"ylygtianxiang",
		],
	},
];

const staticReport = staticTargets.map(t => ({ ...t, ...extractStaticTriggers(t.file, t.names) }));
fs.writeFileSync(path.join(outDir, "static-trigger-map.json"), JSON.stringify(staticReport, null, 2));
log(`static_map: ${path.join(outDir, "static-trigger-map.json")} (${staticReport.reduce((n, x) => n + x.hits.length, 0)} hits)`);

// —— 动态：挑战模式，开局前写入 BOSS 与三名参战角色，自动选将走「再战」同一条 init 路径 ——
const runtimeTrace = [];
runtimeTrace.commands = [];
let boot;
let hookedBeforePlayers = false;
try {
	boot = bootLiuli({
		root,
		seed: 20261016,
		enableExtensions: ["龙血玄黄"],
		config: {
			mode: "boss",
			extension_龙血玄黄_enable: true,
			extension_龙血玄黄_shengjian: true,
			// 与 game.js「再战」保存的 continue_name_boss 同形：三名参战角色，玩家侧。
			continue_name_boss: { boss: false, links: ["mb_caomao", "caocao", "liubei"] },
			auto_confirm: true,
		},
		localStorageExtra: {
			"noname_0.9_nodb": "true",
			"noname_0.9_boss": { current: "boss_zhugeliang" },
			"noname_0.9_directstart": "true",
			extension_龙血玄黄_shengjian: "true",
		},
		beforeScripts(window) {
			runtimeTrace.rngDraws = () => boot?.rng?.state?.draws ?? null;
			const timer = window.setInterval(() => {
				const lib = window.lib;
				if (!lib) return;
				if (hookSetContent(lib, runtimeTrace)) hookedBeforePlayers = !window.game?.players?.length;
				hookPlayerCommands(lib, runtimeTrace);
			}, 15);
			window.__lxHookTimer = timer;
		},
		onConsole(level, line) {
			if (level === "error") runtimeTrace.push({ kind: "console.error", line: line.slice(0, 400) });
		},
	});

	const ok = await boot.waitLib(45000);
	log(`lib_ready: ${ok}`);
	const w = boot.window;
	const lib = w.lib;
	if (!lib) {
		log("FAIL: lib missing");
	} else {
		hookSetContent(lib, runtimeTrace);
		hookPlayerCommands(lib, runtimeTrace);
		const names = Object.keys(lib.skill || {}).filter(k => INTEREST.test(k));
		const summaries = names.map(n => summarizeSkill(n, lib.skill[n]));
		fs.writeFileSync(path.join(outDir, "runtime-skill-summaries.json"), JSON.stringify(summaries, null, 2));
		log(`runtime_skills: ${names.length} -> runtime-skill-summaries.json`);
		log(`has_mbqianlong: ${!!lib.skill.mbqianlong}`);
		log(`has_xiangxing: ${!!lib.skill.xiangxing}`);
		log(`has_fengqi: ${!!lib.skill.fengqi}`);
		log(`has_gaiming: ${!!lib.skill.gaiming}`);
		log(`has_yueyin: ${!!lib.skill.yueyin}`);
		log(`ylyg_in_lib: ${names.filter(n => n.startsWith("ylyg")).length}`);
		log(`has_character_mb_caomao: ${!!lib.character?.mb_caomao}`);
		log(`has_character_boss_zhugeliang: ${!!lib.character?.boss_zhugeliang}`);
		log(`has_character_ylyg_pangtong: ${!!lib.character?.ylyg_pangtong}`);
		log(`hooked_before_players: ${hookedBeforePlayers}`);
		log(`extensions: ${JSON.stringify(lib.config?.extensions || [])}`);
		log(`storage_current: ${lib.storage?.current}`);

		const g = boot.game();
		if (w._status) w._status.auto = true;
		const clickNamed = text => {
			const nodes = w.document.querySelectorAll("div,button,span");
			for (const node of nodes) {
				if ((node.textContent || "").trim() !== text) continue;
				if (node.childElementCount > 2) continue;
				try {
					node.click();
				} catch (e) {
					runtimeTrace.push({ kind: "click.error", text, message: String(e.message || e).slice(0, 160) });
				}
			}
		};
		let midRestore = null;
		for (let i = 0; i < 80; i++) {
			try {
				hookSetContent(lib, runtimeTrace);
				if (w._status) w._status.auto = true;
				clickNamed("确定");
				if (g && typeof g.resume === "function") g.resume();
			} catch (e) {
				runtimeTrace.push({ kind: "resume.error", message: String(e.message || e).slice(0, 200) });
			}
			await new Promise(r => setTimeout(r, 100));
			const hits = runtimeTrace.filter(x => x.kind === "skill.content").length;
			if (hits > 0 && !midRestore && g?.players?.length) {
				midRestore = snapshotPlayers(g, boot.rng.state.draws, w._status?.event);
				runtimeTrace.push({ kind: "mid_restore_snapshot", ...midRestore });
			}
			if (hits >= 3 && i > 15) break;
		}
		const fmtHp = n => (n === Infinity ? "Infinity" : n);
		const players = (g?.players || []).map(p => ({
			name: p.name || p.name1,
			hp: fmtHp(p.hp),
			maxHp: fmtHp(p.maxHp),
			skills: typeof p.getSkills === "function" ? p.getSkills().filter(s => INTEREST.test(s)) : [],
		}));
		log(`players: ${JSON.stringify(players)}`);
		const ev = w._status?.event;
		const contentHits = runtimeTrace.filter(x => x.kind === "skill.content");
		log(
			`after_pump: players=${g?.players?.length ?? "-"} event=${ev ? ev.name + ":" + ev.step : "-"} paused=${w._status?.paused} content_hits=${contentHits.length} commands=${(runtimeTrace.commands || []).length} rng=${boot.rng.state.draws}`
		);
		log(`content_skills: ${[...new Set(contentHits.map(h => h.skill))].join(",")}`);
		fs.writeFileSync(path.join(outDir, "mid-restore.json"), JSON.stringify(midRestore, null, 2));
		fs.writeFileSync(path.join(outDir, "commands.json"), JSON.stringify(runtimeTrace.commands || [], null, 2));
	}
} catch (e) {
	log(`boot_error: ${e && (e.stack || e.message || e)}`);
} finally {
	if (boot) {
		if (boot.window.__lxHookTimer) boot.window.clearInterval(boot.window.__lxHookTimer);
		boot.stop();
	}
}

function snapshotPlayers(g, rngDraws, ev) {
	return {
		rngDraws,
		event: ev ? { name: ev.name, step: ev.step, player: ev.player?.name || ev.player?.name1 || null } : null,
		players: (g.players || []).map(p => ({
			name: p.name || p.name1,
			hp: jsonSafe(p.hp),
			maxHp: jsonSafe(p.maxHp),
			identity: p.identity,
			storage: jsonSafe(p.storage),
			skills: typeof p.getSkills === "function" ? p.getSkills().slice(0, 40) : [],
		})),
	};
}

function jsonSafe(value, depth = 0) {
	if (value == null) return null;
	if (typeof value === "number") {
		if (value === Infinity) return "Infinity";
		if (value === -Infinity) return "-Infinity";
		if (Number.isNaN(value)) return "NaN";
		return value;
	}
	if (depth > 3) return null;
	if (typeof value === "string" || typeof value === "boolean") return value;
	if (Array.isArray(value)) return value.slice(0, 20).map(v => jsonSafe(v, depth + 1));
	if (typeof value === "object") {
		const out = {};
		for (const [k, v] of Object.entries(value).slice(0, 30)) {
			if (typeof v === "function") continue;
			out[k] = jsonSafe(v, depth + 1);
		}
		return out;
	}
	return null;
}

fs.writeFileSync(path.join(outDir, "runtime-trace.json"), JSON.stringify(runtimeTrace, null, 2));
fs.writeFileSync(path.join(outDir, "sample-log.txt"), lines.join("\n") + "\n");
log(`wrote: ${outDir}`);
process.exit(0);
