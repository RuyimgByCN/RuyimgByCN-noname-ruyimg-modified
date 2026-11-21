import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, resolve } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";
import {
  DECADE_ANIMATION_CATALOG,
  DECADE_CONFIG_SPECS,
  DECADE_CSS_SURFACE_SPECS,
  DECADE_JS_SURFACE_IDS,
  DECADE_PLAYER_NODE_KEYS,
  DECADE_SURFACE_MODULES,
  appendChatMessage,
  applyDecadeConfigUpdate,
  buildPauseHistoryRows,
  cardsToGuanXingItems,
  clampSliderValue,
  createDefaultDecadeConfig,
  createGuanXingState,
  createInitialChatBoxState,
  createInitialDecadeUiRuntimeState,
  createInitialExtensionSurfacesState,
  decadeArenaDatasetAttrs,
  dispatchDecadeUiHook,
  moveGuanXingCardRow,
  normalizeDecadeConfig,
  parseDecadeConfigImportJson,
  resolveActiveDecadeCssSurfaces,
  resolveGameOverBanner,
  resolveGameStartBanner,
  restoreExtensionSurfacesForLoad,
  serializeExtensionSurfacesSnapshot,
  swapGuanXingCards,
  updateDecadeConfigInState,
  updateExtensionDecadeConfig,
} from "../src/engine/extensionSurfaceRuntime.ts";
import { computeDecadeSeatAnchor } from "../src/engine/surfaces/decadeTabletopLayout.ts";

const scriptsDir = dirname(fileURLToPath(import.meta.url));
const repoRoot = resolve(scriptsDir, "../../..");

function loadL08InventoryItems() {
  const invPath = resolve(repoRoot, "docs/migration/g0/inventory_status.json");
  const inv = JSON.parse(readFileSync(invPath, "utf8"));
  const l08 = Object.values(inv.items).filter((item) => item.source_id === "L08");
  return {
    configs: l08
      .filter((item) => item.category === "configs")
      .map((item) => item.id),
    surfaces: l08
      .filter((item) => item.category === "surfaces")
      .map((item) => item.id),
  };
}

test("L08 44 configs match g0/inventory_status.json 1-to-1 and expose 20 player node keys", () => {
  const { configs } = loadL08InventoryItems();
  assert.equal(configs.length, 44);
  assert.equal(DECADE_CONFIG_SPECS.length, 44);
  assert.equal(DECADE_PLAYER_NODE_KEYS.length, 20);

  const specKeys = new Set(DECADE_CONFIG_SPECS.map((s) => s.key));
  const defaults = createDefaultDecadeConfig();
  for (const key of configs) {
    assert.ok(specKeys.has(key), `Missing DECADE_CONFIG_SPECS entry for ${key}`);
    assert.ok(key in defaults, `Missing default value for config ${key}`);
  }
  assert.deepEqual(new Set(Object.keys(defaults)), new Set(configs));
});

test("L08 43 surfaces (10 JS + 33 CSS) match g0/inventory_status.json 1-to-1 without Spine binary assets", () => {
  const { surfaces } = loadL08InventoryItems();
  assert.equal(surfaces.length, 43);
  assert.equal(DECADE_JS_SURFACE_IDS.length, 10);
  assert.equal(DECADE_CSS_SURFACE_SPECS.length, 33);
  assert.equal(DECADE_SURFACE_MODULES.length, 43);

  const expected = new Set(surfaces);
  assert.deepEqual(new Set(DECADE_SURFACE_MODULES), expected);

  // Cleanroom: animationlist catalog has zero .skel / .atlas / .png binary references
  assert.ok(DECADE_ANIMATION_CATALOG.length >= 20);
  for (const entry of DECADE_ANIMATION_CATALOG) {
    assert.ok(entry.name.length > 0 && entry.fallbackLabel.length > 0);
    assert.ok(!/\.(skel|atlas|png|mp3|ogg)$/i.test(JSON.stringify(entry)));
  }

  // Every single one of the 33 CSS stylesheets activates cleanly under valid config + viewport combos
  const base = createDefaultDecadeConfig();
  const seenCss = new Set([
    ...resolveActiveDecadeCssSurfaces(base, { mobileViewport: false }),
    ...resolveActiveDecadeCssSurfaces(
      { ...base, decadeLayout: "off", newDecadeStyle: "off" },
      { mobileViewport: false },
    ),
    ...resolveActiveDecadeCssSurfaces(
      { ...base, decadeLayout: "off", newDecadeStyle: "off" },
      { mobileViewport: true },
    ),
  ]);
  for (const spec of DECADE_CSS_SURFACE_SPECS) {
    assert.ok(seenCss.has(spec.id), `CSS surface ${spec.id} was never activated`);
  }
});

