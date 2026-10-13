//! 命令信任边界 + 挂起机器快照/恢复验收（进度文档 P0 两项）。
//!
//! 边界：错误玩家、过期修订、重复命令、越界 ID 全部拒绝且不 panic；
//! 快照：任意静止点保存 → 恢复 → 每个选择点继续到相同结果；规则版本绑定。

use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::choice::ChoiceSpec;
use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot,
    Phase, RejectReason, SnapshotError,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn fresh_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn submit_cmd(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let expected_revision = e.revision();
    e.submit(GameCommand {
        actor,
        expected_revision,
        dedup_id: fresh_dedup(),
        action,
    })
}

fn next_phase(e: &mut Engine) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::NextPhase)
}

fn answer_sel(e: &mut Engine, selection: Vec<ChoiceOption>) -> CommandOutcome {
    let choice = e.open_choice().expect("choice open").clone();
    submit_cmd(
        e,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection,
        },
    )
}

fn answer(e: &mut Engine, opt: ChoiceOption) -> CommandOutcome {
    answer_sel(e, vec![opt])
}

fn play(e: &mut Engine, card: u32, targets: Vec<u8>) -> CommandOutcome {
    let actor = e.turn().current;
    submit_cmd(e, actor, GameAction::PlayCard { card, targets })
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play {
            return;
        }
        match next_phase(e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(choice.spec.min.max(1))
                    .collect();
                assert!(matches!(answer_sel(e, sel), CommandOutcome::Accepted { .. }));
            }
            other => panic!("advance rejected: {other:?}"),
        }
    }
    panic!("advance_to_play did not converge");
}

/// 布阵到「0 号在出牌阶段持一张杀、1 号无手牌」。
fn setup_play_with_sha() -> (Engine, u32) {
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    (e, sha)
}

/// 布阵到「散财确认选择挂起」（嵌套选择链共 3 个选择点）。
fn setup_sancai_pending() -> Engine {
    let mut e = Engine::new(23, 4);
    e.debug_grant(0, "散财");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    assert_eq!(e.open_choice().expect("pending").spec.prompt, "trigger_confirm");
    e
}

/// 按提示生成确定性应答。
fn answer_for(spec: &ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "trigger_confirm" => vec![ChoiceOption::Confirm],
        "sancai_target" => vec![ChoiceOption::Player(2)],
        "sancai_card" => vec![*spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
            .expect("card option")],
        other => panic!("unexpected prompt {other}"),
    }
}

// ---------------------------------------------------------------------------
// 命令信任边界
// ---------------------------------------------------------------------------

/// 应答他人的选择 → WrongPlayer；状态不变。
#[test]
fn answer_by_wrong_player_rejected() {
    let mut e = setup_sancai_pending();
    let id = e.open_choice().unwrap().id;
    assert_eq!(e.open_choice().unwrap().observer, 0);
    let before = e.state_hash();
    let out = e.submit(GameCommand {
        actor: 1, // 非观察者
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::WrongPlayer)), "{out:?}");
    assert_eq!(e.state_hash(), before);
    // 观察者本人应答正常
    assert!(matches!(
        e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: id,
                selection: vec![ChoiceOption::Confirm],
            },
        }),
        CommandOutcome::Pending { .. } // 链到下一个选择（sancai_target）
    ));
}

/// 非当前回合玩家的 PlayCard/NextPhase → NotYourTurn。
#[test]
fn commands_by_non_current_player_rejected() {
    let (mut e, sha) = setup_play_with_sha();
    assert_eq!(e.turn().current, 0);
    let before = e.state_hash();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NotYourTurn)), "{out:?}");
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NotYourTurn)), "{out:?}");
    assert_eq!(e.state_hash(), before);
}

/// actor 越界 → UnknownActor，不 panic。
#[test]
fn out_of_bounds_actor_rejected() {
    let (mut e, _) = setup_play_with_sha();
    let out = submit_cmd(&mut e, 250, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::UnknownActor)), "{out:?}");
}

/// 过期修订号 → StaleRevision；拒绝不消耗去重键（同键以正确修订号重试成功）。
#[test]
fn stale_revision_rejected_then_retry_same_dedup() {
    let (mut e, _) = setup_play_with_sha();
    let rev = e.revision();
    let fixed_dedup = 90_000;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev + 1,
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::StaleRevision)), "{out:?}");
    assert_eq!(e.revision(), rev, "拒绝不改状态");
    // 同键、正确修订号 → 接受（证明拒绝路径未消耗去重键）
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.revision(), rev + 1);
}

/// 已接受命令的去重键不可复用；键 0 恒拒绝。
#[test]
fn duplicate_and_zero_dedup_rejected() {
    let (mut e, _) = setup_play_with_sha();
    let rev = e.revision();
    let fixed_dedup = 91_000;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: rev,
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 完全相同的键（即使内容以新状态合法）→ DuplicateCommand
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fixed_dedup,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)), "{out:?}");
    // 更小的键同样拒绝
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fixed_dedup - 1,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)), "{out:?}");
    // 键 0 恒拒绝
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 0,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)), "{out:?}");
}

/// 越界目标/牌 ID 显式拒绝（此前路径可 panic）。
#[test]
fn out_of_bounds_ids_rejected_not_panic() {
    let (mut e, sha) = setup_play_with_sha();
    let before = e.state_hash();
    // 杀 → 越界目标
    let out = play(&mut e, sha, vec![200]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "{out:?}");
    assert_eq!(e.state_hash(), before);
    // 铁索 → 越界目标（debug_place 本身改状态，以其之后为新基准）
    let chain = e.debug_place(0, "chain");
    let before_chain = e.state_hash();
    let out = play(&mut e, chain, vec![200]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)), "{out:?}");
    // 越界牌 → CardNotInHand
    let out = play(&mut e, u32::MAX, vec![1]);
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::CardNotInHand)), "{out:?}");
    assert_eq!(e.state_hash(), before_chain);
}

/// 无选择挂起时 Answer → NoSuchChoice。
#[test]
fn answer_without_open_choice_rejected() {
    let (mut e, _) = setup_play_with_sha();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: 1,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::NoSuchChoice)), "{out:?}");
}

/// 应答校验的重复守卫：同一选项出现两次 → 拒绝（防下游二次移除同一实例）。
#[test]
fn duplicate_selection_rejected_by_spec() {
    let spec = ChoiceSpec {
        kind: engine_native::ChoiceKind::SelectCards,
        prompt: "unit",
        options: vec![ChoiceOption::Card(1), ChoiceOption::Card(2)],
        min: 2,
        max: 2,
        allow_cancel: false,
    };
    assert!(spec.accepts(&[ChoiceOption::Card(1), ChoiceOption::Card(2)]));
    assert!(!spec.accepts(&[ChoiceOption::Card(1), ChoiceOption::Card(1)]));
}

// ---------------------------------------------------------------------------
// 挂起机器快照与恢复
// ---------------------------------------------------------------------------

/// 静止点保存 → 恢复 → 哈希一致；恢复后继续推进与未恢复线完全同哈希。
#[test]
fn save_restore_at_rest_continues_identically() {
    let mut e = Engine::new(11, 4);
    advance_to_play(&mut e, 0);
    let snap = e.save();
    let mut r = Engine::restore(&snap).expect("restore");
    assert_eq!(r.state_hash(), e.state_hash(), "恢复后哈希必须一致");
    let rev_before = e.revision();
    assert_eq!(r.revision(), rev_before);
    // 两线各推进两个阶段（含弃牌挂起），每步后哈希保持一致
    for _ in 0..2 {
        drive_phase(&mut e);
        drive_phase(&mut r);
        assert_eq!(r.state_hash(), e.state_hash(), "恢复后继续推进仍同哈希");
    }
}

/// 推进一个阶段；若挂起则按最小牌应答（弃牌等）直至本段完结。
fn drive_phase(e: &mut Engine) {
    match next_phase(e) {
        CommandOutcome::Accepted { .. } => {}
        CommandOutcome::Pending { choice } => {
            let sel: Vec<ChoiceOption> = choice
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(choice.spec.min.max(1))
                .collect();
            let out = answer_sel(e, sel);
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
        }
        other => panic!("drive_phase rejected: {other:?}"),
    }
}

