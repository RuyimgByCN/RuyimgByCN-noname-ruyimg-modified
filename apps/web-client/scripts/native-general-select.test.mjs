// NC05 前端选将接线回归：候选来自内核目录、按名装配走正式入口。
//
// 接线前（缺口证据见 evidence/.../repro-before.log）：`nativeBackend.ts` 与
// `engine.native.worker.ts` 完全没有 `general_candidates_json` / `assign_general`
// 面，前端唯一能改变座位技能的路径是建局随机装配——玩家无法选将。
//
// 本文件分两段：
//   A. 解析/边界（桩窄壳，无需 WASM）：畸形返回逐项报错、缺导出报错、缓存失效；
//   B. 真实 WASM（engine-native-wasm web 产物）：候选与装配对照真实内核。
// 运行：pnpm --dir apps/web-client test
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { test } from "node:test";
import { fileURLToPath, pathToFileURL } from "node:url";

import {
  chooseGeneral,
  generalCandidates,
  nativeCreate,
  nativeEnvelope,
  parseGeneralAssignmentJson,
  parseGeneralCandidatesJson,
} from "../src/worker/nativeBackend.ts";

const HERE = dirname(fileURLToPath(import.meta.url));
const PKG_DIR = join(HERE, "..", "src", "worker", "native-pkg");

const ACCEPTED = JSON.stringify({
  outcome: "accepted",
  revision: 3,
  finished: false,
  reason: null,
});

/** 桩 observe 视图：座位 0 的技能集从桩状态读，便于断言「装配后技能集变化」。 */
function viewJson(skills) {
  return JSON.stringify({
    seat: 0,
    players: [
      {
        seat: 0,
        alive: true,
        hp: 4,
        maxHp: 4,
        hand: { own: [11, 12] },
        handCount: 2,
        equips: [],
        judge: [],
        turnedOver: false,
        skills,
        revealedGenerals: [],
      },
      {
        seat: 1,
        alive: true,
        hp: 4,
        maxHp: 4,
        hand: { hidden: true },
        handCount: 4,
        equips: [],
        judge: [],
        turnedOver: false,
        skills: [],
        revealedGenerals: [],
      },
    ],
    deckCount: 100,
    discard: [],
    table: [],
    turn: 0,
    phase: "play",
    round: 1,
    outcome: null,
  });
}

/** 固定目录（桩）：def/name/skills 与内核投影同形。 */
const STUB_CANDIDATES = {
  count: 2,
  candidates: [
    { def: 0, name: "甲", skills: ["鬼道", "天妒"] },
    { def: 1, name: "乙", skills: ["奸雄"] },
  ],
};

/**
 * 窄壳桩：`assign_general` **不推进 revision**（真实内核实测如此），只改技能集
 * 与状态摘要——这正是「缓存必须显式失效」的根因。记录每次权威查询次数。
 */
function stubHandle({ assign } = {}) {
  const calls = [];
  const state = { revision: 3, skills: [], hash: "aa".repeat(32), queries: 0 };
  const engine = {
    revision: () => BigInt(state.revision),
    is_finished: () => false,
    view_json: () => viewJson(state.skills),
    open_choice_json: () => "null",
    next_phase: () => ACCEPTED,
    auto_step: () => ACCEPTED,
    activate_skill: () => ACCEPTED,
    legal_actions_json: (seat) => {
      state.queries += 1;
      calls.push({ call: "legal_actions_json", seat, revision: state.revision });
      return JSON.stringify({
        seat,
        revision: state.revision,
        actions: state.skills.map((name, skill) => ({
          ActivateSkill: { skill, targets: [], card_targets: [] },
        })),
      });
    },
    general_candidates_json: () => JSON.stringify(STUB_CANDIDATES),
    assign_general: (seat, name) => {
      calls.push({ call: "assign_general", seat, name });
      if (assign) return assign(state, seat, name);
      const row = STUB_CANDIDATES.candidates.find((c) => c.name === name);
      if (!row) throw "assign_general 被拒绝: NoSuchGeneral";
      state.skills = [...row.skills];
      state.hash = String(seat) + "b".repeat(63);
      return JSON.stringify({
        seat,
        name,
        revision: state.revision,
        state_hash: state.hash,
      });
    },
  };
  return {
    handle: { engine, seed: 1, players: 2, observer: 0 },
    calls,
    state,
    commands: () => calls.filter((c) => c.call === "assign_general"),
  };
}

// ───────────────────────── A. 解析与边界（桩） ─────────────────────────

