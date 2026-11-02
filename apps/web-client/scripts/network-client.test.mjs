import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import ts from "typescript";
import { computed, ref } from "vue";
import { matchOutcomePresentation } from "../src/engine/outcome.ts";
import { NetworkEngineClient, onlineReconnectKey } from "../src/engine/networkClient.ts";
import { applyEnvelope, clearEffects, createViewStore } from "../src/engine/viewStore.ts";

function fixture(t) {
  const previous = { WebSocket: globalThis.WebSocket, window: globalThis.window };
  const sockets = [];
  const timers = new Map();
  let timerId = 0;
  class Socket {
    static OPEN = 1;
    readyState = 0;
    sent = [];
    constructor(url) { this.url = url; sockets.push(this); }
    send(json) {
      if (this.sendError) throw new Error("send failed");
      this.sent.push(JSON.parse(json));
    }
    open(id = sockets.length) {
      this.readyState = 1;
      this.onopen?.();
      this.receive("Welcome", { connection_id: id, protocol_version: 7, versions: { network_protocol: 7 } });
    }
    receive(type, payload) { this.onmessage?.({ data: JSON.stringify({ type, payload }) }); }
    close() { this.readyState = 3; this.onclose?.(); }
  }
  globalThis.WebSocket = Socket;
  globalThis.window = {
    setInterval: () => ++timerId,
    clearInterval: () => {},
    setTimeout: (callback) => { timers.set(++timerId, callback); return timerId; },
    clearTimeout: (id) => timers.delete(id),
  };
  const client = new NetworkEngineClient();
  t.after(() => {
    client.dispose();
    globalThis.WebSocket = previous.WebSocket;
    globalThis.window = previous.window;
  });
  return {
    client, sockets,
    async connect() {
      const pending = client.connect("http://server/ws", "player");
      const socket = sockets.at(-1);
      socket.open();
      await pending;
      assert.equal(socket.url, "ws://server/ws");
      return socket;
    },
    reconnect() {
      const entry = timers.entries().next().value;
      assert.ok(entry, "disconnect schedules reconnect");
      timers.delete(entry[0]);
      entry[1]();
      return sockets.at(-1);
    },
  };
}

function view(revision = 1) {
  return {
    revision, state_hash: Array(32).fill(revision), observer: { Player: 1 },
    round: 1, current_player: 1, phase: "Play", seats: [], known_cards: [], outcome: null,
  };
}

function join(socket, { role = "Player", token = "token-1", choice = null, actions = [] } = {}) {
  socket.receive("JoinOk", {
    room_id: 7, player_id: role === "Player" ? 1 : 0, seat: role === "Player" ? 0 : 255,
    role, reconnect_token: token, view: view(), choice, legal_actions: actions,
  });
}

test("JoinOk、推送与快照保留权威 choice 和合法答案，观战空选择不合成", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const choice = { choice_id: 8, observer: 1, options: [{ token: 1, label_key: "pass" }] };
  const answer = { ResolveChoice: { choice_id: 8, selections: [1], cancelled: false } };
  const pending = client.joinRoom(7);
  join(socket, { choice, actions: [answer] });
  assert.deepEqual((await pending).envelope.choice, choice);
  const envelopes = [];
  client.onEnvelope((envelope) => envelopes.push(envelope));
  socket.receive("CommandOutcome", {
    view: view(2), choice, legal_actions: [answer], result: { Accepted: {} }, idempotency_key: 99,
  });
  assert.deepEqual(envelopes.at(-1).legal_actions, [answer]);
  const snapshot = client.requestSnapshot();
  const request = socket.sent.at(-1);
  socket.receive("Snapshot", { header: request.payload.header, view: view(2), choice: null, legal_actions: [] });
  assert.equal((await snapshot).revision, 2);
  assert.equal(envelopes.at(-1).choice, null);
  assert.deepEqual(envelopes.at(-1).legal_actions, []);
});

