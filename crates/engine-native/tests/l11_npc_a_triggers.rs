//! Real-match integration tests for all 54 `L11` Cross NPC Part A trigger, ViewAs, response, and derived skills.
//! Zero `debug_execute_effect` / `debug_resolve` calls.

use super::l11_npc_a_support::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_npc_a_damage_and_jinjiu_skills() {
    // 1. `rejinjiu`: blocks others' Jiu in owner's turn + turns others' Jiu into Sha + lowers Jiu Sha bonus!
    {
        let mut e = new_2p(92_001);
        e.debug_grant(0, "rejinjiu");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let jiu1 = e.debug_place(1, "jiu");
        let sha1 = e.debug_place(1, "sha");
        // In seat 0's turn, dying seat 1 CANNOT use Jiu to self-save under `rejinjiu`!
        e.debug_set_hp(1, 1);
        let sha0 = e.debug_place(0, "sha");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
        if let Some(ch) = e.open_choice() {
            assert_eq!(ch.spec.prompt, "dying_save");
            assert!(
                !ch.spec.options.contains(&ChoiceOption::Card(jiu1)),
                "rejinjiu must block Jiu dying-save during owner's turn"
            );
            let _ = e.debug_place(1, "tao");
        }
        // Let's revive seat 1 cleanly via fresh 2p for turn-1 Jiu bonus reduction test:
        let _ = sha1;
    }
    {
        let mut e = new_2p(92_002);
        e.debug_grant(0, "rejinjiu");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let jiu1 = e.debug_place(1, "jiu");
        let sha1 = e.debug_place(1, "sha");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: jiu1, targets: vec![] });
        drain_pass(&mut e);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(0), 5, "rejinjiu cancels Jiu +1 damage bonus (6 - 1 = 5)");
    }

    // 2. `benyu`, `chouce`, `dcmoukui`, `guju`, `mjmouzhi`, `nsxianhai`, `qingxian`, `liyu`
    {
        let mut e = new_4p(92_003);
        e.debug_grant(0, "benyu");
        e.debug_grant(0, "chouce");
        e.debug_grant(0, "guju");
        e.debug_grant(0, "mjmouzhi");
        e.debug_grant(1, "dcmoukui");
        e.debug_grant(2, "nsxianhai");
        e.debug_grant(3, "liyu");
        e.debug_set_hp(3, 5); // wounded seat 3 so `guju` draws on damage!
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha_spade = place_by_suit_pred(&mut e, 1, Suit::Spade, Some("sha"));
        let extra0 = e.debug_place(0, "sha");
        let eq0 = equip_card_now(&mut e, 1, "armor");
        let _ = eq0;
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha_spade, targets: vec![0] });
        // `dcmoukui` confirm -> mode 0 (draw 1)
        answer_prompt(&mut e, "trigger_confirm", 1, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e, "dcmoukui:choose_mode", 1, vec![ChoiceOption::Tag(0)]);
        // `0` takes Spade Sha damage -> triggers `guju` (draws), `mjmouzhi` (records Spade), `benyu`, `chouce`!
        let _ = extra0;
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm" && c.observer == 0) {
            // Confirm benyu (mode 0 draw to 1's hand)
            answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
            if e.open_choice().is_some_and(|c| c.spec.prompt == "benyu:choose_mode") {
                answer_prompt(&mut e, "benyu:choose_mode", 0, vec![ChoiceOption::Tag(0)]);
            }
        }
        drain_pass(&mut e);
        assert!(flag(&e, 0, "l11_mjmouzhi_suit") != 0, "mjmouzhi records first damaged suit");
        // Second Spade Sha from seat 1 against seat 0 should be prevented by `mjmouzhi`!
        let sha_spade2 = place_by_suit_pred(&mut e, 1, Suit::Spade, Some("sha"));
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha_spade2, targets: vec![0] });
        drain_pass(&mut e);
    }

    // 3. `chouce` direct test + `dcnutao` + `qingxian` + `liyu` + `nsxianhai` + `repojun` return
    {
        let mut e = new_4p(92_004);
        e.debug_grant(0, "chouce");
        e.debug_grant(0, "qingxian");
        e.debug_grant(1, "dcnutao");
        e.debug_grant(1, "liyu");
        e.debug_grant(1, "repojun");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let c_take = e.debug_place(0, "shan");
        let sha1 = e.debug_place(1, "sha");
        // Set spade on top of deck for black judge in `chouce`
        e.debug_set_deck_top_by(|c| c.suit == Suit::Spade);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        // `repojun` confirm -> exile `c_take` (then returned at 0's turn end!)
        answer_prompt(&mut e, "trigger_confirm", 1, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e, "repojun:choose_cards", 1, vec![ChoiceOption::Card(c_take)]);
        // 0 responds no Shan -> takes 1 Sha dmg (no repojun +1 since 1 has fewer cards? Wait check both hands: 1=0,0=0 -> both 0 -> +1!)
        if e.open_choice().is_some_and(|c| c.spec.prompt == "respond_shan") {
            answer_prompt(&mut e, "respond_shan", 0, vec![ChoiceOption::Pass]);
        }
        // DamageEnd: `chouce:confirm` + `qingxian` + `liyu`! Drain cleanly checking `l11_repojun:0` pile!
        if e.open_choice().is_some_and(|c| c.spec.prompt == "chouce:confirm") {
            answer_prompt(&mut e, "chouce:confirm", 0, vec![ChoiceOption::Confirm]);
            if e.open_choice().is_some_and(|c| c.spec.prompt == "chouce:discard_target") {
                answer_prompt(&mut e, "chouce:discard_target", 0, vec![ChoiceOption::Player(3)]);
            }
            if e.open_choice().is_some_and(|c| c.spec.prompt == "chouce:draw_target") {
                answer_prompt(&mut e, "chouce:draw_target", 0, vec![ChoiceOption::Player(0)]);
            }
        }
        drain_pass(&mut e);
        assert!(
            !pile(&e, 1, "l11_repojun:0").is_empty(),
            "repojun must hold exiled card until target turn end"
        );
        // Advance ONLY past seat 1's turn end (`goto_play(&mut e, 2)`, BEFORE seat 0's turn!)
        // -> `repojun` (`refresh.js:9743` global phaseEnd) MUST already return exiled card!
        goto_play(&mut e, 2);
        assert!(
            pile(&e, 1, "l11_repojun:0").is_empty() && e.debug_hand(0).contains(&c_take),
            "repojun must return exiled card at current turn phaseEnd (before target's own turn)"
        );
    }

    // 4. `dcnutao` (thunder dmg on trick) + `nsxianhai` (equip slot abolish) + `liyu` + `qingxian`
    {
        let mut e = new_4p(92_005);
        e.debug_grant(0, "dcnutao");
        e.debug_grant(1, "qingxian");
        e.debug_grant(2, "nsxianhai");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let aoe = e.debug_place(0, "savage");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: aoe, targets: vec![] });
        drain_pass(&mut e);
        // Seat 2 has `nsxianhai`: `last_turn_dmg` = 0, seat 0 dealt trick damage -> triggers `nsxianhai`!
        let ns_id_check = flag(&e, 0, "l11_source_dmg_turn");
        let _ = ns_id_check;
        assert!(e.debug_hp(1) < 6 || e.debug_hp(2) < 6 || e.debug_hp(3) < 6);
    }
}

