use super::*;

/// 星宿阶段2系星夏片（规则 v242/存档 v57）：星夏（祝融主动技）——①对
/// 焰灵 2 点火伤；②追击线：选一名敌方 → 其弃一张红色牌（手牌+装备）或
/// 受 1 点火伤（可放弃弃牌）；③可放弃追击；④同轮再发动被间隔门控拒；
/// ⑤xingxia_used_round 入存档往返。祝融条目（群目录近似/hp5/[神裔,星
/// 夏]；赤炎3 已处置关卡面；赤翼/白翼=青翼同族轮次编排面已登记处置）。
#[test]
fn boss_huoshenzhurong_xingxia_yanling_burn() {
    // 建局：1=祝融（发动者）、4=fellow 焰灵（对焰灵专用目标）
    let mut e = Engine::new_boss_stage(62001, 4, 1, 3, 6, 4, 0);
    let zr_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 18) as u16; // BOSS_GENERALS[18] 祝融
    let yl_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 17) as u16; // BOSS_GENERALS[17] 焰灵
    e.debug_set_general_def(1, zr_def);
    e.debug_set_hp(1, 5);
    let fellow = e.debug_add_fellow(4).expect("fellow 入队");
    e.debug_set_general_def(fellow, yl_def);
    assert_eq!(
        e.catalog().general_meta(zr_def).map(|g| g.0),
        Some("祝融"),
        "祝融入目录"
    );
    let xdef = e.catalog().skill_by_name("星夏").expect("星夏入目录");
    assert!(e.state_core().skills.contains_key(&(1, xdef)));
    assert_eq!(e.debug_hp(fellow), 4, "焰灵就位");

    // ① 发动：损焰灵线（2 点火伤）+ 追击询问挂起
    advance_to_play(&mut e, 1);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: xdef },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 不 drain：机器已推进至敌方询问挂起（min=0 的询问会被空应答吞掉）
    assert_eq!(e.debug_hp(fellow), 2, "星夏：焰灵受 2 点火伤");
    assert!(
        e.state_core().player(1).xingxia_used_round == e.state_core().turn.round,
        "轮次登记"
    );

    // ② 追击线：选敌方 2 号（持红牌）→ 其弃红牌（免伤）
    let c = e.open_choice().expect("敌方询问应挂起");
    assert_eq!(c.observer, 1);
    assert!(c.spec.allow_cancel, "追击可放弃");
    answer_sel(&mut e, vec![ChoiceOption::Player(2)]);
    // 不 drain：弃红询问同为 min=0，空应答会被吞
    let c = e.open_choice().expect("弃红询问应挂起");
    assert_eq!(c.observer, 2);
    let red = c
        .spec
        .options
        .iter()
        .find_map(|o| match o {
            ChoiceOption::Card(x) => Some(*x),
            _ => None,
        })
        .expect("2 号应有红色牌候选");
    answer_sel(&mut e, vec![ChoiceOption::Card(red)]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 4, "弃红免伤");

    // ③ 同轮再发动 → 间隔门控拒
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: xdef },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "同轮再发动应被拒：{out:?}"
    );

    // ④ 放弃追击线 + 弃牌拒 → 1 火伤（新 engine：换 3 号作目标、其持黑牌）
    let mut e2 = Engine::new_boss_stage(62002, 4, 1, 3, 6, 4, 0);
    e2.debug_set_general_def(1, zr_def);
    e2.debug_set_hp(1, 5);
    let fellow2 = e2.debug_add_fellow(4).expect("fellow 入队");
    e2.debug_set_general_def(fellow2, yl_def);
    advance_to_play(&mut e2, 1);
    let out = e2.submit(GameCommand {
        actor: 1,
        expected_revision: e2.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: xdef },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 不 drain：敌方询问挂起（min=0 会被空应答吞掉）
    assert_eq!(e2.debug_hp(fellow2), 2);
    // 放弃追击（空应答）
    let c = e2.open_choice().expect("敌方询问应挂起");
    assert_eq!(c.spec.min, 0, "追击可放弃（min=0）");
    answer_empty(&mut e2);
    drain_choices(&mut e2);
    assert!(e2.open_choice().is_none(), "放弃追击后不再有询问");
    assert_eq!(e2.debug_hp(2), 4, "未追击：2 号无损");

    // ⑤ 弃牌拒绝 → 1 火伤（恢复线用新局：3 号持黑牌被选为目标）
    let mut e3 = Engine::new_boss_stage(62003, 4, 1, 3, 6, 4, 0);
    e3.debug_set_general_def(1, zr_def);
    e3.debug_set_hp(1, 5);
    let fellow3 = e3.debug_add_fellow(4).expect("fellow 入队");
    e3.debug_set_general_def(fellow3, yl_def);
    advance_to_play(&mut e3, 1);
    e3.debug_clear_hand(2);
    let out = e3.submit(GameCommand {
        actor: 1,
        expected_revision: e3.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: xdef },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 不 drain：敌方询问挂起
    assert_eq!(e3.debug_hp(fellow3), 2);
    let c = e3.open_choice().expect("敌方询问应挂起");
    assert_eq!(c.observer, 1);
    answer_sel(&mut e3, vec![ChoiceOption::Player(2)]);
    // 不 drain：弃红询问挂起（min=0 会被空应答吞掉）
    // 2 号手牌已清 → 无红色候选 → 直接受伤（1 火伤）
    let c = e3.open_choice().expect("弃红询问应挂起");
    assert!(c.spec.options.is_empty(), "无红牌候选");
    answer_empty(&mut e3);
    drain_choices(&mut e3);
    assert_eq!(e3.debug_hp(2), 3, "弃牌拒绝 → 1 点火伤");

    // ⑥ 存档 v57 往返：xingxia_used_round 持久
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(
        r.state_core().player(1).xingxia_used_round,
        e.state_core().player(1).xingxia_used_round,
        "轮次入存档"
    );
    assert!(
        r.state_core().skills.contains_key(&(1, xdef)),
        "星夏实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
    assert_zone_conservation(&e3);
}