/// 挂起点保存 → 恢复 → 每个选择点继续到相同结果（散财 3 选择点链）。
#[test]
fn restore_at_every_choice_point_continues_identically() {
    // 基准线：无中断跑完整链，收集每个选择点（id, 提示）、应答序列与终态
    let mut base = setup_sancai_pending();
    let mut points: Vec<(u64, &'static str)> = Vec::new();
    let mut answers: Vec<Vec<ChoiceOption>> = Vec::new();
    for _ in 0..8 {
        let Some(choice) = base.open_choice().cloned() else {
            break;
        };
        points.push((choice.id, choice.spec.prompt));
        let sel = answer_for(&choice.spec);
        let out = answer_sel(&mut base, sel.clone());
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        answers.push(sel);
    }
    assert_eq!(answers.len(), 3, "散财链应有 3 个选择点");
    let base_hash = base.state_hash();

    // 快照线：从第一个选择点起，每个点都「保存 → 恢复 → 应答」
    let snap0 = setup_sancai_pending().save();
    let mut cur = Engine::restore(&snap0).expect("restore");
    for (i, sel) in answers.iter().enumerate() {
        // 每个选择点先做一次保存/恢复往返
        let mid = cur.save();
        let mut resumed = Engine::restore(&mid).expect("restore mid");
        assert_eq!(resumed.state_hash(), cur.state_hash(), "选择点 {} 恢复哈希一致", i);
        // 恢复件在该点的开放选择与基准线一致
        let got = resumed
            .open_choice()
            .map(|c| (c.id, c.spec.prompt))
            .expect("选择点应挂起");
        assert_eq!(got, points[i], "选择点 {} 的开放选择应一致", i);
        let out = answer_sel(&mut resumed, sel.clone());
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        cur = resumed;
    }
    assert_eq!(cur.state_hash(), base_hash, "恢复线与基准线终态必须相同");
}

/// 规则版本绑定：版本不符的快照拒绝恢复。
#[test]
fn restore_rejects_rule_version_mismatch() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.ruleset_version += 1;
    match Engine::restore(&snap) {
        Err(SnapshotError::RuleVersionMismatch { snapshot, catalog }) => {
            assert_eq!(snapshot, snap.ruleset_version);
            assert_eq!(catalog, e.ruleset_version());
        }
        other => panic!("expected rule version mismatch, got {:?}", other.map(|_| ())),
    }
    // 状态内版本与快照一致时可恢复
    assert_eq!(e.save().ruleset_version, e.ruleset_version());
}

/// 递归寻找快照中的散财机器并把目标改为越界玩家（闸门/窗口包裹的机器在深层）。
fn patch_sancai_target(m: &mut MachineSnapshot) -> bool {
    match m {
        MachineSnapshot::SanCai { target, .. } => {
            *target = Some(200);
            true
        }
        MachineSnapshot::OptionalGate { inner, .. } => inner
            .as_mut()
            .map(|m| patch_sancai_target(m))
            .unwrap_or(false),
        MachineSnapshot::NullifyWindow { effect, .. } => effect
            .as_mut()
            .map(|m| patch_sancai_target(m))
            .unwrap_or(false),
        _ => false,
    }
}

/// 越界 ID 的快照拒绝恢复（核心牌区 / 机器局部 / 开放选择）。
#[test]
fn restore_rejects_out_of_bounds_ids() {
    let e = setup_sancai_pending();
    // (1) 核心状态：手牌里塞越界牌
    let mut bad_core = e.save();
    bad_core.core.players[0].hand.push(u32::MAX);
    assert!(matches!(
        Engine::restore(&bad_core),
        Err(SnapshotError::IdOutOfBounds { what: "card", .. })
    ));
    // (2) 机器局部：把散财目标改成越界玩家。
    //     恢复 confirm 点后应答 confirm；此时散财机器在 OptionalGate 内层。
    let snap = e.save();
    let mut r = Engine::restore(&snap).unwrap();
    answer(&mut r, ChoiceOption::Confirm);
    let mut snap2 = r.save();
    let mut patched = false;
    for park in &mut snap2.resolution.parks {
        patched |= patch_sancai_target(&mut park.machine);
    }
    assert!(patched, "挂起的散财机器应在快照中");
    assert!(matches!(
        Engine::restore(&snap2),
        Err(SnapshotError::IdOutOfBounds { what: "player", .. })
    ));
    // (3) 开放选择：选项里塞越界玩家
    let mut snap3 = e.save();
    if let Some(open) = &mut snap3.resolution.open_choice {
        open.spec.options.push(ChoiceOption::Player(250));
    }
    assert!(matches!(
        Engine::restore(&snap3),
        Err(SnapshotError::IdOutOfBounds { what: "player", .. })
    ));
}

/// 并行评审复现洞的回归：篡改核心状态内规则版本（顶层一致）必须被拒绝。
#[test]
fn restore_rejects_tampered_core_ruleset_version() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    assert_eq!(snap.ruleset_version, snap.core.ruleset_version);
    snap.core.ruleset_version += 1; // 只改核心副本，顶层保持一致
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ));
}

/// 并行评审复现洞的回归：技能实例值内 owner 与键不一致必须被拒绝。
#[test]
fn restore_rejects_inconsistent_skill_instance_owner() {
    use engine_native::state::SkillInstance;
    let e = setup_sancai_pending();
    let mut snap = e.save();
    // 键 (0, 0) 合法；实例值 owner=255 与键不一致（越权篡改形态）
    snap.core.skills.insert(
        (0, 0),
        SkillInstance {
            owner: 255,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "skill_instance_owner", got: 255 })
    ));
}

/// 挂起点恢复件的摘要与未恢复件一致（含机器快照字节）。
#[test]
fn pending_state_hash_stable_across_save_restore() {
    let e = setup_sancai_pending();
    let hash_before = e.state_hash();
    let snap = e.save();
    let r = Engine::restore(&snap).unwrap();
    assert_eq!(r.state_hash(), hash_before);
}

/// 恢复后的引擎延续去重高水位：恢复前已接受的键在恢复件中不可复用。
#[test]
fn restore_carries_dedup_high_water() {
    let (mut e, _) = setup_play_with_sha();
    // 取当前全局计数作为基键：必大于本引擎此前接受的任何键
    let base = fresh_dedup();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: base,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    let snap = e.save();
    let mut r = Engine::restore(&snap).unwrap();
    let out = r.submit(GameCommand {
        actor: r.turn().current,
        expected_revision: r.revision(),
        dedup_id: base,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::DuplicateCommand)), "{out:?}");
    // 新键正常
    let out = r.submit(GameCommand {
        actor: r.turn().current,
        expected_revision: r.revision(),
        dedup_id: base + 1,
        action: GameAction::NextPhase,
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
}

/// N04-e：篡改 Sha 机器 `asked`（true→false）使开放选择失去承接机器
/// → 恢复必须拒绝（OpenChoiceMismatch）；机器字段的摘要敏感性由
/// src 单元测试 `park_machine_state_changes_digest` 覆盖。
#[test]
fn restore_rejects_sha_asked_tamper() {
    let (mut e, sha) = setup_play_with_sha();
    let _shan = e.debug_place(1, "shan");
    play(&mut e, sha, vec![1]); // 杀响应窗口挂起（目标持闪）
    let mut snap = e.save();
    let mut patched = false;
    for park in &mut snap.resolution.parks {
        if let MachineSnapshot::Sha { asked, .. } = &mut park.machine {
            *asked = false; // 已询问却被改回未询问 → 选择失去承接
            patched = true;
        }
    }
    assert!(patched, "挂起的杀结算机器应在快照中");
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::OpenChoiceMismatch { observer: 1 })
    ));
}

// ---------------------------------------------------------------------------
// N04 结构校验（快照内栈帧/挂起/身份/胜负关系）+ N05 逐机器暂停点往返矩阵
// ---------------------------------------------------------------------------

use engine_native::event::{EventData, QueuedTrigger};
use engine_native::state::{GameMode, Outcome, Role, Winner};
use engine_native::resolution::Slot;

#[test]
fn restore_rejects_broken_frame_parent_chain() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.stack[0].parent = Some(999);
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "frame_parent", .. })
    ));
}

#[test]
fn restore_rejects_park_depth_overrun() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.parks[0].resume_depth = 999;
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "resume_depth", .. })
    ));
}

#[test]
fn restore_rejects_dangling_park_frame() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.parks[0].slot = Slot::Trigger { frame: 999 };
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "park_frame", .. })
    ));
}

#[test]
fn restore_rejects_duplicate_frame_id() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    let original_id = snap.resolution.stack[0].id;
    let mut dup = snap.resolution.stack[0].clone();
    dup.parent = Some(original_id); // 父引用合法，命中的是帧号去重
    snap.resolution.stack.push(dup);
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "event_id_dup", .. })
    ));
}

#[test]
fn restore_rejects_oob_event_payload() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.stack[0].data = EventData::Linked { player: 200, linked: true };
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "player", .. })
    ));
}

#[test]
fn restore_rejects_oob_queue_trigger() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.stack[0].queue.push(QueuedTrigger {
        def: 9999,
        owner: 0,
        order: (1, 0, 0, 0, 0),
    });
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "skill_def", .. })
    ));
}

#[test]
fn restore_rejects_choice_id_overrun() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    if let Some(open) = &mut snap.resolution.open_choice {
        open.id = u64::MAX;
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "choice_id", .. })
    ));
}

/// N04-a：OptionalGate 计划事件载荷中的 ID 越界必须被拒绝。
#[test]
fn restore_rejects_tampered_gate_event_payload() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    let mut patched = false;
    for park in &mut snap.resolution.parks {
        if let MachineSnapshot::OptionalGate { event, .. } = &mut park.machine {
            // 散财确认闸门挂在伤害帧上：载荷为 Damage
            if let EventData::Damage(d) = event {
                d.target = 255;
                patched = true;
            }
        }
    }
    assert!(patched, "挂起的闸门机器应携带 Damage 事件载荷");
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "player", .. })
    ));
}

/// N04-b：父帧自引用必须被拒绝。
#[test]
fn restore_rejects_self_referencing_frame_parent() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    let self_id = snap.resolution.stack[0].id;
    snap.resolution.stack[0].parent = Some(self_id);
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "frame_parent", .. })
    ));
}

