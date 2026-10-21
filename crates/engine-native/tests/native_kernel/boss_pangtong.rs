use super::*;

/// 天狱：1 号（庞统）结束阶段——未横置的自体保持、其余未横置者横置；
/// 自体已横置时先解除再横置全场。
#[test]
fn boss_pangtong_tianyu_end_link_all() {
    let mut e = engine_native::Engine::new_boss_stage(87064, 4, 0, 3, 6, 4, 0);
    let pt_def = pangtong_def_of(&e);
    e.debug_set_general_def(1, pt_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(pt_def).map(|g| g.0),
        Some("庞统"),
        "庞统入目录"
    );
    for sk in ["天狱", "栖梧"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }
    advance_to_play(&mut e, 0);
    // 推进到 1 号结束阶段：全场横置（自体未横置→保持，0/2/3 横置）
    advance_to_play(&mut e, 1);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    assert!(e.state_core().player(0).linked, "天狱：0 号横置");
    assert!(e.state_core().player(2).linked, "天狱：2 号横置");
    assert!(e.state_core().player(3).linked, "天狱：3 号横置");
    assert!(!e.state_core().player(1).linked, "天狱：自体未横置保持");
    // 下一轮 1 号结束阶段：自体仍未横置→无变化（全场已横置）
    let round2: Vec<bool> = (0..4).map(|t| e.state_core().player(t).linked).collect();
    advance_to_play(&mut e, 1);
    next_phase_settled(&mut e);
    next_phase_settled(&mut e);
    let after: Vec<bool> = (0..4).map(|t| e.state_core().player(t).linked).collect();
    assert_eq!(after, round2, "天狱：全场已横置时幂等");
}

/// 栖梧：使用梅花牌（顺手牵羊梅花实体）后受伤回复 1；满血不回复。
#[test]
fn boss_pangtong_qiwu_club_recover() {
    let mut e = engine_native::Engine::new_boss_stage(87065, 4, 0, 0, 6, 4, 0);
    let pt_def = pangtong_def_of(&e);
    e.debug_set_general_def(1, pt_def);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(1, 2); // 先压伤（max_hp=4 不变）
    advance_to_play(&mut e, 0);
    // 1 号回合：找梅花牌使用（顺手=可自选目标；无顺手则梅花杀打相邻）
    advance_to_play(&mut e, 1);
    let club_card = {
        let core = e.state_core();
        core.deck
            .iter()
            .copied()
            .find(|&c| {
                e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Club
                    && e.catalog().cards[c as usize].kind == CardKind::Snatch
            })
            .or_else(|| {
                core.deck.iter().copied().find(|&c| {
                    e.catalog().cards[c as usize].suit == engine_native::catalog::Suit::Club
                        && e.catalog().cards[c as usize].kind == CardKind::Sha
                })
            })
            .expect("牌堆中无梅花顺手/杀")
    };
    {
        let core = e.state_core_mut();
        core.deck.retain(|&x| x != club_card);
        core.player_mut(1).hand.push(club_card);
    }
    let out = play(&mut e, club_card, vec![0]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出牌受理（{out:?}）"
    );
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) => {
                let (cid, obs) = (c.id, c.observer);
                submit_cmd(
                    &mut e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![ChoiceOption::Pass],
                    },
                );
            }
            None => break,
        }
    }
    assert_eq!(hp(&e, 1), 3, "栖梧：梅花牌使用后回复 1（2→3）");
}