test("拒绝提交仍同步最新投影与 choice，再向调用方显示拒绝", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const joined = client.joinRoom(7);
  join(socket);
  await joined;
  const envelopes = [];
  client.onEnvelope((envelope) => envelopes.push(envelope));
  const command = client.endPhase();
  const request = socket.sent.at(-1);
  const choice = { choice_id: 9, observer: 1, options: [] };
  socket.receive("CommandOutcome", {
    view: view(2), choice, legal_actions: [], idempotency_key: request.payload.idempotency_key,
    result: { Rejected: { StaleRevision: { current: 2, expected: 1 } } },
  });
  await assert.rejects(command, /StaleRevision/);
  assert.equal(envelopes.at(-1)?.revision, 2);
  assert.deepEqual(envelopes.at(-1)?.choice, choice);
});

test("断线后复用房间密码与最新令牌，重新入房前禁止提交", async (t) => {
  const f = fixture(t);
  const socket = await f.connect();
  const joined = f.client.createRoom(4, 42, "secret");
  join(socket);
  await joined;
  socket.close();
  await assert.rejects(f.client.endPhase(), /入座|连接/);
  const replacement = f.reconnect();
  replacement.open(2);
  await new Promise((resolve) => setImmediate(resolve));
  await assert.rejects(f.client.endPhase(), /入座/);
  const rejoined = f.client.rejoinRoom();
  assert.deepEqual(replacement.sent.at(-1), {
    type: "JoinRoom", payload: { room_id: 7, password: "secret", preferred_seat: null, reconnect_token: "token-1" },
  });
  join(replacement, { token: "token-2" });
  await rejoined;
  const again = f.client.rejoinRoom();
  assert.equal(replacement.sent.at(-1).payload.reconnect_token, "token-2");
  join(replacement, { token: "token-2" });
  await again;
});

test("旁观重连仍发送 Spectate，不取得玩家身份", async (t) => {
  const f = fixture(t);
  const socket = await f.connect();
  const joined = f.client.spectate(7);
  join(socket, { role: "Spectator" });
  await joined;
  socket.close();
  const replacement = f.reconnect();
  replacement.open(2);
  await new Promise((resolve) => setImmediate(resolve));
  const rejoined = f.client.rejoinRoom();
  assert.deepEqual(replacement.sent.at(-1), { type: "Spectate", payload: { room_id: 7 } });
  join(replacement, { role: "Spectator" });
  assert.equal((await rejoined).playerId, 0);
});

test("入房/快照 send 失败释放 waiter，重复入房不多发请求", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  socket.sendError = true;
  await assert.rejects(Promise.resolve().then(() => client.joinRoom(7)), /send failed/);
  await assert.rejects(Promise.resolve().then(() => client.requestSnapshot()), /send failed/);
  socket.sendError = false;
  const joined = client.joinRoom(7);
  const count = socket.sent.length;
  await assert.rejects(client.spectate(7), /已有入房/);
  assert.equal(socket.sent.length, count);
  join(socket);
  await joined;
});

test("dispose 后的旧 socket 关闭不影响新的连接，销毁不保留监听器", async (t) => {
  const f = fixture(t);
  const old = await f.connect();
  let closes = 0;
  f.client.onClose(() => closes++);
  f.client.dispose();
  closes = 0;
  const fresh = await f.connect();
  old.onclose?.();
  const joined = f.client.spectate(7);
  join(fresh, { role: "Spectator" });
  await joined;
  assert.equal(closes, 0);
});

test("重连令牌按实际 WS origin 与房号隔离，HTTP 输入规范化为同一服务", () => {
  const key = onlineReconnectKey("http://localhost:7700/ws", 7);
  assert.equal(key, onlineReconnectKey("ws://LOCALHOST:7700/ws", 7));
  assert.notEqual(key, onlineReconnectKey("ws://localhost:7701/ws", 7));
  assert.notEqual(key, onlineReconnectKey("ws://localhost:7700/ws", 8));
  assert.notEqual(key, onlineReconnectKey("wss://localhost:7700/ws", 7));
});

