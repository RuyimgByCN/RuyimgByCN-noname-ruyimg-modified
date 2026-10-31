use super::*;

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
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
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
        assert_eq!(
            resumed.state_hash(),
            cur.state_hash(),
            "选择点 {} 恢复哈希一致",
            i
        );
        // 恢复件在该点的开放选择与基准线一致
        let got = resumed
            .open_choice()
            .map(|c| (c.id, c.spec.prompt))
            .expect("选择点应挂起");
        assert_eq!(got, points[i], "选择点 {} 的开放选择应一致", i);
        let out = answer_sel(&mut resumed, sel.clone());
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
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
        other => panic!(
            "expected rule version mismatch, got {:?}",
            other.map(|_| ())
        ),
    }
    // 状态内版本与快照一致时可恢复
    assert_eq!(e.save().ruleset_version, e.ruleset_version());
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
        Err(SnapshotError::IdOutOfBounds {
            what: "skill_instance_owner",
            got: 255
        })
    ));
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
    assert!(
        matches!(
            out,
            CommandOutcome::Rejected(RejectReason::DuplicateCommand)
        ),
        "{out:?}"
    );
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

#[test]
fn restore_rejects_broken_frame_parent_chain() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.stack[0].parent = Some(999);
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "frame_parent",
            ..
        })
    ));
}

#[test]
fn restore_rejects_park_depth_overrun() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.parks[0].resume_depth = 999;
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "resume_depth",
            ..
        })
    ));
}

#[test]
fn restore_rejects_dangling_park_frame() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.parks[0].slot = Slot::Trigger { frame: 999 };
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "park_frame",
            ..
        })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "event_id_dup",
            ..
        })
    ));
}

#[test]
fn restore_rejects_oob_event_payload() {
    let e = setup_sancai_pending();
    let mut snap = e.save();
    snap.resolution.stack[0].data = EventData::Linked {
        player: 200,
        linked: true,
    };
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
        Err(SnapshotError::IdOutOfBounds {
            what: "skill_def",
            ..
        })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "choice_id",
            ..
        })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "frame_parent",
            ..
        })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "choice_without_machine",
            ..
        })
    ));
}

/// N04-c 补充：挂起深度低于当前栈深是合法形态（AOE 段间伤害帧在机器之上
/// 先行结算）——此类快照必须可恢复。
#[test]
fn restore_accepts_park_depth_below_stack() {
    let mut e = setup_sancai_pending();
    e.set_time_slice(Some(1));
    for _ in 0..100 {
        let snap = e.save();
        if snap
            .resolution
            .parks
            .iter()
            .any(|park| park.resume_depth < snap.resolution.stack.len())
        {
            Engine::restore(&snap).expect("a real child frame above its parent park must restore");
            return;
        }
        if let Some(choice) = e.open_choice() {
            let selection = answer_for(&choice.spec);
            answer_sel(&mut e, selection);
        } else if e.is_suspended() {
            e.resume();
        } else {
            break;
        }
    }
    panic!("the real continuation must yield with a child above its parent park");
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
        Err(SnapshotError::IdOutOfBounds {
            what: "lord_seat",
            ..
        })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "roles_len",
            ..
        })
    ));
    // (2) 全忠臣（首位非主公 → lord_seat 先触发）
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        *roles = vec![Role::Loyalist; 5];
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "lord_seat",
            ..
        })
    ));
    // (3) 主公在首位但无内奸 → 组成校验
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        *roles = vec![
            Role::Lord,
            Role::Loyalist,
            Role::Rebel,
            Role::Rebel,
            Role::Rebel,
        ];
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "roles_composition",
            ..
        })
    ));
    // (4) 忠臣数与人数表不符（5 人局 2 忠）→ 拒绝（N04 精确人数表）
    let mut snap = e.save();
    if let GameMode::Identity { roles } = &mut snap.core.mode {
        *roles = vec![
            Role::Lord,
            Role::Loyalist,
            Role::Loyalist,
            Role::Rebel,
            Role::Traitor,
        ];
    }
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "roles_loyalist_count",
            ..
        })
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
    snap.core.outcome = Some(Outcome {
        winner: Winner::LordSide,
    });
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "outcome_mode_mismatch",
            ..
        })
    ));
    // 身份局 + 玩家胜者 → 拒绝
    let e = Engine::new_identity(185, 5);
    let mut snap = e.save();
    snap.core.outcome = Some(Outcome {
        winner: Winner::Player(0),
    });
    assert!(matches!(
        Engine::restore(&snap),
        Err(SnapshotError::IdOutOfBounds {
            what: "outcome_mode_mismatch",
            ..
        })
    ));
}

/// N04-d：改判阶段 asked=true 但改判队列已空的快照必须拒绝
/// （恢复后应答会在空队列上 panic——复核探针复现项的回归）。
#[test]
fn restore_rejects_rewrite_asked_with_empty_rewriters() {
    let e = build_guidao_rewrite();
    let mut snap = e.save();
    let mut patched = false;
    for park in &mut snap.resolution.parks {
        if let MachineSnapshot::JudgePhase {
            current: Some(c), ..
        } = &mut park.machine
        {
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
    assert!(
        r.debug_hand(2).contains(&old_judge),
        "恢复件：旧判定牌应由改判者获得"
    );
    assert!(
        r.state_core().discard.contains(&weapon),
        "恢复件：武器应随乐效果弃置"
    );
    assert_eq!(
        r.log()
            .iter()
            .filter(|l| matches!(l, LE::JudgementFinalized { .. }))
            .count(),
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
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_nested",
            ..
        })
    ));
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
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_missing_for_open_segment",
            ..
        })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_revision_ahead",
            ..
        })
    ));
    // (3) 检查点 dedup 高水位超前
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.last_dedup = m.last_dedup + 1;
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_dedup_ahead",
            ..
        })
    ));
    // (4) 检查点座数与当前不一致
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.core.players.truncate(3);
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_players_mismatch",
            ..
        })
    ));
    // (5) 检查点模式与当前不一致
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.core.mode = engine_native::GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_mode_mismatch",
            ..
        })
    ));
    // (6) 检查点自身段开放
    let (mut m, mut ckpt) = mk(&snap);
    ckpt.segment_open = true;
    m.segment_checkpoint = Some(Box::new(ckpt));
    assert!(matches!(
        Engine::restore(&m),
        Err(SnapshotError::IdOutOfBounds {
            what: "checkpoint_segment_open",
            ..
        })
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
        if e.open_choice().is_some() {
            break;
        }
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
            Err(SnapshotError::IdOutOfBounds {
                what: "card_missing_from_all_zones",
                ..
            })
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
        Err(SnapshotError::IdOutOfBounds {
            what: "card_missing_from_all_zones",
            ..
        })
    ));
}
