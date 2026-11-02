import { MAX_ENGINE_OUTSTANDING } from "./desktopEngineTransport";
import { presentationMetadata } from "./presentationMetadata";
import { validateObserverMetadata } from "./observerMetadata";
import { publicEventLabel } from "./publicEvents";
import type {
  ChoiceRequest,
  EngineEnvelope,
  LegalAction,
  ObserverSnapshot,
} from "../types/engine";

const NETWORK_PROTOCOL_VERSION = 7;

type CommandKind =
  | "EndPhase"
  | "Surrender"
  | { PlayCard: { card: number; targets: number[]; card_targets: number[] } }
  | { ActivateSkill: { skill: number; targets: number[]; card_targets: number[] } }
  | {
      UseSkillCard: {
        skill: number;
        card_kind: number;
        targets: number[];
        equip_slot: number;
      };
    }
  | {
      ResolveChoice: {
        choice_id: number;
        selections: number[];
        cancelled: boolean;
      };
    };

interface MessageHeader {
  protocol_version: number;
  room_id: number;
  conn_seq: number;
  revision: number;
}

export interface RoomRoster {
  room_id: number;
  seats: {
    seat: number;
    player_id: number;
    display_name: string;
    connected: boolean;
    managed: boolean;
  }[];
  choice_timeout_secs: number | null;
  started: boolean;
  finished: boolean;
}

const PENDING_STORAGE_KEY = "noname-next.pending-command";
/** 刷新后仍重放 pending 的最长时间；过期视为陈旧意图，避免隔天误重放。 */
const PENDING_TTL_MS = 10 * 60 * 1000;

type PersistedPending = {
  roomId: number;
  key: number;
  kind: CommandKind;
  createdAt: number;
};

function loadPersistedPending(): PersistedPending | null {
  try {
    if (typeof sessionStorage === "undefined") return null;
    const raw = sessionStorage.getItem(PENDING_STORAGE_KEY);
    if (!raw) return null;
    const parsed = JSON.parse(raw) as Partial<PersistedPending>;
    if (
      !parsed ||
      typeof parsed.roomId !== "number" ||
      typeof parsed.key !== "number" ||
      typeof parsed.createdAt !== "number" ||
      !parsed.kind
    ) {
      return null;
    }
    if (Date.now() - parsed.createdAt > PENDING_TTL_MS) return null;
    return {
      roomId: parsed.roomId,
      key: parsed.key,
      kind: parsed.kind as CommandKind,
      createdAt: parsed.createdAt,
    };
  } catch {
    return null;
  }
}

function storePending(kind: CommandKind, key: number, roomId: number): void {
  try {
    if (typeof sessionStorage === "undefined" || !roomId) return;
    sessionStorage.setItem(
      PENDING_STORAGE_KEY,
      JSON.stringify({ roomId, key, kind, createdAt: Date.now() }),
    );
  } catch {
    /* 存储不可用时保持原有同实例重连语义 */
  }
}

function clearPendingStorage(key: number): void {
  try {
    if (typeof sessionStorage === "undefined") return;
    const raw = sessionStorage.getItem(PENDING_STORAGE_KEY);
    if (!raw) return;
    const parsed = JSON.parse(raw) as Partial<PersistedPending>;
    if (parsed.key === key) sessionStorage.removeItem(PENDING_STORAGE_KEY);
  } catch {
    /* ignore */
  }
}

export interface OnlineSession {
  roomId: number;
  playerId: number;
  seat: number;
  role: "Player" | "Spectator" | "Referee";
  reconnectToken: string;
  view: ObserverSnapshot;
  envelope: EngineEnvelope;
}

type PendingCommand = {
  /** 原命令内容；重连后必须用**同一 key 与同一命令**重放，不能换 key 重做。 */
  kind: CommandKind;
  key: number;
  resolve: (value: EngineEnvelope) => void;
  reject: (reason: Error) => void;
};

type JoinWaiter = {
  resolve: (value: OnlineSession) => void;
  reject: (reason: Error) => void;
};

/**
 * Browser transport for the authoritative game-server WebSocket protocol.
 * It only submits normalized commands and consumes observer projections.
 */
