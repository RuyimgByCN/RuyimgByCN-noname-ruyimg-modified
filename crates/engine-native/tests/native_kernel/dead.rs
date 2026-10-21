use super::*;

/// N08 动态技能资格（死亡）：改判者死亡后即使持黑牌也被资格队列排除，
/// 判定静默完成（对齐 kernel 询问循环先查存活再查黑牌）。
#[test]
fn dead_skill_holder_excluded_from_rewrite() {
    let mut e = Engine::new(281, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼道");
    let _black = place_black_sha(&mut e, 2);
    // 判定前杀死 2 号（原型局 4 人，不触发终局）
    e.debug_set_hp(2, 1);
    let out = e.debug_deal_damage(None, 2, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(2));
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    for _ in 0..24 {
        if e.log()
            .iter()
            .any(|l| matches!(l, LE::JudgementFinalized { .. }))
        {
            break;
        }
        assert!(e.open_choice().is_none(), "死亡改判者不应被询问");
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementRewritten { .. })),
        0
    );
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementFinalized { .. })),
        1
    );
}
