use super::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn l11_vm_c_audit36_parity_a_baonu_to_retieji() {
    {
        let e = start_2p(2026_9501);
        assert_is_vm_c(
            &e,
            &[
                "baonu", "chongzhen", "jintao", "new_repaoxiao", "new_repaoxiao2", "nzry_cunmu",
                "redangxian", "rejizhi", "reqiangxi", "retieji",
            ],
        );
    }

    // 1. `baonu`: starts/granted with +2 marks; gains marks equal to damage dealt/taken.
    {
        let mut e = start_2p(2026_9502);
        e.debug_grant(0, "baonu");
        assert_eq!(flag(&e, 0, "l11c_baonu_marks"), 2);
        e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
        drain_pass(&mut e, 8);
        assert_eq!(flag(&e, 0, "l11c_baonu_marks"), 4, "baonu gains marks on damage dealt");
        e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        drain_pass(&mut e, 8);
        assert_eq!(flag(&e, 0, "l11c_baonu_marks"), 5, "baonu gains marks on damage taken");
    }

    // 2. `chongzhen`: longdan conversion steals 1 hand card from counterpart (confirm + restore + AI).
    {
        let mut e = start_2p(2026_9503);
        e.debug_grant(0, "龙胆");
        e.debug_grant(0, "chongzhen");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let shan_as_sha = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let steal_c = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        let ld_def = e.catalog().skill_by_name("龙胆").unwrap();
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: shan_as_sha, targets: vec![1] });
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.options.contains(&ChoiceOption::Skill(ld_def)) {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Skill(ld_def)]);
            }
        }
        let ch_cz = e.open_choice().cloned().expect("chongzhen:confirm");
        assert_eq!(ch_cz.spec.prompt, "chongzhen:confirm");
        let mut e = roundtrip_restore(&e);
        let ai_act = plan_ai_answer(&e, 0);
        assert_eq!(ai_act, GameAction::Answer { choice_id: ch_cz.id, selection: vec![ChoiceOption::Confirm] });
        let _ = submit(&mut e, 0, ai_act);
        let ch_pick = e.open_choice().cloned().expect("chongzhen:choose_card");
        assert_eq!(ch_pick.spec.prompt, "chongzhen:choose_card");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(steal_c)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).hand.contains(&steal_c), "chongzhen stole target hand card");
    }

    // 3. `jintao` + 4. `new_repaoxiao` + `new_repaoxiao2`:
    {
        // jintao: 1st Play Sha +1 dmg (even if qiangwu exempted count!), 2nd Play Sha unrespondable!
        let mut ej = start_3p(2026_9504);
        ej.debug_grant(0, "jintao");
        advance_to_phase(&mut ej, 0, Phase::Play, None);
        ej.debug_clear_hand(0);
        ej.debug_clear_hand(1);
        ej.debug_clear_hand(2);
        let s1 = place_hand_kind_any(&mut ej, 0, CardKind::Sha);
        let s2 = place_hand_kind_any(&mut ej, 0, CardKind::Sha);
        let _p2_shan = place_hand_kind_any(&mut ej, 2, CardKind::Shan);
        let hp1 = ej.state_core().player(1).hp;
        let hp2 = ej.state_core().player(2).hp;
        let _ = submit(&mut ej, 0, GameAction::PlayCard { card: s1, targets: vec![1] });
        drain_pass(&mut ej, 16);
        assert_eq!(ej.state_core().player(1).hp, hp1 - 2, "jintao 1st Sha deals +1 dmg");
        let _ = submit(&mut ej, 0, GameAction::PlayCard { card: s2, targets: vec![2] });
        assert!(ej.open_choice().is_none(), "jintao 2nd Sha must be unrespondable");
        drain_pass(&mut ej, 16);
        assert_eq!(ej.state_core().player(2).hp, hp2 - 1);

        // new_repaoxiao: unlimited Sha + after 1st Sha in Play, Sha in turn ignores distance (`refresh.js:11676`).
        let mut erp = start_2p(2026_9505);
        erp.debug_grant(0, "new_repaoxiao");
        equip_kind_suit(&mut erp, 0, CardKind::HorsePlus, None);
        advance_to_phase(&mut erp, 0, Phase::Play, None);
        erp.debug_clear_hand(0);
        erp.debug_clear_hand(1);
        let s_a = place_hand_kind_any(&mut erp, 0, CardKind::Sha);
        let s_b = place_hand_kind_any(&mut erp, 0, CardKind::Sha);
        let s_c = place_hand_kind_any(&mut erp, 0, CardKind::Sha);
        let shan_1 = place_hand_kind_any(&mut erp, 1, CardKind::Shan);
        let res_a = submit(&mut erp, 0, GameAction::PlayCard { card: s_a, targets: vec![1] });
        assert!(!matches!(res_a, CommandOutcome::Rejected(_)));
        let _ = answer(&mut erp, 1, vec![ChoiceOption::Card(shan_1)]);
        drain_pass(&mut erp, 16);
        assert_eq!(erp.state_core().player(1).hp, 4);
        // After 1st Sha, distance unlimited + unlimited Sha count even vs HorsePlus!
        equip_kind_suit(&mut erp, 1, CardKind::HorsePlus, None);
        let res_b = submit(&mut erp, 0, GameAction::PlayCard { card: s_b, targets: vec![1] });
        assert!(!matches!(res_b, CommandOutcome::Rejected(_)), "new_repaoxiao2 ignores distance");
        drain_pass(&mut erp, 16);
        assert_eq!(erp.state_core().player(1).hp, 3, "new_repaoxiao 2nd Sha hits despite HorsePlus");
        let res_c = submit(&mut erp, 0, GameAction::PlayCard { card: s_c, targets: vec![1] });
        assert!(!matches!(res_c, CommandOutcome::Rejected(_)), "new_repaoxiao grants unlimited Sha");
        drain_pass(&mut erp, 16);
        assert_eq!(erp.state_core().player(1).hp, 2);
    }

    // 5. `nzry_cunmu`: draws from bottom of deck!
    {
        let mut e = start_2p(2026_9506);
        e.debug_grant(0, "nzry_cunmu");
        e.debug_clear_hand(0);
        let bot0 = e.state_core().deck[0];
        let bot1 = e.state_core().deck[1];
        e.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).hand.contains(&bot0) && e.state_core().player(0).hand.contains(&bot1), "nzry_cunmu must draw from deck bottom");
    }

    // 6. `redangxian`: TurnBegin recovers 1 discarded Sha and adds an extra Play phase before Draw.
    {
        let mut e = start_2p(2026_9507);
        e.debug_grant(0, "redangxian");
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha_disc = take_deck_or_discard_by(&mut e, |d| d.kind == CardKind::Sha);
        e.state_core_mut().discard.clear();
        e.state_core_mut().discard.push(sha_disc);
        e.debug_set_turn(1, Phase::End, 1);
        let _ = submit(&mut e, 1, GameAction::NextPhase); // -> TurnBegin(0) triggers redangxian; begin_judge pops Phase::Play first!
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).hand.contains(&sha_disc), "redangxian recovered discarded Sha");
        assert_eq!(e.state_core().turn.phase, Phase::Play, "redangxian enters Play phase first");
        assert_eq!(
            e.state_core().turn.extra_phases,
            vec![0],
            "redangxian queues Judge (extra_phases code 0) after Play"
        );
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().turn.phase, Phase::Judge, "redangxian transitions Play -> Judge");
    }

    // 7. `rejizhi`: trick draw 1 -> if basic can discard for +1 hand limit; else keep.
    {
        let mut e = start_2p(2026_9508);
        e.debug_grant(0, "rejizhi");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let trk = place_hand_card_named(&mut e, 0, "wuzhong");
        let top_basic = take_deck_or_discard_by(&mut e, |d| d.kind == CardKind::Sha);
        e.state_core_mut().deck.push(top_basic);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: trk, targets: vec![] });
        let ch = e.open_choice().cloned().expect("rejizhi:confirm");
        assert_eq!(ch.spec.prompt, "rejizhi:confirm");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        let ch_d = e.open_choice().cloned().expect("rejizhi:discard");
        assert_eq!(ch_d.spec.prompt, "rejizhi:discard");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(top_basic)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), e.state_core().player(0).hp as usize + 1, "rejizhi discard basic gave +1 hand limit");

        // Physical Delayed Trick (`Le`) DOES trigger `rejizhi`, whereas `huoji` converted trick (`use_as`) does NOT!
        let mut er = start_2p(2026_9511);
        er.debug_grant(0, "rejizhi");
        er.debug_grant(0, "huoji");
        advance_to_phase(&mut er, 0, Phase::Play, None);
        er.debug_clear_hand(0);
        let red_c = place_card_suit_rank(&mut er, 0, Suit::Heart, CardKind::Sha, 1, 13);
        let _ = activate(&mut er, 0, "huoji");
        let _ = answer(&mut er, 0, vec![ChoiceOption::Card(red_c)]);
        let _ = answer(&mut er, 0, vec![ChoiceOption::Player(1)]);
        if let Some(ch_r) = er.open_choice() {
            assert_ne!(ch_r.spec.prompt, "rejizhi:confirm", "converted trick via huoji must NOT trigger rejizhi");
        }
        drain_pass(&mut er, 16);
        let le_p = place_hand_kind_any(&mut er, 0, CardKind::Le);
        let _ = submit(&mut er, 0, GameAction::PlayCard { card: le_p, targets: vec![1] });
        let ch_le = er.open_choice().cloned().expect("physical Le delayed trick must trigger rejizhi:confirm");
        assert_eq!(ch_le.spec.prompt, "rejizhi:confirm");
        let _ = answer(&mut er, 0, vec![ChoiceOption::Confirm]);
        drain_pass(&mut er, 16);
    }

    // 8. `reqiangxi`: once per target per turn in range, discard weapon OR lose 1 HP -> 1 dmg.
    {
        let mut e = start_2p(2026_9509);
        e.debug_grant(0, "reqiangxi");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        let w = place_hand_kind_any(&mut e, 0, CardKind::Weapon);
        let _ = activate(&mut e, 0, "reqiangxi");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(w)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 3);
        assert_eq!(e.state_core().player(0).hp, 4, "discarded weapon instead of losing HP");
        assert!(matches!(activate(&mut e, 0, "reqiangxi"), CommandOutcome::Rejected(_)), "same target blocked for rest of turn");
    }

    // 9. `retieji`: seals non-locked skills + judge suit discard or Sha unrespondable (`roundtrip_restore`!).
    {
        let mut e = start_2p(2026_9510);
        e.assign_general_named(0, "吕蒙").unwrap();
        e.assign_general_named(1, "甘宁").unwrap();
        e.debug_grant(0, "retieji");
        e.debug_grant(1, "奸雄");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sp_card = place_suit_rank_any(&mut e, 1, Suit::Spade, 1, 13);
        let _shan = place_card_suit_rank(&mut e, 1, Suit::Diamond, CardKind::Shan, 1, 13);
        let _ = move_to_deck_top_suit_rank(&mut e, Suit::Heart, 6, 6);
        let _ = sp_card;
        let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let ch = e.open_choice().cloned().expect("retieji:confirm");
        assert_eq!(ch.spec.prompt, "retieji:confirm");
        let mut e = roundtrip_restore(&e);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        assert!(
            e.open_choice().is_none(),
            "missing Heart suit skips retieji:discard & disables Shan, got {:?}",
            e.open_choice().map(|c| (&c.spec.prompt, c.observer))
        );
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 3);
        assert!(e.state_core().player(1).hand.contains(&sp_card), "sealed jianxiong did NOT steal Sha!");
    }
}
