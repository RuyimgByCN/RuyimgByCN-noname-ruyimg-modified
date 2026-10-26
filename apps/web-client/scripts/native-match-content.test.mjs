import assert from "node:assert/strict";
import { beforeEach, test } from "node:test";
import {
  activateExtension, deactivateExtension, installExtension, installAndActivateAll,
  readExtensionStore, revokeExtension, rollbackExtension, setMatchFrozen,
  uninstallExtension, upgradeExtension, putNativeSave, listNativeSaves,
} from "../src/engine/extensionStore.ts";
import { EXTENSION_CATALOG, extensionKey } from "../src/engine/extensionCatalog.ts";
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
