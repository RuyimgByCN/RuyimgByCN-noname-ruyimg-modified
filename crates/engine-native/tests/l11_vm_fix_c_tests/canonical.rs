use super::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, RejectReason};

#[test]
fn l11_vm_c_fix27_canonical_taoluan_xionghuo_xinhongyan() {
    // 25. `taoluan` (`yijiang.js:5073`): real CardUsed + taoluan:choose_targets + strict diff type + Dying + no Nullify/Shan.
    {
        let mut e = start_3p(2026_9301);
        e.debug_grant(0, "taoluan");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let c_sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let h1_trick = place_hand_kind_any(&mut e, 1, CardKind::Dismantle);
        let h1_basic = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        // Activate taoluan -> convert `c_sha` into `Wuzhong` (Tag 3)!
        let res = activate(&mut e, 0, "taoluan");
        assert!(!matches!(res, CommandOutcome::Rejected(_)));
        let ch_c = e.open_choice().cloned().expect("taoluan:choose_card");
        assert_eq!(ch_c.spec.prompt, "taoluan:choose_card");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_sha)]);
        let ch_v = e.open_choice().cloned().expect("taoluan:choose_virtual");
        assert_eq!(ch_v.spec.prompt, "taoluan:choose_virtual");
        // Verify Nullify/Shan are NEVER in taoluan_virtual (tags 0..14).
        assert!(ch_v.spec.options.len() <= 14);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(3)]); // Wuzhong!
        // Draws 2 from real Wuzhong AFTER CardUsed, then asks taoluan:choose_helper!
        let ch_h = e.open_choice().cloned().expect("taoluan:choose_helper");
        assert_eq!(ch_h.spec.prompt, "taoluan:choose_helper");
        assert_eq!(e.state_core().player(0).hand.len(), 2, "real Wuzhong drew 2");
        let mut e = roundtrip_restore(&e);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let ch_g = e.open_choice().cloned().expect("taoluan:helper_give");
        assert_eq!(ch_g.spec.prompt, "taoluan:helper_give");
        assert!(!ch_g.spec.options.contains(&ChoiceOption::Card(h1_trick)), "strictly excludes same type (Trick)");
        assert!(ch_g.spec.options.contains(&ChoiceOption::Card(h1_basic)), "allows diff type (Basic)");
        let _ = answer(&mut e, 1, vec![ChoiceOption::Card(h1_basic)]);
        drain_pass(&mut e, 8);
        assert!(e.state_core().player(0).hand.contains(&h1_basic));

        // Now use taoluan as `Dismantle` (Tag 4) -> opens `taoluan:choose_targets` -> helper cancels -> LoseHp + disabled!
        let c2 = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let _ = activate(&mut e, 0, "taoluan");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c2)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(4)]); // Dismantle!
        let ch_tg = e.open_choice().cloned().expect("taoluan:choose_targets");
        assert_eq!(ch_tg.spec.prompt, "taoluan:choose_targets");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        // Dismantle target choice (seat 1 has h1_trick in hand, wait - seat 1 auto-picks or asks)
        while let Some(ch) = e.open_choice().cloned() {
            if ch.spec.prompt == "taoluan:choose_helper" {
                let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
                break;
            }
            let first = ch.spec.options[0];
            let _ = answer(&mut e, ch.observer, vec![first]);
        }
        // Seat 1 has no non-Trick -> taoluan skips helper_give & owner loses 1 HP + disabled this turn!
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(0).hp, 3, "taoluan LoseHp 1 on no diff-type give");
        assert_eq!(flag(&e, 0, "taoluan_disabled_turn"), 1);
        assert!(matches!(activate(&mut e, 0, "taoluan"), CommandOutcome::Rejected(_)));

        // Dying Tao via taoluan:
        let mut ed = start_2p(2026_9302);
        ed.debug_grant(0, "taoluan");
        ed.debug_set_hp(0, 1);
        advance_to_phase(&mut ed, 1, Phase::Play, None);
        ed.debug_clear_hand(0);
        ed.debug_clear_hand(1);
        let save_c = place_hand_kind_any(&mut ed, 0, CardKind::Sha);
        let h1_helper_card = place_hand_kind_any(&mut ed, 1, CardKind::Dismantle);
        let sha1 = place_hand_kind_any(&mut ed, 1, CardKind::Sha);
        let _ = submit(&mut ed, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        let ch_s = ed.open_choice().cloned().expect("dying save with taoluan");
        let tl_def = ed.catalog().skill_by_name("taoluan").unwrap();
        assert!(ch_s.spec.options.contains(&ChoiceOption::Skill(tl_def)));
        let _ = answer(&mut ed, 0, vec![ChoiceOption::Skill(tl_def)]);
        let _ = answer(&mut ed, 0, vec![ChoiceOption::Card(save_c)]);
        if let Some(ch_h) = ed.open_choice().cloned() {
            if ch_h.spec.prompt == "taoluan:choose_helper" {
                let _ = answer(&mut ed, 0, vec![ChoiceOption::Player(1)]);
                let _ = answer(&mut ed, 1, vec![ChoiceOption::Card(h1_helper_card)]);
            }
        }
        drain_pass(&mut ed, 16);
        assert!(ed.state_core().player(0).alive && ed.state_core().player(0).hp == 1, "taoluan dying Tao saved");
    }

    // 26. `xinfu_xionghuo` (`sp.js:23002`): 3 start marks, Play grants mark, PlayBegin consumes mark (3 modes!), +1 dmg & chanyuan!
    {
        let mut e = start_2p(2026_9303);
        e.debug_grant(0, "xinfu_xionghuo");
        assert_eq!(flag(&e, 0, "xinfu_xionghuo_marks"), 3, "granting xinfu_xionghuo starts with 3 marks");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        let res = activate(&mut e, 0, "xinfu_xionghuo");
        assert!(!matches!(res, CommandOutcome::Rejected(_)));
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut e, 8);
        assert_eq!(flag(&e, 0, "xinfu_xionghuo_marks"), 2);
        assert_eq!(flag(&e, 1, "xinfu_xionghuo_marked"), 1);
        assert_eq!(flag(&e, 1, "xionghuo"), 1);

        // Caster deals +1 damage to marked victim!
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let hp1 = e.state_core().player(1).hp;
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, hp1 - 2, "xinfu_xionghuo caster deals +1 dmg");

        // Marked victim PlayBegin consumes mark and runs one of 3 random modes!
        let mut saw_m1 = false;
        let mut saw_m2 = false;
        let mut saw_m3 = false;
        for seed in 2026_9310u64..2026_9340u64 {
            let mut em = start_2p(seed);
            em.debug_grant(0, "xinfu_xionghuo");
            advance_to_phase(&mut em, 0, Phase::Play, None);
            let _ = activate(&mut em, 0, "xinfu_xionghuo");
            let _ = answer(&mut em, 0, vec![ChoiceOption::Player(1)]);
            drain_pass(&mut em, 8);
            let _eq1 = equip_kind_suit(&mut em, 1, CardKind::Armor, None);
            let _h1 = place_hand_kind_any(&mut em, 1, CardKind::Sha);
            let _ = place_hand_kind_any(&mut em, 1, CardKind::Shan);
            em.debug_clear_hand(0);
            let hp_pre = em.state_core().player(1).hp;
            let h0_pre = em.state_core().player(0).hand.len();
            advance_to_phase(&mut em, 1, Phase::Play, None);
            drain_pass(&mut em, 8);
            if flag(&em, 1, "xinfu_xionghuo_disable_src") == 1 {
                saw_m1 = true;
                let s_blk = place_hand_kind_any(&mut em, 1, CardKind::Sha);
                assert!(matches!(
                    submit(&mut em, 1, GameAction::PlayCard { card: s_blk, targets: vec![0] }),
                    CommandOutcome::Rejected(RejectReason::IllegalTargets)
                ), "Mode 1 blocks Sha targeting xinfu_xionghuo caster");
            } else if flag(&em, 1, "xinfu_xionghuo_hand_minus") == 1
                && em.state_core().player(1).hp == hp_pre - 1
            {
                assert_eq!(
                    em.state_core().hand_limit(em.catalog(), 1),
                    (em.state_core().player(1).hp as usize).saturating_sub(1),
                    "xinfu_xionghuo Mode 2 reduces hand limit by 1"
                );
                saw_m2 = true;
            } else if em.state_core().player(0).hand.len() >= h0_pre + 2 {
                saw_m3 = true;
            }
            if saw_m1 && saw_m2 && saw_m3 {
                break;
            }
        }
        assert!(saw_m1 && saw_m2 && saw_m3, "xionghuo must exercise all 3 modes across seeds ({saw_m1},{saw_m2},{saw_m3})");

        // Chanyuan seal blocks +1 caster bonus!
        let mut ec = start_2p(2026_9350);
        ec.debug_grant(0, "xinfu_xionghuo");
        ec.debug_grant(0, "chanyuan");
        ec.debug_set_hp(0, 1);
        ec.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "xinfu_xionghuo_marked".to_string()), 1);
        ec.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "xionghuo".to_string()), 1);
        advance_to_phase(&mut ec, 0, Phase::Play, None);
        ec.debug_clear_hand(0);
        ec.debug_clear_hand(1);
        let sha_c = place_hand_kind_any(&mut ec, 0, CardKind::Sha);
        let _ = submit(&mut ec, 0, GameAction::PlayCard { card: sha_c, targets: vec![1] });
        drain_pass(&mut ec, 16);
        assert_eq!(ec.state_core().player(1).hp, 3, "chanyuan sealed xinfu_xionghuo must NOT add +1 dmg");
    }

    // 27. `xinhongyan` (`l20_batch_b_asks.rs:643`): effective_card_suit treats Spade judge as Heart!
    {
        let mut e = start_2p(2026_9360);
        e.debug_grant(0, "xinhongyan");
        e.debug_grant(1, "刚烈");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        // ganglie checks `!Heart` (if Heart judge -> ganglie FAILS!). Put Spade judge on deck top for ganglie!
        // With xinhongyan on seat 0 (wait: ganglie owner judges -> if ganglie owner has xinhongyan, Spade -> Heart -> fails!).
        let mut eg = start_2p(2026_9361);
        eg.debug_grant(1, "xinhongyan");
        eg.debug_grant(1, "刚烈");
        advance_to_phase(&mut eg, 0, Phase::Play, None);
        eg.debug_clear_hand(0);
        eg.debug_clear_hand(1);
        let sha = place_hand_kind_any(&mut eg, 0, CardKind::Sha);
        let _keep0 = place_hand_kind_any(&mut eg, 0, CardKind::Shan);
        let _keep0b = place_hand_kind_any(&mut eg, 0, CardKind::Tao);
        let _ = move_to_deck_top_suit_rank(&mut eg, Suit::Spade, 5, 5);
        let _ = submit(&mut eg, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        // seat 1 takes 1 dmg, triggers ganglie -> Spade converted to Heart by xinhongyan -> NO discard/dmg on seat 0!
        drain_pass(&mut eg, 16);
        assert_eq!(eg.state_core().player(0).hp, 4, "xinhongyan Heart conversion prevents ganglie damage");
        let _ = &mut e;
    }
}

