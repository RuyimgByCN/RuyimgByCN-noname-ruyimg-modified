use super::*;

// ---------------------------------------------------------------------------
// N05：负体力连续救援（濒死询问轮转，ADR 0726）
// ---------------------------------------------------------------------------

/// N05 机制：1 血吃雷（1→-2 负体力），同一持桃者跨三轮询问连用三桃救回
/// （-2→-1→0→1）。旧实现响应者应答一次即出队，负体力无法自救。
#[test]
fn negative_hp_rescued_by_three_taos() {
    let mut e = Engine::new(241, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let _sd = e.debug_place_judge(1, "shandian");
    let taos: Vec<u32> = (0..3).map(|_| e.debug_place(1, "tao")).collect();
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && (2..=9).contains(&def.rank));
    // 推进到 1 的判定阶段：雷击 3 伤（1→-2 濒死），首个救援询问挂起
    loop {
        if e.open_choice().is_some() {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.debug_hp(1), -2, "雷击后应为负体力 -2");
    assert_eq!(prompt_of(&e), "dying_save");
    assert_eq!(e.open_choice().unwrap().observer, 1, "濒死者先自救");
    // 连续三桃：-2 → -1 → 0 → 1（轮转询问把持桃者排回队尾）
    for i in 0..3 {
        let c = e.open_choice().expect("救援询问应挂起");
        assert_eq!(c.spec.prompt, "dying_save");
        assert_eq!(c.observer, 1, "第 {} 轮仍应由持桃者自救", i + 1);
        let tao = c
            .spec
            .options
            .iter()
            .find_map(|o| match o {
                ChoiceOption::Card(id) if taos.contains(id) => Some(*id),
                _ => None,
            })
            .expect("询问应包含剩余的桃");
        let cid = c.id;
        let out = e.submit(GameCommand {
            actor: 1,
            expected_revision: e.revision(),
            dedup_id: fresh_dedup(),
            action: GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Card(tao)],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        if i < 2 {
            assert!(e.open_choice().is_some(), "仍处濒死应继续询问下一轮");
        }
    }
    assert!(e.debug_alive(1), "三桃后应脱离濒死");
    assert_eq!(e.debug_hp(1), 1, "-2 + 3 桃 = 1 体力");
    assert_eq!(log_count(&e, |l| matches!(l, LE::DyingSaved { .. })), 3);
    assert!(!log_has(&e, |l| matches!(
        l,
        LE::PlayerDied { player: 1, .. }
    )));
    assert!(e.debug_judge(1).is_empty(), "闪电已结算离场");
    assert_zone_conservation(&e);
}
