import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { originalCharacterAudio, originalResourceUrl } from "../src/engine/originalResources.ts";

const root = new URL("../../../", import.meta.url);
const catalog = JSON.parse(readFileSync(new URL("content/original/catalog.json", root)));
const manifest = JSON.parse(readFileSync(new URL("content/original/asset-manifest.json", root)));
const base = "http://127.0.0.1:7700/original-content";

test("UP37 available voices resolve from the existing manifest and actual catalog identities", () => {
  const cases = [
    ["sp2:star_dongyun", "starbishi1.mp3"],
    ["onlyOL:ol_sb_zhouyu", "olsbguqu1.mp3"],
    ["tw:huan_zhouyu", "twdumou1.mp3"],
    ["xianding:dc_sb_zhouyu", "dcsbronghuo1.mp3"],
    ["collab:ol_chitu", "olfangma1.mp3"],
    ["collab:ol_le_menghuo", "oljunzhu1.mp3"],
  ];
  for (const [id, expected] of cases) {
    const character = catalog.characters.find((row) => row.id === id);
    const voices = originalCharacterAudio(manifest, character);
    assert.ok(voices.some((voice) => voice.path.endsWith(expected)), id);
    assert.ok(voices.some((voice) => voice.source_path === `audio/die/${character.name}.mp3`), id);
    for (const voice of voices) {
      assert.equal(originalResourceUrl(base, `assets/${voice.path}`), `${base}/assets/${voice.path}`);
    }
  }
  const zhangliang = catalog.characters.find((row) => row.id === "collab:tw_dm_zhangliang");
  const inherited = originalCharacterAudio(manifest, zhangliang);
  assert.equal(inherited.length, 4);
  assert.ok(inherited.some((voice) => voice.path.endsWith("twjianming1.mp3")));
  assert.ok(inherited.some((voice) => voice.path.endsWith("relonghun1.mp3")));
  assert.equal(zhangliang.migration_status, "source_synced");
  const tangtang = catalog.characters.find((row) => row.id === "sp:tangtang");
  assert.deepEqual(originalCharacterAudio(manifest, tangtang), []);
  assert.equal(tangtang.migration_status, "source_synced");
});

test("resource URL rejects traversal, absolute paths, encoded separators and invalid segments", () => {
  for (const value of ["../secret", "/assets/a.mp3", "https://evil/a", "assets/../a", "assets/%2e%2e/a", "assets/a\\b", "assets/a?x", "assets//a", "assets/./a", "assets/a#x"]) {
    assert.equal(originalResourceUrl(base, value), "", value);
  }
  assert.equal(originalResourceUrl(base, null), "");
  assert.equal(originalResourceUrl(base, "assets/portraits/tangtang.png"), `${base}/assets/portraits/tangtang.png`);
});

test("audio lookup requires exact identity, valid provenance and a supported audio file", () => {
  const character = { name: "star_dongyun", skills: ["starbishi"] };
  const entry = manifest.assets.find((row) => row.source_path === "audio/skill/starbishi1.mp3");
  assert.ok(entry, "available upstream voice missing from manifest");
  for (const change of [
    { path: "audio/../secret.mp3" }, { source_path: "audio/skill/starbishi_extra1.mp3" },
    { source_category: "skin" }, { sha256: "bad" }, { size: 0 },
    { path: "audio/skill/starbishi1.js", source_path: "audio/skill/starbishi1.js" },
  ]) {
    assert.deepEqual(originalCharacterAudio({ assets: [{ ...entry, ...change }] }, character), []);
  }
  assert.deepEqual(originalCharacterAudio({ assets: [entry] }, { name: "other", skills: ["star"] }), []);
  assert.deepEqual(originalCharacterAudio({ assets: [] }, character), []);
});
