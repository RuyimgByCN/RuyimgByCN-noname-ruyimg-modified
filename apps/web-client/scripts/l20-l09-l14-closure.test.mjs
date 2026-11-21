import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  L20_CONFIG_KEYS,
  L20_CSS_SURFACE_IDS,
  L20_UI_HOOK_IDS,
  QHLY_CONFIG_SPECS,
  QHLY_SURFACE_PRESETS,
  QHLY_UI_HOOK_IDS,
  canTriggerQhlyVoiceWithNotbb,
  clearQhlySkinSettings,
  createDefaultShoushaConfig,
  createInitialExtensionSurfacesState,
  createInitialL20RuntimeState,
  createInitialQhlyRuntimeState,
  deserializeExtensionSurfacesSnapshot,
  dispatchL20UiHook,
  highlightSkillKeywords,
  ingestBattleEnvelopeIntoSurfaces,
  l20ArenaDatasetAttrs,
  parseQhlyConfigImportJson,
  qhlyArenaDatasetAttrs,
  resolveActiveShoushaSurfaces,
  resolveQhlyCharacterSubtitle,
  restoreOfficialSkins,
  serializeExtensionSurfacesSnapshot,
  syncL20MarkEffectsFromEnvelope,
  updateExtensionL20Config,
  updateExtensionQhlyConfig,
  updateQhlyConfigInState,
} from "../src/engine/extensionSurfaceRuntime.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(scriptsDir, "..");
const repoRoot = resolve(scriptsDir, "../../..");

function loadInventoryItems(sourceId) {
  const invPath = resolve(repoRoot, "docs/migration/g0/inventory_status.json");
  const inv = JSON.parse(readFileSync(invPath, "utf8"));
  return Object.values(inv.items).filter((item) => item.source_id === sourceId);
}

test("L14 remaining qiDongYe & charactermenu upstream alignment", () => {
  const base = createDefaultShoushaConfig();
  assert.ok(resolveActiveShoushaSurfaces({ ...base, qiDongYe: "on" }).includes("qidongye/layout_old.css"));
  assert.ok(!resolveActiveShoushaSurfaces({ ...base, qiDongYe: "on" }).includes("qidongye/layout_new.css"));

  const othersOn = resolveActiveShoushaSurfaces({ ...base, qiDongYe: "othersOn" });
  assert.ok(othersOn.includes("qidongye/layout_new.css"));
  assert.ok(!othersOn.includes("qidongye/layout_xbig.css"));

  const othersTwo = resolveActiveShoushaSurfaces({ ...base, qiDongYe: "othersTwo" });
  assert.ok(othersTwo.includes("qidongye/layout_new.css"));
  assert.ok(othersTwo.includes("qidongye/layout_xbig.css"));

  const off = resolveActiveShoushaSurfaces({ ...base, qiDongYe: "off" });
  assert.ok(!off.some((id) => id.startsWith("qidongye/")));

  assert.ok(resolveActiveShoushaSurfaces({ ...base, charactermenu: "old" }).includes("character/main1_old.css"));
  assert.ok(resolveActiveShoushaSurfaces({ ...base, charactermenu: "new" }).includes("character/main1_new.css"));
  assert.ok(!resolveActiveShoushaSurfaces({ ...base, charactermenu: "none" }).includes("character/main1_new.css"));
});

test("L20 标记补充: 1-to-1 coverage of all 46 ui_hooks + 3 CSS surfaces + 15 configs", () => {
  const l20 = loadInventoryItems("L20");
  const invHooks = l20.filter((i) => i.category === "ui_hooks").map((i) => i.id);
  const invSurfs = l20.filter((i) => i.category === "surfaces").map((i) => i.id);
  const invCfgs = l20.filter((i) => i.category === "configs").map((i) => i.id);

  assert.deepEqual(new Set(L20_UI_HOOK_IDS), new Set(invHooks));
  assert.deepEqual(new Set(L20_CSS_SURFACE_IDS), new Set(invSurfs));
  assert.deepEqual(new Set(L20_CONFIG_KEYS), new Set(invCfgs));
  assert.equal(invHooks.length, 46);
  assert.equal(invSurfs.length, 3);
  assert.equal(invCfgs.length, 15);

  let state = createInitialL20RuntimeState();
  for (const hook of L20_UI_HOOK_IDS) {
    state = dispatchL20UiHook(state, hook, {
      seat: 1,
      faction: "shu",
      markName: "yinyang_mark",
      cardName: "ShaFire",
      isDamageCard: true,
      stars: 5,
      weaponType: "dao",
      binglinSkill: "xiangyang",
    });
  }
  assert.equal(state.recastCount, 1);
  assert.equal(state.binglinCrest, "xiangyang");
  assert.equal(state.binglinTeammateHint, true);

  // Explicit envelope sync verification
  const env = {
    result: null,
    events: [
      { type: "CardPlayed", source: 0, card: "Jiu" },
      { type: "CardPlayed", source: 0, card: "Sha" },
      { type: "DamageDealt", source: 0, target: 1, amount: 3 },
      { type: "HpRecovered", source: 1, target: 1, amount: 1 },
      { type: "PlayerDied", source: 0, target: 1 },
    ],
    choice: null,
    view: {
      revision: 5,
      state_hash: "h",
      observer: 0,
      round: 2,
      current_player: 0,
      phase: "Prepare",
      seats: [],
      known_cards: [],
      outcome: null,
    },
    legal_actions: [],
    hash_hex: "h",
    revision: 5,
    finished: false,
  };
  const synced = syncL20MarkEffectsFromEnvelope(createInitialL20RuntimeState(), env);
  assert.equal(synced.killStreakBySeat[0], 1);
  assert.equal(synced.rankSweepBySeat[0], 4);
  assert.equal(synced.weaponStrikeBySeat[1], "jian");
  assert.ok(l20ArenaDatasetAttrs(synced)["data-l20-surfaces"].includes("mark.css"));

  const snap = updateExtensionL20Config(ingestBattleEnvelopeIntoSurfaces(createInitialExtensionSurfacesState(), env), {
    daojianfu_type: "dao",
  });
  const restored = deserializeExtensionSurfacesSnapshot(serializeExtensionSurfacesSnapshot(snap));
  assert.equal(restored.uiSuites?.l20Ui?.configs.daojianfu_type, "dao");
});

