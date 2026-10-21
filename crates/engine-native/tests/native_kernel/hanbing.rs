use super::*;

/// P2/N08 批③-b 第三把（规则 v67）+ 逐技对照批一（规则 v75）：寒冰剑
///（♠2，武器射程 2）——来源确认防止伤害后，目标从手牌+装备**自选**两张
/// 逐张强制弃（对齐上游 discardPlayerCard("he", forced)×2）。
#[test]
fn hanbing_prevents_damage_discards_two_cards() {
    // ① 寒冰确认 → 目标自选：先弃注入的 +1 马（装备面），再弃手牌桃；
    //    伤害被防止、两张都离场、选序按应答执行。
    let mut e = Engine::new(170, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let hb = e.debug_move_named_card_to_hand(0, "hanbing");
    let out = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: hb,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备寒冰: {out:?}"
    );
    // 桃非闪（闪会先开出闪询问，寒冰确认在其后）：走无闪直伤路径。
    let c1 = e.debug_place(1, "tao");
    let horse = e.debug_move_named_card_to_hand(1, "horse_plus");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|c| *c != horse);
        snap.core.players[1].equips.push(horse);
        e = Engine::restore(&snap).expect("注入恢复");
    }
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    let ask = e.open_choice().expect("寒冰防止确认应挂起");
    assert_eq!(
        (ask.spec.prompt, ask.observer),
        ("hanbing_prevent_confirm", 0)
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    // 目标自选第一次：弃装备马。
    let ask = e.open_choice().expect("寒冰目标自选弃牌应挂起");
    assert_eq!(
        (ask.spec.prompt, ask.spec.kind, ask.observer),
        (
            "hanbing_target_discard",
            engine_native::ChoiceKind::SelectCards,
            1
        )
    );
    assert!(
        ask.spec.options.contains(&ChoiceOption::Card(c1))
            && ask.spec.options.contains(&ChoiceOption::Card(horse)),
        "选项=手牌+装备"
    );
    assert_eq!(
        (ask.spec.min, ask.spec.allow_cancel),
        (1, false),
        "强制弃置不可取消"
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(horse)],
        },
    );
    assert!(!e.debug_equips(1).contains(&horse), "选中的马被弃");
    // 目标自选第二次：只剩手牌桃。
    let ask = e.open_choice().expect("寒冰目标第二次自选应挂起");
    assert_eq!(
        (ask.spec.prompt, ask.observer),
        ("hanbing_target_discard", 1)
    );
    assert_eq!(
        ask.spec.options,
        vec![ChoiceOption::Card(c1)],
        "装备已空仅剩手牌"
    );
    let (aid, aobs) = (ask.id, ask.observer);
    submit_cmd(
        &mut e,
        aobs,
        GameAction::Answer {
            choice_id: aid,
            selection: vec![ChoiceOption::Card(c1)],
        },
    );
    assert_eq!(e.debug_hp(1), 4, "寒冰：伤害被防止");
    assert!(!e.debug_hand(1).contains(&c1), "第二张弃手牌");
    assert_eq!(e.debug_hand(1).len(), 0, "目标手牌弃空");

    // ② 放弃路径：下回合再杀 → 放弃确认 → 照常伤害。
    let mut left_play = false;
    for _ in 0..96 {
        if left_play
            && e.turn().current == 0
            && e.turn().phase == Phase::Play
            && e.open_choice().is_none()
        {
            break;
        }
        if e.turn().current != 0 {
            left_play = true;
        }
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
            None => {
                let actor = e.turn().current;
                match submit_cmd(&mut e, actor, GameAction::NextPhase) {
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                    other => panic!("advance: {other:?}"),
                }
            }
        }
    }
    e.debug_clear_hand(1);
    let keep = e.debug_place(1, "tao");
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![1]);
    let ask2 = e.open_choice().expect("寒冰防止确认（放弃路径）");
    eprintln!(
        "PROBE-HB2 ask2 prompt={} obs={}",
        ask2.spec.prompt, ask2.observer
    );
    let (aid2, aobs2) = (ask2.id, ask2.observer);
    submit_cmd(
        &mut e,
        aobs2,
        GameAction::Answer {
            choice_id: aid2,
            selection: vec![],
        },
    );
    eprintln!(
        "PROBE-HB2 after hp1={} hand1={:?}",
        e.debug_hp(1),
        e.debug_hand(1)
    );
    assert_eq!(e.debug_hp(1), 3, "放弃：照常伤害（4→3）");
    assert!(e.debug_hand(1).contains(&keep), "放弃：目标手牌保留");
}
