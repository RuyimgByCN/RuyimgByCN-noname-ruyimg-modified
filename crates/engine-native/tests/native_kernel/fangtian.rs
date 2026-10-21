use super::*;

/// P2/N08 批③-b⑨（规则 v73，2026-10-07）：方天画戟（♦5，武器射程 4）
/// ——该杀为最后手牌时可指定至多 3 个目标（逐目标独立结算：各自出闪/
/// 伤害；登记差异：多目标逐个推进为顺序结算）。
#[test]
fn fangtian_multi_target_sha() {
    // 基线：非最后手牌的杀多目标 → 拒绝。
    let mut e = Engine::new(240, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let ft = e.debug_move_named_card_to_hand(0, "fangtian");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ft,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备方天: {out:?}"
    );
    let keep = e.debug_place(0, "tao"); // 留一张非杀手牌 → 不满足「最后手牌」
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1, 2, 3]);
    assert!(
        matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "非最后手牌多目标应拒绝: {out:?}"
    );

    // 方天路径：弃掉留牌 → 杀为最后手牌 → 三目标合法（4 人局射程内全达）。
    {
        let mut snap = e.save();
        // keep 仍在手牌：直接弃入弃牌堆（合法单区）。
        assert!(snap.core.players[0].hand.contains(&keep));
        snap.core.players[0].hand.retain(|c| *c != keep);
        snap.core.discard.push(keep);
        e = Engine::restore(&snap).expect("弃留牌注入恢复");
        let _ = keep;
    }
    // 直接以快照层清空 0 号手牌并注入杀（确保为最后手牌）。
    {
        let mut snap = e.save();
        snap.core.players[0].hand.clear();
        snap.core.players[0].hand.push(sha);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    let out = play(&mut e, sha, vec![1, 2, 3]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "方天三目标接受: {out:?}"
    );
    // 三目标各无闪 → 各受 1 伤。
    for _ in 0..24 {
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
    assert_eq!(e.debug_hp(1), 3, "目标 1 受 1 伤");
    assert_eq!(e.debug_hp(2), 3, "目标 2 受 1 伤");
    assert_eq!(e.debug_hp(3), 3, "目标 3 受 1 伤");

    // 两目标变体：无方天时普通杀仍单目标（既有口径由其余测试覆盖），
    // 方天下两目标也合法。
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
    {
        let mut snap = e.save();
        snap.core.discard.retain(|c| *c != sha); // 首轮已弃：移出弃牌堆避免双区
                                                 // 手牌清空前先归入弃牌堆（牌归属恰一区约束）。
        let drained: Vec<u32> = snap.core.players[0].hand.drain(..).collect();
        snap.core.discard.extend(drained);
        snap.core.players[0].hand.push(sha); // 方天条件：杀为最后手牌
        e = Engine::restore(&snap).expect("注入恢复");
    }
    eprintln!(
        "PROBE-FT pre: hand0={:?} alive={:?} eq0={:?}",
        e.debug_hand(0),
        (0..4).map(|p| e.debug_alive(p)).collect::<Vec<_>>(),
        e.debug_equips(0)
    );
    let out = play(&mut e, sha, vec![1, 2]);
    eprintln!("PROBE-FT out={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "方天两目标合法: {out:?}"
    );
}
