// 扩展管理面冒烟：自包含复刻 extensionStore 关键路径（不依赖 TS 解析）
const store = new Map();
globalThis.localStorage = {
  getItem: (k) => (store.has(k) ? store.get(k) : null),
  setItem: (k, v) => store.set(k, String(v)),
  removeItem: (k) => store.delete(k),
};

const STORAGE_KEY = "noname-next.extension-store.v1";
const CATALOG = [
  { namespace: "upstream-noname", packageId: "huanle-cards", version: "1.0.0", dir: "huanle", fp: "6ae1907c" },
  { namespace: "upstream-noname", packageId: "wudian", version: "1.0.0", dir: "wudian", fp: "e0d488b6" },
];
const keyOf = (e) => `${e.namespace}/${e.packageId}`;

function load() {
  const raw = localStorage.getItem(STORAGE_KEY);
  return raw ? JSON.parse(raw) : { installed: [], contentLock: [], matchFrozen: false };
}
function save(s) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(s));
}

function install(key) {
  const cat = CATALOG.find((e) => keyOf(e) === key);
  if (!cat) throw new Error("unknown");
  const s = load();
  let r = s.installed.find((x) => x.key === key);
  if (!r) {
    s.installed.push({
      key,
      version: cat.version,
      history: [cat.version],
      lifecycle: "installed",
      logicFingerprint: cat.fp,
    });
  }
  save(s);
}
function activate(key) {
  const s = load();
  if (s.matchFrozen) throw new Error("frozen");
  const r = s.installed.find((x) => x.key === key);
  if (!r || r.lifecycle === "revoked") throw new Error("bad");
  r.lifecycle = "active";
  if (!s.contentLock.includes(key)) s.contentLock.push(key);
  save(s);
}
function deactivate(key) {
  const s = load();
  if (s.matchFrozen) throw new Error("frozen");
  const r = s.installed.find((x) => x.key === key);
  r.lifecycle = "disabled";
  s.contentLock = s.contentLock.filter((k) => k !== key);
  save(s);
}
function upgrade(key, ver) {
  const s = load();
  if (s.matchFrozen) throw new Error("frozen");
  const r = s.installed.find((x) => x.key === key);
  if (!r.history.includes(r.version)) r.history.push(r.version);
  r.version = ver;
  if (!r.history.includes(ver)) r.history.push(ver);
  save(s);
}
function rollback(key, ver) {
  const s = load();
  const r = s.installed.find((x) => x.key === key);
  if (!r.history.includes(ver)) throw new Error("no hist");
  r.version = ver;
  save(s);
}
function uninstall(key) {
  const s = load();
  if (s.matchFrozen) throw new Error("frozen");
  s.contentLock = s.contentLock.filter((k) => k !== key);
  s.installed = s.installed.filter((x) => x.key !== key);
  save(s);
}
function revoke(key) {
  const s = load();
  if (s.matchFrozen) throw new Error("frozen");
  const r = s.installed.find((x) => x.key === key);
  r.lifecycle = "revoked";
  s.contentLock = s.contentLock.filter((k) => k !== key);
  save(s);
}

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok });
  console.error(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " | " + detail : ""}`);
};

const k = keyOf(CATALOG[0]);
install(k);
activate(k);
check("install_activate", load().contentLock.includes(k));
upgrade(k, "1.0.1");
check("upgrade", load().installed[0].version === "1.0.1" && load().installed[0].history.includes("1.0.0"));
rollback(k, "1.0.0");
check("rollback", load().installed[0].version === "1.0.0");
deactivate(k);
check("deactivate", !load().contentLock.includes(k));
activate(k);
load().matchFrozen = true;
save({ ...load(), matchFrozen: true });
let threw = false;
try {
  uninstall(k);
} catch {
  threw = true;
}
check("frozen_blocks", threw);
save({ ...load(), matchFrozen: false });
revoke(k);
threw = false;
try {
  activate(k);
} catch {
  threw = true;
}
check("revoke_blocks", threw);
uninstall(k);
check("uninstall", load().installed.length === 0);

// 转换包目录文件面：原始 15 个 G1 转换包必须在场；包总数随逐来源验收增长
// （2026-10-17 起不再断言精确 15）。
import { readdirSync, existsSync } from "node:fs";
import { join, dirname } from "node:path";
import { fileURLToPath } from "node:url";
const ext = join(dirname(fileURLToPath(import.meta.url)), "../../../packages/extensions");
const dirs = readdirSync(ext).filter((d) => existsSync(join(ext, d, "package.npack.json")));
const g1 = [
  "huanle-upstream-noname",
  "cardpile-upstream-noname",
  "cardpile-upstream-liuli",
  "wudian-upstream-noname",
  "shahai-gujian-upstream-noname",
  "shahai-gwent-upstream-noname",
  "shahai-hearth-upstream-noname",
  "shahai-yunchou-upstream-noname",
  "shahai-swd-upstream-noname",
  "shahai-mtg-upstream-noname",
  "shahai-zhenfa-upstream-noname",
  "wuxing-upstream-liuli",
  "d3-upstream-noname",
  "shaai-ow-upstream-noname",
  "shaai-xianjian-upstream-noname",
];
const missing = g1.filter((d) => !dirs.includes(d));
check("catalog_files_g1", dirs.length >= g1.length && missing.length === 0, `n=${dirs.length} missing=${missing.join(",")}`);

console.log(JSON.stringify({ total: results.length, failed: results.filter((r) => !r.ok).length }));
process.exit(results.some((r) => !r.ok) ? 1 : 0);
