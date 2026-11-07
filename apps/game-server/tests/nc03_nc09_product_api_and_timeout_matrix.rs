//! NC03 & NC09 产品 API 错误码矩阵、7 模式全信息裁判投影与 30s 超时 31s AI 自动应答 WAL 重放测试。

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

use engine_api::{
    CardId as ApiCardId, CardZoneTag, ChoiceId as ApiChoiceId, Command, CommandKind, CommandResult,
    ExpectedRevision, FaultKind, IdempotencyKey, PlayerId as ApiPlayerId, RejectReason,
};
use engine_core::rule_vm::bytecode::{caps, op};
use engine_core::rule_vm::BytecodeModule;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
use engine_native::{ChoiceOption, Engine, GameAction, Phase};
use engine_protocol::{decode_s2c, encode_s2c, MessageHeader, ServerMessage};
use native_room_store::NativeRoomStore;
use native_view::{choice_for, observer_snapshot_for, RoomRole};
use room::RoomMsg;
use room_native::NativeRoomSession;
use std::collections::BTreeSet;

fn runaway_vm_pack() -> ExtensionPack {
    let mut code = Vec::new();
    // 非法宿主效果 tag (0xFFFF) + HALT：通过模块静态校验，运行期触发 VmFault
    code.push(op::HOST_EFFECT_EXT);
    code.extend_from_slice(&0xFFFFu16.to_le_bytes());
    code.push(op::HALT);
    let module = BytecodeModule {
        name: "nc03_runaway".to_string(),
        code,
        local_count: 1,
        max_stack: 8,
        entry_pc: 0,
        logic_hash: [0x33; 32],
        caps_mask: caps::EMIT_EFFECTS | caps::REQUEST_CHOICE,
    };
    let decl = VmSkillDecl {
        r05_identity: None,
        declaration_hash: None,
        presentation_binding: None,
        name: "nc03_vm_fault_skill".to_string(),
        module,
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::PhaseEnd,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern("nc03-vm-fault-pack".to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc03".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    }
}

/// 1. 完整协议错误码覆盖矩阵：
/// - `StaleRevision`（`RevisionMismatch`）
/// - `InvalidActor`（`WrongPlayer` / `NotYourTurn`）
/// - `InvalidChoice`（`NoSuchChoice` / `InvalidChoice` / `ChoicePending`）
/// - `IllegalAction`（非法牌 ID / 不合法出牌）
/// - 幂等 `command_receipt`（相同 `idempotency_key` 重试原样返回已确认回执，不二次执行）
/// - `Fault { kind: ContentFault }`（`VmFault` 整段回滚，状态哈希与修订号不变）
#[tokio::test]
async fn nc03_nc09_protocol_error_code_matrix_idempotent_receipt_and_vm_fault_rollback() {
    let mut engine = Engine::new(20261108, 4);
    // 令 0 号手牌多于体力上限，使得进入弃牌阶段必打开 SelectCards 选择
    engine.debug_set_hp(0, 1);

    let mut session = NativeRoomSession::new(engine, 4);
    let p0 = session.join(10, Some(0), None).unwrap();
    let p1 = session.join(11, Some(1), None).unwrap();
    let _p2 = session.join(12, Some(2), None).unwrap();
    let _p3 = session.join(13, Some(3), None).unwrap();

    // (a) StaleRevision (RevisionMismatch)
    let stale_res = session
        .submit(10, &CommandKind::EndPhase, session.engine().revision() + 99)
        .unwrap();
    assert_eq!(
        stale_res,
        CommandResult::Rejected {
            reason: RejectReason::StaleRevision {
                current: 0,
                expected: 99,
            },
        }
    );

    // (b) InvalidActor (WrongPlayer / NotYourTurn)
    let wrong_player_res = session
        .submit(11, &CommandKind::EndPhase, session.engine().revision())
        .unwrap();
    assert_eq!(
        wrong_player_res,
        CommandResult::Rejected {
            reason: RejectReason::InvalidActor,
        }
    );

    // (c) IllegalAction（在非出牌阶段或使用不存在的手牌出牌）
    let illegal_play = session
        .submit(
            10,
            &CommandKind::PlayCard {
                card: ApiCardId::new(9999),
                targets: vec![],
                card_targets: vec![],
            },
            session.engine().revision(),
        )
        .unwrap();
    assert!(
        matches!(
            illegal_play,
            CommandResult::Rejected {
                reason: RejectReason::IllegalAction { .. }
            }
        ),
        "非法出牌应返回 IllegalAction: {illegal_play:?}"
    );

    // 推进到弃牌阶段以打开 SelectCards 询问
    while session.engine().open_choice().is_none() {
        let rev = session.engine().revision();
        let out = session.submit(10, &CommandKind::EndPhase, rev).unwrap();
        assert!(matches!(out, CommandResult::Accepted { .. }));
    }
    let open = session.engine().open_choice().unwrap().clone();
    assert_eq!(open.observer, 0);

    // (d) ChoicePending -> InvalidChoice（有挂起选择时提交 EndPhase）
    let pending_reject = session
        .submit(10, &CommandKind::EndPhase, session.engine().revision())
        .unwrap();
    assert_eq!(
        pending_reject,
        CommandResult::Rejected {
            reason: RejectReason::InvalidChoice,
        }
    );

    // (e) WrongPlayer on ResolveChoice -> InvalidActor
    let wrong_observer_choice = session
        .submit(
            11,
            &CommandKind::ResolveChoice {
                choice_id: ApiChoiceId::new(open.id as u32),
                selections: vec![0],
                cancelled: false,
            },
            session.engine().revision(),
        )
        .unwrap();
    assert_eq!(
        wrong_observer_choice,
        CommandResult::Rejected {
            reason: RejectReason::InvalidActor,
        }
    );

    // (f) InvalidChoice / NoSuchChoice（提交不满足 min 数量的空选择，以及错误 choice_id）
    let invalid_choice_res = session
        .submit_native(
            0,
            GameAction::Answer {
                choice_id: open.id,
                selection: vec![],
            },
            session.engine().revision(),
        )
        .unwrap();
    assert_eq!(
        invalid_choice_res,
        CommandResult::Rejected {
            reason: RejectReason::InvalidChoice,
        }
    );
    let no_such_choice_res = session
        .submit_native(
            0,
            GameAction::Answer {
                choice_id: open.id + 777,
                selection: vec![],
            },
            session.engine().revision(),
        )
        .unwrap();
    assert_eq!(
        no_such_choice_res,
        CommandResult::Rejected {
            reason: RejectReason::InvalidChoice,
        }
    );

    // 合法完成弃牌选择
    let valid_sel: Vec<ChoiceOption> = open
        .spec
        .options
        .iter()
        .take(open.spec.min.max(1))
        .copied()
        .collect();
    let resolve_ok = session
        .submit_native(
            0,
            GameAction::Answer {
                choice_id: open.id,
                selection: valid_sel,
            },
            session.engine().revision(),
        )
        .unwrap();
    assert!(matches!(resolve_ok, CommandResult::Accepted { .. }));
    assert_eq!(session.engine().turn().phase, Phase::Discard);

    // (g) VmFault 整段回滚：装载带故障 VM 技能的包并在 PhaseEnd 触发
    let pack = runaway_vm_pack();
    let mut fault_engine = Engine::new_with_extensions(20261108, 4, &[&pack]).unwrap();
    fault_engine.debug_grant(0, "nc03_vm_fault_skill");
    let mut fault_session = NativeRoomSession::new_with_packs(fault_engine, vec![pack], 4);
    fault_session.join(10, Some(0), None).unwrap();
    let hash_before_fault = fault_session.engine().state_hash();
    let rev_before_fault = fault_session.engine().revision();
    let vm_fault_res = fault_session
        .submit(10, &CommandKind::EndPhase, rev_before_fault)
        .unwrap();
    match vm_fault_res {
        CommandResult::Rejected {
            reason: RejectReason::Fault { kind, message },
        } => {
            assert_eq!(kind, FaultKind::ContentFault);
            assert!(
                message.contains("nc03_vm_fault_skill"),
                "故障消息应包含技能名: {message}"
            );
        }
        other => panic!("VM 故障应映射为 Fault(ContentFault): {other:?}"),
    }
    assert_eq!(
        fault_session.engine().state_hash(),
        hash_before_fault,
        "VmFault 必须整段回滚状态哈希"
    );
    assert_eq!(
        fault_session.engine().revision(),
        rev_before_fault,
        "VmFault 不得推进修订号"
    );

    // (h) 幂等 command_receipt：通过持久化原生房间 Actor 提交同一 idempotency_key 两次，原样返回同一 Accepted 结果
    let _ = (p0, p1);
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let clean_session = NativeRoomSession::new(Engine::new(4242, 2), 2);
    let handle = room_native_actor::spawn_room_native_persisted(
        309,
        clean_session,
        16,
        10,
        String::new(),
        Some(store),
    );
    let (tx, _rx) = tokio::sync::mpsc::channel(64);
    let (reply, response) = tokio::sync::oneshot::channel();
    handle
        .send(RoomMsg::Join {
            connection_id: 1,
            display_name: "p1".into(),
            preferred_seat: Some(0),
            reconnect_token: None,
            password: String::new(),
            outbound: session::Outbound::new(tx),
            reply,
        })
        .await
        .unwrap();
    let joined = response.await.unwrap().unwrap();
    assert_eq!(joined.view.revision, 0);

    let submit_once = |replay: bool| {
        let handle = handle.clone();
        async move {
            let (reply, response) = tokio::sync::oneshot::channel();
            handle
                .send(RoomMsg::Submit {
                    connection_id: 1,
                    header: MessageHeader::new(309, 77, 0),
                    command: Command {
                        actor: ApiPlayerId::new(1),
                        expected_revision: ExpectedRevision(0),
                        idempotency_key: IdempotencyKey(777),
                        kind: CommandKind::EndPhase,
                    },
                    idempotency_key: IdempotencyKey(777),
                    replay,
                    reply,
                })
                .await
                .unwrap();
            response.await.unwrap().unwrap()
        }
    };
    let first_msg = submit_once(false).await;
    let second_msg = submit_once(true).await;
    match (&first_msg, &second_msg) {
        (
            ServerMessage::CommandOutcome {
                result: CommandResult::Accepted { new_revision: r1, state_hash: h1 },
                ..
            },
            ServerMessage::CommandOutcome {
                result: CommandResult::Accepted { new_revision: r2, state_hash: h2 },
                ..
            },
        ) => {
            assert_eq!(*r1, 1);
            assert_eq!((r1, h1), (r2, h2), "幂等重放必须返回同一已确认回执且不二次推进");
        }
        other => panic!("幂等提交应返回相同 Accepted 回执: {other:?}"),
    }
}

/// 2. 7 模式（`identity`, `kingdoms`, `versus`, `boss`, `doudizhu`, `brawl`, `single`）
/// `Referee` vs `Player` vs `Spectator` 全信息投影与选择契约：
/// - `Referee` 可见全牌面（手牌、牌堆顺序、身份/阵营、开放选择）；
/// - `Player` 仅可见本人手牌与本人开放选择，不泄露对手手牌或牌堆未知牌；
/// - `Spectator` 不含任何私有手牌与私有选择。
#[test]
fn nc03_nc09_seven_modes_referee_player_spectator_projection_matrix() {
    let modes: Vec<(&str, Engine)> = vec![
        ("identity", Engine::new_identity(20261109, 4)),
        ("kingdoms", Engine::new_kingdoms(20261109, 4)),
        ("versus", Engine::new_versus(20261109, [vec![0, 2], vec![1, 3]])),
        ("boss", Engine::new_boss(20261109, 4, 0, 2)),
        ("doudizhu", Engine::new_doudizhu(20261109, 3, 0)),
        ("brawl", Engine::new_brawl(20261109, 4)),
        ("single", Engine::new_single(20261109)),
    ];

    for (mode_name, engine) in modes {
        let seats = engine.state_core().players.len();
        let ref_view = observer_snapshot_for(&engine, RoomRole::Referee, None);
        assert_eq!(
            ref_view.known_cards.len(),
            engine.catalog().cards.len(),
            "{mode_name}: Referee 必须可见全部物理卡牌"
        );
        let unique_ids: BTreeSet<u32> =
            ref_view.known_cards.iter().map(|c| c.id.raw()).collect();
        assert_eq!(
            unique_ids.len(),
            engine.catalog().cards.len(),
            "{mode_name}: Referee 投影卡牌 ID 无重复无遗漏"
        );
        if mode_name == "identity" {
            assert!(
                ref_view.seats.iter().all(|s| s.identity.is_some()),
                "{mode_name}: Referee 可见全部座位真实身份"
            );
        }
        let draw_pile_ids: Vec<u32> = ref_view
            .known_cards
            .iter()
            .filter(|c| c.zone == CardZoneTag::DrawPile)
            .map(|c| c.id.raw())
            .collect();
        let actual_deck: Vec<u32> = engine.state_core().deck.iter().copied().collect();
        assert_eq!(
            draw_pile_ids, actual_deck,
            "{mode_name}: Referee 牌堆顺序与权威状态完全一致"
        );

        // Player 1 视角：只包含本人手牌，不包含任何其他座位手牌或牌堆牌
        let p1 = ApiPlayerId::new(1);
        let p1_view = observer_snapshot_for(&engine, RoomRole::Player, Some(p1));
        assert!(
            p1_view
                .known_cards
                .iter()
                .all(|c| c.zone == CardZoneTag::Hand && c.owner == Some(p1)),
            "{mode_name}: Player 视角只能看到本人手牌"
        );
        assert_eq!(
            p1_view.known_cards.len(),
            engine.state_core().player(0).hand.len(),
            "{mode_name}: Player 1 可见手牌数须等于座位 0 真实手牌数"
        );

        // Spectator 视角：无任何私有牌面与私有选择
        let spec_view = observer_snapshot_for(&engine, RoomRole::Spectator, None);
        assert!(
            spec_view.known_cards.is_empty(),
            "{mode_name}: Spectator 初始不得看到任何手牌或牌堆牌"
        );
        assert!(
            choice_for(&engine, RoomRole::Spectator, None).is_none(),
            "{mode_name}: Spectator 不得暴露私有选择"
        );

        // 线协议编解码往返验证
        let msg = ServerMessage::Snapshot {
            header: MessageHeader::new(1, 1, engine.revision()),
            view: ref_view.clone(),
            legal_actions: vec![],
            choice: choice_for(&engine, RoomRole::Referee, None),
        };
        let wire = encode_s2c(&msg).unwrap();
        let decoded = decode_s2c(&wire).unwrap();
        match decoded {
            ServerMessage::Snapshot { header, view, .. } => {
                assert_eq!(header.protocol_version, 8);
                assert_eq!(view.seats.len(), seats);
                assert_eq!(view.known_cards.len(), ref_view.known_cards.len());
            }
            other => panic!("{mode_name}: 快照解码结果异常: {other:?}"),
        }
    }
}

/// 3. 30s 真实选择超时配置 → 31s（窗口打开后 31_000ms，即 `deadline_unix_ms + 1_000`）触发 AI 自动应答（`TimeoutAction`）
/// → 座位转入托管（`managed_seats`）→ `NativeRoomStore` 落盘与 WAL 完整重放恢复一致。
#[test]
fn nc03_nc09_30s_choice_timeout_31s_auto_reply_and_wal_store_replay() {
    let mut session = NativeRoomSession::new(Engine::new(4242, 4), 4);
    session.set_choice_timeout_secs(30).unwrap();
    assert_eq!(session.timeout_state().choice_timeout_secs, 30);

    for seat in 0..4 {
        session
            .join(10 + u64::from(seat), Some(seat), None)
            .unwrap();
    }
    while session.engine().open_choice().is_none() {
        let seat = session.engine().turn().current;
        let action = session
            .engine()
            .legal_actions(seat)
            .into_iter()
            .next()
            .unwrap();
        let res = session
            .submit_native(seat, action, session.engine().revision())
            .unwrap();
        assert!(matches!(res, CommandResult::Accepted { .. }));
    }

    let deadline = session.timeout_state().deadline.clone().unwrap();
    let seat = deadline.observer;
    let opened_unix_ms = deadline.deadline_unix_ms - 30_000;

    // 29.999s 时（deadline_unix_ms - 1）未超时
    assert!(session
        .automatic_command(opened_unix_ms + 29_999, true)
        .is_none());

    // 31s 时（opened_unix_ms + 31_000 == deadline_unix_ms + 1_000）触发 TimeoutAction 并标记 takeover = true
    let (auto_seat, auto_action, takeover) = session
        .automatic_command(opened_unix_ms + 31_000, true)
        .expect("31s 时必须产生自动超时动作");
    assert_eq!(auto_seat, seat);
    assert!(takeover, "30s 超时首次触发须接管座位 (takeover = true)");
    assert!(matches!(auto_action, GameAction::TimeoutAction { .. }));

    let outcome = session
        .submit_automatic(auto_seat, auto_action, takeover)
        .unwrap();
    assert!(matches!(outcome, CommandResult::Accepted { .. }));
    assert!(
        session.timeout_state().managed_seats.contains(&seat),
        "超时自动应答后座位须进入 managed_seats"
    );

    // 落盘到 NativeRoomStore 并从磁盘恢复，验证 WAL 包含 TimeoutAction 且状态与快照完全一致
    let dir = tempfile::tempdir().unwrap();
    let store = NativeRoomStore::new(dir.path());
    let exported = session.export_state(&session.snapshot_hex());
    let record = native_room_store::StoredRoom {
        room_id: 3090,
        password_hash: native_room_store::salted_password_hash(3090, ""),
        checksum: native_room_store::state_checksum(&exported),
        state: exported,
    };
    store.save(&record).unwrap();

    let loaded_records = store.load_all().unwrap();
    assert_eq!(loaded_records.len(), 1);
    let restored = NativeRoomSession::from_saved(&loaded_records[0].state, vec![]).unwrap();
    assert_eq!(
        restored.engine().state_hash(),
        session.engine().state_hash(),
        "NativeRoomStore 恢复后状态哈希一致"
    );
    assert_eq!(
        restored.timeout_state().managed_seats,
        session.timeout_state().managed_seats,
        "NativeRoomStore 恢复后托管座位一致"
    );
    let wal_json = serde_json::to_value(&loaded_records[0].state.wal).unwrap();
    assert!(
        wal_json["entries"]
            .as_array()
            .unwrap()
            .iter()
            .any(|e| e["action"].get("TimeoutAction").is_some()),
        "持久化 WAL 必须记录 TimeoutAction 条目"
    );
}
