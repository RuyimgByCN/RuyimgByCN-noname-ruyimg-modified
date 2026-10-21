use super::*;

/// 遗计：受到伤害后摸两张并可给牌——确认 → 摸 2 → 给 1 号牌一张给 0 号
/// → 放弃第二次 → 结束；牌区守恒。
#[test]
fn guojia_jiji_draws_and_gives_one_card() {
    let mut e = Engine::new(467, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "遗计");
    // 无来源伤害（避免反馈类干扰）：0 打 1？直接伤害注入（来源 0）
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 遗计闸门确认
    let c = e.open_choice().expect("遗计触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    assert_eq!(c.observer, 1);
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 摸 2 后：jiji_target 询问（观察者 1，目标选项为其他玩家）
    let c = e.open_choice().expect("jiji_target 应挂起");
    assert_eq!(c.spec.prompt, "jiji_target");
    assert_eq!(c.observer, 1);
    assert_eq!(e.debug_hand(1).len(), 2, "遗计应已摸两张");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(0)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "应询问交哪张牌: {out:?}"
    );
    // jiji_card：交出第一张
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "jiji_card");
    let Some(ChoiceOption::Card(give)) = c.spec.options.first().copied() else {
        panic!("应存在手牌选项");
    };
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(give)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 第二轮目标询问：空应答放弃
    let c = e.open_choice().expect("第二轮目标询问");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 终态：1 交出一张给 0；守恒
    assert!(e.debug_hand(0).contains(&give), "0 号应收到给出的牌");
    assert!(!e.debug_hand(1).contains(&give), "1 号不再持有给出的牌");
    assert!(log_has(&e, |l| matches!(l,
        LE::CardsGiven { from: 1, to: 0, cards } if cards.contains(&give))));
    assert_zone_conservation(&e);
}
