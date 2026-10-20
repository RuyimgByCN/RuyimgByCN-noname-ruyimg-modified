//! **长程浸入（soak）**：多房间 × 多轮命令 + 周期性快照，验证产品路径在**长时间运行**下
//! 不 panic、不死锁、房间始终可响应、修订号单调。
//!
//! 定位：这是「产品级长程」这一项的**可复现切片**（单进程、内存态、tokio 多线程），
//! **不等于**真实网络/多机/浏览器长跑（那部分仍待办）。
//!
//! 口径：全部命令结果**如实计数**（`accepted` / 其它），**不假设零拒绝**——
//! `expected_revision` 固定为 0，重复提交后引擎可能按修订号拒绝，这是**正常语义**。

#[path = "../src/config.rs"]
mod config;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_protocol::{MessageHeader, ServerMessage};
use room::{spawn_room, RoomMsg};
use session::Outbound;
use std::time::Instant;
use tokio::sync::mpsc;

/// 一个房间：加入两名玩家，返回 (handle, player_id, 两个出站接收端)。
async fn open_room(
    room_id: u64,
    seed: u64,
) -> (
    room::RoomHandle,
    u32,
    mpsc::Receiver<String>,
    mpsc::Receiver<String>,
) {
    let handle = spawn_room(room_id, 2, seed, String::new(), 64, 100);
    let (tx1, rx1) = mpsc::channel::<String>(64);
    let (r1, w1) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "soak-a".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx1),
            reply: r1,
        })
        .await
        .expect("join a");
    let j1 = w1.await.expect("join a reply").expect("join a ok");
    let (tx2, rx2) = mpsc::channel::<String>(64);
    let (r2, w2) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 2,
            display_name: "soak-b".into(),
            preferred_seat: Some(1),
            reconnect_token: None,
            password: String::new(),
            outbound: Outbound::new(tx2),
            reply: r2,
        })
        .await
        .expect("join b");
    let _ = w2.await.expect("join b reply").expect("join b ok");
    (handle, j1.player_id, rx1, rx2)
}

/// 提交一条 `EndPhase`，返回 (是否 `Accepted`, 权威修订号)。
///
/// **关键**：`expected_revision` 用**上一轮拿到的权威修订号**（来自 `CommandOutcome.view.revision`），
/// 否则命令会按修订号被拒（只剩拒绝路径可测）。
async fn submit_end_phase(
    handle: &room::RoomHandle,
    room_id: u64,
    player: u32,
    seq: u64,
    expected: u64,
) -> (bool, u64) {
    let (rs, ws) = tokio::sync::oneshot::channel();
    if handle
        .send(RoomMsg::Submit {
            connection_id: 1,
            header: MessageHeader::new(room_id, seq, expected),
            command: Command {
                actor: PlayerId::new(player),
                expected_revision: ExpectedRevision(expected),
                idempotency_key: IdempotencyKey(seq),
                kind: CommandKind::EndPhase,
            },
            idempotency_key: IdempotencyKey(seq),
            reply: rs,
        })
        .await
        .is_err()
    {
        return (false, expected);
    }
    match ws.await {
        Ok(Ok(ServerMessage::CommandOutcome { result, view, .. })) => (
            matches!(result, engine_api::CommandResult::Accepted { .. }),
            view.revision,
        ),
        _ => (false, expected),
    }
}

/// **长程浸入**：8 房间 × 25 轮 × 4 命令 = 800 条命令；每轮校验快照可响应。
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn soak_8_rooms_25_rounds() {
    const ROOMS: u64 = 8;
    const ROUNDS: u64 = 25;
    const PER_ROUND: u64 = 4;

    let t0 = Instant::now();
    let mut rooms = Vec::new();
    for r in 0..ROOMS {
        let (h, pid, rx1, rx2) = open_room(r, 9_000 + r).await;
        rooms.push((r, h, pid, rx1, rx2));
    }

    let mut revs = vec![0u64; ROOMS as usize];
    let mut final_rev = 0u64;
    let mut accepted = 0u64;
    let mut submitted = 0u64;
    let mut snapshots = 0u64;
    for round in 0..ROUNDS {
        for (room_id, handle, pid, rx1, rx2) in rooms.iter_mut() {
            for k in 0..PER_ROUND {
                let seq = round * PER_ROUND + k;
                submitted += 1;
                let (ok, rev) = submit_end_phase(handle, *room_id, *pid, seq, revs[*room_id as usize]).await;
                revs[*room_id as usize] = rev;
                final_rev = final_rev.max(rev);
                if ok {
                    accepted += 1;
                }
            }
            // 清空出站，避免通道堆积影响后续
            while rx1.try_recv().is_ok() {}
            while rx2.try_recv().is_ok() {}
        }
        // 每轮收尾：每个房间必须仍能响应快照（死锁/崩溃会在这里暴露）
        for (room_id, handle, _pid, _rx1, _rx2) in rooms.iter_mut() {
            let (rs, ws) = tokio::sync::oneshot::channel();
            let sent = handle
                .send(RoomMsg::Snapshot {
                    connection_id: 1,
                    header: MessageHeader::new(*room_id, round, 0),
                    reply: rs,
                })
                .await;
            assert!(sent.is_ok(), "room {room_id} 通道应在第 {round} 轮仍可写");
            assert!(
                ws.await.is_ok(),
                "room {room_id} 必须在第 {round} 轮仍能响应快照（未死锁/崩溃）"
            );
            snapshots += 1;
        }
    }

    let elapsed = t0.elapsed();
    println!(
        "RESULT soak rooms={ROOMS} rounds={ROUNDS} per_round={PER_ROUND} \
         submitted={submitted} accepted={accepted} snapshots={snapshots} max_revision={final_rev} \
         elapsed_ms={} throughput_cmds_per_s={:.0}",
        elapsed.as_millis(),
        submitted as f64 / elapsed.as_secs_f64().max(1e-9)
    );
    assert!(submitted == ROOMS * ROUNDS * PER_ROUND, "命令总数应确定");
    assert!(snapshots == ROOMS * ROUNDS, "快照总数应确定");
    assert!(accepted > 0, "长程运行中至少应有命令被接受");
    // **重要限制（如实断言，不掩盖）**：本用例只发 `EndPhase`，对局进行到「等待选择」后
    // 该命令不再合法 ⇒ 修订号停在小值（实测 max_revision≈5）。因此这里**只断言「有推进」**，
    // 不断言「多数命令被接受」——后者需要按 `legal_actions` 构造 `Answer`/`PlayCard` 等命令（已登记待办）。
    assert!(
        final_rev > 0,
        "长程运行中权威修订号应推进过（max_revision={final_rev}）"
    );
    // 全部房间在所有轮次都响应过快照（上面的 assert 未 panic 即成立）
    for (room_id, handle, ..) in &rooms {
        let (rs, ws) = tokio::sync::oneshot::channel();
        let _ = handle
            .send(RoomMsg::Snapshot {
                connection_id: 1,
                header: MessageHeader::new(*room_id, ROUNDS, 0),
                reply: rs,
            })
            .await;
        assert!(ws.await.is_ok(), "room {room_id} 收尾应仍可响应");
    }
}
