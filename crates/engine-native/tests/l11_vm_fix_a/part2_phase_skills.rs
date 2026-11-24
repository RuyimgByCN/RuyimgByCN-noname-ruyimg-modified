use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_vm_fix_a_miji_fulin_and_guzheng() {
    // 1. miji (damaged 2 HP -> confirm trigger_confirm -> draw 2 -> give 2 cards to ally)
    let mut e = start_clean_4p_play(910_101);
    e.debug_grant(0, "miji");
    e.debug_set_hp(0, 3);
    e.debug_clear_hand(3);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut e, 0, GameAction::NextPhase); // -> End
    assert!(prompt_is(&e, "trigger_confirm"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert!(prompt_is(&e, "l11_vm_a_miji_cards"));
    let opts = e.open_choice().unwrap().spec.options.clone();
    assert_eq!(opts.len(), 2);
    let _ = answer(&mut e, 0, opts);
    assert!(prompt_is(&e, "l11_vm_a_miji_target"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(3)]);
    assert_eq!(e.debug_hand(3).len(), 2);
    assert_eq!(e.debug_hand(0).len(), 0);

    // 2. fulin (cards gained during own turn bypass discard limit & cannot be discarded in Phase::Discard)
    let mut e2 = start_clean_4p_play(910_102);
    e2.debug_grant(0, "fulin");
    e2.debug_set_hp(0, 1);
    let c_keep = e2.debug_place(0, "tao");
    let wz = e2.debug_place(0, "wuzhong");
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    let drawn_in_turn = e2.debug_hand(0).into_iter().filter(|&c| c != c_keep).collect::<Vec<_>>();
    assert_eq!(drawn_in_turn.len(), 2);
    // Enter Discard phase: hp == 1, total hand == 3 -> only c_keep counts against limit -> NO discard ask, OR wait:
    // keep 2 `tao` before own turn so total ignored = 2, non-ignored = 2, limit = 1 -> must discard 1 and ONLY c_keep* are discardable!
    let c_keep2 = e2.debug_place(0, "shan");
    // Remove fulin tag from c_keep2 manually since debug_place didn't fire Draw: only drawn_in_turn came from CardsDrawn!
    assert_eq!(
        e2.state_core().vm_aux.flags.get(&(0, format!("l11_fulin_tag:{}", drawn_in_turn[0]))).copied().unwrap_or(0),
        1
    );
    assert_eq!(
        e2.state_core().vm_aux.flags.get(&(0, format!("l11_fulin_tag:{c_keep2}"))).copied().unwrap_or(0),
        0
    );
    let _ = submit(&mut e2, 0, GameAction::NextPhase); // Play -> Discard
    let disc_ch = e2.open_choice().expect("discard choice must open for 2 untagged > hp 1");
    assert!(!disc_ch.spec.options.contains(&ChoiceOption::Card(drawn_in_turn[0])));
    assert!(!disc_ch.spec.options.contains(&ChoiceOption::Card(drawn_in_turn[1])));
    assert!(disc_ch.spec.options.contains(&ChoiceOption::Card(c_keep)));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Card(c_keep)]);
    drain_ai(&mut e2, 8);
    assert!(e2.debug_hand(0).contains(&drawn_in_turn[0]));
    assert!(e2.debug_hand(0).contains(&drawn_in_turn[1]));

    // 3. guzheng (other player discards >= 2 in Phase::Discard -> keep 1 returned to them, gain rest)
    let mut e3 = start_clean_4p_play(910_103);
    e3.debug_grant(2, "guzheng");
    advance_to_phase(&mut e3, 0, Phase::Play);
    e3.debug_set_hp(0, 1);
    let g1 = e3.debug_place(0, "sha");
    let g2 = e3.debug_place(0, "shan");
    let g3 = e3.debug_place(0, "tao");
    let _ = submit(&mut e3, 0, GameAction::NextPhase); // -> Discard
    let disc_out = answer(&mut e3, 0, vec![ChoiceOption::Card(g1), ChoiceOption::Card(g2)]);
    assert!(!matches!(disc_out, CommandOutcome::Rejected(_)), "discard rejected: {disc_out:?}");
    let _ = submit(&mut e3, 0, GameAction::NextPhase); // DiscardEnd -> triggers guzheng
    assert!(prompt_is(&e3, "l11_vm_a_guzheng_keep"));
    let _ = answer(&mut e3, 2, vec![ChoiceOption::Card(g1)]);
    assert!(prompt_is(&e3, "l11_vm_a_guzheng_gain"));
    let _ = answer(&mut e3, 2, vec![ChoiceOption::Confirm]);
    assert!(e3.debug_hand(0).contains(&g1));
    assert!(e3.debug_hand(0).contains(&g3));
    assert!(e3.debug_hand(2).contains(&g2));

    // guzheng 1-card confirm returns card to target without extra gain
    let mut e3b = start_clean_4p_play(910_110);
    e3b.debug_grant(2, "guzheng");
    e3b.debug_set_hp(0, 1);
    let ga1 = e3b.debug_place(0, "sha");
    let _ga2 = e3b.debug_place(0, "shan");
    let _ = submit(&mut e3b, 0, GameAction::NextPhase);
    let _ = answer(&mut e3b, 0, vec![ChoiceOption::Card(ga1)]);
    let _ = submit(&mut e3b, 0, GameAction::NextPhase);
    assert!(prompt_is(&e3b, "l11_vm_a_guzheng_confirm"));
    let _ = answer(&mut e3b, 2, vec![ChoiceOption::Confirm]);
    assert!(e3b.debug_hand(0).contains(&ga1));
    assert!(e3b.debug_hand(2).is_empty());

    // guzheng >=2 cards: forfeit rest by Pass on guzheng_gain
    let mut e3c = start_clean_4p_play(910_111);
    e3c.debug_grant(2, "guzheng");
    e3c.debug_set_hp(0, 1);
    let gb1 = e3c.debug_place(0, "sha");
    let gb2 = e3c.debug_place(0, "shan");
    let _gb3 = e3c.debug_place(0, "tao");
    let _ = submit(&mut e3c, 0, GameAction::NextPhase);
    let _ = answer(&mut e3c, 0, vec![ChoiceOption::Card(gb1), ChoiceOption::Card(gb2)]);
    let _ = submit(&mut e3c, 0, GameAction::NextPhase);
    assert!(prompt_is(&e3c, "l11_vm_a_guzheng_keep"));
    let _ = answer(&mut e3c, 2, vec![ChoiceOption::Card(gb1)]);
    assert!(prompt_is(&e3c, "l11_vm_a_guzheng_gain"));
    let _ = answer(&mut e3c, 2, vec![ChoiceOption::Pass]);
    assert!(e3c.debug_hand(0).contains(&gb1));
    assert!(!e3c.debug_hand(2).contains(&gb2));
    assert!(e3c.debug_hand(2).is_empty());
}

