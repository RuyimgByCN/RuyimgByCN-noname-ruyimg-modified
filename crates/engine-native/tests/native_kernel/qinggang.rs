use super::*;

/// P2/N08 批③-b 第一把（规则 v65，2026-10-07）：青釭剑（♠6，武器射程 2）
/// ——锁定无视目标防具：来源装备青釭时，八卦阵判定与仁王盾黑杀免疫
/// 均不生效。
#[test]
fn qinggang_pierces_armor() {
    // ① 无青釭基线：目标装备八卦阵（Armor），黑判定 → 不免费闪避 → 出闪询问。
    let mut e = Engine::new(150, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let armor = e.debug_place(1, "armor");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != armor);
        snap.core.players[1].equips.push(armor);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    e.debug_set_deck_top_by(|d| d.suit.is_spade() || d.suit.is_club()); // 黑判定：八卦不闪
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 排空应答（1 号无闪 → 八卦黑判定后直接受伤）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(g) => {
                if g.spec.options.is_empty() {
                    let _ = g;
                    break;
                }
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
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
    assert_eq!(e.debug_hp(1), 3, "基线：黑判定后受伤");
    assert!(
        log_has(&e, |l| matches!(l, LE::ArmorJudged { .. })),
        "基线：八卦判定已发生"
    );

    // ② 青釭：0 号装备青釭剑 → 杀同目标：无八卦判定（无 ArmorJudged 日志）、
    //    直接出闪询问；黑判定牌留 top 不被消耗。
    // 推进一个整回合（每出牌阶段限一张杀）。
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
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else if !g.spec.options.is_empty() {
                    vec![g.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (g.id, g.observer);
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
    e.debug_move_named_card_to_hand(0, "qinggang");
    let qg = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "qinggang")
        .expect("青釭在目录") as u32;
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qg,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备青釭: {out:?}"
    );
    e.debug_set_deck_top_by(|d| d.suit.is_spade() || d.suit.is_club());
    let top_before = e.state_core().deck.last().copied();
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![1]);
    // 无闪：青釭下无八卦判定 → 直接受伤（判定牌留 top 未消耗）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(g) => {
                if g.spec.options.is_empty() {
                    let _ = g;
                    break;
                }
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
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
    let top_after = e.state_core().deck.last().copied();
    assert_eq!(
        top_before, top_after,
        "青釭：八卦判定未发生（判定牌未被消耗）"
    );
    assert_eq!(e.debug_hp(1), 2, "青釭穿透：黑杀照常伤害");

    // 推进一个整回合（每出牌阶段限一张杀）。
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
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else if !g.spec.options.is_empty() {
                    vec![g.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (g.id, g.observer);
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
    // ③ 青釭 vs 仁王盾：2 号装仁王盾，0 号换杀目标 → 黑杀穿透仁王盾。
    e.debug_move_named_card_to_hand(2, "renwang");
    let rw = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "renwang")
        .expect("仁王盾在目录") as u32;
    {
        let mut snap = e.save();
        snap.core.players[2].hand.retain(|c| *c != rw);
        snap.core.players[2].equips.push(rw);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    e.debug_set_deck_top_by(|d| d.suit.is_spade() || d.suit.is_club());
    let sha3 = e.debug_place(0, "sha");
    play(&mut e, sha3, vec![2]);
    // 排空应答（无仁王盾免疫 → 正常出闪/伤害流程；2 号无闪 → 直接受伤）。
    for _ in 0..8 {
        match e.open_choice() {
            Some(g) => {
                if g.spec.options.is_empty() {
                    let _ = g;
                    break;
                }
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else {
                    vec![g.spec.options[0]]
                };
                let (id2, obs2) = (g.id, g.observer);
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
    assert_eq!(e.debug_hp(2), 3, "青釭穿透仁王盾：黑杀照常伤害");
}