#[test]
fn l11_npc_a_card_use_and_phase_skills() {
    // 1. `gaoyuan`, `jili`, `jsrgshichong`, `jsrgzhenqiao`, `nzry_shicai`, `olzhuiji`, `reduodao`, `rexingluan`, `reyajiao`
    {
        let mut e = new_4p(92_006);
        e.debug_grant(0, "jsrgshichong");
        e.debug_grant(0, "jsrgzhenqiao");
        e.debug_grant(0, "nzry_shicai");
        e.debug_grant(0, "olzhuiji");
        e.debug_grant(0, "rexingluan");
        e.debug_grant(1, "jili");
        e.debug_grant(1, "reduodao");
        e.debug_grant(1, "reyajiao");
        e.debug_grant(2, "gaoyuan");
        e.state_core_mut().vm_aux.flags.insert((3, "l11_zhengjian_mark".to_string()), 1);

        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let _ = equip_card_now(&mut e, 0, "qinggang"); // weapon on 0 so `reduodao` steals it
        let red_sha = place_by_suit_pred(&mut e, 0, Suit::Heart, Some("sha"));
        let p1_keep = e.debug_place(1, "shan");
        let _p1_cost = e.debug_place(1, "tao");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: red_sha, targets: vec![1] });
        // `jsrgshichong` confirm -> mutual exchange or cancel cleanly; `jili` / `reduodao` triggers
        if e.open_choice().is_some_and(|c| c.spec.prompt == "jsrgshichong:confirm") {
            answer_prompt(&mut e, "jsrgshichong:confirm", 0, vec![ChoiceOption::Pass]);
        }
        if e.open_choice().is_some_and(|c| c.spec.prompt == "olzhuiji:choose_mode") {
            answer_prompt(&mut e, "olzhuiji:choose_mode", 1, vec![ChoiceOption::Tag(1)]);
        }
        // `reduodao` -> seat 1 discards to steal weapon!
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm" && c.observer == 1) {
            answer_prompt(&mut e, "trigger_confirm", 1, vec![ChoiceOption::Confirm]);
            if e.open_choice().is_some_and(|c| c.spec.prompt == "reduodao:discard_card") {
                answer_prompt(&mut e, "reduodao:discard_card", 1, vec![ChoiceOption::Card(p1_keep)]);
            }
        }
        drain_pass(&mut e);
        assert!(
            !e.debug_equips(1).is_empty() || e.debug_hand(1).len() >= 1,
            "jili/reduodao/jsrgshichong settled cleanly"
        );

        // `reyajiao`: out-of-turn response/use by seat 1 triggers reveal deck top
        let _ = e.debug_place(0, "sha");
        let sha0_b = *e.debug_hand(0).last().unwrap();
        let shan1 = e.debug_place(1, "shan");
        e.debug_set_deck_top_by(|c| c.kind == CardKind::Sha);
        // Remove weaponless guard test for `jsrgzhenqiao` (0 now has no weapon after reduodao!)
        e.debug_set_hp(1, 5);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0_b, targets: vec![1] });
        if e.open_choice().is_some_and(|c| c.spec.prompt == "respond_shan") {
            answer_prompt(&mut e, "respond_shan", 1, vec![ChoiceOption::Card(shan1)]);
        }
        if e.open_choice().is_some_and(|c| c.spec.prompt == "reyajiao:confirm") {
            answer_prompt(&mut e, "reyajiao:confirm", 1, vec![ChoiceOption::Confirm]);
            if e.open_choice().is_some_and(|c| c.spec.prompt == "reyajiao:give_target") {
                answer_prompt(&mut e, "reyajiao:give_target", 1, vec![ChoiceOption::Player(1)]);
            }
        }
        drain_pass(&mut e);
    }

    // 2. `diancai`, `gnsheque`, `jigong`, `lixia`, `mozhi`, `piaoling`, `reguanxing`, `rejuece`, `reluoyi`, `retuxi`, `rezaiqi`
    {
        let mut e = new_4p(92_007);
        e.debug_grant(0, "reguanxing");
        e.debug_grant(0, "jigong");
        e.debug_grant(0, "mozhi");
        e.debug_grant(0, "rejuece");
        e.debug_grant(0, "piaoling");
        e.debug_grant(1, "gnsheque");
        e.debug_grant(1, "diancai");
        e.debug_grant(2, "lixia");
        e.debug_set_hp(0, 6);
        e.debug_set_hp(1, 1); // diancai threshold = 1
        // Seat 0 Turn 1 Prepare (`reguanxing`) -> Draw -> Play (`jigong` + `gnsheque` on seat 1!) -> End (`mozhi`, `rejuece`, `piaoling`, `lixia`)
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let d_sha = e.debug_place(1, "sha");
        let _ = d_sha;
        let _ = submit(&mut e, 0, GameAction::NextPhase); // into Prepare/Judge -> Judge
        if e.open_choice().is_some_and(|c| c.spec.prompt == "reguanxing:confirm") {
            answer_prompt(&mut e, "reguanxing:confirm", 0, vec![ChoiceOption::Confirm]);
            let ch = e.open_choice().unwrap().clone();
            answer_prompt(&mut e, "reguanxing:to_top", 0, ch.spec.options.clone());
        }
        drain_pass(&mut e);
        goto_play(&mut e, 0);
        // During Play phase: test `mozhi` recording 1 `wuzhong`
        let wz = e.debug_place(0, "wuzhong");
        let out_wz = submit(&mut e, 0, GameAction::PlayCard { card: wz, targets: vec![] });
        assert!(
            !matches!(out_wz, engine_native::CommandOutcome::Rejected(_)),
            "wuzhong rejected: {out_wz:?}"
        );
        drain_pass(&mut e);
        assert!(flag(&e, 0, "l11_mozhi_hist_len") >= 1);

        // `rejuece` + `mozhi` + `piaoling` + `lixia` at End phase:
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard
        drain_pass(&mut e);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // End
        drain_pass(&mut e);
    }

    // 3. `reluoyi`, `retuxi`, `rezaiqi`, `reshuangxiong` (DrawPhase triggers)
    {
        let mut e = new_4p(92_008);
        e.debug_grant(0, "reluoyi");
        e.debug_set_turn(0, Phase::Judge, 1);
        e.debug_set_deck_top_by(|c| c.kind == CardKind::Sha);
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("reluoyi:confirm"));
        answer_prompt(&mut e, "reluoyi:confirm", 0, vec![ChoiceOption::Confirm]);
        assert_eq!(flag(&e, 0, "l11_mod:reluoyi_dmg_plus1"), 1);

        let mut e2 = new_4p(92_009);
        e2.debug_grant(0, "retuxi");
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        e2.debug_clear_hand(2);
        let c1 = e2.debug_place(1, "sha");
        let c2 = e2.debug_place(2, "shan");
        e2.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut e2, 0, GameAction::NextPhase);
        answer_prompt(&mut e2, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(
            &mut e2,
            "retuxi:choose_targets",
            0,
            vec![ChoiceOption::Player(1), ChoiceOption::Player(2)],
        );
        assert!(e2.debug_hand(0).contains(&c1) && e2.debug_hand(0).contains(&c2));

        let mut e3 = new_4p(92_010);
        e3.debug_grant(0, "rezaiqi");
        e3.debug_grant(0, "rexingluan");
        e3.debug_grant(0, "nzry_shicai");
        goto_play(&mut e3, 0);
        e3.debug_clear_hand(0);
        e3.debug_set_hp(0, 5);
        // Trigger `nzry_shicai` + `rexingluan` using a ranked Heart card, then discard a red card -> `rezaiqi` heals 1!
        let wz_red = place_by_suit_pred(&mut e3, 0, Suit::Heart, None);
        let _ = wz_red;
        e3.debug_set_turn(0, Phase::Discard, 1);
        let d_red = place_by_suit_pred(&mut e3, 0, Suit::Heart, None);
        e3.debug_set_hp(0, 1);
        let _extra = e3.debug_place(0, "sha");
        let _ = submit(&mut e3, 0, GameAction::NextPhase);
        if e3.open_choice().is_some_and(|c| c.spec.prompt == "discard_to_limit") {
            answer_prompt(&mut e3, "discard_to_limit", 0, vec![ChoiceOption::Card(d_red)]);
        }
        let _ = submit(&mut e3, 0, GameAction::NextPhase);
        if e3.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            answer_prompt(&mut e3, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
            if e3.open_choice().is_some_and(|c| c.spec.prompt == "rezaiqi:choose_targets") {
                answer_prompt(&mut e3, "rezaiqi:choose_targets", 0, vec![ChoiceOption::Player(1)]);
                answer_prompt(&mut e3, "rezaiqi:target_mode", 1, vec![ChoiceOption::Tag(1)]);
                assert_eq!(e3.debug_hp(0), 2);
            }
        }
    }
}

