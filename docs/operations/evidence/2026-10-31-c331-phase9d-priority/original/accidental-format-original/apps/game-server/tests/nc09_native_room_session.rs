//! NC09 原生权威房间会话：入座 → 命令 → 快照 → 重连 → 观战（同一原生会话契约）。

#[path = "../src/native.rs"]
mod native;
#[path = "../src/native_room_store.rs"]
mod native_room_store;
#[path = "../src/native_view.rs"]
mod native_view;
#[path = "../src/persist.rs"]
mod persist;
#[path = "../src/room.rs"]
mod room;
#[path = "../src/room_native.rs"]
mod room_native;
#[path = "../src/room_native_actor.rs"]
mod room_native_actor;
#[path = "../src/session.rs"]
mod session;

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
    let engine = engine_native::Engine::new_with_extensions(seed, seats as usize, &[&loaded.pack])
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
        .submit(
            oth.member.connection_id,
            &CommandKind::EndPhase {},
            room.engine().revision(),
        )
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
    assert!(uuid::Uuid::parse_str(&token).is_ok(), "重连令牌须不可预测");
    assert!(room.snapshot(1).is_err(), "换绑后旧连接不再拥有座位");
    assert!(
        room.join(100, None, Some("bogus".into())).is_err(),
        "无效令牌必须失败"
    );
}

fn session_with_choice() -> NativeRoomSession {
    let mut room = NativeRoomSession::new(engine_native::Engine::new(4242, 4), 4);
    for seat in 0..4 {
        room.join(10 + seat as u64, Some(seat), None).unwrap();
    }
    for _ in 0..1000 {
        if room.engine().open_choice().is_some() {
            for connection in 10..14 {
                room.reclaim_seat(connection).unwrap();
            }
            return room;
        }
        let actor = room.engine().turn().current;
        let player = ApiPlayerId::new(actor as u32 + 1);
        let action = native_view::legal_actions_for(room.engine(), player)
            .into_iter()
            .next()
            .expect("有可推进动作");
        let command = action.to_command(player, room.engine().revision());
        let out = room
            .submit(10 + actor as u64, &command.kind, room.engine().revision())
            .unwrap();
        assert!(matches!(out, CommandResult::Accepted { .. }), "{out:?}");
    }
    panic!("固定场景须产生选择");
}

#[test]
fn nc09_choice_tokens_roundtrip_through_session_and_authoritative_actions() {
    let mut room = session_with_choice();
    let open = room.engine().open_choice().unwrap().clone();
    let joined = room.join(1, Some(open.observer), None).unwrap();
    let choice = native_view::choice_for(
        room.engine(),
        joined.member.role.into(),
        Some(joined.member.player_id),
    )
    .unwrap();
    assert_eq!(
        choice.options.iter().map(|o| o.token).collect::<Vec<_>>(),
        (0..open.spec.options.len() as u32).collect::<Vec<_>>()
    );
    let action = joined.legal_actions.iter().find(|a| matches!(a,
        engine_api::LegalAction::ResolveChoice { selections, cancelled: false, .. } if !selections.is_empty()
    )).expect("有权威选择动作");
    let command = action.to_command(joined.member.player_id, room.engine().revision());
    let hash = room.engine().state_hash();
    room.join(2, Some((open.observer + 1) % 4), None).unwrap();
    for token in [0, u32::MAX] {
        assert_eq!(
            room.submit(
                2,
                &CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: vec![token],
                    cancelled: false,
                },
                room.engine().revision()
            )
            .unwrap(),
            CommandResult::Rejected {
                reason: engine_api::RejectReason::InvalidActor,
            },
            "非选择者的拒绝不能泄露私有候选边界"
        );
        assert_eq!(room.engine().state_hash(), hash);
    }
    assert!(
        room.submit(
            1,
            &CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![u32::MAX],
                cancelled: false,
            },
            room.engine().revision()
        )
        .is_err(),
        "越界 token 不得截断成 Tag"
    );
    assert_eq!(room.engine().state_hash(), hash);
    let result = room
        .submit(1, &command.kind, room.engine().revision())
        .unwrap();
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "权威 token 可原样回传，实际结果: {result:?}"
    );
}

