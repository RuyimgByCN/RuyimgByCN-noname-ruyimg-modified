use super::*;

/// 青囊：出牌阶段限一次，弃一张手牌令一名已受伤的其他角色回复 1 点体力。
#[test]
fn huatuo_qingnang_heals_wounded_player() {
    let mut e = Engine::new(583, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "青囊");
    e.debug_set_hp(1, 2); // 1 号受伤
                          // 准备药引手牌
    let med = e.debug_place(0, "shan");
    // 发动青囊
    let skill_def = (0..e.catalog().skills.len() as u16)
        .find(|i| e.catalog().skills[*i as usize].name == "青囊")
        .expect("目录应有青囊");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::ActivateSkill { skill: skill_def },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "应挂起于目标询问: {out:?}"
    );
    // 目标询问：选择 1 号
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "qingnang_target");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Player(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "应询问弃哪张手牌: {out:?}"
    );
    // 弃牌询问：弃掉 med
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "qingnang_card");
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(med)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 目标回血 + 药引弃置 + 守恒
    assert_eq!(hp(&e, 1), 3, "青囊应回复 1 点体力");
    assert_eq!(discard_count(&e, med), 1, "药引手牌应弃置恰一份");
    assert_zone_conservation(&e);
}
