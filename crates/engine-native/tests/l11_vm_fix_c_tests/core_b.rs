use super::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn l11_vm_c_fix27_core_b_jieying_to_rekuangcai() {
    // 12. `nzry_jieying`: auto-links owner, blocks unlink, End links another, linked players get +2 hand limit.
    {
        let mut e = start_3p(2026_9201);
        e.debug_grant(0, "nzry_jieying");
        assert!(e.state_core().player(0).linked, "granting nzry_jieying must link owner");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        // Chain cannot unlink owner while nzry_jieying active.
        let ch = place_hand_kind_any(&mut e, 1, CardKind::Chain);
        advance_to_phase(&mut e, 1, Phase::Play, None);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: ch, targets: vec![0] });
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).linked, "nzry_jieying cannot be unlinked");

        advance_to_phase(&mut e, 0, Phase::End, Some("nzry_jieying:"));
        let ask = e.open_choice().cloned().expect("nzry_jieying:choose_target");
        assert_eq!(ask.spec.prompt, "nzry_jieying:choose_target");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(1).linked, "nzry_jieying End links target");
        assert_eq!(
            e.state_core().hand_limit(e.catalog(), 1),
            e.state_core().player(1).hp as usize + 2,
            "nzry_jieying grants +2 hand limit to linked player"
        );
    }

    // 13. `qiaoshui` + `qiaoshui3` / `qiaoshui2`: win adds extra target or removes on multitarget; lose blocks tricks.
    {
        let mut e = start_3p(2026_9202);
        e.debug_grant(0, "qiaoshui");
        e.debug_set_turn(0, Phase::Draw, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let k_win = place_suit_rank_any(&mut e, 0, Suit::Spade, 11, 13);
        let lo0 = place_suit_rank_any(&mut e, 0, Suit::Heart, 2, 4);
        let lo1 = place_suit_rank_any(&mut e, 1, Suit::Club, 2, 6);
        let dis = place_hand_kind_any(&mut e, 0, CardKind::Dismantle);
        let _p1_keep = place_suit_rank_any(&mut e, 1, Suit::Diamond, 2, 6);
        let _p2_keep = place_hand_kind_any(&mut e, 2, CardKind::Shan);
        let _ = (lo0, lo1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let ch_t = e.open_choice().cloned().expect("qiaoshui:target");
        assert_eq!(ch_t.spec.prompt, "qiaoshui:target");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(k_win)]);
        drain_pass(&mut e, 8);
        assert!(e.catalog().skill_by_name("qiaoshui3").is_some_and(|d| e.state_core().has_skill(0, d)));
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: dis, targets: vec![1] });
        let ch_add = e.open_choice().cloned().expect("qiaoshui3:add_target");
        assert_eq!(ch_add.spec.prompt, "qiaoshui3:add_target");
        let mut e = roundtrip_restore(&e);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(1).hand.is_empty() && e.state_core().player(2).hand.is_empty());

        // Negative (lose Pindian -> qiaoshui2 blocks Tricks this turn):
        let mut el = start_2p(2026_9203);
        el.debug_grant(0, "qiaoshui");
        el.debug_set_turn(0, Phase::Draw, 1);
        el.debug_clear_hand(0);
        el.debug_clear_hand(1);
        let _lo = place_suit_rank_any(&mut el, 0, Suit::Heart, 2, 4);
        let _hi = place_suit_rank_any(&mut el, 1, Suit::Spade, 11, 13);
        let trk = place_hand_kind_any(&mut el, 0, CardKind::Dismantle);
        let _ = submit(&mut el, 0, GameAction::NextPhase);
        let _ = answer(&mut el, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut el, 8);
        assert!(el.catalog().skill_by_name("qiaoshui2").is_some_and(|d| el.state_core().has_skill(0, d)));
        assert!(matches!(
            submit(&mut el, 0, GameAction::PlayCard { card: trk, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ), "qiaoshui2 must prevent using trick cards");

        // Negative (`qiaoshui3` is non-locked: non-locked seal must disable qiaoshui3:add_target):
        let mut es = start_3p(2026_9216);
        es.debug_grant(0, "qiaoshui");
        es.debug_set_turn(0, Phase::Draw, 1);
        es.debug_clear_hand(0);
        es.debug_clear_hand(1);
        let ks = place_suit_rank_any(&mut es, 0, Suit::Spade, 12, 13);
        let _lo0s = place_suit_rank_any(&mut es, 0, Suit::Heart, 2, 4);
        let _l1 = place_suit_rank_any(&mut es, 1, Suit::Heart, 2, 4);
        let _ = submit(&mut es, 0, GameAction::NextPhase);
        let _ = answer(&mut es, 0, vec![ChoiceOption::Player(1)]);
        let _ = answer(&mut es, 0, vec![ChoiceOption::Card(ks)]);
        drain_pass(&mut es, 8);
        assert!(es.catalog().skill_by_name("qiaoshui3").is_some_and(|d| es.state_core().has_skill(0, d)));
        es.debug_grant(0, "txtanbei_effect1");
        let dis_s = place_hand_kind_any(&mut es, 0, CardKind::Dismantle);
        let _h1s = place_hand_kind_any(&mut es, 1, CardKind::Shan);
        let _ = submit(&mut es, 0, GameAction::PlayCard { card: dis_s, targets: vec![1] });
        if let Some(ch_s) = es.open_choice() {
            assert_ne!(ch_s.spec.prompt, "qiaoshui3:add_target", "non-locked seal must disable qiaoshui3");
        }
        drain_pass(&mut es, 16);
    }

    // 14. `qice`: discards all hand cards once per phase; single-target clamped [1, 1], Chain [1, 2], SavageAttack AOE.
    {
        let mut e = start_3p(2026_9204);
        e.debug_grant(0, "qice");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let _h1 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _h2 = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let hp1 = e.state_core().player(1).hp;
        let hp2 = e.state_core().player(2).hp;
        let res = activate(&mut e, 0, "qice");
        assert!(!matches!(res, CommandOutcome::Rejected(_)));
        let ch_tr = e.open_choice().cloned().expect("qice:choose_trick");
        assert_eq!(ch_tr.spec.prompt, "qice:choose_trick");
        // Tag 3 = Duel (or find SavageAttack tag in QICE_KINDS: index 4 = SavageAttack!)
        let mut e = roundtrip_restore(&e);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(4)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).hand.is_empty(), "qice discards entire hand");
        assert_eq!(e.state_core().player(1).hp, hp1 - 1);
        assert_eq!(e.state_core().player(2).hp, hp2 - 1);
        // Empty hand / second time in same phase -> rejected!
        let _h3 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        assert!(matches!(activate(&mut e, 0, "qice"), CommandOutcome::Rejected(_)));

        // Single-target trick via qice with 3 hand cards MUST clamp target count to [1, 1] (and Chain [1, 2])!
        let mut eq = start_3p(2026_9217);
        eq.debug_grant(0, "qice");
        advance_to_phase(&mut eq, 0, Phase::Play, None);
        eq.debug_clear_hand(0);
        let _qa = place_hand_kind_any(&mut eq, 0, CardKind::Sha);
        let _qb = place_hand_kind_any(&mut eq, 0, CardKind::Shan);
        let _qc = place_hand_kind_any(&mut eq, 0, CardKind::Tao);
        let _ = activate(&mut eq, 0, "qice");
        // Tag 3 = Duel (single-target trick in QICE_KINDS: Wuzhong,Taoyuan,Wugu,Duel,SavageAttack,ArcheryAttack,Dismantle,Snatch,Huogong,Chain,Jiedao)
        let _ = answer(&mut eq, 0, vec![ChoiceOption::Tag(3)]);
        let ch_qt = eq.open_choice().cloned().expect("qice:choose_targets for Duel");
        assert_eq!(ch_qt.spec.prompt, "qice:choose_targets");
        assert_eq!((ch_qt.spec.min, ch_qt.spec.max), (1, 1), "qice single-target trick must clamp targets to [1, 1]");
        assert!(matches!(
            submit(
                &mut eq,
                0,
                GameAction::Answer {
                    choice_id: ch_qt.id,
                    selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
                }
            ),
            CommandOutcome::Rejected(RejectReason::InvalidChoice)
        ));
        let _ = answer(&mut eq, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut eq, 16);

        // Chain via qice retains [1, 2] target range!
        let mut eqc = start_3p(2026_9219);
        eqc.debug_grant(0, "qice");
        advance_to_phase(&mut eqc, 0, Phase::Play, None);
        eqc.debug_clear_hand(0);
        let _qca = place_hand_kind_any(&mut eqc, 0, CardKind::Sha);
        let _qcb = place_hand_kind_any(&mut eqc, 0, CardKind::Shan);
        let _ = activate(&mut eqc, 0, "qice");
        let _ = answer(&mut eqc, 0, vec![ChoiceOption::Tag(9)]); // Chain!
        let ch_qc = eqc.open_choice().cloned().expect("qice:choose_targets for Chain");
        assert_eq!((ch_qc.spec.min, ch_qc.spec.max), (1, 2), "qice Chain must retain [1, 2] targets");
        let _ = answer(&mut eqc, 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
        drain_pass(&mut eqc, 16);
        assert!(eqc.state_core().player(1).linked && eqc.state_core().player(2).linked);
    }

    // 15. `ranshang` + `ranshang2`: fire damage adds marks -> Phase::End loses X HP; X > 2 also loses 2 max_hp + draws 2.
    {
        let mut e = start_2p(2026_9205);
        e.debug_grant(1, "ranshang");
        e.debug_set_max_hp(1, 5);
        e.debug_set_hp(1, 5);
        e.debug_deal_damage(Some(0), 1, 3, DamageNature::Fire);
        drain_pass(&mut e, 16);
        assert_eq!(flag(&e, 1, "l11c_ranshang_marks"), 3);
        assert!(e.catalog().skill_by_name("ranshang2").is_some_and(|d| e.state_core().has_skill(1, d)));
        e.debug_set_hp(1, 5);
        e.debug_clear_hand(1);
        advance_to_phase(&mut e, 1, Phase::End, None);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 2, "ranshang2 lost 3 HP");
        assert_eq!(e.state_core().player(1).max_hp, 3, "ranshang2 >2 marks lost 2 max_hp");
        assert!(e.state_core().player(1).hand.len() >= 2, "ranshang2 >2 marks drew 2");
    }

    // 16. `rekuangcai`: Discard begin without card used/damage adds/subtracts hand limit; End draws min(5, dmg).
    {
        let mut e = start_2p(2026_9206);
        e.debug_grant(0, "rekuangcai");
        advance_to_phase(&mut e, 0, Phase::Discard, None);
        drain_pass(&mut e, 16);
        assert_eq!(flag(&e, 0, "l11c_rekuangcai_pos"), 1, "no card used in turn grants +1 hand limit mark");
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), e.state_core().player(0).hp as usize + 1);
        // Next turn deal 2 Sha dmg -> End draws 2!
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_grant(0, "new_repaoxiao");
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let s1 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let s2 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: s1, targets: vec![1] });
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: s2, targets: vec![1] });
        drain_pass(&mut e, 16);
        let h_before_end = e.state_core().player(0).hand.len();
        advance_to_phase(&mut e, 0, Phase::End, None);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(0).hand.len(), h_before_end + 2, "rekuangcai End draws turn damage dealt");
    }
}