export class NetworkEngineClient {
  private socket: WebSocket | null = null;
  private connectionId = 0;
  private roomId = 0;
  private playerId = 0;
  private revision = 0;
  private connSeq = 0;
  private idemSeq = 1;
  private choice: ChoiceRequest | null = null;
  private session: OnlineSession | null = null;
  private roomPassword = "";
  private welcomeWaiter: { resolve: () => void; reject: (e: Error) => void } | null = null;
  private joinWaiter: JoinWaiter | null = null;
  private pending = new Map<number, PendingCommand>();
  private persistedPending: PersistedPending | null = loadPersistedPending();
  private snapshotWaiters = new Map<
    number,
    { resolve: (view: ObserverSnapshot) => void; reject: (e: Error) => void }
  >();
  private envelopeListeners = new Set<(envelope: EngineEnvelope) => void>();
  private rosterListeners = new Set<(roster: RoomRoster) => void>();
  private closeListeners = new Set<() => void>();
  private heartbeat: number | null = null;
  private clockProbe: { connSeq: number; roomId: number; sentAt: number } | null = null;
  private serverTimeAnchor: { unixMs: number; monotonicMs: number } | null = null;
  private disposed = false;
  /// NC09 自动重连：最近一次连接参数（断开后按退避重试）
  private lastUrl: string | null = null;
  private lastName = "";
  private reconnectTimer: number | null = null;
  private reconnectAttempts = 0;
  private reconnectListeners = new Set<(state: "reconnecting" | "reconnected" | "failed") => void>();

  async connect(url: string, displayName: string): Promise<void> {
    if (this.socket?.readyState === WebSocket.OPEN) return;
    this.disposed = false;
    this.clockProbe = null;
    this.serverTimeAnchor = null;
    this.lastUrl = url;
    this.lastName = displayName;

    const welcome = new Promise<void>((resolve, reject) => {
      this.welcomeWaiter = { resolve, reject };
    });
    const opened = new Promise<void>((resolve, reject) => {
      const socket = new WebSocket(normalizeWsUrl(url));
      this.socket = socket;
      socket.onopen = () => {
        if (this.socket !== socket) {
          reject(new Error("WebSocket 连接已替换"));
          return;
        }
        this.connSeq = 0;
        this.idemSeq = 1;
        resolve();
      };
      socket.onerror = () => reject(new Error("WebSocket 连接失败"));
      socket.onmessage = (event) => {
        if (this.socket === socket) this.handleMessage(String(event.data));
      };
      socket.onclose = () => {
        reject(new Error("WebSocket 连接已关闭"));
        if (this.socket === socket) this.handleClose();
      };
    });

    // NC09：`opened` 失败时 `welcome` 仍会在稍后 reject——必须预先挂一个 no-op catch，
    // 否则控制台出现 `Uncaught (in promise)`（UI 已显示「WebSocket 连接失败」但仍有未处理拒绝）。
    void welcome.catch(() => undefined);
    await opened;
    await welcome;
    this.send({
      type: "Hello",
      payload: {
        versions: {
          engine_api: 1,
          rule_vm_abi: 167,
          content_schema: 3,
          network_protocol: NETWORK_PROTOCOL_VERSION,
          replay_schema: 1,
          snapshot_schema: 1,
          package_format: 1,
          engine_build_id: "noname-next-web",
        },
        display_name: displayName,
      },
    });
    this.sendHeartbeat();
  }

  /** Display estimate only; server deadlines remain authoritative. */
  secondsUntilServerDeadline(deadline: number | undefined): number | null {
    if (deadline == null || !this.serverTimeAnchor) return null;
    const elapsed = performance.now() - this.serverTimeAnchor.monotonicMs;
    return Math.max(0, Math.ceil((deadline - this.serverTimeAnchor.unixMs - elapsed) / 1000));
  }

  createRoom(players: number, seed: number, password = "", choiceTimeoutSecs = 30): Promise<OnlineSession> {
    return this.joinRequest({ type: "CreateRoom", payload: { players, seed, password, choice_timeout_secs: choiceTimeoutSecs } }, password);
  }

