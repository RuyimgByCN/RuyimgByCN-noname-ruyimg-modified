use super::*;

/// P2/N08 缺牌种批①-2（规则 v63，2026-10-07）：五谷丰登（♥3/♥4，
/// 亮存活角色数的牌、从使用者起按座位序逐人择一）与借刀杀人（♣12/♣13，
/// 武器持有者出杀或交械；登记差异：出杀为直接伤害结算，无出闪窗口）。
#[test]
fn wugu_jiedao_trick_behaviors() {
    // ① 五谷：亮 3 张（三人局）→ 0/1/2 按座位序各择一张。
    let mut e = Engine::new(130, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let wg = e.debug_place(0, "wugu");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wg,
            targets: vec![],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    // 无懈窗口排空。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    let hands: Vec<usize> = (0..3).map(|p| e.debug_hand(p).len()).collect();
    let mut first_picks: Vec<u32> = Vec::new();
    for picker in 0..3u8 {
        let ask = e.open_choice().expect("五谷择牌询问");
        assert_eq!((ask.spec.prompt, ask.observer), ("wugu_pick", picker));
        assert_eq!(ask.spec.options.len(), 3 - picker as usize, "剩余亮牌递减");
        let chosen = ask.spec.options[0];
        if picker == 0 {
            first_picks = ask
                .spec
                .options
                .iter()
                .filter_map(|o| match o {
                    ChoiceOption::Card(c) => Some(*c),
                    _ => None,
                })
                .collect();
        }
        let (id2, obs2) = (ask.id, ask.observer);
        submit_cmd(
            &mut e,
            obs2,
            GameAction::Answer {
                choice_id: id2,
                selection: vec![chosen],
            },
        );
    }
    for p in 0..3 {
        assert_eq!(
            e.debug_hand(p).len(),
            hands[p as usize] + 1,
            "{p} 号各得一张"
        );
    }
    assert!(first_picks.len() == 3);

    // ② 借刀（出杀路径）：1 号装武器 → 0 号借刀 [1,2] → 1 号选出杀 →
    // 2 号受 1 伤（登记差异：无出闪窗口）。
    let mut e = Engine::new(131, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 武器布阵走注入（1 号非当前回合不能出牌）。
    let weapon = e.debug_place(1, "weapon2");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != weapon);
        snap.core.players[1].equips.push(weapon);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    let jd = e.debug_place(0, "jiedao");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jd,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    let ask = e.open_choice().expect("借刀选择应挂起");
    assert_eq!((ask.spec.prompt, ask.observer), ("jiedao_sha_or_give", 1));
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert_eq!(
        e.debug_hp(2),
        3,
        "出杀路径：2 号受 1 伤（无出闪窗口，登记差异）"
    );
    assert_eq!(e.debug_hp(1), 4, "持有者不掉血");
    assert!(
        e.save().core.players[1].equips.contains(&weapon),
        "武器仍在"
    );

    // ③ 借刀（交械路径）：下一回合 0 号再借 → 1 号选交武器 → 武器转移到 2 号。
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
    let jd2 = e.debug_place(0, "jiedao");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jd2,
            targets: vec![1, 2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Pending { .. } | CommandOutcome::Accepted { .. }
        ),
        "{out:?}"
    );
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) if c.spec.prompt == "nullify_window" => {
                let (id2, obs2) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            _ => break,
        }
    }
    let ask2 = e.open_choice().expect("借刀选择（交械路径）");
    let (aid2, aobs2) = (ask2.id, ask2.observer);
    submit_cmd(
        &mut e,
        aobs2,
        GameAction::Answer {
            choice_id: aid2,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert!(
        !e.save().core.players[1].equips.contains(&weapon),
        "1 号交出武器"
    );
    assert!(
        e.save().core.players[2].equips.contains(&weapon),
        "2 号获得武器"
    );
    assert_eq!(e.debug_hp(2), 3, "交械路径不造成伤害");
}