/// N04-b：父帧指向更晚的帧（向前引用）必须被拒绝。
#[test]
fn restore_rejects_forward_referencing_frame_parent() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    let mut second = snap.resolution.stack[0].clone();
    second.id += 1; // 唯一且低于水位
    second.parent = None;
    let second_id = second.id;
    snap.resolution.stack[0].parent = Some(second_id); // 引用栈中更晚的帧
    snap.resolution.stack.push(second);
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "frame_parent", got }) if got == second_id as u64
    ));
}

/// N04-c：开放选择但没有任何挂起机器承接 → 拒绝。
#[test]
fn restore_rejects_choice_without_machine() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.parks.clear();
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "choice_without_machine", .. })
    ));
}

/// N04-c 补充：挂起深度低于当前栈深是合法形态（AOE 段间伤害帧在机器之上
/// 先行结算）——此类快照必须可恢复。
#[test]
fn restore_accepts_park_depth_below_stack() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    let depth = snap.resolution.stack.len();
    assert!(depth >= 1);
    snap.resolution.parks[0].resume_depth = depth - 1;
    // 深度参与摘要：篡改后哈希不同但仍必须可恢复（合法形态）
    let _r = Engine::restore(&snap).expect("深度低于栈深且存在承接机器的快照应合法");
}

/// N04 补充：身份局主公不在座位 0 → 拒绝。
#[test]
fn restore_rejects_lord_not_at_first_seat() {
    let e = Engine::new_identity(183, 5);
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        roles.rotate_left(1); // 主公移到末位
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "lord_seat", .. })
    ));
}

/// N04：身份列表长度/组成校验（清空、无主公均拒绝）。
#[test]
fn restore_rejects_tampered_identity_roles() {
    let e = Engine::new_identity(183, 5);
    // (1) roles 清空
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        roles.clear();
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "roles_len", .. })
    ));
    // (2) 全忠臣（首位非主公 → lord_seat 先触发）
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        *roles = vec![Role::Loyalist; 5];
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "lord_seat", .. })
    ));
    // (3) 主公在首位但无内奸 → 组成校验
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        *roles = vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Rebel, Role::Rebel];
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "roles_composition", .. })
    ));
    // (4) 忠臣数与人数表不符（5 人局 2 忠）→ 拒绝（N04 精确人数表）
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        *roles = vec![Role::Lord, Role::Loyalist, Role::Loyalist, Role::Rebel, Role::Traitor];
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "roles_loyalist_count", .. })
    ));
    // (3) 合法身份快照往返一致
    let snap = e.save();
    let r = Engine::restore(&snap).expect("合法身份快照必须可恢复");
    assert_eq!(r.state_hash(), e.state_hash());
}

/// N04：胜负与模式一致性（原型局不得有阵营胜者、身份局不得有玩家胜者）。
#[test]
fn restore_rejects_outcome_mode_mismatch() {
    // 原型局 + 阵营胜者 → 拒绝
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.core.outcome = Some(Outcome { winner: Winner::LordSide });
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "outcome_mode_mismatch", .. })
    ));
    // 身份局 + 玩家胜者 → 拒绝
    let e = Engine::new_identity(185, 5);
    let mut snap = e.save();
    snap.core.outcome = Some(Outcome { winner: Winner::Player(0) });
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "outcome_mode_mismatch", .. })
    ));
}

// ---------------------------------------------------------------------------
// N05：逐机器暂停点往返矩阵
// ---------------------------------------------------------------------------

/// 通用矩阵：build() 停在首个挂起点；基准线直跑收集应答与终态；
/// 快照线从同一快照恢复，每个挂起点「保存→恢复→应答」，终态必须一致。
fn assert_pause_roundtrip(
    build: fn() -> Engine,
    answer_for: fn(&engine_native::ChoiceSpec) -> Vec<ChoiceOption>,
) {
    // 守恒断言（N01–N03 回归守护）：每张实体牌在各牌区或机器持牌中恰出现一次。
    // 判定阶段挂起点上，待判锦囊由机器队列持有；使用时窗口的待置入锦囊
    // 在桌面上（PlaceDelayed 不持牌）。
    fn machine_cards(m: &MachineSnapshot, out: &mut Vec<u32>) {
        match m {
            MachineSnapshot::JudgePhase { queue, current, .. } => {
                out.extend(queue.iter().copied());
                // 进行中判定：待判锦囊（current.trick）与判定牌（judge_card）
                // 均由机器持有；天妒获得（入手牌）、放弃或随效果弃置
                // （judge_held=false）后不再计机器持牌。
                if let Some(c) = current.as_ref() {
                    // finalized（效果已套用）后锦囊已离机（弃置/转移/判定区），
                    // current.trick 仅为簿记，不再计入机器持牌。
                    if !c.finalized {
                        out.push(c.trick);
                    }
                    if c.judge_held {
                        out.push(c.judge_card);
                    }
                }
            }
            MachineSnapshot::OptionalGate { inner, .. } => {
                if let Some(m) = inner.as_ref() {
                    machine_cards(m, out);
                }
            }
            MachineSnapshot::NullifyWindow { effect, .. } => {
                if let Some(m) = effect.as_ref() {
                    machine_cards(m, out);
                }
            }
            MachineSnapshot::Aoe { inner, .. } => {
                if let Some(m) = inner.as_ref() {
                    machine_cards(m, out);
                }
            }
            _ => {}
        }
    }
    fn assert_conservation(e: &Engine) {
        let mut ids: Vec<u32> = Vec::new();
        let core = e.state_core();
        for p in &core.players {
            ids.extend(p.hand.iter().copied());
            ids.extend(p.equips.iter().copied());
            ids.extend(p.judge.iter().copied());
        }
        ids.extend(core.deck.iter().copied());
        ids.extend(core.discard.iter().copied());
        ids.extend(core.table.iter().copied());
        for park in e.save().resolution.parks {
            machine_cards(&park.machine, &mut ids);
        }
        ids.sort_unstable();
        let total = ids.len();
        let dups: Vec<u32> = ids.windows(2).filter(|w| w[0] == w[1]).map(|w| w[0]).collect();
        ids.dedup();
        assert_eq!(ids.len(), total, "实体牌出现在多个牌区/机器: 重复={dups:?}");
        assert_eq!(ids.len(), e.catalog().cards.len(), "目录牌与牌区不守恒: 缺失={:?}", {
            let all: Vec<u32> = (0..e.catalog().cards.len() as u32).collect();
            let missing: Vec<u32> = all.iter().filter(|c| !ids.contains(c)).copied().collect();
            missing
        });
    }

    // 基准线：记录每个选择点前的日志水位，收集应答序列
    let mut base = build();
    assert!(base.open_choice().is_some(), "build 必须停在首个挂起点");
    let first_hash = base.state_hash();
    let mut answers: Vec<Vec<ChoiceOption>> = Vec::new();
    let mut log_marks: Vec<usize> = Vec::new();
    for _ in 0..12 {
        assert_conservation(&base);
        let Some(choice) = base.open_choice().cloned() else { break };
        let sel = answer_for(&choice.spec);
        assert!(
            !sel.is_empty() || choice.spec.allow_cancel,
            "answer_for 必须给出合法应答（空应答仅限可放弃选择）"
        );
        log_marks.push(base.log().len());
        answers.push(sel.clone());
        let out = answer_sel(&mut base, sel);
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
    }
    assert!(!answers.is_empty());
    // 终止断言：两条线最终都不再有开放选择
    assert!(base.open_choice().is_none(), "基准线必须走到无挂起终态");
    let base_hash = base.state_hash();
    let base_log: Vec<_> = base.log().to_vec();

    // 快照线：从同一首点快照恢复，逐点保存/恢复后应答；
    // 比对每点规范摘要、终态摘要，以及恢复点之后的**事件增量**（恢复件日志为空）。
    let snap0 = build().save();
    let mut cur = Engine::restore(&snap0).expect("restore");
    assert_eq!(cur.state_hash(), first_hash, "恢复件与基准首点同哈希");
    for (i, sel) in answers.iter().enumerate() {
        let Some(_c) = cur.open_choice() else {
            panic!("第 {i} 个选择点应挂起");
        };
        let mid = cur.save();
        let mut resumed = Engine::restore(&mid).expect("restore mid");
        assert_eq!(resumed.state_hash(), cur.state_hash(), "选择点 {i} 往返一致");
        assert!(resumed.log().is_empty(), "恢复件宿主日志应为空");
        let out = answer_sel(&mut resumed, sel.clone());
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        // 增量段 = 本次应答到下一次挂起（或终态）之间的日志
        let seg_end = log_marks.get(i + 1).copied().unwrap_or(base_log.len());
        assert_eq!(
            resumed.log(),
            &base_log[log_marks[i]..seg_end],
            "选择点 {i} 之后的事件增量必须与基准线一致"
        );
        assert_conservation(&resumed);
        cur = resumed;
    }
    assert_eq!(cur.state_hash(), base_hash, "快照线终态必须与基准一致");
    assert!(cur.open_choice().is_none(), "快照线必须走到无挂起终态");
}

/// 按提示给出确定性应答：确认类答 Confirm，响应/窗口类答 Pass（首选项），
/// 目标类答玩家 2，弃牌类取前 min 张。
fn answer_by_prompt(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "trigger_confirm" => vec![ChoiceOption::Confirm],
        "sancai_target" | "yishang_target" => vec![ChoiceOption::Player(2)],
        "discard_to_limit" => spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .take(spec.min)
            .collect(),
        _ => vec![spec.options[0]],
    }
}

