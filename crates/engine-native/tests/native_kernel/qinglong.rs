use super::*;

/// P2/N08 批③-b 第六把（规则 v70，2026-10-07）：青龙偃月刀（♠5，射程 3）
/// ——杀被目标出闪闪避后，来源可立即对相同目标再使用一张手牌中的杀
///（可选可放弃；每次被闪都可再追问，来源手牌杀尽即自然终止）。
#[test]
fn qinglong_replays_sha_after_dodge() {
    let mut e = Engine::new(200, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let ql = e.debug_move_named_card_to_hand(0, "qinglong");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ql,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备青龙刀: {out:?}"
    );
    // 目标持两张闪；来源持两张杀。
    let shan_t = e.debug_place(1, "shan");
    let shan_t2 = e.debug_place(1, "shan");
    let sha_a = e.debug_place(0, "sha");
    let sha_b = e.debug_place(0, "sha");
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 目标出闪 → 青龙再杀询问。
    let shan_ask = e.open_choice().expect("出闪询问应挂起");
    assert_eq!(shan_ask.observer, 1);
    let (sid, sobs) = (shan_ask.id, shan_ask.observer);
    submit_cmd(
        &mut e,
        sobs,
        GameAction::Answer {
            choice_id: sid,
            selection: vec![ChoiceOption::Card(shan_t)],
        },
    );
    let replay = e.open_choice().expect("青龙再杀询问应挂起");
    assert_eq!(
        (replay.spec.prompt, replay.observer),
        ("qinglong_replay_sha", 0)
    );
    assert!(replay.spec.options.contains(&ChoiceOption::Card(sha_a)));
    assert!(replay.spec.options.contains(&ChoiceOption::Card(sha_b)));
    // 选第一张杀 → 新杀结算 → 目标再出闪 → 再追问 → 放弃 → 无伤。
    let (rid, robs) = (replay.id, replay.observer);
    submit_cmd(
        &mut e,
        robs,
        GameAction::Answer {
            choice_id: rid,
            selection: vec![ChoiceOption::Card(sha_a)],
        },
    );
    assert!(!e.debug_hand(0).contains(&sha_a), "再杀已从手牌打出");
    let shan_ask2 = e.open_choice().expect("第二杀的出闪询问");
    assert_eq!(shan_ask2.observer, 1);
    let (sid2, sobs2) = (shan_ask2.id, shan_ask2.observer);
    submit_cmd(
        &mut e,
        sobs2,
        GameAction::Answer {
            choice_id: sid2,
            selection: vec![ChoiceOption::Card(shan_t2)],
        },
    );
    let replay2 = e.open_choice().expect("第二次青龙再杀询问");
    assert_eq!(replay2.spec.prompt, "qinglong_replay_sha");
    let (rid2, robs2) = (replay2.id, replay2.observer);
    submit_cmd(
        &mut e,
        robs2,
        GameAction::Answer {
            choice_id: rid2,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hp(1), 4, "两次出闪 + 放弃再杀：目标无伤");
    assert!(e.debug_hand(0).contains(&sha_b), "未选的杀留手牌");
}
