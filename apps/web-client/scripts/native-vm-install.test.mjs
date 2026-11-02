// GT-014（NC02／NC05／NC09）：含 VM 模块的作者产物经本地正式导入入口完成生命周期，
// 并补齐包校验与错误分支（失败不改写已安装内容）。真实 Worker 触发与存读档见
// native-worker-smoke.mjs 的 GT-014 段；服务端同包行为见 game-server 的 gt014 测试。
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { beforeEach, test } from "node:test";
import {
  activateExtension, deactivateExtension, importExtensionPackage, importExtensionPackageValidated,
  installExtension, readExtensionStore, revokeExtension, rollbackExtension, setMatchFrozen,
  uninstallExtension, upgradeExtension,
} from "../src/engine/extensionStore.ts";
import { EXTENSION_CATALOG, extensionKey } from "../src/engine/extensionCatalog.ts";
import { validateNativePackage } from "../src/engine/bundledNativePackages.ts";
import { prepareNativeMatchContent, restoreNativeMatchContent } from "../src/engine/nativeMatchContent.ts";

const storage = new Map();
globalThis.localStorage = {
  getItem: (key) => storage.get(key) ?? null,
  setItem: (key, value) => storage.set(key, String(value)),
  removeItem: (key) => storage.delete(key),
};
const storeKey = "noname-next.extension-store.v1";
const fixture = (dir) => JSON.parse(readFileSync(new URL(`./fixtures/${dir}/package.npack.json`, import.meta.url), "utf8"));
const v1 = fixture("vm-trigger");
const v2 = fixture("vm-trigger-v2");
const vmKey = `${v1.manifest.namespace}/${v1.manifest.package_id}`;
beforeEach(() => storage.clear());

function record() {
  return readExtensionStore().installed.find((r) => r.key === vmKey);
}

/** 断言一次导入被拒绝，且存储逐字不变。 */
function rejects(input, pattern) {
  const before = storage.get(storeKey) ?? null;
  assert.throws(() => importExtensionPackage(typeof input === "string" ? input : JSON.stringify(input)), pattern);
  assert.equal(storage.get(storeKey) ?? null, before);
}

test("fixtures: v2 shares identity with v1 but has a new version, bytecode and fingerprint", () => {
  assert.ok(v1.modules.length > 0 && v2.modules.length > 0);
  assert.equal(`${v2.manifest.namespace}/${v2.manifest.package_id}`, vmKey);
  assert.equal(v1.manifest.version, "0.1.0");
  assert.equal(v2.manifest.version, "0.1.1");
  assert.equal(v1.modules[0].name, v2.modules[0].name);
  assert.notDeepEqual(v1.modules[0].code, v2.modules[0].code);
  assert.notDeepEqual(v1.manifest.logic_hash, v2.manifest.logic_hash);
  assert.equal(validateNativePackage(v1), v1);
  assert.equal(validateNativePackage(v2), v2);
});

test("formal import installs a VM package with its modules and reaches the match", () => {
  importExtensionPackage(JSON.stringify(v1));
  const rec = record();
  assert.ok(rec);
  assert.equal(rec.lifecycle, "installed");
  assert.equal(rec.version, "0.1.0");
  assert.deepEqual(rec.history, ["0.1.0"]);
  assert.equal(rec.dir, "imported");
  assert.equal(rec.namespace, v1.manifest.namespace);
  assert.equal(rec.packageId, v1.manifest.package_id);
  assert.match(rec.logicFingerprint, /^[a-f0-9]{8}$/);
  const stored = JSON.parse(rec.nativeVersions["0.1.0"]);
  assert.deepEqual(stored, v1);
  assert.ok(stored.modules.length === 1 && stored.trigger_index.length === 1);
  // 未启用不进内容锁；启用后正式开局读取的正是安装副本。
  assert.deepEqual(prepareNativeMatchContent().contentLock, []);
  activateExtension(vmKey);
  const content = prepareNativeMatchContent();
  assert.deepEqual(content.contentLock, [vmKey]);
  assert.deepEqual(JSON.parse(content.packsJson), [v1]);
  assert.ok(content.lockSeed.startsWith(`${vmKey}@0.1.0:`));
  // 同版本重复导入幂等（内容一致）。
  importExtensionPackage(JSON.stringify(v1));
  assert.equal(record().lifecycle, "active");
  assert.deepEqual(record().history, ["0.1.0"]);
});

