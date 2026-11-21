import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  SHOUSHA_CONFIG_SPECS,
  SHOUSHA_SURFACE_SPECS,
  SHOUSHA_UI_HOOK_IDS,
  SHOUSHA_SURFACE_MODULES,
  assignPauseGuessInState,
  computeHandLuckScore,
  createDefaultShoushaConfig,
  createInitialDecadeUiRuntimeState,
  createInitialExtensionSurfacesState,
  createInitialShoushaUiRuntimeState,
  deserializeExtensionSurfacesSnapshot,
  dispatchShoushaUiHook,
  ingestBattleEnvelopeIntoSurfaces,
  parseShoushaConfigImportJson,
  resolveActiveDecadeCssSurfaces,
  resolveActiveShoushaSurfaces,
  resolveShoushaGtbbBanner,
  resolveShoushaGuanjie,
  serializeExtensionSurfacesSnapshot,
  shoushaArenaDatasetAttrs,
  sortShoushaHandCards,
  syncGuanXingChoice,
  syncShoushaUiFromEnvelope,
  updateDecadePlayerCssVariant,
  updateExtensionShoushaConfig,
} from "../src/engine/extensionSurfaceRuntime.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const webClientDir = resolve(scriptsDir, "..");
const repoRoot = resolve(scriptsDir, "../../..");

function loadInventoryItemsBySource(sourceId) {
  const invPath = resolve(repoRoot, "docs/migration/g0/inventory_status.json");
  const inv = JSON.parse(readFileSync(invPath, "utf8"));
  return Object.values(inv.items).filter((item) => item.source_id === sourceId);
}

test("L14 手杀ui: exact 1-to-1 coverage of all 141 inventory items (48 configs + 53 hooks + 40 surfaces)", () => {
  const l14 = loadInventoryItemsBySource("L14");
  const configs = l14.filter((item) => item.category === "configs");
  const hooks = l14.filter((item) => item.category === "ui_hooks");
  const surfaces = l14.filter((item) => item.category === "surfaces");

  assert.equal(configs.length, 48);
  assert.equal(hooks.length, 53);
  assert.equal(surfaces.length, 40);

  assert.deepEqual(
    new Set(SHOUSHA_CONFIG_SPECS.map((s) => s.key)),
    new Set(configs.map((c) => c.id)),
  );
  assert.deepEqual(new Set(SHOUSHA_UI_HOOK_IDS), new Set(hooks.map((h) => h.id)));
  assert.deepEqual(new Set(SHOUSHA_SURFACE_MODULES), new Set(SHOUSHA_SURFACE_SPECS.map((s) => s.id)));

  for (const invSurf of surfaces) {
    const expectedTail = `/手杀ui/`;
    const idx = invSurf.upstream_ref.indexOf(expectedTail);
    assert.ok(idx >= 0, `unexpected L14 upstream_ref: ${invSurf.upstream_ref}`);
    const relPath = invSurf.upstream_ref.slice(idx + expectedTail.length).split(":")[0];
    assert.ok(
      SHOUSHA_SURFACE_SPECS.some((s) => s.id === relPath),
      `missing L14 surface ${relPath}`,
    );
  }
});

test("L14 手杀ui: all 53 UI hooks, muniu hand sort, guanjie & handLuckScore run cleanly", () => {
  let state = createInitialShoushaUiRuntimeState();
  assert.equal(state.zlltOutcropEnabled, false);

  for (const hook of SHOUSHA_UI_HOOK_IDS) {
    state = dispatchShoushaUiHook(state, hook, {
      seat: 1,
      onlineRoom: false,
      sayText: "测试发言",
      handCards: [
        { id: 10, name: "sha", suit: "diamond", number: 7, muniu: false },
        { id: 20, name: "shan", suit: "spade", number: 2, muniu: true },
        { id: 30, name: "sha", suit: "spade", number: 5, muniu: false },
      ],
    });
  }
  assert.deepEqual(state.sortedHandCardIds, [20, 30, 10]);
  assert.equal(state.chatBubbleBySeat[1], "测试发言");

  const sorted = sortShoushaHandCards([
    { id: 1, name: "tao", suit: "heart", number: 6, muniu: false },
    { id: 2, name: "sha", suit: "club", number: 9, muniu: true },
  ]);
  assert.deepEqual(sorted.map((c) => c.id), [2, 1]);

  assert.equal(resolveShoushaGuanjie({ identity: "Lord", max_hp: 5, hp: 5 }, 8).label, "大元帅");
  assert.equal(resolveShoushaGuanjie({ identity: "Rebel", max_hp: 4, hp: 4 }, 8).label, "大将军");
  assert.ok(computeHandLuckScore([]) === 0);
  assert.ok(
    computeHandLuckScore([
      { id: 1, def: 1, suit: "Heart", rank: 4, owner: 0, zone: "Hand", kind: "Tao" },
      { id: 2, def: 2, suit: "Spade", rank: 7, owner: 0, zone: "Hand", kind: "Sha" },
    ]) >= 60,
  );
});