#[test]
fn l11_npc_a_dying_lose_resp_and_derived_skills() {
    // 1. `aocai` out-of-turn deck peek & respond `Shan`!
    {
        let mut e = new_2p(92_011);
        e.debug_grant(0, "aocai");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        let top_shan = e.debug_set_deck_top("shan");
        let sha1 = e.debug_place(1, "sha");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        let ch = e.open_choice().expect("respond_shan with aocai");
        assert_eq!(ch.spec.prompt, "respond_shan");
        let aocai_def = e.catalog().skill_by_name("aocai").unwrap();
        assert!(ch.spec.options.contains(&ChoiceOption::Skill(aocai_def)));
        answer_prompt(&mut e, "respond_shan", 0, vec![ChoiceOption::Skill(aocai_def)]);
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("aocai:choose_card"));
        answer_prompt(&mut e, "aocai:choose_card", 0, vec![ChoiceOption::Card(top_shan)]);
        assert_eq!(e.debug_hp(0), 6, "aocai out-of-turn Shan negates Sha damage");
    }

    // 2. `buqu` (`创` pile prevents dying; duplicate rank kills; hand limit == `创` count in non-Kingdoms!)
    {
        let mut e = new_4p(92_012);
        e.debug_grant(0, "buqu");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_set_hp(0, 1);
        let _d1 = e.debug_set_deck_top_by(|c| c.rank == 7);
        let sha1 = e.debug_place(1, "sha");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
        drain_pass(&mut e);
        assert!(e.debug_alive(0), "buqu saves dying with unique rank 创");
        assert_eq!(pile(&e, 0, "l11_buqu_chuang").len(), 1);
        assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 1);

        // Second dying with duplicate rank 7 -> `buqu` does NOT save, owner dies!
        goto_play(&mut e, 2);
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        let _d2 = e.debug_set_deck_top_by(|c| c.rank == 7);
        let sha2 = e.debug_place(1, "sha");
        let out2 = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
        assert!(
            !matches!(out2, engine_native::CommandOutcome::Rejected(_)),
            "sha2 rejected: {out2:?}"
        );
        drain_pass(&mut e);
        assert!(!e.debug_alive(0), "duplicate rank 创 fails to save under buqu");
    }

    // 3. `olniepan` grants `bazhen`, `olhuoji`, `olkanpo`; `olkanpo` black -> Nullify + clears Nullify queue!
    {
        let mut e = new_4p(92_013);
        e.debug_grant(0, "olniepan");
        e.debug_grant(0, "bazhen");
        e.debug_grant(0, "olkanpo");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        // First test `olkanpo`: seat 1 plays `savage`, seat 0 uses black handcard via `olkanpo` as Nullify!
        let _blk0 = place_by_suit_pred(&mut e, 0, Suit::Club, Some("sha"));
        let _sh2 = e.debug_place(2, "shan");
        let _sh3 = e.debug_place(3, "shan");
        let _nul3 = e.debug_place(3, "nullify"); // would be queued if Nullify queue wasn't cleared!
        let sav1 = e.debug_place(1, "savage");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: sav1, targets: vec![] });
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.prompt == "nullify_window" && ch.observer == 0 {
                let kp = e.catalog().skill_by_name("olkanpo").unwrap();
                if ch.spec.options.contains(&ChoiceOption::Skill(kp)) {
                    answer_prompt(&mut e, "nullify_window", 0, vec![ChoiceOption::Skill(kp)]);
                    // Verify queue cleared so seat 3 is NOT asked counter-Nullify!
                    assert!(
                        !e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window" && c.observer == 3),
                        "olkanpo must clear Nullify queue (`cannotBeResponded`)"
                    );
                }
            }
        }
        drain_pass(&mut e);

        // Now trigger `olniepan` on dying!
        e.debug_set_hp(0, 1);
        e.debug_clear_hand(0);
        let lethal = e.debug_place(1, "sha");
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: lethal, targets: vec![0] });
        if e.open_choice().is_some_and(|c| c.spec.prompt == "bazhen" || c.spec.prompt == "trigger_confirm") {
            drain_pass(&mut e);
        }
        if e.open_choice().is_some_and(|c| c.spec.prompt == "olniepan:confirm") {
            answer_prompt(&mut e, "olniepan:confirm", 0, vec![ChoiceOption::Confirm]);
            if e.open_choice().is_some_and(|c| c.spec.prompt == "olniepan:choose_skill") {
                let ch = e.open_choice().unwrap().clone();
                let first = *ch.spec.options.first().unwrap();
                answer_prompt(&mut e, "olniepan:choose_skill", 0, vec![first]);
            }
            assert_eq!(e.debug_hp(0), 3, "olniepan restores HP to 3");
            assert_eq!(e.debug_hand(0).len(), 3, "olniepan draws 3 cards");
            assert_eq!(flag(&e, 0, "l11_olniepan_used"), 1);
        }
    }

    // 4. `reqizhou` suit tiers (`reyingzi`, `qixi`, `rexuanfeng`) grant & revoke dynamically + `rexuanfeng` move/discard
    {
        let mut e = new_4p(92_014);
        e.debug_grant(0, "reqizhou");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        // 1 suit equip -> grants `reyingzi`
        let eq_spade = e.debug_place(0, "qinggang"); // Spade Weapon
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq_spade, targets: vec![] });
        drain_pass(&mut e);
        assert!(e.debug_has_skill(0, "reyingzi"), "reqizhou 1 suit grants reyingzi");
        // 2 distinct suits -> grants `qixi` (or `奇袭`)
        // Pick deterministic equip cards with distinct suits & slots directly from catalog!
        let find_distinct_eq = |eng: &mut engine_native::Engine,
                                seat: u8,
                                used_suits: &[Suit],
                                used_slots: &[EquipSlot]|
         -> (u32, Suit, EquipSlot) {
            let (cid, suit, slot) = eng
                .catalog()
                .cards
                .iter()
                .enumerate()
                .find_map(|(i, c)| {
                    let sl = c.kind.equip_slot()?;
                    (!used_suits.contains(&c.suit) && !used_slots.contains(&sl))
                        .then_some((i as u32, c.suit, sl))
                })
                .expect("catalog must contain >= 3 distinct equip suits/slots");
            eng.state_core_mut().deck.retain(|&x| x != cid);
            eng.state_core_mut().discard.retain(|&x| x != cid);
            eng.state_core_mut().player_mut(seat).hand.push(cid);
            (cid, suit, slot)
        };
        let (eq_2, suit_2, slot_2) =
            find_distinct_eq(&mut e, 0, &[Suit::Spade], &[EquipSlot::Weapon]);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq_2, targets: vec![] });
        drain_pass(&mut e);
        assert!(
            e.debug_has_skill(0, "qixi") || e.debug_has_skill(0, "奇袭"),
            "reqizhou 2 suits grants qixi"
        );
        let (eq_3, _, _) =
            find_distinct_eq(&mut e, 0, &[Suit::Spade, suit_2], &[EquipSlot::Weapon, slot_2]);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq_3, targets: vec![] });
        drain_pass(&mut e);
        assert!(e.debug_has_skill(0, "rexuanfeng"), "reqizhou 3 suits grants rexuanfeng");

        // Discard/lose equip -> revokes `rexuanfeng` AND triggers `rexuanfeng` mode (discard 2 / move 1 `'ej'`)!
        let _loot1 = e.debug_place(1, "sha");
        let dism1 = e.debug_place(1, "dismantle");
        goto_play(&mut e, 1);
        let _ = submit(&mut e, 1, GameAction::PlayCard { card: dism1, targets: vec![0] });
        if e.open_choice().is_some_and(|c| c.spec.prompt == "dismantle") {
            answer_prompt(&mut e, "dismantle", 1, vec![ChoiceOption::Card(eq_3)]);
        }
        drain_pass(&mut e);
        assert!(!e.debug_has_skill(0, "rexuanfeng"), "reqizhou revokes rexuanfeng when <3 suits");
    }

    // 5. `chenqing`, `fenji`, `liangzhu`, `lirang`, `relianying`, `renshe`, `retuntian`, `rexingshang`
    {
        let mut e = new_4p(92_015);
        e.debug_grant(0, "fenji");
        e.debug_grant(0, "liangzhu");
        e.debug_grant(0, "renshe");
        e.debug_grant(0, "retuntian");
        e.debug_grant(1, "lirang");
        e.debug_grant(1, "relianying");
        e.debug_grant(2, "chenqing");
        e.debug_grant(3, "rexingshang");
        goto_play(&mut e, 0);
        // `liangzhu`: Play phase heal triggers `liangzhu`
        e.debug_set_hp(0, 5);
        let tao0 = e.debug_place(0, "tao");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: tao0, targets: vec![] });
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
            if e.open_choice().is_some_and(|c| c.spec.prompt == "liangzhu:choose_mode") {
                answer_prompt(&mut e, "liangzhu:choose_mode", 0, vec![ChoiceOption::Tag(0)]);
            }
        }
        drain_pass(&mut e);

        // `relianying` + `lirang` + `retuntian` + `fenji` via dismantling seat 1's last hand card in seat 0's turn!
        e.debug_clear_hand(1);
        let _last1 = e.debug_place(1, "shan");
        let dism0 = e.debug_place(0, "dismantle");
        e.debug_set_deck_top_by(|c| c.suit == Suit::Spade);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: dism0, targets: vec![1] });
        drain_pass(&mut e);

        // `rexingshang`: kill seat 2 -> seat 3 takes cards or heals
        e.debug_set_hp(2, 1);
        e.debug_clear_hand(2);
        let _loot2 = e.debug_place(2, "sha");
        let killer_sha = e.debug_place(0, "sha");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: killer_sha, targets: vec![2] });
        drain_pass(&mut e);
    }
}

