#!/usr/bin/env node

const url = process.argv[2] ?? "ws://127.0.0.1:7700/ws";

class Peer {
  constructor(name) {
    this.name = name;
    this.socket = null;
    this.queue = [];
    this.waiters = [];
    this.seq = 0;
    this.idemSeq = 1;
    this.latest = null;
  }

  async connect() {
    this.socket = new WebSocket(url);
    this.socket.onmessage = (event) => {
      const message = JSON.parse(String(event.data));
      if (message.payload?.view) {
        this.latest = {
          view: message.payload.view,
          legalActions: message.payload.legal_actions ?? [],
        };
      }
      if (message.type === "JoinOk") this.playerId = message.payload.player_id;
      const index = this.waiters.findIndex((waiter) => waiter.predicate(message));
      if (index >= 0) this.waiters.splice(index, 1)[0].resolve(message);
      else this.queue.push(message);
    };
    await new Promise((resolve, reject) => {
      this.socket.onopen = resolve;
      this.socket.onerror = () => reject(new Error(`${this.name}: connect failed`));
    });
    const welcome = await this.next("Welcome");
    this.connectionId = welcome.payload.connection_id;
    this.send("Hello", {
      versions: {
        engine_api: 1,
        rule_vm_abi: 3,
        content_schema: 3,
        network_protocol: 4,
        replay_schema: 1,
        snapshot_schema: 1,
        package_format: 1,
        engine_build_id: "websocket-smoke",
      },
      display_name: this.name,
    });
  }

  send(type, payload) {
    this.socket.send(JSON.stringify({ type, payload }));
  }

  header(roomId, revision) {
    return {
      protocol_version: 4,
      room_id: roomId,
      conn_seq: ++this.seq,
      revision,
    };
  }

  next(type, timeoutMs = 3_000) {
    return this.nextWhere((message) => message.type === type, type, timeoutMs);
  }

  nextWhere(predicate, description, timeoutMs = 3_000) {
    const index = this.queue.findIndex(predicate);
    if (index >= 0) return Promise.resolve(this.queue.splice(index, 1)[0]);
    return new Promise((resolve, reject) => {
      let timer;
      const waiter = {
        predicate,
        resolve: (message) => {
          clearTimeout(timer);
          resolve(message);
        },
        reject,
      };
      this.waiters.push(waiter);
      timer = setTimeout(() => {
        const current = this.waiters.indexOf(waiter);
        if (current >= 0) this.waiters.splice(current, 1);
        reject(new Error(`${this.name}: timed out waiting for ${description}`));
      }, timeoutMs);
    });
  }

  async stateAt(revision) {
    if ((this.latest?.view.revision ?? -1) >= revision) return this.latest;
    await this.nextWhere(
      (message) => (message.payload?.view?.revision ?? -1) >= revision,
      `revision ${revision}`
    );
    return this.latest;
  }

  async submit(roomId, action) {
    const revision = this.latest?.view.revision ?? 0;
    const idempotencyKey = this.connectionId * 1_000_000 + this.idemSeq++;
    this.send("SubmitCommand", {
      header: this.header(roomId, revision),
      command: {
        actor: this.playerId,
        expected_revision: revision,
        idempotency_key: idempotencyKey,
        kind: commandKind(action),
      },
      idempotency_key: idempotencyKey,
    });
    return this.nextWhere(
      (message) =>
        message.type === "CommandOutcome" &&
        message.payload.idempotency_key === idempotencyKey,
      `command outcome ${idempotencyKey}`,
      5_000
    );
  }

  close() {
    this.socket?.close(1000, "smoke complete");
  }
}

const host = new Peer("host");
const guest = new Peer("guest");
const reconnect = new Peer("host-reconnect");
const spectator = new Peer("spectator");
const extraPeers = [];

