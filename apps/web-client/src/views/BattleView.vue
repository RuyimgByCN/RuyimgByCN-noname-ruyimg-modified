<script setup lang="ts">
import { computed, onMounted, onUnmounted, ref, watch } from "vue";
import { useRoute, useRouter } from "vue-router";
import { EngineClient } from "../engine/client";
import {
  configureLocalAppearanceCatalog,
  configureOnlineAppearanceCatalog,
} from "../engine/appearanceStore";
import { setDesktopMatchFrozen } from "../engine/desktopHost";
import {
  contentLockSummary,
  putNativeSave,
  setMatchFrozen,
} from "../engine/extensionStore";
import {
  NetworkEngineClient,
  type RoomRoster,
} from "../engine/networkClient";
import {
  applyEnvelope,
  clearEffects,
  createViewStore,
  presentationDelay,
  pushLog,
} from "../engine/viewStore";
import {
  applyCandidatePayload,
  failCandidateLoad,
  initialGeneralSelectState,
  markCandidatesLoading,
  seatProjectionFromView,
  setGeneralSelectSeat,
  submitGeneralSelection,
  withSeatProjection,
} from "../engine/generalSelectModel";
import BattleBoard from "../components/battle/BattleBoard.vue";
import GeneralSelectPanel from "../components/battle/GeneralSelectPanel.vue";
import type { EngineEnvelope, LegalAction } from "../types/engine";
import { outcomeText } from "../engine/outcome";

const route = useRoute();
const router = useRouter();
const store = createViewStore();
const localClient = new EngineClient();
const networkClient = new NetworkEngineClient();
const selfId = ref(1);
const players = ref(Number(route.query.players ?? 4) || 4);
const seed = ref(Number(route.query.seed ?? 42) || 42);
const autoTimer = ref<number | null>(null);
const autoRunning = ref(false);
const statusLine = ref("未连接");
const roomRoster = ref<RoomRoster | null>(null);
const roomId = ref(0);
const roomStarted = ref(false);
const isOnline = computed(() => route.query.mode === "online");
const isSpectator = computed(() => isOnline.value && selfId.value === 0);
const generalSelect = ref(initialGeneralSelectState(selfId.value));
const showGeneralSelect = computed(
  () => !isOnline.value && store.connected && localClient.isNative(),
);

interface OnlineIntent {
  action: "create" | "join" | "spectate";
  wsUrl: string;
  displayName: string;
  password: string;
  roomId: number;
  players: number;
  seed: number;
}

async function withBusy<T>(fn: () => Promise<T>): Promise<T | undefined> {
  store.busy = true;
  store.error = null;
  try {
    return await fn();
  } catch (e) {
    store.error = e instanceof Error ? e.message : String(e);
    statusLine.value = store.error;
    return undefined;
  } finally {
    store.busy = false;
  }
}

async function handleEnvelope(env: EngineEnvelope) {
  applyEnvelope(store, env);
  await presentationDelay(store, 120);
  clearEffects(store);
  const prefix = isOnline.value ? `房间 #${roomId.value} · ` : "";
  const hashHint = (env.hash_hex || "").slice(0, 12);
  statusLine.value = env.finished
    ? `${prefix}终局 · ${outcomeText(env.view?.outcome)} · rev ${env.revision}`
    : `${prefix}rev ${env.revision}${hashHint ? ` · ${hashHint}…` : ""}${
        localClient.isNative() ? " · native" : ""
      }`;
  syncGeneralSelectProjection();
}

/** 选将面板的座位投影跟随主视图更新（开局/读档/动作后都能看到技能刷新）。 */
function syncGeneralSelectProjection() {
  generalSelect.value = withSeatProjection(
    generalSelect.value,
    seatProjectionFromView(store.view, generalSelect.value.seat),
  );
}

/**
 * 拉取原生目录候选。失败（旧产物缺导出 / 畸形回包）显示在面板里，不静默变成
 * 「没有候选」；空列表由模型区分为合法空目录。
 */
async function loadGeneralCandidates() {
  if (isOnline.value || !localClient.isNative()) return;
  generalSelect.value = markCandidatesLoading(generalSelect.value);
  try {
    const payload = await localClient.generalCandidates();
    generalSelect.value = applyCandidatePayload(generalSelect.value, payload);
  } catch (error) {
    generalSelect.value = failCandidateLoad(generalSelect.value, error);
  }
}

/** 点击候选：按名装配到面板座位；成功后应用刷新信封，失败把错误留在面板。 */
async function chooseGeneralCandidate(name: string) {
  if (isOnline.value || !localClient.isNative()) return;
  if (store.busy || generalSelect.value.selectionStatus === "pending") return;
  const state = generalSelect.value;
  await withBusy(async () => {
    generalSelect.value = await submitGeneralSelection(
      state,
      name,
      {
        chooseGeneral: (seat, generalName) =>
          localClient.chooseGeneral(seat, generalName),
        applyEnvelope: (env) => {
          applyEnvelope(store, env);
          pushLog(store, `选将 P${state.seat} → ${name}`);
        },
      },
      (pending) => {
        generalSelect.value = pending;
      },
    );
    if (generalSelect.value.selectionStatus === "accepted") {
      const hash = generalSelect.value.lastStateHash ?? "";
      statusLine.value = `选将 P${state.seat} · ${name}${
        hash ? ` · ${hash.slice(0, 12)}…` : ""
      }`;
    } else if (generalSelect.value.selectionStatus === "rejected") {
      statusLine.value = generalSelect.value.error ?? "选将失败";
    }
  });
}

