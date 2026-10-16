// 旧线无头运行环境：jsdom 页面 + 本地文件资源加载器 + 浏览器 API 桩。
// 上游源码只读：运行时根目录必须是 archive 之外的副本（scratch/legacy-harness/...）。
import fs from "node:fs";
import path from "node:path";
import { JSDOM, ResourceLoader, VirtualConsole } from "jsdom";
import { installConfirmPolicy, installCordovaStubs } from "./stubs.mjs";

export const HOST = "noname.local";

export function assertOutsideArchive(root) {
	const abs = path.resolve(root);
	if (abs.split(path.sep).includes("archive")) {
		throw new Error(`运行根目录不能位于 archive 下（只读上游）：${abs}`);
	}
	return abs;
}

function mapUrlToFile(root, url) {
	const u = new URL(url);
	if (u.host !== HOST) return null;
	return path.join(root, decodeURIComponent(u.pathname));
}

class LocalFileLoader extends ResourceLoader {
	constructor(root, log) {
		super();
		this.root = root;
		this.log = log;
	}
	fetch(url, options) {
		const file = mapUrlToFile(this.root, url);
		if (!file) {
			this.log.blocked.push(url);
			return null;
		}
		const promise = fs.promises.readFile(file).then(
			buf => {
				this.log.loaded.push(url);
				return buf;
			},
			err => {
				this.log.missing.push(url);
				throw err;
			}
		);
		promise.abort = () => {};
		return promise;
	}
}

function canvasStub() {
	const ctx = new Proxy(
		{ canvas: null },
		{
			get(t, k) {
				if (k in t) return t[k];
				if (k === "measureText") return () => ({ width: 0 });
				if (k === "getImageData") return () => ({ data: new Uint8ClampedArray(4) });
				if (k === "createLinearGradient" || k === "createRadialGradient" || k === "createPattern") return () => ({ addColorStop() {} });
				return () => undefined;
			},
			set(t, k, v) {
				t[k] = v;
				return true;
			},
		}
	);
	return ctx;
}

// 页面内 XHR/fetch：只读本地副本，其余域一律拒绝（无网络语义）。
function installLocalNetwork(window, root, log) {
	const readLocal = url => {
		const abs = new URL(url, window.location.href).href;
		const file = mapUrlToFile(root, abs);
		if (!file) {
			log.blocked.push(abs);
			return { ok: false, status: 0, text: "" };
		}
		try {
			const text = fs.readFileSync(file, "utf8");
			log.loaded.push(abs);
			return { ok: true, status: 200, text };
		} catch {
			log.missing.push(abs);
			return { ok: false, status: 404, text: "" };
		}
	};
	class LocalXHR {
		constructor() {
			this.readyState = 0;
			this.status = 0;
			this.responseText = "";
			this.response = null;
			this.responseType = "";
			this._listeners = {};
		}
		open(method, url) {
			this._url = url;
			this.readyState = 1;
		}
		setRequestHeader() {}
		overrideMimeType() {}
		addEventListener(type, fn) {
			(this._listeners[type] ||= []).push(fn);
		}
		removeEventListener() {}
		abort() {}
		getAllResponseHeaders() {
			return "";
		}
		getResponseHeader() {
			return null;
		}
		send() {
			const r = readLocal(this._url);
			window.setTimeout(() => {
				this.status = r.status;
				this.readyState = 4;
				this.responseText = r.text;
				if (this.responseType === "json") {
					try {
						this.response = JSON.parse(r.text);
					} catch {
						this.response = null;
					}
				} else if (this.responseType === "arraybuffer") {
					this.response = new TextEncoder().encode(r.text).buffer;
				} else {
					this.response = r.text;
				}
				const fire = type => {
					const ev = { type, target: this, currentTarget: this };
					if (typeof this["on" + type] === "function") this["on" + type](ev);
					for (const fn of this._listeners[type] || []) fn.call(this, ev);
				};
				fire("readystatechange");
				fire(r.ok ? "load" : "error");
				fire("loadend");
			}, 0);
		}
	}
	window.XMLHttpRequest = LocalXHR;
	window.fetch = async url => {
		const r = readLocal(typeof url === "string" ? url : url.url);
		return {
			ok: r.ok,
			status: r.status,
			text: async () => r.text,
			json: async () => JSON.parse(r.text),
			arrayBuffer: async () => new TextEncoder().encode(r.text).buffer,
			blob: async () => new window.Blob([r.text]),
		};
	};
}