#[test]
fn l11_vm_c_fix27_core_b_relianji_to_zhuikong() {
    // 17. `relianji`: once per Play phase selects 2 others -> both draw 1 & deals virtual card.
    {
        let mut e = start_3p(2026_9207);
        e.debug_grant(0, "relianji");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let res = activate(&mut e, 0, "relianji");
        assert!(!matches!(res, CommandOutcome::Rejected(_)));
        let ch = e.open_choice().cloned().expect("relianji:choose_targets");
        assert_eq!(ch.spec.prompt, "relianji:choose_targets");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
        drain_pass(&mut e, 16);
        assert!(matches!(activate(&mut e, 0, "relianji"), CommandOutcome::Rejected(_)), "once per Play phase");
    }

    // 18. `reshanxi` + `reshanxi2`: PlayBegin discards red basic -> exiles up to hp cards into reshanxi2, returns at End!
    {
        let mut e = start_2p(2026_9208);
        e.debug_grant(0, "reshanxi");
        e.debug_set_hp(0, 2);
        e.debug_set_turn(0, Phase::Draw, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let rb = place_card_suit_rank(&mut e, 0, Suit::Heart, CardKind::Sha, 1, 13);
        let t1 = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let t2 = place_hand_kind_any(&mut e, 1, CardKind::Shan);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let ch_c = e.open_choice().cloned().expect("reshanxi:choose_cost");
        assert_eq!(ch_c.spec.prompt, "reshanxi:choose_cost");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(rb)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let ch_ex = e.open_choice().cloned().expect("reshanxi:exile_cards");
        assert_eq!(ch_ex.spec.max, 2);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(t1), ChoiceOption::Card(t2)]);
        drain_pass(&mut e, 16);
        assert_eq!(pile(&e, 1, "reshanxi2").len(), 2);
        assert!(e.state_core().player(1).hand.is_empty());
        // End of turn returns cards via reshanxi2.
        advance_to_phase(&mut e, 1, Phase::Judge, None);
        drain_pass(&mut e, 16);
        assert_eq!(pile(&e, 1, "reshanxi2").len(), 0);
        assert!(e.state_core().player(1).hand.contains(&t1) && e.state_core().player(1).hand.contains(&t2));
    }

    // 19. `rezhenxing`: End/Damage peeks top 3 -> unique-suit pick + AI empty pick when max == 0.
    {
        let mut e = start_2p(2026_9209);
        e.debug_grant(0, "rezhenxing");
        e.debug_set_turn(0, Phase::Discard, 1);
        e.debug_clear_hand(0);
        let s_u = move_to_deck_top_suit_rank(&mut e, Suit::Spade, 1, 13);
        let h_1 = move_to_deck_top_suit_rank(&mut e, Suit::Heart, 1, 13);
        let h_2 = move_to_deck_top_suit_rank(&mut e, Suit::Heart, 1, 13);
        let _ = (h_1, h_2);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let ch = e.open_choice().cloned().expect("rezhenxing:confirm");
        assert_eq!(ch.spec.prompt, "rezhenxing:confirm");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        let ch_p = e.open_choice().cloned().expect("rezhenxing:pick_card");
        assert_eq!(ch_p.spec.options, vec![ChoiceOption::Card(s_u)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(s_u)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).hand.contains(&s_u), "rezhenxing gained strictly unique-suit card");

        // Negative: all 3 cards same suit -> max == 0, AI returns empty vec!
        let mut e0 = start_2p(2026_9210);
        e0.debug_grant(0, "rezhenxing");
        e0.debug_set_turn(0, Phase::Discard, 1);
        let _ = move_to_deck_top_suit_rank(&mut e0, Suit::Club, 1, 13);
        let _ = move_to_deck_top_suit_rank(&mut e0, Suit::Club, 1, 13);
        let _ = move_to_deck_top_suit_rank(&mut e0, Suit::Club, 1, 13);
        let _ = submit(&mut e0, 0, GameAction::NextPhase);
        let _ = answer(&mut e0, 0, vec![ChoiceOption::Confirm]);
        let ch0 = e0.open_choice().cloned().expect("rezhenxing:pick_card empty max");
        assert_eq!(ch0.spec.max, 0);
        let ai_act = plan_ai_answer(&e0, 0);
        assert_eq!(ai_act, GameAction::Answer { choice_id: ch0.id, selection: Vec::new() });
        let _ = submit(&mut e0, 0, ai_act);
        drain_pass(&mut e0, 16);
    }

    // 20. `twjiange`: once per turn non-basic as unlimited-dist/no-count Sha; Duel respond draws 1 off-turn.
    {
        let mut e = start_3p(2026_9211);
        e.debug_grant(0, "twjiange");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(2);
        let nb = place_hand_kind_any(&mut e, 0, CardKind::Dismantle);
        equip_kind_suit(&mut e, 0, CardKind::HorsePlus, None);
        equip_kind_suit(&mut e, 2, CardKind::HorsePlus, None);
        let res = activate(&mut e, 0, "twjiange");
        assert!(!matches!(res, CommandOutcome::Rejected(_)));
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(nb)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(2).hp, 3, "twjiange non-basic Sha ignores distance");
        assert!(!e.state_core().turn.sha_used_play_phase, "twjiange Sha has no use count");
    }

    // 21. `xinniluan` + 22. `xswuyan` + 23. `zhuikong` + `zhuikong_distance`.
    {
        // xinniluan: off-turn End after seat 1 used card targeting seat 0 (including Taoyuan with empty targets) -> seat 0 uses Sha!
        let mut e = start_2p(2026_9212);
        e.debug_grant(0, "xinniluan");
        advance_to_phase(&mut e, 1, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let keep0 = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let sha0 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let dis1 = place_hand_kind_any(&mut e, 1, CardKind::Dismantle);
        let keep1 = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: dis1, targets: vec![0] });
        if let Some(ch_d) = e.open_choice().cloned() {
            if ch_d.spec.options.contains(&ChoiceOption::Card(keep0)) {
                let _ = answer(&mut e, ch_d.observer, vec![ChoiceOption::Card(keep0)]);
            }
        }
        drain_pass(&mut e, 8);
        advance_to_phase(&mut e, 1, Phase::End, Some("xinniluan:"));
        let _ch = e.open_choice().cloned().expect("xinniluan:choose_card");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(sha0)]);
        drain_pass(&mut e, 4);
        if e.open_choice().is_some_and(|c| c.spec.prompt == "xinniluan:discard_target") {
            let _ = answer(&mut e, 0, vec![ChoiceOption::Card(keep1)]);
        }
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 3, "xinniluan dealt 1 Sha dmg");

        // xinniluan after Taoyuan (CardUsed has empty targets in play_card) still triggers!
        let mut ety = start_2p(2026_9218);
        ety.debug_grant(0, "xinniluan");
        ety.debug_set_hp(0, 3);
        advance_to_phase(&mut ety, 1, Phase::Play, None);
        ety.debug_clear_hand(0);
        ety.debug_clear_hand(1);
        let c_nl = place_hand_kind_any(&mut ety, 0, CardKind::Sha);
        let ty = place_hand_kind_any(&mut ety, 1, CardKind::Taoyuan);
        let _ = submit(&mut ety, 1, GameAction::PlayCard { card: ty, targets: vec![] });
        drain_pass(&mut ety, 8);
        advance_to_phase(&mut ety, 1, Phase::End, Some("xinniluan:"));
        let ch_ty = ety.open_choice().cloned().expect("xinniluan must trigger after Taoyuan targeted owner");
        assert_eq!(ch_ty.spec.prompt, "xinniluan:choose_card");
        let _ = answer(&mut ety, 0, vec![ChoiceOption::Card(c_nl)]);
        drain_pass(&mut ety, 16);
        assert_eq!(ety.state_core().player(1).hp, 3, "xinniluan after Taoyuan dealt 1 Sha dmg");

        // xswuyan: trick card by/targeting owner (player != target) cannot take effect.
        let mut ew = start_2p(2026_9213);
        ew.debug_grant(1, "xswuyan");
        advance_to_phase(&mut ew, 0, Phase::Play, None);
        ew.debug_clear_hand(0);
        ew.debug_clear_hand(1);
        let nan = place_hand_kind_any(&mut ew, 0, CardKind::SavageAttack);
        let _ = submit(&mut ew, 0, GameAction::PlayCard { card: nan, targets: vec![] });
        drain_pass(&mut ew, 16);
        assert_eq!(ew.state_core().player(1).hp, 4, "xswuyan blocks trick targeting owner");

        // zhuikong: owner wounded + Pindian win -> target zishou2; lose -> zhuikong_distance unlimited!
        let mut ez = start_3p(2026_9214);
        ez.debug_grant(0, "zhuikong");
        ez.debug_set_hp(0, 2);
        ez.debug_clear_hand(0);
        ez.debug_clear_hand(1);
        let _hi0 = place_suit_rank_any(&mut ez, 0, Suit::Spade, 11, 13);
        let _lo1 = place_suit_rank_any(&mut ez, 1, Suit::Heart, 2, 4);
        ez.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut ez, 0, GameAction::NextPhase); // -> seat 1 Prepare triggers zhuikong!
        let ch_z = ez.open_choice().cloned().expect("zhuikong:confirm");
        assert_eq!(ch_z.spec.prompt, "zhuikong:confirm");
        let _ = answer(&mut ez, 0, vec![ChoiceOption::Confirm]);
        drain_pass(&mut ez, 16);
        assert!(ez.catalog().skill_by_name("zishou2").is_some_and(|d| ez.state_core().has_skill(1, d)));
        advance_to_phase(&mut ez, 1, Phase::Play, None);
        let s_ill = place_hand_kind_any(&mut ez, 1, CardKind::Sha);
        assert!(matches!(
            submit(&mut ez, 1, GameAction::PlayCard { card: s_ill, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ), "zhuikong win imposes zishou2 blocking targeting others");

        // zhuikong lose -> target distance to owner unlimited (`zhuikong_distance`)!
        let mut ezl = start_3p(2026_9215);
        ezl.debug_grant(0, "zhuikong");
        ezl.debug_set_hp(0, 2);
        equip_kind_suit(&mut ezl, 0, CardKind::HorsePlus, None);
        ezl.debug_clear_hand(0);
        ezl.debug_clear_hand(1);
        let _lo0 = place_suit_rank_any(&mut ezl, 0, Suit::Heart, 2, 4);
        let _hi1 = place_suit_rank_any(&mut ezl, 1, Suit::Spade, 11, 13);
        ezl.debug_set_turn(0, Phase::End, 1);
        let _ = submit(&mut ezl, 0, GameAction::NextPhase);
        let _ = answer(&mut ezl, 0, vec![ChoiceOption::Confirm]);
        drain_pass(&mut ezl, 16);
        assert_eq!(ezl.state_core().computed_distance(ezl.catalog(), 1, 0), 1, "zhuikong_distance sets dist=1");
    }
}
