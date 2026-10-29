import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";
import { compile, computed, createSSRApp, reactive } from "vue";
import { renderToString } from "vue/server-renderer";
import * as outcome from "../src/engine/outcome.ts";
import { nativeObserverToSnapshot } from "../src/worker/nativeBackend.ts";
import { applyEnvelope, createViewStore } from "../src/engine/viewStore.ts";

const sfc = readFileSync(new URL("../src/views/BattleView.vue", import.meta.url), "utf8");
const script = sfc.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
const ast = ts.createSourceFile("BattleView.ts", script, ts.ScriptTarget.Latest, true);
const declaration = ast.statements.flatMap((node) =>
  ts.isVariableStatement(node) ? [...node.declarationList.declarations] : [])
  .find((node) => node.name.getText(ast) === "matchOutcome");
assert.ok(declaration, "execute the SFC's actual outcome computed");
const source = `const ${ts.createPrinter().printNode(ts.EmitHint.Unspecified, declaration, ast)};`;
const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
const banner = sfc.match(/<section v-if="store.finished && matchOutcome"[\s\S]*?<\/section>/)[0];
const render = compile(banner);

function fixture(identity = "Rebel", player = 0) {
  const store = reactive({
    finished: true, revision: 9, lastEvents: [],
    view: { observer: player, seats: [{ player, identity }], outcome: null },
  });
  const info = new Function("computed", "store", "selfId", ...Object.keys(outcome),
    js + "\nreturn matchOutcome;")(computed, store, { value: player }, ...Object.values(outcome));
  return {
    store, info,
    html: () => renderToString(createSSRApp({
      setup: () => ({ store, matchOutcome: info, isOnline: false, connectAndStart() {} }),
      render,
    })),
  };
}

function nativeView(result) {
  return {
    seat: 0, turn: 0, phase: "end", round: 4, deckCount: 0, discard: [], table: [],
    outcome: result,
    players: [0, 1, 2, 3].map((seat) => ({
      seat, alive: seat === 3, hp: 1, maxHp: 4, hand: { own: [] }, handCount: 0,
      equips: [], judge: [], skills: [], revealedGenerals: [], turnedOver: false,
    })),
  };
}

test("native SingleWin protocol players 1/4 become local seats 0/3 without mutating protocol data", () => {
  for (const player of [1, 4]) {
    const raw = nativeView({ SingleWin: { player } });
    const snapshot = nativeObserverToSnapshot(raw);
    assert.deepEqual(snapshot.outcome, { SingleWin: { player: player - 1 } });
    assert.ok(snapshot.seats.some((seat) => seat.player === snapshot.outcome.SingleWin.player));
    assert.deepEqual(raw.outcome, { SingleWin: { player } });
    assert.equal(snapshot.observer, 0);
    assert.equal(snapshot.current_player, 0);
  }
});

test("native other outcome shapes and unknown detail pass through unchanged", () => {
  for (const result of [null, "finished", { RebelWin: null }, { LoyalistWin: null },
    { RenegadeWin: null }, { GroupWin: { group: "shu" } }, { FutureWin: { detail: 7 } }]) {
    assert.deepEqual(nativeObserverToSnapshot(nativeView(result)).outcome, result);
  }
});

test("actual BattleView computed and banner recognize a rebel observer's win and loss", async () => {
  const f = fixture("Rebel");
  f.store.view.outcome = { RebelWin: null };
  assert.equal(f.info.value.isSelfWin, true);
  let html = await f.html();
  assert.match(html, /outcome-banner win/);
  assert.match(html, /反贼胜/);
  assert.match(html, /🎉/);
  f.store.view.outcome = { LoyalistWin: null };
  assert.equal(f.info.value.isSelfWin, false);
  html = await f.html();
  assert.match(html, /outcome-banner lose/);
  assert.match(html, /主公方胜/);
});

test("identity victory follows each observer's own role, including loyalists losing to rebels", () => {
  for (const identity of ["Lord", "Loyalist", "Rebel", "Renegade"]) {
    const f = fixture(identity);
    for (const winner of ["LoyalistWin", "RebelWin", "RenegadeWin"]) {
      f.store.view.outcome = { [winner]: null };
      assert.equal(f.info.value.isSelfWin,
        winner === "LoyalistWin" ? ["Lord", "Loyalist"].includes(identity)
          : identity === (winner === "RebelWin" ? "Rebel" : "Renegade"), `${identity}: ${winner}`);
    }
  }
});

test("SingleWin zero and normalized last seat render a banner with the matching observer result", async () => {
  for (const player of [0, 3]) {
    const f = fixture(null, player);
    f.store.view = nativeObserverToSnapshot({ ...nativeView({ SingleWin: { player: player + 1 } }), seat: player });
    assert.equal(f.info.value.label, `单挑胜者 P${player}`);
    assert.equal(f.info.value.isSelfWin, true);
    assert.match(await f.html(), new RegExp(`单挑胜者 P${player}`));
    f.store.view.outcome = { SingleWin: { player: player === 0 ? 3 : 0 } };
    assert.equal(f.info.value.isSelfWin, false);
  }
});

test("group, unknown, unmodeled, draw, abort and hidden identity render neutral banners", async () => {
  for (const [result, text] of [
    [{ GroupWin: { group: "shu" } }, "势力胜（shu）"],
    [{ FutureWin: { detail: 7 } }, '已结束（{"FutureWin":{"detail":7}}）'],
    ["finished", "已结束（胜负类型未建模）"],
    [{ Draw: null }, "平局"], [{ Aborted: { reason_code: 1 } }, "对局中止"],
    [{ RebelWin: null }, "反贼胜"],
  ]) {
    const f = fixture(result?.GroupWin ? "Rebel" : null);
    f.store.view.outcome = result;
    assert.equal(f.info.value.label, text);
    assert.equal(f.info.value.isSelfWin, null);
    const html = await f.html();
    assert.match(html, /outcome-banner/);
    assert.doesNotMatch(html, /outcome-banner (win|lose)|💀|🎉|阵营获胜/);
  }
});

test("view outcome is authoritative; both MatchEnded event shapes use the same fallback presentation", () => {
  const f = fixture("Rebel");
  for (const event of [{ type: "MatchEnded", outcome: { RebelWin: null } },
    { MatchEnded: { outcome: { RebelWin: null } } }]) {
    f.store.view.outcome = null;
    f.store.lastEvents = [event];
    assert.equal(f.info.value.label, outcome.outcomeText({ RebelWin: null }));
    assert.equal(f.info.value.isSelfWin, true);
    f.store.view.outcome = { LoyalistWin: null };
    assert.equal(f.info.value.isSelfWin, false);
    assert.equal(f.info.value.label, outcome.outcomeText({ LoyalistWin: null }));
  }
});

test("MatchEnded timeline uses shared outcome wording", () => {
  const store = createViewStore();
  applyEnvelope(store, {
    revision: 9, hash_hex: "", finished: true, view: nativeObserverToSnapshot(nativeView(null)),
    choice: null, legal_actions: [], events: [{ MatchEnded: { outcome: { SingleWin: { player: 0 } } } }],
  });
  assert.equal(store.log[0].text, "终局 · 单挑胜者 P0");
});
