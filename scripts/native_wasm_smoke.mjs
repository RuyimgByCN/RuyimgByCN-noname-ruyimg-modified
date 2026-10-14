// T13 客户端产品面冒烟：engine-native-wasm 窄壳（Node，O4 harness 同款口径）。
// 断言面：建局/座位视图隔离/auto_step 自动对局/存档往返与同 seed 重放一致/双实例确定性/选择可见性。
// 用法：先 wasm-pack build crates/engine-native-wasm --target nodejs --release --no-opt
//      --out-dir <pkg>，再 PKG=<pkg> node scripts/native_wasm_smoke.mjs
const { NativeEngine } = await import(process.env.PKG + "/engine_native_wasm.js");
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok, detail });
  console.error(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " | " + detail : ""}`);
};

// ① 建局 + 座位视图信息隔离
const e = new NativeEngine(4, 4242n);
const v0 = JSON.parse(e.view_json(0));
check("build", true, `ruleset=v${v0.rulesetVersion} players=${v0.players.length}`);
const own = v0.players[0].hand;
const other = v0.players[1].hand;
check("view_own_hand", Array.isArray(own.own) && own.own.length === 4, `own=${own.own.length}`);
check("view_hidden_hand", other.hidden === true, JSON.stringify(other));
check("view_deck_count_only", typeof v0.deckCount === "number" && v0.deckCount > 0, `deck=${v0.deckCount}`);
const v1 = JSON.parse(e.view_json(1));
check("view_per_seat", v1.players[1].hand.own.length === 4 && v1.players[0].hand.hidden === true);

// ② auto_step 自动对局推进 + 中途存档往返
let steps = 0, choiceSeen = 0, suspended = 0;
let mid = null, midView = null;
while (!e.is_finished() && steps < 2000) {
  const r = JSON.parse(e.auto_step());
  if (r.kind === "answer") choiceSeen++;
  const out = r.outcome ? JSON.parse(r.outcome).outcome : null;
  if (out === "suspended") suspended++;
  if (steps === 60) {
    mid = e.save_hex();
    midView = e.view_json(0);
  }
  steps++;
}
check("auto_game_progress", steps < 2000 && (e.is_finished() || steps === 1999),
  `steps=${steps} finished=${e.is_finished()} answers=${choiceSeen}`);
check("auto_suspended_none_or_resumable", suspended < 50, `suspended=${suspended}`);

// ③ 存档往返：新实例恢复后视图一致
const e2 = new NativeEngine(4, 999n);
e2.restore_hex(mid);
check("restore_view_equal", e2.view_json(0) === midView, "mid-save restore per-view equal");
const e3 = new NativeEngine(4, 4242n);
for (let i = 0; i < 61; i++) e3.auto_step();
check("restore_matches_same_seed", e3.view_json(0) === midView, "same-seed 60 steps equal restored view");

// ④ 跨实例确定性（T13 口径复用）：双实例同 seed 独立推进 200 步
const a = new NativeEngine(4, 777n), b = new NativeEngine(4, 777n);
for (let i = 0; i < 200; i++) { a.auto_step(); b.auto_step(); }
check("dual_instance_determinism", a.view_json(0) === b.view_json(0),
  `hash_equal=${a.view_json(0) === b.view_json(0)}`);

// ⑤ 挂起选择可见面（选择存在性公开、选项仅本人）
const c = new NativeEngine(4, 31415n);
let sawOwn = false, sawOther = false;
for (let i = 0; i < 400 && !(sawOwn && sawOther); i++) {
  for (const seat of [0, 1, 2, 3]) {
    const ch = JSON.parse(c.open_choice_json(seat));
    if (ch && ch.hidden === undefined) sawOwn = true;
    if (ch && ch.hidden === true) sawOther = true;
  }
  if (!(sawOwn && sawOther)) c.auto_step();
}
check("choice_visibility_own_and_other", sawOwn && sawOther, `own=${sawOwn} other=${sawOther}`);

const fails = results.filter(r => !r.ok);
console.log(JSON.stringify({
  total: results.length,
  failed: fails.length,
  results,
}, null, 0));
process.exit(fails.length ? 1 : 0);
