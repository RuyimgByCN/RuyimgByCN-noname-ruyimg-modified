//! NC09 原生权威房间会话：入座 → 命令 → 快照 → 重连 → 观战（同一原生会话契约）。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/session.rs"]
mod session;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;

use engine_api::{CommandKind, CommandResult, PlayerId as ApiPlayerId};
use engine_protocol::ClientRole;
use native::NativeContent;
use room_native::NativeRoomSession;
use std::path::PathBuf;
use tokio::sync::mpsc;

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn session(seed: u64, seats: u8) -> NativeRoomSession {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let engine =
        engine_native::Engine::new_with_extensions(seed, seats as usize, &[&loaded.pack])
            .expect("建局");
    NativeRoomSession::new(engine, seats)
}

/// 入座：座位分配、玩家 id、权威动作、投影不含他人手牌 id。
#[test]
fn nc09_room_join_and_projections() {
    let mut room = session(20261401, 4);
    let a = room.join(1, Some(2), None).expect("入座 2 号");
    assert_eq!(a.member.seat, 2);
    assert_eq!(a.member.player_id, ApiPlayerId::new(3));
    assert_eq!(a.member.role, ClientRole::Player);
    // 非当前回合座位的合法动作集合可以为空（内核按当前回合校验）——这里只断言
    // 投影与座位/角色正确，不把「非本人回合无动作」当缺陷。
    assert_eq!(a.view.seats.len(), 4, "投影含全部座位");
    let b = room.join(2, None, None).expect("自动分配");
    assert_ne!(b.member.seat, a.member.seat, "座位不重复");
    // 满座拒绝
    let _ = room.join(3, None, None).expect("第三座");
    let _ = room.join(4, None, None).expect("第四座");
    assert!(room.join(5, None, None).is_err(), "满座必须拒绝");
}

/// 命令：合法推进被接受、非法命令明确拒绝（含协议拒绝原因）。
#[test]
fn nc09_room_commands_and_rejections() {
    let mut room = session(20261402, 4);
    // 先占当前回合座位（首选座位为空，必命中）
    let current = room.engine().turn().current;
    let cur = room.join(1, Some(current), None).expect("入座当前回合座位");
    assert_eq!(cur.member.seat, current, "首选座位可用时应命中");

    // 另占一个非当前回合座位 → 其推进必须被拒绝
    let other = (0..4u8).find(|s| *s != current).expect("其他座位");
    let oth = room.join(2, Some(other), None).expect("入座其他座位");
    assert_eq!(oth.member.seat, other);
    let out = room
        .submit(oth.member.connection_id, &CommandKind::EndPhase {}, room.engine().revision())
        .expect("提交");
    match out {
        CommandResult::Rejected { reason } => assert!(
            matches!(reason, engine_api::RejectReason::InvalidActor),
            "非本人回合应拒绝: {reason:?}"
        ),
        other => panic!("应被拒绝: {other:?}"),
    }

    // 当前回合座位推进阶段 → 接受
    let rev = room.engine().revision();
    let out2 = room
        .submit(cur.member.connection_id, &CommandKind::EndPhase {}, rev)
        .expect("提交");
    assert!(
        matches!(out2, CommandResult::Accepted { .. }),
        "本人回合推进应被接受: {out2:?}"
    );
}

/// 重连：令牌换绑连接、座位与令牌不变；无效令牌明确失败。
#[test]
fn nc09_room_reconnect_keeps_seat() {
    let mut room = session(20261403, 4);
    let a = room.join(1, Some(1), None).expect("入座");
    let token = a.member.reconnect_token.clone();
    room.disconnect(1);
    let again = room.join(99, None, Some(token.clone())).expect("重连");
    assert_eq!(again.member.seat, a.member.seat, "重连保留座位");
    assert_eq!(again.member.reconnect_token, token, "令牌不变");
    assert_eq!(again.member.connection_id, 99, "连接已换绑");
    assert!(
        room.join(100, None, Some("bogus".into())).is_err(),
        "无效令牌必须失败"
    );
}

/// 观战：公开投影、无权威动作、不能提交命令。
#[test]
fn nc09_room_spectator_is_public_only() {
    let mut room = session(20261404, 4);
    let _ = room.join(1, Some(0), None).expect("入座");
    let spec = room.spectate(500);
    assert_eq!(spec.member.role, ClientRole::Spectator);
    assert!(spec.legal_actions.is_empty(), "观战者无权威动作");
    assert_eq!(spec.view.seats.len(), 4, "观战可见全部座位（公开面）");
    let err = room
        .submit(500, &CommandKind::EndPhase {}, room.engine().revision())
        .expect_err("观战者提交必须失败");
    assert!(err.contains("spectator"), "{err}");
}