fn build_sha_respond() -> Engine {
    let (mut e, sha) = setup_play_with_sha();
    let _shan = e.debug_place(1, "shan");
    play(&mut e, sha, vec![1]);
    e
}

fn build_dying_save() -> Engine {
    let mut e = Engine::new(41, 4);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _tao = e.debug_place(0, "tao");
    let sha = e.debug_place(1, "sha");
    play(&mut e, sha, vec![0]);
    e
}

fn build_duel_respond() -> Engine {
    let mut e = Engine::new(141, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let _sha_t = e.debug_place(2, "sha");
    play(&mut e, duel, vec![2]);
    e
}

fn build_aoe_mid_queue() -> Engine {
    let mut e = Engine::new(131, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let _sha1 = e.debug_place(1, "sha");
    let _sha2 = e.debug_place(2, "sha");
    play(&mut e, savage, vec![]);
    e
}

fn build_judge_window() -> Engine {
    let mut e = Engine::new(175, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 使用时无懈窗口：先放无懈再出乐；队列自来源 0 起（0 无无懈跳过）→ 2 被询问
    let _nul = e.debug_place(2, "nullify");
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    e
}

fn build_discard_to_limit() -> Engine {
    let mut e = Engine::new(91, 4);
    e.debug_clear_hand(0);
    for _ in 0..4 {
        e.debug_place(0, "shan");
    }
    e.debug_set_hp(0, 2);
    advance_to_play(&mut e, 0);
    for _ in 0..8 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

fn build_yishang_target() -> Engine {
    let mut e = Engine::new(31, 4);
    e.debug_grant(0, "移伤");
    let sha = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha, vec![0]);
    // 首个挂起点 = 可选触发确认
    e
}

#[test]
fn pause_roundtrip_sha_respond() {
    assert_pause_roundtrip(build_sha_respond, answer_by_prompt);
}

#[test]
fn pause_roundtrip_dying_save() {
    assert_pause_roundtrip(build_dying_save, answer_by_prompt);
}

#[test]
fn pause_roundtrip_duel_respond() {
    assert_pause_roundtrip(build_duel_respond, answer_by_prompt);
}

#[test]
fn pause_roundtrip_aoe_mid_queue() {
    assert_pause_roundtrip(build_aoe_mid_queue, answer_by_prompt);
}

#[test]
fn pause_roundtrip_judge_window() {
    assert_pause_roundtrip(build_judge_window, answer_by_prompt);
}

#[test]
fn pause_roundtrip_discard_to_limit() {
    assert_pause_roundtrip(build_discard_to_limit, answer_by_prompt);
}

#[test]
fn pause_roundtrip_yishang_chain() {
    assert_pause_roundtrip(build_yishang_target, answer_by_prompt);
}

/// 应答策略：无懈窗口出无懈（若有），其余沿用 answer_by_prompt。
fn answer_plays_nullify(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    if spec.prompt == "nullify_window" {
        if let Some(card) = spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
        {
            return vec![*card];
        }
    }
    answer_by_prompt(spec)
}

fn build_nullify_chain() -> Engine {
    let mut e = Engine::new(153, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let _nul1 = e.debug_place(1, "nullify");
    let _nul2 = e.debug_place(2, "nullify");
    let _sha1 = e.debug_place(1, "sha");
    play(&mut e, savage, vec![]);
    e
}

/// 主动无懈与反无懈逐点往返（窗口 asked/队列重建/逐目标推进全链路）。
#[test]
fn pause_roundtrip_active_nullify_chain() {
    assert_pause_roundtrip(build_nullify_chain, answer_plays_nullify);
}

/// 应答策略：主动判定负载——改判询问出第一张黑牌（无牌才 Pass），
/// 天妒确认获得；其余沿用 answer_by_prompt。
fn answer_active_judgement(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "gui_dao_rewrite" => {
            match spec.options.iter().find(|o| matches!(o, ChoiceOption::Card(_))) {
                Some(card) => vec![*card],
                None => vec![spec.options[0]],
            }
        }
        "tian_du_gain" => vec![ChoiceOption::Confirm],
        _ => answer_by_prompt(spec),
    }
}

/// 布阵到「1 号判定阶段改判/天妒询问」：乐在 1 判定区、判定牌红桃；
/// grants 列出授予 1 号（或 2 号）的技能名。
fn build_judgement_window_with(grants: &[(&str, u8)], black_to: Option<u8>) -> Engine {
    let mut e = Engine::new(263, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for (skill, who) in grants {
        e.debug_grant(*who, skill);
    }
    if let Some(who) = black_to {
        let _black = place_black_sha(&mut e, who);
    }
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    let _ = play(&mut e, le, vec![1]);
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

fn build_guidao_active_rewrite() -> Engine {
    // 2 号鬼道持黑牌：首个挂起点 = 对 2 的改判询问（主动出黑牌）
    build_judgement_window_with(&[("鬼道", 2)], Some(2))
}

fn build_tiandu_confirm() -> Engine {
    // 1 号天妒（限本人）：无改判者，首个挂起点 = 天妒询问（Confirm 获得）
    build_judgement_window_with(&[("天妒", 1)], None)
}

fn build_rewrite_then_tiandu_combo() -> Engine {
    // 1 号兼持鬼道+天妒：改判（自出黑牌）→ 天妒 Confirm 两点链
    build_judgement_window_with(&[("鬼道", 1), ("天妒", 1)], Some(1))
}

/// N05：主动鬼道改判（真实出黑牌）逐点往返；逐提交含机器持牌守恒。
#[test]
fn pause_roundtrip_guidao_active_rewrite() {
    assert_pause_roundtrip(build_guidao_active_rewrite, answer_active_judgement);
}

/// N05：天妒确认获得逐点往返（获得路径 judge_held→手牌守恒）。
#[test]
fn pause_roundtrip_tiandu_confirm() {
    assert_pause_roundtrip(build_tiandu_confirm, answer_active_judgement);
}

/// N05：改判→天妒两点链逐点往返（挂起于改判窗口与天妒窗口各一次）。
#[test]
fn pause_roundtrip_rewrite_then_tiandu_combo() {
    assert_pause_roundtrip(build_rewrite_then_tiandu_combo, answer_active_judgement);
}

/// 应答策略：濒死救援出桃（若有），其余沿用 answer_by_prompt。
fn answer_plays_tao(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    if spec.prompt == "dying_save" {
        if let Some(card) = spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
        {
            return vec![*card];
        }
    }
    answer_by_prompt(spec)
}

/// 布阵：1 血 + 闪电判定（雷击 3 伤 → -2 负体力）+ 三桃；
/// 停在首个濒死救援询问（每轮一人一桃，持桃者被排回队尾连续自救）。
fn build_negative_dying_rescue() -> Engine {
    let mut e = Engine::new(241, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let _sd = e.debug_place_judge(1, "shandian");
    for _ in 0..3 {
        e.debug_place(1, "tao");
    }
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    for _ in 0..32 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

/// N05：负体力多桃连续救援逐点往返——三个濒死询问点各挂起一次，
/// 基准线/快照线每点哈希、事件增量、终态与守恒一致。
#[test]
fn pause_roundtrip_negative_hp_multi_tao_rescue() {
    assert_pause_roundtrip(build_negative_dying_rescue, answer_plays_tao);
}

/// 应答策略：改判/天妒一律放弃（Pass）——覆盖 Decline 分支
///（改判跳过、判定牌随效果弃置、天妒放弃后 judge_held→弃置）。
fn answer_declines_judgement(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    match spec.prompt {
        "gui_dao_rewrite" => vec![ChoiceOption::Pass],
        // Confirm 类选择的放弃 = 空应答（allow_cancel）
        "tian_du_gain" => vec![],
        _ => answer_by_prompt(spec),
    }
}

/// N05 Decline 分支：改判询问放弃——无 Rewritten、判定牌随效果弃置。
#[test]
fn pause_roundtrip_guidao_pass() {
    assert_pause_roundtrip(build_guidao_active_rewrite, answer_declines_judgement);
}

/// N05 Decline 分支：天妒询问放弃——判定牌从机器持有转入弃牌堆恰一份。
#[test]
fn pause_roundtrip_tiandu_decline() {
    fn build_tiandu_decline() -> Engine {
        build_judgement_window_with(&[("天妒", 1)], None)
    }
    assert_pause_roundtrip(build_tiandu_decline, answer_declines_judgement);
}

/// N05 无懈×洗牌布阵并入通用矩阵：南蛮多目标无懈链逐点往返
///（真实 DeckReshuffled×恢复组合见 savage_multi_nullify_judge_reshuffle_restore）。
#[test]
fn pause_roundtrip_nullify_reshuffle_chain() {
    assert_pause_roundtrip(build_savage_nullify_reshuffle, answer_plays_nullify);
}

// ---------------------------------------------------------------------------
// N05 补充：多目标无懈 × 真实洗牌（DeckReshuffled）× 恢复组合
// ---------------------------------------------------------------------------

/// 多重集守恒（含机器持牌）：每张实体牌在牌区或机器中恰出现一次。
fn assert_multiset_conservation_named(e: &Engine, at: &str) {
    fn machine_cards(m: &MachineSnapshot, out: &mut Vec<u32>) {
        match m {
            MachineSnapshot::JudgePhase { queue, current, .. } => {
                out.extend(queue.iter().copied());
                if let Some(c) = current.as_ref() {
                    if !c.finalized {
                        out.push(c.trick);
                    }
                    if c.judge_held {
                        out.push(c.judge_card);
                    }
                }
            }
            MachineSnapshot::OptionalGate { inner, .. }
            | MachineSnapshot::NullifyWindow { effect: inner, .. }
            | MachineSnapshot::Aoe { inner, .. } => {
                if let Some(m) = inner.as_ref() {
                    machine_cards(m, out);
                }
            }
            _ => {}
        }
    }
    let mut ids: Vec<u32> = Vec::new();
    let core = e.state_core();
    for p in &core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(core.deck.iter().copied());
    ids.extend(core.discard.iter().copied());
    ids.extend(core.table.iter().copied());
    for park in e.save().resolution.parks {
        machine_cards(&park.machine, &mut ids);
    }
    ids.sort_unstable();
    let total = ids.len();
    let dups: Vec<u32> = ids.windows(2).filter(|w| w[0] == w[1]).map(|w| w[0]).collect();
    ids.dedup();
    assert_eq!(ids.len(), total, "{at}: 重复={dups:?}");
    assert_eq!(
        ids.len(),
        e.catalog().cards.len(),
        "{at}: 目录牌与牌区不守恒（缺失或多余）"
    );
}

/// 布阵：牌库清空 + 南蛮（1 号持无懈）+ 乐留 0 手牌；停在首个无懈窗口。
fn build_savage_nullify_reshuffle() -> Engine {
    let mut e = Engine::new(379, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 牌库清空：下一次翻判定牌触发真实洗回（DeckReshuffled）
    e.debug_drain_deck_to_discard();
    let savage = e.debug_place(0, "savage");
    let _n1 = e.debug_place(1, "nullify");
    let le = e.debug_place(0, "le");
    let _ = play(&mut e, savage, vec![]);
    let _ = le;
    e
}

/// 无懈窗口排干：1 号有牌即出（南蛮多目标窗口与反无懈窗），
/// 其余一律 Pass；每次应答后多重集守恒（含机器持牌）。
fn drive_nullify_windows(e: &mut Engine) {
    while let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "nullify_window", "此段只应有无懈窗口");
        let sel = if c.observer == 1 {
            c.spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(_)))
                .map(|o| vec![*o])
                .unwrap_or_else(|| vec![ChoiceOption::Pass])
        } else {
            vec![ChoiceOption::Pass]
        };
        let cid = c.id;
        let obs = c.observer;
        let out = submit_cmd(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        assert_multiset_conservation_named(e, "无懈应答后");
    }
}

/// 南蛮结算后对 1 用乐，推进至 1 的判定阶段：牌库空 → 真实洗回。
/// 返回终态哈希（判定完成后）。
fn continue_to_judge_reshuffle(e: &mut Engine) -> [u8; 32] {
    let le = e
        .debug_hand(0)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == CardKind::Le)
        .expect("乐应在 0 手牌");
    let out = play(e, le, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    drive_nullify_windows(e);
    for _ in 0..32 {
        if e.log().iter().any(|l| matches!(l, LE::DeckReshuffled)) {
            break;
        }
        if let Some(c) = e.open_choice() {
            let cid = c.id;
            let obs = c.observer;
            let out = submit_cmd(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![ChoiceOption::Pass],
                },
            );
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            continue;
        }
        match next_phase(e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert!(
        e.log().iter().any(|l| matches!(l, LE::DeckReshuffled)),
        "翻判定牌应触发真实 DeckReshuffled"
    );
    assert_multiset_conservation_named(e, "洗牌后");
    e.state_hash()
}

/// N05 组合回归：南蛮多目标无懈链 → 乐判定翻牌触发真实 DeckReshuffled →
/// 首个无懈窗口挂起保存/恢复后继续，终态与基准线完全一致。
#[test]
fn savage_multi_nullify_judge_reshuffle_restore() {
    // 基准线
    let mut base = build_savage_nullify_reshuffle();
    drive_nullify_windows(&mut base);
    let hash_base = continue_to_judge_reshuffle(&mut base);

    // 快照线：首个无懈窗口挂起点保存/恢复，同脚本继续
    let b = build_savage_nullify_reshuffle();
    assert!(b.open_choice().is_some(), "应停在首个无懈窗口");
    let snap = b.save();
    let mut r = Engine::restore(&snap).expect("restore");
    assert_eq!(r.state_hash(), b.state_hash(), "恢复件与挂起点同哈希");
    drive_nullify_windows(&mut r);
    let hash_r = continue_to_judge_reshuffle(&mut r);
    assert_eq!(hash_r, hash_base, "恢复线终态必须与基准一致");
    assert_eq!(
        r.log().iter().filter(|l| matches!(l, LE::DeckReshuffled)).count(),
        1,
        "恢复线同样恰好发生一次真实洗牌"
    );
}



fn build_judge_lightning_dying() -> Engine {
    let mut e = Engine::new(209, 4);
    e.debug_set_hp(1, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _sd = e.debug_place_judge(1, "shandian");
    let _tao = e.debug_place(2, "tao");
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

/// 应答策略：濒死救援用桃救回，其余沿用 answer_by_prompt。
fn answer_saves_dying(spec: &engine_native::ChoiceSpec) -> Vec<ChoiceOption> {
    if spec.prompt == "dying_save" {
        if let Some(card) = spec
            .options
            .iter()
            .find(|o| matches!(o, ChoiceOption::Card(_)))
        {
            return vec![*card];
        }
    }
    answer_by_prompt(spec)
}

/// 判定→闪电伤害→濒死救援嵌套链（JudgePhase→JudgeTrick→Damage→DyingSave）。
#[test]
fn pause_roundtrip_judge_lightning_dying_save() {
    assert_pause_roundtrip(build_judge_lightning_dying, answer_saves_dying);
}

// ---------------------------------------------------------------------------
// N08：鬼道改判窗口挂起点往返
// ---------------------------------------------------------------------------

use engine_native::catalog::Suit;

/// 从目录挑一张黑色（黑桃/梅花）杀布进 `player` 手牌。
fn place_black_sha(e: &mut Engine, player: u8) -> u32 {
    for _ in 0..6 {
        let c = e.debug_place(player, "sha");
        if matches!(e.catalog().cards[c as usize].suit, Suit::Spade | Suit::Club) {
            return c;
        }
    }
    panic!("目录中应存在黑色杀");
}

fn build_guidao_rewrite() -> Engine {
    let mut e = Engine::new(207, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼道");
    let _black = place_black_sha(&mut e, 2);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    // 推进到 1 的判定阶段：鬼道改判询问（自拥有者 1 起，1 无鬼道 → 2）
    for _ in 0..24 {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    e
}

/// 鬼道改判窗口挂起点往返（改判队列/进行态判定牌随快照延续）。
#[test]
fn pause_roundtrip_guidao_rewrite() {
    assert_pause_roundtrip(build_guidao_rewrite, answer_by_prompt);
}

/// N04-d：改判阶段 asked=true 但改判队列已空的快照必须拒绝
/// （恢复后应答会在空队列上 panic——复核探针复现项的回归）。
#[test]
fn restore_rejects_rewrite_asked_with_empty_rewriters() {
    let e = build_guidao_rewrite();
    let mut snap = e.save();
    let mut patched = false;
    for park in &mut snap.resolution.parks {
        if let MachineSnapshot::JudgePhase { current: Some(c), .. } = &mut park.machine {
            c.asked = true;
            c.rewriters.clear();
            patched = true;
        }
    }
    assert!(patched, "布阵应停在改判询问挂起点");
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "rewrite_asked_without_rewriter",
            ..
        })
    ));
}

// ---------------------------------------------------------------------------
// N08/N04 补充：鬼道装备区改判 + 嵌套检查点层级拒绝
// ---------------------------------------------------------------------------

use engine_native::LogEvent as LE;

/// 逐段推进直至鬼道改判询问打开；途中自动应答弃牌至上限等非改判选择。
/// 每步重压红桃布顶：跨回合摸牌会消耗布顶，判定牌需为红桃以固定基线。
fn advance_to_rewrite_ask(e: &mut Engine) {
    for _ in 0..48 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "gui_dao_rewrite" {
                return;
            }
            let sel: Vec<ChoiceOption> = if c.spec.prompt == "nullify_window" {
                vec![ChoiceOption::Pass]
            } else {
                c.spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(c.spec.min.max(1))
                    .collect()
            };
            let out = submit_cmd(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            continue;
        }
        e.debug_set_deck_top_by(|def| def.suit.is_heart());
        match next_phase(e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    panic!("48 步内未到达改判询问");
}

/// N08-c 布阵：2 号经**真实出牌动作**装备黑武器（PlayCard → Equipped →
/// 装备区），随后清空其手牌；3 号对 1 使用乐，判定落在下一轮 1 的判定阶段。
/// 返回武器牌 id。
fn setup_real_equip_rewrite() -> (Engine, u32) {
    let mut e = Engine::new(217, 4);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼道");
    // 回合 2：2 号真实出牌装备黑武器
    advance_to_play(&mut e, 2);
    let weapon = e.debug_place(2, "weapon2");
    assert!(matches!(
        e.catalog().cards[weapon as usize].suit,
        Suit::Spade | Suit::Club
    ));
    let out = play(&mut e, weapon, vec![]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(e.debug_equips(2), vec![weapon], "武器应进入装备区");
    // 回合 3：3 号对 1 使用乐（下一轮 1 的判定阶段触发改判）
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(2); // 此后 2 号仅剩装备区黑牌
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(3, "le");
    let _ = play(&mut e, le, vec![1]);
    // 无懈窗口：摸牌可能带来无懈，一律 Pass 使乐落入判定区
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let cid = c.id;
                let obs = c.observer;
                let out = submit_cmd(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
                assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
            }
            _ => break,
        }
    }
    (e, weapon)
}

fn judged_card_of_1(e: &Engine) -> u32 {
    e.log()
        .iter()
        .find_map(|l| match l {
            LE::Judged { player: 1, card, .. } => Some(*card),
            _ => None,
        })
        .expect("判定应已翻牌")
}

/// 牌区守恒（本文件局部版）：每张实体牌恰出现在一个牌区。
fn assert_equip_window_conservation(e: &Engine) {
    let core = e.state_core();
    let mut ids: Vec<u32> = Vec::new();
    for p in &core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(core.deck.iter().copied());
    ids.extend(core.discard.iter().copied());
    ids.extend(core.table.iter().copied());
    let total = ids.len();
    ids.sort_unstable();
    ids.dedup();
    assert_eq!(ids.len(), total, "实体牌重复出现在多个牌区");
    assert_eq!(ids.len(), e.catalog().cards.len(), "每张目录牌须恰在一个牌区");
}

fn rewrite_card_options(e: &Engine) -> Vec<u32> {
    e.open_choice()
        .unwrap()
        .spec
        .options
        .iter()
        .filter_map(|o| match o {
            ChoiceOption::Card(id) => Some(*id),
            _ => None,
        })
        .collect()
}

/// N08-c 反例一（仅装备区）：此前资格过滤只查手牌，仅装备黑武器者被跳过、
/// 不产生改判询问。现在询问到达且唯一牌选项来自装备区；出武器替换判定牌：
/// 武器离开装备区、随乐效果弃置，旧判定牌由改判者获得，Finalized 恰一次。
#[test]
fn guidao_rewrite_with_real_black_equip_only() {
    let (mut e, weapon) = setup_real_equip_rewrite();
    advance_to_rewrite_ask(&mut e);
    assert_eq!(e.open_choice().unwrap().observer, 2, "改判询问应到达 2 号");
    assert_eq!(
        rewrite_card_options(&e),
        vec![weapon],
        "唯一牌选项应为装备区黑武器"
    );
    let choice_id = e.open_choice().unwrap().id;
    let out = submit_cmd(
        &mut e,
        2,
        GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    // 判定牌被武器替换；旧判定牌由改判者获得（ADR 0714）
    let old_judge = judged_card_of_1(&e);
    assert!(e.log().iter().any(|l| matches!(l,
        LE::JudgementRewritten { player: 2, card, replaced }
        if *card == weapon && *replaced == old_judge)));
    assert!(e.debug_equips(2).is_empty(), "武器应离开装备区");
    assert!(e.debug_hand(2).contains(&old_judge), "旧判定牌应由改判者获得");
    assert!(
        e.state_core().discard.contains(&weapon),
        "武器应随乐效果弃置"
    );
    // Finalized 恰一次、在 Rewritten 之后、牌面为最终判定牌
    let p_rw = e
        .log()
        .iter()
        .position(|l| matches!(l, LE::JudgementRewritten { .. }))
        .unwrap();
    let p_fin = e
        .log()
        .iter()
        .position(|l| matches!(l, LE::JudgementFinalized { .. }))
        .unwrap();
    assert!(p_rw < p_fin, "Rewritten 先于 Finalized");
    assert_eq!(
        e.log().iter().filter(|l| matches!(l, LE::JudgementFinalized { .. })).count(),
        1,
        "Finalized 恰好一次"
    );
    assert!(e.log().iter().any(|l| matches!(l,
        LE::JudgementFinalized { player: 1, card } if *card == weapon)));
    assert_equip_window_conservation(&e);
}

/// N08-c 反例二（混合）：黑色手牌 + 黑色装备并存时选项须含两者
///（手牌在前、装备在后，与 kernel 选项构建一致）；出装备牌改判后手牌保留。
#[test]
fn guidao_rewrite_mixed_hand_and_equip_options() {
    let (mut e, weapon) = setup_real_equip_rewrite();
    let sha = place_black_sha(&mut e, 2);
    advance_to_rewrite_ask(&mut e);
    assert_eq!(e.open_choice().unwrap().observer, 2);
    assert_eq!(
        rewrite_card_options(&e),
        vec![sha, weapon],
        "选项应为手牌黑杀在前、装备区武器在后"
    );
    let choice_id = e.open_choice().unwrap().id;
    let out = submit_cmd(
        &mut e,
        2,
        GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert!(e.debug_hand(2).contains(&sha), "未选的黑杀应留在手牌");
    assert!(e.debug_equips(2).is_empty(), "武器应离开装备区");
    assert!(e.state_core().discard.contains(&weapon), "武器应随乐效果弃置");
    assert_equip_window_conservation(&e);
}

/// N08-c × N05：仅装备改判窗口挂起 → 保存/恢复 → 含装备选项完整重建 →
/// 恢复件出武器与原实例同归（此前 save 后 restore 报 OpenChoiceMismatch）。
#[test]
fn restore_guidao_rewrite_equips_options() {
    let (mut e, weapon) = setup_real_equip_rewrite();
    advance_to_rewrite_ask(&mut e);
    let c0 = e.open_choice().unwrap();
    assert_eq!(c0.observer, 2);
    let snap = e.save();
    let mut r = Engine::restore(&snap).unwrap();
    let c1 = r.open_choice().expect("恢复后改判询问应保持开放");
    let restored_id = c1.id;
    let restored_observer = c1.observer;
    assert_eq!(restored_id, c0.id, "选择 id 一致");
    assert_eq!(restored_observer, c0.observer, "观察者一致");
    assert_eq!(
        rewrite_card_options(&r),
        rewrite_card_options(&e),
        "恢复须重建含装备区的改判选项"
    );
    let out = submit_cmd(
        &mut r,
        2,
        GameAction::Answer {
            choice_id: restored_id,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    // 判定牌 id 从原实例日志取（快照不携带恢复前日志）
    let old_judge = judged_card_of_1(&e);
    assert!(r.log().iter().any(|l| matches!(l,
        LE::JudgementRewritten { player: 2, card, replaced }
        if *card == weapon && *replaced == old_judge)));
    assert!(r.debug_equips(2).is_empty(), "恢复件：武器应离开装备区");
    assert!(r.debug_hand(2).contains(&old_judge), "恢复件：旧判定牌应由改判者获得");
    assert!(r.state_core().discard.contains(&weapon), "恢复件：武器应随乐效果弃置");
    assert_eq!(
        r.log().iter().filter(|l| matches!(l, LE::JudgementFinalized { .. })).count(),
        1,
        "恢复件 Finalized 仍恰好一次"
    );
    assert_equip_window_conservation(&r);
}

/// N04：嵌套检查点（检查点内的检查点）→ 恢复拒绝（层级一致性）。
#[test]
fn restore_rejects_nested_checkpoint() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    // 构造双层：snap.segment_checkpoint 内层自身也有 segment_checkpoint
    let inner = snap.clone();
    snap.segment_checkpoint = Some(Box::new(engine_native::StateSnapshot {
        segment_checkpoint: Some(Box::new(inner)),
        ..snap.clone()
    }));
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_nested", .. })
    ));
}

/// 构造真实开放段：时间片 1 → 提交后 Suspended（段开放、携带检查点）。
fn build_open_segment() -> (Engine, engine_native::StateSnapshot) {
    let mut e = Engine::new(7, 4);
    e.debug_set_budget(100);
    e.set_time_slice(Some(1));
    let cur = e.turn().current;
    let out = submit_cmd(
        &mut e,
        cur,
        GameAction::NextPhase,
    );
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    assert!(e.is_suspended());
    let snap = e.save();
    (e, snap)
}

/// N04 检查点与当前段关系（正向）：开放段快照的检查点单调不超前、
/// 静止可中止；恢复件保留段开放状态且 abort 可用。
#[test]
fn open_segment_checkpoint_association_positive() {
    let (e, snap) = build_open_segment();
    let ckpt = snap
        .segment_checkpoint
        .as_ref()
        .expect("开放段快照必须携带检查点");
    // 单调关联：检查点 revision/dedup 不超前当前；自身静止（非段开放）
    assert!(ckpt.core.revision <= snap.core.revision, "检查点 revision 单调");
    assert!(ckpt.last_dedup <= snap.last_dedup, "检查点 dedup 高水位单调");
    assert!(!ckpt.segment_open, "检查点是段启动时的静止状态");
    assert!(!ckpt.segment_checkpoint.is_some(), "检查点不再嵌套");
    // 恢复件：段开放延续、abort 可回滚到段前并继续对局
    let mut r = Engine::restore(&snap).expect("restore open segment");
    assert!(r.is_suspended(), "恢复件应延续段开放状态");
    r.abort_segment().expect("恢复件 abort 应可用");
    assert!(!r.is_suspended());
    let cur = r.turn().current;
    let out = submit_cmd(&mut r, cur, GameAction::NextPhase);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let _ = e;
}

/// N04 检查点与当前段关系（负向）：段开放但无检查点、检查点 revision/dedup
/// 超前、座数/模式不一致、检查点自身段开放——全部恢复拒绝。
#[test]
fn restore_rejects_segment_checkpoint_mismatches() {
    // (1) 段开放但无检查点
    let (_e, snap) = build_open_segment();
    let mut no_ckpt = snap.clone();
    no_ckpt.segment_checkpoint = None;
    assert!(matches!(
        Engine::restore(&no_ckpt),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_missing_for_open_segment", .. })
    ));

    fn mk(
        snap: &engine_native::StateSnapshot,
    ) -> (engine_native::StateSnapshot, engine_native::StateSnapshot) {
        let m = snap.clone();
        let ckpt = (**m.segment_checkpoint.as_ref().unwrap()).clone();
        (m, ckpt)
    }
    // (2) 检查点 revision 超前
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.core.revision = m.core.revision + 5;
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_revision_ahead", .. })
    ));
    // (3) 检查点 dedup 高水位超前
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.last_dedup = m.last_dedup + 1;
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_dedup_ahead", .. })
    ));
    // (4) 检查点座数与当前不一致
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.core.players.truncate(3);
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_players_mismatch", .. })
    ));
    // (5) 检查点模式与当前不一致
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.core.mode = engine_native::GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_mode_mismatch", .. })
    ));
    // (6) 检查点自身段开放
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.segment_open = true;
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds { what: "checkpoint_segment_open", .. })
    ));
}