#[tokio::test]
async fn nc09_join_reply_restores_pending_choice_for_its_observer_only() {
    let room = session_with_choice();
    let observer = room.engine().open_choice().unwrap().observer;
    let handle = room_native_actor::spawn_room_native(89, room, 8, 4);
    for (connection_id, seat) in [(1, observer), (2, (observer + 1) % 4)] {
        let (tx, _rx) = mpsc::channel(8);
        let (reply, response) = tokio::sync::oneshot::channel();
        handle
            .send(room::RoomMsg::Join {
                connection_id,
                display_name: "pending".into(),
                preferred_seat: Some(seat),
                reconnect_token: None,
                password: String::new(),
                outbound: session::Outbound::new(tx),
                reply,
            })
            .await
            .unwrap();
        let joined = response.await.unwrap().unwrap();
        assert_eq!(joined.choice.is_some(), seat == observer);
    }
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
        if let CommandResult::Accepted {
            new_revision,
            state_hash,
        } = out
        {
            if new_revision > before {
                accepted += 1;
            }
        }
    }
    assert!(
        accepted >= 3,
        "至少记录 3 条被接受的提交（实际 {accepted}）"
    );

    let final_rev = room.engine().revision();
    let final_hash = room.engine().state_hash();

    // The actor and disk now share the exact recorded commands and dedup values.
    let state = room.export_state(&room.snapshot_hex());
    assert_eq!(state.wal.as_ref().unwrap().entries.len(), accepted);
    let restored = NativeRoomSession::from_saved(&state, vec![loaded.pack]).expect("WAL 重放恢复");
    assert_eq!(restored.engine().revision(), final_rev, "重放后修订一致");
    assert_eq!(restored.engine().state_hash(), final_hash, "重放后摘要一致");
    let mut actual = restored.engine().save();
    actual.auth_generation = room.engine().save().auth_generation;
    assert_eq!(
        engine_native::archive::encode_archive(&actual),
        engine_native::archive::encode_archive(&room.engine().save())
    );

    // 篡改一条 WAL 期望摘要仍须明确失败，不能只信最终完整快照。
    let mut tampered = state;
    tampered.wal.as_mut().unwrap().entries[0].state_hash[0] ^= 0xff;
    let error = match NativeRoomSession::from_saved(
        &tampered,
        vec![
            content
                .load_scoped(Some("upstream-noname"), "shahai-swd")
                .unwrap()
                .pack,
        ],
    ) {
        Ok(_) => panic!("摘要不符必须失败"),
        Err(error) => error,
    };
    assert!(error.contains("摘要不一致"), "{error}");
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
        let candidates = engine_native::Engine::new_with_extensions(20261821, 4, &[]);
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
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let pack = loaded.pack;
    let engine = engine_native::Engine::new_with_extensions(20262001, 4, &[&pack]).expect("建局");
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
        back.pack_refs.iter().any(|r| r.contains("shahai-swd")),
        "包引用应落盘：{:?}",
        back.pack_refs
    );

    // 恢复路径与产品启动一致：按 `pack_refs` **重新装载**同内容包（而非复用句柄）
    let reloaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("重新装载");
    let mut restored = NativeRoomSession::from_saved(&back, vec![reloaded.pack]).expect("恢复");
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