async function connectAndStart() {
  await withBusy(async () => {
    await setDesktopMatchFrozen(true);
    setMatchFrozen(true);
    if (isOnline.value) {
      const intent = readOnlineIntent();
      configureOnlineAppearanceCatalog(intent.wsUrl);
      await connectOnline(intent);
      return;
    }
    configureLocalAppearanceCatalog();
    await localClient.connect();
    store.connected = true;
    statusLine.value = localClient.isNative()
      ? "原生 Worker 就绪，开局中…"
      : "Worker 就绪，开局中…";
    // 原生座位从 0 起；旧线观察者常为 1。保持 query 兼容，原生路径夹到 [0, players)。
    if (localClient.isNative()) {
      selfId.value = Math.max(0, Math.min(selfId.value, players.value - 1));
      generalSelect.value = setGeneralSelectSeat(generalSelect.value, selfId.value);
    }
    const env = await localClient.start(players.value, seed.value, selfId.value);
    await handleEnvelope(env);
    if (localClient.isNative()) await loadGeneralCandidates();
    const lock = contentLockSummary();
    pushLog(
      store,
      `开局 ${players.value} 人 seed=${seed.value}${
        lock.packages.length ? ` · 内容锁 ${lock.packages.length} 包` : ""
      }`,
    );
  });
}

async function saveMatch() {
  await withBusy(async () => {
    if (isOnline.value) throw new Error("联机模式请使用服务端存档");
    if (!localClient.isNative()) throw new Error("存档仅原生后端支持");
    const hex = await localClient.saveHex();
    const lock = contentLockSummary();
    const id = `save-${Date.now()}`;
    putNativeSave({
      id,
      label: `rev${store.revision} · ${players.value}p seed=${seed.value}`,
      hex,
      seed: seed.value,
      players: players.value,
      contentLock: lock.packages.map((p) => p.key),
      lockSeed: lock.hashSeed,
      savedAt: new Date().toISOString(),
      revision: store.revision,
    });
    pushLog(store, `已存档 ${id}（${hex.length / 2} 字节）`);
    statusLine.value = `已存档 · ${id}`;
  });
}

async function loadLatestSave() {
  await withBusy(async () => {
    if (isOnline.value) throw new Error("联机模式不支持本地读档");
    if (!localClient.isNative()) throw new Error("读档仅原生后端支持");
    const { listNativeSaves } = await import("../engine/extensionStore");
    const slots = listNativeSaves();
    const slot = slots[0];
    if (!slot) throw new Error("无本地存档");
    // 内容锁错版拒绝（G5 客户端闸）：锁摘要不一致则拒
    const lock = contentLockSummary();
    if (slot.lockSeed && lock.hashSeed !== slot.lockSeed) {
      throw new Error(
        `内容锁不匹配：存档 ${slot.lockSeed || "∅"} ≠ 当前 ${lock.hashSeed || "∅"}`,
      );
    }
    const env = await localClient.restoreHex(slot.hex);
    // 读档替换了引擎状态：上一局的选将成功态不能留到读档后的界面。
    generalSelect.value = setGeneralSelectSeat(
      generalSelect.value,
      generalSelect.value.seat,
    );
    await handleEnvelope(env);
    pushLog(store, `读档 ${slot.id} · rev ${slot.revision}`);
  });
}