// ---------------------------------------------------------------------------
// N05 补充：鬼道改判恢复组合 + 足量桃存活 + 判定段完成
// N07 补充：非空日志中止
// ---------------------------------------------------------------------------


/// N05 改判恢复组合：鬼道改判窗口挂起 → 保存 → 恢复 → 继续应答。
#[test]
fn restore_guidao_rewrite_mid_window() {
    let mut e = Engine::new(227, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼道");
    let black = place_black_sha(&mut e, 2);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    // 推进到 1 的判定阶段：鬼道改判窗口
    for _ in 0..24 {
        if e.open_choice().is_some() { break; }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.open_choice().unwrap().observer, 2);
    // 保存改判窗口状态
    let snap = e.save();
    let mut r = Engine::restore(&snap).unwrap();
    assert_eq!(r.state_hash(), e.state_hash());
    // 恢复件继续应答改判
    let r2 = r.submit(GameCommand {
        actor: 2,
        expected_revision: r.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: snap.resolution.open_choice.as_ref().unwrap().id,
            selection: vec![ChoiceOption::Card(black)],
        },
    });
    assert!(matches!(r2, CommandOutcome::Accepted { .. }), "{r2:?}");
    // 乐改判成黑桃 → 1 跳过出牌阶段
    advance_to_play(&mut r, 1);
    assert!(r.debug_judge(1).is_empty());
}


