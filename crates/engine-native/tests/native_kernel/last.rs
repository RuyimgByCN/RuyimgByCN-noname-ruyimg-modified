use super::*;

/// N06 奖惩先后：先除内奸后主公杀末反贼——奖励摸三与 LordSide 终局
/// 在同一命令段内完成（奖励先于终局判定，不被终局短路）。
#[test]
fn last_rebel_reward_precedes_victory() {
    let mut e = Engine::try_new_identity(203, 4).expect("identity 4p");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let lord = seats_with_role(&e, Role::Lord)[0];
    let loyal = seats_with_role(&e, Role::Loyalist)[0];
    let traitor = seats_with_role(&e, Role::Traitor)[0];
    let rebel = seats_with_role(&e, Role::Rebel)[0];
    // 第一步：忠臣除掉内奸（无奖惩；反贼仍存活不终局）
    e.debug_set_hp(traitor, 1);
    let out = e.debug_deal_damage(Some(loyal), traitor, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(traitor));
    assert!(e.outcome().is_none(), "内奸死、反贼活：不终局");
    // 第二步：主公杀最后反贼——奖励摸三 + LordSide 终局同段完成
    let before = e.debug_hand(lord).len();
    e.debug_set_hp(rebel, 1);
    let out = e.debug_deal_damage(Some(lord), rebel, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(rebel));
    assert!(
        matches!(
            e.outcome(),
            Some(engine_native::Outcome {
                winner: Winner::LordSide
            })
        ),
        "反贼内奸全灭应判主公方胜"
    );
    assert_eq!(
        e.debug_hand(lord).len(),
        before + 3,
        "杀反贼摸三必须在终局判定同一命令段内入账（不被终局短路）"
    );
    // 事件：死亡事件先于奖励摸牌事件（奖励随死亡结算触达）
    let died = e
        .log()
        .iter()
        .position(|l| matches!(l, LogEvent::PlayerDied { player, .. } if *player == rebel));
    let drawn = e.log().iter().position(|l| {
        matches!(l,
        LogEvent::CardsDrawn { player, cards } if *player == lord && cards.len() == 3)
    });
    let (died, drawn) = (died.expect("死亡事件"), drawn.expect("奖励摸三事件"));
    assert!(died < drawn, "死亡事件应先于奖励摸牌");
    assert_zone_conservation(&e);
}