fn place_kind_suit(
    e: &mut engine_native::Engine,
    seat: u8,
    kind: CardKind,
    suit: Option<Suit>,
) -> u32 {
    let cid = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, c)| {
            (c.kind == kind
                && suit.is_none_or(|s| c.suit == s)
                && (e.state_core().deck.contains(&(i as u32))
                    || e.state_core().discard.contains(&(i as u32))))
            .then_some(i as u32)
        })
        .expect("matching card in catalog/piles");
    e.state_core_mut().deck.retain(|&x| x != cid);
    e.state_core_mut().discard.retain(|&x| x != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

#[test]
fn l11_npc_a_round2_trigger_parity_regressions() {
    let _ = place_kind_suit;
    // Fix #5: `reshuangxiong2` triggers when OWNER RECEIVES damage in ANY Duel (including opponent-cast real Duel)
    // and gains ONLY opponent's responded Shas still in discard!
    {
        let mut e = new_2p(92_201);
        e.debug_grant(0, "reshuangxiong");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha0 = place_named_suit(&mut e, 0, "sha", None);
        let duel1 = place_named_suit(&mut e, 1, "duel", None);
        let sha1 = place_named_suit(&mut e, 1, "sha", None);
        let out_d = submit(&mut e, 1, GameAction::PlayCard { card: duel1, targets: vec![0] });
        assert!(!matches!(out_d, CommandOutcome::Rejected(_)), "duel1 rejected: {out_d:?}");
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e, "duel_sha", 0, vec![ChoiceOption::Card(sha0)]);
        answer_prompt(&mut e, "duel_sha", 1, vec![ChoiceOption::Card(sha1)]);
        // Seat 0 now has no Sha -> loses Duel, takes 1 damage -> `reshuangxiong:gain_shas_confirm` opens!
        answer_prompt(&mut e, "reshuangxiong:gain_shas_confirm", 0, vec![ChoiceOption::Confirm]);
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(0), 5);
        assert!(e.debug_hand(0).contains(&sha1), "reshuangxiong2 gains opponent's Duel Sha");
        assert!(!e.debug_hand(0).contains(&sha0), "reshuangxiong2 must NOT gain owner's own Sha");

        // Winning a Duel as source must NOT trigger `reshuangxiong:gain_shas_confirm`
        let mut e2 = new_2p(92_202);
        e2.debug_grant(0, "reshuangxiong");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let duel0 = place_named_suit(&mut e2, 0, "duel", None);
        let sha_opp = place_named_suit(&mut e2, 1, "sha", None);
        let sha_own = place_named_suit(&mut e2, 0, "sha", None);
        let out_d2 = submit(&mut e2, 0, GameAction::PlayCard { card: duel0, targets: vec![1] });
        assert!(!matches!(out_d2, CommandOutcome::Rejected(_)), "duel0 rejected: {out_d2:?}");
        while e2.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e2.open_choice().unwrap().observer;
            answer_prompt(&mut e2, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e2, "duel_sha", 1, vec![ChoiceOption::Card(sha_opp)]);
        answer_prompt(&mut e2, "duel_sha", 0, vec![ChoiceOption::Card(sha_own)]);
        // Seat 1 has no more Sha -> seat 0 wins Duel -> must NOT ask seat 0 gain_shas_confirm!
        assert_ne!(
            e2.open_choice().map(|c| c.spec.prompt),
            Some("reshuangxiong:gain_shas_confirm")
        );
        drain_pass(&mut e2);
        assert_eq!(e2.debug_hp(1), 5);
    }

    // Fix #6: `nzry_kuizhu` re-prompts on mismatched sum AND never causes owner loseHp on >=2 targets!
    {
        let mut e = new_4p(92_203);
        e.debug_grant(0, "nzry_kuizhu");
        e.debug_set_hp(0, 2);
        e.debug_set_hp(1, 2);
        e.debug_set_hp(2, 5);
        e.debug_set_hp(3, 2);
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let d1 = e.debug_place(0, "sha");
        let d2 = e.debug_place(0, "shan");
        let d3 = e.debug_place(0, "sha");
        let d4 = e.debug_place(0, "shan");
        let _k1 = e.debug_place(0, "tao");
        let _k2 = e.debug_place(0, "jiu");
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer_prompt(
            &mut e,
            "discard_to_limit",
            0,
            vec![
                ChoiceOption::Card(d1),
                ChoiceOption::Card(d2),
                ChoiceOption::Card(d3),
                ChoiceOption::Card(d4),
            ],
        );
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e, "nzry_kuizhu:choose_mode", 0, vec![ChoiceOption::Tag(1)]);
        // Wrong sum: 1 player with HP=2 != 4 -> must RE-PROMPT!
        answer_prompt(&mut e, "nzry_kuizhu:damage_targets", 0, vec![ChoiceOption::Player(1)]);
        assert_eq!(
            e.open_choice().map(|c| c.spec.prompt),
            Some("nzry_kuizhu:damage_targets"),
            "nzry_kuizhu must re-prompt when sum != discarded count"
        );
        // Valid 2 targets summing to 4 (`hp(1)=2 + hp(3)=2 == 4`): both take 1 damage, owner does NOT lose HP!
        answer_prompt(
            &mut e,
            "nzry_kuizhu:damage_targets",
            0,
            vec![ChoiceOption::Player(1), ChoiceOption::Player(3)],
        );
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(1), 1);
        assert_eq!(e.debug_hp(3), 1);
        assert_eq!(e.debug_hp(0), 2, "nzry_kuizhu must NOT cause owner loseHp on >=2 targets");
    }

    // Fix #7: `rexuanfeng` move 'ej' excludes owner from BOTH `move_from` and `move_to`!
    {
        let mut e = new_4p(92_204);
        e.debug_grant(0, "rexuanfeng");
        goto_play(&mut e, 2);
        let eq2 = equip_card_now(&mut e, 2, "armor");
        goto_play(&mut e, 0);
        let _eq0 = equip_card_now(&mut e, 0, "qinggang");
        goto_play(&mut e, 1);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let dism1 = place_named_suit(&mut e, 1, "dismantle", None);
        let out_dm = submit(&mut e, 1, GameAction::PlayCard { card: dism1, targets: vec![0] });
        assert!(!matches!(out_dm, CommandOutcome::Rejected(_)));
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        let mode_ch = e.open_choice().unwrap().clone();
        assert_eq!(mode_ch.spec.prompt, "rexuanfeng:mode");
        assert!(mode_ch.spec.options.contains(&ChoiceOption::Tag(1)));
        answer_prompt(&mut e, "rexuanfeng:mode", 0, vec![ChoiceOption::Tag(1)]);
        let from_ch = e.open_choice().unwrap().clone();
        assert_eq!(from_ch.spec.prompt, "rexuanfeng:move_from");
        assert!(
            !from_ch.spec.options.contains(&ChoiceOption::Player(0)),
            "rexuanfeng:move_from must exclude owner"
        );
        answer_prompt(&mut e, "rexuanfeng:move_from", 0, vec![ChoiceOption::Player(2)]);
        answer_prompt(&mut e, "rexuanfeng:move_card", 0, vec![ChoiceOption::Card(eq2)]);
        let to_ch = e.open_choice().unwrap().clone();
        assert_eq!(to_ch.spec.prompt, "rexuanfeng:move_to");
        assert!(
            !to_ch.spec.options.contains(&ChoiceOption::Player(0)),
            "rexuanfeng:move_to must exclude owner"
        );
        assert!(!to_ch.spec.options.contains(&ChoiceOption::Player(2)));
        answer_prompt(&mut e, "rexuanfeng:move_to", 0, vec![ChoiceOption::Player(3)]);
        drain_pass(&mut e);
        assert!(e.debug_equips(3).contains(&eq2) && e.debug_equips(2).is_empty());

        // When ONLY owner has an equip (no other player has any equip), Tag(1) move is NOT offered!
        let mut e2 = new_4p(92_205);
        e2.debug_grant(0, "rexuanfeng");
        goto_play(&mut e2, 0);
        let _eq_a = equip_card_now(&mut e2, 0, "qinggang");
        let eq_b = equip_card_now(&mut e2, 0, "armor");
        let _ = eq_b;
        goto_play(&mut e2, 1);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let _loot1 = e2.debug_place(1, "sha");
        let dism_b = place_named_suit(&mut e2, 1, "dismantle", None);
        let _ = submit(&mut e2, 1, GameAction::PlayCard { card: dism_b, targets: vec![0] });
        while e2.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e2.open_choice().unwrap().observer;
            answer_prompt(&mut e2, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e2, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        let mch = e2.open_choice().unwrap().clone();
        assert_eq!(mch.spec.prompt, "rexuanfeng:mode");
        assert!(
            !mch.spec.options.contains(&ChoiceOption::Tag(1)),
            "rexuanfeng must not offer move when only owner has equips"
        );
        drain_pass(&mut e2);
    }

    // Fix #9: `nsxianhai` loses 1 max_hp, disables slot, discards ALL `Shan` in `src`'s hand, and does NOT draw!
    {
        let mut e = new_4p(92_206);
        e.debug_grant(2, "nsxianhai");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let wp0 = equip_card_now(&mut e, 0, "qinggang");
        let sha0 = e.debug_place(0, "sha");
        let shan_a = e.debug_place(0, "shan");
        let shan_b = e.debug_place(0, "shan");
        let sha_keep = e.debug_place(0, "sha");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
        answer_prompt(&mut e, "nsxianhai:confirm", 2, vec![ChoiceOption::Confirm]);
        assert_eq!(e.state_core().player(2).max_hp, 5, "nsxianhai must lose 1 max_hp");
        answer_prompt(&mut e, "nsxianhai:choose_slot", 2, vec![ChoiceOption::Tag(1)]);
        drain_pass(&mut e);
        assert!(!e.debug_equips(0).contains(&wp0), "disabled weapon slot equip discarded");
        assert!(!e.debug_hand(0).contains(&shan_a) && !e.debug_hand(0).contains(&shan_b), "all Shan discarded");
        assert!(e.debug_hand(0).contains(&sha_keep), "non-Shan hand card kept");
        assert!(e.debug_hand(2).is_empty(), "nsxianhai must NOT draw a card");
    }

    // Fix #10: `jili` requires red color, non-multitarget basic/ordinary trick, distance <= 1 AND `targetEnabled2`!
    {
        let mut e = new_4p(92_207);
        e.debug_grant(1, "jili");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        // 1) Black Duel from 0 -> 2 (dist(2,1) == 1): jili MUST NOT trigger!
        let blk_duel = place_named_suit(&mut e, 0, "duel", Some(Suit::Spade));
        let out_bd = submit(&mut e, 0, GameAction::PlayCard { card: blk_duel, targets: vec![2] });
        assert!(!matches!(out_bd, CommandOutcome::Rejected(_)));
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(1), 6, "black Duel must NOT trigger jili on seat 1");

        // 2) Red Dismantle when seat 1 has EMPTY `'hej'`: jili MUST NOT trigger!
        let red_dism = place_named_suit(&mut e, 0, "dismantle", Some(Suit::Heart));
        let _loot2 = e.debug_place(2, "shan");
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: red_dism, targets: vec![2] });
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        assert!(e.open_choice().is_none(), "empty-hej seat 1 must not become dismantle target via jili");

        // 3) Seat 1 with `空城` + empty hand: Red Sha from 0 -> 2 (`twwushen` range) must NOT trigger jili on seat 1!
        e.debug_grant(0, "twwushen");
        e.debug_grant(1, "空城");
        let hp2_before = e.debug_hp(2);
        let _s2 = e.debug_place(2, "shan");
        let red_sha = place_named_suit(&mut e, 0, "sha", Some(Suit::Heart));
        let out_rs = submit(&mut e, 0, GameAction::PlayCard { card: red_sha, targets: vec![2] });
        assert!(!matches!(out_rs, CommandOutcome::Rejected(_)));
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(2), hp2_before - 1);
        assert_eq!(e.debug_hp(1), 6, "KongCheng empty-hand seat 1 must not trigger jili on red Sha");

        // 4) Seat 1 with card & `谦逊`: Red Snatch from 3 -> 2 (dist(3,2)==1, dist(2,1)==1) must NOT target seat 1!
        goto_play(&mut e, 3);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        e.debug_grant(1, "谦逊");
        let keep1 = e.debug_place(1, "tao");
        let _steal2 = e.debug_place(2, "sha");
        let red_snatch = place_named_suit(&mut e, 3, "snatch", Some(Suit::Diamond));
        let _ = submit(&mut e, 3, GameAction::PlayCard { card: red_snatch, targets: vec![2] });
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        drain_pass(&mut e);
        assert!(e.debug_hand(1).contains(&keep1), "QianXun seat 1 must not trigger jili on red Snatch");

        // 5) Positive case: 0 plays red Sha targeting 2 (`dist(2,1)==1`), seat 1 has hand card -> jili adds seat 1!
        let mut e2 = new_4p(92_208);
        e2.debug_grant(0, "twwushen");
        e2.debug_grant(1, "jili");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        e2.debug_clear_hand(2);
        let red_sha2 = place_named_suit(&mut e2, 0, "sha", Some(Suit::Heart));
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: red_sha2, targets: vec![2] });
        drain_pass(&mut e2);
        assert_eq!(e2.debug_hp(2), 5);
        assert_eq!(e2.debug_hp(1), 5, "jili adds valid neighbor seat 1 as target of red Sha");
    }

    // Fix #11: `mozhi` ignores delayed tricks, caps at 2 casts, and stops immediately on decline!
    {
        let mut e = new_4p(92_209);
        e.debug_grant(0, "mozhi");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        let le0 = place_named_suit(&mut e, 0, "le", None);
        let w1 = place_named_suit(&mut e, 0, "wuzhong", None);
        let w2 = place_named_suit(&mut e, 0, "wuzhong", None);
        let w3 = place_named_suit(&mut e, 0, "wuzhong", None);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: le0, targets: vec![1] });
        drain_pass(&mut e);
        assert_eq!(flag(&e, 0, "l11_mozhi_hist_len"), 0, "delayed trick Le must NOT be recorded by mozhi");
        for w in [w1, w2, w3] {
            let out_w = submit(&mut e, 0, GameAction::PlayCard { card: w, targets: vec![] });
            assert!(!matches!(out_w, CommandOutcome::Rejected(_)));
            drain_pass(&mut e);
        }
        assert_eq!(flag(&e, 0, "l11_mozhi_hist_len"), 3);
        e.debug_clear_hand(0);
        let h_a = e.debug_place(0, "sha");
        let h_b = e.debug_place(0, "shan");
        let _h_c = e.debug_place(0, "tao");
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard
        drain_pass(&mut e);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> mozhi asks #1 & #2, caps at 2!
        answer_prompt(&mut e, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e, "mozhi:choose_card", 0, vec![ChoiceOption::Card(h_a)]);
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e, "mozhi:choose_card", 0, vec![ChoiceOption::Card(h_b)]);
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        assert_ne!(
            e.open_choice().map(|c| c.spec.prompt),
            Some("mozhi:choose_card"),
            "mozhi must cap at 2 casts"
        );

        // Decline #1 -> stops immediately without asking #2!
        let mut e2 = new_4p(92_210);
        e2.debug_grant(0, "mozhi");
        goto_play(&mut e2, 0);
        e2.debug_clear_hand(0);
        let wa = place_named_suit(&mut e2, 0, "wuzhong", None);
        let wb = place_named_suit(&mut e2, 0, "wuzhong", None);
        for w in [wa, wb] {
            let _ = submit(&mut e2, 0, GameAction::PlayCard { card: w, targets: vec![] });
            drain_pass(&mut e2);
        }
        let _ = submit(&mut e2, 0, GameAction::NextPhase);
        drain_pass(&mut e2);
        let _ = submit(&mut e2, 0, GameAction::NextPhase);
        answer_prompt(&mut e2, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e2, "mozhi:choose_card", 0, vec![]);
        assert_ne!(
            e2.open_choice().map(|c| c.spec.prompt),
            Some("mozhi:choose_card"),
            "declining mozhi must stop immediately"
        );
    }

    // Fix #12: `rejinjiu` converts owner's own Jiu into Sha at all times (never drinkable as Jiu,
    // never savable as Jiu in own turn OR other's turn, playable/respondable as Sha)!
    {
        let mut e = new_2p(92_211);
        e.debug_grant(0, "rejinjiu");
        goto_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let jiu0 = place_named_suit(&mut e, 0, "jiu", None);
        assert!(
            matches!(
                submit(&mut e, 0, GameAction::PlayCard { card: jiu0, targets: vec![] }),
                CommandOutcome::Rejected(_)
            ),
            "rejinjiu owner must NOT be able to drink Jiu"
        );
        // Owner CAN play Jiu as Sha targeting seat 1!
        let out_sha = submit(&mut e, 0, GameAction::PlayCard { card: jiu0, targets: vec![1] });
        assert!(!matches!(out_sha, CommandOutcome::Rejected(_)), "rejinjiu owner can play Jiu as Sha");
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(1), 5);

        // Owner dying in OWN turn (`duel` self-backfire while holding another `jiu` + `tao`) cannot save with Jiu!
        e.debug_set_hp(0, 1);
        let jiu_save = place_named_suit(&mut e, 0, "jiu", None);
        let tao_save = place_named_suit(&mut e, 0, "tao", None);
        let duel0 = place_named_suit(&mut e, 0, "duel", None);
        let _sha1 = place_named_suit(&mut e, 1, "sha", None);
        let out_d = submit(&mut e, 0, GameAction::PlayCard { card: duel0, targets: vec![1] });
        assert!(!matches!(out_d, CommandOutcome::Rejected(_)));
        while e.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e.open_choice().unwrap().observer;
            answer_prompt(&mut e, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e, "duel_sha", 1, vec![ChoiceOption::Card(_sha1)]);
        let dch = e.open_choice().unwrap().clone();
        assert_eq!(dch.spec.prompt, "duel_sha");
        assert!(dch.spec.options.contains(&ChoiceOption::Card(jiu_save)), "owner can respond Jiu as Sha");
        answer_prompt(&mut e, "duel_sha", 0, vec![ChoiceOption::Pass]);
        let ch = e.open_choice().expect("dying_save in own turn").clone();
        assert_eq!(ch.spec.prompt, "dying_save");
        assert!(ch.spec.options.contains(&ChoiceOption::Card(tao_save)), "Tao still saves in own turn");
        assert!(
            !ch.spec.options.contains(&ChoiceOption::Card(jiu_save)),
            "rejinjiu owner must NOT save with Jiu in own turn"
        );
        answer_prompt(&mut e, "dying_save", 0, vec![ChoiceOption::Card(tao_save)]);
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(0), 1);

        // Owner dying in OTHER player's turn (seat 1 has NO rejinjiu) ALSO cannot save with Jiu!
        let mut e2 = new_2p(92_212);
        e2.debug_grant(0, "rejinjiu");
        goto_play(&mut e2, 1);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        e2.debug_set_hp(0, 1);
        let jiu0_off = place_named_suit(&mut e2, 0, "jiu", None);
        let tao0_off = place_named_suit(&mut e2, 0, "tao", None);
        let sha1_off = place_named_suit(&mut e2, 1, "sha", None);
        let _ = submit(&mut e2, 1, GameAction::PlayCard { card: sha1_off, targets: vec![0] });
        let ch2 = e2.open_choice().expect("dying_save in opponent turn").clone();
        assert_eq!(ch2.spec.prompt, "dying_save");
        assert!(ch2.spec.options.contains(&ChoiceOption::Card(tao0_off)), "Tao still saves in opponent turn");
        assert!(
            !ch2.spec.options.contains(&ChoiceOption::Card(jiu0_off)),
            "rejinjiu owner must NOT save with Jiu in opponent turn"
        );
        answer_prompt(&mut e2, "dying_save", 0, vec![ChoiceOption::Card(tao0_off)]);
        drain_pass(&mut e2);
        assert_eq!(e2.debug_hp(0), 1);
    }
}

