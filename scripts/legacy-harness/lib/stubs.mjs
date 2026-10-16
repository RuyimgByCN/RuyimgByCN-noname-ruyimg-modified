// 旧线无头环境额外浏览器/Cordova 桩：避免扩展装载失败路径抛错中断启动。

function fakeEntry(url) {
	const entry = {
		isFile: true,
		isDirectory: false,
		name: String(url).split("/").pop() || "entry",
		fullPath: String(url),
		nativeURL: String(url),
		filesystem: null,
		getMetadata(ok) {
			if (ok) ok({ size: 0, modificationTime: new Date() });
		},
		remove(ok) {
			if (ok) ok();
		},
		createReader() {
			return {
				readEntries(ok) {
					if (ok) ok([]);
				},
			};
		},
		getDirectory(_n, _o, ok, err) {
			if (typeof err === "function") err({ code: 1 });
			else if (ok) ok(fakeEntry(_n));
		},
		getFile(_n, _o, ok, err) {
			if (typeof err === "function") err({ code: 1 });
			else if (ok) ok(fakeEntry(_n));
		},
	};
	return entry;
}

export function installCordovaStubs(window, log) {
	window.resolveLocalFileSystemURL = (url, ok, err) => {
		log.workers.push(`resolveLocalFileSystemURL:${url}`);
		// 返回空壳 entry，避免 removeExtension 等路径因缺少 Cordova API 同步抛错。
		if (typeof ok === "function") {
			try {
				ok(fakeEntry(url));
			} catch (e) {
				if (typeof err === "function") err(e);
			}
		} else if (typeof err === "function") {
			err({ code: 1, message: "harness: no cordova fs" });
		}
	};
	window.LocalFileSystem = { PERSISTENT: 0, TEMPORARY: 1 };
	window.requestFileSystem = (_t, _s, ok, err) => {
		if (typeof ok === "function") ok({ root: fakeEntry("/") });
		else if (typeof err === "function") err({ code: 1, message: "harness: no cordova fs" });
	};
	if (!window.navigator) window.navigator = {};
	window.navigator.notification = {
		confirm(msg, cb) {
			log.dialogs.push({ kind: "cordova-confirm", msg: String(msg).slice(0, 300) });
			if (typeof cb === "function") cb(1);
		},
		alert(msg, cb) {
			log.dialogs.push({ kind: "cordova-alert", msg: String(msg).slice(0, 300) });
			if (typeof cb === "function") cb();
		},
	};
}

/** confirm 策略：已知启动提示按答案表应答，其余默认 false（避免误关窗）。 */
export function installConfirmPolicy(window, log, answers = {}) {
	const defaults = {
		触屏模式: false,
		GPLv3: true,
		新手向导: false,
		...answers,
	};
	window.confirm = msg => {
		const s = String(msg);
		log.dialogs.push({ kind: "confirm", msg: s.slice(0, 300) });
		for (const [key, val] of Object.entries(defaults)) {
			if (s.includes(key)) return !!val;
		}
		return false;
	};
	window.alert = msg => {
		log.dialogs.push({ kind: "alert", msg: String(msg).slice(0, 300) });
	};
}
