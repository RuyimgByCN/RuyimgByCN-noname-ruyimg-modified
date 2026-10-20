//! **对局机器人长程**：按服务端下发的 `legal_actions` 自动选步，驱动**真实对局**多回合推进。
//!
//! 与 `long_run_soak.rs` 的区别：那个只发 `EndPhase`（到「等待选择」即停）；本用例用
//! `LegalAction::to_command` 把**权威合法动作**直接转成命令，因此能越过选择窗口。
//!
//! **三次关键修正**（2026-10-19）：
//! 1. `ResolveChoice` 必须**原样使用权威给出的 `selections`**——自己构造 `selections: vec![]`
//!    会得到 `InvalidChoice`；
//! 2. 每步都要**用最新结果里的 `legal_actions` 覆盖**（过期动作会得到 `not current` / `choice` 拒绝）；
//! 3. 房间有**两条连接**，当前行动者可能是任一方 ⇒ 逐连接维护 `actor` 与 `legal`，谁有动作谁走；
//! 4. **本连接没有合法动作时取 `Snapshot`**——它携带 `choice: Option<ChoiceRequest>`，
//!    据此按 `min_count` 构造 `ResolveChoice{selections: 0..min_count}`（**这才是越过选择窗口的关键**）。
//!
//! 口径：选步策略是**固定优先级**（`ResolveChoice` → `PlayCard`/`ActivateSkill` → `EndPhase`），
//! **不是**强 AI；只用于**驱动状态推进**。断言只看「有实质推进 + 房间始终可响应」。

#[path = "../src/config.rs"]
mod config;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, LegalAction, PlayerId};
use engine_protocol::{MessageHeader, ServerMessage};
use room::{spawn_room, RoomMsg};
use session::Outbound;
use std::time::Instant;
use tokio::sync::mpsc;

const CONNS: usize = 2;

struct Room {
    id: u64,
    handle: room::RoomHandle,
    /// 每条连接的玩家身份（= 该连接的 actor）。
    actors: [PlayerId; CONNS],
    /// 每条连接最近一次收到的合法动作（**该连接视角**）。
    legal: [Vec<LegalAction>; CONNS],
    rev: u64,
    seq: u64,
    cursor: [usize; CONNS],
}

/// 提交一条命令（指定连接），返回 (是否 Accepted, 权威修订号, 该连接视角的合法动作, 原因)。
async fn submit(
    handle: &room::RoomHandle,
    id: u64,
    conn: usize,
    seq: u64,
    cmd: Command,
) -> (bool, u64, Vec<LegalAction>, String) {
    let (rs, ws) = tokio::sync::oneshot::channel();
    if handle
        .send(RoomMsg::Submit {
            connection_id: (conn + 1) as u64,
            header: MessageHeader::new(id, seq, cmd.expected_revision.0),
            command: cmd.clone(),
            idempotency_key: cmd.idempotency_key,
            reply: rs,
        })
        .await
        .is_err()
    {
        return (false, 0, Vec::new(), "send-fail".into());
    }
    match ws.await {
        Ok(Ok(ServerMessage::CommandOutcome {
            result,
            view,
            legal_actions,
            ..
        })) => {
            let ok = matches!(result, engine_api::CommandResult::Accepted { .. });
            let why = if ok {
                "accepted".to_string()
            } else {
                format!("{result:?}").chars().take(44).collect()
            };
            (ok, view.revision, legal_actions, why)
        }
        other => (
            false,
            0,
            Vec::new(),
            format!("{other:?}").chars().take(28).collect(),
        ),
    }
}