test("L09 千幻聆音: 1-to-1 coverage of all 82 configs + 8 hooks + 12 surfaces & Decision 34/38", () => {
  const l09 = loadInventoryItems("L09");
  const invCfgs = l09.filter((i) => i.category === "configs").map((i) => i.id);
  const invHooks = l09.filter((i) => i.category === "ui_hooks").map((i) => i.id);
  const invSurfs = l09.filter((i) => i.category === "surfaces").map((i) => i.id);

  assert.equal(QHLY_CONFIG_SPECS.length, 82);
  assert.deepEqual(new Set(QHLY_CONFIG_SPECS.map((s) => s.key)), new Set(invCfgs));
  assert.deepEqual(new Set(QHLY_UI_HOOK_IDS), new Set(invHooks));
  assert.deepEqual(new Set(Object.keys(QHLY_SURFACE_PRESETS)), new Set(invSurfs));

  // Strict Decision 38 import validation
  const ok = parseQhlyConfigImportJson('{"qhly_fontsize1":"8","qhly_notbb":"5","qhly_skinButton":true}');
  assert.equal(ok.importedKeys, 3);
  assert.throws(() => parseQhlyConfigImportJson('{"unknownQhly":1}'));
  assert.throws(() => parseQhlyConfigImportJson('{"qhly_skinButton":"notbool"}'));
  assert.throws(() => parseQhlyConfigImportJson('{"qhly_currentViewSkin":"<script>x</script>"}'));

  // Notbb cooldown, keyword highlight, subtitle replacement, clear/restore
  let qh = updateQhlyConfigInState(createInitialQhlyRuntimeState(), {
    qhly_notbb: "5",
    qhly_notbb_range: "skill",
    qhly_keymarkopen: true,
    qhly_titlereplace: "pkg",
    qhly_fontsize1: "9",
    qhly_currentViewSkin: "shuimo",
  });
  assert.equal(qh.activeStylesheet, "newui_shuimo.css");
  assert.equal(qhlyArenaDatasetAttrs(qh.config)["data-qhly-viewskin"], "shuimo");
  assert.equal(resolveQhlyCharacterSubtitle(qh.config, { title: "魏帝", skinName: "龙腾", pkgName: "谋攻" }), "谋攻");

  const segs = highlightSkillKeywords("锁定技：出牌阶段限定技。", true);
  assert.ok(segs.some((s) => s.text === "锁定技" && s.color === "#60a5fa"));
  assert.ok(segs.some((s) => s.text === "限定技" && s.color === "#fb923c"));

  const first = canTriggerQhlyVoiceWithNotbb(qh, "jianxiong", "caocao", 10_000);
  assert.equal(first.allowed, true);
  const blocked = canTriggerQhlyVoiceWithNotbb(first.nextState, "jianxiong", "caocao", 12_000);
  assert.equal(blocked.allowed, false);
  const allowedAfter = canTriggerQhlyVoiceWithNotbb(first.nextState, "jianxiong", "caocao", 16_000);
  assert.equal(allowedAfter.allowed, true);

  const cleared = clearQhlySkinSettings(qh);
  assert.deepEqual(cleared.replaceSkinByGeneral, {});
  const restoredOfficial = restoreOfficialSkins(cleared);
  assert.equal(restoredOfficial.replaceSkinByGeneral.caocao, "origin");

  const snap = updateExtensionQhlyConfig(createInitialExtensionSurfacesState(), {
    qhly_fontsize1: "8",
    qhly_skinButton: true,
    qhly_keymarkopen: true,
  });
  const roundtrip = deserializeExtensionSurfacesSnapshot(serializeExtensionSurfacesSnapshot(snap));
  assert.equal(roundtrip.uiSuites?.qhly?.config.qhly_fontsize1, "8");
  assert.equal(roundtrip.uiSuites?.qhly?.config.qhly_skinButton, true);
});

test("Cleanroom check: zero Node-only imports in Batch UI-5 TS modules", () => {
  for (const rel of [
    "src/engine/surfaces/l20MarkSupplementRuntime.ts",
    "src/engine/surfaces/qhlyConfigSchema.ts",
    "src/engine/surfaces/qhlySurface.ts",
  ]) {
    const src = readFileSync(resolve(webClientDir, rel), "utf8");
    assert.ok(!/from\s+["'](?:node:|fs|path|child_process)/.test(src), `Node import in ${rel}`);
    assert.ok(!/\beval\s*\(|new\s+Function\s*\(/.test(src), `eval/Function in ${rel}`);
  }
});
