use super::*;

// ---------------------------------------------------------------------------
// 弃牌阶段：超过体力值需弃牌
// ---------------------------------------------------------------------------

#[test]
fn discard_phase_enforces_hand_limit() {
    let mut e = Engine::new(91, 4);
    // 0 号清空后放 4 张 + 摸 2 = 6 张；把体力压到 2 → 弃 4
    e.debug_clear_hand(0);
    for _ in 0..4 {
        e.debug_place(0, "shan");
    }
    e.debug_set_hp(0, 2);
    advance_to_play(&mut e, 0);
    // 从出牌阶段直接推到弃牌阶段
    let mut discarded = 0usize;
    for _ in 0..4 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                assert_eq!(choice.spec.prompt, "discard_to_limit");
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(choice.spec.min)
                    .collect();
                discarded += sel.len();
                let _ = answer_sel(&mut e, sel);
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.debug_hand(0).len(), 2, "弃牌后手牌应等于体力值");
    assert_eq!(discarded, 4);
}