/// 取一条连接的个人化快照：返回 (修订号, 合法动作, 待答选择)。
async fn snapshot(
    handle: &room::RoomHandle,
    id: u64,
    conn: usize,
    seq: u64,
    rev: u64,
) -> Option<(u64, Vec<LegalAction>, Option<engine_api::ChoiceRequest>)> {
    let (rs, ws) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Snapshot {
            connection_id: (conn + 1) as u64,
            header: MessageHeader::new(id, seq, rev),
            reply: rs,
        })
        .await
        .ok()?;
    match ws.await {
        Ok(Ok(ServerMessage::Snapshot {
            view,
            choice,
            legal_actions,
            ..
        })) => Some((view.revision, legal_actions, choice)),
        _ => None,
    }
}

/// 固定优先级选步：`ResolveChoice`（**原样**）→ 出牌/技能 → `EndPhase`。
fn pick(legal: &[LegalAction], cursor: &mut usize) -> Option<LegalAction> {
    if legal.is_empty() {
        return None;
    }
    if let Some(a) = legal.iter().find(|a| matches!(a, LegalAction::ResolveChoice { .. })) {
        return Some(a.clone());
    }
    let start = *cursor % legal.len();
    let rotated: Vec<LegalAction> = legal[start..]
        .iter()
        .chain(legal[..start].iter())
        .cloned()
        .collect();
    for a in &rotated {
        if matches!(
            a,
            LegalAction::PlayCard { .. }
                | LegalAction::ActivateSkill { .. }
                | LegalAction::UseSkillCard { .. }
        ) {
            return Some(a.clone());
        }
    }
    rotated.first().cloned()
}

async fn join(handle: &room::RoomHandle, conn: usize, seat: u8, name: &str) -> PlayerId {
    let (tx, _rx) = mpsc::channel::<String>(256);
    let (r, w) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: (conn + 1) as u64,
            display_name: name.into(),
            preferred_seat: Some(seat),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx),
            reply: r,
        })
        .await
        .expect("join send");
    let j = w.await.expect("join reply").expect("join ok");
    PlayerId::new(j.player_id)
}

async fn open(id: u64, seed: u64) -> Room {
    let handle = spawn_room(id, 2, seed, String::new(), 64, 100);
    let p0 = join(&handle, 0, 0, "bot-a").await;
    let p1 = join(&handle, 1, 1, "bot-b").await;
    Room {
        id,
        handle,
        actors: [p0, p1],
        legal: [Vec::new(), Vec::new()],
        rev: 0,
        seq: 0,
        cursor: [0, 0],
    }
}

/// 引导：两条连接各发一条 `EndPhase`，换取**各自视角**的 `legal_actions` 与权威修订号。
async fn bootstrap(room: &mut Room) -> Vec<String> {
    let mut whys = Vec::new();
    for conn in 0..CONNS {
        room.seq += 1;
        let cmd = Command {
            actor: room.actors[conn],
            expected_revision: ExpectedRevision(room.rev),
            idempotency_key: IdempotencyKey(room.seq),
            kind: CommandKind::EndPhase,
        };
        let (_ok, rev, legal, why) = submit(&room.handle, room.id, conn, room.seq, cmd).await;
        if rev > 0 {
            room.rev = rev;
        }
        room.legal[conn] = legal;
        whys.push(why);
    }
    whys
}

