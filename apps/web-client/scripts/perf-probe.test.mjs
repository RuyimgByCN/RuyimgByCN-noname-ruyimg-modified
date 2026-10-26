// 表现层性能采样：统计口径与 Worker 分段回包的单测（不依赖浏览器）。
import { test } from "node:test";
import assert from "node:assert/strict";
import {
  frameDeltas,
  percentile,
  summarizeFrames,
  summarizeInteractions,
  toInteractionRecord,
} from "../src/engine/perfStats.ts";
import {
  postWithStepTimings,
  setStepTimingsEnabled,
  stepBegin,
  stepMark,
} from "../src/worker/stepTimings.ts";

test("nearest-rank 分位与 NC12 口径一致；空输入为 null", () => {
  const v = [5, 1, 4, 2, 3];
  assert.equal(percentile(v, 50), 3);
  assert.equal(percentile(v, 95), 5);
  assert.equal(percentile(v, 0), 1);
  assert.equal(percentile([], 50), null);
});

test("帧间隔统计：刷新周期估计与超预算比例", () => {
  const ts = [0, 16.7, 33.4, 50.1, 100.1, 116.8];
  const deltas = frameDeltas(ts);
  assert.equal(deltas.length, 5);
  const s = summarizeFrames(deltas);
  assert.ok(Math.abs(s.refresh_estimate_ms - 16.7) < 1e-9);
  assert.equal(s.over_8_33_ratio, 1);
  assert.equal(s.over_16_67_ratio, 1);
  assert.equal(s.over_1_5x_refresh_ratio, 0.2);
  assert.equal(summarizeFrames([]).over_16_67_ratio, null);
});

test("交互记录：有输入时端到端从输入起算，缺失段为 null", () => {
  const rec = toInteractionRecord({
    id: 7,
    type: "autoStep",
    input_ts: 100,
    input_kind: "click",
    sent: 101,
    received: 110,
    received_epoch_ms: 5010,
    apply_start: 111,
    apply_end: 113,
    patched: 115,
    frame: 120,
    after_paint: 124,
    worker: {
      dispatch_ms: 2,
      view_json_ms: 1,
      view_parse_ms: 0.5,
      adapt_ms: 3,
      total_ms: 6.5,
      post_ms: 0.2,
      post_epoch_ms: 5008,
      view_json_chars: 900,
      envelope_json_chars: 2000,
    },
  });
  assert.equal(rec.input_to_send_ms, 1);
  assert.equal(rec.roundtrip_ms, 9);
  assert.equal(rec.transfer_ms, 2);
  assert.equal(rec.apply_ms, 2);
  assert.equal(rec.vue_patch_ms, 2);
  assert.equal(rec.patch_to_frame_ms, 5);
  assert.equal(rec.frame_to_paint_ms, 4);
  assert.equal(rec.input_to_paint_ms, 24);

  const auto = toInteractionRecord({
    id: 8,
    type: "autoStep",
    input_ts: null,
    input_kind: null,
    sent: 200,
    received: 205,
    received_epoch_ms: null,
    apply_start: null,
    apply_end: null,
    patched: null,
    frame: null,
    after_paint: null,
    worker: null,
  });
  assert.equal(auto.input_to_send_ms, null);
  assert.equal(auto.transfer_ms, null);
  assert.equal(auto.input_to_paint_ms, null);
  const summary = summarizeInteractions([rec, auto]);
  assert.equal(summary.roundtrip_ms.count, 2);
  assert.equal(summary.input_to_paint_ms.count, 1);
});

test("Worker 分段：关闭时只发原回包，开启时回包后追加分段", () => {
  const sent = [];
  const post = (m) => sent.push(m);
  setStepTimingsEnabled(false);
  stepBegin();
  stepMark("envelope_enter");
  postWithStepTimings(post, { id: 1, ok: true, data: { a: 1 } });
  assert.deepEqual(sent, [{ id: 1, ok: true, data: { a: 1 } }]);

  sent.length = 0;
  setStepTimingsEnabled(true);
  stepBegin();
  stepMark("envelope_enter");
  stepMark("view_json", 42);
  stepMark("view_parse");
  stepMark("envelope_done");
  postWithStepTimings(post, { id: 2, ok: true, data: { b: 2 } });
  assert.equal(sent.length, 2);
  assert.deepEqual(sent[0], { id: 2, ok: true, data: { b: 2 } });
  assert.equal(sent[1].type, "step_timings");
  assert.equal(sent[1].id, 2);
  const t = sent[1].timings;
  for (const key of ["dispatch_ms", "view_json_ms", "view_parse_ms", "adapt_ms", "total_ms", "post_ms"]) {
    assert.ok(typeof t[key] === "number" && t[key] >= 0, key);
  }
  assert.equal(t.view_json_chars, 42);
  assert.equal(t.envelope_json_chars, JSON.stringify({ b: 2 }).length);

  // 无 id 的诊断消息（如 init_timings）不追加分段
  sent.length = 0;
  postWithStepTimings(post, { type: "init_timings" });
  assert.equal(sent.length, 1);
  setStepTimingsEnabled(false);
});
