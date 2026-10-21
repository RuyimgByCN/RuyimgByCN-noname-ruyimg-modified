use super::*;

/// 龙胆：杀当闪——respond_shan 选项含目标手牌中的杀，出杀代闪无伤害。
#[test]
fn zhaoyun_longdan_sha_as_dodge() {
    let mut e = Engine::new(499, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(1, "龙胆");
    let sha_for_dodge = e.debug_place(1, "sha");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]); // 出杀 → 出闪询问（含龙胆转化选项）
    let c = e.open_choice().expect("出闪询问应挂起");
    assert_eq!(c.spec.prompt, "respond_shan");
    // 选项应包含手牌中的杀（龙胆转化）
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(sha_for_dodge)),
        "龙胆选项应含杀: {:?}",
        c.spec.options
    );
    let cid = c.id;
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(sha_for_dodge)],
        },
    });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert_eq!(hp(&e, 1), 4, "杀当闪应免除伤害");
    assert_eq!(discard_count(&e, sha_for_dodge), 1, "转化打出的杀照常弃置");
    assert_zone_conservation(&e);
}