test("L08 mutual exclusions, _connectMode routing, aloneEquip gate, and dataset projection match upstream", () => {
  const base = createDefaultDecadeConfig();

  // Offline cardBorder disables game_cardSet (extension.js:12181-12241)
  const offlineSet = applyDecadeConfigUpdate(base, { game_cardSet: "gold", game_cardSet2: "black" });
  const offlineAttr = decadeArenaDatasetAttrs(offlineSet);
  assert.equal(offlineAttr["data-card-set"], "gold");

  const offlineBorder = applyDecadeConfigUpdate(offlineSet, { cardBorder: "four" });
  assert.equal(offlineBorder.game_cardSet, "off");
  assert.equal(offlineBorder.game_cardSet2, "black");
  assert.equal(decadeArenaDatasetAttrs(offlineBorder)["data-card-border"], "four");

  // Online mode routes cardSet2 / cardHelasisy2 and disables cardSet2 on cardPrettify change
  const onlineSet = applyDecadeConfigUpdate(offlineSet, {
    _connectMode: true,
    cardHelasisy: "helasisy",
    cardHelasisy2: "hakimi",
  });
  const onlineAttr = decadeArenaDatasetAttrs(onlineSet);
  assert.equal(onlineAttr["data-card-set"], "black");
  assert.equal(onlineAttr["data-card-helasisy"], "hakimi");

  const onlinePretty = applyDecadeConfigUpdate(onlineSet, { cardPrettify: "webp" });
  assert.equal(onlinePretty.game_cardSet2, "off");
  assert.equal(onlinePretty.game_cardSet, "gold");

  // aloneEquip is gated on newDecadeStyle === 'on'
  assert.equal(
    decadeArenaDatasetAttrs({ ...base, newDecadeStyle: "on", aloneEquip: true })["data-alone-equip"],
    "on",
  );
  assert.equal(
    decadeArenaDatasetAttrs({ ...base, newDecadeStyle: "off", aloneEquip: true })["data-alone-equip"],
    "off",
  );
});

test("L08 10 JS surface helpers and hooks drive real state transitions without side effects", () => {
  // component.js: slider + chatBox
  assert.deepEqual(clampSliderValue(10, 100, 140), { value: 100, percent: 100 });
  assert.deepEqual(clampSliderValue(10, 100, 55), { value: 55, percent: 50 });
  let chat = createInitialChatBoxState();
  chat = appendChatMessage(chat, "关羽", "忠义当先");
  assert.equal(chat.messages.length, 2);
  assert.equal(chat.messages[1].text, "忠义当先");

  // content.js: chooseGuanXing card sort/move
  const items = cardsToGuanXingItems(
    [
      { id: 11, def: 1, suit: "Heart", rank: 1, owner: null, zone: "DrawPile", kind: "Tao" },
      { id: 12, def: 2, suit: "Spade", rank: 13, owner: null, zone: "DrawPile", kind: "Sha" },
    ],
    (k) => String(k),
  );
  let gx = createGuanXingState(items);
  gx = swapGuanXingCards(gx, 11, 12);
  assert.deepEqual(gx.topCards.map((c) => c.id), [12, 11]);
  gx = moveGuanXingCardRow(gx, 11);
  assert.deepEqual(gx.topCards.map((c) => c.id), [12]);
  assert.deepEqual(gx.bottomCards.map((c) => c.id), [11]);

  // splash.js + pause.js
  assert.ok(resolveGameStartBanner("SF_kaizhan_eff_dasima")?.includes("大司马"));
  assert.equal(resolveGameStartBanner("off"), null);
  assert.ok(resolveGameOverBanner("shizhounian", true)?.includes("凯旋"));
  assert.equal(resolveGameOverBanner("off", false), null);
  const pauseRows = buildPauseHistoryRows({
    revision: 5,
    round: 2,
    phase: "Play",
    current_player: 1,
    seats: [
      {
        player: 1,
        character_def: 1,
        seat: 0,
        hp: 3,
        max_hp: 4,
        alive: true,
        turned_over: false,
        hand_count: 5,
        identity: "Lord",
        skills: [],
      },
    ],
    known_cards: [],
  });
  assert.equal(pauseRows[0].name, "P1");
  assert.equal(pauseRows[0].hpText, "3/4");

  // effect.js + meihua.js + animation hooks via dispatchDecadeUiHook
  let runtime = createInitialDecadeUiRuntimeState();
  runtime = updateDecadeConfigInState(
    runtime,
    { EffectWood: true, cardUseEffect: true, playerKillEffect: true },
    { persist: false },
  );
  runtime = dispatchDecadeUiHook(runtime, "_HpRecover", { seat: 1 });
  runtime = dispatchDecadeUiHook(runtime, "_decadeUI_usecardBegin", { seat: 1, equipName: "过河拆桥" });
  runtime = dispatchDecadeUiHook(runtime, "_decadeUI_dieKillEffect", {
    seat: 0,
    killerName: "赵云",
    victimName: "许褚",
  });
  runtime = dispatchDecadeUiHook(runtime, "_dyjinengtx", {
    seat: 0,
    generalName: "赵云",
    skillName: "龙胆",
  });
  runtime = dispatchDecadeUiHook(runtime, "_recasting", { seat: 0 });
  assert.ok(runtime.healAuraSeats.includes(1));
  assert.equal(runtime.cardUseCueBySeat[1], "过河拆桥");
  assert.equal(runtime.killOverlay?.killerName, "赵云");
  assert.equal(runtime.skillPopup?.skillName, "龙胆");
  assert.equal(runtime.recastCountBySeat[0], 1);
});

