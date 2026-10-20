//! **对局机器人长程**：按服务端下发的 `legal_actions` 自动选步，驱动**真实对局**多回合推进。
//!
//! 与 `long_run_soak.rs` 的区别：那个只发 `EndPhase`（到「等待选择」即停）；
//! 本用例用 `LegalAction::to_command`（`engine-api` 提供）把**权威合法动作**直接转成命令，
//! 因此能越过选择窗口，做**整局级**长程压测。
//!
//! 口径（**如实**）：
//! * 选步策略是**固定优先级**（先 `PlayCard`/`ActivateSkill`/`UseSkillCard`，再 `EndPhase`，
//!   最后 `ResolveChoice{cancelled:true}`）——**不是**强 AI，只用于**驱动状态推进**；
//! * 全部结果**如实计数**（accepted / 其它），不假设零拒绝；
//! * 断言只看「**有实质推进** + **房间始终可响应**」，不看胜负。

#[path = "../src/config.rs"]
mod config;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, LegalAction, PlayerId};
use engine_protocol::{MessageHeader, ServerMessage};
use room::{spawn_room, RoomMsg};
use session::Outbound;
use std::time::Instant;
use tokio::sync::mpsc;

struct Room {
    id: u64,
    handle: room::RoomHandle,
    actor: PlayerId,
    rev: u64,
    legal: Vec<LegalAction>,
    /// 选步游标：被拒后**轮换**到下一个合法动作，避免死磕同一个。
    cursor: usize,
    seq: u64,
}

/// 提交一条命令，返回 (是否 Accepted, 权威修订号, 个人化合法动作)。
async fn submit(
    handle: &room::RoomHandle,
    id: u64,
    seq: u64,
    cmd: Command,
) -> (bool, u64, Vec<LegalAction>, String, engine_api::PlayerId) {
    let (rs, ws) = tokio::sync::oneshot::channel();
    if handle
        .send(RoomMsg::Submit {
            connection_id: 1,
            header: MessageHeader::new(id, seq, cmd.expected_revision.0),
            command: cmd.clone(),
            idempotency_key: cmd.idempotency_key,
            reply: rs,
        })
        .await
        .is_err()
    {
        return (false, 0, Vec::new(), "send-fail".into(), engine_api::PlayerId::new(0));
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
                format!("{result:?}").chars().take(48).collect()
            };
            (ok, view.revision, legal_actions, why, view.current_player)
        }
        other => (
            false,
            0,
            Vec::new(),
            format!("{other:?}").chars().take(32).collect(),
            engine_api::PlayerId::new(0),
        ),
    }
}

/// 固定优先级选步（**不是** AI，只求推进）；`cursor` 用于被拒后轮换。
fn pick(legal: &[LegalAction], cursor: &mut usize) -> Option<LegalAction> {
    if legal.is_empty() {
        return None;
    }
    let start = *cursor % legal.len();
    let rotated: Vec<&LegalAction> = legal[start..].iter().chain(legal[..start].iter()).collect();
    let legal: Vec<LegalAction> = rotated.into_iter().cloned().collect();
    for a in &legal {
        if matches!(
            a,
            LegalAction::PlayCard { .. } | LegalAction::ActivateSkill { .. } | LegalAction::UseSkillCard { .. }
        ) {
            return Some(a.clone());
        }
    }
    if let Some(a) = legal.iter().find(|a| matches!(a, LegalAction::ResolveChoice { .. })) {
        // 无法得知合法 selections，取**取消**（不合法时会被拒，计入拒绝数，不掩盖）
        if let LegalAction::ResolveChoice { choice_id, .. } = a {
            return Some(LegalAction::ResolveChoice {
                choice_id: *choice_id,
                selections: vec![],
                cancelled: true,
            });
        }
    }
    legal.first().cloned()
}

async fn open(id: u64, seed: u64) -> Room {
    let handle = spawn_room(id, 2, seed, String::new(), 64, 100);
    let (tx1, _rx1) = mpsc::channel::<String>(128);
    let (r1, w1) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "bot".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx1),
            reply: r1,
        })
        .await
        .expect("join");
    let j = w1.await.expect("join reply").expect("join ok");
    let (tx2, _rx2) = mpsc::channel::<String>(128);
    let (r2, w2) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 2,
            display_name: "idle".into(),
            preferred_seat: Some(1),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx2),
            reply: r2,
        })
        .await
        .expect("join2");
    let _ = w2.await.expect("join2 reply").expect("join2 ok");
    Room {
        id,
        handle,
        actor: PlayerId::new(j.player_id),
        rev: 0,
        legal: Vec::new(),
        cursor: 0,
        seq: 0,
    }
}

