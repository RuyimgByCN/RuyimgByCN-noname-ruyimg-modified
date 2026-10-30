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

test("N01 declaration reference and Zhouyu declared variants resolve actual source voices", () => {
  const n01 = catalog.characters.find((row) => row.id === "onlyOL:ol_jie_zhuran");
  assert.deepEqual(originalCharacterAudio(manifest, n01).map((row) => row.source_path), [
    "audio/skill/xindanshou1.mp3", "audio/skill/xindanshou2.mp3",
  ]);
  const zhouyu = catalog.characters.find((row) => row.id === "xianding:dc_sb_zhouyu");
  const voices = originalCharacterAudio(manifest, zhouyu);
  for (const skill of zhouyu.skills) {
    for (const index of [1, 2]) {
      assert.ok(voices.some((row) => row.source_path === `audio/skill/${skill}_dc_sb_zhouyu_shadow${index}.mp3`));
    }
  }
});

test("declared aliases isolate exact numeric skill IDs, validate backing bytes and reject duplicates", () => {
  const source = (name) => ({ kind: "audio", path: `audio/skill/${name}.mp3`, source_path: `audio/skill/${name}.mp3`, source_category: "core", size: 12, sha256: "b".repeat(64) });
  const direct = source("voice1"), numeric = source("voice21");
  const alias = { ...direct, path: "audio-declared/fixture/character/actor/skill/voice/base/0001/voice1.mp3" };
  const otherAlias = { ...numeric, path: "audio-declared/fixture/character/actor/skill/voice2/base/0001/voice21.mp3" };
  const character = { pack: "fixture", name: "actor", skills: ["voice"], resolved_skills: ["fixture:voice"] };
  assert.deepEqual(originalCharacterAudio({ assets: [direct, alias] }, character), [alias], "positive selectable fixture");
  assert.deepEqual(originalCharacterAudio({ assets: [direct, numeric, alias, otherAlias] }, character), [alias]);
  assert.deepEqual(originalCharacterAudio({ assets: [direct, alias, { ...alias, sha256: "a".repeat(64) }] }, character), []);
  assert.deepEqual(originalCharacterAudio({ assets: [direct, alias, { ...direct, sha256: "a".repeat(64) }] }, character), []);
  for (const change of [
    { kind: "portraits" }, { path: 17 }, { source_path: null }, { sha256: "bad" },
    { sha256: "a".repeat(64) }, { size: 0 }, { size: 1.5 }, { size: 13 }, { source_category: "skin" },
    { source_path: "audio/skill/voice_extra1.mp3" },
    { source_path: "audio/skill/voice1.mp3?query" }, { source_path: "audio/%2e%2e/voice1.mp3" },
    { path: "audio-declared/fixture/character/actor/skill/voice/../voice1.mp3" },
    { path: "audio-declared/fixture/character/actor/skill/voice/base/0001/voice1.mp3#fragment" },
    { path: "audio-declared/fixture/character/actor/skill/voice/base/0001/voice1\\bad.mp3" },
  ]) assert.deepEqual(originalCharacterAudio({ assets: [direct, { ...alias, ...change }] }, character), []);
  for (const change of [{ sha256: "bad" }, { sha256: null }, { size: 0 }, { size: 1.5 }, { size: NaN }]) {
    assert.deepEqual(originalCharacterAudio({ assets: [{ ...direct, ...change }, { ...alias, ...change }] }, character), [], "matching invalid fields must still reject");
  }
  assert.deepEqual(originalCharacterAudio({ assets: [direct, alias, { ...alias }] }, character), [], "duplicate canonical row also rejects identical metadata");
  assert.deepEqual(originalCharacterAudio({ assets: [direct, { ...alias, path: alias.path.replace("voice1.mp3", "other.mp3") }] }, character), [], "basename must match backing source");
  const jsSource = { ...direct, path: "audio/skill/voice1.js", source_path: "audio/skill/voice1.js" };
  const jsAlias = { ...jsSource, path: alias.path.replace(/mp3$/, "js") };
  assert.deepEqual(originalCharacterAudio({ assets: [jsSource, jsAlias] }, character), [], "unsupported type with otherwise valid binding");
  for (const change of [{ kind: "portraits" }, { source_category: "skin" }, { source_path: "audio/skill/other.mp3" }, { sha256: "a".repeat(64) }, { size: 13 }]) {
    assert.deepEqual(originalCharacterAudio({ assets: [{ ...direct, ...change }, alias] }, character), [], "invalid backing provenance");
  }
  assert.deepEqual(originalCharacterAudio({ assets: [numeric] }, character), [], "voice21 alone proves no binding to voice");
  assert.deepEqual(originalCharacterAudio({ assets: [direct, numeric, otherAlias] }, character), []);
  assert.deepEqual(originalCharacterAudio({ assets: [direct, alias] }, { ...character, skills: ["unknown"] }), []);
  assert.deepEqual(originalCharacterAudio({ assets: [direct, alias] }, { ...character, name: "other" }), []);
  assert.deepEqual(originalCharacterAudio({ assets: [] }, character), []);
});

test("B current character isolates unrelated shared-skill actors and retains declared substitute forms", () => {
  const liubei = catalog.characters.find((row) => row.id === "standard:liubei");
  const sources = originalCharacterAudio(manifest, liubei).map((row) => row.source_path);
  assert.ok(sources.includes("audio/skill/jijiang11.mp3"));
  assert.ok(!sources.some((value) => /liushan|re_liubei|sbjijiang/.test(value)), JSON.stringify(sources));
  const zhouyu = catalog.characters.find((row) => row.id === "xianding:dc_sb_zhouyu");
  assert.ok(catalog.skins.substitutes.some((row) => row.character === zhouyu.name && row.substitute === "dc_sb_zhouyu_shadow"));
  assert.equal(originalCharacterAudio(manifest, zhouyu).filter((row) => row.source_path.includes("_dc_sb_zhouyu_shadow")).length, 4);
});

test("B real tempname chooses its declared voice and missing actor bindings never borrow the skill pool", () => {
  const jianyong = catalog.characters.find((row) => row.id === "onlyOL:ol_jianyong");
  const voices = originalCharacterAudio(manifest, jianyong).filter((row) => row.source_path.includes("jyzongshi"));
  assert.deepEqual(voices.map((row) => row.source_path), ["audio/skill/jyzongshi_re_jianyong1.mp3", "audio/skill/jyzongshi_re_jianyong2.mp3"]);
  const liushan = catalog.characters.find((row) => row.id === "shenhua:liushan");
  assert.ok(!originalCharacterAudio(manifest, liushan).some((row) => row.source_path.includes("jijiang")));
  const liubei = catalog.characters.find((row) => row.id === "standard:liubei");
  const poolOnly = { assets: manifest.assets.filter((row) => !row.path.includes("/character/")) };
  assert.ok(!originalCharacterAudio(poolOnly, liubei).some((row) => row.source_path.includes("jijiang")));
});

test("cross-pack substitute declaration binds to its global exact character owner", () => {
  const character = catalog.characters.find((row) => row.id === "bingshi:pot_zhouyu");
  const voices = originalCharacterAudio(manifest, character);
  for (const source of ["audio/die/pot_zhouyu_shadow.mp3", "audio/skill/potchiyun_pot_zhouyu_shadow1.mp3"]) {
    assert.ok(manifest.assets.some((row) => row.path === source), `real source missing: ${source}`);
    assert.ok(voices.some((row) => row.source_path === source), `declared cross-pack voice missing: ${source}`);
  }
});
