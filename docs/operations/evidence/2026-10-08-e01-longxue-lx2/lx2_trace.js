// LX.2 隔离动态采样：记录型 Proxy 桩跑 content，采集注册写入与调用面。
const fs = require('fs');
const vm = require('vm');
const SRC = process.argv[2];
const OUTDIR = process.argv[3];
let s = fs.readFileSync(SRC, 'utf8');
s = s.replace(/\bdebugger\b/g, ';');

// ---- 记录型桩 ----
const records = []; // {op:'set'|'call', path, info}
function summarize(v, depth = 0) {
  if (v === null) return 'null';
  if (v === undefined) return 'undefined';
  const t = typeof v;
  if (t === 'string') return v.length > 80 ? `<str ${v.length}>${v.slice(0, 60)}…` : JSON.stringify(v);
  if (t === 'number' || t === 'boolean') return String(v);
  if (t === 'function') return `<fn${v.length ? `~${v.length}args` : ''}>`;
  if (Array.isArray(v)) return `<arr ${v.length}>`;
  if (t === 'object') {
    const keys = Object.keys(v);
    return `<obj {${keys.slice(0, 8).join(',')}${keys.length > 8 ? ',…' : ''}}>`;
  }
  return t;
}
function makeRec(base, path, depth) {
  const target = base || {};
  return new Proxy(target, {
    get(t, k) {
      if (typeof k === 'symbol') return undefined;
      if (k === 'then' || k === 'toJSON') return undefined;
      if (!(k in t)) {
        // 常见方法名返回良性函数（反篡改链 lib.version.includes 等不炸）
        if (/^(includes|indexOf|startsWith|endsWith|charAt|charCodeAt|slice|split|trim|toLowerCase|toUpperCase|concat|toString|some|every|filter|map|forEach|find|findIndex|join|reverse|sort|reduce|push|pop|shift|unshift|splice|flat|at)$/.test(String(k))) {
          records.push({ op: 'call', path: `${path}.${String(k)}()`, info: '<benign stub>' });
          return () => undefined;
        }
        if (depth < 3) return makeRec(undefined, `${path}.${k}`, depth + 1);
        return undefined;
      }
      return t[k];
    },
    set(t, k, v) {
      if (typeof k !== 'symbol') {
        records.push({ op: 'set', path: `${path}.${String(k)}`, info: summarize(v) });
      }
      t[k] = v;
      return true;
    },
    has(t, k) { return k in t; },
  });
}
const lib = makeRec({}, 'lib', 0);
const ui = makeRec({}, 'ui', 0);
const get = makeRec({}, 'get', 0);
const ai = makeRec({}, 'ai', 0);
const ctx0 = { atob, console: { log: () => {}, error: () => {} },
  setTimeout: () => 0, setInterval: () => 0, clearTimeout: () => {},
  lib, ui, get, ai, _status: {},
  window: null,
  document: {
    createElement: (t) => ({ tag: t, style: {}, appendChild() {}, addEventListener() {}, classList: { add() {}, remove() {} }, dataset: {}, innerHTML: '', textContent: '', firstChild: null, parentNode: null, remove() {} }),
    getElementById: () => null, querySelector: () => null,
    body: { appendChild() {}, classList: { add() {}, remove() {} } }, head: { appendChild() {} },
    addEventListener() {},
  },
  alert: () => {} };
const ctx = vm.createContext(ctx0);
const vmGlobal = vm.runInContext('globalThis', ctx);
const _status = ctx0._status;
const game = new Proxy({}, {
  get(t, k) {
    if (typeof k === 'symbol') return undefined;
    if (k === 'import') {
      return (n, f) => { vmGlobal.__mod = f(lib, game, ui, get, ai, _status); };
    }
    records.push({ op: 'call', path: `game.${String(k)}`, info: '<getter>' });
    return (...a) => {
      records.push({ op: 'call', path: `game.${String(k)}()`, info: a.slice(0, 2).map(summarize).join(', ') });
      return undefined;
    };
  },
  set(t, k, v) { records.push({ op: 'set', path: `game.${String(k)}`, info: summarize(v) }); t[k] = v; return true; },
});
ctx0.window = vmGlobal;
vmGlobal.game = game;
vm.runInContext(s, ctx, { timeout: 20000 });
const mod = vm.runInContext('__mod', ctx);
console.log('mod keys:', Object.keys(mod));
// precontent + content
const runs = [
  ['content(config={})', '__mod.content.call(globalThis, {}, {})'],
  ['content(config={shengjian:true})', '__mod.content.call(globalThis, { shengjian: true }, {})'],
];
for (const [label, code] of runs) {
  const before = records.length;
  try {
    vm.runInContext(code, ctx, { timeout: 30000 });
    console.log(label, 'ok, records:', records.length - before);
  } catch (e) {
    console.log(label, 'threw:', e.message.slice(0, 100), '| records so far:', records.length - before);
  }
}
try {
  vm.runInContext('__mod.precontent && __mod.precontent.call(globalThis, {}, {})', ctx, { timeout: 30000 });
  console.log('precontent ok, records:', records.length - before);
} catch (e) {
  console.log('precontent threw:', e.message.slice(0, 100));
}
// 汇总输出
const lines = records.map(r => `${r.op} ${r.path} ${r.info}`);
fs.writeFileSync(`${OUTDIR}/behavior-samples.txt`, lines.join('\n') + '\n');
console.log('total records:', records.length);
// 分类统计
const byPrefix = {};
for (const r of records) {
  const p = r.path.split('.').slice(0, 2).join('.');
  byPrefix[p] = (byPrefix[p] || 0) + 1;
}
console.log('by prefix:', JSON.stringify(byPrefix, null, 0));
