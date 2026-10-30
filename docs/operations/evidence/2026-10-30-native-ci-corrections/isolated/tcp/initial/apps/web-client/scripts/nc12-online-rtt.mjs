// NC12 联机往返时延（RTT）：Node 原生 WebSocket 客户端直连 game-server，测量
// 「提交命令 → 收到 CommandOutcome」的墙钟时间（p50/p95/max）。
//
// 前置：game-server 在 127.0.0.1:7700（NONAME_NATIVE_ROOMS=1，含 shahai-swd 包）。
// 命令：node apps/web-client/scripts/nc12-online-rtt.mjs [iters] [ws-url]
//
// 口径：本机回环（无真实网络）；绝对值受负载影响，只有 p50 可比较；不含客户端渲染与动画。

const iters = Number(process.argv[2] ?? 30);
const wsUrl = process.argv[3] ?? "ws://127.0.0.1:7700/ws";

const socket = new WebSocket(wsUrl);
const pending = new Map();
let seq = 0;

function send(type, payload) {
  seq += 1;
  const id = seq;
  socket.send(JSON.stringify({ type, payload }));
  return new Promise((resolve, reject) => pending.set(id, { resolve, reject }));
}
// 服务器消息按类型匹配（协议无请求 id 回显，按序等待特定类型）
const waiters = [];
socket.onmessage = (ev) => {
  const msg = JSON.parse(String(ev.data));
  for (let i = 0; i < waiters.length; i++) {
    if (waiters[i].match(msg)) {
      const w = waiters.splice(i, 1)[0];
      w.resolve(msg);
      return;
    }
  }
};
function waitFor(match, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const w = { match, resolve };
    waiters.push(w);
    setTimeout(() => {
      const i = waiters.indexOf(w);
      if (i >= 0) {
        waiters.splice(i, 1);
        reject(new Error("等待超时"));
      }
    }, timeoutMs);
  });
}
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

await new Promise((r, j) => { socket.onopen = r; socket.onerror = j; });
const welcome = waitFor((m) => m.type === "Welcome");
send("Hello", {
  versions: {
    engine_api: 1,
    rule_vm_abi: 167,
    content_schema: 3,
    network_protocol: 6,
    replay_schema: 1,
    snapshot_schema: 1,
    package_format: 1,
    engine_build_id: "nc12-rtt",
  },
  display_name: "rtt-probe",
});
await welcome;
console.log("STEP hello ok");

const created = waitFor((m) => m.type === "JoinOk");
send("CreateRoom", { players: 2, seed: 20262800, password: "" });
const join = await created;
let roomId = join.payload?.session?.room_id ?? join.payload?.room_id ?? 0;
console.log("STEP room " + roomId);

function pct(samples, p) {
  const v = [...samples].sort((a, b) => a - b);
  return v[Math.min(v.length - 1, Math.round((v.length - 1) * p))];
}

// 第二名玩家入座（房间满座后才接受对局命令）
const socket2 = new WebSocket(wsUrl);
const waiters2 = [];
socket2.onmessage = (ev) => {
  const msg = JSON.parse(String(ev.data));
  for (let i = 0; i < waiters2.length; i++) {
    if (waiters2[i].match(msg)) {
      const w = waiters2.splice(i, 1)[0];
      w.resolve(msg);
      return;
    }
  }
};
function waitFor2(match, timeoutMs = 10000) {
  return new Promise((resolve, reject) => {
    const w = { match, resolve };
    waiters2.push(w);
    setTimeout(() => {
      const i = waiters2.indexOf(w);
      if (i >= 0) {
        waiters2.splice(i, 1);
        reject(new Error("等待超时(socket2)"));
      }
    }, timeoutMs);
  });
}
await new Promise((r, j) => { socket2.onopen = r; socket2.onerror = j; });
const welcome2 = waitFor2((m) => m.type === "Welcome");
socket2.send(
  JSON.stringify({
    type: "Hello",
    payload: {
      versions: {
        engine_api: 1,
        rule_vm_abi: 167,
        content_schema: 3,
        network_protocol: 6,
        replay_schema: 1,
        snapshot_schema: 1,
        package_format: 1,
        engine_build_id: "nc12-rtt-2",
      },
      display_name: "rtt-probe-2",
    },
  }),
);
await welcome2;
const joined2 = waitFor2((m) => m.type === "JoinOk" || m.type === "Error");
socket2.send(
  JSON.stringify({
    type: "JoinRoom",
    payload: { room_id: roomId, password: "", preferred_seat: 1, reconnect_token: null },
  }),
);
const j2 = await joined2;
console.log("STEP join2 " + j2.type);

// 原生房间**建局即开局**，无需 StartGame（发送会因座位数不一致被拒）。
// 每 4 条命令换一个新房间：避免对局进入需要 `ResolveChoice` 的阶段（本探针只测
// 「提交 → 回包」往返，不模拟选择应答）；换房后仍走完整 WS 往返，不引入额外等待。
const rtts = [];
let revision = 0;
let accepted = 0;
let rejected = 0;
let roomSeq = 0;
for (let i = 0; i < iters; i++) {
  if (i > 0 && i % 4 === 0) {
    const again = waitFor((m) => m.type === "JoinOk" || m.type === "Error");
    send("CreateRoom", { players: 2, seed: 20262800 + i, password: "" });
    const j = await again;
    roomSeq += 1;
    // 新房间号必须更新：否则后续命令仍打到旧房间（会被拒）
    roomId = j.payload?.session?.room_id ?? j.payload?.room_id ?? roomId;
    if (j.type === "Error") {
      console.log("STEP recreate_error " + JSON.stringify(j.payload ?? {}).slice(0, 160));
    }
    revision = 0;
  }
  const outcome = waitFor((m) => m.type === "CommandOutcome" || m.type === "Error");
  const header = { protocol_version: 1, room_id: roomId, conn_seq: i + 1, revision };
  const t0 = process.hrtime.bigint();
  send("SubmitCommand", {
    header,
    command: {
      actor: 1,
      expected_revision: revision,
      idempotency_key: i + 1,
      kind: "EndPhase", // 单元变体：serde 外部标签枚举用字符串（{EndPhase:{}} 会 BadMessage）
    },
    idempotency_key: i + 1,
  });
  const msg = await outcome;
  const dt = Number(process.hrtime.bigint() - t0);
  const result = msg.payload?.result ?? {};
  // 只把**被接受**的命令计入时延（被拒命令走快速失败路径，不是本指标口径）
  if (result.Accepted) rtts.push(dt);
  if (result.Accepted) accepted += 1;
  else {
    rejected += 1;
    if (rejected === 1) {
      console.log("STEP first_non_accepted " + JSON.stringify(msg).slice(0, 260));
    }
  }
  revision = msg.payload?.header?.revision ?? revision;
}
console.log(
  `RESULT scenario=online_rtt tier=loopback n=${rtts.length} p50_ns=${pct(rtts, 0.5)} p95_ns=${pct(rtts, 0.95)} max_ns=${Math.max(...rtts)} mean_ns=${Math.round(rtts.reduce((a, b) => a + b, 0) / rtts.length)}`,
);
console.log(
  `RESULT scenario=online_rtt tier=loopback result_accepted=${accepted} result_rejected=${rejected} final_revision=${revision}`,
);
console.log("NOTE 本机回环、无真实网络；不含客户端渲染/动画；绝对值受负载影响。");
socket.close();
