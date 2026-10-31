// Actual schema5 import → native WASM admission, retaining the original live handle on errors.
import assert from "node:assert/strict";
import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { importExtensionPackageValidated, activateExtension } from "../src/engine/extensionStore.ts";
import { prepareNativeMatchContent } from "../src/engine/nativeMatchContent.ts";
import { validateNativePackage } from "../src/engine/bundledNativePackages.ts";
const evidence = process.argv[2];
const { NativeEngine, validate_pack_json } = createRequire(import.meta.url)(join(evidence, "node-wasm/engine_native_wasm.js"));
const memory = new Map();
globalThis.localStorage = { getItem:key => memory.get(key) ?? null, setItem:(key,value) => memory.set(key,String(value)), removeItem:key => memory.delete(key) };
const goodRaw = readFileSync(join(evidence,"genuine-schema5.json"),"utf8");
const good = JSON.parse(goodRaw);
let forwarded;
const validate = async text => { forwarded = text; validate_pack_json(text); };
await importExtensionPackageValidated(goodRaw, validate);
assert.equal(forwarded,goodRaw);
activateExtension(`${good.manifest.namespace}/${good.manifest.package_id}`);
const content = prepareNativeMatchContent();
const handle = NativeEngine.new_with_packs_json(3,10825n,content.packsJson);
const archive = handle.save_hex(), hash = handle.state_hash_hex(), view = handle.view_json(0);
const results = [];
async function rejectEveryEntry(name, raw, pattern, duplicate = false) {
  const stored = JSON.stringify([...memory]);
  await assert.rejects(() => importExtensionPackageValidated(raw,validate),pattern);
  if (duplicate) assert.equal(forwarded,raw,"original duplicate text reaches the actual native reader");
  assert.throws(() => validate_pack_json(raw),pattern);
  assert.throws(() => NativeEngine.new_with_packs_json(3,10825n,`[${raw}]`),pattern);
  assert.throws(() => handle.restore_hex_with_packs_json(archive,`[${raw}]`),pattern);
  assert.equal(JSON.stringify([...memory]),stored);
  assert.equal(handle.save_hex(),archive); assert.equal(handle.state_hash_hex(),hash); assert.equal(handle.view_json(0),view);
  results.push({name,importRejected:true,directWasmRejected:true,admissionRejected:true,restoreRejected:true,storeHandleCatalogArchiveUnchanged:true});
}
for (const encoded of ['"charlotte":true,"charlotte":false','"charlotte":true,"\\u0063harlotte":false']) {
  const raw = goodRaw.replace('"charlotte":false',encoded); assert.notEqual(raw,goodRaw);
  await rejectEveryEntry(encoded,raw,/duplicate field.*charlotte/,true);
}
const assetsRaw = readFileSync(join(evidence,"genuine-asset-only.json"),"utf8");
validate_pack_json(assetsRaw);
await importExtensionPackageValidated(assetsRaw,validate);
const assetsHandle = NativeEngine.new_with_packs_json(3,10826n,`[${assetsRaw}]`); assetsHandle.free();
for (const [schema,value] of [[5,true],[3,false],[3,null],[4,false],[4,null]]) {
  const partial = JSON.parse(assetsRaw); partial.manifest.content_schema_version = schema;
  partial.skill_index = [{skill_def_id:99108020,name:"repair_draw",charlotte:value}];
  await rejectEveryEntry(`omitted-modules-${schema}-${value}`,JSON.stringify(partial),/schema5|partial.*modules/);
}
for (const schema of [3,4,5]) {
  const partial = JSON.parse(assetsRaw); partial.manifest.content_schema_version = schema;partial.active_index = [];
  await rejectEveryEntry(`omitted-modules-empty-active-${schema}`,JSON.stringify(partial),/partial.*modules/);
}
const activeRaw=readFileSync(join(evidence,"active-conflict-recomputed.json"),"utf8");
assert.throws(() => validateNativePackage(JSON.parse(activeRaw)),/conflict/);
await rejectEveryEntry("active-executor-conflict-valid-recomputed-hash",activeRaw,/conflict|exactly one/);
handle.free();
const output={actualCurrentWasm:true,actualSdkStore:true,originalTextToNative:true,positiveSchema5:true,positiveOriginalAssetOnly:true,results};
writeFileSync(join(evidence,"actual-sdk-wasm-result.json"),JSON.stringify(output,null,2));
console.log(JSON.stringify(output,null,2));