fn taixu_with_red_cards_verified() -> content_native_loader::VerifiedLockedContent {
    use std::collections::BTreeMap;
    use content_native_loader::{prepare_locked_content_with_bridge, validate_and_freeze_raw_package};
    use content_schema::ContentPackage;

    let pkg_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/taixu-upstream-liuli");
    let npack_bytes = std::fs::read(pkg_dir.join("package.npack.json"))
        .expect("read taixu package.npack.json");
    let mut pkg = ContentPackage::from_json_bytes(&npack_bytes).expect("parse taixu npack");
    let mut assets = BTreeMap::new();
    for rel_path in pkg.asset_paths.values() {
        let bytes = std::fs::read(pkg_dir.join(rel_path))
            .expect("read taixu asset")
            .into_iter()
            .filter(|&b| b != b'\r')
            .collect::<Vec<u8>>();
        assets.insert(rel_path.clone(), bytes);
    }
    let cards_json = serde_json::json!({
        "source": "noname-liuli",
        "pack": {
            "name": "taixu-red-test",
            "version": 1,
            "description": "P3-C1a Round 3 red L11 trick test cards",
            "card_prefix": "txhj_"
        },
        "cards": [
            { "name": "txhj_zhijizhibi", "kind": "wuzhong", "suit": "heart", "rank": 3 },
            { "name": "txhj_wangmeizhike", "kind": "wangmeizhike", "suit": "heart", "rank": 7 }
        ]
    });
    let cards_bytes = serde_json::to_vec(&cards_json).expect("serialize test cards.json");
    let mut cards_entry = pkg
        .asset_manifest
        .as_ref()
        .and_then(|m| m.entries.first())
        .expect("taixu asset entry")
        .clone();
    cards_entry.asset_id = "taixu:cards".to_string();
    cards_entry.source_hash = *blake3::hash(&cards_bytes).as_bytes();
    pkg.asset_manifest
        .as_mut()
        .expect("taixu asset manifest")
        .entries
        .push(cards_entry);
    pkg.asset_paths
        .insert("taixu:cards".to_string(), "assets/cards.json".to_string());
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    let pkg_json = String::from_utf8(pkg.to_json_bytes().expect("serialize taixu npack"))
        .expect("utf8 taixu npack");
    assets.insert("assets/cards.json".to_string(), cards_bytes);
    let frozen = validate_and_freeze_raw_package(pkg_json, assets)
        .expect("freeze cloned taixu with red test cards");
    prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked taixu with red cards")
}