test("Decision 34 snapshot save/restore preserves all L08 configs and Decision 38 rejects unsafe import", () => {
  let snap = createInitialExtensionSurfacesState();
  snap = updateExtensionDecadeConfig(
    snap,
    {
      rightLayout: false,
      newDecadeStyle: "off",
      decadeLayout: "off",
      cardBorder: "night",
      dynamicBackground: "decade_jade",
      playerKillEffect: true,
      EffectWood: true,
    },
    { persist: false },
  );
  assert.equal(snap.theme.shoushaLayout, "left");
  assert.equal(snap.theme.decadeSkinStyle, "shousha");

  const json = serializeExtensionSurfacesSnapshot(snap);
  const restored = restoreExtensionSurfacesForLoad(json, { revision: 2 });
  assert.equal(restored.uiSuites?.decadeUi?.config.cardBorder, "night");
  assert.equal(restored.uiSuites?.decadeUi?.config.rightLayout, false);
  assert.equal(restored.uiSuites?.decadeUi?.config.dynamicBackground, "decade_jade");
  assert.equal(restored.uiSuites?.decadeUi?.config.playerKillEffect, true);

  // Decision 38 safe JSON config import validation
  const okImport = parseDecadeConfigImportJson(JSON.stringify({ borderLevel: "five", asyncMode: true }));
  assert.equal(okImport.importedKeys, 2);
  assert.equal(okImport.config.borderLevel, "five");
  assert.equal(okImport.config.asyncMode, true);
  assert.throws(() => parseDecadeConfigImportJson("<script>alert(1)</script>"), /DECADE_CONFIG_IMPORT_UNSAFE/);
  assert.throws(() => parseDecadeConfigImportJson("[1,2,3]"), /DECADE_CONFIG_IMPORT_OBJECT_REQUIRED/);
  assert.throws(() => parseDecadeConfigImportJson('{"cardBorder":123}'), /DECADE_CONFIG_IMPORT_INVALID_ENUM/);
  assert.throws(() => parseDecadeConfigImportJson('{"rightLayout":"bad"}'), /DECADE_CONFIG_IMPORT_INVALID_TYPE/);
  assert.throws(() => parseDecadeConfigImportJson('{"node":["x"]}'), /DECADE_CONFIG_IMPORT_INVALID_VALUE:node/);
  assert.throws(() => parseDecadeConfigImportJson('{"unknownKey":true}'), /DECADE_CONFIG_IMPORT_UNKNOWN_KEY/);

  // Corrupt persisted configs fall back safely per key during normalizeDecadeConfig
  const sanitized = normalizeDecadeConfig({ cardBorder: 123, rightLayout: "bad", node: ["x"], borderLevel: "four" });
  assert.equal(sanitized.cardBorder, "off");
  assert.equal(sanitized.rightLayout, true);
  assert.equal(sanitized.node.length, 20);
  assert.equal(sanitized.borderLevel, "four");

  // decadeLayout "on" vs "off" shifts seat geometry + line centerPct cleanly
  const anchorOn = computeDecadeSeatAnchor(2, 6, { decadeLayout: "on" });
  const anchorOff = computeDecadeSeatAnchor(2, 6, { decadeLayout: "off" });
  assert.notEqual(anchorOn.cssVars["--seat-top"], anchorOff.cssVars["--seat-top"]);
  assert.notDeepEqual(anchorOn.centerPct, anchorOff.centerPct);
});
