use super::*;

/// 马术（马超）：与他人的距离 -1——4 人局隔座（距离 2）可杀。
#[test]
fn machao_mashu_distance_minus_one() {
    let sha_test = |mashu: bool| -> CommandOutcome {
        let mut e = Engine::new(956, 4);
        advance_to_play(&mut e, 0);
        if mashu {
            e.debug_grant(0, "马术");
        }
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let sha = e.debug_place(0, "sha");
        play(&mut e, sha, vec![2]) // 隔座距离 2 > 射程 1；马术下为 1
    };
    assert!(
        matches!(sha_test(true), CommandOutcome::Accepted { .. }),
        "马术下隔座可杀"
    );
    assert!(
        matches!(
            sha_test(false),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "无马术隔座不可杀"
    );
}

/// 铁骑（马超）：杀指定目标后判定——红色 → 目标不能出闪（直接伤害）；
/// 黑色 → 正常出闪询问。
#[test]
fn machao_tieji_judgement_blocks_shan_on_red() {
    // (1) 红桃判定 → 无出闪询问，直接 1 点伤害
    let mut e = Engine::new(957, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "铁骑");
    e.debug_set_hp(1, 4);
    let _shan = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    assert!(e.open_choice().is_none(), "红色判定后不得出现出闪询问");
    assert_eq!(e.debug_hp(1), 3, "红色判定 → 不能出闪 → 1 点伤害");

    // (2) 黑桃判定 → 正常出闪询问
    let mut e2 = Engine::new(958, 3);
    advance_to_play(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "铁骑");
    e2.debug_set_hp(1, 4);
    let _shan = e2.debug_place(1, "shan");
    e2.debug_set_deck_top_by(|def| def.suit.is_spade());
    let sha2 = e2.debug_place(0, "sha");
    play(&mut e2, sha2, vec![1]);
    let c = e2.open_choice().expect("黑色判定应正常询问出闪");
    assert_eq!(c.spec.prompt, "respond_shan");
    answer(&mut e2, ChoiceOption::Pass);
    assert_eq!(e2.debug_hp(1), 3, "放弃出闪 → 1 点伤害");
    assert_zone_conservation(&e2);
}
