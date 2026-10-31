use super::*;

// ---------------------------------------------------------------------------
// 濒死救援：座位顺序持桃询问 / 自救 / 无人救则死亡
// ---------------------------------------------------------------------------

#[test]
fn dying_rescue_and_death() {
    // 自救：1/2/3 无桃自动跳过，0 用桃自救
    let mut e = Engine::new(41, 4);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let tao = e.debug_place(0, "tao");
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    for p in 2..4 {
        e.debug_clear_hand(p);
    }
    play(&mut e, sha, vec![0]);
    assert_eq!(prompt_of(&e), "dying_save");
    let out = answer(&mut e, ChoiceOption::Card(tao));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 0), 1);
    assert!(e.debug_alive(0));
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::DyingSaved { player: 0, .. }
    )));

    // 他人救援：1 持桃先放弃，2 无桃跳过，3 用桃救
    let mut e = Engine::new(43, 4);
    e.debug_set_hp(0, 1);
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(1, "sha");
    e.debug_place(1, "tao");
    let tao3 = e.debug_place(3, "tao");
    play(&mut e, sha, vec![0]);
    assert_eq!(prompt_of(&e), "dying_save");
    // 1 放弃后立即轮到下一位持桃者：仍处挂起（Pending）
    let out = answer(&mut e, ChoiceOption::Pass);
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "pass should chain to next responder: {out:?}"
    );
    // 2 无桃 → 自动跳过；3 有桃 → 被询问
    assert_eq!(prompt_of(&e), "dying_save");
    let out = answer(&mut e, ChoiceOption::Card(tao3));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(e.debug_alive(0));

    // 无人救 → 死亡：手牌进弃牌堆、技能移除、存活数下降
    let mut e = Engine::new(45, 4);
    e.debug_set_hp(0, 1);
    e.debug_grant(0, "散财"); // 死亡时应被移除（防止型技能会干扰致死，故选被动型）
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(1, "sha");
    let discard_before = e.debug_discard_len();
    let dead_hand = e.debug_hand(0).len();
    play(&mut e, sha, vec![0]);
    // 散财的真实 targetDamage admission 先于 Dying，拒绝后无人持桃才死亡。
    if prompt_of(&e) == "trigger_confirm" {
        let c = e.open_choice().unwrap().clone();
        submit_cmd(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![],
            },
        );
    }
    assert!(!e.debug_alive(0), "pending={:?}", e.open_choice());
    assert!(!e.debug_has_skill(0, "散财"));
    assert_eq!(
        e.debug_discard_len(),
        discard_before + dead_hand + 1,
        "致死的杀应随结算入弃牌堆（手牌已清空）"
    );
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::PlayerDied { player: 0, .. }
    )));
    // 场上仍 3 人，未终局
    assert!(e.outcome().is_none());
}

/// 濒死酒选项的恢复派生：手牌只有酒时挂起濒死询问，字节往返后仍
/// 推导出相同选项（含酒）。
#[test]
fn dying_save_with_jiu_derivation_restores() {
    use engine_native::archive::{decode_archive, encode_archive};
    let mut e = Engine::new(3004, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    let jiu = e.debug_place(1, "jiu");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    let bytes = encode_archive(&e.save());
    let restored = Engine::restore(&decode_archive(&bytes).expect("decode")).expect("恢复");
    let c = restored.open_choice().expect("恢复件仍挂起濒死询问");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(jiu)),
        "恢复派生应含酒"
    );
}
