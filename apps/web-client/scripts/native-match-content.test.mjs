import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import {
  activateExtension, deactivateExtension, installExtension, installAndActivateAll,
  readExtensionStore, revokeExtension, rollbackExtension, setMatchFrozen,
  uninstallExtension, upgradeExtension, putNativeSave, listNativeSaves,
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
const key = extensionKey(EXTENSION_CATALOG.find((entry) => entry.dir === "huanle-upstream-noname"));
const storeKey = "noname-next.extension-store.v1";
beforeEach(() => storage.clear());

function installActive() {
  installExtension(key);
  activateExtension(key);
  return prepareNativeMatchContent();
}

test("all discoverable packages install actual versioned data; active set reaches the match", () => {
  installAndActivateAll();
  const content = prepareNativeMatchContent();
  const packs = JSON.parse(content.packsJson);
  assert.equal(packs.length, EXTENSION_CATALOG.length);
  assert.ok(packs.some((p) => p.cards));
  assert.ok(packs.some((p) => p.characters));
  assert.ok(packs.some((p) => p.pile));
  assert.ok(packs.every((p) => p.manifest.logic_hash.length === 32));
  deactivateExtension(key);
  assert.ok(!prepareNativeMatchContent().contentLock.includes(key));
});

test("same-version installation/upgrade preserves installed bytes and refuses invented versions", () => {
  const content = installActive();
  installExtension(key);
  upgradeExtension(key);
  assert.deepEqual(prepareNativeMatchContent(), content);
  const before = storage.get(storeKey);
  assert.throws(() => upgradeExtension(key, "99.0.0"), /无可安装/);
  assert.equal(storage.get(storeKey), before);
  rollbackExtension(key, "1.0.0");
  assert.deepEqual(prepareNativeMatchContent(), content);
  const snap = readExtensionStore();
  const corrupt = JSON.parse(snap.installed[0].nativeVersions["1.0.0"]);
  corrupt.manifest.namespace = "wrong-source";
  snap.installed[0].nativeVersions["1.0.0"] = JSON.stringify(corrupt);
  storage.set(storeKey, JSON.stringify(snap));
  const corruptBefore = storage.get(storeKey);
  assert.throws(() => rollbackExtension(key, "1.0.0"), /身份或版本不匹配/);
  assert.equal(storage.get(storeKey), corruptBefore);
});

test("all lifecycle content mutations are blocked during a match", () => {
  installActive();
  setMatchFrozen(true);
  const before = storage.get(storeKey);
  for (const mutate of [
    () => installExtension(key), () => activateExtension(key), () => deactivateExtension(key),
    () => upgradeExtension(key), () => rollbackExtension(key, "1.0.0"),
    () => uninstallExtension(key), () => revokeExtension(key),
  ]) assert.throws(mutate, /对局中/);
  assert.equal(storage.get(storeKey), before);
});

test("save retains original package copy across disable, uninstall and later saves", () => {
  const content = installActive();
  const save = { id: "one", hex: "aa", ...content };
  putNativeSave(save);
  deactivateExtension(key);
  assert.deepEqual(restoreNativeMatchContent(listNativeSaves()[0]), content);
  uninstallExtension(key);
  const restored = restoreNativeMatchContent(save);
  assert.deepEqual(restored, content);
  putNativeSave({ ...save, id: "two", ...restored });
  assert.deepEqual(restoreNativeMatchContent(listNativeSaves()[0]), content);
  assert.equal(prepareNativeMatchContent().packsJson, undefined);
});

test("revocation rejects a saved package even when its original bytes remain", () => {
  const content = installActive();
  revokeExtension(key);
  assert.throws(() => restoreNativeMatchContent(content), /已撤销/);
  assert.throws(() => rollbackExtension(key, "1.0.0"), /已撤销/);
  uninstallExtension(key);
  assert.throws(() => restoreNativeMatchContent(content), /已撤销/);
  assert.throws(() => installExtension(key), /已撤销/);
});

test("missing installed payload, dangling lock and duplicate lock fail without fallback", () => {
  installActive();
  const snap = readExtensionStore();
  delete snap.installed[0].nativeVersions;
  assert.throws(() => prepareNativeMatchContent(snap), /缺少已安装包内容/);
  snap.installed = [];
  assert.throws(() => prepareNativeMatchContent(snap), /未启用/);
  const duplicate = readExtensionStore();
  duplicate.contentLock.push(key);
  assert.throws(() => prepareNativeMatchContent(duplicate), /重复/);
});

test("legacy metadata-only installs are repaired explicitly by upgrade", () => {
  const content = installActive();
  const snap = readExtensionStore();
  delete snap.installed[0].nativeVersions;
  storage.set(storeKey, JSON.stringify(snap));
  upgradeExtension(key);
  assert.deepEqual(prepareNativeMatchContent(), content);
});

test("changed version, fingerprint, missing package and malformed payload reject restore", () => {
  const content = installActive();
  const packs = JSON.parse(content.packsJson);
  for (const change of [
    (p) => { p[0].manifest.version = "1.0.1"; },
    (p) => { p[0].manifest.logic_hash[31] ^= 1; },
    (p) => { p.length = 0; },
  ]) {
    const altered = structuredClone(packs);
    change(altered);
    assert.throws(() => restoreNativeMatchContent({ ...content, packsJson: JSON.stringify(altered) }), /内容锁/);
  }
  assert.throws(() => restoreNativeMatchContent({ ...content, packsJson: "{}" }), /数组/);
  assert.throws(() => restoreNativeMatchContent({ ...content, packsJson: undefined }), /缺少实际包内容/);
  assert.throws(() => restoreNativeMatchContent({ ...content, packsJson: '[{"manifest":{}}]' }), /无效/);
});

test("old standard saves without packages remain compatible", () => {
  assert.deepEqual(restoreNativeMatchContent({ contentLock: [], lockSeed: "" }), {
    contentLock: [], lockSeed: "", packsJson: undefined,
  });
});

for (const [changed, change] of [
  ["logic fingerprint", (p) => { p.manifest.logic_hash[0] ^= 1; }],
  ["asset content", (p) => { p.cards.cards[0].name += "-changed"; }],
]) {
  for (const [name, action] of [
    ["installation", () => installExtension(key)],
    ["upgrade", () => upgradeExtension(key)],
  ]) test(`same-version ${changed} conflict rejects ${name} atomically`, () => {
    storage.clear();
    installActive();
    const snap = readExtensionStore();
    const record = snap.installed[0];
    const old = JSON.parse(record.nativeVersions[record.version]);
    change(old);
    record.nativeVersions[record.version] = JSON.stringify(old);
    record.logicFingerprint = old.manifest.logic_hash.slice(0, 4)
      .map((n) => n.toString(16).padStart(2, "0")).join("");
    storage.set(storeKey, JSON.stringify(snap));
    const before = storage.get(storeKey);
    const original = prepareNativeMatchContent();
    assert.throws(action, /同版本.*内容|版本.*冲突/);
    assert.equal(storage.get(storeKey), before);
    assert.deepEqual(prepareNativeMatchContent(), original);
  });
}

test("same-version copies tolerate JSON formatting and object key order", () => {
  const content = installActive();
  const snap = readExtensionStore();
  const record = snap.installed[0];
  const old = JSON.parse(record.nativeVersions[record.version]);
  const reordered = Object.fromEntries(Object.entries(old).reverse());
  const saved = JSON.stringify(reordered, null, 2);
  record.nativeVersions[record.version] = saved;
  storage.set(storeKey, JSON.stringify(snap));
  installExtension(key);
  upgradeExtension(key);
  assert.equal(readExtensionStore().installed[0].nativeVersions[record.version], saved);
  assert.deepEqual(JSON.parse(prepareNativeMatchContent().packsJson), JSON.parse(content.packsJson));
});

test("unsafe identity fields cannot enter the browser content lock", () => {
  const content = installActive();
  const original = JSON.parse(content.packsJson)[0];
  const accepted = [];
  for (const field of ["namespace", "package_id", "version"]) {
    for (const bad of [".", "..", "name/child", "name\\child", "   "]) {
      const pack = structuredClone(original);
      pack.manifest[field] = bad;
      try {
        validateNativePackage(pack);
        accepted.push(`${field}=${JSON.stringify(bad)}`);
      } catch (error) {
        assert.match(String(error), /身份|无效/);
      }
    }
  }
  assert.deepEqual(accepted, []);
});

test("package dependencies are checked for match start and frozen save restore", () => {
  const baseKey = key;
  const dependentKey = extensionKey(EXTENSION_CATALOG.find((entry) => entry.dir === "d3-upstream-noname"));
  for (const name of [baseKey, dependentKey]) { installExtension(name); activateExtension(name); }
  const original = readExtensionStore();
  const baseRecord = original.installed.find((record) => record.key === baseKey);
  const depRecord = original.installed.find((record) => record.key === dependentKey);
  const base = JSON.parse(baseRecord.nativeVersions[baseRecord.version]);
  const dependent = JSON.parse(depRecord.nativeVersions[depRecord.version]);
  const required = { namespace: base.manifest.namespace, package_id: base.manifest.package_id,
    version: base.manifest.version, logic_hash: base.manifest.logic_hash };
  dependent.manifest.dependencies = [required];

  const storeFor = (packs) => ({
    installed: packs.map((pack) => {
      const m = pack.manifest;
      return { key: `${m.namespace}/${m.package_id}`, version: m.version, lifecycle: "active",
        logicFingerprint: m.logic_hash.slice(0, 4).map((n) => n.toString(16).padStart(2, "0")).join(""),
        nativeVersions: { [m.version]: JSON.stringify(pack) } };
    }),
    contentLock: packs.map((pack) => `${pack.manifest.namespace}/${pack.manifest.package_id}`),
    matchFrozen: false,
  });
  const saveFor = (packs) => ({
    packsJson: JSON.stringify(packs),
    contentLock: packs.map((p) => `${p.manifest.namespace}/${p.manifest.package_id}`),
    lockSeed: packs.map((p) => `${p.manifest.namespace}/${p.manifest.package_id}@${p.manifest.version}:${p.manifest.logic_hash.map((n) => n.toString(16).padStart(2, "0")).join("")}`).join("|"),
  });
  const valid = [dependent, base];
  assert.deepEqual(prepareNativeMatchContent(storeFor(valid)), saveFor(valid));
  assert.deepEqual(restoreNativeMatchContent(saveFor(valid), { installed: [], contentLock: [], matchFrozen: false }), saveFor(valid));
  for (const hash of [undefined, Array(32).fill(0)]) {
    const wild = structuredClone(dependent);
    if (hash === undefined) delete wild.manifest.dependencies[0].logic_hash;
    else wild.manifest.dependencies[0].logic_hash = hash;
    assert.deepEqual(prepareNativeMatchContent(storeFor([wild, base])), saveFor([wild, base]));
  }
  const cycleBase = structuredClone(base);
  cycleBase.manifest.dependencies = [{ namespace: dependent.manifest.namespace,
    package_id: dependent.manifest.package_id, version: dependent.manifest.version }];
  assert.deepEqual(prepareNativeMatchContent(storeFor([dependent, cycleBase])), saveFor([dependent, cycleBase]));
  const cases = [["missing", [dependent]]];
  for (const [label, field, value] of [
    ["namespace", "namespace", "different"], ["version", "version", "wrong"],
    ["hash", "logic_hash", Array(32).fill(17)], ["bad hash", "logic_hash", [1, 2]],
    ["bad id", "package_id", "../bad"],
  ]) {
    const bad = structuredClone(dependent); bad.manifest.dependencies[0][field] = value;
    cases.push([label, [bad, base]]);
  }
  const duplicate = structuredClone(dependent); duplicate.manifest.dependencies.push(required);
  cases.push(["duplicate", [duplicate, base]]);
  const self = structuredClone(dependent);
  Object.assign(self.manifest.dependencies[0], { namespace: dependent.manifest.namespace, package_id: dependent.manifest.package_id });
  cases.push(["self", [self, base]]);
  const invalid = structuredClone(dependent); invalid.manifest.dependencies = null;
  cases.push(["not an array", [invalid, base]]);
  const accepted = [];
  for (const [label, packs] of cases) for (const [entry, run] of [
    ["start", () => prepareNativeMatchContent(storeFor(packs))],
    ["restore", () => restoreNativeMatchContent(saveFor(packs), { installed: [], contentLock: [], matchFrozen: false })],
  ]) {
    try { run(); accepted.push(`${label}/${entry}`); }
    catch (error) { assert.match(String(error), /依赖/, label); }
  }
  assert.deepEqual(accepted, []);
});