/// N07 非空日志中止：在已有事件日志的段上 abort 后日志截断至段前。
#[test]
fn abort_segment_truncates_log() {
    let mut e = Engine::new(7, 4);
    // 第一段：用小预算制造快速耗尽
    e.debug_set_budget(1);
    let pre_hash = e.state_hash();
    let cur = e.turn().current;
    let out = e.submit(GameCommand { actor: cur, expected_revision: e.revision(), dedup_id: fresh_dedup(),
        action: GameAction::NextPhase });
    assert!(matches!(out, CommandOutcome::Rejected(RejectReason::BudgetExhausted)), "{out:?}");
    // 第二段：正常预算 + 时间片 → 暂停后中止
    e.debug_set_budget(1000);
    e.set_time_slice(Some(1));
    let out = e.submit(GameCommand { actor: cur, expected_revision: e.revision(), dedup_id: fresh_dedup(),
        action: GameAction::NextPhase });
    assert!(matches!(out, CommandOutcome::Suspended), "{out:?}");
    // 续行几次后中止
    let _ = e.resume();
    e.abort_segment().expect("abort open segment");
    assert!(!e.is_suspended());
    assert_eq!(e.state_hash(), pre_hash, "中止后应回到段前状态");
    // 取消时间片后继续对局
    e.set_time_slice(None);
    // 对局可继续
    let cur = e.turn().current;
    let out = e.submit(GameCommand { actor: cur, expected_revision: e.revision(), dedup_id: fresh_dedup(),
        action: GameAction::NextPhase });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
}

