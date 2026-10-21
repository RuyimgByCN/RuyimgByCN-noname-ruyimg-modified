use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：大乔（国色）——view-as 延时锦囊（ADR 0751）
// ---------------------------------------------------------------------------

/// 国色：方块手牌当乐不思蜀使用——置入判定区、判定生效跳过出牌阶段。
#[test]
fn daqiao_guose_diamond_as_le() {
    let mut e = Engine::new(597, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "国色");
    // 找一张方块手牌（国色仅方块可转化）
    let diamond = loop {
        let c = e.debug_place(0, "shan");
        if e.catalog().cards[c as usize].suit == Suit::Diamond {
            break c;
        }
    };
    e.debug_set_deck_top_by(|def| def.suit.is_spade() && !(2..=9).contains(&def.rank));
    // 国色转化使用：方块牌当乐对 1 号使用
    let out = play(&mut e, diamond, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 置入 1 号判定区
    assert_eq!(e.debug_judge(1), vec![diamond], "乐应置入 1 号判定区");
    // 推进到 1 号的判定阶段：判定牌（黑桃）非红桃 → 跳过出牌阶段
    // （v67 牌堆位移：途中可能出现弃牌等询问，逐个应答排空）。
    loop {
        if e.turn().current == 1 && (e.turn().phase == Phase::Draw || e.turn().phase == Phase::End)
        {
            break;
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
            None => match next_phase(&mut e) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("{other:?}"),
            },
        }
    }
    assert!(e.debug_judge(1).is_empty(), "判定完成后乐应离场");
    // 国色声明牌种生效：黑桃判定 → 乐生效跳过出牌阶段
    assert!(
        e.state_core().turn.skip_play_phase,
        "黑桃判定应跳过出牌阶段"
    );
    assert_zone_conservation(&e);
}