test("候选解析：内核形态通过，逐项结构化且不改写字段", () => {
  const raw = JSON.stringify({
    count: 2,
    candidates: [
      { def: 7, name: "司马懿", skills: ["鬼才", "反馈"] },
      { def: 8, name: "无技能将", skills: [] },
    ],
  });
  const parsed = parseGeneralCandidatesJson(raw);
  assert.equal(parsed.count, 2);
  assert.deepEqual(parsed.candidates, [
    { def: 7, name: "司马懿", skills: ["鬼才", "反馈"] },
    { def: 8, name: "无技能将", skills: [] },
  ]);
  // 返回的是副本：改动结果不得回写解析输入
  parsed.candidates[0].skills.push("注入");
  assert.deepEqual(parseGeneralCandidatesJson(raw).candidates[0].skills, ["鬼才", "反馈"]);
});

test("候选解析：畸形返回逐项显式报错，不静默过滤成更小候选集", () => {
  // 非 JSON
  assert.throws(() => parseGeneralCandidatesJson("<html>oops"), /非 JSON/);
  // 顶层结构：null / 数组 / 缺字段 / count 类型错
  assert.throws(() => parseGeneralCandidatesJson("null"), /结构不符/);
  assert.throws(() => parseGeneralCandidatesJson("[]"), /结构不符/);
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 0 })),
    /结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ candidates: [] })),
    /结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: "2", candidates: [] })),
    /结构不符/,
  );
  // 项结构：缺字段 / 类型错 / 空名字 / def 非整数 / def 负数 / skills 非字符串
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 1, candidates: [{ def: 0, skills: [] }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 1, candidates: [{ def: 0, name: "甲" }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 1, candidates: [{ def: 0, name: "", skills: [] }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 1, candidates: [{ def: 1.5, name: "甲", skills: [] }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 1, candidates: [{ def: -1, name: "甲", skills: [] }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 1, candidates: [{ def: 0, name: "甲", skills: [1] }] })),
    /第 0 项结构不符/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 2, candidates: ["甲"] })),
    /第 0 项结构不符/,
  );
  // 项键冲突与计数不一致：坏数据的两种「看不出来」形态
  assert.throws(
    () =>
      parseGeneralCandidatesJson(
        JSON.stringify({
          count: 2,
          candidates: [
            { def: 3, name: "甲", skills: [] },
            { def: 3, name: "乙", skills: [] },
          ],
        }),
      ),
    /第 1 项 def=3 与前面重复/,
  );
  assert.throws(
    () => parseGeneralCandidatesJson(JSON.stringify({ count: 9, candidates: STUB_CANDIDATES.candidates })),
    /count=9 与候选数 2 不一致/,
  );
});

test("选将返回解析：与请求不一致即报错（错座位/错名字不算成功）", () => {
  const ok = parseGeneralAssignmentJson(
    JSON.stringify({ seat: 2, name: "甲", revision: 5, state_hash: "ab".repeat(32) }),
    { seat: 2, name: "甲" },
  );
  assert.deepEqual(ok, { seat: 2, name: "甲", revision: 5, state_hash: "ab".repeat(32) });

  const req = { seat: 2, name: "甲" };
  assert.throws(() => parseGeneralAssignmentJson("nope", req), /非 JSON/);
  assert.throws(() => parseGeneralAssignmentJson("null", req), /结构不符/);
  assert.throws(
    () => parseGeneralAssignmentJson(JSON.stringify({ seat: 1, name: "甲", revision: 5, state_hash: "ab" }), req),
    /与请求不一致/,
  );
  assert.throws(
    () => parseGeneralAssignmentJson(JSON.stringify({ seat: 2, name: "乙", revision: 5, state_hash: "ab" }), req),
    /与请求不一致/,
  );
  assert.throws(
    () => parseGeneralAssignmentJson(JSON.stringify({ seat: 2, name: "甲", state_hash: "ab" }), req),
    /与请求不一致/,
  );
  assert.throws(
    () => parseGeneralAssignmentJson(JSON.stringify({ seat: 2, name: "甲", revision: 5, state_hash: "" }), req),
    /与请求不一致/,
  );
  assert.throws(
    () => parseGeneralAssignmentJson(JSON.stringify({ seat: 2, name: "甲", revision: 5.5, state_hash: "zz" }), req),
    /与请求不一致/,
  );
});