#[test]
fn l11_vm_fix_a_langmie_both_branches_dealt_vs_taken() {
    let mut e = start_clean_4p_play(910_104);
    e.debug_grant(2, "langmie");
    let sha1 = e.debug_place(0, "sha");
    let sha2 = e.debug_place(0, "sha");
    let eq_w = place_kind(&mut e, 0, CardKind::Zhuge);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq_w, targets: vec![] });
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    drain_ai(&mut e, 8);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    drain_ai(&mut e, 8);
    assert_eq!(e.debug_hp(1), 3, "seat 0 dealt 2 damage during turn");

    let cost1 = e.debug_place(2, "tao");
    let cost2 = e.debug_place(2, "shan");
    let h2_before = e.debug_hand(2).len();
    let _ = submit(&mut e, 0, GameAction::NextPhase); // PlayEnd -> triggers langmie category (>=2 basic Sha)
    assert!(prompt_is(&e, "l11_vm_a_langmie_play_discard"));
    let _ = answer(&mut e, 2, vec![ChoiceOption::Card(cost1)]);
    assert_eq!(e.debug_hand(2).len(), h2_before + 1, "discard 1 draw 2");

    // End turn -> TurnEndWindow fires langmie_damage because seat 0 DEALT 2 dmg (> 1)
    while e.open_choice().is_none() && e.state_core().turn.current == 0 {
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    assert!(
        prompt_is(&e, "l11_vm_a_langmie_end_discard"),
        "open_choice={:?}, turn={:?}, phase={:?}, dmg_dealt={:?}",
        e.open_choice().map(|c| (c.observer, c.spec.prompt)),
        e.state_core().turn.current,
        e.state_core().turn.phase,
        e.state_core().vm_aux.flags.get(&(0, "l11_turn_dmg_dealt".to_string())),
    );
    let _ = answer(&mut e, 2, vec![ChoiceOption::Card(cost2)]);
    drain_ai(&mut e, 8);
    assert_eq!(e.debug_hp(0), 4, "seat 0 takes 1 damage from langmie_damage");
}

