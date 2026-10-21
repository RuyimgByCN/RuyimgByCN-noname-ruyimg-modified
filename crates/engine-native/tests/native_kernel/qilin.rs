use super::*;

/// P2/N08 批③-b 第二把（规则 v66，2026-10-07）：麒麟弓（♥5，武器射程 5）
/// ——杀造成伤害时来源可弃目标装备区一张坐骑牌（可选，可放弃）。
#[test]
fn qilin_bow_discards_target_horse_on_sha_damage() {
    // ① 弃马路径：1 号装 +1 马；0 号装麒麟弓 → 杀 1 → 询问弃马 → 弃。
    let mut e = Engine::new(160, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let qilin = e.debug_move_named_card_to_hand(0, "qilin");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: qilin,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备麒麟弓: {out:?}"
    );
    let horse = e.debug_move_named_card_to_hand(1, "horse_plus");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != horse);
        snap.core.players[1].equips.push(horse);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    let sha = e.debug_place(0, "sha");
    let (dist, rng) = (
        e.state_core().computed_distance(e.catalog(), 0, 1),
        e.state_core().attack_range(e.catalog(), 0),
    );
    eprintln!(
        "PROBE-QL pre dist={dist} range={rng} hand0_has_sha={} alive1={}",
        e.debug_hand(0).contains(&sha),
        e.debug_alive(1)
    );
    let sha_out = play(&mut e, sha, vec![1]);
    eprintln!(
        "PROBE-QL sha_out={sha_out:?} eq0={:?} eq1={:?} hp1={}",
        e.debug_equips(0),
        e.debug_equips(1),
        e.debug_hp(1)
    );
    let ask = e.open_choice().expect("麒麟弓弃马询问应挂起");
    assert_eq!((ask.spec.prompt, ask.observer), ("qilin_discard_horse", 0));
    assert!(ask.spec.options.contains(&ChoiceOption::Card(horse)));
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(horse)],
        },
    );
    assert!(
        !e.save().core.players[1].equips.contains(&horse),
        "马已弃置"
    );
    assert!(e.save().core.discard.contains(&horse), "马入弃牌堆");
    assert_eq!(e.debug_hp(1), 3, "伤害照常");

    // ② 放弃路径：下回合再杀（1 号再装一匹 -1 马）→ 放弃 → 马保留。
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
    let horse2 = e.debug_move_named_card_to_hand(1, "horse_minus");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != horse2);
        snap.core.players[1].equips.push(horse2);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![1]);
    let ask2 = e.open_choice().expect("麒麟弓弃马询问（放弃路径）");
    assert!(ask2.spec.options.contains(&ChoiceOption::Card(horse2)));
    let (aid2, aobs2) = (ask2.id, ask2.observer);
    submit_cmd(
        &mut e,
        aobs2,
        GameAction::Answer {
            choice_id: aid2,
            selection: vec![],
        },
    );
    assert!(
        e.save().core.players[1].equips.contains(&horse2),
        "放弃：马保留"
    );
    assert_eq!(e.debug_hp(1), 2, "放弃：伤害照常");
}
