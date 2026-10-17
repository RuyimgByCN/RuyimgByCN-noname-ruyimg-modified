// 琉璃版旧线无头启动：默认配置 + 虚拟时钟 + 等待 lib 就绪。
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { createPage } from "./env.mjs";
import { makeRecordedRandom } from "./rng.mjs";
import { installVirtualTimers } from "./vtime.mjs";
import { installConfirmPolicy, installCordovaStubs } from "./stubs.mjs";

const HERE = path.dirname(fileURLToPath(import.meta.url));
export const DEFAULT_LIULI_ROOT = path.resolve(HERE, "../../../../scratch/legacy-harness/liuli");

export function defaultLiuliConfig(overrides = {}) {
	return {
		mode: "identity",
		show_splash: "off",
		extensions: [],
		totouched: true,
		new_tutorial: true,
		guide_check: false,
		touchscreen: false,
		phonelayout: false,
		auto_confirm: true,
		...overrides,
	};
}

export function bootLiuli({
	root = DEFAULT_LIULI_ROOT,
	seed = 20261016,
	config,
	enableExtensions = [],
	disableAllExtensions = false,
	virtualTimers = true,
	onConsole,
	// 脚本执行前写入的额外 localStorage。值不是字符串时 JSON.stringify。
	// 挑战模式的 lib.storage 来自 noname_0.9_<mode>，必须在脚本前写好。
	localStorageExtra = {},
	// 页面脚本开始前调用，便于在 lib 出现的同一加载间隙挂上技能钩子。
	beforeScripts: beforeScriptsExtra,
} = {}) {
	const abs = path.resolve(root);
	const html = fs.readFileSync(path.join(abs, "index.html"), "utf8");
	const rng = makeRecordedRandom(seed);
	let vt = null;
	const cfg = defaultLiuliConfig(config);
	if (enableExtensions.length) cfg.extensions = [...new Set([...(cfg.extensions || []), ...enableExtensions])];

	const { window, log, dom } = createPage({
		root: abs,
		html,
		seedRandom: rng.random,
		onConsole,
		beforeScripts(window, pageLog) {
			if (virtualTimers) {
				vt = installVirtualTimers(window, {
					ignoreNames: ["reset"],
					onError(e) {
						pageLog.jsdomErrors.push(String(e && (e.stack || e.message || e)).slice(0, 2000));
					},
				});
			}
			window.localStorage.setItem("legacy_harness", "1");
			window.localStorage.setItem("noname_0.9_config", JSON.stringify(cfg));
			window.localStorage.setItem("First_hide_splash", "init");
			window.localStorage.setItem("gplv3_noname_alerted", "true");
			window.sessionStorage.setItem("Network", "offline");
			if (disableAllExtensions) {
				window.localStorage.setItem("noname_0.9_disable_extension", "true");
			} else {
				window.localStorage.removeItem("noname_0.9_disable_extension");
				for (const name of enableExtensions) {
					window.localStorage.setItem(`extension_${name}_enable`, "true");
				}
			}
			for (const [key, value] of Object.entries(localStorageExtra)) {
				window.localStorage.setItem(key, typeof value === "string" ? value : JSON.stringify(value));
			}
			if (typeof beforeScriptsExtra === "function") beforeScriptsExtra(window, pageLog);
		},
	});

	return {
		window,
		log,
		dom,
		rng,
		vt,
		game() {
			return window.game || window.lib?.game || null;
		},
		async waitFor(predicate, { timeoutMs = 30000, intervalMs = 100 } = {}) {
			const t0 = Date.now();
			while (Date.now() - t0 < timeoutMs) {
				if (predicate(window, this.game())) return true;
				await new Promise(r => setTimeout(r, intervalMs));
			}
			return false;
		},
		async waitLib(timeoutMs = 30000) {
			return this.waitFor(w => !!w.lib && !!w._status, { timeoutMs });
		},
		stop() {
			if (vt) vt.stop();
		},
	};
}