test("L14 手杀ui: all 40 surfaces reachable by config presets & dataset attributes reflect config", () => {
  const defaultCfg = createDefaultShoushaConfig();
  const seen = new Set();
  const variants = [
    [defaultCfg, { mobileViewport: false, bossMode: false }],
    [{ ...defaultCfg, yangshi: "on", charactermenu: "old", KGMH: "1", yangshi_bg: "1", qiDongYe: "on", xindchoose: "shousha", xindchoose_type: "old" }, { mobileViewport: true, bossMode: false }],
    [{ ...defaultCfg, yangshi: "on", qiDongYe: "othersTwo" }, { mobileViewport: true, bossMode: true }],
    [{ ...defaultCfg, yangshi: "off", KGMH: "2", yangshi_bg: "2", qiDongYe: "off", xindchoose: "shizhounian" }, { mobileViewport: false, bossMode: false }],
    [{ ...defaultCfg, yangshi: "off" }, { mobileViewport: true, bossMode: false }],
  ];
  for (const [cfg, opts] of variants) {
    for (const id of resolveActiveShoushaSurfaces(cfg, opts)) seen.add(id);
  }
  assert.equal(seen.size, 40, `missing surfaces: ${SHOUSHA_SURFACE_MODULES.filter((m) => !seen.has(m))}`);

  const attrs = shoushaArenaDatasetAttrs({ ...defaultCfg, LTAN: true, jindutiaoYangshi: "2", GTBBYangshi: "off" });
  assert.equal(attrs["data-shousha-ltan"], "hide");
  assert.equal(attrs["data-shousha-jindutiao-style"], "2");
  assert.equal(attrs["data-shousha-gtbb-style"], "off");
  assert.ok(resolveShoushaGtbbBanner({ ...defaultCfg, GTBB: true, GTBBYangshi: "off" }, 1)?.includes("十周年播报"));
  assert.equal(resolveShoushaGtbbBanner({ ...defaultCfg, GTBB: false }, 1), null);
});

test("L14 Decision 38 & Decision 34: strict JSON import guard + config/snapshot save-restore", () => {
  const ok = parseShoushaConfigImportJson('{"yangshi":"off","jindutiaoSet":"35","GTBB":false}');
  assert.equal(ok.importedKeys, 3);
  assert.equal(ok.patch.jindutiaoSet, "35");

  assert.throws(() => parseShoushaConfigImportJson('{"unknownKey":1}'));
  assert.throws(() => parseShoushaConfigImportJson('{"GTBB":"yes"}'));
  assert.throws(() => parseShoushaConfigImportJson('{"yangshi":"<script>alert(1)</script>"}'));

  let snap = createInitialExtensionSurfacesState();
  snap = updateExtensionShoushaConfig(snap, {
    yangshi: "off",
    jindutiaoSet: "36",
    xindsingle: false,
    ZLLT: false,
  });
  const roundtrip = deserializeExtensionSurfacesSnapshot(serializeExtensionSurfacesSnapshot(snap));
  assert.equal(roundtrip.uiSuites?.shoushaUi?.config.yangshi, "off");
  assert.equal(roundtrip.uiSuites?.shoushaUi?.config.jindutiaoSet, "36");
  assert.equal(roundtrip.uiSuites?.shoushaUi?.config.xindsingle, false);
});