async function connectOnline(intent: OnlineIntent) {
  networkClient.onEnvelope((envelope) => {
    void handleEnvelope(envelope);
  });
  // NC09：断线可见——服务端中途崩溃时写入状态行与日志（不静默保持旧状态）
  networkClient.onClose(() => {
    statusLine.value = "连接已断开（服务端不可达）";
    store.connected = false;
    pushLog(store, "连接已断开：服务端不可达或已停止");
  });
  // NC09：自动重连状态可见
  networkClient.onReconnect((state) => {
    if (state === "reconnecting") {
      statusLine.value = "连接已断开，正在重连…";
      store.connected = false;
    } else if (state === "reconnected") {
      statusLine.value = "已重连（等待房间状态）";
      store.connected = true;
      void networkClient.joinRoom(
        roomId.value || 1,
        "",
        localStorage.getItem(reconnectKey(roomId.value || 1)),
      )
        .then((s) => {
          selfId.value = s.playerId;
          roomId.value = s.roomId;
          statusLine.value = `房间 #${s.roomId} · 已重连`;
          return handleEnvelope(s.envelope);
        })
        .catch((error: unknown) => {
          const message = error instanceof Error ? error.message : String(error);
          statusLine.value = `重连后回房失败：${message}`;
          pushLog(store, `重连后回房失败：${message}`);
        });
    } else {
      statusLine.value = "重连失败（服务端不可达）";
      store.connected = false;
    }
  });
  networkClient.onRoster((roster) => {
    roomRoster.value = roster;
    roomId.value = roster.room_id;
    const justStarted = roster.started && !roomStarted.value;
    roomStarted.value = roster.started;
    statusLine.value = roster.started
      ? `房间 #${roster.room_id} · 已开局`
      : `房间 #${roster.room_id} · 等待入座 ${roster.seats.length}/${players.value}`;
    if (justStarted) void networkClient.requestSnapshot().catch(() => undefined);
  });

  statusLine.value = `连接 ${intent.wsUrl}…`;
  await networkClient.connect(intent.wsUrl, intent.displayName);
  let session;
  if (intent.action === "create") {
    session = await networkClient.createRoom(
      intent.players,
      intent.seed,
      intent.password
    );
  } else if (intent.action === "spectate") {
    session = await networkClient.spectate(intent.roomId);
  } else {
    const token = localStorage.getItem(reconnectKey(intent.roomId));
    session = await networkClient.joinRoom(
      intent.roomId,
      intent.password,
      token
    );
  }

  roomId.value = session.roomId;
  selfId.value = session.playerId;
  players.value = intent.players;
  if (session.role === "Player" && session.reconnectToken) {
    localStorage.setItem(reconnectKey(session.roomId), session.reconnectToken);
  }
  store.connected = true;
  await handleEnvelope(session.envelope);
  pushLog(
    store,
    session.role === "Spectator"
      ? `旁观房间 #${session.roomId}`
      : `加入房间 #${session.roomId} · 座位 ${session.seat + 1}`
  );
}

function readOnlineIntent(): OnlineIntent {
  const saved = sessionStorage.getItem("noname-next.online-intent");
  if (saved) {
    try {
      return JSON.parse(saved) as OnlineIntent;
    } catch {
      sessionStorage.removeItem("noname-next.online-intent");
    }
  }
  const secure = window.location.protocol === "https:";
  return {
    action: String(route.query.action ?? "join") as OnlineIntent["action"],
    wsUrl: `${secure ? "wss" : "ws"}://${window.location.hostname || "127.0.0.1"}:7700/ws`,
    displayName: "player",
    password: "",
    roomId: Number(route.query.room ?? 1) || 1,
    players: players.value,
    seed: seed.value,
  };
}

function reconnectKey(id: number): string {
  return `noname-next.reconnect.${id}`;
}

function commandClient() {
  return isOnline.value ? networkClient : localClient;
}

async function endPhase() {
  await withBusy(async () => {
    const env = await commandClient().endPhase(selfId.value);
    await handleEnvelope(env);
  });
}

async function autoStep() {
  await withBusy(async () => {
    if (isOnline.value) throw new Error("联机模式不提供客户端自动步进");
    const env = await localClient.autoStep();
    await handleEnvelope(env);
  });
}

async function playCard(cardId: number, targets: number[]) {
  await withBusy(async () => {
    const env = await commandClient().playCard(selfId.value, cardId, targets);
    await handleEnvelope(env);
    pushLog(store, `出牌 #${cardId}${targets.length > 0 ? ` → P${targets[0]}` : ""}`);
  });
}

async function activateSkill(skillId: number, targets: number[], cardTargets: number[]) {
  await withBusy(async () => {
    const env = await commandClient().activateSkill(
      selfId.value,
      skillId,
      targets,
      cardTargets
    );
    await handleEnvelope(env);
    pushLog(
      store,
      `发动技能 #${skillId}${targets.length > 0 ? ` → P${targets[0]}` : ""}`
    );
  });
}

async function resolveLegalChoice(action: Extract<LegalAction, { ResolveChoice: unknown }>) {
  await withBusy(async () => {
    const env = await commandClient().resolveChoice(
      selfId.value,
      action.ResolveChoice.choice_id,
      action.ResolveChoice.selections,
      action.ResolveChoice.cancelled
    );
    await handleEnvelope(env);
  });
}

const choiceActions = computed(() =>
  store.legalActions.filter(
    (action): action is Extract<LegalAction, { ResolveChoice: unknown }> =>
      typeof action === "object" && "ResolveChoice" in action
  )
);

const canEndPhase = computed(() => store.legalActions.includes("EndPhase"));

function toggleAuto() {
  if (autoRunning.value) {
    stopAuto();
    return;
  }
  autoRunning.value = true;
  const tick = async () => {
    if (!autoRunning.value || store.finished) {
      stopAuto();
      return;
    }
    await autoStep();
    if (autoRunning.value && !store.finished) {
      const wait = store.skipAnims || store.reduceMotion ? 30 : 280 / store.speed;
      autoTimer.value = window.setTimeout(tick, wait);
    } else {
      stopAuto();
    }
  };
  void tick();
}

function stopAuto() {
  autoRunning.value = false;
  if (autoTimer.value != null) {
    clearTimeout(autoTimer.value);
    autoTimer.value = null;
  }
}

