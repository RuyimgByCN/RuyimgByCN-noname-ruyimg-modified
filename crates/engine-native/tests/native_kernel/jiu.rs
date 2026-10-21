use super::*;

// ---------------------------------------------------------------------------
// G1 cardpile 前置（规则版本 30）：【酒】/【雷杀】牌种。
// 酒：出牌阶段每回合限一次，本回合使用的下一张杀伤害 +1；
//     濒死时对自身使用回复 1 点体力。雷杀：按杀结算（共享次数限制），
//     伤害为雷属性。
// ---------------------------------------------------------------------------

/// 酒 → 下一张杀 +1 伤害；同回合第二张酒被拒；杀结算后效果消耗
///（下一张杀回到 1 伤害）。
#[test]
fn jiu_boosts_next_sha_once_per_turn() {
    let mut e = Engine::new(3001, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jiu1 = e.debug_place(0, "jiu");
    let jiu2 = e.debug_place(0, "jiu");
    let sha = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    // 先饮酒，再出杀
    let outj = play(&mut e, jiu1, vec![]);
    assert!(matches!(outj, CommandOutcome::Accepted { .. }), "{outj:?}");
    play(&mut e, sha, vec![1]);
    assert_eq!(e.debug_hp(1), 2, "酒加成下杀应造成 2 点伤害");

    // 同回合第二张酒：CardLimit
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jiu2,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::CardLimit)),
        "每回合限一次: {out:?}"
    );

    // 效果已消耗：下一张杀回到 1 伤害
    e.debug_place(0, "sha");
    let hand = e.debug_hand(0);
    let sha2 = *hand.last().unwrap();
    let out2 = play(&mut e, sha2, vec![1]);
    assert!(
        matches!(out2, CommandOutcome::Rejected(RejectReason::ShaLimit)),
        "杀次数限制仍生效"
    );
    assert_zone_conservation(&e);
}

/// 酒效果跨恢复保持：酒→杀挂起出闪询问，字节往返后放弃出闪，
/// 伤害仍为 2（jiu_bonus 入快照与字节）。
#[test]
fn jiu_bonus_survives_byte_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let mut e = Engine::new(3002, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jiu = e.debug_place(0, "jiu");
    let sha = e.debug_place(0, "sha");
    let shan = e.debug_place(1, "shan");
    e.debug_set_hp(1, 4);
    play(&mut e, jiu, vec![]);
    play(&mut e, sha, vec![1]);
    let c = e.open_choice().expect("出闪询问挂起");
    assert_eq!((c.spec.prompt, c.observer), ("respond_shan", 1));
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore(&decode_archive(&bytes).expect("decode")).expect("restore");
    let cid = restored.open_choice().unwrap().id;
    submit_cmd(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan)],
        },
    );
    assert_eq!(restored.debug_hp(1), 4, "出闪免伤（4 血未受伤）");
    assert_eq!(
        restored.save().core.turn.jiu_active,
        false,
        "酒效果已随杀使用消耗"
    );
    assert_zone_conservation(&restored);
}

/// 濒死时酒可当桃使用：1 血角色受 1 点伤害 → 用酒回复至 1 血。
#[test]
fn jiu_saves_dying_player() {
    let mut e = Engine::new(3003, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let jiu = e.debug_place(1, "jiu");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    let c = e.open_choice().expect("濒死救援询问");
    assert_eq!(c.spec.prompt, "dying_save");
    assert_eq!(c.observer, 1);
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(jiu)),
        "濒死选项应含酒"
    );
    answer_sel(&mut e, vec![ChoiceOption::Card(jiu)]);
    assert_eq!(e.debug_hp(1), 1, "酒濒死使用回复 1 点");
    assert!(!e.debug_hand(1).contains(&jiu), "酒已消耗");
    assert_zone_conservation(&e);
}
