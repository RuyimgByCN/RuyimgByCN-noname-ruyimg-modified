use super::*;

/// 刚烈（夏侯惇）正例：置顶非红桃 → 确认后来源受 1 点反伤，判定牌弃置。
#[test]
fn xiahoudun_ganglie_non_heart_reflects() {
    let mut e = Engine::new(802, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "刚烈");
    e.debug_set_hp(1, 4);
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    let hp1 = e.debug_hp(1);
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    answer(&mut e, ChoiceOption::Confirm);
    assert_eq!(
        e.debug_hp(1),
        hp1 - 1,
        "非红桃判定应对伤害来源造成 1 点反伤"
    );
    assert!(e.outcome().is_none());

    // 反例：红桃判定 → 无反伤。
    let mut e2 = Engine::new(803, 3);
    advance_to_play(&mut e2, 0);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(0, "刚烈");
    e2.debug_set_hp(1, 4);
    e2.debug_set_deck_top_by(|def| def.suit.is_heart());
    let (hp0, hp1) = (e2.debug_hp(0), e2.debug_hp(1));
    e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    answer(&mut e2, ChoiceOption::Confirm);
    assert_eq!(e2.debug_hp(1), hp1, "红桃判定不得反伤");
    assert_eq!(e2.debug_hp(0), hp0 - 1, "受伤者仅承受原始伤害");
    assert_zone_conservation(&e2);
}