// 执行 SFC 内的实际函数与 Vue reactive store；只替换表现延迟，不需要 DOM 框架。
function presentationFixture() {
  const sfc = readFileSync(new URL("../src/views/BattleView.vue", import.meta.url), "utf8");
  const script = sfc.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile("BattleView.ts", script, ts.ScriptTarget.Latest, true);
  const declaration = ast.statements.find((node) =>
    ts.isFunctionDeclaration(node) && node.name?.text === "handleEnvelope");
  assert.ok(declaration, "SFC still defines its shared envelope handler");
  const source = ts.createPrinter().printNode(ts.EmitHint.Unspecified, declaration, ast);
  const compiled = ts.transpileModule(source, {
    compilerOptions: { target: ts.ScriptTarget.ES2022 },
  }).outputText;
  const store = createViewStore();
  store.connected = true;
  const status = { value: "connected" };
  let release;
  const handle = new Function(
    "store", "applyEnvelope", "nc12Mark", "presentationDelay", "clearEffects", "isOnline",
    "roomId", "statusLine", "localClient", "syncGeneralSelectProjection", "matchOutcome",
    `${compiled}; return handleEnvelope;`,
  )(
    store, applyEnvelope, () => {}, () => new Promise((resolve) => { release = resolve; }),
    clearEffects, { value: true }, { value: 7 }, status, { isNative: () => true }, () => {},
    computed(() => matchOutcomePresentation(store.view, store.lastEvents, 1)),
  );
  const env = (revision) => ({
    result: null, events: [], choice: null, view: view(revision), legal_actions: [],
    hash_hex: String(revision), revision, finished: false,
  });
  return { store, status, handle, env, release: () => release() };
}

test("表现延迟期间断线：旧信封不能覆盖重连提示", async () => {
  const f = presentationFixture();
  const pending = f.handle(f.env(2));
  f.store.connected = false;
  f.status.value = "连接已断开，正在重连…";
  f.release();
  await pending;
  assert.equal(f.status.value, "连接已断开，正在重连…");
  f.store.connected = true;
  const current = f.handle(f.env(3));
  f.release();
  await current;
  assert.match(f.status.value, /rev 3/);
});

test("表现延迟期间新投影到达：旧信封不能覆盖新状态提示或清掉新效果", async () => {
  const f = presentationFixture();
  const pending = f.handle(f.env(2));
  applyEnvelope(f.store, f.env(3));
  f.status.value = "房间 #7 · rev 3";
  f.store.effects = [{ id: 1, type: "draw", text: "新效果" }];
  f.release();
  await pending;
  assert.equal(f.status.value, "房间 #7 · rev 3");
  assert.equal(f.store.effects.length, 1);
});

test("断线保留待确认命令，重连后用原 key 重放并由服务端回执结算", async (t) => {
  const f = fixture(t);
  const socket = await f.connect();
  const joined = f.client.joinRoom(7);
  join(socket);
  await joined;

  let settled = false;
  const command = f.client.endPhase().then(
    (value) => {
      settled = true;
      return value;
    },
    (error) => {
      settled = true;
      throw error;
    }
  );
  const request = socket.sent.at(-1);
  assert.equal(request.type, "SubmitCommand");
  const key = request.payload.idempotency_key;
  assert.ok(key > 0);
  assert.equal(request.payload.command.kind, "EndPhase");

  socket.close();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(settled, false, "ACK 丢失时不得把未确认提交判成失败");

  const replacement = f.reconnect();
  replacement.open(2);
  await new Promise((resolve) => setImmediate(resolve));
  const rejoined = f.client.rejoinRoom();
  join(replacement, { token: "token-2" });
  await rejoined;

  const replayIndex = replacement.sent.findIndex((m) => m.type === "SubmitCommand");
  assert.ok(replayIndex >= 0, "重连后必须重放待确认命令");
  const joinIndex = replacement.sent.findIndex((m) => m.type === "JoinRoom");
  assert.ok(joinIndex >= 0 && replayIndex > joinIndex, "重放必须发生在重新入房之后");
  const replay = replacement.sent[replayIndex];
  assert.equal(replay.payload.idempotency_key, key, "重放必须沿用原 key");
  assert.equal(replay.payload.command.idempotency_key, key);
  assert.equal(replay.payload.command.kind, "EndPhase", "重放不得改变命令内容");

  replacement.receive("CommandOutcome", {
    view: view(2),
    choice: null,
    legal_actions: [],
    result: { Accepted: {} },
    idempotency_key: key,
  });
  const envelope = await command;
  assert.equal(envelope.revision, 2);
  assert.equal(settled, true);
  const submitted = replacement.sent.filter(
    (m) => m.type === "SubmitCommand" && m.payload.idempotency_key === key
  );
  assert.equal(submitted.length, 1, "同一意图只应以原 key 重放一次");
});