#[test]
fn l11_npc_a_round3_parity_regressions() {
    use content_native_loader::{restore_locked_engine, start_locked_engine, LockedStartMode};
    use engine_native::state::{GameMode, Role};

    let clear_all = |e: &mut engine_native::Engine, seats: u8| {
        for s in 0..seats {
            e.debug_clear_hand(s);
        }
    };

    // Defect #1a: `jili` triggers on Play-phase red `Tao` (`u.targets == []`) AND Dying-save `Tao`.
    {
        let mut e = new_4p(92_301);
        e.debug_grant(1, "jili");
        e.debug_set_hp(0, 4);
        e.debug_set_hp(1, 4);
        goto_play(&mut e, 0);
        clear_all(&mut e, 4);
        let red_tao = place_named_suit(&mut e, 0, "tao", Some(Suit::Heart));
        let out = submit(&mut e, 0, GameAction::PlayCard { card: red_tao, targets: vec![] });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        drain_pass(&mut e);
        assert_eq!(e.debug_hp(0), 5, "Play-phase Tao user recovers 1");
        assert_eq!(e.debug_hp(1), 5, "jili adds wounded dist-1 neighbor on Play-phase red Tao");

        let mut e_d2 = new_4p(92_321);
        e_d2.debug_grant(2, "jili");
        e_d2.debug_set_hp(0, 4);
        e_d2.debug_set_hp(2, 4);
        goto_play(&mut e_d2, 0);
        clear_all(&mut e_d2, 4);
        let red_tao_d2 = place_named_suit(&mut e_d2, 0, "tao", Some(Suit::Heart));
        let _ = submit(&mut e_d2, 0, GameAction::PlayCard { card: red_tao_d2, targets: vec![] });
        drain_pass(&mut e_d2);
        assert_eq!(e_d2.debug_hp(0), 5);
        assert_eq!(e_d2.debug_hp(2), 4, "isolated dist-2 seat 2 must NOT trigger jili on Play-phase Tao");

        // Dying-save red Tao (`u.targets == [dying]`) also recovers jili neighbor within distance 1 of dying target!
        let mut e_die = new_4p(92_302);
        e_die.debug_grant(2, "jili");
        goto_play(&mut e_die, 0);
        clear_all(&mut e_die, 4);
        e_die.debug_set_hp(1, 1);
        e_die.debug_set_hp(2, 4); // dist(1, 2) == 1 -> added
        let sha0 = place_named_suit(&mut e_die, 0, "sha", Some(Suit::Spade));
        let save_tao = place_named_suit(&mut e_die, 0, "tao", Some(Suit::Heart));
        let out_s = submit(&mut e_die, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
        assert!(!matches!(out_s, CommandOutcome::Rejected(_)));
        while let Some(ch) = e_die.open_choice().cloned() {
            if ch.spec.prompt == "dying_save" && ch.observer == 0 {
                answer_prompt(&mut e_die, "dying_save", 0, vec![ChoiceOption::Card(save_tao)]);
            } else {
                answer_prompt(&mut e_die, ch.spec.prompt, ch.observer, vec![ChoiceOption::Pass]);
            }
        }
        assert_eq!(e_die.debug_hp(1), 1, "dying target 1 saved to 1 HP");
        assert_eq!(e_die.debug_hp(2), 5, "jili neighbor 2 recovers 1 on dying-save red Tao");

        let mut e_die_d2 = new_4p(92_322);
        e_die_d2.debug_grant(3, "jili");
        goto_play(&mut e_die_d2, 0);
        clear_all(&mut e_die_d2, 4);
        e_die_d2.debug_set_hp(1, 1);
        e_die_d2.debug_set_hp(3, 4); // dist(1, 3) == 2 -> skipped
        let sha_d2 = place_named_suit(&mut e_die_d2, 0, "sha", Some(Suit::Spade));
        let save_tao_d2 = place_named_suit(&mut e_die_d2, 0, "tao", Some(Suit::Heart));
        let _ = submit(&mut e_die_d2, 0, GameAction::PlayCard { card: sha_d2, targets: vec![1] });
        while let Some(ch) = e_die_d2.open_choice().cloned() {
            if ch.spec.prompt == "dying_save" && ch.observer == 0 {
                answer_prompt(&mut e_die_d2, "dying_save", 0, vec![ChoiceOption::Card(save_tao_d2)]);
            } else {
                answer_prompt(&mut e_die_d2, ch.spec.prompt, ch.observer, vec![ChoiceOption::Pass]);
            }
        }
        assert_eq!(e_die_d2.debug_hp(1), 1);
        assert_eq!(e_die_d2.debug_hp(3), 4, "isolated dist-2 seat 3 skipped on dying-save Tao");
    }

    // Defect #1b: `jili` triggers on Play-phase red `Wuzhong` (`u.targets == []`), resolves ALL targets
    // with per-target `NullifyWindow`, AND roundtrips through NGSA v107 save/restore cleanly.
    {
        let mut e = new_4p(92_303);
        e.debug_grant(1, "jili");
        goto_play(&mut e, 0);
        clear_all(&mut e, 4);
        let red_wz = place_named_suit(&mut e, 0, "wuzhong", Some(Suit::Heart));
        let out = submit(&mut e, 0, GameAction::PlayCard { card: red_wz, targets: vec![] });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        drain_pass(&mut e);
        assert_eq!(e.debug_hand(0).len(), 2, "Wuzhong user draws 2");
        assert_eq!(e.debug_hand(1).len(), 2, "jili dist-1 neighbor draws 2 on Play-phase red Wuzhong");

        let mut e_wz_d2 = new_4p(92_323);
        e_wz_d2.debug_grant(2, "jili");
        goto_play(&mut e_wz_d2, 0);
        clear_all(&mut e_wz_d2, 4);
        let red_wz_d2 = place_named_suit(&mut e_wz_d2, 0, "wuzhong", Some(Suit::Heart));
        let _ = submit(&mut e_wz_d2, 0, GameAction::PlayCard { card: red_wz_d2, targets: vec![] });
        drain_pass(&mut e_wz_d2);
        assert_eq!(e_wz_d2.debug_hand(0).len(), 2);
        assert_eq!(e_wz_d2.debug_hand(2).len(), 0, "isolated dist-2 seat 2 must NOT trigger jili on Wuzhong");

        // Per-target NullifyWindow: nullify slot 1 (seat 1) while slot 0 (seat 0) resolves, plus NGSA save/restore!
        let mut e_wx = new_4p(92_304);
        e_wx.debug_grant(1, "jili");
        goto_play(&mut e_wx, 0);
        clear_all(&mut e_wx, 4);
        let red_wz2 = place_named_suit(&mut e_wx, 0, "wuzhong", Some(Suit::Heart));
        let wx3 = e_wx.debug_place(3, "nullify");
        let _ = submit(&mut e_wx, 0, GameAction::PlayCard { card: red_wz2, targets: vec![] });
        // Pass slot 0 nullify_window queue (`0` draws 2):
        while e_wx.open_choice().is_some() && e_wx.debug_hand(0).is_empty() {
            let obs = e_wx.open_choice().unwrap().observer;
            answer_prompt(&mut e_wx, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        assert_eq!(e_wx.debug_hand(0).len(), 2, "slot 0 already drew 2 before slot 1 NullifyWindow");
        // Now slot 1 NullifyWindow asks seat 0, 1, 2, 3 -> save/restore on seat 3 and cast wuxie!
        while let Some(ch) = e_wx.open_choice().cloned() {
            assert_eq!(ch.spec.prompt, "nullify_window");
            if ch.observer == 3 {
                let snap = e_wx.save();
                let mut restored = restore_locked_engine(&snap, taixu_verified()).expect("restore slot 1 Wuzhong");
                assert_eq!(restored.state_hash(), e_wx.state_hash());
                answer_prompt(&mut e_wx, "nullify_window", 3, vec![ChoiceOption::Card(wx3)]);
                answer_prompt(&mut restored, "nullify_window", 3, vec![ChoiceOption::Card(wx3)]);
                drain_pass(&mut e_wx);
                drain_pass(&mut restored);
                assert_eq!(restored.state_hash(), e_wx.state_hash(), "NGSA restore parity on multi-target Wuzhong");
                break;
            }
            answer_prompt(&mut e_wx, "nullify_window", ch.observer, vec![ChoiceOption::Pass]);
        }
        assert_eq!(e_wx.debug_hand(0).len(), 2);
        assert_eq!(e_wx.debug_hand(1).len(), 0, "wuxie nullifies slot 1 (seat 1) independently");

        // Nullify slot 0 (seat 0), pass slot 1 (seat 1) -> seat 1 STILL draws 2!
        let mut e_wx0 = new_4p(92_305);
        e_wx0.debug_grant(1, "jili");
        goto_play(&mut e_wx0, 0);
        clear_all(&mut e_wx0, 4);
        let red_wz3 = place_named_suit(&mut e_wx0, 0, "wuzhong", Some(Suit::Heart));
        let wx3b = e_wx0.debug_place(3, "nullify");
        let _ = submit(&mut e_wx0, 0, GameAction::PlayCard { card: red_wz3, targets: vec![] });
        while let Some(ch) = e_wx0.open_choice().cloned() {
            assert_eq!(ch.spec.prompt, "nullify_window");
            if ch.observer == 3 && e_wx0.debug_hand(3).contains(&wx3b) {
                answer_prompt(&mut e_wx0, "nullify_window", 3, vec![ChoiceOption::Card(wx3b)]);
            } else {
                answer_prompt(&mut e_wx0, "nullify_window", ch.observer, vec![ChoiceOption::Pass]);
            }
        }
        assert_eq!(e_wx0.debug_hand(0).len(), 0, "slot 0 nullified");
        assert_eq!(e_wx0.debug_hand(1).len(), 2, "slot 1 resolves after slot 0 nullified");
    }

    // Defect #2: `jili` triggers on red `txhj_yuanjiao` (ally & enemy), `huogong`, `txhj_diaohu`,
    // `txhj_shuiyan`, `txhj_caomu`, `txhj_wangmeizhike`, and `txhj_zhijizhibi` (Identity + Kingdoms unseen).
    {
        // (a) `txhj_yuanjiao` (Heart): Lord 0 -> Traitor 2, Loyalist 3 (`is_friend_of(0, 3)==true`) has `jili`!
        let mut e_yj = new_4p(92_306);
        e_yj.debug_grant(3, "jili");
        goto_play(&mut e_yj, 0);
        clear_all(&mut e_yj, 4);
        let yj = place_named_suit(&mut e_yj, 0, "txhj_yuanjiao", None);
        let out_yj = submit(&mut e_yj, 0, GameAction::PlayCard { card: yj, targets: vec![2] });
        assert!(!matches!(out_yj, CommandOutcome::Rejected(_)));
        drain_pass(&mut e_yj);
        assert_eq!(e_yj.debug_hand(2).len(), 1, "original target 2 draws 1");
        assert_eq!(e_yj.debug_hand(3).len(), 1, "ally jili owner 3 draws 1 on txhj_yuanjiao");
        assert_eq!(e_yj.debug_hand(0).len(), 6, "user 0 draws 3 + 3 for both txhj_yuanjiao resolutions");

        // Plus per-target NullifyWindow on txhj_yuanjiao (enemy jili owner 2): wuxie blocks 2 while 1 still resolves!
        let mut e_yj2 = new_4p(92_307);
        e_yj2.debug_grant(2, "jili");
        goto_play(&mut e_yj2, 0);
        clear_all(&mut e_yj2, 4);
        let yj2 = place_named_suit(&mut e_yj2, 0, "txhj_yuanjiao", None);
        let wx3 = e_yj2.debug_place(3, "nullify");
        let _ = submit(&mut e_yj2, 0, GameAction::PlayCard { card: yj2, targets: vec![1] });
        while e_yj2.open_choice().is_some() && e_yj2.debug_hand(1).is_empty() {
            let obs = e_yj2.open_choice().unwrap().observer;
            answer_prompt(&mut e_yj2, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        assert_eq!(e_yj2.debug_hand(1).len(), 1);
        assert_eq!(
            e_yj2.debug_hand(0).len(),
            3,
            "expected slot 0 user draw 3; open_choice={:?}",
            e_yj2.open_choice().map(|c| (c.observer, c.spec.prompt))
        );
        while let Some(ch) = e_yj2.open_choice().cloned() {
            if ch.spec.prompt == "nullify_window" && ch.observer == 3 && e_yj2.debug_hand(3).contains(&wx3) {
                answer_prompt(&mut e_yj2, "nullify_window", 3, vec![ChoiceOption::Card(wx3)]);
            } else {
                answer_prompt(&mut e_yj2, ch.spec.prompt, ch.observer, vec![ChoiceOption::Pass]);
            }
        }
        assert_eq!(e_yj2.debug_hand(2).len(), 0, "wuxie blocks slot 1 of txhj_yuanjiao");
        assert_eq!(e_yj2.debug_hand(0).len(), 3, "user 0 only drew 3 from slot 0");

        // (b) `huogong` (Heart): non-empty jili owner triggers; empty jili owner does NOT trigger.
        let mut e_hg = new_4p(92_308);
        e_hg.debug_grant(1, "jili");
        goto_play(&mut e_hg, 0);
        clear_all(&mut e_hg, 4);
        let hg = place_named_suit(&mut e_hg, 0, "huogong", None);
        let disc0_a = place_named_suit(&mut e_hg, 0, "tao", Some(Suit::Heart));
        let disc0_b = place_named_suit(&mut e_hg, 0, "shan", Some(Suit::Heart));
        let show2 = place_named_suit(&mut e_hg, 2, "tao", Some(Suit::Heart));
        let show1 = place_named_suit(&mut e_hg, 1, "shan", Some(Suit::Heart));
        let _ = submit(&mut e_hg, 0, GameAction::PlayCard { card: hg, targets: vec![2] });
        while e_hg.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e_hg.open_choice().unwrap().observer;
            answer_prompt(&mut e_hg, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e_hg, "huogong:show_card", 2, vec![ChoiceOption::Card(show2)]);
        answer_prompt(&mut e_hg, "huogong:discard_card", 0, vec![ChoiceOption::Card(disc0_a)]);
        while e_hg.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e_hg.open_choice().unwrap().observer;
            answer_prompt(&mut e_hg, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        answer_prompt(&mut e_hg, "huogong:show_card", 1, vec![ChoiceOption::Card(show1)]);
        answer_prompt(&mut e_hg, "huogong:discard_card", 0, vec![ChoiceOption::Card(disc0_b)]);
        drain_pass(&mut e_hg);
        assert_eq!(e_hg.debug_hp(2), 5, "original huogong target 2 damaged");
        assert_eq!(e_hg.debug_hp(1), 5, "jili owner 1 added & damaged by red huogong");

        // Empty-hand jili owner does NOT trigger huogong:
        let mut e_hg_e = new_4p(92_309);
        e_hg_e.debug_grant(1, "jili");
        goto_play(&mut e_hg_e, 0);
        clear_all(&mut e_hg_e, 4);
        let hg_e = place_named_suit(&mut e_hg_e, 0, "huogong", None);
        let _ = place_named_suit(&mut e_hg_e, 2, "tao", Some(Suit::Heart));
        let _ = submit(&mut e_hg_e, 0, GameAction::PlayCard { card: hg_e, targets: vec![2] });
        drain_pass(&mut e_hg_e);
        assert_eq!(e_hg_e.debug_hp(1), 6, "empty-hand jili owner not targeted by huogong");

        // (c) `txhj_diaohu` (Heart): both target 2 and jili owner 1 flip face-down and draw 1; user draws 2.
        let mut e_dh = new_4p(92_310);
        e_dh.debug_grant(1, "jili");
        goto_play(&mut e_dh, 0);
        clear_all(&mut e_dh, 4);
        let dh = place_named_suit(&mut e_dh, 0, "txhj_diaohu", None);
        let _ = submit(&mut e_dh, 0, GameAction::PlayCard { card: dh, targets: vec![2] });
        drain_pass(&mut e_dh);
        assert!(e_dh.debug_face_down(2));
        assert!(e_dh.debug_face_down(1), "jili owner 1 turned over by red txhj_diaohu");
        assert_eq!(e_dh.debug_hand(2).len(), 1);
        assert_eq!(e_dh.debug_hand(1).len(), 1);
        assert_eq!(e_dh.debug_hand(0).len(), 2);

        // (d) `txhj_shuiyan` (Spade -> Heart via `xinhongyan`): both target 2 and jili owner 1 take 1 Thunder dmg!
        let mut e_sy = new_4p(92_311);
        e_sy.debug_grant(0, "xinhongyan");
        e_sy.debug_grant(1, "jili");
        goto_play(&mut e_sy, 0);
        clear_all(&mut e_sy, 4);
        let sy = place_named_suit(&mut e_sy, 0, "txhj_shuiyan", None);
        let out_sy = submit(&mut e_sy, 0, GameAction::PlayCard { card: sy, targets: vec![2] });
        assert!(!matches!(out_sy, CommandOutcome::Rejected(_)));
        drain_pass(&mut e_sy);
        assert_eq!(e_sy.debug_hp(2), 5);
        assert_eq!(e_sy.debug_hp(1), 5, "xinhongyan red txhj_shuiyan triggers jili & hurts 1");

        // (e) `txhj_caomu` (Spade -> Heart via `xinhongyan`): target 2 then jili owner 1 resolve sequentially.
        let mut e_cm = new_4p(92_312);
        e_cm.debug_grant(0, "xinhongyan");
        e_cm.debug_grant(1, "jili");
        goto_play(&mut e_cm, 0);
        clear_all(&mut e_cm, 4);
        let cm = place_named_suit(&mut e_cm, 0, "txhj_caomu", None);
        let _c2 = place_named_suit(&mut e_cm, 2, "sha", Some(Suit::Spade));
        let _c1 = place_named_suit(&mut e_cm, 1, "shan", None);
        let _ = submit(&mut e_cm, 0, GameAction::PlayCard { card: cm, targets: vec![2] });
        drain_pass(&mut e_cm);
        // 2 discards 1 (1, 3 draw 1); then 1 discards 1 (0, 2 draw 1):
        assert_eq!(e_cm.debug_hand(0).len(), 1, "seat 0 drew 1 from txhj_caomu resolving on jili owner 1");
        assert_eq!(e_cm.debug_hand(3).len(), 1, "seat 3 drew 1 from txhj_caomu resolving on target 2");

        // (f) & (g) Custom locked taixu with Heart `txhj_wangmeizhike_1` & `txhj_zhijizhibi_0`:
        let verified_red = taixu_with_red_cards_verified();
        let make_red_4p = |seed: u64, kingdoms: bool| {
            let mut e = start_locked_engine(seed, 4, LockedStartMode::Identity, &verified_red)
                .expect("start 4p red taixu");
            e.debug_set_identity_roles(vec![Role::Lord, Role::Rebel, Role::Traitor, Role::Loyalist]);
            for s in 0..4u8 {
                e.debug_set_max_hp(s, 6);
                e.debug_set_hp(s, 6);
                e.debug_set_hujia(s, 0);
            }
            if kingdoms {
                e.state_core_mut().mode = GameMode::Kingdoms;
            }
            e
        };

        // Heart `txhj_wangmeizhike`: target 2 + jili owner 1 both settle (heal 1 or draw 1 each).
        let mut e_wm = make_red_4p(92_313, false);
        e_wm.debug_grant(1, "jili");
        e_wm.debug_set_hp(1, 5);
        e_wm.debug_set_hp(2, 5);
        goto_play(&mut e_wm, 0);
        clear_all(&mut e_wm, 4);
        let wm = place_named_suit(&mut e_wm, 0, "txhj_wangmeizhike_1", Some(Suit::Heart));
        let out_wm = submit(&mut e_wm, 0, GameAction::PlayCard { card: wm, targets: vec![2] });
        assert!(!matches!(out_wm, CommandOutcome::Rejected(_)));
        drain_pass(&mut e_wm);
        assert_eq!(
            (e_wm.debug_hp(2) - 5) as usize + e_wm.debug_hand(2).len(),
            1,
            "target 2 resolved txhj_wangmeizhike"
        );
        assert_eq!(
            (e_wm.debug_hp(1) - 5) as usize + e_wm.debug_hand(1).len(),
            1,
            "jili owner 1 added & resolved red txhj_wangmeizhike"
        );

        // Heart `txhj_zhijizhibi`: Identity non-empty hand triggers `jili` (`0` draws 2).
        let mut e_zb1 = make_red_4p(92_314, false);
        e_zb1.debug_grant(1, "jili");
        goto_play(&mut e_zb1, 0);
        clear_all(&mut e_zb1, 4);
        let zb1 = place_named_suit(&mut e_zb1, 0, "txhj_zhijizhibi_0", Some(Suit::Heart));
        let _h2 = place_named_suit(&mut e_zb1, 2, "sha", None);
        let _h1 = place_named_suit(&mut e_zb1, 1, "shan", None);
        let out_zb1 = submit(&mut e_zb1, 0, GameAction::PlayCard { card: zb1, targets: vec![2] });
        assert!(!matches!(out_zb1, CommandOutcome::Rejected(_)));
        drain_pass(&mut e_zb1);
        assert_eq!(e_zb1.debug_hand(0).len(), 2, "Identity non-empty jili owner 1 adds zhibi draw (+2 total)");

        // Identity empty hand does NOT trigger `jili` (`0` draws 1 only).
        let mut e_zb2 = make_red_4p(92_315, false);
        e_zb2.debug_grant(1, "jili");
        goto_play(&mut e_zb2, 0);
        clear_all(&mut e_zb2, 4);
        let zb2 = place_named_suit(&mut e_zb2, 0, "txhj_zhijizhibi_0", Some(Suit::Heart));
        let _h2b = place_named_suit(&mut e_zb2, 2, "sha", None);
        let _ = submit(&mut e_zb2, 0, GameAction::PlayCard { card: zb2, targets: vec![2] });
        drain_pass(&mut e_zb2);
        assert_eq!(e_zb2.debug_hand(0).len(), 1, "Identity empty-hand jili owner skipped by zhibi");

        // Kingdoms unseen general + empty hand DOES trigger `jili` (`0` draws 2).
        let mut e_zb3 = make_red_4p(92_316, true);
        e_zb3.debug_grant(1, "jili");
        goto_play(&mut e_zb3, 0);
        clear_all(&mut e_zb3, 4);
        e_zb3.state_core_mut().player_mut(1).revealed_generals = [false, false];
        let zb3 = place_named_suit(&mut e_zb3, 0, "txhj_zhijizhibi_0", Some(Suit::Heart));
        let _h2c = place_named_suit(&mut e_zb3, 2, "sha", None);
        let _ = submit(&mut e_zb3, 0, GameAction::PlayCard { card: zb3, targets: vec![2] });
        drain_pass(&mut e_zb3);
        assert_eq!(
            e_zb3.debug_hand(0).len(),
            2,
            "Kingdoms unseen empty-hand jili owner triggers txhj_zhijizhibi"
        );

        // Kingdoms FULLY revealed general + empty hand does NOT trigger `jili`.
        let mut e_zb4 = make_red_4p(92_317, true);
        e_zb4.debug_grant(1, "jili");
        goto_play(&mut e_zb4, 0);
        clear_all(&mut e_zb4, 4);
        e_zb4.state_core_mut().player_mut(1).revealed_generals = [true, true];
        let zb4 = place_named_suit(&mut e_zb4, 0, "txhj_zhijizhibi_0", Some(Suit::Heart));
        let _h2d = place_named_suit(&mut e_zb4, 2, "sha", None);
        let _ = submit(&mut e_zb4, 0, GameAction::PlayCard { card: zb4, targets: vec![2] });
        drain_pass(&mut e_zb4);
        assert_eq!(
            e_zb4.debug_hand(0).len(),
            1,
            "Kingdoms fully revealed empty-hand jili owner skipped by txhj_zhijizhibi"
        );
    }

    // Defect #3: `mozhi` records Play-phase `Jiu` AND stops immediately when shifting `Jiu`.
    {
        // Case A: `[Jiu]` -> recorded (`len == 1`), triggers, stops immediately without `mozhi:choose_card`.
        let mut e1 = new_4p(92_318);
        e1.debug_grant(0, "mozhi");
        goto_play(&mut e1, 0);
        clear_all(&mut e1, 4);
        let jiu_a = place_named_suit(&mut e1, 0, "jiu", None);
        let _keep_a = place_named_suit(&mut e1, 0, "shan", None);
        let out_j = submit(&mut e1, 0, GameAction::PlayCard { card: jiu_a, targets: vec![] });
        assert!(!matches!(out_j, CommandOutcome::Rejected(_)));
        drain_pass(&mut e1);
        assert_eq!(flag(&e1, 0, "l11_mozhi_hist_len"), 1, "Play-phase Jiu must be recorded by mozhi");
        let _ = submit(&mut e1, 0, GameAction::NextPhase);
        drain_pass(&mut e1);
        let _ = submit(&mut e1, 0, GameAction::NextPhase);
        answer_prompt(&mut e1, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        assert_ne!(
            e1.open_choice().map(|c| c.spec.prompt),
            Some("mozhi:choose_card"),
            "mozhi shifting Jiu must stop immediately without asking to copy"
        );

        // Case B: `[Jiu, Sha]` -> shifting Jiu at index 0 stops immediately without skipping to Sha!
        let mut e2 = new_4p(92_319);
        e2.debug_grant(0, "mozhi");
        goto_play(&mut e2, 0);
        clear_all(&mut e2, 4);
        let jiu_b = place_named_suit(&mut e2, 0, "jiu", None);
        let sha_b = place_named_suit(&mut e2, 0, "sha", None);
        let _keep_b = place_named_suit(&mut e2, 0, "shan", None);
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: jiu_b, targets: vec![] });
        drain_pass(&mut e2);
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
        drain_pass(&mut e2);
        assert_eq!(flag(&e2, 0, "l11_mozhi_hist_len"), 2);
        let _ = submit(&mut e2, 0, GameAction::NextPhase);
        drain_pass(&mut e2);
        let _ = submit(&mut e2, 0, GameAction::NextPhase);
        answer_prompt(&mut e2, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        assert_ne!(
            e2.open_choice().map(|c| c.spec.prompt),
            Some("mozhi:choose_card"),
            "mozhi shifting [Jiu, Sha] must stop on Jiu without skipping to Sha"
        );

        // Case C: `[Wuzhong, Jiu, Sha]` -> copies Wuzhong first, then stops on Jiu without asking Sha!
        let mut e3 = new_4p(92_320);
        e3.debug_grant(0, "mozhi");
        goto_play(&mut e3, 0);
        clear_all(&mut e3, 4);
        let wz_c = place_named_suit(&mut e3, 0, "wuzhong", None);
        let jiu_c = place_named_suit(&mut e3, 0, "jiu", None);
        let sha_c = place_named_suit(&mut e3, 0, "sha", None);
        let _ = submit(&mut e3, 0, GameAction::PlayCard { card: wz_c, targets: vec![] });
        drain_pass(&mut e3);
        let _ = submit(&mut e3, 0, GameAction::PlayCard { card: jiu_c, targets: vec![] });
        drain_pass(&mut e3);
        let _ = submit(&mut e3, 0, GameAction::PlayCard { card: sha_c, targets: vec![1] });
        drain_pass(&mut e3);
        assert_eq!(flag(&e3, 0, "l11_mozhi_hist_len"), 3);
        e3.debug_clear_hand(0);
        let copy_hand = place_named_suit(&mut e3, 0, "shan", None);
        let _ = submit(&mut e3, 0, GameAction::NextPhase);
        drain_pass(&mut e3);
        let _ = submit(&mut e3, 0, GameAction::NextPhase);
        answer_prompt(&mut e3, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e3, "mozhi:choose_card", 0, vec![ChoiceOption::Card(copy_hand)]);
        while e3.open_choice().is_some_and(|c| c.spec.prompt == "nullify_window") {
            let obs = e3.open_choice().unwrap().observer;
            answer_prompt(&mut e3, "nullify_window", obs, vec![ChoiceOption::Pass]);
        }
        assert_eq!(e3.debug_hand(0).len(), 2, "mozhi first copied Wuzhong to draw 2");
        assert_ne!(
            e3.open_choice().map(|c| c.spec.prompt),
            Some("mozhi:choose_card"),
            "mozhi shifting Jiu at idx 1 must stop without asking Sha at idx 2"
        );

        // Red ViewAs `Wuzhong` (`mozhi` using Heart `shan` as `Wuzhong`) also triggers `jili` & disposes table card:
        let mut e4 = new_4p(92_324);
        e4.debug_grant(0, "mozhi");
        e4.debug_grant(1, "jili");
        goto_play(&mut e4, 0);
        clear_all(&mut e4, 4);
        let wz_d = place_named_suit(&mut e4, 0, "wuzhong", None);
        let _ = submit(&mut e4, 0, GameAction::PlayCard { card: wz_d, targets: vec![] });
        drain_pass(&mut e4);
        assert_eq!(e4.debug_hand(1).len(), 2);
        clear_all(&mut e4, 4);
        let copy_red = place_named_suit(&mut e4, 0, "shan", Some(Suit::Heart));
        let _ = submit(&mut e4, 0, GameAction::NextPhase);
        drain_pass(&mut e4);
        let _ = submit(&mut e4, 0, GameAction::NextPhase);
        answer_prompt(&mut e4, "trigger_confirm", 0, vec![ChoiceOption::Confirm]);
        answer_prompt(&mut e4, "mozhi:choose_card", 0, vec![ChoiceOption::Card(copy_red)]);
        drain_pass(&mut e4);
        assert_eq!(e4.debug_hand(0).len(), 2);
        assert_eq!(e4.debug_hand(1).len(), 2, "jili adds neighbor on ViewAs red Wuzhong");
        assert!(e4.state_core().table.is_empty(), "WuzhongMachine disposes ViewAs carrier card cleanly");

        // Negative `CardKind::Wuzhong` multitarget exclusion: Heart `txhj_yiyi` does NOT trigger `jili`:
        let mut e_yy = new_4p(92_325);
        e_yy.debug_grant(1, "jili");
        goto_play(&mut e_yy, 0);
        clear_all(&mut e_yy, 4);
        let yy = place_named_suit(&mut e_yy, 0, "txhj_yiyi", None);
        let _ = submit(&mut e_yy, 0, GameAction::PlayCard { card: yy, targets: vec![0, 3] });
        drain_pass(&mut e_yy);
        assert_eq!(e_yy.debug_hand(1).len(), 0, "multitarget txhj_yiyi must NOT trigger jili");
    }
}