test("import rejects malformed packages without touching installed content", () => {
  importExtensionPackage(JSON.stringify(v1));
  activateExtension(vmKey);
  const mutate = (fn) => { const bad = structuredClone(v1); fn(bad); return bad; };
  rejects("{not json", /合法 JSON/);
  rejects(mutate((p) => { delete p.manifest; }), /身份或逻辑指纹无效/);
  rejects(mutate((p) => { p.manifest.package_id = "../x"; }), /身份或逻辑指纹无效/);
  rejects(mutate((p) => { p.manifest.logic_hash = p.manifest.logic_hash.slice(1); }), /身份或逻辑指纹无效/);
  rejects(mutate((p) => { p.modules = {}; }), /VM 模块格式无效/);
  rejects(mutate((p) => { p.modules = [null]; }), /VM 模块格式无效/);
  rejects(mutate((p) => { delete p.modules[0].name; }), /模块名缺失或重复/);
  rejects(mutate((p) => { p.modules.push(structuredClone(p.modules[0])); }), /模块名缺失或重复/);
  rejects(mutate((p) => { p.modules[0].code = []; }), /字节码无效/);
  rejects(mutate((p) => { p.modules[0].code = [256]; }), /字节码无效/);
  rejects(mutate((p) => { p.modules[0].entry_pc = p.modules[0].code.length; }), /入口或预算字段无效/);
  rejects(mutate((p) => { p.modules[0].local_count = -1; }), /入口或预算字段无效/);
  rejects(mutate((p) => { p.modules[0].logic_hash = [1, 2, 3]; }), /模块指纹格式无效/);
  rejects(mutate((p) => { p.trigger_index[0].module = "ghost"; }), /不存在的 VM 模块/);
  rejects(mutate((p) => { p.trigger_index[0].entry_pc = 999; }), /触发索引入口或技能引用无效/);
  rejects(mutate((p) => { p.trigger_index[0].skill_def_id = 424242; }), /触发索引入口或技能引用无效/);
  rejects(mutate((p) => { p.skill_index.push(structuredClone(p.skill_index[0])); }), /技能索引条目无效或重复/);
  rejects(mutate((p) => { p.trigger_index = "x"; }), /触发索引格式无效/);
  // 同版本内容冲突：不覆盖已安装副本。
  rejects(mutate((p) => { p.modules[0].caps_mask += 1; }), /同版本包内容冲突/);
  assert.deepEqual(JSON.parse(record().nativeVersions["0.1.0"]), v1);
  assert.equal(record().lifecycle, "active");
});

test("import upgrades to a new version, keeps the old copy, and rolls back", () => {
  importExtensionPackage(JSON.stringify(v1));
  activateExtension(vmKey);
  const content1 = prepareNativeMatchContent();
  importExtensionPackage(JSON.stringify(v2));
  const rec = record();
  assert.equal(rec.version, "0.1.1");
  assert.equal(rec.lifecycle, "active");
  assert.deepEqual(rec.history, ["0.1.0", "0.1.1"]);
  assert.deepEqual(JSON.parse(rec.nativeVersions["0.1.0"]), v1);
  assert.deepEqual(JSON.parse(rec.nativeVersions["0.1.1"]), v2);
  const content2 = prepareNativeMatchContent();
  assert.deepEqual(JSON.parse(content2.packsJson), [v2]);
  assert.notEqual(content2.lockSeed, content1.lockSeed);
  // 升级只影响后续建局：旧局存档按其冻结副本恢复。
  assert.deepEqual(restoreNativeMatchContent(content1), content1);
  // 目录式升级对导入包不适用，错误信息指向导入新版本。
  assert.throws(() => upgradeExtension(vmKey), /导入包没有内置目录版本/);
  rollbackExtension(vmKey, "0.1.0");
  assert.equal(record().version, "0.1.0");
  assert.deepEqual(JSON.parse(prepareNativeMatchContent().packsJson), [v1]);
  assert.throws(() => rollbackExtension(vmKey, "9.9.9"), /无历史版本/);
});

