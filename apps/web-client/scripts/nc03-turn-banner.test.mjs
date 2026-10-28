import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";
import { computed, reactive } from "vue";

// Execute the actual SFC computed expression with a reactive view.
function fixture() {
  const sfc = readFileSync(new URL("../src/views/BattleView.vue", import.meta.url), "utf8");
  const script = sfc.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile("BattleView.ts", script, ts.ScriptTarget.Latest, true);
  const declarations = ast.statements.flatMap((node) =>
    ts.isVariableStatement(node) ? [...node.declarationList.declarations] : []);
  const get = (name) => declarations.find((node) => node.name.getText(ast) === name);
  const labels = get("PHASE_LABELS");
  const turn = get("turnInfo");
  assert.ok(labels && turn);
  const printer = ts.createPrinter();
  const source = `const ${printer.printNode(ts.EmitHint.Unspecified, labels, ast)};
const ${printer.printNode(ts.EmitHint.Unspecified, turn, ast)};`;
  const js = ts.transpileModule(source, { compilerOptions: { target: ts.ScriptTarget.ES2022 } }).outputText;
  const store = reactive({ view: null, lastEvents: [] });
  const info = new Function("computed", "store", js + "\nreturn turnInfo;")(computed, store);
  return { store, info };
}

test("原生零号座位的回合与结束阶段可见", () => {
  const f = fixture();
  f.store.view = { current_player: 0, phase: "Play", round: 1, seats: [{ player: 0 }, { player: 1 }] };
  assert.deepEqual(f.info.value, { currentPlayer: 0, phase: "Play", phaseLabel: "出牌", round: 1 });
  f.store.view.phase = "End";
  assert.equal(f.info.value.phaseLabel, "结束");
});

test("非零座位与准备阶段保持可见", () => {
  const f = fixture();
  f.store.view = { current_player: 2, phase: "Prepare", round: 3, seats: [{ player: 2 }] };
  assert.deepEqual(f.info.value, { currentPlayer: 2, phase: "Prepare", phaseLabel: "准备", round: 3 });
});

test("旧线无当前玩家的零哨兵仍使用事件回退或不显示", () => {
  const f = fixture();
  f.store.view = { current_player: 0, phase: "Play", round: 2, seats: [{ player: 1 }] };
  assert.equal(f.info.value, null);
  f.store.lastEvents = [{ type: "TurnStateChanged", current: 1, phase: "Draw" }];
  assert.deepEqual(f.info.value, { currentPlayer: 1, phase: "Draw", phaseLabel: "摸牌", round: 2 });
  f.store.view = { current_player: 0, phase: "Idle", round: 2, seats: [{ player: 0 }] };
  f.store.lastEvents = [];
  assert.equal(f.info.value, null);
});
