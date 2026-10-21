use super::*;

/// 裸衣（许褚）：摸牌阶段少摸一张（手牌末尾弃置）→ 本回合杀伤害 +1。
/// 无闪目标路径与「有闪但放弃」路径各 +1；下回合加成清除。
#[test]
fn xuchu_luoyi_boosts_sha_damage() {
    let build = |seed: u64| -> Engine {
        let mut e = Engine::new(seed, 3);
        advance_to_play(&mut e, 0);
        e.debug_grant(0, "裸衣");
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        advance_to_judge(&mut e, 0);
        e.debug_clear_hand(0);
        e
    };

    let mut e = build(806);
    next_phase(&mut e); // Draw：固定摸 2 → 裸衣闸门
    let c = e.open_choice().expect("裸衣触发确认应挂起");
    assert_eq!(c.spec.prompt, "trigger_confirm");
    answer(&mut e, ChoiceOption::Confirm);
    // 少摸一张：手牌 2-1 = 1
    assert_eq!(e.debug_hand(0).len(), 1, "裸衣应少摸一张");
    // 摸牌阶段经 NextPhase 收束进入出牌阶段。
    next_phase(&mut e);
    assert_eq!(e.turn().phase, Phase::Play, "应已进入出牌阶段");
    e.debug_set_hp(1, 4);
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    // 行进途中目标可能摸到闪：选择放弃出闪 → 承受 1+1 伤害（出闪路径）。
    if e.open_choice().map(|c| c.spec.prompt) == Some("respond_shan") {
        answer(&mut e, ChoiceOption::Pass);
    }
    assert_eq!(e.debug_hp(1), 2, "裸衣加成下放弃出闪应承受 2 点伤害");

    // 决斗路径同样 +1：目标无杀可出（闪无用）→ 承受 1+1 = 2 → 濒死无救死亡。
    e.debug_clear_hand(1);
    let _shan = e.debug_place(1, "shan");
    let duel = e.debug_place(0, "duel");
    play(&mut e, duel, vec![1]);
    // 排空决斗响应/濒死询问（1 号无杀无桃 → 应答后死亡）。
    for _ in 0..16 {
        match e.open_choice() {
            Some(g) => {
                let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                    g.spec.options.iter().take(g.spec.min).copied().collect()
                } else if !g.spec.options.is_empty() {
                    vec![g.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (g.id, g.observer);
                submit_cmd(
                    &mut e,
                    obs2,
                    GameAction::Answer {
                        choice_id: id2,
                        selection: sel,
                    },
                );
            }
            None => break,
        }
    }
    assert!(!e.debug_alive(1), "2 点体力吃裸衣决斗应致死");
    assert_zone_conservation(&e);
}
