// 可复现随机流：mulberry32。旧线 Math.random 被替换为同种子同序列，
// 采样记录每次取数的序号，便于对照“随机流”。
export function mulberry32(seed) {
	let a = seed >>> 0;
	return function next() {
		a = (a + 0x6d2b79f5) >>> 0;
		let t = a;
		t = Math.imul(t ^ (t >>> 15), t | 1);
		t ^= t + Math.imul(t ^ (t >>> 7), t | 61);
		return ((t ^ (t >>> 14)) >>> 0) / 4294967296;
	};
}

export function makeRecordedRandom(seed) {
	const next = mulberry32(seed);
	const state = { draws: 0 };
	const random = () => {
		state.draws++;
		return next();
	};
	return { random, state };
}