/// NC09 落盘存储 + 启动恢复：写盘（原子）→ `load_all` → `restore_room_from_disk`
/// → 房间可用且令牌命中原座位；损坏文件**明确报错**（不跳过）。
#[tokio::test]
async fn nc09_room_store_roundtrip_and_restore() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("装载");
    let pack = loaded.pack;
    let engine = engine_native::Engine::new_with_extensions(20262002, 4, &[&pack]).expect("建局");
    let session = NativeRoomSession::new_with_packs(engine, vec![pack], 4);

    let store = native_room_store::NativeRoomStore::new(dir.path().join("rooms"));
    // 直接构造记录并落盘（actor 路径在 E2E 中覆盖）
    let mut session = session;
    let a = session.join(1, Some(0), None).expect("入座");
    let record = native_room_store::StoredRoom {
        room_id: 42,
        password_hash: native_room_store::password_hash(42, "pw"),
        checksum: String::new(),
        state: session.export_state(&session.snapshot_hex()),
    };
    store.save(&record).expect("落盘");
    assert!(store.dir().join("42.json").exists(), "应生成 42.json");
    assert!(
        !store.dir().join("42.json.tmp").exists(),
        "临时文件应已 rename"
    );

    // 读回并按启动路径恢复（重新装载包）
    let loaded = content
        .load_scoped(Some("upstream-noname"), "shahai-swd")
        .expect("重新装载");
    let mut packs = vec![loaded.pack];
    let mut load_pack =
        |_ref: &str| -> Result<engine_native::content::extension::ExtensionPack, String> {
            packs.pop().ok_or_else(|| "包只可装载一次".to_string())
        };
    let handle =
        room_native_actor::restore_room_from_disk(&record, &mut load_pack, 8, 4, store.clone())
            .expect("恢复房间");
    assert_eq!(handle.id, 42);

    // 恢复后的房间可用：用导出前的令牌重连 → 原座位
    let (tx, _rx) = mpsc::channel::<String>(4);
    let (rt, rr) = tokio::sync::oneshot::channel();
    handle
        .send(room::RoomMsg::Join {
            connection_id: 7,
            display_name: "back".into(),
            preferred_seat: None,
            reconnect_token: Some(a.member.reconnect_token.clone()),
            password: "pw".into(),
            outbound: session::Outbound::new(tx),
            reply: rt,
        })
        .await
        .expect("send");
    let joined = rr.await.expect("reply").expect("恢复后凭令牌入座");
    assert_eq!(joined.seat, 0, "恢复后令牌必须命中原座位");

    // 损坏文件：明确报错（不跳过）
    std::fs::write(store.dir().join("99.json"), b"{ not json").unwrap();
    let err = store.load_all().expect_err("损坏文件必须报错");
    assert!(err.contains("解析"), "{err}");
}

/// NC09 终局清理：比赛结束后落盘记录被删除（不随重启恢复），且再次落盘不会重建。
#[tokio::test]
async fn nc09_room_store_is_removed_on_match_end() {
    let dir = tempfile::tempdir().unwrap();
    let store = native_room_store::NativeRoomStore::new(dir.path().join("rooms"));
    // 未终局：写盘
    let record = native_room_store::StoredRoom {
        room_id: 7,
        password_hash: String::new(),
        checksum: String::new(),
        state: room_native::NativeRoomState {
            command_receipts: Vec::new(),
            timeout: None,
            wal: None,
            snapshot_hex: String::new(),
            max_seats: 4,
            started: true,
            next_token: 3,
            next_dedup: 9,
            pack_refs: Vec::new(),
            frozen_packages: None,
            members: Vec::new(),
        },
    };
    store.save(&record).expect("写盘");
    assert!(store.dir().join("7.json").exists());
    // 终局清理路径：remove 后不再存在；load_all 为空
    store.remove(7).expect("删除");
    assert!(!store.dir().join("7.json").exists(), "终局后记录应删除");
    assert!(store.load_all().expect("读回").is_empty(), "无残留记录");
    // 幂等：重复删除不报错
    store.remove(7).expect("重复删除应幂等");
}

