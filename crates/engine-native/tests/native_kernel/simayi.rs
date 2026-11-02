use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：司马懿（鬼才 + 反馈）（ADR 0736）
// ---------------------------------------------------------------------------

/// 鬼才：任意花色手牌可改判任意角色的判定（区别于鬼道的黑色限定）。
/// 红桃手牌改判乐不思蜀 → 判定生效为红桃 → 不跳过出牌阶段。
#[test]
fn simayi_guicai_rewrites_with_any_suit_card() {
    let mut e = Engine::new(207, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(2, "鬼才");
    // 鬼才的关键差异：红桃手牌（鬼道不可用）也进入选项
    let heart_card = loop {
        let c = e.debug_place(2, "shan");
        if e.catalog().cards[c as usize].suit == Suit::Heart {
            break c;
        }
    };
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    let le = e.debug_place(0, "le");
    play(&mut e, le, vec![1]);
    loop {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(prompt_of(&e), "gui_dao_rewrite");
    assert_eq!(e.open_choice().unwrap().observer, 2);
    // 选项必须包含红桃牌（鬼才语义）
    let opts = e.open_choice().unwrap().spec.options.clone();
    assert!(
        opts.contains(&ChoiceOption::Card(heart_card)),
        "鬼才选项应含红桃手牌: {opts:?}"
    );
    let old_judge = judged_card_of(&e);
    let cid = e.open_choice().unwrap().id;
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(heart_card)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 改判后的判定牌生效（红桃 → 乐不生效不跳过出牌阶段——此处仅断言事件与守恒）
    assert!(log_has(&e, |l| matches!(l,
        LE::JudgementRewritten { player: 2, card, replaced }
        if *card == heart_card && *replaced == old_judge)));
    assert!(!e.debug_hand(2).contains(&old_judge), "鬼才不获得旧判定牌");
    assert!(
        e.state_core().discard.contains(&old_judge),
        "鬼才旧判定牌弃置；鬼道/天道获得路径另测"
    );
    assert_eq!(
        log_count(&e, |l| matches!(l, LE::JudgementFinalized { .. })),
        1
    );
    assert_zone_conservation(&e);
}

/// 反馈：受到伤害后从来源处取牌（手牌优先）；来源无牌/无来源不触发。
#[test]
fn simayi_fankui_steals_from_damage_source() {
    let mut e = Engine::new(431, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "反馈");

    // (1) 来源无牌（全场清空）→ 不触发
    e.debug_set_hp(1, 4);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(e.open_choice().is_none(), "来源无牌时反馈不应触发");

    // (2) 来源有牌：0 号攻击 1 号（1 无闪）→ 反馈确认 → 取 0 号手牌
    // 伤害注入触发反馈链：0 号手牌恰一张（将被取走）
    let stolen = e.debug_place(0, "shan");
    let before1 = e.debug_hand(1).len();
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 可选触发确认询问挂起（观察者 1，闸门 trigger_confirm）
    let c = e.open_choice().expect("反馈询问应挂起");
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
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(e.debug_hand(1).contains(&stolen), "反馈应取得来源手牌");
    assert_eq!(e.debug_hand(1).len(), before1 + 1);
    assert!(log_has(&e, |l| matches!(l,
        LE::CardsGiven { from: 0, to: 1, cards } if cards.contains(&stolen))));
    assert_zone_conservation(&e);
}

/// 反馈放弃分支：空应答（allow_cancel）放弃取得。
#[test]
fn simayi_fankui_decline_takes_nothing() {
    let mut e = Engine::new(431, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "反馈");
    let kept = e.debug_place(0, "shan");
    let before1 = e.debug_hand(1).len();
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().expect("反馈询问应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(e.debug_hand(1).len() == before1, "放弃后不取得");
    assert!(e.debug_hand(0).contains(&kept), "来源手牌不动");
}