/// **机器人长程**：4 房间 × 40 轮 × 6 步 = 960 条命令，全部由 `legal_actions` 驱动。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore = "整局机器人未完成：按 legal_actions 固定优先级 + 被拒轮换只能推进到 rev≈9 \
            （960 条中仅 14 条被接受）；继续推进需要按 choice 语义构造精确 selections \
            与正确的 actor，见 2026-10-19-product-bot 证据"]
async fn bot_drives_matches_long_run() {
    const ROOMS: usize = 4;
    const ROUNDS: usize = 40;
    const PER_ROUND: usize = 6;

    let t0 = Instant::now();
    let mut rooms = Vec::new();
    for r in 0..ROOMS as u64 {
        rooms.push(open(r, 7_700 + r).await);
    }

    // **引导步**：先发一条 `EndPhase` 换取个人化 `legal_actions`（否则机器人无从选步）。
    for room in rooms.iter_mut() {
        room.seq += 1;
        let cmd = Command {
            actor: room.actor,
            expected_revision: engine_api::ExpectedRevision(room.rev),
            idempotency_key: engine_api::IdempotencyKey(room.seq),
            kind: engine_api::CommandKind::EndPhase,
        };
        let (_ok, rev, legal, why, cur) = submit(&room.handle, room.id, room.seq, cmd).await;
        eprintln!("DBG bot 引导结果：room {} why={why}", room.id);
        if rev > 0 {
            room.rev = rev;
        }
        room.legal = legal;
        if cur.0 != room.actor.0 {
            room.actor = cur; // **关键**：actor 跟随 `view.current_player`
        }
        eprintln!(
            "DBG bot 引导：room {} actor={} rev={} legal_actions={}",
            room.id,
            room.actor.0,
            room.rev,
            room.legal.len()
        );
    }

    let mut submitted = 0u64;
    let mut accepted = 0u64;
    let mut snapshots = 0u64;
    let mut max_rev = 0u64;
    let mut reasons: std::collections::BTreeMap<String, u64> = std::collections::BTreeMap::new();
    for _round in 0..ROUNDS {
        for room in rooms.iter_mut() {
            for _ in 0..PER_ROUND {
                let Some(action) = pick(&room.legal, &mut room.cursor) else { break };
                // **重试**：若被拒的原因是「不是当前行动者」，用新 actor 重发**同一动作**一次。
                let mut ok = false;
                let mut rev = 0u64;
                let mut legal_now = Vec::new();
                let mut why = String::new();
                for _attempt in 0..2 {
                    let cmd = action.to_command(room.actor, room.rev);
                    room.seq += 1;
                    submitted += 1;
                    let (a, r, l, w, cur) = submit(&room.handle, room.id, room.seq, cmd).await;
                    ok = a;
                    rev = r;
                    legal_now = l;
                    why = w;
                    if ok {
                        break;
                    }
                    // 首次被拒且行动者变化 ⇒ 更新 actor 后重试
                    if cur.0 != 0 && cur.0 != room.actor.0 {
                        room.actor = cur;
                        continue;
                    }
                    break;
                }
                let legal = legal_now;
                *reasons.entry(if ok { "accepted".to_string() } else { why }).or_insert(0u64) += 1;
                if ok {
                    accepted += 1;
                    room.cursor = 0; // 成功则回到首个优先动作
                } else {
                    room.cursor += 1; // 被拒则轮换
                }
                if rev > 0 {
                    room.rev = rev;
                    max_rev = max_rev.max(rev);
                }
                if !legal.is_empty() {
                    room.legal = legal;
                }
            }
            // 探活
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
        "RESULT bot rooms={ROOMS} rounds={ROUNDS} per_round={PER_ROUND} submitted={submitted} \
         accepted={accepted} snapshots={snapshots} max_revision={max_rev} elapsed_ms={} \
         throughput_cmds_per_s={:.0}",
        elapsed.as_millis(),
        submitted as f64 / elapsed.as_secs_f64().max(1e-9)
    );
    eprintln!("DBG bot 终局：reasons={reasons:?}");
    for room in rooms.iter() {
        eprintln!(
            "DBG bot 终局：room {} actor={} rev={} legal={} 变体={:?}",
            room.id,
            room.actor.0,
            room.rev,
            room.legal.len(),
            room.legal
                .iter()
                .map(|a| format!("{a:?}").chars().take(28).collect::<String>())
                .collect::<Vec<_>>()
        );
    }
    assert!(submitted > 0, "机器人应至少提交过命令");
    // 当前实现只能推进到 ~9；提高门槛前需先解决「精确 selections / 正确 actor」两件事。
    assert!(
        max_rev >= 5,
        "机器人应至少把对局推进若干步（max_revision={max_rev}）"
    );
    assert!(snapshots == (ROOMS * ROUNDS) as u64, "快照总数应为 房间×轮次");
}