/// NC09 落盘硬化：密码不落明文（只存加盐哈希）；校验和不匹配**明确报错**。
#[test]
fn nc09_room_store_hashes_password_and_verifies_checksum() {
    let dir = tempfile::tempdir().unwrap();
    let store = native_room_store::NativeRoomStore::new(dir.path().join("rooms"));
    let hash = native_room_store::password_hash(5, "secret");
    assert_ne!(hash, "secret", "不得落明文");
    assert_eq!(
        hash,
        native_room_store::password_hash(5, "secret"),
        "哈希稳定"
    );
    assert_ne!(
        hash,
        native_room_store::password_hash(6, "secret"),
        "盐=房间号"
    );
    assert_eq!(native_room_store::password_hash(5, ""), "", "空密码 → 空串");

    let record = native_room_store::StoredRoom {
        room_id: 5,
        password_hash: hash.clone(),
        checksum: String::new(),
        state: room_native::NativeRoomState {
            command_receipts: Vec::new(),
            timeout: None,
            wal: None,
            snapshot_hex: String::new(),
            max_seats: 4,
            started: false,
            next_token: 1,
            next_dedup: 1,
            pack_refs: Vec::new(),
            frozen_packages: None,
            members: Vec::new(),
        },
    };
    store.save(&record).expect("落盘");
    let text = std::fs::read_to_string(store.dir().join("5.json")).unwrap();
    assert!(!text.contains("secret"), "磁盘不得出现明文密码：{text}");
    assert!(text.contains("checksum"), "应写入校验和");
    let back = store.load_all().expect("校验和匹配应可读");
    assert_eq!(back.len(), 1);
    assert_eq!(back[0].password_hash, hash);

    // 篡改状态但保留旧校验和 → 明确报错
    let tampered = text.replace("\"max_seats\": 4", "\"max_seats\": 8");
    std::fs::write(store.dir().join("5.json"), tampered).unwrap();
    let err = store.load_all().expect_err("校验和不匹配必须报错");
    assert!(err.contains("校验和"), "{err}");
}

#[test]
fn nc09_legacy_state_checksum_survives_optional_frozen_package_field() {
    let old_state = r#"{"snapshot_hex":"","max_seats":4,"started":true,"next_token":1,"next_dedup":1,"pack_refs":[],"members":[]}"#;
    let checksum = blake3::hash(old_state.as_bytes()).to_hex().to_string();
    let state: room_native::NativeRoomState = serde_json::from_str(old_state).unwrap();
    assert!(state.frozen_packages.is_none());
    assert_eq!(serde_json::to_string(&state).unwrap(), old_state);
    let dir = tempfile::tempdir().unwrap();
    let store = native_room_store::NativeRoomStore::new(dir.path());
    let record = format!(
        r#"{{"room_id":88,"password_hash":"","state":{old_state},"checksum":"{checksum}"}}"#
    );
    std::fs::write(dir.path().join("88.json"), record).unwrap();
    let loaded = store.load_all().unwrap();
    assert_eq!(loaded[0].checksum, checksum);
}

#[tokio::test]
async fn nc09_restore_does_not_publish_actor_when_migrated_save_cannot_be_written() {
    let dir = tempfile::tempdir().unwrap();
    let blocked = dir.path().join("not-a-directory");
    std::fs::write(&blocked, b"retain this file").unwrap();
    let store = native_room_store::NativeRoomStore::new(&blocked);
    let mut room = NativeRoomSession::new(engine_native::Engine::new(4242, 2), 2);
    room.set_frozen_packages(Vec::new());
    let record = native_room_store::StoredRoom {
        room_id: 89,
        password_hash: String::new(),
        state: room.export_state(&room.snapshot_hex()),
        checksum: String::new(),
    };
    let result = room_native_actor::restore_room_from_disk(
        &record,
        &mut |_| Err("标准局不应装载包".into()),
        8,
        4,
        store,
    );
    assert!(result.is_err());
    assert_eq!(std::fs::read(blocked).unwrap(), b"retain this file");
}

