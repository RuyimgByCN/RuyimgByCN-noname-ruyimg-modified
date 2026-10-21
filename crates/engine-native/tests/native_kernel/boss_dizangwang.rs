use super::*;

/// 不佛①：受到大于 1 点的伤害改为其中 1 点（3→2）；1 点不减。
#[test]
fn boss_dizangwang_bufo_damage_cap() {
    let mut e = engine_native::Engine::new_boss_stage(87054, 4, 0, 0, 6, 4, 0);
    let dzw_def = dizangwang_def_of(&e);
    e.debug_set_general_def(1, dzw_def);
    e.debug_set_hp(1, 8);
    assert_eq!(
        e.catalog().general_meta(dzw_def).map(|g| g.0),
        Some("地藏王"),
        "地藏王入目录"
    );
    for sk in ["不佛", "无量", "谛听"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);
    let out = e.debug_deal_damage(None, 1, 3, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 6, "不佛：3 点伤害减为 2");
    let out = e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    let _ = out;
    assert_eq!(hp(&e, 1), 5, "不佛：1 点伤害不减");
}

/// 不佛②：准备阶段距离 1 以内的其他存活受 1 火伤（谛听停用隔离——
/// 0/2 相邻受击、3 距离 2 不受）。
#[test]
fn boss_dizangwang_bufo_prepare_range1_fire() {
    let mut e = engine_native::Engine::new_boss_stage(87055, 4, 0, 3, 6, 4, 0);
    let dzw_def = dizangwang_def_of(&e);
    e.debug_set_general_def(1, dzw_def);
    e.debug_set_hp(1, 8);
    e.debug_set_skill_enabled(1, "谛听", false);
    advance_to_play(&mut e, 0);
    let (n0, n2, n3) = (hp(&e, 0), hp(&e, 2), hp(&e, 3));
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    assert_eq!(hp(&e, 0), n0 - 1, "不佛：相邻 0 号受 1 火伤");
    assert_eq!(hp(&e, 2), n2 - 1, "不佛：相邻 2 号受 1 火伤");
    assert_eq!(hp(&e, 3), n3, "不佛：距离 2 的 3 号不受");
}

/// 无量：准备阶段体力<3 回复至 3；结束阶段摸 2。
#[test]
fn boss_dizangwang_wuliang_prepare_recover_end_draw() {
    let mut e = engine_native::Engine::new_boss_stage(87056, 4, 0, 3, 6, 4, 0);
    let dzw_def = dizangwang_def_of(&e);
    e.debug_set_general_def(1, dzw_def);
    e.debug_set_hp(1, 2);
    e.debug_set_skill_enabled(1, "谛听", false);
    advance_to_play(&mut e, 0);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    assert_eq!(hp(&e, 1), 3, "无量：准备阶段回复至 3");
    // 推进 1 号回合到结束阶段：摸 2
    advance_to_play(&mut e, 1);
    next_phase_settled(&mut e); // Play→Discard
    let h_before = e.state_core().player(1).hand.len();
    next_phase_settled(&mut e); // Discard→End（无量摸 2）
    assert_eq!(
        e.state_core().player(1).hand.len(),
        h_before + 2,
        "无量：结束阶段摸 2"
    );
}

/// 谛听：自己算到他人 -1（1 号可杀 3 号）+他人算到自己 +1（3 号不可
/// 杀 1 号）。
#[test]
fn boss_dizangwang_diting_distance_both_ways() {
    let mut e = engine_native::Engine::new_boss_stage(87057, 4, 0, 3, 6, 4, 0);
    let dzw_def = dizangwang_def_of(&e);
    e.debug_set_general_def(1, dzw_def);
    e.debug_set_hp(1, 8);
    advance_to_play(&mut e, 0);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    // 1 号（谛听 from -1）算到 3 号：座位距离 2 → 1 → 可杀
    advance_to_play(&mut e, 1);
    let sha1 = give_black_sha(&mut e, 1);
    let out = play(&mut e, sha1, vec![3]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "谛听：1→3 距离 1 可杀（{out:?}）"
    );
    for _ in 0..4 {
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            None => break,
        }
    }
    // 3 号（谛听 to +1）算到 1 号：座位距离 2 → 3 → 不可杀
    advance_to_play(&mut e, 3);
    let sha3 = give_black_sha(&mut e, 3);
    assert!(
        matches!(
            play(&mut e, sha3, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "谛听：3→1 距离 3 不可杀"
    );
}