// ---------------------------------------------------------------------------
// N11 字节存档 v2：选择种类显式编码（此前由提示词静态推导并已漂移——
// nullify_window 实为 Confirm、jiji_target 实为 SelectPlayers、
// tian_du_gain 未注册；三类挂起点在 v1 下字节恢复均被拒绝）。
// ---------------------------------------------------------------------------

use engine_native::archive::{decode_archive, encode_archive, ArchiveError};
use engine_native::ChoiceKind;

/// 挂起点字节往返：encode→decode→restore 后种类/提示词/哈希必须逐一还原。
fn assert_choice_byte_roundtrip(
    build: fn() -> Engine,
    expected_prompt: &str,
    expected_kind: ChoiceKind,
) {
    let e = build();
    let choice = e.open_choice().expect("build 必须停在挂起点");
    assert_eq!(choice.spec.prompt, expected_prompt, "挂起提示词");
    assert_eq!(choice.spec.kind, expected_kind, "机器实际种类");
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).expect("decode");
    let restored = Engine::restore(&decoded).expect("字节恢复必须接受合法挂起快照");
    let rc = restored.open_choice().expect("恢复件仍挂起");
    assert_eq!(rc.spec.prompt, expected_prompt, "恢复后提示词");
    assert_eq!(rc.spec.kind, expected_kind, "恢复后种类必须逐字节还原");
    assert_eq!(restored.state_hash(), e.state_hash(), "恢复件同哈希");
}

/// 使用时无懈窗口：Confirm（v1 曾还原为 RespondCard → OpenChoiceMismatch）。
#[test]
fn archive_choice_kind_nullify_window_roundtrip() {
    assert_choice_byte_roundtrip(build_judge_window, "nullify_window", ChoiceKind::Confirm);
}

/// 遗计给牌目标：SelectPlayers（v1 曾还原为 RespondCard）。
#[test]
fn archive_choice_kind_jiji_target_roundtrip() {
    fn build() -> Engine {
        let mut e = Engine::new(468, 4);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "遗计");
        let out = e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
        let c = e.open_choice().expect("遗计触发确认");
        assert_eq!(c.spec.prompt, "trigger_confirm");
        answer_sel(&mut e, vec![ChoiceOption::Confirm]);
        e
    }
    assert_choice_byte_roundtrip(build, "jiji_target", ChoiceKind::SelectPlayers);
}

/// 天妒获得判定牌：Confirm；提示词 tian_du_gain 必须在注册表内（v1 未注册）。
#[test]
fn archive_choice_kind_tian_du_gain_roundtrip() {
    fn build() -> Engine {
        let mut e = Engine::new(213, 4);
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "天妒");
        let le = e.debug_place(0, "le");
        play(&mut e, le, vec![1]);
        loop {
            if e.open_choice().is_some() {
                break;
            }
            match next_phase(&mut e) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("{other:?}"),
            }
        }
        e
    }
    assert_choice_byte_roundtrip(build, "tian_du_gain", ChoiceKind::Confirm);
}

/// 决斗应杀：RespondCard 种类同样逐字节还原。
#[test]
fn archive_choice_kind_duel_sha_roundtrip() {
    assert_choice_byte_roundtrip(build_duel_respond, "duel_sha", ChoiceKind::RespondCard);
}

/// 弃牌上限：SelectCards 种类同样逐字节还原。
#[test]
fn archive_choice_kind_discard_to_limit_roundtrip() {
    assert_choice_byte_roundtrip(build_discard_to_limit, "discard_to_limit", ChoiceKind::SelectCards);
}

/// 注册表守卫：未注册提示词在解码期拒绝（BadTag），版本门禁同时拦截旧 v1 字节。
#[test]
fn archive_rejects_unknown_prompt_and_v1_bytes() {
    let e = build_judge_window();
    let mut snap = e.save();
    let open = snap
        .resolution
        .open_choice
        .as_mut()
        .expect("挂起快照含开放选择");
    open.spec.prompt = "rogue_prompt_from_extension";
    assert_eq!(
        decode_archive(&encode_archive(&snap)),
        Err(ArchiveError::BadTag("prompt_unknown", 0)),
        "未知提示词必须被注册表拒绝"
    );

    // v1 字节（格式版本字段=1）在 v2 解码器下明确拒绝，不静默按旧布局解读
    //（校验和仅覆盖 payload，头部版本字段可直接改写）。
    let mut bytes = encode_archive(&e.save());
    bytes[4] = 1;
    bytes[5] = 0;
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::UnsupportedFormatVersion(1)),
        "旧 v1 布局不得被 v2 解码器静默解读"
    );
}

// ---------------------------------------------------------------------------
// N04-f 牌归属完整核算：本局物理牌集合 = 目录全集。此前只查
// 「0..最高已见连续」——删最高 Card(79) 或清空全部牌区的快照都被接受。
// ---------------------------------------------------------------------------