test("自动重连耗尽后，待确认命令明确标为结果不确定而不是已撤销", async (t) => {
  const f = fixture(t);
  const socket = await f.connect();
  const joined = f.client.joinRoom(7);
  join(socket);
  await joined;
  let settled = false;
  const command = f.client.endPhase().catch((error) => {
    settled = true;
    return error;
  });
  socket.close();
  await new Promise((resolve) => setImmediate(resolve));
  assert.equal(settled, false);

  for (let attempt = 0; attempt < 5; attempt += 1) {
    const next = f.reconnect();
    next.close();
    await new Promise((resolve) => setImmediate(resolve));
  }
  const error = await command;
  assert.ok(error instanceof Error);
  assert.match(error.message, /不确定/);
});

test("刷新后的新客户端用原 key 重放未确认命令", async (t) => {
  const previousStorage = globalThis.sessionStorage;
  const store = new Map();
  globalThis.sessionStorage = {
    getItem: (key) => store.get(key) ?? null,
    setItem: (key, value) => { store.set(key, String(value)); },
    removeItem: (key) => { store.delete(key); },
  };
  t.after(() => {
    globalThis.sessionStorage = previousStorage;
    store.clear();
  });

  const first = fixture(t);
  const firstSocket = await first.connect();
  join(firstSocket);
  const pending = first.client.endPhase();
  pending.catch(() => undefined);
  const submitted = firstSocket.sent.find((message) => message.type === "SubmitCommand");
  assert.ok(submitted, "首个客户端应发出命令");
  assert.equal(submitted.payload.replay, false, "首次提交不是重放");
  const key = submitted.payload.idempotency_key;
  assert.equal(submitted.payload.command.kind, "EndPhase");

  // 模拟刷新：新客户端从 sessionStorage 读取同一 key/命令。
  const secondClient = new NetworkEngineClient();
  t.after(() => secondClient.dispose());
  const secondConnecting = secondClient.connect("http://server/ws", "player");
  const secondSocket = first.sockets.at(-1);
  secondSocket.open();
  await secondConnecting;
  join(secondSocket);
  const replayed = secondSocket.sent.find(
    (message) =>
      message.type === "SubmitCommand" &&
      message.payload.idempotency_key === key &&
      message.payload.command.kind === "EndPhase",
  );
  assert.ok(replayed, "刷新后的客户端必须用原 key 重放");
  assert.equal(replayed.payload.replay, true, "刷新重放必须标记 replay");
  assert.equal(secondSocket.sent.filter((m) => m.type === "SubmitCommand").length, 1);

  secondSocket.receive("CommandOutcome", {
    idempotency_key: key,
    view: view(2),
    choice: null,
    public_event_codes: [],
    legal_actions: [],
    result: { Accepted: {} },
  });
  assert.equal(store.size, 0, "回执结算后必须清除刷新持久化，避免下次重放");
});

test("取消待确认命令会清除刷新持久化", async (t) => {
  const previousStorage = globalThis.sessionStorage;
  const store = new Map();
  globalThis.sessionStorage = {
    getItem: (key) => store.get(key) ?? null,
    setItem: (key, value) => { store.set(key, String(value)); },
    removeItem: (key) => { store.delete(key); },
  };
  t.after(() => { globalThis.sessionStorage = previousStorage; store.clear(); });

  const f = fixture(t);
  const socket = await f.connect();
  join(socket);
  const pending = f.client.endPhase();
  pending.catch(() => undefined);
  assert.equal(store.size, 1, "发送后应持久化");
  f.client.cancelPendingCommands();
  assert.equal(store.size, 0, "取消后必须清除持久化");
});