test("旧 wasm 产物缺导出：显式报错 + build:native，不静默降级", () => {
  const stale = stubHandle();
  delete stale.handle.engine.general_candidates_json;
  delete stale.handle.engine.assign_general;
  assert.throws(
    () => generalCandidates(stale.handle),
    /缺少 general_candidates_json\/assign_general[\s\S]*build:native/,
    "候选面缺导出必须给出重建命令",
  );
  assert.throws(
    () => chooseGeneral(stale.handle, 0, "甲"),
    /缺少 general_candidates_json\/assign_general[\s\S]*build:native/,
    "选将面缺导出必须给出重建命令",
  );
  assert.deepEqual(stale.commands(), [], "缺导出时不得调用任何内核方法");

  // 只缺一个（半旧产物）同样报错并点名缺的那个：产物是整体过期的判据，
  // 不允许「候选面还能用就先凑合」——那会让界面在选将时才发现坏掉。
  const half = stubHandle();
  delete half.handle.engine.assign_general;
  assert.throws(() => chooseGeneral(half.handle, 0, "甲"), /缺少 assign_general\b/);
  assert.throws(() => generalCandidates(half.handle), /缺少 assign_general\b/);
  assert.deepEqual(half.commands(), [], "缺导出时不得调用任何内核方法");
});

test("选将失败：原始错误文本保留，且不改缓存（状态未变）", () => {
  const stub = stubHandle();
  nativeEnvelope(stub.handle);
  assert.equal(stub.state.queries, 1);

  assert.throws(
    () => chooseGeneral(stub.handle, 0, "查无此将"),
    /native: 选将失败\(seat=0, name=查无此将\): assign_general 被拒绝: NoSuchGeneral/,
  );
  assert.deepEqual(stub.commands(), [{ call: "assign_general", seat: 0, name: "查无此将" }]);
  nativeEnvelope(stub.handle);
  assert.equal(stub.state.queries, 1, "被拒绝的选将不改状态 → 缓存仍有效");
});

test("选将成功：技能集可见变化，且 revision 不变时缓存必须失效", () => {
  const stub = stubHandle();
  const before = nativeEnvelope(stub.handle);
  assert.deepEqual(before.legal_actions, [], "装配前无技能 → 无主动技候选");
  nativeEnvelope(stub.handle);
  assert.equal(stub.state.queries, 1, "同一修订号只查一次内核");

  const revisionBefore = stub.state.revision;
  const assignment = chooseGeneral(stub.handle, 0, "甲");
  assert.deepEqual(assignment, {
    seat: 0,
    name: "甲",
    revision: revisionBefore,
    state_hash: "0" + "b".repeat(63),
  });
  assert.equal(stub.state.revision, revisionBefore, "桩与真实内核一致：装配不推进 revision");

  const after = nativeEnvelope(stub.handle);
  assert.equal(stub.state.queries, 2, "装配改了状态却没推进 revision → 必须显式失效");
  assert.deepEqual(
    after.view.seats[0].skills.map((s) => s.name).sort(),
    ["天妒", "鬼道"],
    "该座位技能集变化对界面可见",
  );
  assert.deepEqual(
    after.legal_actions,
    [
      { ActivateSkill: { skill: 0, targets: [], card_targets: [] } },
      { ActivateSkill: { skill: 1, targets: [], card_targets: [] } },
    ],
    "候选按装配后的技能集重查",
  );
});

test("选将已生效但返回畸形：报错同时保持缓存失效（不放旧候选回界面）", () => {
  const stub = stubHandle({
    assign: (state, seat, name) => {
      state.skills = ["奸雄"];
      state.hash = "cc".repeat(32);
      return JSON.stringify({ seat, name, revision: state.revision }); // 缺 state_hash
    },
  });
  nativeEnvelope(stub.handle);
  assert.equal(stub.state.queries, 1);
  assert.throws(
    () => chooseGeneral(stub.handle, 0, "乙"),
    /已生效但返回无法解析[\s\S]*结构不符或与请求不一致/,
  );
  nativeEnvelope(stub.handle);
  assert.equal(stub.state.queries, 2, "解析失败也必须失效：状态已经变了");
});

test("候选面逐项来自内核：适配层不裁剪、不排序、不补默认将", () => {
  const stub = stubHandle();
  const parsed = generalCandidates(stub.handle);
  assert.deepEqual(parsed, STUB_CANDIDATES, "逐项等于内核投影，顺序原样");
  assert.deepEqual(stub.calls, [], "候选查询只读：不触发任何其它内核调用");
});

// ───────────────────────── B. 真实 WASM 引擎 ─────────────────────────

let wasmReady = null;

/** web 目标产物在 Node 下用字节初始化（与 native-worker-smoke.mjs 同法）。 */
async function ensureWasm() {
  if (!wasmReady) {
    wasmReady = (async () => {
      const mod = await import(pathToFileURL(join(PKG_DIR, "engine_native_wasm.js")).href);
      await mod.default({
        module_or_path: readFileSync(join(PKG_DIR, "engine_native_wasm_bg.wasm")),
      });
    })();
  }
  await wasmReady;
}