  joinRoom(
    roomId: number,
    password = "",
    reconnectToken: string | null = null
  ): Promise<OnlineSession> {
    this.roomId = roomId;
    return this.joinRequest({
      type: "JoinRoom",
      payload: {
        room_id: roomId,
        password,
        preferred_seat: null,
        reconnect_token: reconnectToken,
      },
    }, password);
  }

  spectate(roomId: number): Promise<OnlineSession> {
    this.roomId = roomId;
    return this.joinRequest({ type: "Spectate", payload: { room_id: roomId } }, "");
  }

  rejoinRoom(): Promise<OnlineSession> {
    const session = this.session;
    if (!session) return Promise.reject(new Error("尚未加入房间"));
    return session.role === "Spectator"
      ? this.spectate(session.roomId)
      : this.joinRoom(session.roomId, this.roomPassword, session.reconnectToken);
  }

  onEnvelope(listener: (envelope: EngineEnvelope) => void): () => void {
    this.envelopeListeners.add(listener);
    return () => this.envelopeListeners.delete(listener);
  }

  /// 断线通知（NC09）：注册后返回取消函数。
  onClose(listener: () => void): () => void {
    this.closeListeners.add(listener);
    return () => this.closeListeners.delete(listener);
  }

  onRoster(listener: (roster: RoomRoster) => void): () => void {
    this.rosterListeners.add(listener);
    return () => this.rosterListeners.delete(listener);
  }

  /// 取消所有尚未确认的命令（UI 取消入口）：拒绝等待者并清除刷新持久化，
  /// 避免下一次入房继续重放。服务端是否已提交需另行查询/接受不确定语义。
  cancelPendingCommands(reason = "已取消待确认命令"): void {
    for (const pending of this.pending.values()) {
      clearPendingStorage(pending.key);
      pending.reject(new Error(reason));
    }
    this.pending.clear();
    if (this.persistedPending) {
      clearPendingStorage(this.persistedPending.key);
      this.persistedPending = null;
    }
  }

  endPhase(_actor = 0): Promise<EngineEnvelope> {
    return this.submit("EndPhase");
  }

  playCard(
    _actor: number,
    card: number,
    targets: number[]
  ): Promise<EngineEnvelope> {
    return this.submit({ PlayCard: { card, targets, card_targets: [] } });
  }

  activateSkill(
    _actor: number,
    skill: number,
    targets: number[],
    cardTargets: number[]
  ): Promise<EngineEnvelope> {
    return this.submit({
      ActivateSkill: { skill, targets, card_targets: cardTargets },
    });
  }

  useSkillCard(
    _actor: number,
    skill: number,
    cardKind: number,
    targets: number[],
    equipSlot: number
  ): Promise<EngineEnvelope> {
    return this.submit({
      UseSkillCard: {
        skill,
        card_kind: cardKind,
        targets,
        equip_slot: equipSlot,
      },
    });
  }

  resolveChoice(
    _actor: number,
    choiceId: number,
    selections: number[],
    cancelled = false
  ): Promise<EngineEnvelope> {
    return this.submit({
      ResolveChoice: {
        choice_id: choiceId,
        selections,
        cancelled,
      },
    });
  }

  surrender(): Promise<EngineEnvelope> {
    return this.submit("Surrender");
  }

  requestSnapshot(): Promise<ObserverSnapshot> {
    const header = this.nextHeader();
    return new Promise<ObserverSnapshot>((resolve, reject) => {
      this.snapshotWaiters.set(header.conn_seq, { resolve, reject });
      try {
        this.send({
          type: "RequestSnapshot",
          payload: { header, last_revision: this.revision },
        });
      } catch (error) {
        this.snapshotWaiters.delete(header.conn_seq);
        reject(error);
      }
    });
  }

