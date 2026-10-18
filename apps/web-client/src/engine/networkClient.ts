import type {
  ChoiceRequest,
  EngineEnvelope,
  LegalAction,
  ObserverSnapshot,
} from "../types/engine";

const NETWORK_PROTOCOL_VERSION = 5;

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
  }[];
  started: boolean;
  finished: boolean;
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
  private welcomeWaiter: { resolve: () => void; reject: (e: Error) => void } | null = null;
  private joinWaiter: JoinWaiter | null = null;
  private pending = new Map<number, PendingCommand>();
  private snapshotWaiters = new Map<
    number,
    { resolve: (view: ObserverSnapshot) => void; reject: (e: Error) => void }
  >();
  private envelopeListeners = new Set<(envelope: EngineEnvelope) => void>();
  private rosterListeners = new Set<(roster: RoomRoster) => void>();
  private closeListeners = new Set<() => void>();
  private heartbeat: number | null = null;
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
    this.lastUrl = url;
    this.lastName = displayName;

    const welcome = new Promise<void>((resolve, reject) => {
      this.welcomeWaiter = { resolve, reject };
    });
    const opened = new Promise<void>((resolve, reject) => {
      const socket = new WebSocket(normalizeWsUrl(url));
      this.socket = socket;
      socket.onopen = () => resolve();
      socket.onerror = () => reject(new Error("WebSocket 连接失败"));
      socket.onmessage = (event) => this.handleMessage(String(event.data));
      socket.onclose = () => this.handleClose();
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
  }

  createRoom(players: number, seed: number, password = ""): Promise<OnlineSession> {
    const joined = this.waitForJoin();
    this.send({ type: "CreateRoom", payload: { players, seed, password } });
    return joined;
  }

  joinRoom(
    roomId: number,
    password = "",
    reconnectToken: string | null = null
  ): Promise<OnlineSession> {
    this.roomId = roomId;
    const joined = this.waitForJoin();
    this.send({
      type: "JoinRoom",
      payload: {
        room_id: roomId,
        password,
        preferred_seat: null,
        reconnect_token: reconnectToken,
      },
    });
    return joined;
  }

  spectate(roomId: number): Promise<OnlineSession> {
    this.roomId = roomId;
    const joined = this.waitForJoin();
    this.send({ type: "Spectate", payload: { room_id: roomId } });
    return joined;
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
    const promise = new Promise<ObserverSnapshot>((resolve, reject) => {
      this.snapshotWaiters.set(header.conn_seq, { resolve, reject });
    });
    this.send({
      type: "RequestSnapshot",
      payload: { header, last_revision: this.revision },
    });
    return promise;
  }

  dispose(): void {
    this.disposed = true;
    if (this.reconnectTimer != null) {
      window.clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }
    this.reconnectAttempts = 0;
    if (this.heartbeat != null) window.clearInterval(this.heartbeat);
    this.heartbeat = null;
    this.socket?.close(1000, "client dispose");
    this.socket = null;
    this.rejectOutstanding(new Error("连接已关闭"));
    this.envelopeListeners.clear();
    this.rosterListeners.clear();
  }

  private waitForJoin(): Promise<OnlineSession> {
    if (this.joinWaiter) return Promise.reject(new Error("已有入房请求"));
    return new Promise((resolve, reject) => {
      this.joinWaiter = { resolve, reject };
    });
  }

  private submit(kind: CommandKind): Promise<EngineEnvelope> {
    if (!this.roomId || !this.playerId) {
      return Promise.reject(new Error("当前不是已入座玩家"));
    }
    const header = this.nextHeader();
    const idempotencyKey = this.connectionId * 1_000_000 + this.idemSeq++;
    const promise = new Promise<EngineEnvelope>((resolve, reject) => {
      this.pending.set(idempotencyKey, { resolve, reject });
    });
    try {
      this.send({
        type: "SubmitCommand",
        payload: {
          header,
          command: {
            actor: this.playerId,
            expected_revision: this.revision,
            idempotency_key: idempotencyKey,
            kind,
          },
          idempotency_key: idempotencyKey,
        },
      });
    } catch (error) {
      this.pending.delete(idempotencyKey);
      return Promise.reject(error);
    }
    return promise;
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

    switch (message.type) {
      case "Welcome":
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
        this.joinWaiter?.resolve(session);
        this.joinWaiter = null;
        this.startHeartbeat();
        break;
      }
      case "RoomRoster": {
        const roster = payload as unknown as RoomRoster;
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
    const key = Number(payload.idempotency_key ?? 0);
    const pending = this.pending.get(key);
    if (pending) {
      this.pending.delete(key);
      const rejection = commandRejection(payload.result);
      if (rejection) pending.reject(new Error(rejection));
      else pending.resolve(envelope);
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
      const header = this.nextHeader();
      this.send({ type: "Heartbeat", payload: { header } });
    }, 15_000);
  }

  private handleClose(): void {
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
    if (!this.disposed) {
      this.rejectOutstanding(new Error("与服务端的连接已断开"));
      this.scheduleReconnect();
    }
  }

  /// 断线自动重连（指数退避，最多 5 次；成功/失败均广播状态）。
  private scheduleReconnect(): void {
    if (this.reconnectTimer != null) return;
    if (!this.lastUrl) return;
    if (this.reconnectAttempts >= 5) {
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

  private rejectOutstanding(error: Error): void {
    this.welcomeWaiter?.reject(error);
    this.welcomeWaiter = null;
    this.joinWaiter?.reject(error);
    this.joinWaiter = null;
    for (const pending of this.pending.values()) pending.reject(error);
    this.pending.clear();
    for (const pending of this.snapshotWaiters.values()) pending.reject(error);
    this.snapshotWaiters.clear();
  }
}

function normalizeWsUrl(url: string): string {
  const trimmed = url.trim();
  if (trimmed.startsWith("http://")) return `ws://${trimmed.slice(7)}`;
  if (trimmed.startsWith("https://")) return `wss://${trimmed.slice(8)}`;
  return trimmed;
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
      PublicEvent: { code, label: PUBLIC_EVENT_LABELS[code] ?? `事件 ${code}` },
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
  return `命令被拒绝：${JSON.stringify(rejected)}`;
}

const PUBLIC_EVENT_LABELS: Record<number, string> = {
  1: "对局开始",
  2: "摸牌",
  3: "使用卡牌",
  4: "造成伤害",
  5: "回复体力",
  6: "进入濒死",
  7: "玩家阵亡",
  8: "回合推进",
  9: "对局结束",
  10: "等待选择",
};