function logEntryClass(text: string): string {
  if (text.includes('伤害') || text.includes('阵亡')) return 'log-damage';
  if (text.includes('回复') || text.includes('摸牌')) return 'log-positive';
  if (text.includes('濒死') || text.includes('等待')) return 'log-warning';
  if (text.includes('回合') || text.includes('使用')) return 'log-action';
  return '';
}

const showHotkeys = ref(false);

function onKeydown(e: KeyboardEvent) {
  if (e.key === '?') {
    showHotkeys.value = !showHotkeys.value;
    return;
  }
  if (e.key === 'Escape') {
    showHotkeys.value = false;
    return;
  }
  if (store.busy || store.finished) return;
  if (e.key === ' ') {
    e.preventDefault();
    if (!isOnline.value) void autoStep();
  } else if (e.key === 'e' || e.key === 'E') {
    if (canEndPhase.value) void endPhase();
  } else if (e.key === 's' || e.key === 'S') {
    store.skipAnims = !store.skipAnims;
  }
}

function releaseMatchLocks() {
  stopAuto();
  localClient.dispose();
  networkClient.dispose();
  void setDesktopMatchFrozen(false);
  setMatchFrozen(false);
}

onMounted(() => {
  window.addEventListener("keydown", onKeydown);
  // 整页跳转不会走 onUnmounted，需 pagehide 解冻内容锁
  window.addEventListener("pagehide", releaseMatchLocks);
  // NC09：连接失败必须**可见且不产生未处理拒绝**——此前 ws 不可达时 UI 显示了
  // 「WebSocket 连接失败」但 promise 未捕获，控制台出现 `Uncaught (in promise)`。
  void connectAndStart().catch((error: unknown) => {
    const message = error instanceof Error ? error.message : String(error);
    statusLine.value = `连接失败：${message}`;
    pushLog(store, `连接失败：${message}`);
    store.connected = false;
  });
});

onUnmounted(() => {
  window.removeEventListener("keydown", onKeydown);
  window.removeEventListener("pagehide", releaseMatchLocks);
  releaseMatchLocks();
});

watch(
  () => store.speed,
  () => {
    /* timeline rate only */
  }
);

const PHASE_LABELS: Record<string, string> = {
  Draw: "摸牌",
  Play: "出牌",
  Discard: "弃牌",
  Judge: "判定",
};

const matchOutcome = computed(() => {
  const events = store.lastEvents ?? [];
  for (const e of events) {
    const ev = e as { type?: string; outcome?: Record<string, unknown> };
    if (ev?.type === "MatchEnded" && ev.outcome) {
      const oc = Object.keys(ev.outcome)[0];
      if (oc === "RebelWin") return { winner: "反贼", label: "反贼胜利！", isSelfWin: false };
      if (oc === "LoyalistWin") return { winner: "忠臣", label: "忠臣/主公胜利！", isSelfWin: true };
      if (oc === "RenegadeWin") return { winner: "内奸", label: "内奸胜利！", isSelfWin: false };
      if (oc === "Draw") return { winner: "平局", label: "平局", isSelfWin: false };
      if (oc === "Aborted") return { winner: "中止", label: "对局中止", isSelfWin: false };
    }
  }
  const outcome = store.view?.outcome;
  const tag = typeof outcome === "string" ? outcome : Object.keys((outcome ?? {}) as object)[0];
  if (tag === "RebelWin") return { winner: "反贼", label: "反贼胜利！", isSelfWin: false };
  if (tag === "LoyalistWin") return { winner: "忠臣", label: "忠臣/主公胜利！", isSelfWin: true };
  if (tag === "RenegadeWin") return { winner: "内奸", label: "内奸胜利！", isSelfWin: false };
  if (tag === "Draw") return { winner: "平局", label: "平局", isSelfWin: false };
  if (tag === "Aborted") return { winner: "中止", label: "对局中止", isSelfWin: false };
  return null;
});

function returnToLobby() {
  void router.push("/");
}

const turnInfo = computed(() => {
  if (store.view && store.view.current_player !== 0 && store.view.phase !== "Idle") {
    return {
      currentPlayer: store.view.current_player,
      phase: store.view.phase,
      phaseLabel: PHASE_LABELS[store.view.phase] ?? store.view.phase,
      round: store.view.round,
    };
  }
  const events = store.lastEvents ?? [];
  for (const e of events) {
    const ev = e as { type?: string; current?: number; phase?: string };
    if (ev?.type === "TurnStateChanged") {
      return {
        currentPlayer: ev.current ?? 0,
        phase: ev.phase ?? "?",
        phaseLabel: PHASE_LABELS[ev.phase ?? ""] ?? ev.phase ?? "?",
        round: store.view?.round ?? 0,
      };
    }
  }
  return null;
});
</script>

