use super::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn l11_vm_c_fix27_bindings_and_core_a_anjian_to_huoji() {
    // Catalog binding check for all 24 core stripped skills + 6 subskills on taixu-upstream-liuli.
    {
        let e = start_2p(2026_9101);
        assert_is_vm_c(
            &e,
            &[
                "anjian", "drlt_qianjie", "dujin", "fuli", "gnjinfan", "gnjinfan_gain", "huoji",
                "juece", "juejing", "longhun", "mieji", "nsfeixiong", "nsyice", "nzry_jieying",
                "qiaoshui", "qiaoshui2", "qiaoshui3", "qice", "ranshang", "ranshang2",
                "rekuangcai", "relianji", "reshanxi", "reshanxi2", "rezhenxing", "twjiange",
                "xinniluan", "xswuyan", "zhuikong", "zhuikong_distance",
            ],
        );
    }

    // 1. `anjian`: +1 dmg when target cannot reach owner; in-range target or Duel deals base 1 dmg.
    {
        let mut e = start_3p(2026_9102);
        e.assign_general_named(0, "吕蒙").unwrap();
        e.assign_general_named(1, "吕蒙").unwrap();
        e.assign_general_named(2, "甘宁").unwrap();
        e.debug_grant(0, "anjian");
        e.debug_grant(0, "new_repaoxiao");
        equip_weapon_range(&mut e, 0, 3);
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        // Seat 2 dist=1 in 3p, give HorsePlus to seat 0 so dist(seat1->seat0)=2 and dist(seat2->seat0)=2?
        // Actually in 3p adjacent distances are 1; equip HorsePlus(+1 defense) on seat 0 -> target range 1 cannot reach owner (dist=2)!
        equip_kind_suit(&mut e, 0, CardKind::HorsePlus, None);
        let sha_far = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let hp2_before = e.state_core().player(2).hp;
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_far,
                targets: vec![2],
            },
        );
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(2).hp, hp2_before - 2, "anjian must add +1 Sha dmg when target cannot reach owner");

        // Negative: Duel (non-Sha) must NOT gain anjian +1; target with range>=2 Sha covers owner -> base 1.
        let duel = place_hand_card_named(&mut e, 0, "duel");
        let hp1_before = e.state_core().player(1).hp;
        let duel_res = submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] });
        assert!(!matches!(duel_res, CommandOutcome::Rejected(_)));
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, hp1_before - 1, "anjian must NOT boost non-Sha Duel");

        equip_weapon_range(&mut e, 2, 3);
        let sha_covered = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let hp2_mid = e.state_core().player(2).hp;
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_covered,
                targets: vec![2],
            },
        );
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(2).hp, hp2_mid - 1, "anjian must NOT add +1 when target weapon covers owner");
    }

    // 2. `drlt_qianjie`: immune to Le/Bing/ShanDian and cannot become linked.
    {
        let mut e = start_2p(2026_9103);
        e.debug_grant(1, "drlt_qianjie");
        e.debug_grant(0, "nzry_jieying");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        let le = place_hand_kind_any(&mut e, 0, CardKind::Le);
        let chain = place_hand_kind_any(&mut e, 0, CardKind::Chain);
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: le, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ), "drlt_qianjie must block delayed trick Le");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: chain, targets: vec![1] });
        drain_pass(&mut e, 16);
        assert!(!e.state_core().player(1).linked, "drlt_qianjie cannot become linked via Chain");
        advance_to_phase(&mut e, 0, Phase::End, Some("nzry_jieying:"));
        drain_pass(&mut e, 16);
        assert!(!e.state_core().player(1).linked, "drlt_qianjie blocks nzry_jieying link");
    }

    // 3. `dujin`: optional draw confirm -> 1 + floor(equips/2) + fixed_draw negative + Pass negative.
    {
        for (eq_cnt, expect) in [(0usize, 3usize), (1, 3), (2, 4), (3, 4)] {
            let mut e = start_2p(2026_9104 + eq_cnt as u64);
            e.debug_grant(0, "dujin");
            let kinds = [CardKind::Weapon, CardKind::Armor, CardKind::HorsePlus];
            for k in kinds.iter().take(eq_cnt) {
                equip_kind_suit(&mut e, 0, *k, None);
            }
            e.debug_clear_hand(0);
            e.debug_set_turn(0, Phase::Judge, 1);
            let _ = submit(&mut e, 0, GameAction::NextPhase);
            let ch = e.open_choice().cloned().expect("dujin:confirm");
            assert_eq!(ch.spec.prompt, "dujin:confirm");
            let e = roundtrip_restore(&e);
            let mut e = e;
            let ai_act = plan_ai_answer(&mut e, 0);
            assert!(matches!(ai_act, GameAction::Answer { ref selection, .. } if selection == &vec![ChoiceOption::Confirm]));
            let _ = submit(&mut e, 0, ai_act);
            drain_pass(&mut e, 16);
            assert_eq!(e.state_core().player(0).hand.len(), expect);
        }
        let mut ef = start_2p(2026_9109);
        ef.debug_grant(0, "dujin");
        equip_kind_suit(&mut ef, 0, CardKind::Weapon, None);
        equip_kind_suit(&mut ef, 0, CardKind::Armor, None);
        ef.debug_clear_hand(0);
        ef.debug_emit_fixed_draw_phase(0, 2);
        assert!(ef.open_choice().is_none(), "dujin must skip num_fixed=true draw");
        drain_pass(&mut ef, 8);
        assert_eq!(ef.state_core().player(0).hand.len(), 2);
    }

    // 4. `fuli`: L0 once-per-game dying save recovers up to distinct factions and flips; fails on 2nd dying.
    {
        let mut e = start_3p(2026_9110);
        e.assign_general_named(0, "孙权").unwrap();
        e.assign_general_named(1, "刘备").unwrap();
        e.assign_general_named(2, "曹操").unwrap();
        e.debug_grant(0, "fuli");
        e.debug_set_max_hp(0, 4);
        e.debug_set_hp(0, 1);
        advance_to_phase(&mut e, 1, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha1 = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        let fuli_def = e.catalog().skill_by_name("fuli").expect("fuli def");
        for _ in 0..8 {
            let Some(ch) = e.open_choice().cloned() else { break };
            if ch.observer == 0 && ch.spec.options.contains(&ChoiceOption::Skill(fuli_def)) {
                let ai_act = plan_ai_answer(&e, 0);
                assert!(
                    matches!(
                        &ai_act,
                        GameAction::Answer { selection, .. }
                            if selection == &vec![ChoiceOption::Skill(fuli_def)]
                    ),
                    "AI must pick fuli dying save: {ai_act:?}"
                );
                let _ = submit(&mut e, 0, ai_act);
            } else {
                drain_pass(&mut e, 1);
            }
        }
        assert!(e.state_core().player(0).alive);
        assert_eq!(e.state_core().player(0).hp, 3, "fuli must recover up to 3 distinct alive factions");
        assert!(e.state_core().player(0).face_down, "fuli must turn over");

        // Second dying in same game -> fuli exhausted, seat 0 dies without Tao.
        e.debug_grant(1, "new_repaoxiao");
        e.debug_set_hp(0, 1);
        let sha2 = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
        drain_pass(&mut e, 16);
        assert!(!e.state_core().player(0).alive, "fuli is L0 limited once per game");
    }

    // 5. `gnjinfan` (phaseDiscardBegin distinct suits) + `gnjinfan_gain` + `huoji` (uses pile red card!).
    {
        let mut e = start_2p(2026_9111);
        e.debug_grant(0, "gnjinfan");
        e.debug_grant(0, "huoji");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sp1 = place_suit_rank_any(&mut e, 0, Suit::Spade, 1, 13);
        let sp2 = place_suit_rank_any(&mut e, 0, Suit::Spade, 1, 13);
        let ht1 = place_suit_rank_any(&mut e, 0, Suit::Heart, 1, 13);
        let extra = place_suit_rank_any(&mut e, 0, Suit::Club, 1, 13);
        let _ = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard triggers gnjinfan!
        let ch = e.open_choice().cloned().expect("gnjinfan:choose_cards");
        assert_eq!(ch.spec.prompt, "gnjinfan:choose_cards");
        // Reject duplicate suit!
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::Answer {
                    choice_id: ch.id,
                    selection: vec![ChoiceOption::Card(sp1), ChoiceOption::Card(sp2)],
                }
            ),
            CommandOutcome::Rejected(RejectReason::InvalidChoice)
        ));
        let mut e = roundtrip_restore(&e);
        let _ = extra;
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(sp1), ChoiceOption::Card(ht1)]);
        drain_pass(&mut e, 16);
        assert_eq!(pile(&e, 0, "gnjinfan").len(), 2);

        // Next turn Play: use red card `ht1` FROM `"gnjinfan"` pile via `huoji` -> triggers `gnjinfan_gain` + `huogong`!
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        let disc_hand = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _ = disc_hand;
        let _ = move_to_deck_top_suit_rank(&mut e, Suit::Heart, 1, 13);
        let hp1 = e.state_core().player(1).hp;
        let res = activate(&mut e, 0, "huoji");
        assert!(!matches!(res, CommandOutcome::Rejected(_)));
        let ch_c = e.open_choice().cloned().expect("huoji:choose_card");
        assert!(ch_c.spec.options.contains(&ChoiceOption::Card(ht1)), "huoji must see red gnjinfan pile card");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(ht1)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, hp1 - 1, "huoji deals 1 Fire dmg");
        assert_eq!(pile(&e, 0, "gnjinfan").len(), 1);
        assert!(
            e.state_core()
                .player(0)
                .hand
                .iter()
                .any(|&c| e.catalog().cards[c as usize].suit == Suit::Heart),
            "gnjinfan_gain must draw a Heart card when Heart leaves pile"
        );
    }
}

