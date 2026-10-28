import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { validateNativePackage } from "../src/engine/bundledNativePackages.ts";
import { prepareNativeMatchContent, restoreNativeMatchContent } from "../src/engine/nativeMatchContent.ts";

const vmPackage = JSON.parse(readFileSync(new URL("./fixtures/vm-trigger/package.npack.json", import.meta.url)));
const emptyPackage = JSON.parse(readFileSync(new URL("../../../packages/extensions/huanle-upstream-noname/package.npack.json", import.meta.url)));

function activeStore(pack) {
  const manifest = pack.manifest;
  const key = `${manifest.namespace}/${manifest.package_id}`;
  return {
    installed: [{ key, version: manifest.version, lifecycle: "active",
      logicFingerprint: manifest.logic_hash.slice(0, 4).map((n) => n.toString(16).padStart(2, "0")).join(""),
      nativeVersions: { [manifest.version]: JSON.stringify(pack) } }],
    contentLock: [key], matchFrozen: false,
  };
}

test("complete VM-only package survives match preparation and frozen save restoration", () => {
  assert.ok(vmPackage.modules.length > 0);
  assert.ok(!vmPackage.cards && !vmPackage.characters && !vmPackage.pile);
  const store = activeStore(vmPackage);
  const content = prepareNativeMatchContent(store);
  assert.deepEqual(JSON.parse(content.packsJson), [vmPackage]);
  assert.deepEqual(restoreNativeMatchContent(content, { installed: [], contentLock: [], matchFrozen: false }), content);
  assert.throws(() => restoreNativeMatchContent(content, { ...store, revokedKeys: store.contentLock }), /已撤销/);

  const changed = structuredClone(vmPackage);
  changed.manifest.logic_hash[0] ^= 1;
  assert.throws(() => restoreNativeMatchContent({ ...content, packsJson: JSON.stringify([changed]) }, store), /内容锁/);
});

test("VM payload must have an array of modules and does not excuse malformed assets", () => {
  assert.deepEqual(emptyPackage.modules, []);
  assert.equal(validateNativePackage(emptyPackage), emptyPackage);
  for (const modules of [null, {}, "bad", 1]) {
    const bad = { ...vmPackage, modules, cards: {} };
    assert.throws(() => validateNativePackage(bad), /VM|模块/);
    assert.throws(() => prepareNativeMatchContent(activeStore(bad)), /VM|模块/);
  }
  assert.throws(() => validateNativePackage({ ...vmPackage, cards: null }), /卡牌|数据/);
  const missing = structuredClone(vmPackage);
  delete missing.modules;
  assert.throws(() => validateNativePackage(missing), /缺少/);
});