<template>
  <div class="battle stack">
    <!-- Keyboard hotkey overlay -->
    <section v-if="showHotkeys" class="panel hotkey-overlay" role="dialog" aria-label="快捷键">
      <h2>⌨ 快捷键</h2>
      <dl class="hotkey-list">
        <dt><kbd>Space</kbd></dt><dd>自动一步</dd>
        <dt><kbd>E</kbd></dt><dd>结束阶段</dd>
        <dt><kbd>S</kbd></dt><dd>切换跳过动画</dd>
        <dt><kbd>?</kbd></dt><dd>显示/隐藏此面板</dd>
        <dt><kbd>Esc</kbd></dt><dd>关闭此面板</dd>
      </dl>
    </section>
    <!-- Match outcome banner -->
    <section v-if="store.finished && matchOutcome" class="panel outcome-banner" :class="{ win: matchOutcome.isSelfWin, lose: !matchOutcome.isSelfWin }" role="alert" aria-live="assertive">
      <div class="outcome-content">
        <span class="outcome-icon">{{ matchOutcome.isSelfWin ? "🎉" : "💀" }}</span>
        <div>
          <h2 class="outcome-label">{{ matchOutcome.label }}</h2>
          <p class="muted">{{ matchOutcome.winner }}阵营获胜 · rev {{ store.revision }}</p>
        </div>
      </div>
      <button class="primary" type="button" @click="isOnline ? returnToLobby() : connectAndStart()">
        {{ isOnline ? "返回大厅" : "再来一局" }}
      </button>
    </section>

    <section v-if="turnInfo" class="panel turn-bar" role="status" aria-live="polite">
      <span class="turn-player">► 第 {{ turnInfo.round }} 轮 · P{{ turnInfo.currentPlayer }}</span>
      <span class="phase-badge" :data-phase="turnInfo.phase">{{ turnInfo.phaseLabel }}</span>
    </section>

    <section class="panel toolbar" aria-label="对局控制">
      <div class="row">
        <button v-if="!isOnline" class="primary" type="button" :disabled="store.busy" @click="connectAndStart">
          重新开局
        </button>
        <span v-else class="online-badge">联机房间 #{{ roomId || "…" }}</span>
        <button type="button" :disabled="store.busy || store.finished || isSpectator || (isOnline && !roomStarted) || !canEndPhase" @click="endPhase">
          结束阶段
        </button>
        <button v-if="!isOnline" type="button" :disabled="store.busy || store.finished" @click="autoStep">
          自动一步
        </button>
        <button v-if="!isOnline" type="button" :disabled="store.busy" @click="toggleAuto">
          {{ autoRunning ? "停止自动" : "连续自动" }}
        </button>
        <button
          v-if="!isOnline"
          type="button"
          :disabled="store.busy || !store.connected"
          @click="saveMatch"
        >
          存档
        </button>
        <button
          v-if="!isOnline"
          type="button"
          :disabled="store.busy || !store.connected"
          @click="loadLatestSave"
        >
          读档
        </button>
      </div>
      <div class="row muted controls">
        <label>
          倍速
          <select v-model.number="store.speed" aria-label="播放倍速">
            <option :value="0.5">0.5×</option>
            <option :value="1">1×</option>
            <option :value="2">2×</option>
            <option :value="4">4×</option>
          </select>
        </label>
        <label class="check">
          <input v-model="store.skipAnims" type="checkbox" />
          跳过动画
        </label>
        <span>{{ isSpectator ? "公共旁观投影" : `观察者 P${selfId}` }}</span>
      </div>
      <p class="status" role="status" aria-live="polite">{{ statusLine }}</p>
      <p v-if="store.error" class="error" role="alert">{{ store.error }}</p>
    </section>

    <GeneralSelectPanel
      v-if="showGeneralSelect"
      :state="generalSelect"
      :busy="store.busy"
      @choose="chooseGeneralCandidate"
      @reload="loadGeneralCandidates"
    />

    <section v-if="isOnline && roomRoster" class="panel online-roster" aria-label="联机房间座位">
      <div class="row online-roster-head">
        <h2>房间 #{{ roomRoster.room_id }}</h2>
        <span :class="roomRoster.started ? 'started' : 'waiting'">
          {{ roomRoster.started ? "对局进行中" : "等待玩家" }}
        </span>
      </div>
      <div class="row online-seats">
        <span v-for="seat in roomRoster.seats" :key="seat.seat" class="online-seat">
          {{ seat.seat + 1 }} · {{ seat.display_name }}
          <i :class="seat.connected ? 'connected' : 'offline'">{{ seat.connected ? "在线" : "离线" }}</i>
        </span>
      </div>
    </section>

    <section v-if="store.choice" class="panel choice" aria-label="待选择">
      <h2>选择屏障</h2>
      <p class="muted">
        choice #{{ store.choice.choice_id }} · 玩家
        {{ store.choice.observer }} · {{ store.choice.kind ?? "Custom" }}
      </p>
      <div v-if="choiceActions.length" class="row">
        <button
          v-for="(action, index) in choiceActions"
          :key="index"
          type="button"
          @click="resolveLegalChoice(action)"
        >
          {{ action.ResolveChoice.cancelled ? "取消" : `选择 ${action.ResolveChoice.selections.join(", ") || "空"}` }}
        </button>
      </div>
      <p v-else class="muted">当前观察者无权处理此选择。</p>
    </section>

    <!-- Player overview roster -->
    <section v-if="store.view" class="panel roster" aria-label="玩家概况">
      <h2>玩家概况</h2>
      <div class="roster-grid">
        <div
          v-for="s in store.view.seats"
          :key="s.player"
          class="roster-entry"
          :class="{ dead: !s.alive, self: s.player === selfId }"
        >
          <span class="roster-player">P{{ s.player }}{{ s.turned_over ? ' · 翻面' : '' }}</span>
          <span class="roster-hp" :style="{ color: !s.alive ? 'var(--color-negative)' : s.hp <= 0 ? 'var(--color-warning)' : 'inherit' }">
            {{ !s.alive ? '✗' : s.hp <= 0 ? '⚠' : '' }} {{ s.hp }}/{{ s.max_hp }}
          </span>
          <span class="roster-hand muted">{{ s.hand_count }}🂠</span>
          <span v-if="s.identity" class="roster-id" :data-id="s.identity">{{ s.identity === 'Lord' ? '主' : s.identity === 'Loyalist' ? '忠' : s.identity === 'Rebel' ? '反' : '奸' }}</span>
          <span v-else class="roster-id muted">?</span>
        </div>
      </div>
    </section>

    <section class="panel battle-section">
      <BattleBoard
        :view="store.view"
        :self-id="selfId"
        :legal-actions="store.legalActions"
        @play-card="playCard"
        @activate-skill="activateSkill"
      />
      <!-- Effect overlay -->
      <div v-if="store.effects.length" class="effect-overlay" aria-hidden="true">
        <div
          v-for="fx in store.effects"
          :key="fx.id"
          class="effect-popup"
          :class="fx.type"
        >
          <span class="effect-icon">{{
            fx.type === 'damage' ? '⚔' :
            fx.type === 'heal' ? '✚' :
            fx.type === 'draw' ? '🂠' :
            fx.type === 'play' ? '▸' :
            fx.type === 'dying' ? '⚠' : '✗'
          }}</span>
          <span class="effect-text">{{ fx.text }}</span>
          <span v-if="fx.player != null" class="effect-player">P{{ fx.player }}</span>
        </div>
      </div>
    </section>

    <!-- Match stats panel -->
    <section v-if="store.view" class="panel match-stats" aria-label="对局统计">
      <div class="stats-grid">
        <div class="stat">
          <span class="stat-label muted">Revision</span>
          <span class="stat-value mono">{{ store.revision }}</span>
        </div>
        <div class="stat">
          <span class="stat-label muted">State Hash</span>
          <span class="stat-value mono">{{ store.hash.slice(0, 16) }}…</span>
        </div>
        <div class="stat">
          <span class="stat-label muted">已知卡牌</span>
          <span class="stat-value">{{ store.view.known_cards.length }}</span>
        </div>
        <div class="stat">
          <span class="stat-label muted">存活玩家</span>
          <span class="stat-value">{{ store.view.seats.filter(s => s.alive).length }} / {{ store.view.seats.length }}</span>
        </div>
      </div>
    </section>

    <!-- Seat status overview -->
    <section v-if="store.view" class="panel seat-status" aria-label="座位状态一览">
      <h2>座位状态</h2>
      <div class="seat-status-grid">
        <div
          v-for="s in store.view.seats"
          :key="'ss-' + s.player"
          class="ss-entry"
          :class="{ 'ss-dead': !s.alive, 'ss-self': s.player === selfId }"
        >
          <div class="ss-header">
            <span class="ss-player">P{{ s.player }}</span>
            <span v-if="s.player === selfId" class="ss-tag">你</span>
          </div>
          <div class="ss-hp-bar">
            <div
              class="ss-hp-fill"
              :style="{
                width: Math.max(Math.min(s.hp / s.max_hp * 100, 100), 0) + '%',
                background: !s.alive ? 'var(--color-negative)' : s.hp <= 0 ? 'var(--color-warning)' : 'var(--color-positive)'
              }"
            ></div>
          </div>
          <div class="ss-hp-text">
            <span class="ss-icon">{{ !s.alive ? '✗' : s.hp <= 0 ? '⚠' : '' }}</span>
            {{ Math.max(s.hp, 0) }}/{{ s.max_hp }}
          </div>
          <div class="ss-meta muted">
            <span>{{ s.hand_count }}🂠</span>
            <span v-if="s.identity" class="ss-id" :data-id="s.identity">{{
              s.identity === 'Lord' ? '主' : s.identity === 'Loyalist' ? '忠' : s.identity === 'Rebel' ? '反' : '奸'
            }}</span>
            <span v-else class="muted">?</span>
          </div>
        </div>
      </div>
    </section>

    <section class="panel log" aria-label="事件日志">
      <div class="log-header">
        <h2>战斗日志</h2>
        <button v-if="store.log.length > 0" type="button" class="clear-btn" @click="store.log = []" aria-label="清空日志">清空</button>
      </div>
      <p class="muted tiny">
        日志来自 DomainEvent 摘要，不是状态真相源。真相源为 Command Log + 投影（§22）。
      </p>
      <ol class="log-list" v-auto-scroll>
        <li
          v-for="e in store.log.slice().reverse()"
          :key="e.id"
          class="log-entry"
          :class="logEntryClass(e.text)"
        >
          <span class="log-rev">rev{{ e.at }}</span>
          <span class="log-text">{{ e.text }}</span>
        </li>
      </ol>
      <p v-if="store.log.length === 0" class="muted empty-log">暂无事件</p>
    </section>
  </div>