export function installBrowserStubs(window, log) {
	const record = (kind, msg) => log.dialogs.push({ kind, msg: String(msg).slice(0, 300) });
	window.alert = msg => record("alert", msg);
	window.confirm = msg => {
		record("confirm", msg);
		return false;
	};
	window.prompt = msg => {
		record("prompt", msg);
		return null;
	};
	window.HTMLCanvasElement.prototype.getContext = function () {
		return canvasStub();
	};
	window.HTMLCanvasElement.prototype.toDataURL = () => "data:,";
	window.HTMLMediaElement.prototype.play = function () {
		return Promise.resolve();
	};
	window.HTMLMediaElement.prototype.pause = function () {};
	window.HTMLMediaElement.prototype.load = function () {};
	window.matchMedia = () => ({ matches: false, addListener() {}, removeListener() {}, addEventListener() {}, removeEventListener() {} });
	window.scrollTo = () => {};
	window.Element.prototype.scrollIntoView = function () {};
	class NoopObserver {
		observe() {}
		unobserve() {}
		disconnect() {}
	}
	window.ResizeObserver = NoopObserver;
	window.IntersectionObserver = NoopObserver;
	class FakeAudioContext {
		constructor() {
			this.destination = {};
			this.currentTime = 0;
		}
		createGain() {
			return { gain: { value: 1 }, connect() {} };
		}
		createBufferSource() {
			return { connect() {}, start() {}, stop() {} };
		}
		decodeAudioData(_buf, ok) {
			if (ok) ok({});
			return Promise.resolve({});
		}
		resume() {
			return Promise.resolve();
		}
	}
	window.AudioContext = FakeAudioContext;
	window.webkitAudioContext = FakeAudioContext;
	window.Worker = class {
		constructor(url) {
			log.workers.push(String(url));
		}
		postMessage() {}
		terminate() {}
		addEventListener() {}
	};
}

export function createPage({ root, html, seedRandom, beforeScripts, onConsole }) {
	const abs = assertOutsideArchive(root);
	/** @type {{loaded:string[],missing:string[],blocked:string[],dialogs:object[],workers:string[],console:object[],jsdomErrors:string[]}} */
	const log = { loaded: [], missing: [], blocked: [], dialogs: [], workers: [], console: [], jsdomErrors: [] };
	const virtualConsole = new VirtualConsole();
	for (const level of ["log", "info", "warn", "error"]) {
		virtualConsole.on(level, (...args) => {
			const line = args.map(a => (a instanceof Error ? a.stack : String(a))).join(" ");
			log.console.push({ level, line: line.slice(0, 2000) });
			if (onConsole) onConsole(level, line);
		});
	}
	virtualConsole.on("jsdomError", e => {
		log.jsdomErrors.push(String(e && (e.stack || e.message || e)).slice(0, 4000));
	});
	const dom = new JSDOM(html, {
		url: `http://${HOST}/index.html`,
		runScripts: "dangerously",
		resources: new LocalFileLoader(abs, log),
		pretendToBeVisual: true,
		virtualConsole,
		beforeParse(window) {
			if (seedRandom) window.Math.random = seedRandom;
			installBrowserStubs(window, log);
			installCordovaStubs(window, log);
			installConfirmPolicy(window, log);
			installLocalNetwork(window, abs, log);
			if (beforeScripts) beforeScripts(window, log);
		},
	});
	return { dom, window: dom.window, log };
}
