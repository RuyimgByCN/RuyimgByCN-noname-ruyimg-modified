use super::*;

// ---------------------------------------------------------------------------
// 官方武将迁移：吕布（无双）/ 黄月英（集智）（ADR 0744）
// ---------------------------------------------------------------------------

/// 无双：来源持有时杀需连续两张闪——一张闪后仍需第二张，
/// 两张齐出才免除伤害；仅一张闪则受伤。
#[test]
fn lvbu_wushuang_sha_needs_two_dodges() {
    let mut e = Engine::new(509, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "无双");
    // 0 号出杀攻 1 号；1 号持两张闪（无双：需连续两张）
    let _shan1 = e.debug_place(1, "shan");
    let _shan2 = e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]); // 出杀询问 1 出闪
    let c = e.open_choice().expect("第一张闪询问");
    assert_eq!(c.spec.prompt, "respond_shan");
    assert_eq!(c.observer, 1);
    let shan1 = e.debug_hand(1)[0];
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "无双下第一张闪后应继续询问: {out:?}"
    );
    // 第二张闪
    let c = e.open_choice().expect("第二张闪询问");
    assert_eq!(c.spec.prompt, "respond_shan");
    let shan2 = e
        .debug_hand(1)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == CardKind::Shan)
        .expect("第二张闪");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(shan2)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 1), 4, "两张闪应完全抵消伤害");
    assert_eq!(discard_count(&e, shan1) + discard_count(&e, shan2), 2);
    assert_zone_conservation(&e);
}

/// 无双决斗部分：与无双持有者决斗的角色每次需连续打出两张杀——
/// 0 号（无双）与 2 号决斗：2 打两张杀后 0 无杀受伤，决斗结束。
#[test]
fn lvbu_wushuang_duel_needs_double_sha() {
    let mut e = Engine::new(523, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "无双");
    let duel = e.debug_place(0, "duel");
    let _s1 = e.debug_place(2, "sha");
    let _s2 = e.debug_place(2, "sha");
    play(&mut e, duel, vec![2]);
    // 决斗询问序列：2 连续两张杀 → 0 无杀受伤 → 决斗结束
    let c = e.open_choice().expect("决斗询问应挂起");
    assert_eq!(c.spec.prompt, "duel_sha");
    assert_eq!(c.observer, 2);
    let s1 = e.debug_hand(2)[0];
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(s1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "无双下应继续询问第二张杀: {out:?}"
    );
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "duel_sha");
    let s2 = e
        .debug_hand(2)
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) == CardKind::Sha)
        .expect("第二张杀");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(s2)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. })
            || matches!(out, CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 终态：2 打出两张杀后 0 无杀 → 0 受 1 点伤害，决斗结束
    assert_eq!(hp(&e, 2), 4, "2 号两张杀均抵消，不受伤害");
    assert_eq!(hp(&e, 0), 3, "0 号无杀应受 1 点伤害");
    assert_eq!(
        discard_count(&e, s1) + discard_count(&e, s2),
        2,
        "两张杀均弃置"
    );
    assert_eq!(discard_count(&e, duel), 1, "决斗牌弃置恰一份");
    assert_zone_conservation(&e);
}
