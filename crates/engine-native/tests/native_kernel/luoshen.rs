use super::*;

/// 洛神：判定阶段开始时循环判定——黑色获得并再判，红色结束。
/// seed 固定 → 获得张数确定（>=1，脚本内验证循环终止与守恒）。
#[test]
fn luoshen_repeat_judgement_gains_black_cards() {
    let mut e = Engine::new(487, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "洛神");
    // 牌堆构成变更（v62）后种子敏感：注入黑色判定首张去随机化
    //（牌堆顶=末尾）。
    {
        let snap = e.save();
        let black = {
            let cat = e.catalog();
            snap.core
                .deck
                .iter()
                .copied()
                .find(|c| {
                    matches!(
                        cat.cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                    )
                })
                .expect("黑色牌在牌堆")
        };
        let mut snap = snap;
        snap.core.deck.retain(|c| *c != black);
        snap.core.deck.push(black);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    let before = e.debug_hand(1).len();
    // 推进到 1 号判定阶段（洛神触发确认）
    loop {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    let c = e.open_choice().expect("洛神触发确认应挂起");
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
    // 循环判定：至少获得一张黑色牌（判定牌推入手牌）
    let gained = e.debug_hand(1).len() - before;
    assert!(gained >= 1, "洛神至少获得一张黑色判定牌");
    // 全黑色牌数与日志一致；循环终止（当前无悬挂选择且阶段推进正常）
    assert!(e.open_choice().is_none());
    let black_draws = log_count(
        &e,
        |l| matches!(l, LE::CardsDrawn { player: 1, cards } if cards.len() == 1),
    );
    assert!(black_draws >= 1, "应有洛神获得日志");
    assert_zone_conservation(&e);
}