test("过期刷新持久化不会重放陈旧命令", async (t) => {
  const previousStorage = globalThis.sessionStorage;
  const store = new Map();
  globalThis.sessionStorage = {
    getItem: (key) => store.get(key) ?? null,
    setItem: (key, value) => { store.set(key, String(value)); },
    removeItem: (key) => { store.delete(key); },
  };
  t.after(() => { globalThis.sessionStorage = previousStorage; store.clear(); });

  store.set("noname-next.pending-command", JSON.stringify({
    roomId: 7,
    key: 999,
    kind: "EndPhase",
    createdAt: Date.now() - 11 * 60 * 1000,
  }));
  const f = fixture(t);
  const socket = await f.connect();
  join(socket);
  assert.equal(
    socket.sent.filter((m) => m.type === "SubmitCommand").length,
    0,
    "超过 10 分钟的待确认命令不应重放",
  );
});

test("旧房间无回执的重放拒绝显示为结果不确定", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const joined = client.joinRoom(7);
  join(socket);
  await joined;
  const command = client.endPhase();
  const request = socket.sent.at(-1);
  socket.receive("CommandOutcome", {
    view: view(1),
    choice: null,
    legal_actions: [],
    idempotency_key: request.payload.idempotency_key,
    result: { Rejected: { IllegalAction: { code: "legacy_replay_no_receipt" } } },
  });
  await assert.rejects(command, /结果不确定/);
});

test("联机公开事件与本地一致地展示粗粒度语义", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const joined = client.joinRoom(7);
  join(socket);
  await joined;
  const envelopes = [];
  client.onEnvelope((envelope) => envelopes.push(envelope));
  socket.receive("CommandOutcome", {
    view: view(2), choice: null, legal_actions: [], result: { Accepted: {} },
    idempotency_key: 99, public_event_codes: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 99],
  });
  const labels = [
    "对局开始", "摸牌", "使用或响应牌", "伤害结算", "回复体力", "濒死状态变化",
    "玩家阵亡", "阶段或回合推进", "对局结束", "选择状态变化", "事件 99",
  ];
  assert.deepEqual(envelopes.at(-1).events.map((event) => event.PublicEvent.label), labels);
  const store = createViewStore();
  applyEnvelope(store, envelopes.at(-1));
  assert.deepEqual(store.log.map((event) => event.text), labels.toReversed());
});


test("network SingleWin stays one-based with all authoritative projection fields unchanged", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const joined = client.joinRoom(7);
  join(socket);
  await joined;
  const envelopes = [];
  client.onEnvelope((envelope) => envelopes.push(envelope));
  for (const player of [1, 4]) {
    const projection = {
      ...view(player + 1), outcome: { SingleWin: { player } },
      seats: [1, 2, 3, 4].map((id) => ({ player: id, seat: id - 1 })),
    };
    socket.receive("CommandOutcome", {
      view: projection, choice: null, legal_actions: [], result: { Accepted: {} }, idempotency_key: 99,
    });
    assert.deepEqual(envelopes.at(-1).view, projection);
    assert.equal(envelopes.at(-1).finished, true);
  }
});


test("finished status shares the banner's view and MatchEnded fallback wording", async () => {
  const f = presentationFixture();
  for (const event of [{ type: "MatchEnded", outcome: { RebelWin: null } },
    { MatchEnded: { outcome: { RebelWin: null } } }]) {
    const env = { ...f.env(4), finished: true, events: [event] };
    let pending = f.handle(env);
    f.release();
    await pending;
    assert.match(f.status.value, /终局 · 反贼胜 · rev 4/);
    env.view = { ...env.view, outcome: { SingleWin: { player: 1 } } };
    pending = f.handle(env);
    f.release();
    await pending;
    assert.match(f.status.value, /终局 · 单挑胜者 P1 · rev 4/);
  }
});

