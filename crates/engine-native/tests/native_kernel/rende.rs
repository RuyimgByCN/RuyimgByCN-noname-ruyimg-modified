use super::*;

/// 仁德分次累计（ADR 0776 对齐上游 rende1/content）：同一出牌阶段
/// 先给 1 张（不回血），第二次再给 1 张累计跨过 2 → 回复 1 点；
/// 累计计数入回合状态（字节存档恢复后口径不变）。
#[test]
fn rende_cumulative_crosses_two_across_activations() {
    use engine_native::archive::{decode_archive, encode_archive};
    let mut e = Engine::new(2901, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "仁德");
    e.debug_set_hp(0, 3);
    let c1 = e.debug_place(0, "shan");
    let c2 = e.debug_place(0, "shan");
    let c3 = e.debug_place(0, "shan");
    let skill = e.catalog().skill_by_name("仁德").unwrap();

    // 第一次发动：给 1 张给 1 号 → 不回血（1 < 2）
    submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    answer_sel(&mut e, vec![ChoiceOption::Card(c1)]);
    answer_sel(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.debug_hp(0), 3, "给 1 张不应回血");

    // 字节存档往返（累计计数 = 1 在快照中）
    let decoded = Engine::restore(&decode_archive(&encode_archive(&e.save())).expect("decode"))
        .expect("字节恢复");
    let mut e = decoded;

    // 第二次发动：再给 1 张给 2 号 → 累计 2 → 回复 1 点
    submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    answer_sel(&mut e, vec![ChoiceOption::Card(c2)]);
    answer_sel(&mut e, vec![ChoiceOption::Player(2)]);
    assert_eq!(
        e.debug_hp(0),
        4,
        "累计跨过 2 应回复 1 点（旧反例：被拒/不回血）"
    );

    // 第三次发动：再给 1 张 → 累计 3，不再回血
    submit_cmd(&mut e, 0, GameAction::ActivateSkill { skill });
    answer_sel(&mut e, vec![ChoiceOption::Card(c3)]);
    answer_sel(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.debug_hp(0), 4, "跨过后不再回血");
    assert_zone_conservation(&e);
}
