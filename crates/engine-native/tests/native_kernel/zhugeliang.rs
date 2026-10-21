use super::*;

// ---------------------------------------------------------------------------
// N08 第六十批：诸葛亮（观星/空城）+ 陆逊（谦逊/连营）（ADR 0759）
// ---------------------------------------------------------------------------

/// 观星（诸葛亮）：判定阶段确认后可重排牌库顶——所选按应答顺序置顶
/// （先选先摸），未选置底；询问挂起时可存档续行。
#[test]
fn zhugeliang_guanxing_reorders_deck_top() {
    let mut e = Engine::new(901, 3);
    e.debug_grant(0, "观星");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    // 走到 0 的下一回合：判定阶段开始时观星闸门挂起
    let mut shown: Vec<u32> = Vec::new();
    let mut e2_saved: Option<engine_native::snap::StateSnapshot> = None;
    for _ in 0..128 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => match choice.spec.prompt {
                "trigger_confirm" => {
                    assert_eq!(choice.observer, 0, "观星闸门只问持有者");
                    answer(&mut e, ChoiceOption::Confirm);
                    // 确认后落位询问应同步挂起（同一命令段内）
                    let arr = e.open_choice().expect("guanxing_arrange 应挂起");
                    assert_eq!(arr.spec.prompt, "guanxing_arrange");
                    shown = arr
                        .spec
                        .options
                        .iter()
                        .filter_map(|o| match o {
                            ChoiceOption::Card(c) => Some(*c),
                            _ => None,
                        })
                        .collect();
                    assert_eq!(shown.len(), 3, "存活数 3 → 观看 3 张");
                    assert_eq!(arr.spec.min, 0);
                    e2_saved = Some(e.save());
                    let sel = vec![ChoiceOption::Card(shown[1]), ChoiceOption::Card(shown[0])];
                    let ans_out = answer_sel(&mut e, sel);
                    assert!(
                        matches!(ans_out, CommandOutcome::Accepted { .. }),
                        "arrange answer: {ans_out:?}"
                    );
                    break;
                }
                other => panic!("unexpected prompt: {other}"),
            },
            o => panic!("walk failed: {o:?}"),
        }
    }
    let mut e2 = Engine::restore(&e2_saved.expect("guanxing ask snapshot")).expect("restore");
    assert_eq!(shown.len(), 3, "存活数 3 → 观看 3 张");
    // e2 续行：以相同选择应答其挂起的落位询问
    let e2_ask = e2.open_choice().expect("e2 落位询问应挂起");
    assert_eq!(e2_ask.spec.prompt, "guanxing_arrange");
    answer_sel(
        &mut e2,
        vec![ChoiceOption::Card(shown[1]), ChoiceOption::Card(shown[0])],
    );
    // 期望：牌库顶 = shown[1]（先选先摸）、次顶 = shown[0]；未选 shown[2] 置底。
    // 断言与段推进深度无关：直接推进到摸牌开始后核对入手顺序。
    for eng in [&mut e, &mut e2] {
        for _ in 0..16 {
            if eng.turn().phase == Phase::Draw {
                break;
            }
            match next_phase(eng) {
                CommandOutcome::Accepted { .. } => {}
                CommandOutcome::Pending { choice } => panic!("unexpected {}", choice.spec.prompt),
                o => panic!("walk: {o:?}"),
            }
        }
        assert_eq!(eng.turn().phase, Phase::Draw);
        let snap = eng.save();
        let deck = &snap.core.deck;
        let hand = &snap.core.players[0].hand;
        assert_eq!(deck[0], shown[2], "未选牌置于牌库底");
        assert_eq!(
            &hand[hand.len() - 2..],
            &[shown[1], shown[0]],
            "先选者先摸（摸牌立即兑现）"
        );
    }
    assert_eq!(e.state_hash(), e2.state_hash(), "观星存档续行终态一致");
}
/// 空城（诸葛亮）：无手牌时不能成为杀/决斗目标；有手牌或无技能时不受限。
#[test]
fn zhugeliang_kongcheng_blocks_sha_and_duel_targets() {
    let mut e = Engine::new(902, 3);
    advance_to_play(&mut e, 0);
    e.debug_grant(1, "空城");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    assert!(
        matches!(
            play(&mut e, sha, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "空城无手牌不得成为杀目标"
    );
    let duel = e.debug_place(0, "duel");
    assert!(
        matches!(
            play(&mut e, duel, vec![1]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "空城无手牌不得成为决斗目标"
    );
    // 有手牌 → 可成为目标
    let _shan = e.debug_place(1, "shan");
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![1]);
    assert!(
        !matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "空城有手牌不设限: {out:?}"
    );
    // 结算出闪询问（放弃出闪 → 承受 1 点伤害）
    if e.open_choice().map(|c| c.spec.prompt) == Some("respond_shan") {
        answer(&mut e, ChoiceOption::Pass);
        assert_eq!(e.debug_hp(1), 3, "有手牌时空城不挡杀");
    }
}
