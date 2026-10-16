// 虚拟时钟：把页面内 setTimeout/setInterval/requestAnimationFrame 换成按“到期虚拟时刻 + 注册序”
// 排序的离散事件队列，每次真实宏任务只推进一个定时器（让 jsdom 的脚本加载等真实 I/O 与 Promise 微任务插入）。
// 同种子同输入时，对局阶段的定时器触发顺序与虚拟时刻都可复现；旧线 game.delay 不再真实等待。
export function installVirtualTimers(window, options = {}) {
	const ignoreNames = new Set(options.ignoreNames || []);
	const state = { now: 0, seq: 0, fired: 0, ignored: 0 };
	const heap = [];
	const live = new Map();
	const less = (a, b) => (a.t === b.t ? a.id < b.id : a.t < b.t);
	const push = item => {
		heap.push(item);
		let i = heap.length - 1;
		while (i > 0) {
			const p = (i - 1) >> 1;
			if (!less(heap[i], heap[p])) break;
			[heap[i], heap[p]] = [heap[p], heap[i]];
			i = p;
		}
	};
	const pop = () => {
		const top = heap[0];
		const last = heap.pop();
		if (heap.length) {
			heap[0] = last;
			let i = 0;
			for (;;) {
				const l = 2 * i + 1;
				const r = l + 1;
				let m = i;
				if (l < heap.length && less(heap[l], heap[m])) m = l;
				if (r < heap.length && less(heap[r], heap[m])) m = r;
				if (m === i) break;
				[heap[i], heap[m]] = [heap[m], heap[i]];
				i = m;
			}
		}
		return top;
	};
	let pumping = false;
	let stopped = false;
	const errors = [];
	const schedulePump = () => {
		if (pumping || stopped) return;
		pumping = true;
		setImmediate(pump);
	};
	function pump() {
		pumping = false;
		if (stopped) return;
		while (heap.length && heap[0].cancelled) pop();
		if (!heap.length) return;
		const item = pop();
		live.delete(item.id);
		state.now = Math.max(state.now, item.t);
		state.fired++;
		if (item.interval != null) {
			item.t = state.now + item.interval;
			item.id = ++state.seq;
			live.set(item.handle, item);
			push(item);
		}
		try {
			item.fn.apply(window, item.args);
		} catch (e) {
			errors.push(e);
			if (options.onError) options.onError(e);
		}
		schedulePump();
	}
	const toFn = fn => (typeof fn === "function" ? fn : () => window.eval(String(fn)));
	function setTimeoutV(fn, delay, ...args) {
		const f = toFn(fn);
		if (f.name && ignoreNames.has(f.name)) {
			state.ignored++;
			return 0;
		}
		const id = ++state.seq;
		const item = { t: state.now + Math.max(0, Number(delay) || 0), id, handle: id, fn: f, args, interval: null };
		live.set(id, item);
		push(item);
		schedulePump();
		return id;
	}
	function setIntervalV(fn, delay, ...args) {
		const f = toFn(fn);
		const id = ++state.seq;
		const interval = Math.max(1, Number(delay) || 0);
		const item = { t: state.now + interval, id, handle: id, fn: f, args, interval };
		live.set(id, item);
		push(item);
		schedulePump();
		return id;
	}
	function clear(handle) {
		const item = live.get(handle);
		if (item) {
			item.cancelled = true;
			live.delete(handle);
		}
	}
	window.setTimeout = setTimeoutV;
	window.setInterval = setIntervalV;
	window.clearTimeout = clear;
	window.clearInterval = clear;
	window.requestAnimationFrame = fn => setTimeoutV(() => fn(state.now), 16);
	window.cancelAnimationFrame = clear;
	return {
		state,
		errors,
		pending: () => heap.filter(i => !i.cancelled).length,
		stop: () => {
			stopped = true;
		},
	};
}