/// **机器人长程**：4 房间 × 60 轮，两条连接轮流走；总计最多 4×60 步。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn bot_drives_matches_long_run() {
    const ROOMS: usize = 4;
    const ROUNDS: usize = 60;

    let t0 = Instant::now();
    let mut rooms = Vec::new();
    for r in 0..ROOMS as u64 {
        let mut room = open(r, 7_700 + r).await;
        let whys = bootstrap(&mut room).await;
        eprintln!(
            "DBG bot 引导：room {} rev={} legal=[{}, {}] why={whys:?}",
            room.id,
            room.rev,
            room.legal[0].len(),
            room.legal[1].len()
        );
        rooms.push(room);
    }

    let mut submitted = 0u64;
    let mut accepted = 0u64;
    let mut snapshots = 0u64;
    let mut max_rev = 0u64;
    let mut reasons: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();

    for _round in 0..ROUNDS {
        for room in rooms.iter_mut() {
            // 两条连接：谁有合法动作谁走（一般只有当前行动者那侧非空）
            for conn in 0..CONNS {
                // **刷新**：本连接没有合法动作时，取个人化快照；若快照带「待答选择」，
                // 就按 `min_count` 构造 `ResolveChoice`（selections 为选项下标）。
                if room.legal[conn].is_empty() {
                    if let Some((rev, legal, choice)) =
                        snapshot(&room.handle, room.id, conn, room.seq + 1, room.rev).await
                    {
                        if rev > 0 {
                            room.rev = rev;
                            max_rev = max_rev.max(rev);
                        }
                        room.legal[conn] = legal;
                        if room.legal[conn].is_empty() {
                            if let Some(c) = choice {
                                if c.observer.0 == room.actors[conn].0 {
                                    let take = (c.min_count as usize).min(c.options.len());
                                    let selections: Vec<u32> =
                                        (0..take as u32).collect();
                                    room.legal[conn].push(LegalAction::ResolveChoice {
                                        choice_id: c.choice_id,
                                        selections,
                                        cancelled: take == 0 && c.allow_cancel,
                                    });
                                    eprintln!(
                                        "DBG bot 选择：room {} conn {} choice={:?} min={} opts={} \
                                         构造 selections={:?}",
                                        room.id,
                                        conn,
                                        c.choice_id,
                                        c.min_count,
                                        c.options.len(),
                                        (0..take as u32).collect::<Vec<_>>()
                                    );
                                }
                            }
                        }
                    }
                }
                let Some(action) = pick(&room.legal[conn], &mut room.cursor[conn]) else {
                    continue;
                };
                let cmd = action.to_command(room.actors[conn], room.rev);
                room.seq += 1;
                submitted += 1;
                let (ok, rev, legal, why) =
                    submit(&room.handle, room.id, conn, room.seq, cmd).await;
                *reasons.entry(if ok { "accepted".into() } else { why }).or_insert(0) += 1;
                if ok {
                    accepted += 1;
                    room.cursor[conn] = 0;
                } else {
                    room.cursor[conn] += 1;
                }
                if rev > 0 {
                    room.rev = rev;
                    max_rev = max_rev.max(rev);
                }
                room.legal[conn] = legal;
            }
            let (rs, ws) = tokio::sync::oneshot::channel();
            let sent = room
                .handle
                .send(RoomMsg::Snapshot {
                    connection_id: 1,
                    header: MessageHeader::new(room.id, room.seq, room.rev),
                    reply: rs,
                })
                .await;
            assert!(sent.is_ok(), "room {} 通道应可写", room.id);
            assert!(ws.await.is_ok(), "room {} 必须能响应快照（未死锁）", room.id);
            snapshots += 1;
        }
    }

    let elapsed = t0.elapsed();
    println!(
        "RESULT bot rooms={ROOMS} rounds={ROUNDS} submitted={submitted} accepted={accepted} \
         snapshots={snapshots} max_revision={max_rev} elapsed_ms={} throughput_cmds_per_s={:.0}",
        elapsed.as_millis(),
        submitted as f64 / elapsed.as_secs_f64().max(1e-9)
    );
    eprintln!("DBG bot 终局：reasons={reasons:?}");
    for room in rooms.iter() {
        eprintln!(
            "DBG bot 终局：room {} rev={} legal=[{}, {}]",
            room.id,
            room.rev,
            room.legal[0].len(),
            room.legal[1].len()
        );
    }
    assert!(submitted > 0, "机器人应至少提交过命令");
    assert!(
        max_rev >= 20,
        "两条连接轮流驱动后应把对局推进到较高修订号（max_revision={max_rev}）"
    );
    assert_eq!(snapshots, (ROOMS * ROUNDS) as u64, "快照总数应为 房间×轮次");
}