#[test]
fn l11_vm_fix_a_yingjian_kunfen_lianpo_fenming_gzyinghun() {
    // 1. yingjian + kunfen (locked at 1 HP dies before drawing -> kunfen_unlocked optional) + gzyinghun
    let mut e = start_clean_4p_play(910_105);
    e.debug_grant(1, "yingjian");
    e.debug_grant(1, "kunfen");
    e.debug_grant(1, "gzyinghun");
    e.debug_set_hp(1, 3);
    e.debug_clear_hand(3);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    // Advance to seat 1 Prepare -> yingjian & gzyinghun trigger
    advance_to_choice_on_seat(&mut e, 1, Phase::Prepare);
    // Drive yingjian / gzyinghun deterministically via manual answers when prompted:
    loop {
        let Some(ch) = e.open_choice() else { break; };
        match ch.spec.prompt {
            "l11_vm_a_yingjian_target" => {
                let _ = answer(&mut e, 1, vec![ChoiceOption::Player(3)]);
            }
            "l11_vm_a_gzyinghun_target" => {
                let _ = answer(&mut e, 1, vec![ChoiceOption::Player(2)]);
            }
            "l11_vm_a_gzyinghun_mode" => {
                let _ = answer(&mut e, 1, vec![ChoiceOption::Tag(1)]); // Draw 1, Discard 2
            }
            "l11_vm_a_gzyinghun_discard" => {
                let picks: Vec<_> = ch.spec.options.clone().into_iter().take(ch.spec.min).collect();
                let _ = answer(&mut e, 2, picks);
            }
            _ => {
                drain_ai(&mut e, 8);
                break;
            }
        }
    }
    assert_eq!(e.debug_hp(3), 4, "yingjian virtual Sha ignored distance to seat 3");

    // gzyinghun lost==1 skips mode prompt (draw 1, discard 1); lost==2 mode 0 draws 2, discards 1
    let mut e_gy1 = start_clean_4p_play(910_112);
    e_gy1.debug_grant(1, "gzyinghun");
    e_gy1.debug_set_hp(1, 4);
    let _c2 = e_gy1.debug_place(2, "tao");
    let _ = submit(&mut e_gy1, 0, GameAction::NextPhase);
    advance_to_choice_on_seat(&mut e_gy1, 1, Phase::Prepare);
    assert!(prompt_is(&e_gy1, "l11_vm_a_gzyinghun_target"));
    let _ = answer(&mut e_gy1, 1, vec![ChoiceOption::Player(2)]);
    assert!(prompt_is(&e_gy1, "l11_vm_a_gzyinghun_discard"), "lost==1 must skip mode prompt");
    let pick1 = e_gy1.open_choice().unwrap().spec.options[0];
    assert_eq!(e_gy1.open_choice().unwrap().spec.min, 1);
    let _ = answer(&mut e_gy1, 2, vec![pick1]);
    assert_eq!(e_gy1.debug_hand(2).len(), 1);

    let mut e_gy2 = start_clean_4p_play(910_113);
    e_gy2.debug_grant(1, "gzyinghun");
    e_gy2.debug_set_hp(1, 3);
    let _ = submit(&mut e_gy2, 0, GameAction::NextPhase);
    advance_to_choice_on_seat(&mut e_gy2, 1, Phase::Prepare);
    let _ = answer(&mut e_gy2, 1, vec![ChoiceOption::Player(2)]);
    assert!(prompt_is(&e_gy2, "l11_vm_a_gzyinghun_mode"));
    let _ = answer(&mut e_gy2, 1, vec![ChoiceOption::Tag(0)]); // Draw 2, Discard 1
    assert!(prompt_is(&e_gy2, "l11_vm_a_gzyinghun_discard"));
    let ch2 = e_gy2.open_choice().unwrap();
    assert_eq!(ch2.spec.min, 1);
    let pick2 = ch2.spec.options[0];
    let _ = answer(&mut e_gy2, 2, vec![pick2]);
    assert_eq!(e_gy2.debug_hand(2).len(), 1);

    // Test unlocked vs locked kunfen at End phase
    e.debug_set_hp(1, 3);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // -> Play or Draw
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase); // -> Discard
    let _ = submit(&mut e, 1, GameAction::NextPhase); // -> End: locked kunfen auto-fires LoseHp(1) + Draw(2)
    drain_ai(&mut e, 8);
    assert_eq!(e.debug_hp(1), 2);
    assert_eq!(e.debug_hand(1).len(), 2);

    // Unlock kunfen -> prompts ASK_KUNFEN_CONFIRM and allows Pass!
    advance_to_phase(&mut e, 1, Phase::Play);
    e.state_core_mut().vm_aux.flags.insert((1, "kunfen_unlocked".to_string()), 1);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    let _ = submit(&mut e, 1, GameAction::NextPhase);
    assert!(prompt_is(&e, "l11_vm_a_kunfen_confirm"));
    let _ = answer(&mut e, 1, vec![ChoiceOption::Pass]);
    assert_eq!(e.debug_hp(1), 2);

    // 2. fenming: linked owner discards 1 he from each linked player
    let mut e2 = start_clean_4p_play(910_106);
    e2.debug_grant(0, "fenming");
    e2.state_core_mut().player_mut(0).linked = true;
    e2.state_core_mut().player_mut(1).linked = true;
    let c0 = e2.debug_place(0, "sha");
    let c1 = e2.debug_place(1, "shan");
    let c2 = e2.debug_place(2, "tao");
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    let _ = submit(&mut e2, 0, GameAction::NextPhase);
    assert!(prompt_is(&e2, "l11_vm_a_fenming_confirm"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    assert!(prompt_is(&e2, "l11_vm_a_fenming_card"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Card(c0)]);
    assert!(prompt_is(&e2, "l11_vm_a_fenming_card"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Card(c1)]);
    assert!(!e2.debug_hand(0).contains(&c0));
    assert!(!e2.debug_hand(1).contains(&c1));
    assert!(e2.debug_hand(2).contains(&c2));

    // 3. lianpo: kill opponent in turn -> TurnPhaseAfter prompts extra turn for owner
    let mut e3 = start_clean_4p_play(910_107);
    e3.debug_grant(0, "lianpo");
    e3.debug_set_hp(1, 1);
    let k_sha = e3.debug_place(0, "sha");
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: k_sha, targets: vec![1] });
    drain_ai(&mut e3, 16);
    assert!(!e3.debug_alive(1));
    for _ in 0..4 {
        if e3.open_choice().is_some() || e3.state_core().turn.current != 0 {
            break;
        }
        let _ = submit(&mut e3, 0, GameAction::NextPhase);
    }
    assert!(prompt_is(&e3, "l11_vm_a_lianpo_confirm"));
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e3.state_core().turn.current, 0, "lianpo grants immediate extra turn to owner");
}

