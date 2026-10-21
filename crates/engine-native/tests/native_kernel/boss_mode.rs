use super::*;

/// Boss 基线+重整复活回环（P2/N08 模式入口批三-a，规则 v82）：
/// 死亡玩家轮到回合时重整 tick（计数+回血）、达阈值复活；
/// boss 死亡挑战者胜、挑战者全灭 boss 胜；存档往返。
#[test]
fn boss_mode_chongzheng_revival_and_victory() {
    let mut e = Engine::new_boss(2201, 3, 0, 2);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // ① boss(0) 杀挑战者 1：未终局（2 号存活）；无奖惩（boss 手牌数不变）
    e.debug_set_hp(1, 1);
    let hand0 = e.debug_hand(0).len();
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(1));
    assert_eq!(e.debug_hand(0).len(), hand0, "Boss 模式无击杀奖惩");
    assert!(e.outcome().is_none(), "2 号存活，未终局");

    // ② 回合推进到 1 号的重整 tick（cnt=1、hp 0→1），然后 2 号接续回合
    for _ in 0..96 {
        let snap = e.save();
        if snap.core.players[1].boss_chongzheng >= 1 {
            break;
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
    let snap = e.save();
    assert_eq!(snap.core.players[1].boss_chongzheng, 1, "重整计数 +1");
    assert_eq!(snap.core.players[1].hp, 1, "重整回血 0→1");
    assert_eq!(e.turn().current, 2, "tick 后落在 2 号回合");
    assert!(!e.debug_alive(1), "未达阈值仍死亡");

    // ③ 存档往返：Boss 模式表一致
    {
        use engine_native::archive::{decode_archive, encode_archive};
        let bytes = encode_archive(&snap);
        let back = decode_archive(&bytes).expect("解码");
        assert_eq!(back.core.mode, snap.core.mode, "Boss 模式往返一致");
        assert_eq!(back.core.players[1].boss_chongzheng, 1, "重整计数往返一致");
    }

    // ④ 再过一轮 → 第二次 tick 达阈值复活
    for _ in 0..96 {
        if e.debug_alive(1) {
            break;
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
                    other => panic!("advance2: {other:?}"),
                }
            }
        }
    }
    assert!(e.debug_alive(1), "第二次重整 tick 达阈值复活");
    assert_eq!(
        e.save().core.players[1].boss_chongzheng,
        0,
        "复活后计数清零"
    );
    assert!(e.outcome().is_none());

    // ⑤ boss 死亡 → 挑战者胜
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(0));
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::Boss { boss_won: false }),
        "{:?}",
        e.outcome()
    );

    // ⑥ 反向：挑战者全灭 → boss 胜（新局，chongzheng=0 无复活）
    let mut e = Engine::new_boss(2103, 3, 0, 0);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    for seat in [1u8, 2] {
        e.debug_set_hp(seat, 1);
        let out = e.debug_deal_damage(Some(0), seat, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
        if seat == 1 {
            assert!(e.outcome().is_none(), "2 号存活，未终局");
        }
    }
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::Boss { boss_won: true }),
        "{:?}",
        e.outcome()
    );
}

/// addFellow 中途入队（v84 批三-b）：入队座位=队尾、直授 4 张、
/// 座位表扩张、fellows 登记、存档往返、fellow 不算挑战者。
/// Boss 批三-b（规则 v84）：助战中途入队——队尾追加座位、直授 4 牌、
/// fellows 登记；fellow 属 boss 方（不算挑战者）；存档往返含扩张后座位表。
#[test]
fn boss_mode_add_fellow_seat_expansion() {
    let mut e = Engine::new_boss(2205, 3, 0, 2);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // ① 先杀一名挑战者（2 号）：未终局
    e.debug_set_hp(2, 1);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(2));
    assert!(e.outcome().is_none(), "1 号存活，未终局");

    // ② 助战入队：座位 3（队尾）、4 张手牌、alive；挑战者 1 仍存活不终局
    let fellow = e.debug_add_fellow(4).expect("Boss 模式应可入队");
    assert_eq!(fellow, 3, "入队座位=队尾");
    assert!(e.debug_alive(3));
    assert_eq!(e.debug_hand(3).len(), 4, "直授 4 张手牌");
    assert_eq!(e.save().core.players.len(), 4, "座位表扩张");
    assert_eq!(e.save().core.boss_fellows().unwrap(), &[3], "fellows 登记");
    assert!(e.outcome().is_none());

    // ③ 存档往返：扩张后座位表 + fellows 一致
    {
        use engine_native::archive::{decode_archive, encode_archive};
        let bytes = encode_archive(&e.save());
        let back = decode_archive(&bytes).expect("解码");
        assert_eq!(back.core.players.len(), 4, "扩张座位表往返");
        assert_eq!(back.core.mode, e.save().core.mode, "fellows 往返一致");
        assert_eq!(
            back.core.players[3].hand,
            e.save().core.players[3].hand,
            "助歌手牌往返"
        );
    }

    // ④ 再杀挑战者 1 → 挑战者全灭（fellow 3 属 boss 方不算）→ boss 胜
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 1);
    let out = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 助战直授牌可能含桃 → 濒死救援询问（强制应答）：Pass 放弃救援
    if let Some(c) = e.open_choice() {
        assert_eq!(c.spec.prompt, "dying_save");
        let (cid, obs) = (c.id, c.observer);
        submit_cmd(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert!(!e.debug_alive(1));
    assert!(e.debug_alive(3), "助战存活");
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::Boss { boss_won: true }),
        "{:?}",
        e.outcome()
    );
}
