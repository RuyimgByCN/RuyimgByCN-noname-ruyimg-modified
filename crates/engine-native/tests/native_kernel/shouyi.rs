use super::*;

// ---------------------------------------------------------------------------
// 动态技能获得与到期（授艺 → 移伤）
// ---------------------------------------------------------------------------

#[test]
fn shouyi_grants_temporary_yishang_until_owner_turn_end() {
    let mut e = Engine::new(61, 4);
    e.debug_grant(0, "授艺");
    let sha_b = e.debug_place(1, "sha");
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    play(&mut e, sha_b, vec![0]);
    assert_eq!(hp(&e, 0), 3);
    assert!(e.debug_has_skill(0, "移伤"), "受伤后应获得移伤");
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::SkillToggled {
            player: 0,
            skill: "移伤",
            enabled: true
        }
    )));

    // 同回合稍后：3 攻击 0，移伤可选转移
    advance_to_play(&mut e, 3);
    let sha_d = e.debug_place(3, "sha");
    e.debug_clear_hand(0);
    play(&mut e, sha_d, vec![0]);
    assert_eq!(prompt_of(&e), "trigger_confirm");
    answer(&mut e, ChoiceOption::Confirm);
    assert_eq!(prompt_of(&e), "yishang_target");
    answer(&mut e, ChoiceOption::Player(1));
    assert_eq!(hp(&e, 0), 3);
    assert_eq!(hp(&e, 1), 3);

    // 获得者的回合结束后到期
    advance_to_play(&mut e, 0);
    for _ in 0..5 {
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                let sel: Vec<ChoiceOption> = choice
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .take(choice.spec.min.max(1))
                    .collect();
                let _ = answer_sel(&mut e, sel);
            }
            other => panic!("{other:?}"),
        }
        if matches!(e.debug_has_skill(0, "移伤"), false) {
            break;
        }
    }
    assert!(!e.debug_has_skill(0, "移伤"), "回合结束后临时技能应到期");
}