  dispose(): void {
    this.disposed = true;
    if (this.reconnectTimer != null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.reconnectAttempts = 0;
    if (this.heartbeat != null && typeof window !== "undefined") {
      window.clearInterval(this.heartbeat);
    }
    this.heartbeat = null;
    this.clockProbe = null;
    this.serverTimeAnchor = null;
    const socket = this.socket;
    this.socket = null;
    socket?.close(1000, "client dispose");
    this.session = null;
    this.roomPassword = "";
    this.playerId = 0;
    this.rejectOutstanding(new Error("连接已关闭"));
    this.envelopeListeners.clear();
    this.rosterListeners.clear();
    this.closeListeners.clear();
    this.reconnectListeners.clear();
  }

  private async joinRequest(message: unknown, password: string): Promise<OnlineSession> {
    const joined = this.waitForJoin();
    try {
      this.send(message);
      this.roomPassword = password;
    } catch (error) {
      this.joinWaiter?.reject(error instanceof Error ? error : new Error(String(error)));
      this.joinWaiter = null;
    }
    return joined;
  }

  private waitForJoin(): Promise<OnlineSession> {
    if (this.joinWaiter) throw new Error("已有入房请求");
    return new Promise((resolve, reject) => {
      this.joinWaiter = { resolve, reject };
    });
  }

  private submit(kind: CommandKind): Promise<EngineEnvelope> {
    if (!this.roomId || !this.playerId) {
      return Promise.reject(new Error("当前不是已入座玩家"));
    }
    if (this.pending.size >= MAX_ENGINE_OUTSTANDING) return Promise.reject(new Error("Engine busy; request was not submitted（未提交）"));
    const key = this.connectionId * 1_000_000 + this.idemSeq++;
    let intent!: PendingCommand;
    const promise = new Promise<EngineEnvelope>((resolve, reject) => {
      intent = { kind, key, resolve, reject };
      this.pending.set(key, intent);
    });
    try {
      this.sendSubmit(intent);
      this.persistedPending = { roomId: this.roomId, key, kind, createdAt: Date.now() };
      storePending(kind, key, this.roomId);
    } catch (error) {
      this.pending.delete(key);
      clearPendingStorage(key);
      return Promise.reject(error);
    }
    return promise;
  }

  /// 用命令自己的 key/内容提交。重连重放与新命令共用这一条路径，
  /// 确保重放绝不会分配新 key 或改变命令内容。
  private sendSubmit(intent: PendingCommand, replay = false): void {
    this.send({
      type: "SubmitCommand",
      payload: {
        header: this.nextHeader(),
        command: {
          actor: this.playerId,
          expected_revision: this.revision,
          idempotency_key: intent.key,
          kind: intent.kind,
        },
        idempotency_key: intent.key,
        replay,
      },
    });
  }

  /// 断线前已发出但未确认的命令在重新入房后**先用原 key 重放**。
  /// WebSocket 保序，因此 UI 在 JoinOk 之后产生的新提交一定排在这些
  /// 重放之后；服务端按 reconnect token + key 返回原 Accepted 或只执行一次。
  private replayPending(): void {
    const persisted =
      this.persistedPending && this.persistedPending.roomId === this.roomId
        ? this.persistedPending
        : null;
    if (persisted && !this.pending.has(persisted.key) && this.pending.size < MAX_ENGINE_OUTSTANDING) {
      // 页面刷新后的新实例没有旧 Promise；用占位 pending 承接服务端回执，
      // 并通过 envelope 监听者把权威结果交回界面。
      const intent: PendingCommand = {
        kind: persisted.kind,
        key: persisted.key,
        resolve: (envelope) => {
          for (const listener of this.envelopeListeners) listener(envelope);
        },
        reject: () => undefined,
      };
      this.pending.set(intent.key, intent);
    }
    if (!this.pending.size) return;
    for (const intent of [...this.pending.values()]) {
      try {
        this.sendSubmit(intent, true);
      } catch {
        // 连接再次断开：保留意图，下次 JoinOk 继续用原 key 结算。
        return;
      }
    }
  }

  private nextHeader(): MessageHeader {
    return {
      protocol_version: NETWORK_PROTOCOL_VERSION,
      room_id: this.roomId,
      conn_seq: ++this.connSeq,
      revision: this.revision,
    };
  }

  private send(message: unknown): void {
    if (this.socket?.readyState !== WebSocket.OPEN) {
      throw new Error("WebSocket 未连接");
    }
    this.socket.send(JSON.stringify(message));
  }

  private handleMessage(json: string): void {
    let message: { type?: string; payload?: Record<string, unknown> };
    try {
      message = JSON.parse(json) as typeof message;
    } catch {
      this.rejectOutstanding(new Error("服务端返回了无效 JSON"));
      return;
    }
    const payload = message.payload ?? {};
    if (Object.hasOwn(payload, "presentation")) {
      try {
        if (new TextEncoder().encode(json).length > 256 * 1024) throw new Error("presentation envelope exceeds 256KiB");
        if (message.type !== "CommandOutcome" || !payload.view) throw new Error("unexpected presentation location");
        const view = payload.view as ObserverSnapshot;
        presentationMetadata(payload.presentation, view.revision, view.seats.length);
      } catch (error) {
        const failure = new Error(`服务端呈现元数据无效，操作结果未确认: ${String(error)}`);
        const key = payload.idempotency_key;
        const header = payload.header as Partial<MessageHeader> | undefined;
        const matchingReply = header?.room_id === this.roomId && typeof header.conn_seq === "number" && Number.isSafeInteger(header.conn_seq) && header.conn_seq > 0;
        const call = matchingReply && typeof key === "number" && Number.isSafeInteger(key) && key > 0 ? this.pending.get(key) : undefined;
        if (call) call.reject(failure);
        else console.error(failure.message);
        return;
      }
    }
    if (payload.view) {
      try { validateObserverMetadata(payload.view as ObserverSnapshot, (payload.choice as ChoiceRequest | null) ?? null); }
      catch (error) { this.rejectOutstanding(new Error(`服务端元数据无效：${String(error)}`)); return; }
    }

    switch (message.type) {
      case "Welcome":
        if (payload.protocol_version !== NETWORK_PROTOCOL_VERSION || (payload.versions as { network_protocol?: number } | undefined)?.network_protocol !== NETWORK_PROTOCOL_VERSION) {
          this.rejectOutstanding(new Error(`服务端协议版本不兼容：需要 WS${NETWORK_PROTOCOL_VERSION}`));
          this.socket?.close();
          return;
        }
        this.connectionId = Number(payload.connection_id ?? 0);
        this.welcomeWaiter?.resolve();
        this.welcomeWaiter = null;
        break;
      case "RoomCreated":
        this.roomId = Number(payload.room_id ?? 0);
        break;
      case "JoinOk": {
        this.roomId = Number(payload.room_id ?? this.roomId);
        this.playerId = Number(payload.player_id ?? 0);
        const view = payload.view as unknown as ObserverSnapshot;
        this.choice = (payload.choice as ChoiceRequest | null | undefined) ?? null;
        this.revision = view.revision;
        const session: OnlineSession = {
          roomId: this.roomId,
          playerId: this.playerId,
          seat: Number(payload.seat ?? 255),
          role: String(payload.role ?? "Spectator") as OnlineSession["role"],
          reconnectToken: String(payload.reconnect_token ?? ""),
          view,
          envelope: envelopeFromView(
            view,
            this.choice,
            [],
            (payload.legal_actions as LegalAction[] | undefined) ?? [],
            null
          ),
        };
        this.session = session;
        // 先同步发出同 key 重放，再 resolve 入房；UI 在 JoinOk 后的新提交
        // 因此一定排在重放之后（WebSocket 保序）。
        this.replayPending();
        this.joinWaiter?.resolve(session);
        this.joinWaiter = null;
        this.startHeartbeat();
        break;
      }
      case "RoomRoster": {
        const roster = payload as unknown as RoomRoster;
        if ((roster.choice_timeout_secs !== null && ![10, 15, 30, 60, 90].includes(roster.choice_timeout_secs)) || !Array.isArray(roster.seats) || roster.seats.some((seat) => typeof seat.managed !== "boolean")) {
          this.rejectOutstanding(new Error("服务端房间超时配置无效")); return;
        }
        for (const listener of this.rosterListeners) listener(roster);
        break;
      }
      case "CommandOutcome":
        this.handleOutcome(payload);
        break;
      case "Snapshot": {
        const header = payload.header as unknown as MessageHeader;
        const view = payload.view as unknown as ObserverSnapshot;
        this.choice = (payload.choice as ChoiceRequest | null | undefined) ?? null;
        this.revision = view.revision;
        const waiter = this.snapshotWaiters.get(header.conn_seq);
        if (waiter) {
          this.snapshotWaiters.delete(header.conn_seq);
          waiter.resolve(view);
        }
        const envelope = envelopeFromView(
          view,
          this.choice,
          [],
          (payload.legal_actions as LegalAction[] | undefined) ?? [],
          null
        );
        for (const listener of this.envelopeListeners) listener(envelope);
        break;
      }
      case "Pong": {
        const header = payload.header as MessageHeader | undefined;
        const probe = this.clockProbe;
        const receivedAt = performance.now();
        const serverTime = payload.server_time_ms;
        if (probe && header?.protocol_version === NETWORK_PROTOCOL_VERSION &&
            header.conn_seq === probe.connSeq && header.room_id === probe.roomId &&
            typeof serverTime === "number" && Number.isSafeInteger(serverTime) && serverTime > 0 &&
            receivedAt >= probe.sentAt) {
          this.serverTimeAnchor = { unixMs: serverTime + (receivedAt - probe.sentAt) / 2, monotonicMs: receivedAt };
          this.clockProbe = null;
        }
        break;
      }
      case "Error": {
        const error = new Error(String(payload.message ?? "服务端错误"));
        if (this.joinWaiter) {
          this.joinWaiter.reject(error);
          this.joinWaiter = null;
        } else {
          this.rejectOutstanding(error);
        }
        break;
      }
    }
  }

  private handleOutcome(payload: Record<string, unknown>): void {
    const view = payload.view as unknown as ObserverSnapshot;
    const previousRevision = this.revision;
    this.revision = view.revision;
    this.choice = (payload.choice as ChoiceRequest | null | undefined) ?? null;
    const eventCodes = (payload.public_event_codes as number[] | undefined) ?? [];
    const envelope = envelopeFromView(
      view,
      this.choice,
      eventCodes,
      (payload.legal_actions as LegalAction[] | undefined) ?? [],
      payload.result ?? null
    );
    if (Object.hasOwn(payload, "presentation")) envelope.presentation = presentationMetadata(payload.presentation, view.revision, view.seats.length);
    const key = Number(payload.idempotency_key ?? 0);
    clearPendingStorage(key);
    if (this.persistedPending?.key === key) this.persistedPending = null;
    const pending = this.pending.get(key);
    if (pending) {
      this.pending.delete(key);
      const rejection = commandRejection(payload.result);
      if (rejection) {
        for (const listener of this.envelopeListeners) listener(envelope);
        pending.reject(new Error(rejection));
      } else pending.resolve(envelope);
    } else {
      for (const listener of this.envelopeListeners) listener(envelope);
    }

    if (previousRevision > 0 && view.revision > previousRevision + 1) {
      void this.requestSnapshot().catch(() => undefined);
    }
  }

  private startHeartbeat(): void {
    if (this.heartbeat != null) window.clearInterval(this.heartbeat);
    this.heartbeat = window.setInterval(() => {
      if (this.socket?.readyState !== WebSocket.OPEN) return;
      this.sendHeartbeat();
    }, 15_000);
  }

  private sendHeartbeat(): void {
    const header = this.nextHeader();
    this.clockProbe = { connSeq: header.conn_seq, roomId: header.room_id, sentAt: performance.now() };
    this.send({ type: "Heartbeat", payload: { header } });
  }

  private handleClose(): void {
    this.clockProbe = null;
    this.serverTimeAnchor = null;
    if (this.heartbeat != null) window.clearInterval(this.heartbeat);
    // NC09：断线必须**可见**——此前服务端中途崩溃时界面保持旧状态、无任何提示。
    // 这里向监听者广播一次「已断开」，由界面写入状态行与日志。
    for (const listener of this.closeListeners) {
      try {
        listener();
      } catch {
        /* 监听者异常不得影响其它监听者 */
      }
    }
    this.heartbeat = null;
    this.socket = null;
    this.playerId = 0;
    if (!this.disposed) {
      // 待确认命令必须跨重连保留：ACK 可能已丢，服务端也许已提交。
      // 入房/欢迎/快照 waiter 无法跨连接，明确失败；pending 留到 JoinOk 重放。
      this.suspendOutstanding(new Error("与服务端的连接已断开"));
      this.scheduleReconnect();
    }
  }

  /// 断线自动重连（指数退避，最多 5 次；成功/失败均广播状态）。
  private scheduleReconnect(): void {
    if (this.reconnectTimer != null) return;
    if (!this.lastUrl) return;
    if (this.reconnectAttempts >= 5) {
      this.failPendingUncertain();
      for (const l of this.reconnectListeners) l("failed");
      return;
    }
    const delay = Math.min(1000 * 2 ** this.reconnectAttempts, 8000);
    this.reconnectAttempts += 1;
    for (const l of this.reconnectListeners) l("reconnecting");
    this.reconnectTimer = window.setTimeout(() => {
      this.reconnectTimer = null;
      const url = this.lastUrl;
      if (!url || this.disposed) return;
      void this.connect(url, this.lastName)
        .then(() => {
          this.reconnectAttempts = 0;
          for (const l of this.reconnectListeners) l("reconnected");
        })
        .catch(() => {
          this.scheduleReconnect();
        });
    }, delay);
  }

  /// 自动重连状态通知（NC09）。
  onReconnect(listener: (state: "reconnecting" | "reconnected" | "failed") => void): () => void {
    this.reconnectListeners.add(listener);
    return () => this.reconnectListeners.delete(listener);
  }

  /// 连接级 waiter 跨不过断线；命令意图 `pending` 有意保留。
  private suspendOutstanding(error: Error): void {
    this.welcomeWaiter?.reject(error);
    this.welcomeWaiter = null;
    this.joinWaiter?.reject(error);
    this.joinWaiter = null;
    for (const pending of this.snapshotWaiters.values()) pending.reject(error);
    this.snapshotWaiters.clear();
  }

  /// 自动重连彻底失败：不能把未确认提交说成已撤销或已拒绝。
  private failPendingUncertain(): void {
    if (!this.pending.size) return;
    for (const pending of this.pending.values()) {
      pending.reject(
        new Error("提交结果不确定：连接已断开且自动重连失败，请重新进入房间后再操作")
      );
    }
    this.pending.clear();
  }

  private rejectOutstanding(error: Error): void {
    this.suspendOutstanding(error);
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
  }
}

function normalizeWsUrl(url: string): string {
  const trimmed = url.trim();
  if (trimmed.startsWith("http://")) return `ws://${trimmed.slice(7)}`;
  if (trimmed.startsWith("https://")) return `wss://${trimmed.slice(8)}`;
  return trimmed;
}

export function onlineReconnectKey(wsUrl: string, roomId: number): string {
  const origin = new URL(normalizeWsUrl(wsUrl)).origin;
  return `noname-next.reconnect.${encodeURIComponent(origin)}.${roomId}`;
}

function envelopeFromView(
  view: ObserverSnapshot,
  choice: ChoiceRequest | null,
  publicEventCodes: number[],
  legalActions: LegalAction[],
  result: unknown
): EngineEnvelope {
  return {
    result,
    events: publicEventCodes.map((code) => ({
      PublicEvent: { code, label: publicEventLabel(code) },
    })),
    choice,
    view,
    legal_actions: legalActions,
    hash_hex: hashHex(view.state_hash),
    revision: view.revision,
    finished: isFinished(view.outcome),
  };
}

function hashHex(hash: ObserverSnapshot["state_hash"]): string {
  if (typeof hash === "string") return hash;
  return hash.map((byte) => byte.toString(16).padStart(2, "0")).join("");
}

function isFinished(outcome: unknown): boolean {
  if (outcome == null || outcome === "InProgress") return false;
  return !(typeof outcome === "object" && "InProgress" in (outcome as object));
}

function commandRejection(result: unknown): string | null {
  if (!result || typeof result !== "object" || !("Rejected" in result)) return null;
  const rejected = (result as { Rejected: unknown }).Rejected;
  const code =
    rejected && typeof rejected === "object" && "IllegalAction" in rejected
      ? (rejected as { IllegalAction?: { code?: unknown } }).IllegalAction?.code
      : undefined;
  if (code === "legacy_replay_no_receipt") {
    return "提交结果不确定：旧房间没有可核对的回执，已停止重放；请以房间当前状态为准";
  }
  return `命令被拒绝：${JSON.stringify(rejected)}`;
}
