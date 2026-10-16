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
	process.argv[3] || path.join(REPO, "docs/operations/evidence/2026-10-16-t9-lx-battle-trace")
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

function hookSkillTriggers(lib, trace) {
	const skill = lib.skill;
	for (const name of Object.keys(skill)) {
		if (!INTEREST.test(name)) continue;
		const sk = skill[name];
		if (!sk || typeof sk.content !== "function") continue;
		const orig = sk.content;
		if (orig.__lxHooked) continue;
		const wrapped = function (...args) {
			trace.push({
				t: Date.now(),
				kind: "skill.content",
				skill: name,
				event: this?.name,
				step: this?.step,
				player: this?.player?.name || this?.player?.playerid,
			});
			return orig.apply(this, args);
		};
		wrapped.__lxHooked = true;
		sk.content = wrapped;
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
log(`date: 2026-10-16`);
log(`liuli_root: ${root}`);
log(`seed: 20261016`);

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

// —— 动态：无头启动，启用龙血玄黄 ——
const runtimeTrace = [];
let boot;
try {
	boot = bootLiuli({
		root,
		seed: 20261016,
		enableExtensions: ["龙血玄黄"],
		config: {
			extension_龙血玄黄_enable: true,
			extension_龙血玄黄_shengjian: true,
		},
		onConsole(level, line) {
			if (level === "error") runtimeTrace.push({ kind: "console.error", line: line.slice(0, 400) });
		},
	});
	// shengjian 配置写入（扩展 config 键）
	boot.window.localStorage.setItem(
		"extension_龙血玄黄_shengjian",
		"true"
	);

	const ok = await boot.waitLib(45000);
	log(`lib_ready: ${ok}`);
	const w = boot.window;
	const lib = w.lib;
	if (!lib) {
		log("FAIL: lib missing");
	} else {
		hookSkillTriggers(lib, runtimeTrace);
		const names = Object.keys(lib.skill || {}).filter(k => INTEREST.test(k));
		const summaries = names.map(n => summarizeSkill(n, lib.skill[n]));
		fs.writeFileSync(path.join(outDir, "runtime-skill-summaries.json"), JSON.stringify(summaries, null, 2));
		log(`runtime_skills: ${names.length} -> runtime-skill-summaries.json`);
		log(`has_mbqianlong: ${!!lib.skill.mbqianlong}`);
		log(`has__CMeffects: ${!!lib.skill._CMeffects}`);
		log(`extensions: ${JSON.stringify(lib.config?.extensions || [])}`);

		// 尝试推进对局：托管 + resume
		const g = boot.game();
		if (w._status) w._status.auto = true;
		for (let i = 0; i < 40; i++) {
			try {
				if (g && typeof g.resume === "function") g.resume();
				if (g && typeof g.loop === "function" && w._status?.paused) g.resume();
			} catch (e) {
				runtimeTrace.push({ kind: "resume.error", message: String(e.message || e).slice(0, 200) });
			}
			await new Promise(r => setTimeout(r, 250));
		}
		const ev = w._status?.event;
		log(
			`after_pump: players=${g?.players?.length ?? "-"} event=${ev ? ev.name + ":" + ev.step : "-"} paused=${w._status?.paused} content_hits=${runtimeTrace.filter(x => x.kind === "skill.content").length}`
		);
	}
} catch (e) {
	log(`boot_error: ${e && (e.stack || e.message || e)}`);
} finally {
	if (boot) boot.stop();
}

fs.writeFileSync(path.join(outDir, "runtime-trace.json"), JSON.stringify(runtimeTrace, null, 2));
fs.writeFileSync(path.join(outDir, "sample-log.txt"), lines.join("\n") + "\n");
log(`wrote: ${outDir}`);
process.exit(0);
