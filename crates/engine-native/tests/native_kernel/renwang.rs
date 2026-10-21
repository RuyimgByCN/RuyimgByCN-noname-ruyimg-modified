use super::*;

/// P2/N08 批③-a（规则 v64，2026-10-07）：仁王盾（♣2，equip2 防具）——
/// 锁定：黑色【杀】对你无效（先于八卦阵判定；红杀照常进入出闪流程）。
/// 数量对齐见声明表（114 张）。
#[test]
fn renwang_blocks_black_sha_only() {
    // 黑杀：仁王盾免疫（无伤害、无出闪询问、无八卦判定）。
    let mut e = Engine::new(140, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let renwang = e.debug_place(1, "renwang");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != renwang);
        snap.core.players[1].equips.push(renwang);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    e.debug_set_deck_top_by(|d| d.suit.is_spade() || d.suit.is_club());
    let black_sha = {
        let snap = e.save();
        let found = {
            let cat = e.catalog();
            snap.core
                .deck
                .iter()
                .copied()
                .find(|c| {
                    matches!(cat.cards[*c as usize].kind, engine_native::CardKind::Sha)
                        && matches!(
                            cat.cards[*c as usize].suit,
                            engine_native::catalog::Suit::Spade
                                | engine_native::catalog::Suit::Club
                        )
                })
                .expect("黑杀在牌堆")
        };
        let mut snap = snap;
        snap.core.deck.retain(|c| *c != found);
        snap.core.players[0].hand.push(found);
        e = Engine::restore(&snap).expect("注入恢复");
        found
    };
    let hp1 = e.debug_hp(1);
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black_sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "黑杀被仁王盾无效后直接结算完毕: {out:?}"
    );
    assert!(e.open_choice().is_none(), "无出闪询问（仁王盾直接无效）");
    assert_eq!(e.debug_hp(1), hp1, "黑杀不掉血");

    // 红杀：仁王盾不生效——照常进入出闪/伤害流程（置红判定牌避开八卦）。
    // 推进一个整回合再打（每出牌阶段限一张杀）。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 0 {
            left_play = true;
        }
        match e.open_choice() {
            Some(c) => {
                let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else if !c.spec.options.is_empty() {
                    vec![c.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => {
                let actor = e.turn().current;
                match submit_cmd(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    e.debug_set_deck_top_by(|d| {
        matches!(
            d.suit,
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
        )
    });
    let red_sha = {
        let snap = e.save();
        let found = {
            let cat = e.catalog();
            snap.core
                .deck
                .iter()
                .copied()
                .find(|c| {
                    matches!(cat.cards[*c as usize].kind, engine_native::CardKind::Sha)
                        && matches!(
                            cat.cards[*c as usize].suit,
                            engine_native::catalog::Suit::Heart
                                | engine_native::catalog::Suit::Diamond
                        )
                })
                .expect("红杀在牌堆")
        };
        let mut snap = snap;
        snap.core.deck.retain(|c| *c != found);
        snap.core.players[0].hand.push(found);
        e = Engine::restore(&snap).expect("注入恢复");
        found
    };
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red_sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    // 排空应答（红杀照常走闪避/伤害流程；1 号无闪 → 受伤）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) => {
                if c.spec.options.is_empty() {
                    let _ = c;
                    break;
                }
                let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else {
                    vec![c.spec.options[0]]
                };
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => break,
        }
    }
    assert_eq!(e.debug_hp(1), hp1 - 1, "红杀不受仁王盾影响");
}