</template>

<style scoped>
.hotkey-overlay {
  border: 1px solid var(--accent-2);
  background: var(--bg-elevated);
}
.hotkey-overlay h2 { font-size: 1.05rem; }
.hotkey-list {
  display: grid;
  grid-template-columns: max-content 1fr;
  gap: 0.3rem 1rem;
  margin: 0.4rem 0;
}
.hotkey-list dt { text-align: right; }
.hotkey-list dd { margin: 0; font-size: 0.88rem; }
kbd {
  display: inline-block;
  padding: 0.1rem 0.4rem;
  background: var(--bg);
  border: 1px solid var(--border);
  border-bottom-width: 2px;
  border-radius: 4px;
  font-family: var(--mono);
  font-size: 0.8rem;
  color: var(--accent);
}

.roster h2 { font-size: 1.05rem; }
.roster-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(110px, 1fr));
  gap: 0.4rem;
  margin-top: 0.4rem;
}
.roster-entry {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.25rem 0.5rem;
  background: var(--bg-elevated);
  border-radius: 6px;
  font-size: 0.82rem;
}
.roster-entry.self { border: 1px solid var(--accent); }
.roster-entry.dead { opacity: 0.5; }
.roster-player { font-weight: 600; min-width: 1.5rem; }
.roster-hp { font-family: var(--mono); font-size: 0.85rem; }
.roster-hand { font-size: 0.78rem; }
.roster-id {
  font-weight: 700;
  font-size: 0.75rem;
  padding: 0.05rem 0.3rem;
  border-radius: 4px;
  min-width: 1.2rem;
  text-align: center;
}
.roster-id[data-id="Lord"] { background: var(--accent); color: #1a1208; }
.roster-id[data-id="Loyalist"] { background: var(--color-positive); color: #fff; }
.roster-id[data-id="Rebel"] { background: var(--color-negative); color: #fff; }
.roster-id[data-id="Renegade"] { background: var(--color-warning); color: #1a1208; }

.outcome-banner {
  display: flex;
  align-items: center;
  justify-content: space-between;
  padding: 1rem 1.5rem;
  animation: outcome-slide 0.4s ease-out;
}
.outcome-banner.win {
  border-color: var(--color-positive);
  background: rgba(95, 191, 122, 0.1);
}
.outcome-banner.lose {
  border-color: var(--color-negative);
  background: rgba(224, 90, 90, 0.1);
}
.outcome-content {
  display: flex;
  align-items: center;
  gap: 1rem;
}
.outcome-icon {
  font-size: 2rem;
}
.outcome-label {
  margin: 0;
  font-size: 1.3rem;
  font-weight: 700;
}
@keyframes outcome-slide {
  0% { opacity: 0; transform: translateY(-20px); }
  100% { opacity: 1; transform: translateY(0); }
}
@media (prefers-reduced-motion: reduce) {
  .outcome-banner { animation: none; }
}

.turn-bar {
  display: flex;
  align-items: center;
  gap: 1rem;
  padding: 0.5rem 1rem;
}
.turn-player {
  font-weight: 600;
  color: var(--color-accent);
}
.phase-badge {
  display: inline-block;
  padding: 0.15rem 0.6rem;
  border-radius: 999px;
  font-size: 0.85rem;
  font-weight: 600;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
}
.phase-badge[data-phase="Draw"] { color: var(--color-positive); border-color: var(--color-positive); }
.phase-badge[data-phase="Play"] { color: var(--color-accent); border-color: var(--color-accent); }
.phase-badge[data-phase="Discard"] { color: var(--color-warning); border-color: var(--color-warning); }
.phase-badge[data-phase="Judge"] { color: var(--accent-2); border-color: var(--accent-2); }

.toolbar .status {
  margin: 0.5rem 0 0;
  font-family: var(--mono);
  font-size: 0.85rem;
  color: var(--muted);
}
.online-badge {
  color: var(--accent);
  font-family: var(--mono);
  font-weight: 600;
}
.online-roster h2 { margin: 0; font-size: 1rem; }
.online-roster-head { justify-content: space-between; }
.online-roster .started { color: var(--color-positive); }
.online-roster .waiting { color: var(--color-warning); }
.online-seats { flex-wrap: wrap; margin-top: 0.55rem; }
.online-seat {
  padding: 0.3rem 0.55rem;
  border: 1px solid var(--border);
  border-radius: 6px;
  font-size: 0.82rem;
}
.online-seat i { margin-left: 0.4rem; font-style: normal; font-size: 0.72rem; }
.online-seat .connected { color: var(--color-positive); }
.online-seat .offline { color: var(--muted); }
.error {
  color: var(--danger);
  margin: 0.35rem 0 0;
}
.controls label {
  display: inline-flex;
  align-items: center;
  gap: 0.35rem;
}
.controls select {
  background: var(--bg);
  border: 1px solid var(--border);
  color: var(--text);
  border-radius: 6px;
  padding: 0.2rem 0.4rem;
}
.choice h2 {
  font-size: 1.05rem;
}
.log h2 { font-size: 1.05rem; }
.log-header { display: flex; justify-content: space-between; align-items: center; }
.clear-btn {
  font-size: 0.75rem;
  padding: 0.15rem 0.5rem;
  opacity: 0.7;
}
.log-list {
  margin: 0.5rem 0 0;
  padding-left: 0;
  max-height: 250px;
  overflow-y: auto;
  list-style: none;
  font-size: 0.88rem;
}
.log-entry {
  display: flex;
  align-items: baseline;
  gap: 0.4rem;
  padding: 0.15rem 0;
  border-bottom: 1px solid rgba(255,255,255,0.04);
}
.log-rev {
  font-family: var(--mono);
  font-size: 0.72rem;
  color: var(--muted);
  min-width: 3rem;
  flex-shrink: 0;
}
.log-text { flex: 1; }
.log-damage { color: var(--color-negative); }
.log-positive { color: var(--color-positive); }
.log-warning { color: var(--color-warning); }
.log-action { color: var(--color-accent); }
.empty-log { margin: 0.5rem 0; }
.seat-status h2 { font-size: 1.05rem; }
.seat-status-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(130px, 1fr));
  gap: 0.5rem;
}
.ss-entry {
  padding: 0.4rem 0.6rem;
  background: var(--bg-elevated);
  border-radius: 8px;
  border: 1px solid transparent;
}
.ss-entry.ss-self { border-color: var(--accent); }
.ss-entry.ss-dead { opacity: 0.5; }
.ss-header { display: flex; justify-content: space-between; align-items: center; }
.ss-player { font-weight: 600; font-size: 0.9rem; }
.ss-tag { font-size: 0.65rem; background: var(--accent); color: #1a1208; border-radius: 4px; padding: 0.05rem 0.3rem; }
.ss-hp-bar { height: 4px; background: var(--bg-panel); border-radius: 2px; overflow: hidden; margin: 0.3rem 0 0.15rem; }
.ss-hp-fill { height: 100%; transition: width 0.3s; }
.ss-hp-text { font-size: 0.8rem; font-family: var(--mono); display: flex; align-items: center; gap: 0.2rem; }
.ss-icon { font-weight: bold; }
.ss-meta { display: flex; justify-content: space-between; font-size: 0.75rem; margin-top: 0.15rem; }
.ss-id {
  font-weight: 700; font-size: 0.7rem; padding: 0.05rem 0.3rem;
  border-radius: 3px; min-width: 1rem; text-align: center;
}
.ss-id[data-id="Lord"] { background: var(--accent); color: #1a1208; }
.ss-id[data-id="Loyalist"] { background: var(--color-positive); color: #fff; }
.ss-id[data-id="Rebel"] { background: var(--color-negative); color: #fff; }
.ss-id[data-id="Renegade"] { background: var(--color-warning); color: #1a1208; }

.match-stats { padding: 0.5rem 1rem; }
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
  gap: 0.5rem;
}
.stat {
  display: flex;
  flex-direction: column;
  gap: 0.1rem;
}
.stat-label { font-size: 0.75rem; }
.stat-value { font-size: 0.9rem; font-weight: 600; }
.mono { font-family: var(--mono); }
.tiny {
  font-size: 0.8rem;
  margin-top: 0;
}
.log-list {
  margin: 0.5rem 0 0;
  padding-left: 1.1rem;
  max-height: 220px;
  overflow: auto;
  font-size: 0.88rem;
  color: var(--muted);
}

.battle-section {
  position: relative;
}
.effect-overlay {
  position: absolute;
  top: 0;
  right: 1rem;
  display: flex;
  flex-direction: column;
  gap: 0.3rem;
  pointer-events: none;
  z-index: 10;
}
.effect-popup {
  display: flex;
  align-items: center;
  gap: 0.3rem;
  padding: 0.3rem 0.6rem;
  border-radius: 8px;
  font-size: 0.85rem;
  font-weight: 600;
  animation: effect-pop 0.8s ease-out forwards;
  background: var(--bg-elevated);
  border: 1px solid var(--border);
}
.effect-popup.damage { color: var(--color-negative); border-color: var(--color-negative); }
.effect-popup.heal { color: var(--color-positive); border-color: var(--color-positive); }
.effect-popup.draw { color: var(--accent-2); border-color: var(--accent-2); }
.effect-popup.play { color: var(--color-accent); border-color: var(--color-accent); }
.effect-popup.dying { color: var(--color-warning); border-color: var(--color-warning); }
.effect-popup.death { color: var(--color-negative); border-color: var(--color-negative); }
.effect-icon { font-size: 1rem; }
.effect-player { font-size: 0.75rem; opacity: 0.8; }

@keyframes effect-pop {
  0% { opacity: 0; transform: translateY(-10px) scale(0.9); }
  20% { opacity: 1; transform: translateY(0) scale(1); }
  80% { opacity: 1; }
  100% { opacity: 0; transform: translateY(-20px) scale(0.95); }
}
@media (prefers-reduced-motion: reduce) {
  .effect-popup {
    animation: none;
  }
}
</style>