#[test]
fn l11_vm_c_r2_taoluan_shan_wuxie_effective_diff_and_expiry() {
    // `taoluan4` (Shan via chooseToUse, NOT chooseToRespond Wanjian) + `taoluan3` out-of-turn Phase::End expiry.
    {
        let mut e = start_2p(2026_9370);
        e.debug_grant(0, "taoluan");
        advance_to_phase(&mut e, 1, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let cost0 = place_hand_kind_any(&mut e, 0, CardKind::Armor);
        let h1_basic = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        let h1_trick = place_hand_kind_any(&mut e, 1, CardKind::Dismantle);
        let sha1 = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        let ch = e.open_choice().cloned().expect("Shan prompt should offer taoluan");
        let tl_def = e.catalog().skill_by_name("taoluan").unwrap();
        assert!(
            ch.spec.options.contains(&ChoiceOption::Skill(tl_def)),
            "taoluan4 must offer taoluan on Shan chooseToUse"
        );
        let _ = answer(&mut e, 0, vec![ChoiceOption::Skill(tl_def)]);
        let _ = answer(&mut e, 0, vec![ChoiceOption::Card(cost0)]);
        let ch_h = e.open_choice().cloned().expect("taoluan:choose_helper after Shan");
        assert_eq!(ch_h.spec.prompt, "taoluan:choose_helper");
        let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let ch_g = e.open_choice().cloned().expect("taoluan:helper_give after Shan");
        assert!(
            !ch_g.spec.options.contains(&ChoiceOption::Card(h1_basic)),
            "Shan is Basic -> helper_give must exclude Basic cards"
        );
        assert!(
            ch_g.spec.options.contains(&ChoiceOption::Card(h1_trick)),
            "Shan is Basic -> helper_give must allow Trick cards"
        );
        let _ = answer(&mut e, 1, vec![ChoiceOption::Card(h1_trick)]);
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(0).hp, 4, "taoluan4 virtual Shan dodged Sha");
        assert_ne!(flag(&e, 0, "taoluan_used_mask") & (1u32 << 14), 0);

        // `taoluan4` does NOT fire on ArcheryAttack (`Wanjian`, chooseToRespond)!
        let mut ew = start_2p(2026_9371);
        ew.debug_grant(0, "taoluan");
        advance_to_phase(&mut ew, 1, Phase::Play, None);
        ew.debug_clear_hand(0);
        ew.debug_clear_hand(1);
        let _c0 = place_hand_kind_any(&mut ew, 0, CardKind::Armor);
        let shan0w = place_hand_kind_any(&mut ew, 0, CardKind::Shan);
        let wj = place_hand_card_named(&mut ew, 1, "archery");
        let play_res = submit(&mut ew, 1, GameAction::PlayCard { card: wj, targets: vec![] });
        assert!(!matches!(play_res, CommandOutcome::Rejected(_)), "play wj: {play_res:?}");
        // Dismiss NullifyWindow (which offers taoluan5) so ArcheryAttack reaches Shan chooseToRespond!
        let mut saw_aoe_shan = false;
        while let Some(ch_wj) = ew.open_choice().cloned() {
            if ch_wj.spec.prompt == "nullify_window" {
                let ans_res = answer(&mut ew, ch_wj.observer, vec![ChoiceOption::Pass]);
                assert!(!matches!(ans_res, CommandOutcome::Rejected(_)));
                continue;
            }
            let tl_w = ew.catalog().skill_by_name("taoluan").unwrap();
            assert!(ch_wj.spec.options.contains(&ChoiceOption::Card(shan0w)));
            assert!(
                !ch_wj.spec.options.contains(&ChoiceOption::Skill(tl_w)),
                "taoluan4 is chooseToUse only and must NOT fire on ArcheryAttack chooseToRespond"
            );
            saw_aoe_shan = true;
            break;
        }
        assert!(saw_aoe_shan, "ArcheryAttack must reach Shan chooseToRespond prompt");
        drain_pass(&mut ew, 16);

        // Helper has no diff-type card out-of-turn -> LoseHp + disabled, then expires at opponent Phase::End!
        let mut ex = start_2p(2026_9372);
        ex.debug_grant(0, "taoluan");
        advance_to_phase(&mut ex, 1, Phase::Play, None);
        ex.debug_clear_hand(0);
        ex.debug_clear_hand(1);
        let c0x = place_hand_kind_any(&mut ex, 0, CardKind::Armor);
        let _h1_only_basic = place_hand_kind_any(&mut ex, 1, CardKind::Sha);
        let sha1x = place_hand_kind_any(&mut ex, 1, CardKind::Sha);
        let _ = submit(&mut ex, 1, GameAction::PlayCard { card: sha1x, targets: vec![0] });
        let tl_x = ex.catalog().skill_by_name("taoluan").unwrap();
        let _ = answer(&mut ex, 0, vec![ChoiceOption::Skill(tl_x)]);
        let _ = answer(&mut ex, 0, vec![ChoiceOption::Card(c0x)]);
        let _ = answer(&mut ex, 0, vec![ChoiceOption::Player(1)]);
        drain_pass(&mut ex, 16);
        assert_eq!(ex.state_core().player(0).hp, 3, "taoluan failed helper give loses 1 HP");
        assert_eq!(flag(&ex, 0, "taoluan_disabled_turn"), 1);
        // Same turn: disabled blocks taoluan5 on NullifyWindow.
        let _keep0x = place_hand_kind_any(&mut ex, 0, CardKind::Shan);
        let _cost0x2 = place_hand_kind_any(&mut ex, 0, CardKind::Armor);
        let dis1x = take_deck_or_discard_by(&mut ex, |d| {
            d.kind == CardKind::Dismantle && !d.name.starts_with("txhj_")
        });
        ex.state_core_mut().player_mut(1).hand.push(dis1x);
        let _ = submit(&mut ex, 1, GameAction::PlayCard { card: dis1x, targets: vec![0] });
        if let Some(ch_nx) = ex.open_choice() {
            assert_ne!(ch_nx.spec.prompt, "nullify_window", "disabled taoluan5 must NOT open NullifyWindow for owner");
            assert!(!ch_nx.spec.options.contains(&ChoiceOption::Skill(tl_x)));
        }
        drain_pass(&mut ex, 16);
        // Advance past seat 1's Phase::End into seat 0's Play -> taoluan3 expired!
        advance_to_phase(&mut ex, 0, Phase::Play, None);
        assert_eq!(
            flag(&ex, 0, "taoluan_disabled_turn"),
            0,
            "taoluan3 temp block must expire at end of current (opponent) turn"
        );
        assert!(
            !matches!(activate(&mut ex, 0, "taoluan"), CommandOutcome::Rejected(_)),
            "taoluan must be usable again once taoluan3 expired at opponent Phase::End"
        );
    }

    // `taoluan5` (Nullify on NullifyWindow) + `taoluan_diff_type_cards` effective kind (`wushen`).
    {
        let mut en = start_2p(2026_9373);
        en.debug_grant(0, "taoluan");
        advance_to_phase(&mut en, 1, Phase::Play, None);
        en.debug_clear_hand(0);
        en.debug_clear_hand(1);
        let keep0 = place_hand_kind_any(&mut en, 0, CardKind::Tao);
        let cost0 = place_hand_kind_any(&mut en, 0, CardKind::Armor);
        let h1_basic = place_hand_kind_any(&mut en, 1, CardKind::Tao);
        let h1_trick = place_hand_kind_any(&mut en, 1, CardKind::Wuzhong);
        let dis1 = place_hand_kind_any(&mut en, 1, CardKind::Dismantle);
        let _ = submit(&mut en, 1, GameAction::PlayCard { card: dis1, targets: vec![0] });
        let ch_n = en.open_choice().cloned().expect("NullifyWindow should open for taoluan5");
        let tl_def = en.catalog().skill_by_name("taoluan").unwrap();
        assert!(
            ch_n.spec.options.contains(&ChoiceOption::Skill(tl_def)),
            "taoluan5 must offer taoluan on NullifyWindow"
        );
        let _ = answer(&mut en, 0, vec![ChoiceOption::Skill(tl_def)]);
        let _ = answer(&mut en, 0, vec![ChoiceOption::Card(cost0)]);
        let ch_h = en.open_choice().cloned().expect("taoluan:choose_helper after Nullify");
        assert_eq!(ch_h.spec.prompt, "taoluan:choose_helper");
        let _ = answer(&mut en, 0, vec![ChoiceOption::Player(1)]);
        let ch_g = en.open_choice().cloned().expect("taoluan:helper_give after Nullify");
        assert!(
            !ch_g.spec.options.contains(&ChoiceOption::Card(h1_trick)),
            "Nullify is Trick -> helper_give must exclude Trick cards"
        );
        assert!(
            ch_g.spec.options.contains(&ChoiceOption::Card(h1_basic)),
            "Nullify is Trick -> helper_give must allow Basic cards"
        );
        let _ = answer(&mut en, 1, vec![ChoiceOption::Card(h1_basic)]);
        drain_pass(&mut en, 16);
        assert!(en.state_core().player(0).hand.contains(&keep0), "taoluan5 virtual Nullify cancelled Dismantle");
        assert_ne!(flag(&en, 0, "taoluan_used_mask") & (1u32 << 15), 0);

        // `taoluan_diff_type_cards` effective kind: helper holds ONLY `ht_wz` (raw Trick) whose `declared_kinds` effective kind is `Sha` (Basic).
        // When Zhang Rang uses `Sha` (Basic) via taoluan, effective `Sha` is same-type (Basic) -> NOT diff-type -> LoseHp!
        let mut ew = start_2p(2026_9374);
        ew.debug_grant(0, "taoluan");
        advance_to_phase(&mut ew, 0, Phase::Play, None);
        ew.debug_clear_hand(0);
        ew.debug_clear_hand(1);
        let c0 = place_hand_kind_any(&mut ew, 0, CardKind::Armor);
        let ht_wz = place_card_suit_rank(&mut ew, 1, Suit::Heart, CardKind::Wuzhong, 1, 13);
        ew.state_core_mut().declared_kinds.insert(ht_wz, CardKind::Sha);
        let _ = activate(&mut ew, 0, "taoluan");
        let _ = answer(&mut ew, 0, vec![ChoiceOption::Card(c0)]);
        let _ = answer(&mut ew, 0, vec![ChoiceOption::Tag(0)]); // Tag 0 = Sha (Basic)
        assert_eq!(ew.open_choice().expect("taoluan:choose_targets").spec.prompt, "taoluan:choose_targets");
        let _ = answer(&mut ew, 0, vec![ChoiceOption::Player(1)]);
        assert_eq!(ew.open_choice().expect("taoluan:choose_helper").spec.prompt, "taoluan:choose_helper");
        // If helper had effective Trick, `taoluan:helper_give` would open; because wushen turns Heart Wuzhong into Basic Sha, helper has NO diff-type card!
        let _ = answer(&mut ew, 0, vec![ChoiceOption::Player(1)]);
        assert!(ew.open_choice().is_none_or(|c| c.spec.prompt != "taoluan:helper_give"));
        drain_pass(&mut ew, 16);
        assert_eq!(ew.state_core().player(0).hp, 3, "effective Sha card on wushen helper is same-type (Basic) -> no give, LoseHp 1");
    }
}