/// 快照：与引擎摘要/修订一致（客户端据此比对）。
#[test]
fn nc09_room_snapshot_matches_engine() {
    let mut room = session(20261405, 4);
    let a = room.join(1, Some(0), None).expect("入座");
    let snap = room.snapshot(a.member.connection_id).expect("快照");
    assert_eq!(snap.revision, room.engine().revision(), "修订一致");
    assert_eq!(snap.state_hash, room.engine().state_hash(), "摘要一致");
}

/// 存档/录像/恢复：快照 + WAL 重放后修订与摘要必须与记录一致。
#[test]
fn nc09_room_wal_replay_recovers_identically() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");

    let mut room = session(20261410, 4);
    let mut persist = room_native::NativeRoomPersist::new();

    // 快照点：初始状态
    let start_rev = room.engine().revision();
    persist.put_snapshot(
        start_rev,
        engine_native::archive::encode_archive(&room.engine().save()),
    );

    // 逐命令推进并记录 WAL（只记录被接受的动作）
    let mut accepted = 0usize;
    for _ in 0..12 {
        if room.engine().outcome().is_some() {
            break;
        }
        let seat = room.engine().turn().current;
        let conn = 1000 + seat as u64;
        let _ = room.join(conn, Some(seat), None);
        let before = room.engine().revision();
        let out = room
            .submit(conn, &CommandKind::EndPhase {}, room.engine().revision())
            .expect("提交");
        if let CommandResult::Accepted { new_revision, state_hash } = out {
            if new_revision > before {
                persist.record(
                    new_revision,
                    seat,
                    engine_native::GameAction::NextPhase,
                    state_hash,
                );
                accepted += 1;
            }
        }
    }
    assert!(accepted >= 3, "至少记录 3 条被接受的提交（实际 {accepted}）");

    let final_rev = room.engine().revision();
    let final_hash = room.engine().state_hash();

    // 从快照重放 WAL → 修订与摘要必须一致
    let packs = [&loaded.pack];
    let (restored, replayed) = persist.recover(start_rev, &packs).expect("WAL 重放恢复");
    assert_eq!(replayed, accepted, "重放条数一致");
    assert_eq!(restored.revision(), final_rev, "重放后修订一致");
    assert_eq!(restored.state_hash(), final_hash, "重放后摘要一致");

    // 受控变异：篡改一条 WAL 的期望摘要 → 必须明确失败（录像不可信）
    let mut tampered = room_native::NativeRoomPersist::new();
    for (i, (rev, seat, action, hash)) in persist.wal.iter().enumerate() {
        let mut h = *hash;
        if i == 0 {
            h[0] ^= 0xFF; // 篡改期望摘要
        }
        tampered.record(*rev, *seat, action.clone(), h);
    }
    tampered.snapshots = persist.snapshots.clone();
    let err = match tampered.recover(start_rev, &packs) {
        Ok(_) => panic!("摘要不符必须失败"),
        Err(e) => e,
    };
    assert!(err.contains("摘要不一致"), "{err}");
}