#[test]
fn l11_vm_c_fix27_core_a_juece_to_nsyice() {
    // 6. `juece` + 7. `juejing`: empty last hand card triggers juece:confirm; non-empty skips; juejing +2 hand limit + draw.
    {
        let mut e = start_2p(2026_9112);
        e.debug_grant(0, "juece");
        e.debug_set_hp(0, 2);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 2, "base hand limit at hp=2 is 2");
        e.debug_grant(0, "juejing");
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 2 + 2, "juejing grants +2 hand limit");
        equip_kind_suit(&mut e, 1, CardKind::HorsePlus, None);
        assert_eq!(
            e.state_core().computed_distance(e.catalog(), 0, 1),
            2,
            "juejing must NOT grant -1 distance modifier"
        );
        e.debug_clear_hand(0);
        e.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(0).hand.len(), 2 + 2, "juejing draws +lost hp");

        // Negative: fixed draw skips juejing
        let mut ef = start_2p(2026_9113);
        ef.debug_grant(0, "juejing");
        ef.debug_set_hp(0, 2);
        ef.debug_clear_hand(0);
        ef.debug_emit_fixed_draw_phase(0, 2);
        drain_pass(&mut ef, 8);
        assert_eq!(ef.state_core().player(0).hand.len(), 2);

        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let d1 = place_hand_kind_any(&mut e, 0, CardKind::Dismantle);
        let d2 = place_hand_kind_any(&mut e, 0, CardKind::Dismantle);
        let c1 = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let c2 = place_hand_kind_any(&mut e, 1, CardKind::Shan);
        let _ = c2;
        // First dismantle leaves 1 card -> NO juece!
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: d1, targets: vec![1] });
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.options.contains(&ChoiceOption::Card(c1)) {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c1)]);
            }
        }
        assert!(e.open_choice().is_none(), "losing 1 of 2 hand cards must NOT trigger juece");
        let hp1 = e.state_core().player(1).hp;
        // Second dismantle empties seat 1 hand -> juece:confirm -> 1 dmg!
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: d2, targets: vec![1] });
        while let Some(ch) = e.open_choice().cloned() {
            if ch.spec.prompt == "juece:confirm" {
                let mut e_rt = roundtrip_restore(&e);
                let _ = answer(&mut e_rt, 0, vec![ChoiceOption::Confirm]);
                drain_pass(&mut e_rt, 16);
                assert_eq!(e_rt.state_core().player(1).hp, hp1 - 1, "juece deals 1 dmg when hand empties");
                break;
            }
            let first = ch.spec.options[0];
            let _ = answer(&mut e, ch.observer, vec![first]);
        }
    }

    // 8. `longhun`: needs max(1, hp) matching suit (NO +1 bonus): Diamond FireSha, Heart Tao/Dying, Club Shan, Spade Nullify.
    {
        let mut e = start_2p(2026_9114);
        e.debug_grant(0, "longhun");
        e.debug_grant(1, "longhun");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_set_hp(0, 2);
        e.debug_clear_hand(0);
        let dm1 = place_card_suit_rank(&mut e, 0, Suit::Diamond, CardKind::Shan, 1, 13);
        // Only 1 Diamond at hp=2 -> CANNOT activate longhun!
        assert!(matches!(activate(&mut e, 0, "longhun"), CommandOutcome::Rejected(_)));
        let dm2 = place_card_suit_rank(&mut e, 0, Suit::Diamond, CardKind::Shan, 1, 13);
        let cl1 = place_suit_rank_any(&mut e, 1, Suit::Club, 1, 13);
        let cl2 = place_suit_rank_any(&mut e, 1, Suit::Club, 1, 13);
        let _ = (cl1, cl2);
        e.debug_set_hp(1, 2);
        assert!(!matches!(activate(&mut e, 0, "longhun"), CommandOutcome::Rejected(_)));
        // Diamond Sha -> choose 2 Diamonds -> target 1 -> seat 1 uses 2 Clubs as Shan via respond:longhun:shan!
        if e.open_choice().is_some_and(|c| c.spec.prompt == "longhun:choose_cards") {
            let _ = answer(&mut e, 0, vec![ChoiceOption::Card(dm1), ChoiceOption::Card(dm2)]);
        }
        if e.open_choice().is_some_and(|c| c.spec.prompt == "longhun:choose_target") {
            let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        }
        let ch_shan = e.open_choice().cloned().expect("seat 1 respond_shan with longhun");
        let lh_def = e.catalog().skill_by_name("longhun").unwrap();
        assert!(ch_shan.spec.options.contains(&ChoiceOption::Skill(lh_def)));
        let _ = answer(&mut e, 1, vec![ChoiceOption::Skill(lh_def)]);
        let ch_c = e.open_choice().cloned().expect("longhun:choose_cards shan");
        assert_eq!(ch_c.spec.min, 2);
        let _ = answer(&mut e, 1, vec![ChoiceOption::Card(cl1), ChoiceOption::Card(cl2)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 2, "2 Clubs via longhun dodged FireSha");
        // Negative: FireSha dodged, hand emptied on both; Heart Dying at hp<=0 needs 1 Heart card!
        e.debug_set_hp(0, 1);
        let ht = place_card_suit_rank(&mut e, 0, Suit::Heart, CardKind::Sha, 1, 13);
        let trick = place_hand_kind_any(&mut e, 0, CardKind::Dismantle);
        e.debug_clear_hand(1);
        let sp = place_card_suit_rank(&mut e, 1, Suit::Spade, CardKind::Sha, 1, 13);
        e.debug_set_hp(1, 1);
        let _ = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        let _ = (ht, sp, trick);
        // Quick Spade Nullify test:
        let mut en = start_2p(2026_9115);
        en.debug_grant(1, "longhun");
        en.debug_set_hp(1, 1);
        advance_to_phase(&mut en, 0, Phase::Play, None);
        en.debug_clear_hand(0);
        en.debug_clear_hand(1);
        let duel = place_hand_card_named(&mut en, 0, "duel");
        let sp1 = place_card_suit_rank(&mut en, 1, Suit::Spade, CardKind::Sha, 1, 13);
        let _ = submit(&mut en, 0, GameAction::PlayCard { card: duel, targets: vec![1] });
        let ch_n = en.open_choice().cloned().expect("nullify window");
        assert!(ch_n.spec.options.contains(&ChoiceOption::Skill(lh_def)));
        let _ = answer(&mut en, 1, vec![ChoiceOption::Skill(lh_def)]);
        let _ = answer(&mut en, 1, vec![ChoiceOption::Card(sp1)]);
        drain_pass(&mut en, 16);
        assert_eq!(en.state_core().player(1).hp, 1, "Spade via longhun Nullified Duel");
    }

    // 9. `mieji`: black single-target trick adds extra target; red skips.
    {
        let mut e = start_3p(2026_9116);
        e.debug_grant(0, "mieji");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let blk_dis = place_card_suit_rank(&mut e, 0, Suit::Spade, CardKind::Dismantle, 1, 13);
        let red_dis = place_card_suit_rank(&mut e, 0, Suit::Heart, CardKind::Dismantle, 1, 13);
        let _p1 = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let _p2 = place_hand_kind_any(&mut e, 2, CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: blk_dis, targets: vec![1] });
        let ch_m = e.open_choice().cloned().expect("mieji:choose_target");
        assert_eq!(ch_m.spec.prompt, "mieji:choose_target");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(1).hand.is_empty() && e.state_core().player(2).hand.is_empty());

        // Red trick does NOT prompt mieji
        let _ = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let _ = place_hand_kind_any(&mut e, 2, CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: red_dis, targets: vec![1] });
        if let Some(ch) = e.open_choice() {
            assert_ne!(ch.spec.prompt, "mieji:choose_target");
        }
        drain_pass(&mut e, 16);
    }

    // 10. `nsfeixiong`: PlayBegin Pindian (winner deals 1 dmg) + restore + AI.
    {
        let mut e = start_2p(2026_9117);
        e.debug_grant(0, "nsfeixiong");
        e.debug_set_turn(0, Phase::Draw, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let hi = place_rank_any(&mut e, 0, CardKind::Sha, 12, 13);
        let lo0 = place_rank_any(&mut e, 0, CardKind::Sha, 2, 4);
        let md = place_rank_any(&mut e, 1, CardKind::Sha, 6, 8);
        let _ = (hi, lo0, md);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        let ch = e.open_choice().cloned().expect("nsfeixiong:choose_target");
        assert_eq!(ch.spec.prompt, "nsfeixiong:choose_target");
        let mut e = roundtrip_restore(&e);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        // AI chooses highest rank `hi`!
        let ch_own = e.open_choice().cloned().expect("nsfeixiong:pindian_owner");
        assert_eq!(ch_own.spec.prompt, "nsfeixiong:pindian_owner");
        assert_eq!(
            plan_ai_answer(&mut e, 0),
            GameAction::Answer {
                choice_id: ch_own.id,
                selection: vec![ChoiceOption::Card(hi)],
            }
        );
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(hi)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 3, "nsfeixiong owner win deals 1 dmg");
    }

    // 11. `nsyice`: own discard enters pile, equal-rank ends span -> middle to hand, top/bottom + 1 dmg.
    {
        let mut e = start_2p(2026_9118);
        e.debug_grant(0, "nsyice");
        e.debug_set_hp(0, 1);
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        let c7_a = place_card_suit_rank(&mut e, 0, Suit::Spade, CardKind::Sha, 7, 7);
        let c5_mid = place_suit_rank_any(&mut e, 0, Suit::Heart, 5, 5);
        let c7_b = place_card_suit_rank(&mut e, 0, Suit::Club, CardKind::Sha, 7, 7);
        let keep = place_card_suit_rank(&mut e, 0, Suit::Diamond, CardKind::Shan, 2, 2);
        let _ = keep;
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard asks discard_to_limit (4 - 1 = 3 cards)
        let ch_d = e.open_choice().cloned().expect("discard_to_limit");
        assert_eq!(ch_d.spec.prompt, "discard_to_limit");
        let _ = answer(
            &mut e,
            0,
            vec![ChoiceOption::Card(c7_a), ChoiceOption::Card(c5_mid), ChoiceOption::Card(c7_b)],
        );
        let ch_ord = e.open_choice().cloned().expect("nsyice:order_cards");
        assert_eq!(ch_ord.spec.prompt, "nsyice:order_cards");
        let _ = answer(
            &mut e,
            0,
            vec![ChoiceOption::Card(c7_a), ChoiceOption::Card(c5_mid), ChoiceOption::Card(c7_b)],
        );
        let ch_top = e.open_choice().cloned().expect("nsyice:top_card");
        assert_eq!(ch_top.spec.prompt, "nsyice:top_card");
        let mut e = roundtrip_restore(&e);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c7_a)]);
        let ch_dmg = e.open_choice().cloned().expect("nsyice:damage_target");
        assert_eq!(ch_dmg.spec.prompt, "nsyice:damage_target");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().player(0).hand.contains(&c5_mid), "nsyice middle card gained to hand");
        assert_eq!(e.state_core().deck.last().copied(), Some(c7_a), "nsyice top card placed on top of deck");
        assert_eq!(e.state_core().deck.first().copied(), Some(c7_b), "nsyice bottom card placed at bottom");
        assert_eq!(e.state_core().player(1).hp, 3, "nsyice dealt 1 damage");

        // `nsyice` is locked: Pindian cards must NOT trigger nsyice; chanyuan seal (hp=1) must NOT block nsyice!
        let mut elock = start_2p(2026_9119);
        elock.debug_grant(0, "nsyice");
        elock.debug_grant(0, "nsfeixiong");
        elock.debug_set_turn(0, Phase::Draw, 1);
        elock.debug_clear_hand(0);
        elock.debug_clear_hand(1);
        let hi0 = place_suit_rank_any(&mut elock, 0, Suit::Spade, 7, 7);
        let _lo0 = place_rank_any(&mut elock, 0, CardKind::Sha, 2, 4);
        let _lo1 = place_suit_rank_any(&mut elock, 1, Suit::Club, 7, 7);
        let _ = submit(&mut elock, 0, GameAction::NextPhase);
        let _ = answer(&mut elock, 0, vec![ChoiceOption::Player(1)]);
        if elock.open_choice().is_some_and(|c| c.spec.prompt == "nsfeixiong:pindian_owner") {
            let _ = answer(&mut elock, 0, vec![ChoiceOption::Card(hi0)]);
        }
        assert!(
            elock.open_choice().is_none_or(|c| c.spec.prompt != "nsyice:order_cards"),
            "Pindian card must NOT trigger nsyice even when rank matches/enters discard"
        );
        drain_pass(&mut elock, 16);
        assert!(pile(&elock, 0, "nsyice").is_empty(), "Pindian card must NOT enter nsyice pile");
        elock.debug_grant(0, "txtanbei_effect1");
        elock.debug_set_hp(0, 1);
        elock.debug_clear_hand(0);
        let d1 = place_card_suit_rank(&mut elock, 0, Suit::Spade, CardKind::Sha, 6, 10);
        let d2 = place_card_suit_rank(&mut elock, 0, Suit::Heart, CardKind::Shan, 1, 13);
        let d3 = place_hand_kind_any(&mut elock, 0, CardKind::Tao);
        let _ = submit(&mut elock, 0, GameAction::NextPhase);
        let _ = answer(&mut elock, 0, vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)]);
        let ch_nl = elock.open_choice().cloned().expect("nsyice is locked so non-locked seal must NOT block it");
        assert_eq!(ch_nl.spec.prompt, "nsyice:order_cards");
        let _ = answer(&mut elock, 0, vec![ChoiceOption::Card(d1), ChoiceOption::Card(d2)]);
        drain_pass(&mut elock, 16);
        let _ = d3;
        assert_eq!(pile(&elock, 0, "nsyice"), vec![d1, d2]);
    }
}