test("L08 final 14 closure: 7 player CSS variants, GuanXing choice sync, pause turnActionLogs & guesses", () => {
  let decade = createInitialDecadeUiRuntimeState();
  for (const variant of [
    "player_new.css",
    "player1.css",
    "player1_PC.css",
    "player2.css",
    "player2_PC.css",
    "player3.css",
    "player3_PC.css",
  ]) {
    decade = updateDecadePlayerCssVariant(decade, variant);
    assert.equal(decade.playerCssVariant, variant);
    assert.ok(decade.activeCssSurfaces.includes(variant));
    assert.ok(decade.activeCssSurfaces.includes("animation1.css"));
  }
  const activeMode = resolveActiveDecadeCssSurfaces({ ...decade.config, JBDDZ: true });
  assert.ok(activeMode.includes("doudizhu.css"));
  assert.ok(activeMode.includes("junba.css"));

  decade = assignPauseGuessInState(decade, 2, "反");
  assert.equal(decade.pauseGuessBySeat[2], "反");
  decade = assignPauseGuessInState(decade, 2, "");
  assert.equal(decade.pauseGuessBySeat[2], undefined);

  const fakeEnv = {
    result: null,
    events: [{ type: "DamageDealt", target: 1 }, { type: "CardPlayed", source: 0, card: "TrickWuxie" }],
    choice: {
      choice_id: 99,
      observer: 0,
      source_skill: "guanxing",
      prompt: "观星：排列牌堆顶",
      options: [
        { token: 1, label_key: "杀" },
        { token: 2, label_key: "桃" },
      ],
    },
    view: {
      revision: 12,
      state_hash: "abc",
      observer: 0,
      round: 2,
      current_player: 0,
      phase: "Prepare",
      seats: [
        { player: 0, character_def: 1, seat: 0, hp: 4, max_hp: 4, alive: true, turned_over: false, hand_count: 4, identity: "Lord", skills: [] },
        { player: 1, character_def: 2, seat: 1, hp: 3, max_hp: 4, alive: true, turned_over: false, hand_count: 3, identity: null, skills: [] },
      ],
      known_cards: [
        { id: 501, def: 1, suit: "Spade", rank: 7, owner: 0, zone: "Hand", kind: "Sha" },
        { id: 502, def: 3, suit: "Heart", rank: 8, owner: null, zone: "DrawPile", kind: "Tao" },
      ],
      outcome: null,
    },
    legal_actions: [],
    hash_hex: "abc",
    revision: 12,
    finished: false,
  };

  const gxSynced = syncGuanXingChoice(decade, fakeEnv);
  assert.equal(gxSynced.guanXing?.open, true);
  assert.equal(gxSynced.guanXing?.topCards.length, 1);

  const snapAfter = ingestBattleEnvelopeIntoSurfaces(createInitialExtensionSurfacesState(), fakeEnv);
  assert.ok((snapAfter.uiSuites?.decadeUi?.turnActionLogs.length ?? 0) >= 2);
  assert.equal(snapAfter.uiSuites?.shoushaUi?.wuxiePromptActive, true);
  assert.ok(snapAfter.uiSuites?.shoushaUi?.lastGtbbText);
});

test("Cleanroom check: zero Node-only imports in shousha/decade surface TS modules", () => {
  for (const rel of [
    "src/engine/surfaces/shoushaConfigSchema.ts",
    "src/engine/surfaces/shoushaSurfacesRuntime.ts",
    "src/engine/surfaces/shoushaUiSurface.ts",
    "src/engine/surfaces/decadeSurfacesRuntime.ts",
    "src/engine/surfaces/decadeHooksRuntime.ts",
    "src/engine/surfaces/decadeUiSurface.ts",
  ]) {
    const src = readFileSync(resolve(webClientDir, rel), "utf8");
    assert.ok(!/from\s+["'](?:node:|fs|path|child_process)/.test(src), `Node import found in ${rel}`);
    assert.ok(!/\beval\s*\(|new\s+Function\s*\(/.test(src), `Dynamic code eval found in ${rel}`);
  }
});