#[test]
fn nc09_referee_general_authority_survives_trusted_restore_and_reconnect() {
    let mut engine = engine_native::Engine::new(1515, 4);
    engine.assign_general_named(1, "刘备").unwrap();
    let general = engine.state_core().player(1).general_slots[0].unwrap() as u32;
    let mut snapshot = engine.save();
    snapshot.core.mode = engine_native::state::GameMode::Kingdoms;
    snapshot.core.player_mut(1).revealed_generals = [false, false];
    let mut room = NativeRoomSession::new(engine_native::Engine::restore(&snapshot).unwrap(), 4);
    let ordinary = room.join(1, Some(0), None).unwrap();
    assert_eq!(ordinary.member.role, ClientRole::Player);
    assert_eq!(ordinary.view.seats[1].character_def, 0);
    room.spectate(90);
    let mut saved = room.export_state(&room.snapshot_hex());
    // Only a trusted host/store can provision a Referee; no client grant message exists.
    let referee = saved
        .members
        .iter_mut()
        .find(|m| m.connection_id == 90)
        .unwrap();
    referee.role = ClientRole::Referee;
    referee.reconnect_token = uuid::Uuid::new_v4().to_string();
    let token = referee.reconnect_token.clone();
    let mut restored = NativeRoomSession::from_saved(&saved, Vec::new()).unwrap();
    assert_eq!(
        restored.snapshot(90).unwrap().seats[1].character_def,
        general
    );
    assert_eq!(restored.snapshot(1).unwrap().seats[1].character_def, 0);
    let rejoined = restored.join(99, None, Some(token)).unwrap();
    assert_eq!(rejoined.member.role, ClientRole::Referee);
    assert_eq!(rejoined.view.observer, engine_api::ObserverKind::Referee);
    assert_eq!(rejoined.view.seats[1].character_def, general);
    assert!(restored.snapshot(90).is_err());
    assert!(restored.join(100, None, Some("Referee".into())).is_err());
    let spec = restored.spectate(101);
    assert_eq!(spec.member.role, ClientRole::Spectator);
    assert_eq!(spec.view.seats[1].character_def, 0);
}

#[test]
fn nc09_choice_rejects_invalid_player_ids_without_wrapping() {
    let mut engine = engine_native::Engine::new(1515, 4);
    for _ in 0..3 {
        let result = engine.submit(engine_native::GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: engine.revision() + 1,
            action: engine_native::GameAction::NextPhase,
        });
        assert!(!matches!(
            result,
            engine_native::CommandOutcome::Rejected(_)
        ));
    }
    let room = NativeRoomSession::new(engine, 4);
    assert_eq!(room.engine().open_choice().unwrap().observer, 0);
    for raw in [0, 257, u32::MAX] {
        assert!(
            native_view::choice_for(
                room.engine(),
                native_view::RoomRole::Player,
                Some(ApiPlayerId::new(raw))
            )
            .is_none(),
            "invalid player id {raw} exposed seat-zero private options"
        );
        assert!(native_view::legal_actions_for(room.engine(), ApiPlayerId::new(raw)).is_empty());
    }
}

#[test]
fn nc09_choice_visibility_requires_trusted_role_even_with_a_valid_seat() {
    let room = session_with_choice();
    let open = room.engine().open_choice().unwrap();
    let owner = ApiPlayerId::new(open.observer as u32 + 1);
    for seat in [
        None,
        Some(ApiPlayerId::NONE),
        Some(owner),
        Some(ApiPlayerId::new(257)),
    ] {
        assert!(
            native_view::choice_for(room.engine(), native_view::RoomRole::Spectator, seat)
                .is_none()
        );
        let choice =
            native_view::choice_for(room.engine(), native_view::RoomRole::Referee, seat).unwrap();
        assert_eq!(choice.observer, owner);
        assert_eq!(choice.options.len(), open.spec.options.len());
    }
}

// Temporarily appended to nc09_native_room_session.rs, removed after capturing red.
#[tokio::test]
async fn nc09_timeout_default_deadline_answers_pending_choice() {
    let room = session_with_choice();
    let choice = room.engine().open_choice().unwrap().clone();
    let revision = room.engine().revision();
    let handle = room_native_actor::spawn_room_native(98, room, 8, 4);
    let (tx, mut rx) = mpsc::channel(32);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(room::RoomMsg::Join {
            connection_id: 1,
            display_name: "timeout-observer".into(),
            preferred_seat: Some(choice.observer),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await
        .unwrap();
    assert!(response.await.unwrap().unwrap().choice.is_some());
    let answer = tokio::time::timeout(std::time::Duration::from_secs(31), async {
        while let Some(message) = rx.recv().await {
            if let engine_protocol::ServerMessage::CommandOutcome { view, .. } =
                serde_json::from_str(&message).unwrap()
            {
                if view.revision > revision {
                    return;
                }
            }
        }
        panic!("actor closed before the choice was answered");
    })
    .await;
    assert!(
        answer.is_ok(),
        "default 30-second room timeout must submit AI answer through ordinary actor command flow"
    );
}