try {
  await host.connect();
  host.send("CreateRoom", { players: 2, seed: 42, password: "smoke" });
  const created = await host.next("RoomCreated");
  const roomId = created.payload.room_id;
  const hostJoin = await host.next("JoinOk");

  await guest.connect();
  guest.send("JoinRoom", {
    room_id: roomId,
    password: "smoke",
    preferred_seat: null,
    reconnect_token: null,
  });
  const guestJoin = await guest.next("JoinOk");
  let roster;
  do {
    roster = await host.next("RoomRoster");
  } while (!roster.payload.started);
  if (!roster.payload.started || roster.payload.seats.length !== 2) {
    throw new Error("room did not auto-start with two seats");
  }

  host.send("RequestSnapshot", {
    header: host.header(roomId, 0),
    last_revision: 0,
  });
  const snapshot = await host.next("Snapshot");
  const revision = snapshot.payload.view.revision;
  const idempotencyKey = host.connectionId * 1_000_000 + 1;
  host.send("SubmitCommand", {
    header: host.header(roomId, revision),
    command: {
      actor: hostJoin.payload.player_id,
      expected_revision: revision,
      idempotency_key: idempotencyKey,
      kind: "EndPhase",
    },
    idempotency_key: idempotencyKey,
  });
  const outcome = await host.next("CommandOutcome");
  if (!("Accepted" in outcome.payload.result)) {
    throw new Error(`EndPhase rejected: ${JSON.stringify(outcome.payload.result)}`);
  }

  await spectator.connect();
  spectator.send("Spectate", { room_id: roomId });
  const spectateJoin = await spectator.next("JoinOk");
  if (spectateJoin.payload.player_id !== 0 || spectateJoin.payload.role !== "Spectator") {
    throw new Error("spectator received a player authority");
  }
  if (spectateJoin.payload.legal_actions.length !== 0) {
    throw new Error("spectator received personalized legal actions");
  }

  host.close();
  await reconnect.connect();
  reconnect.send("JoinRoom", {
    room_id: roomId,
    password: "smoke",
    preferred_seat: null,
    reconnect_token: hostJoin.payload.reconnect_token,
  });
  const rejoined = await reconnect.next("JoinOk");
  if (rejoined.payload.player_id !== hostJoin.payload.player_id) {
    throw new Error("reconnect token did not restore the original player");
  }

  const playersById = new Map([
    [rejoined.payload.player_id, reconnect],
    [guestJoin.payload.player_id, guest],
  ]);
  const reconnectTokens = new Map([
    [rejoined.payload.player_id, hostJoin.payload.reconnect_token],
    [guestJoin.payload.player_id, guestJoin.payload.reconnect_token],
  ]);
  let choiceReconnectTested = false;
  let latestOutcome = outcome;
  let commands = 1;
  const maxCommands = 2_000;
  while (!isFinished(latestOutcome.payload.view.outcome) && commands < maxCommands) {
    const revisionNow = latestOutcome.payload.view.revision;
    const states = await Promise.all(
      Array.from(playersById.values(), async (peer) => [peer, await peer.stateAt(revisionNow)])
    );
    const active = states.find(([, state]) => state.legalActions.length > 0);
    if (!active) {
      throw new Error(
        `no player has a legal action at revision ${revisionNow}: ${JSON.stringify({
          phase: latestOutcome.payload.view.phase,
          current: latestOutcome.payload.view.current_player,
          choice: latestOutcome.payload.choice,
        })}`
      );
    }
    let [peer, state] = active;
    let action = chooseAction(state.legalActions, commands);
    if (
      !choiceReconnectTested &&
      typeof action === "object" &&
      "ResolveChoice" in action
    ) {
      const playerId = peer.playerId;
      const resumed = new Peer(`choice-reconnect-p${playerId}`);
      peer.close();
      await resumed.connect();
      resumed.send("JoinRoom", {
        room_id: roomId,
        password: "smoke",
        preferred_seat: null,
        reconnect_token: reconnectTokens.get(playerId),
      });
      const resumedJoin = await resumed.next("JoinOk");
      if (
        resumedJoin.payload.player_id !== playerId ||
        resumedJoin.payload.choice == null ||
        resumedJoin.payload.legal_actions.length === 0
      ) {
        throw new Error("reconnect at a choice barrier did not restore the choice contract");
      }
      extraPeers.push(resumed);
      playersById.set(playerId, resumed);
      peer = resumed;
      state = resumed.latest;
      action = chooseAction(state.legalActions, commands);
      choiceReconnectTested = true;
    }
    if (action == null) {
      throw new Error(
        `P${peer.playerId} has no legal action at revision ${revisionNow}`
      );
    }
    latestOutcome = await peer.submit(roomId, action);
    if (!("Accepted" in latestOutcome.payload.result)) {
      throw new Error(
        `command rejected at revision ${revisionNow}: ${JSON.stringify(latestOutcome.payload.result)}`
      );
    }
    commands += 1;
  }
  if (!isFinished(latestOutcome.payload.view.outcome)) {
    throw new Error(`full match did not finish within ${maxCommands} commands`);
  }
  if (!choiceReconnectTested) {
    throw new Error("full match never exercised reconnect at a choice barrier");
  }

  console.log(
    `WEBSOCKET SMOKE PASS room=${roomId} players=${hostJoin.payload.player_id},${guestJoin.payload.player_id} commands=${commands} revision=${latestOutcome.payload.view.revision}`
  );
} finally {
  host.close();
  guest.close();
  reconnect.close();
  spectator.close();
  for (const peer of extraPeers) peer.close();
}

function chooseAction(actions, step) {
  const choices = actions.filter(
    (action) => typeof action === "object" && "ResolveChoice" in action
  );
  if (choices.length > 0) return choices.at(-1);
  const plays = actions.filter(
    (action) => typeof action === "object" && "PlayCard" in action
  );
  if (plays.length > 0) return plays[step % plays.length];
  const skills = actions.filter(
    (action) => typeof action === "object" && "ActivateSkill" in action
  );
  if (skills.length > 0 && step % 5 === 0) return skills[step % skills.length];
  if (actions.includes("EndPhase")) return "EndPhase";
  return skills[0] ?? null;
}

function commandKind(action) {
  if (action === "EndPhase") return action;
  if ("PlayCard" in action) {
    return { PlayCard: { ...action.PlayCard, card_targets: [] } };
  }
  return action;
}

function isFinished(outcome) {
  if (outcome == null || outcome === "InProgress") return false;
  return !(typeof outcome === "object" && "InProgress" in outcome);
}