test("B metadata is preserved on real network consumer messages and malformed metadata refuses join", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const metadata = { def: 0, name: "曹操", faction: "wei", revealed: false };
  const pending = client.joinRoom(7);
  const snapshot = { ...view(), seats: [{ player: 1, character_def: 0, generals: [metadata, null] }] };
  const choice = { choice_id: 1, observer: 1, native_kind: "respond_card", prompt: "respond_shan", kind: "Custom" };
  socket.receive("JoinOk", { room_id: 7, player_id: 1, seat: 0, role: "Player", view: snapshot, choice, legal_actions: [] });
  const joined = await pending;
  assert.deepEqual(joined.envelope.view.seats[0].generals, [metadata, null]);
  assert.deepEqual(joined.envelope.choice, choice);
  const rejected = client.joinRoom(7);
  socket.receive("JoinOk", { view: snapshot, choice: { ...choice, native_kind: "invented_kind" } });
  await assert.rejects(rejected, /元数据无效/);
});

test("WS7 rejects an old Welcome before creating rooms", async (t) => {
  const { client, sockets } = fixture(t);
  const pending = client.connect("ws://localhost:7700/ws", "old-server");
  const socket = sockets.at(-1);
  socket.readyState = 1;
  socket.onopen?.();
  socket.receive("Welcome", { connection_id: 1, protocol_version: 5, versions: { network_protocol: 5 } });
  await assert.rejects(pending, /WS7/);
});

test("WS7 room configuration and deadline reach the actual network consumer", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const pending = client.createRoom(4, 42, "", 10);
  assert.equal(socket.sent.at(-1).payload.choice_timeout_secs, 10);
  const choice = { choice_id: 1, observer: 1, deadline_unix_ms: Date.now() + 10000 };
  socket.receive("JoinOk", { room_id: 7, player_id: 1, seat: 0, role: "Player", view: view(), choice, legal_actions: [] });
  assert.deepEqual((await pending).envelope.choice, choice);
  let roster;
  client.onRoster(value => { roster = value; });
  socket.receive("RoomRoster", { room_id: 7, choice_timeout_secs: 10, started: true, finished: false, seats: [{ seat: 0, player_id: 1, display_name: "p", connected: true, managed: true }] });
  assert.equal(roster.seats[0].managed, true);
  assert.equal(roster.choice_timeout_secs, 10);
});

function battleCountdown(client, deadline) {
  const sfc = readFileSync(new URL('../src/views/BattleView.vue', import.meta.url), 'utf8');
  const script = sfc.match(/<script setup lang="ts">([\s\S]*?)<\/script>/)[1];
  const ast = ts.createSourceFile('BattleView.ts', script, ts.ScriptTarget.Latest, true);
  const names = new Set(['countdownNow', 'countdownTick', 'countdownTimer', 'choiceSecondsLeft']);
  const declarations = ast.statements.filter(statement => ts.isVariableStatement(statement) &&
    statement.declarationList.declarations.some(decl => names.has(decl.name.getText(ast)))).map(statement => statement.getText(ast)).join('\n');
  let tick;
  const compiled = ts.transpileModule(declarations + '\nreturn choiceSecondsLeft;', {compilerOptions:{target:ts.ScriptTarget.ES2022}}).outputText;
  const value = new Function('ref', 'computed', 'store', 'networkClient', 'setInterval', compiled)(ref, computed,
    {choice: {deadline_unix_ms: deadline}}, client, callback => {tick=callback; return 1;});
  return {value, tick: () => tick()};
}