test("真实 WASM：候选来自内核目录，且逐项通过适配层结构校验", async () => {
  await ensureWasm();
  const handle = nativeCreate(4, 4242, 0);
  const parsed = generalCandidates(handle);
  assert.ok(parsed.count > 0, `内核候选不得为空（count=${parsed.count}）`);
  assert.equal(parsed.count, parsed.candidates.length);
  assert.equal(
    new Set(parsed.candidates.map((c) => c.def)).size,
    parsed.count,
    "def 唯一",
  );
  for (const row of parsed.candidates) {
    assert.ok(Number.isInteger(row.def) && row.def >= 0, `def 非法: ${JSON.stringify(row)}`);
    assert.ok(typeof row.name === "string" && row.name.length > 0, `name 非法: ${JSON.stringify(row)}`);
    assert.ok(Array.isArray(row.skills) && row.skills.every((s) => typeof s === "string"));
  }
  // 候选定义与视图技能集同源：按名字装配后视图技能必须是该候选的技能集
  const withSkills = parsed.candidates.find((c) => c.skills.length > 0);
  assert.ok(withSkills, "目录里应有带技能的武将");
  const assignment = chooseGeneral(handle, 0, withSkills.name);
  assert.equal(assignment.seat, 0);
  assert.equal(assignment.name, withSkills.name);
  const view = JSON.parse(handle.engine.view_json(0));
  assert.deepEqual(
    view.players[0].skills.slice().sort(),
    withSkills.skills.slice().sort(),
    `装配后技能集必须等于候选技能集（view=${JSON.stringify(view.players[0].skills)}）`,
  );
  assert.equal(view.players[0].maxHp, view.players[0].hp, "装配回满");
});

test("真实 WASM：装配不改 revision，状态变化只体现在 state_hash", async () => {
  await ensureWasm();
  const handle = nativeCreate(4, 4242, 0);
  const { candidates } = generalCandidates(handle);
  const first = candidates.find((c) => c.skills.length > 0);
  const second = candidates.find(
    (c) => c.skills.length > 0 && c.name !== first.name,
  );
  assert.ok(second, "目录里应有两个带技能的不同武将");

  const revisionBefore = Number(handle.engine.revision());
  const a = chooseGeneral(handle, 0, first.name);
  const b = chooseGeneral(handle, 1, second.name);
  assert.equal(a.revision, revisionBefore);
  assert.equal(b.revision, revisionBefore);
  assert.equal(Number(handle.engine.revision()), revisionBefore, "revision 不因选将推进");
  assert.match(a.state_hash, /^[0-9a-f]{64}$/);
  assert.match(b.state_hash, /^[0-9a-f]{64}$/);
  assert.notEqual(a.state_hash, b.state_hash, "不同装配必须给出不同状态摘要");
  const views = JSON.parse(handle.engine.view_json(0)).players;
  assert.deepEqual(views[0].skills.slice().sort(), first.skills.slice().sort());
  assert.deepEqual(views[1].skills.slice().sort(), second.skills.slice().sort());
});

test("真实 WASM：未知武将与越界座位显式报错，且不改状态", async () => {
  await ensureWasm();
  const handle = nativeCreate(4, 4242, 0);
  const before = handle.engine.view_json(0);
  assert.throws(
    () => chooseGeneral(handle, 0, "查无此将"),
    (error) =>
      /native: 选将失败\(seat=0, name=查无此将\)/.test(error.message) &&
      /NoSuchGeneral/.test(error.message),
    "未知武将必须带上原始拒绝原因",
  );
  const name = generalCandidates(handle).candidates[0].name;
  assert.throws(
    () => chooseGeneral(handle, 9, name),
    /native: 选将失败\(seat=9[\s\S]*座位越界/,
    "越界座位必须报错而不是静默装配到别的座位",
  );
  assert.equal(
    handle.engine.view_json(0),
    before,
    "被拒绝的选将不得改变状态（视图逐字段一致）",
  );
  // 失败后仍可正常选将（状态没被写坏）
  assert.equal(chooseGeneral(handle, 0, name).name, name);
});

test("真实 WASM：选将后权威候选重查（缓存不再命中）", async () => {
  await ensureWasm();
  const handle = nativeCreate(4, 4242, 0);
  let queries = 0;
  const original = handle.engine.legal_actions_json.bind(handle.engine);
  handle.engine.legal_actions_json = (seat) => {
    queries += 1;
    return original(seat);
  };
  nativeEnvelope(handle);
  nativeEnvelope(handle);
  assert.equal(queries, 1, "同一修订号记忆化只查一次");
  const name = generalCandidates(handle).candidates.find((c) => c.skills.length > 0).name;
  chooseGeneral(handle, 0, name);
  nativeEnvelope(handle);
  assert.equal(queries, 2, "选将后必须重查内核（revision 未变，只能靠显式失效）");
});
