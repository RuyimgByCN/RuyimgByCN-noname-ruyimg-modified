use super::*;

/// 星宿阶段2系末片（规则 v240/存档 v56）：不臣（勾芒主动技两线）——
/// ①复活线：场上有已死亡的书精 → 发动失 1 体并复活（回 1 体、摸 2）；
/// ②同轮再发动被间隔门控拒；③恢复线：无死亡书精 → 目标询问（同阵营受
/// 伤者）→ 回 2；④buchun_used_round 入存档往返。勾芒条目（群目录近似/
/// hp5/[神裔,不臣]；青木3 已处置）。
#[test]
fn boss_mushengoumang_buchun_revive_and_recover() {
    // ① 复活线
    let mut e = Engine::new_boss_stage(60001, 4, 1, 3, 6, 4, 0);
    let goumang_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 16) as u16; // BOSS_GENERALS[16] 勾芒
    let shujing_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 6) as u16; // BOSS_GENERALS[6] 书精
    e.debug_set_general_def(1, goumang_def);
    e.debug_set_general_def(2, shujing_def);
    e.debug_set_hp(1, 5);
    assert_eq!(
        e.catalog().general_meta(goumang_def).map(|g| g.0),
        Some("勾芒"),
        "勾芒入目录"
    );
    let bdef = e.catalog().skill_by_name("不臣").expect("不臣入目录");
    assert!(e.state_core().skills.contains_key(&(1, bdef)));

    // 击杀书精（2 号）：清手防自救 → 4 点直伤 → 濒死无人救 → 死亡
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_deal_damage(Some(1), 2, 4, DamageNature::Normal);
    drain_choices(&mut e);
    assert!(!e.debug_alive(2), "书精应死亡");

    // 发动不臣（复活线）：失 1 体 + 书精复活（hp1、摸 2）
    let before_hand = e.debug_hand(2).len();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: bdef },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 4, "不臣复活线：失 1 体");
    assert!(e.debug_alive(2), "书精复活");
    assert_eq!(e.debug_hp(2), 1, "复活至 1 点");
    assert_eq!(e.debug_hand(2).len(), before_hand + 2, "复活摸 2");
    assert!(
        e.state_core().player(1).buchun_used_round == e.state_core().turn.round,
        "轮次登记"
    );

    // ② 同轮再发动 → 间隔门控拒（InvalidChoice）
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: bdef },
    });
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "同轮再发动应被拒：{out:?}"
    );
    assert_zone_conservation(&e);

    // ③ 恢复线（无死亡书精）：fellow 受伤 → 目标询问 → 回 2
    let mut e2 = Engine::new_boss_stage(60002, 4, 1, 3, 6, 4, 0);
    e2.debug_set_general_def(1, goumang_def);
    e2.debug_set_hp(1, 5);
    let fellow = e2.debug_add_fellow(4).expect("fellow 入队");
    e2.debug_set_hp(fellow, 2);
    advance_to_play(&mut e2, 1);
    let out = e2.submit(GameCommand {
        actor: 1,
        expected_revision: e2.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: bdef },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "恢复线应挂目标询问：{out:?}"
    );
    let c = e2.open_choice().expect("目标询问挂起");
    assert_eq!(c.observer, 1);
    assert!(
        c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Player(x) if *x == fellow)),
        "候选含受伤 fellow"
    );
    answer_sel(&mut e2, vec![ChoiceOption::Player(fellow)]);
    drain_choices(&mut e2);
    assert_eq!(e2.debug_hp(fellow), 4, "恢复线：fellow 回 2（2→4）");
    assert!(
        e2.state_core().player(1).buchun_used_round == e2.state_core().turn.round,
        "轮次登记（恢复线）"
    );

    // ④ 存档 v56 往返：buchun_used_round 持久
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(
        r.state_core().player(1).buchun_used_round,
        e.state_core().player(1).buchun_used_round,
        "轮次入存档"
    );
    assert!(
        r.state_core().skills.contains_key(&(1, bdef)),
        "不臣实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e2);
}
