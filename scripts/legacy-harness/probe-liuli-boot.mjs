#!/usr/bin/env node
// 探针：在 jsdom 中启动琉璃版旧线核心，观察启动链路走到哪一步（开发期诊断用）。
// 用法：node probe-liuli-boot.mjs [liuli 运行副本根目录] [秒数]
import path from "node:path";
import { fileURLToPath } from "node:url";
import { bootLiuli } from "./lib/boot.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(
	process.argv[2] || path.join(HERE, "../../../scratch/legacy-harness/liuli")
);
const seconds = Number(process.argv[3] || 20);

const boot = bootLiuli({
	root,
	seed: 20261016,
	disableAllExtensions: true,
	onConsole(level, line) {
		if (level === "error" || level === "warn") console.log(`[page ${level}]`, line.slice(0, 300));
	},
});

const t0 = Date.now();
const timer = setInterval(() => {
	const w = boot.window;
	const g = boot.game();
	const st = w._status;
	const ev = st && st.event;
	const vt = boot.vt;
	console.log(
		`[${((Date.now() - t0) / 1000).toFixed(1)}s] vt=${vt?.state.now ?? "-"} fired=${vt?.state.fired ?? "-"} pending=${vt ? vt.pending() : "-"} w.game=${!!w.game} lib.game=${!!w.lib?.game} lib=${!!w.lib} players=${g?.players?.length ?? "-"} event=${ev ? ev.name + ":" + ev.step : "-"} paused=${st ? st.paused : "-"} loaded=${boot.log.loaded.length} missing=${boot.log.missing.length} errors=${boot.log.jsdomErrors.length} rand=${boot.rng.state.draws}`
	);
}, 2000);

setTimeout(() => {
	clearInterval(timer);
	boot.stop();
	console.log("missing:", boot.log.missing.slice(0, 20));
	console.log(
		"jsdomErrors:",
		boot.log.jsdomErrors.slice(0, 5).map(s => s.slice(0, 800))
	);
	console.log("dialogs:", boot.log.dialogs.slice(0, 10));
	process.exit(0);
}, seconds * 1000);
