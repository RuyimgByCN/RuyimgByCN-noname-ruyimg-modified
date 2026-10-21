use super::*;

// ---------------------------------------------------------------------------
// 决斗（阶段 2 扩面第四项）
// ---------------------------------------------------------------------------

/// 交替出杀：目标先出 → 来源出 → 目标 Pass → 目标受伤；无距离限制（0→2）。
#[test]
fn duel_alternating_sha_until_pass() {
    let mut e = Engine::new(141, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let sha_t1 = e.debug_place(2, "sha");
    let sha_s = e.debug_place(0, "sha");
    let _sha_t2 = e.debug_place(2, "sha");
    assert!(
        matches!(play(&mut e, duel, vec![2]), CommandOutcome::Pending { .. }),
        "决斗无距离限制，0→2 应可发动"
    );
    // 目标先出杀
    let out = answer(&mut e, ChoiceOption::Card(sha_t1));
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(prompt_of(&e), "duel_sha");
    assert_eq!(e.open_choice().unwrap().observer, 0, "轮到来源");
    // 来源出杀 → 再次轮到目标（仍有杀）→ 目标 Pass → 目标受伤
    let out = answer(&mut e, ChoiceOption::Card(sha_s));
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert_eq!(e.open_choice().unwrap().observer, 2, "第二轮轮到目标");
    let out = answer(&mut e, ChoiceOption::Pass);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 0), 4);
    // 已出的响应杀均入弃牌堆
    assert!(!e.debug_hand(2).contains(&sha_t1));
    assert!(!e.debug_hand(0).contains(&sha_s));
    // 伤害来源 = **对方**（上游 `await event.turn.damage(event.source)`，
    // card/standard.js:2227；轮转中 [source, turn] 交换 → 失败方受伤、来源为
    // 对手，与「最后出杀者」在交替制下等价）。断言来源字段而非仅看体力，
    // 以冻结「来源归属不受最后出杀者影响」。
    assert_eq!(
        count_log(&e, |l| matches!(
            l,
            LE::DamageDealt {
                source: Some(0),
                target: 2,
                amount: 1,
                ..
            }
        )),
        1,
        "决斗伤害来源应为对手 0"
    );
    assert_eq!(
        count_log(&e, |l| matches!(
            l,
            LE::DamageDealt {
                source: Some(2),
                ..
            }
        )),
        0,
        "目标不应成为伤害来源"
    );
}

/// 轮到方无杀：不询问直接受伤（0 有杀也不被询问）。
#[test]
fn duel_no_sha_immediate_loss() {
    let mut e = Engine::new(143, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let sha = e.debug_place(0, "sha");
    let asks_before = count_log(&e, |l| matches!(l, LE::ChoiceOpened { .. }));
    let out = play(&mut e, duel, vec![1]);
    // 1 无杀 → 直接受伤，决斗完结（不产生任何询问）
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 1), 3);
    assert_eq!(hp(&e, 0), 4);
    assert!(e.debug_hand(0).contains(&sha), "来源的杀不应被消耗");
    // 上游 `juedou` 同一 autochoose 口径（card/standard.js:2178 起）→ 零询问
    assert_eq!(
        count_log(&e, |l| matches!(l, LE::ChoiceOpened { .. })) - asks_before,
        0,
        "轮到方无杀时不发起询问（对齐上游 autochoose）"
    );
}

/// 决斗不消耗出牌阶段限一次杀。
#[test]
fn duel_does_not_consume_sha_limit() {
    let mut e = Engine::new(145, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    play(&mut e, duel, vec![1]);
    let sha = e.debug_place(0, "sha");
    let out = play(&mut e, sha, vec![1]);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
}

/// 决斗伤害可触发濒死死亡。
#[test]
fn duel_damage_can_kill() {
    let mut e = Engine::new(147, 4);
    e.debug_set_hp(1, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    assert!(matches!(
        play(&mut e, duel, vec![1]),
        CommandOutcome::Accepted { .. }
    ));
    assert!(!e.debug_alive(1));
    assert!(log_has(&e, |l| matches!(
        l,
        LE::PlayerDied { player: 1, .. }
    )));
}

/// 决斗可被无懈抵消：双方无伤、决斗牌弃置。
#[test]
fn duel_can_be_nullified() {
    let mut e = Engine::new(157, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let nul = e.debug_place(1, "nullify");
    play(&mut e, duel, vec![1]);
    assert_eq!(prompt_of(&e), "nullify_window");
    let out = answer(&mut e, ChoiceOption::Card(nul));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 0), 4);
    assert_eq!(hp(&e, 1), 4);
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));
}
