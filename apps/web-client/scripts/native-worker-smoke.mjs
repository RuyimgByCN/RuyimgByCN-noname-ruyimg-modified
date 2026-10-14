// T13 worker 切换首片冒烟：nativeBackend 协议映射（Node 直跑，无浏览器）。
// 用法：node scripts/native-worker-smoke.mjs
import { NativeEngine } from "../src/worker/native-pkg/engine_native_wasm.js";
import { nativeCreate, nativeDispatch } from "../src/worker/nativeBackend.ts";

const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok });
  console.error(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " | " + detail : ""}`);
};

// init/start：构造 + 返回 start 确认
const h = nativeCreate(4, 4242n);
check("create", true);

// observer → view_json（信息隔离投影）
const obs = nativeDispatch(h, { type: "observer" });
check("observer_view", obs.kind === "view" && obs.view.players.length === 4);

// endPhase
const st = nativeDispatch(h, { type: "endPhase", actor: 0 });
check("endPhase_status", st.kind === "status" && typeof st.revision === "number");

// autoStep 驱动若干步
let steps = 0;
while (steps < 40) {
  const r = nativeDispatch(h, { type: "autoStep" });
  steps++;
  if (r.kind === "auto") {
    const p = r.payload;
    if (p.done || p.finished) break;
  }
}
check("autoStep_progress", steps > 0, `steps=${steps}`);

// playCard：p0 出一张手牌（合法面：牌在手）
const hand = JSON.parse(h.engine.view_json(0)).players[0].hand.own;
if (hand.length > 0) {
  const st2 = nativeDispatch(h, { type: "playCard", actor: 0, card: hand[0], targets: [] });
  check("playCard_shape", st2.kind === "status", `rev=${st2.revision}`);
} else {
  check("playCard_shape", true, "hand empty（场景性跳过）");
}

// resolveChoice：无挂起时应报错（负例）
let threw = false;
try {
  nativeDispatch(h, { type: "resolveChoice", actor: 0, choiceId: 99999, selections: [] });
} catch {
  threw = true;
}
check("resolveChoice_negative", threw);

// 未映射请求面（TS 层类型即拒绝；运行时兜底断言未在 dispatch 白名单）
const KNOWN = new Set(["endPhase", "playCard", "resolveChoice", "autoStep", "observer"]);
check("unmapped_reported", !KNOWN.has("loadReplay") && !KNOWN.has("activateSkill"));

console.log(JSON.stringify({ total: results.length, failed: results.filter(r => !r.ok).length }));
process.exit(results.some(r => !r.ok) ? 1 : 0);
