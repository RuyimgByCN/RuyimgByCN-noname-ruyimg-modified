use super::*;

/// N08-c 反例一（仅装备区）：此前资格过滤只查手牌，仅装备黑武器者被跳过、
/// 不产生改判询问。现在询问到达且唯一牌选项来自装备区；出武器替换判定牌：
/// 武器离开装备区、随乐效果弃置，旧判定牌由改判者获得，Finalized 恰一次。
#[test]
fn guidao_rewrite_with_real_black_equip_only() {
    let (mut e, weapon) = setup_real_equip_rewrite();
    advance_to_rewrite_ask(&mut e);
    assert_eq!(e.open_choice().unwrap().observer, 2, "改判询问应到达 2 号");
    assert_eq!(
        rewrite_card_options(&e),
        vec![weapon],
        "唯一牌选项应为装备区黑武器"
    );
    let choice_id = e.open_choice().unwrap().id;
    let out = submit_cmd(
        &mut e,
        2,
        GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    // 判定牌被武器替换；旧判定牌由改判者获得（ADR 0714）
    let old_judge = judged_card_of_1(&e);
    assert!(e.log().iter().any(|l| matches!(l,
        LE::JudgementRewritten { player: 2, card, replaced }
        if *card == weapon && *replaced == old_judge)));
    assert!(e.debug_equips(2).is_empty(), "武器应离开装备区");
    assert!(
        e.debug_hand(2).contains(&old_judge),
        "旧判定牌应由改判者获得"
    );
    assert!(
        e.state_core().discard.contains(&weapon),
        "武器应随乐效果弃置"
    );
    // Finalized 恰一次、在 Rewritten 之后、牌面为最终判定牌
    let p_rw = e
        .log()
        .iter()
        .position(|l| matches!(l, LE::JudgementRewritten { .. }))
        .unwrap();
    let p_fin = e
        .log()
        .iter()
        .position(|l| matches!(l, LE::JudgementFinalized { .. }))
        .unwrap();
    assert!(p_rw < p_fin, "Rewritten 先于 Finalized");
    assert_eq!(
        e.log()
            .iter()
            .filter(|l| matches!(l, LE::JudgementFinalized { .. }))
            .count(),
        1,
        "Finalized 恰好一次"
    );
    assert!(e.log().iter().any(|l| matches!(l,
        LE::JudgementFinalized { player: 1, card } if *card == weapon)));
    assert_equip_window_conservation(&e);
}

/// N08-c 反例二（混合）：黑色手牌 + 黑色装备并存时选项须含两者
///（手牌在前、装备在后，与 kernel 选项构建一致）；出装备牌改判后手牌保留。
#[test]
fn guidao_rewrite_mixed_hand_and_equip_options() {
    let (mut e, weapon) = setup_real_equip_rewrite();
    let sha = place_black_sha(&mut e, 2);
    advance_to_rewrite_ask(&mut e);
    assert_eq!(e.open_choice().unwrap().observer, 2);
    assert_eq!(
        rewrite_card_options(&e),
        vec![sha, weapon],
        "选项应为手牌黑杀在前、装备区武器在后"
    );
    let choice_id = e.open_choice().unwrap().id;
    let out = submit_cmd(
        &mut e,
        2,
        GameAction::Answer {
            choice_id,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert!(e.debug_hand(2).contains(&sha), "未选的黑杀应留在手牌");
    assert!(e.debug_equips(2).is_empty(), "武器应离开装备区");
    assert!(
        e.state_core().discard.contains(&weapon),
        "武器应随乐效果弃置"
    );
    assert_equip_window_conservation(&e);
}