test("disable, uninstall and revoke affect new matches and saved matches as the contract requires", () => {
  importExtensionPackage(JSON.stringify(v1));
  activateExtension(vmKey);
  const content = prepareNativeMatchContent();
  deactivateExtension(vmKey);
  assert.equal(record().lifecycle, "disabled");
  assert.deepEqual(prepareNativeMatchContent().contentLock, []);
  assert.deepEqual(restoreNativeMatchContent(content), content);
  activateExtension(vmKey);
  uninstallExtension(vmKey);
  assert.equal(record(), undefined);
  assert.deepEqual(prepareNativeMatchContent().contentLock, []);
  // 卸载后旧档仍按冻结副本恢复；重新导入可再次使用。
  assert.deepEqual(restoreNativeMatchContent(content), content);
  importExtensionPackage(JSON.stringify(v1));
  activateExtension(vmKey);
  revokeExtension(vmKey);
  assert.throws(() => restoreNativeMatchContent(content), /已撤销/);
  rejects(v1, /已撤销包不可重装/);
  assert.throws(() => activateExtension(vmKey), /已撤销/);
});

test("import is blocked while a match is frozen", () => {
  setMatchFrozen(true);
  rejects(v1, /对局中/);
  setMatchFrozen(false);
  importExtensionPackage(JSON.stringify(v1));
  assert.ok(record());
});

test("catalog packages without VM modules keep their manifest+asset payload", () => {
  for (const entry of EXTENSION_CATALOG) {
    const key = extensionKey(entry);
    installExtension(key);
    const rec = readExtensionStore().installed.find((r) => r.key === key);
    const payload = JSON.parse(rec.nativeVersions[entry.version]);
    assert.equal(payload.modules, undefined, key);
    assert.ok(payload.cards || payload.characters || payload.pile, key);
  }
});

// ---- GT-014 导入期深层校验（裁决第 12 项）：结构层 → deepValidate（产品路径为 Worker 内
// Rust 校验器）→ 提交；拒绝不改写存储；本地导入一律记为未签名。真实 Rust 校验器对同一
// 变异集的拒绝见 engine-native-wasm 的 gt014_import_validation_parity 与 smoke 的
// worker_validate_package_* 段。

test("validated import commits only after the deep validator passes and marks the copy unsigned", async () => {
  const seen = [];
  const snap = await importExtensionPackageValidated(JSON.stringify(v1), async (json) => { seen.push(json); });
  assert.equal(seen.length, 1, "deep validator called exactly once");
  const sent = JSON.parse(seen[0]);
  assert.deepEqual(sent.manifest, v1.manifest, "validator receives the canonical payload with its manifest");
  assert.deepEqual(sent.modules, v1.modules, "…and its VM modules (the Rust validator checks their logic_hash)");
  assert.deepEqual(sent.trigger_index, v1.trigger_index);
  const rec = snap.installed.find((r) => r.key === vmKey);
  assert.equal(rec?.signature, "unsigned");
  assert.equal(record()?.signature, "unsigned");
  // 升级到 v0.1.1 仍为未签名
  await importExtensionPackageValidated(JSON.stringify(v2), async () => {});
  assert.equal(record()?.version, "0.1.1");
  assert.equal(record()?.signature, "unsigned");
});

test("validated import aborts without touching the store when the deep validator rejects", async () => {
  importExtensionPackage(JSON.stringify(v1));
  const before = storage.get(storeKey);
  await assert.rejects(
    () => importExtensionPackageValidated(JSON.stringify(v2), async () => {
      throw new Error("npack: module vm_fixture_phase_end_phase_end: logic_hash mismatch");
    }),
    /logic_hash mismatch/,
  );
  assert.equal(storage.get(storeKey), before, "store byte-identical after a deep rejection");
  assert.equal(record()?.version, "0.1.0");
  assert.deepEqual(Object.keys(record()?.nativeVersions ?? {}), ["0.1.0"], "rejected version leaves no copy");
});