#[test]
fn l11_vm_fix_a_chunlao_and_nzry_longnu() {
    // 1. chunlao store in End phase + spend on ally Dying + wansha block
    let mut e = start_clean_4p_play(910_108);
    e.debug_grant(0, "chunlao");
    let cl_sha1 = e.debug_place(0, "sha");
    let cl_sha2 = e.debug_place(0, "sha");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert!(prompt_is(&e, "l11_vm_a_chunlao_store"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(cl_sha1), ChoiceOption::Card(cl_sha2)]);
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "chunlao".to_string())).map(Vec::len),
        Some(2)
    );

    // Seat 2 kills seat 3 down to 0 HP -> seat 0 prompted in Dying with Skill(chunlao)
    advance_to_phase(&mut e, 2, Phase::Play);
    e.debug_set_hp(3, 1);
    e.debug_clear_hand(3);
    e.debug_clear_hand(2);
    let atk = e.debug_place(2, "sha");
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: atk, targets: vec![3] });
    let ch = e.open_choice().expect("dying choice");
    let cl_id = e.catalog().skill_by_name("chunlao").unwrap();
    assert!(ch.spec.options.contains(&ChoiceOption::Skill(cl_id)));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Skill(cl_id)]);
    assert!(prompt_is(&e, "l11_vm_a_chunlao_spend"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(cl_sha1)]);
    drain_ai(&mut e, 8);
    assert_eq!(e.debug_hp(3), 1, "chunlao Jiu saved dying ally back to 1 HP");
    assert_eq!(
        e.state_core().vm_aux.piles.get(&(0, "chunlao".to_string())).map(Vec::len),
        Some(1)
    );

    // Verify wansha blocks seat 0 from saving someone else via chunlao
    e.debug_grant(2, "wansha");
    e.debug_set_hp(3, 1);
    let atk2 = e.debug_place(2, "sha");
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: atk2, targets: vec![3] });
    if let Some(ch_w) = e.open_choice() {
        if ch_w.observer == 0 {
            assert!(!ch_w.spec.options.contains(&ChoiceOption::Skill(cl_id)), "wansha must block third-party chunlao save");
        }
    }
    drain_ai(&mut e, 16);

    // 2. nzry_longnu: Yang first Play (LoseHp 1 + Red->FireSha unlimited distance) -> Yin next Play (LoseMaxHp 1 + Trick->LeiSha unlimited count)
    let mut e2 = start_clean_4p_play(910_109);
    e2.debug_grant(1, "nzry_longnu");
    advance_to_phase(&mut e2, 1, Phase::Draw);
    e2.state_core_mut().vm_aux.flags.insert((1, "q4_retieji_target_sealed".to_string()), 1);
    assert!(e2.catalog().is_tieji_sealed(e2.state_core(), 1));
    let _ = submit(&mut e2, 1, GameAction::NextPhase); // Draw -> Play (locked triggers under tieji)
    assert_eq!(e2.state_core().turn.phase, Phase::Play);
    assert_eq!(e2.debug_hp(1), 4, "first nzry_longnu (Yang) loses 1 HP even under tieji");
    let red_c = place_by_suit(&mut e2, 1, "shan", Suit::Heart);
    e2.debug_clear_hand(3);
    assert!(
        !matches!(
            submit(&mut e2, 1, GameAction::PlayCard { card: red_c, targets: vec![3] }),
            CommandOutcome::Rejected(_)
        ),
        "Yang nzry_longnu converts Red hand card to distance-unlimited FireSha (seat 3 at dist 2)"
    );
    drain_ai(&mut e2, 8);
    assert_eq!(e2.debug_hp(3), 4);
    advance_to_phase(&mut e2, 1, Phase::Discard);
    assert_eq!(
        e2.state_core().vm_aux.flags.get(&(1, "nzry_longnu_fire_turn".to_string())).copied().unwrap_or(0),
        0,
        "fire mode expires at phaseUseEnd"
    );

    // Next round Play phase: Yin mode (LoseMaxHp 1 + Trick -> LeiSha unlimited count)
    advance_to_phase(&mut e2, 1, Phase::Play);
    assert_eq!(e2.state_core().player(1).max_hp, 4, "second nzry_longnu (Yin) loses 1 max HP");
    let tr1 = e2.debug_place(1, "wuzhong");
    let tr2 = place_vanilla_dismantle(&mut e2, 1);
    let _ = tr1;
    e2.debug_clear_hand(2);
    assert!(!matches!(
        submit(&mut e2, 1, GameAction::PlayCard { card: tr1, targets: vec![2] }),
        CommandOutcome::Rejected(_)
    ));
    assert!(!matches!(
        submit(&mut e2, 1, GameAction::PlayCard { card: tr2, targets: vec![2] }),
        CommandOutcome::Rejected(_)
    ), "Yin nzry_longnu converts Trick hand card to count-unlimited LeiSha");
    drain_ai(&mut e2, 8);
    assert_eq!(e2.debug_hp(2), 3);
}