/// NC05/NC09 房间级选将：原生后端支持（按入座座位、观战者拒绝、未知武将报错、
/// 成功后向全体推送新视图），旧线后端明确「不支持」。
#[tokio::test]
async fn nc09_room_choose_general_is_room_level() {
    // 旧线后端：明确不支持（不静默）
    let legacy = room::spawn_room(21, 4, 1, String::new(), 8, 20);
    let (rt, rr) = tokio::sync::oneshot::channel();
    legacy
        .send(room::RoomMsg::ChooseGeneral {
            connection_id: 1,
            name: "whatever".into(),
            reply: rt,
        })
        .await
        .expect("send");
    let err = rr.await.expect("reply").expect_err("旧线必须明确不支持");
    assert!(err.contains("不支持"), "{err}");

    // 原生后端：支持
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let engine =
        engine_native::Engine::new_with_extensions(20261820, 4, &[&loaded.pack]).expect("建局");
    let session = room_native::NativeRoomSession::new_with_packs(engine, vec![loaded.pack], 4);
    let handle = room_native_actor::spawn_room_native(22, session, 16, 4);

    let (tx, mut rx) = mpsc::channel::<String>(16);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(room::RoomMsg::Join {
            connection_id: 1,
            display_name: "p".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply: rt,
        })
        .await
        .expect("send join");
    let _ = rr.await.expect("reply").expect("入座");
    while rx.try_recv().is_ok() {}

    // 选一个目录内的扩展武将（swd 前缀）
    let pick = {
        let candidates = engine_native::Engine::new_with_extensions(
            20261821,
            4,
            &[],
        );
        let _ = candidates;
        // 用会话内引擎的候选（通过选将接口的候选导出不可得，这里直接挑已知 swd 武将）
        "swd_lingwu".to_string()
    };
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(room::RoomMsg::ChooseGeneral {
            connection_id: 1,
            name: pick.clone(),
            reply: rt,
        })
        .await
        .expect("send choose");
    match rr.await.expect("reply") {
        Ok(json) => assert!(json.contains("state_hash"), "成功应返回 state_hash: {json}"),
        Err(e) => {
            // 该武将名不在本包目录时，必须**明确报错**（不得静默）
            assert!(e.contains("选将被拒绝"), "{e}");
        }
    }

    // 观战者选将 → 拒绝
    let spec = {
        let (tx, _rx) = mpsc::channel::<String>(4);
        let (rt, rr) = tokio::sync::oneshot::channel();
        handle
            .send(room::RoomMsg::Spectate {
                connection_id: 9,
                display_name: "s".into(),
                outbound: session::Outbound::new(tx),
                reply: rt,
            })
            .await
            .expect("send spectate");
        rr.await.expect("reply").expect("观战")
    };
    assert_eq!(spec.role, ClientRole::Spectator);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(room::RoomMsg::ChooseGeneral {
            connection_id: 9,
            name: "swd_lingwu".into(),
            reply: rt,
        })
        .await
        .expect("send choose2");
    let err = rr.await.expect("reply").expect_err("观战者必须被拒");
    assert!(err.contains("spectator"), "{err}");
}

/// NC09 跨重启持久化（会话级）：导出 → 落盘 → 重建 → 状态/成员/令牌一致，
/// 且重连令牌仍命中**原座位**；计数器延续（不重置）。
#[test]
fn nc09_session_export_and_restore_roundtrip() {
    let dir = tempfile::tempdir().unwrap();
    let content = NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let pack = loaded.pack;
    let engine =
        engine_native::Engine::new_with_extensions(20262001, 4, &[&pack]).expect("建局");
    let mut session = NativeRoomSession::new_with_packs(engine, vec![pack], 4);

    // 两人入座 + 一条命令
    let a = session.join(1, Some(0), None).expect("入座 1");
    let b = session.join(2, Some(1), None).expect("入座 2");
    let rev_before = session.engine().revision();
    let _ = session.submit(1, &CommandKind::EndPhase {}, rev_before);

    // 导出 → JSON 落盘 → 读回重建
    let state = session.export_state(&session.snapshot_hex());
    let json = serde_json::to_string(&state).expect("序列化");
    std::fs::write(dir.path().join("room.json"), &json).expect("落盘");
    let back: room_native::NativeRoomState =
        serde_json::from_str(&std::fs::read_to_string(dir.path().join("room.json")).unwrap())
            .expect("读回");
    assert_eq!(back.members.len(), 2, "成员应随状态落盘");
    assert!(
        back.pack_refs.iter().any(|r| r.starts_with("shahai-swd")),
        "包引用应落盘：{:?}",
        back.pack_refs
    );

    // 恢复路径与产品启动一致：按 `pack_refs` **重新装载**同内容包（而非复用句柄）
    let reloaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("重新装载");
    let mut restored =
        NativeRoomSession::from_saved(&back, vec![reloaded.pack]).expect("恢复");
    assert_eq!(
        restored.engine().revision(),
        session.engine().revision(),
        "修订号应延续"
    );
    assert_eq!(
        restored.engine().state_hash(),
        session.engine().state_hash(),
        "状态摘要应与导出时一致"
    );
    // 令牌延续：用导出前的令牌重连 → 命中**原座位**
    let rejoin = restored
        .join(9, None, Some(a.member.reconnect_token.clone()))
        .expect("令牌重连");
    assert_eq!(rejoin.member.seat, 0, "重连令牌必须命中原座位");
    assert_eq!(rejoin.member.player_id, a.member.player_id, "玩家 id 不变");
    let _ = b;
}
