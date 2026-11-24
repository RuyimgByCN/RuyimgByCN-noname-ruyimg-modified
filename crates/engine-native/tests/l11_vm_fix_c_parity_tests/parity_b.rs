use super::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn l11_vm_c_audit36_parity_b_rezhiheng_to_yizhong() {
    {
        let e = start_2p(2026_9601);
        assert_is_vm_c(
            &e,
            &[
                "rezhiheng", "shelie", "shibei", "twkujian", "xiandeng", "xinanjian", "xinkuanggu",
                "xiongyi", "yizhong",
            ],
        );
    }

    // 10. `rezhiheng`: once per Play discard N -> draw N, +1 if ALL hand cards discarded!
    {
        let mut e = start_2p(2026_9602);
        e.debug_grant(0, "rezhiheng");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        let h1 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let h2 = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let eq = equip_kind_suit(&mut e, 0, CardKind::Armor, None);
        let _ = eq;
        let _ = activate(&mut e, 0, "rezhiheng");
        let ch = e.open_choice().cloned().expect("rezhiheng:choose_cards");
        assert_eq!(ch.spec.prompt, "rezhiheng:choose_cards");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(h1), ChoiceOption::Card(h2)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(0).hand.len(), 3, "discarding ALL 2 hand cards draws 2 + 1 = 3");
        assert!(matches!(activate(&mut e, 0, "rezhiheng"), CommandOutcome::Rejected(_)), "once per Play phase");
    }

    // 11. `shelie`: DrawPhaseBegin1 reveals 5, validates distinct suits, restores cleanly, skips normal draw!
    {
        let mut e = start_2p(2026_9603);
        e.debug_grant(0, "shelie");
        e.debug_clear_hand(0);
        let c1 = move_to_deck_top_suit_rank(&mut e, Suit::Spade, 2, 2);
        let c2 = move_to_deck_top_suit_rank(&mut e, Suit::Spade, 3, 3);
        let c3 = move_to_deck_top_suit_rank(&mut e, Suit::Heart, 4, 4);
        let c4 = move_to_deck_top_suit_rank(&mut e, Suit::Club, 5, 5);
        let c5 = move_to_deck_top_suit_rank(&mut e, Suit::Diamond, 6, 6);
        let _ = (c1, c2, c3, c4, c5);
        e.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let _ch_conf = e.open_choice().cloned().expect("optional shelie confirm");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        let ch_p = e.open_choice().cloned().expect("shelie:pick_card");
        assert_eq!(ch_p.spec.prompt, "shelie:pick_card");
        assert_eq!(ch_p.spec.min, 4);
        // Reject duplicate suit (c1+c2 both Spade)!
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::Answer {
                    choice_id: ch_p.id,
                    selection: vec![
                        ChoiceOption::Card(c1),
                        ChoiceOption::Card(c2),
                        ChoiceOption::Card(c3),
                        ChoiceOption::Card(c4),
                    ],
                }
            ),
            CommandOutcome::Rejected(RejectReason::InvalidChoice)
        ));
        let mut e = roundtrip_restore(&e);
        let ai_act = plan_ai_answer(&e, 0);
        let _ = submit(&mut e, 0, ai_act);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(0).hand.len(), 4, "shelie picked 4 distinct suits cleanly");
    }

    // 12. `shibei`: 1st damage in turn heals 1 HP; >= 2nd damage in same turn loses 1 HP.
    {
        let mut e = start_2p(2026_9604);
        e.debug_grant(1, "shibei");
        e.debug_grant(0, "new_repaoxiao");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let s1 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let s2 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: s1, targets: vec![1] });
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(1).hp, 4, "shibei 1st damage in turn recovers 1 HP (4 - 1 + 1 = 4)");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: s2, targets: vec![1] });
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(1).hp, 2, "shibei 2nd damage in turn loses 1 extra HP (4 - 1 - 1 = 2)");
    }

    // 13. `twkujian`: Play gives up to 3 marked cards -> PlayCard triggers owner+target draw 1, discard triggers both discard 1!
    {
        let mut e = start_2p(2026_9605);
        e.debug_grant(0, "twkujian");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let kj1 = place_hand_kind_any(&mut e, 0, CardKind::Tao);
        let kj2 = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let _ = activate(&mut e, 0, "twkujian");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(kj1), ChoiceOption::Card(kj2)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut e, 8);
        assert!(e.state_core().player(1).hand.contains(&kj1) && e.state_core().player(1).hand.contains(&kj2));

        advance_to_phase(&mut e, 1, Phase::Play, None);
        e.debug_set_hp(1, 3);
        let h0_pre = e.state_core().player(0).hand.len();
        let h1_pre = e.state_core().player(1).hand.len();
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: kj1, targets: vec![] });
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(0).hand.len(), h0_pre + 1, "twkujian drew 1 for owner on PlayCard");
        assert_eq!(e.state_core().player(1).hand.len(), h1_pre, "twkujian drew 1 for user (-1 played +1 drawn = net 0)");
    }

    // 14. `xiandeng` + 15. `xinanjian` + 16. `xinkuanggu` + 17. `xiongyi` + 18. `yizhong`:
    {
        // xiandeng: +1 Sha limit AND 1st Sha in turn ignores distance, 2nd does NOT ignore distance.
        let mut ex = start_3p(2026_9606);
        ex.debug_grant(0, "xiandeng");
        equip_kind_suit(&mut ex, 0, CardKind::HorsePlus, None);
        equip_kind_suit(&mut ex, 2, CardKind::HorsePlus, None);
        advance_to_phase(&mut ex, 0, Phase::Play, None);
        ex.debug_clear_hand(0);
        ex.debug_clear_hand(1);
        ex.debug_clear_hand(2);
        let s1 = place_hand_kind_any(&mut ex, 0, CardKind::Sha);
        let s2 = place_hand_kind_any(&mut ex, 0, CardKind::Sha);
        let _ = submit(&mut ex, 0, GameAction::PlayCard { card: s1, targets: vec![2] });
        drain_pass(&mut ex, 8);
        assert_eq!(ex.state_core().player(2).hp, 3, "xiandeng 1st Sha ignores distance");
        assert!(matches!(
            submit(&mut ex, 0, GameAction::PlayCard { card: s2, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ), "xiandeng 2nd Sha does NOT ignore distance");

        // xinanjian + xinkuanggu + yizhong:
        let mut ey = start_3p(2026_9607);
        ey.debug_grant(0, "xinanjian");
        ey.debug_grant(0, "xinkuanggu");
        ey.debug_grant(1, "yizhong");
        equip_kind_suit(&mut ey, 0, CardKind::HorsePlus, None);
        advance_to_phase(&mut ey, 0, Phase::Play, None);
        ey.debug_clear_hand(0);
        ey.debug_clear_hand(1);
        ey.debug_clear_hand(2);
        // Black Sha against yizhong (no armor!) MUST cancel!
        let blk_sha = place_card_suit_rank(&mut ey, 0, Suit::Spade, CardKind::Sha, 1, 13);
        let hp1 = ey.state_core().player(1).hp;
        let _ = submit(&mut ey, 0, GameAction::PlayCard { card: blk_sha, targets: vec![1] });
        drain_pass(&mut ey, 16);
        assert_eq!(ey.state_core().player(1).hp, hp1, "yizhong cancelled black Sha when Armor slot empty");
        // Red Sha against seat 2 (dist=1 for seat 0->2, but target dist=2 without range -> xinanjian +1 dmg + xinkuanggu!)
        ey.debug_grant(0, "new_repaoxiao");
        ey.debug_set_hp(0, 2);
        let red_sha = place_card_suit_rank(&mut ey, 0, Suit::Heart, CardKind::Sha, 1, 13);
        let hp2 = ey.state_core().player(2).hp;
        let _ = submit(&mut ey, 0, GameAction::PlayCard { card: red_sha, targets: vec![2] });
        let ch_xa = ey.open_choice().cloned().expect("xinanjian:choose_mode");
        assert_eq!(ch_xa.spec.prompt, "xinanjian:choose_mode");
        assert_eq!((ch_xa.spec.min, ch_xa.spec.allow_cancel), (1, false));
        let _ = answer(&mut ey, 0, vec![ChoiceOption::Tag(1)]); // +1 damage!
        let ch_kg = ey.open_choice().cloned().expect("xinkuanggu:choose_mode");
        assert_eq!(ch_kg.spec.prompt, "xinkuanggu:choose_mode");
        assert_eq!((ch_kg.spec.min, ch_kg.spec.allow_cancel), (0, true), "xinkuanggu must be cancellable (min=0)");
        let _ = answer(&mut ey, 0, vec![ChoiceOption::Tag(1)]); // recover 1
        let ch_kg2 = ey.open_choice().cloned().expect("xinkuanggu 2nd trigger for 2 dmg");
        assert_eq!((ch_kg2.spec.min, ch_kg2.spec.allow_cancel), (0, true));
        let hp_pre_cancel = ey.state_core().player(0).hp;
        let hand_pre_cancel = ey.state_core().player(0).hand.len();
        let _ = answer(&mut ey, 0, Vec::new()); // decline 2nd trigger!
        drain_pass(&mut ey, 16);
        assert_eq!(ey.state_core().player(2).hp, hp2 - 2, "xinanjian mode 1 dealt +1 dmg");
        assert_eq!(ey.state_core().player(0).hp, hp_pre_cancel, "cancelling xinkuanggu must NOT recover");
        assert_eq!(ey.state_core().player(0).hand.len(), hand_pre_cancel, "cancelling xinkuanggu must NOT draw");

        // `xiongyi`: L0 limited Play active -> targets draw 3 + owner recovers 1 if min HP!
        let mut exy = start_3p(2026_9608);
        exy.debug_grant(0, "xiongyi");
        exy.debug_set_hp(0, 1);
        advance_to_phase(&mut exy, 0, Phase::Play, None);
        exy.debug_clear_hand(0);
        exy.debug_clear_hand(1);
        let _ = activate(&mut exy, 0, "xiongyi");
        let _ = answer(&mut exy, 0, vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]);
        drain_pass(&mut exy, 16);
        assert_eq!(exy.state_core().player(0).hand.len(), 3);
        assert_eq!(exy.state_core().player(1).hand.len(), 3);
        assert_eq!(exy.state_core().player(0).hp, 2, "xiongyi owner at min HP recovers 1");
        assert!(matches!(activate(&mut exy, 0, "xiongyi"), CommandOutcome::Rejected(_)), "xiongyi is L0 limited once per game");
        let _ = EquipSlot::Armor;
    }
}