test("validated import never calls the deep validator when the structural layer or preconditions reject", async () => {
  let calls = 0;
  const deep = async () => { calls += 1; };
  await assert.rejects(() => importExtensionPackageValidated("{not json", deep), /不是合法 JSON/);
  const corrupt = structuredClone(v1);
  corrupt.trigger_index[0].module = "ghost";
  await assert.rejects(() => importExtensionPackageValidated(JSON.stringify(corrupt), deep), /不存在的 VM 模块/);
  // 同版本内容冲突在准备阶段即拒，不进入深层校验。
  importExtensionPackage(JSON.stringify(v1));
  const conflict = structuredClone(v1);
  conflict.modules[0].caps_mask += 1;
  await assert.rejects(() => importExtensionPackageValidated(JSON.stringify(conflict), deep), /同版本包内容冲突/);
  // 对局冻结同样前置拒绝。
  setMatchFrozen(true);
  await assert.rejects(() => importExtensionPackageValidated(JSON.stringify(v2), deep), /对局中不可安装/);
  setMatchFrozen(false);
  assert.equal(calls, 0, "deep validator must not run for structurally rejected or precondition-rejected imports");
});

test("validated import re-checks preconditions at commit time (store may change during async validation)", async () => {
  await assert.rejects(
    () => importExtensionPackageValidated(JSON.stringify(v1), async () => { setMatchFrozen(true); }),
    /对局中不可安装/,
  );
  setMatchFrozen(false);
  assert.equal(record(), undefined, "nothing committed when the match froze during validation");
});

test("schema5 SDK preserves defaults and rejects downgrade presence before local-store mutation", () => {
  for (const [table, field, value] of [["skill_index", "charlotte", false], ["skill_index", "group_children", []], ["trigger_index", "force_die", false]]) {
    const p = structuredClone(v1); p[table][0][field] = value;
    for (const schema of [3, 4]) { p.manifest.content_schema_version = schema; rejects(p, /schema5/); }
    p.manifest.content_schema_version = 5;
    assert.equal(validateNativePackage(p), p);
    assert.deepEqual(p[table][0][field], value);
    p[table][0][field] = null;
    rejects(p, /schema5/);
  }
});
test("schema5 SDK native binding has one authority and defers real factory capabilities to Rust", () => {
  const p = structuredClone(v1); p.manifest.content_schema_version = 5;
  p.trigger_index[0].host_key = "host:sanjian_weapon";
  assert.throws(() => validateNativePackage(p), /conflict/);
  p.trigger_index[0].module = ""; p.trigger_index[0].entry_pc = 0;
  assert.equal(validateNativePackage(p), p);
});

test("repair: omitted modules cannot discard declarations in validated import", async () => {
  const manifest = structuredClone(v1.manifest);
  for (const [schema, value] of [[5, true], [3, false], [4, null]]) {
    const partial = { manifest: { ...manifest, content_schema_version: schema }, characters: { generals: [] }, skill_index: [{ skill_def_id: 99108020, name: "repair_draw", charlotte: value }] };
    const before = storage.get(storeKey);
    await assert.rejects(() => importExtensionPackageValidated(JSON.stringify(partial), async () => {}), /schema5|modules|declaration/);
    assert.equal(storage.get(storeKey), before);
  }
});
test("repair: validated import delivers original text to native typed reader", async () => {
  const raw = JSON.stringify(v1, null, 2);
  await importExtensionPackageValidated(raw, async (received) => assert.equal(received, raw));
});
test("repair: active native and VM executor conflicts reject in SDK", () => {
  const p = structuredClone(v1); p.manifest.content_schema_version = 5;
  p.active_index = [{ skill_def_id: p.skill_index[0].skill_def_id, window: "play_phase", host_key: "host:sanjian_weapon", module: p.modules[0].name, entry_pc: 0 }];
  assert.throws(() => validateNativePackage(p), /conflict/);
});