for (const skew of [60_000, -60_000]) {
  test(`server countdown ignores client wall clock skew ${skew} and later jumps`, async (t) => {
    let wall = 1_000_000 + skew;
    let monotonic = 100;
    t.mock.method(Date, 'now', () => wall);
    t.mock.method(performance, 'now', () => monotonic);
    const f = fixture(t);
    const socket = await f.connect();
    const probe = socket.sent.find(message => message.type === 'Heartbeat');
    socket.receive('Pong', {header:probe?.payload.header ?? {protocol_version:7,room_id:0,conn_seq:1,revision:0},server_time_ms:1_000_000});
    const countdown = battleCountdown(f.client, 1_010_000);
    assert.equal(countdown.value.value, 10, 'actual BattleView calculation must use the server anchor');
    assert.ok(probe, 'connect immediately samples the existing Heartbeat/Pong protocol');
    monotonic += 2_000;wall += 3_600_000;countdown.tick();assert.equal(countdown.value.value, 8);
    wall -= 7_200_000;monotonic += 1_000;countdown.tick();assert.equal(countdown.value.value, 7);
  });
}

test('clock calibration requires matching Pong, measures RTT, and resets across reconnect', async (t) => {
  let now = 100;
  t.mock.method(performance, 'now', () => now);
  const f = fixture(t);const socket = await f.connect();
  const probe = socket.sent.find(message => message.type === 'Heartbeat');
  const countdown = battleCountdown(f.client, 1_010_000);
  assert.equal(countdown.value.value, null, 'uncalibrated display must not invent a duration');
  for (const header of [{...probe.payload.header,conn_seq:99},{...probe.payload.header,room_id:99},{...probe.payload.header,protocol_version:5}]) {
    socket.receive('Pong',{header,server_time_ms:1_000_000});countdown.tick();assert.equal(countdown.value.value,null);
  }
  for (const serverTime of [0, -1, 1.5, Number.MAX_SAFE_INTEGER + 1, "1000000", null]) {
    socket.receive('Pong',{header:probe.payload.header,server_time_ms:serverTime});countdown.tick();assert.equal(countdown.value.value,null);
  }
  now=50;socket.receive('Pong',{header:probe.payload.header,server_time_ms:1_000_000});countdown.tick();assert.equal(countdown.value.value,null,'negative RTT cannot calibrate');
  now=300;socket.receive('Pong',{header:probe.payload.header,server_time_ms:1_000_000});
  assert.equal(f.client.secondsUntilServerDeadline(1_010_100),10,'RTT/2 is accounted for at receipt');
  now=400;socket.receive('Pong',{header:probe.payload.header,server_time_ms:9_000_000});
  assert.equal(f.client.secondsUntilServerDeadline(1_010_100),10,'duplicate Pong cannot move the accepted anchor');
  socket.close();countdown.tick();assert.equal(countdown.value.value,null);
  const replacement=f.reconnect();replacement.open(2);await new Promise(resolve=>setImmediate(resolve));
  const newProbe=replacement.sent.find(message=>message.type==='Heartbeat');assert.ok(newProbe);
  socket.receive('Pong',{header:probe.payload.header,server_time_ms:9_000_000});countdown.tick();assert.equal(countdown.value.value,null,'old connection samples are ignored');
  now=500;replacement.receive('Pong',{header:newProbe.payload.header,server_time_ms:1_004_000});countdown.tick();
  assert.equal(countdown.value.value,6,'same deadline resumes with actual remaining time, not a full window');
});


test("source skill and deadline survive JoinOk, outcome and snapshot", async (t) => {
  const { client, connect } = fixture(t);
  const socket = await connect();
  const choice = { choice_id: 8, observer: 1, options: [{ token: 0, label_key: "confirm" }], prompt: "trigger_confirm", native_kind: "confirm", source_skill: "遗计", deadline_unix_ms: 123456 };
  const pending = client.joinRoom(7); join(socket, { choice });
  assert.deepEqual((await pending).envelope.choice, choice);
  const envelopes = []; client.onEnvelope(envelope => envelopes.push(envelope));
  socket.receive("CommandOutcome", { view: view(2), choice, legal_actions: [], result: { Accepted: {} }, idempotency_key: 99 });
  assert.deepEqual(envelopes.at(-1).choice, choice);
  const snapshot = client.requestSnapshot(); const request = socket.sent.at(-1);
  socket.receive("Snapshot", { header: request.payload.header, view: view(2), choice, legal_actions: [] });
  await snapshot; assert.deepEqual(envelopes.at(-1).choice, choice);
});