/// 从所有牌区删除目录最高 id 的牌 → 恢复必须拒绝（尾端缺失）。
#[test]
fn restore_rejects_trailing_card_missing() {
    let e = Engine::new(443, 4);
    let mut snapshot = e.save();
    let highest = e.catalog().cards.len() as u32 - 1;
    for p in &mut snapshot.core.players {
        p.hand.retain(|c| *c != highest);
        p.equips.retain(|c| *c != highest);
        p.judge.retain(|c| *c != highest);
    }
    snapshot.core.deck.retain(|c| *c != highest);
    snapshot.core.discard.retain(|c| *c != highest);
    snapshot.core.table.retain(|c| *c != highest);
    assert!(
        matches!(
            Engine::restore(&snapshot),
            Err(SnapshotError::IdOutOfBounds { what: "card_missing_from_all_zones", got })
                if got == highest as u64
        ),
        "尾端缺失必须被拒绝"
    );
    // 原快照（全集齐）必须仍然可恢复
    Engine::restore(&e.save()).expect("原快照合法");
}

/// 清空全部牌区（无机器暂离持牌）→ 恢复必须拒绝（全量缺失）。
#[test]
fn restore_rejects_all_cards_missing() {
    let e = Engine::new(444, 3);
    let mut snapshot = e.save();
    for p in &mut snapshot.core.players {
        p.hand.clear();
        p.equips.clear();
        p.judge.clear();
    }
    snapshot.core.deck.clear();
    snapshot.core.discard.clear();
    snapshot.core.table.clear();
    assert!(
        matches!(
            Engine::restore(&snapshot),
            Err(SnapshotError::IdOutOfBounds { what: "card_missing_from_all_zones", .. })
        ),
        "全量缺失必须被拒绝"
    );
}

/// 中段删除（保留两端）→ 恢复必须拒绝（此前已有此语义，随完整核算保留）。
#[test]
fn restore_rejects_middle_card_missing() {
    let e = Engine::new(445, 3);
    let mut snapshot = e.save();
    let mid = e.catalog().cards.len() as u32 / 2;
    for p in &mut snapshot.core.players {
        p.hand.retain(|c| *c != mid);
        p.equips.retain(|c| *c != mid);
        p.judge.retain(|c| *c != mid);
    }
    snapshot.core.deck.retain(|c| *c != mid);
    snapshot.core.discard.retain(|c| *c != mid);
    snapshot.core.table.retain(|c| *c != mid);
    assert!(matches!(
        Engine::restore(&snapshot),
        Err(SnapshotError::IdOutOfBounds { what: "card_missing_from_all_zones", .. })
    ));
}

/// N11/A5：受 tag 冲突影响的挂起机器字节往返与拒绝负例。
///
/// 历史：XiaoJi（枭姬摸牌机）与 Dismantle 曾共用编码 tag 29，解码的
/// XiaoJi 分支不可达——枭姬挂起无法恢复（编译器 unreachable-pattern）。
/// 2026-09-24 修复为 XiaoJi=43（表尾追加）。语义边界：29 的解码语义保持
/// Dismantle（含 Dismantle 挂起的旧 v2 字节兼容）；XiaoJi 挂起的旧字节
/// 从未可正确解码，新表下未分配 tag 显式拒绝。
#[test]
fn xiaoji_machine_byte_roundtrip_and_rejects() {
    use engine_native::snap::ParkSnapshot;

    fn with_machine(m: MachineSnapshot) -> engine_native::StateSnapshot {
        let mut snap = Engine::new(7, 2).save();
        snap.resolution.parks = vec![ParkSnapshot {
            resume_depth: 0,
            slot: Slot::Intrinsic { frame: 0 },
            machine: m,
        }];
        snap
    }
    fn payload(data: &[u8]) -> &[u8] {
        &data[22..data.len() - 32]
    }
    // 重算校验和：篡改 payload 后通过帧完整性检查，进入解码语义校验。
    fn forge_checksum(mut data: Vec<u8>) -> Vec<u8> {
        let payload_len = data.len() - 22 - 32;
        let checksum = blake3::hash(&data[22..22 + payload_len]);
        data[22 + payload_len..].copy_from_slice(checksum.as_bytes());
        data
    }

    // 往返：XiaoJi（新 tag 43）。
    let snap = with_machine(MachineSnapshot::XiaoJi { owner: 1 });
    let data = encode_archive(&snap);
    assert_eq!(decode_archive(&data).expect("枭姬挂起字节往返"), snap);

    // 往返：Dismantle（tag 29 语义不变——旧 v2 字节兼容）。
    let snap_d = with_machine(MachineSnapshot::Dismantle {
        card: 3,
        source: 0,
        target: 1,
        asked: true,
    });
    let data_d = encode_archive(&snap_d);
    assert_eq!(decode_archive(&data_d).expect("拆牌挂起字节往返"), snap_d);

    // 负例：机器 tag 改为未分配的 255 → BadTag("machine", 255)。
    // 定位：payload 末尾 9 字节 = XiaoJi(tag+pid) + open_choice/answer/
    // budget/checkpoint 各 1 字节（budget u32=4）；失准时下方断言先失败。
    let machine_tag_pos = payload(&data).len() - 9;
    assert_eq!(payload(&data)[machine_tag_pos], 43, "机器 tag 定位失准");
    let mut corrupted = data.clone();
    corrupted[22 + machine_tag_pos] = 255;
    let forged = forge_checksum(corrupted);
    assert!(matches!(
        decode_archive(&forged),
        Err(ArchiveError::BadTag("machine", 255))
    ));

    // 负例：短输入（截去末字节，长度字段与实际不符）→ UnexpectedEof。
    assert!(matches!(
        decode_archive(&data[..data.len() - 1]),
        Err(ArchiveError::UnexpectedEof)
    ));
}

/// P0 解码加固（2026-10-06 探针复现缺陷的关闭验证）：
/// ① declared_kinds 中的未知牌种编号返回结构化 `UnknownCardKind`（完整
///    u32），不再 panic；
/// ② 快照读尽后载荷仍有剩余字节（重算长度与校验和的夹带数据）返回
///    `TrailingBytes`，不再被静默接受；
/// ③ `Winner::Faction` 的势力字节走显式 `enc_faction` 表——写入非法
///    势力值被 `dec_faction` 的 BadTag 拒绝，编码/解码同源。
#[test]
fn archive_hardening_rejects_unknown_kind_and_trailing_bytes() {
    use engine_native::catalog::Faction;
    use engine_native::state::Outcome;

    // 重算校验和：篡改/追加 payload 后通过帧完整性检查，进入语义校验。
    fn seal(bytes: &mut [u8]) {
        let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
        let hash = blake3::hash(&bytes[22..22 + len]);
        bytes[22 + len..].copy_from_slice(hash.as_bytes());
    }

    // ① 未知牌种编号（双快照差分定位 declared_kinds 的 kind 字节）。
    let mut snapshot = Engine::new(7, 4).save();
    snapshot.core.declared_kinds.insert(0, CardKind::Sha);
    let mut other = snapshot.clone();
    other.core.declared_kinds.insert(0, CardKind::Tao);
    let mut bytes = encode_archive(&snapshot);
    let other_bytes = encode_archive(&other);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    let offsets: Vec<_> = (22..22 + len)
        .filter(|&i| bytes[i] != other_bytes[i])
        .collect();
    assert_eq!(offsets.len(), 1, "declared_kinds kind 字节应唯一可定位");
    bytes[offsets[0]] = 255;
    seal(&mut bytes);
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::UnknownCardKind(255)),
        "未知牌种编号必须返回结构化错误且保留完整编号"
    );

    // ② 载荷尾部夹带 4 字节（重算长度与校验和）→ TrailingBytes。
    let snapshot = Engine::new(7, 4).save();
    let mut bytes = encode_archive(&snapshot);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    bytes.splice(22 + len..22 + len, [0xde, 0xad, 0xbe, 0xef]);
    bytes[14..22].copy_from_slice(&((len + 4) as u64).to_le_bytes());
    seal(&mut bytes);
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::TrailingBytes { extra: 4 }),
        "快照读尽后的剩余载荷必须被拒绝"
    );

    // ③ Winner::Faction 往返（enc_faction 显式表路径）+ 非法势力值拒绝。
    let mut snapshot = Engine::new(7, 4).save();
    snapshot.core.outcome = Some(Outcome {
        winner: Winner::Faction(Faction::Wu),
    });
    let data = encode_archive(&snapshot);
    assert_eq!(decode_archive(&data).expect("势力胜负往返"), snapshot);
    let mut bytes = data.clone();
    // outcome 段在 payload 中段：用仅势力不同的双快照差分定位 faction 字节。
    let mut other = snapshot.clone();
    other.core.outcome = Some(Outcome {
        winner: Winner::Faction(Faction::Qun),
    });
    let other_bytes = encode_archive(&other);
    let len = u64::from_le_bytes(bytes[14..22].try_into().unwrap()) as usize;
    let offsets: Vec<_> = (22..22 + len)
        .filter(|&i| bytes[i] != other_bytes[i])
        .collect();
    assert_eq!(offsets.len(), 1, "faction 字节应唯一可定位");
    let faction_pos = offsets[0];
    assert_eq!(bytes[faction_pos], 2, "Wu 应编码为 2（显式表）");
    bytes[faction_pos] = 5;
    seal(&mut bytes);
    assert_eq!(
        decode_archive(&bytes),
        Err(ArchiveError::BadTag("faction", 5)),
        "非法势力值必须被显式表拒绝"
    );
}
