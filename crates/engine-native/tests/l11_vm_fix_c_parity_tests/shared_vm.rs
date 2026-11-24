use super::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn l11_vm_c_audit36_baseline_ok_and_shared_vm_part1() {
    // OK-1 `chouhai` + Fixed `rechouhai`: Sha damage +1 when hand empty; Duel untouched; with hand card untouched.
    for sk in ["chouhai", "rechouhai"] {
        let mut e = start_2p(2026_9401);
        e.debug_grant(1, sk);
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let hp_pre = e.state_core().player(1).hp;
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, hp_pre - 2, "{sk} adds +1 Sha dmg on empty hand");

        let mut e2 = start_2p(2026_9402);
        e2.debug_grant(1, sk);
        advance_to_phase(&mut e2, 0, Phase::Play, None);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let _keep = place_hand_kind_any(&mut e2, 1, CardKind::Tao);
        let sha2 = place_hand_kind_any(&mut e2, 0, CardKind::Sha);
        let hp2 = e2.state_core().player(1).hp;
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
        drain_pass(&mut e2, 16);
        assert_eq!(e2.state_core().player(1).hp, hp2 - 1, "{sk} does NOT boost when target holds cards");
    }

    // OK-2 `jiaozi`: strict unique max hand across ALL alive players grants +1 dmg dealt & taken.
    {
        let mut e = start_2p(2026_9403);
        e.debug_grant(0, "jiaozi");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _k = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let _p1 = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        // After playing `sha`, seat 0 has 1 card == seat 1's 1 card -> NOT strict max! Give extra card to seat 0.
        let _extra = place_hand_kind_any(&mut e, 0, CardKind::Tao);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        drain_pass(&mut e, 16);
        assert_eq!(e.state_core().player(1).hp, 2, "jiaozi strict max hand deals +1 dmg");
    }

    // OK-3 `roulin`: Sha targeting Female needs 2 Shan.
    {
        let mut e = start_2p(2026_9404);
        e.debug_grant(0, "roulin");
        e.assign_general_named(1, "甄姬").unwrap();
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let s1 = place_hand_kind_any(&mut e, 1, CardKind::Shan);
        let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let _ch = e.open_choice().cloned().expect("1st shan");
        let _ = answer(&mut e, 1, vec![ChoiceOption::Card(s1)]);
        drain_pass(&mut e, 16);
        assert!(e.state_core().discard.contains(&s1), "1st Shan was spent");
        assert_eq!(e.state_core().player(1).hp, 2, "roulin requires 2nd Shan vs Female (甄姬 starts 3 HP -> 2 HP)");

        // With 2 Shans, Female dodges cleanly!
        let mut e2 = start_2p(2026_9414);
        e2.debug_grant(0, "roulin");
        e2.assign_general_named(1, "甄姬").unwrap();
        advance_to_phase(&mut e2, 0, Phase::Play, None);
        e2.debug_clear_hand(0);
        e2.debug_clear_hand(1);
        let sha2 = place_hand_kind_any(&mut e2, 0, CardKind::Sha);
        let sh_a = place_hand_kind_any(&mut e2, 1, CardKind::Shan);
        let sh_b = place_hand_kind_any(&mut e2, 1, CardKind::Shan);
        let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
        let _ = answer(&mut e2, 1, vec![ChoiceOption::Card(sh_a)]);
        let _ = answer(&mut e2, 1, vec![ChoiceOption::Card(sh_b)]);
        drain_pass(&mut e2, 16);
        assert_eq!(e2.state_core().player(1).hp, 3, "2 Shans dodge roulin Sha completely");
    }

    // Fixed `duanliang`, `new_rewusheng`, `jiuchi`, `xiaoxi_hansui`, `zymoucheng`: natural PlayCard + ViewAs!
    {
        // `duanliang`: black basic/equip as Bing + unlimited distance if target.hand >= owner.hand; natural equips still equip!
        let mut e = start_3p(2026_9405);
        e.debug_grant(0, "duanliang");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        equip_kind_suit(&mut e, 2, CardKind::HorsePlus, None);
        let eq_blk = place_card_suit_rank(&mut e, 0, Suit::Spade, CardKind::Armor, 1, 13);
        let _p2 = place_hand_kind_any(&mut e, 2, CardKind::Sha);
        let _p2b = place_hand_kind_any(&mut e, 2, CardKind::Shan);
        // Normal PlayCard `[ ]` on black equip MUST equip naturally without duanliang hijacking!
        let eq_res = submit(&mut e, 0, GameAction::PlayCard { card: eq_blk, targets: vec![] });
        assert!(!matches!(eq_res, CommandOutcome::Rejected(_)));
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(0).equips, vec![eq_blk], "duanliang must NOT hijack natural Armor equip");
        // ViewAs Bing via PlayCard targeting seat 2 (dist=2, seat2.hand >= seat0.hand)!
        let sha_blk = place_card_suit_rank(&mut e, 0, Suit::Club, CardKind::Sha, 1, 13);
        let bing_res = submit(&mut e, 0, GameAction::PlayCard { card: sha_blk, targets: vec![2] });
        assert!(!matches!(bing_res, CommandOutcome::Rejected(_)), "duanliang black Sha as Bing must hit seat 2");
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(2).judge, vec![sha_blk]);

        // `duanliang` + `paoxiao`: 2nd black Sha targeting in-range seat 1 plays naturally as Sha; out-of-range seat 2 converts to Bing!
        let mut edp = start_3p(2026_9415);
        edp.debug_grant(0, "duanliang");
        edp.debug_grant(0, "咆哮");
        equip_kind_suit(&mut edp, 2, CardKind::HorsePlus, None);
        advance_to_phase(&mut edp, 0, Phase::Play, None);
        edp.debug_clear_hand(0);
        let s_first = place_hand_kind_any(&mut edp, 0, CardKind::Sha);
        let s_blk_in = place_card_suit_rank(&mut edp, 0, Suit::Spade, CardKind::Sha, 1, 13);
        let s_blk_far = place_card_suit_rank(&mut edp, 0, Suit::Club, CardKind::Sha, 1, 13);
        let _ = submit(&mut edp, 0, GameAction::PlayCard { card: s_first, targets: vec![1] });
        drain_pass(&mut edp, 8);
        let _ = submit(&mut edp, 0, GameAction::PlayCard { card: s_blk_in, targets: vec![1] });
        drain_pass(&mut edp, 8);
        assert_eq!(edp.state_core().player(1).hp, 2, "2nd black Sha with paoxiao+duanliang targeting dist=1 must play as Sha");
        assert!(edp.state_core().player(1).judge.is_empty(), "must NOT hijack natural 2nd Sha into Bing");
        let _ = submit(&mut edp, 0, GameAction::PlayCard { card: s_blk_far, targets: vec![2] });
        drain_pass(&mut edp, 8);
        assert_eq!(edp.state_core().player(2).judge, vec![s_blk_far], "out-of-attack-range black Sha still converts to Bing");

        // `new_rewusheng`: Diamond Sha unlimited distance, natural Diamond Tao heals!
        let mut ew = start_3p(2026_9406);
        ew.debug_grant(0, "new_rewusheng");
        ew.debug_set_hp(0, 2);
        equip_kind_suit(&mut ew, 0, CardKind::HorsePlus, None);
        equip_kind_suit(&mut ew, 2, CardKind::HorsePlus, None);
        advance_to_phase(&mut ew, 0, Phase::Play, None);
        ew.debug_clear_hand(0);
        ew.debug_clear_hand(2);
        let d_tao = place_card_suit_rank(&mut ew, 0, Suit::Diamond, CardKind::Tao, 1, 13);
        let _ = submit(&mut ew, 0, GameAction::PlayCard { card: d_tao, targets: vec![] });
        drain_pass(&mut ew, 8);
        assert_eq!(ew.state_core().player(0).hp, 3, "Diamond Tao naturally heals without new_rewusheng hijacking");
        let d_shan = place_card_suit_rank(&mut ew, 0, Suit::Diamond, CardKind::Shan, 1, 13);
        let _ = submit(&mut ew, 0, GameAction::PlayCard { card: d_shan, targets: vec![2] });
        drain_pass(&mut ew, 16);
        assert_eq!(ew.state_core().player(2).hp, 3, "Diamond Sha via new_rewusheng ignores distance");

        // `jiuchi`: Spade plays as Jiu (Play or Dying) while Spade equip equips naturally!
        let mut ej = start_2p(2026_9407);
        ej.debug_grant(0, "jiuchi");
        advance_to_phase(&mut ej, 0, Phase::Play, None);
        ej.debug_clear_hand(0);
        let sp_sha = place_card_suit_rank(&mut ej, 0, Suit::Spade, CardKind::Sha, 1, 13);
        let sp_w = place_card_suit_rank(&mut ej, 0, Suit::Spade, CardKind::Weapon, 1, 13);
        let _ = submit(&mut ej, 0, GameAction::PlayCard { card: sp_w, targets: vec![] });
        drain_pass(&mut ej, 8);
        assert_eq!(ej.state_core().player(0).equips, vec![sp_w], "natural Spade weapon equips cleanly");
        let _ = submit(&mut ej, 0, GameAction::PlayCard { card: sp_sha, targets: vec![] });
        drain_pass(&mut ej, 8);
        assert!(ej.state_core().turn.jiu_active, "Spade via jiuchi activates Jiu");

        // `xiaoxi_hansui`: black card as Sha (`[]`/`[t]`), natural Dismantle `[t]` vs black Wuzhong `[]`!
        let mut ex = start_2p(2026_9408);
        ex.debug_grant(0, "xiaoxi_hansui");
        advance_to_phase(&mut ex, 0, Phase::Play, None);
        ex.debug_clear_hand(0);
        let blk_eq = place_card_suit_rank(&mut ex, 0, Suit::Spade, CardKind::Armor, 1, 13);
        let _ = submit(&mut ex, 0, GameAction::PlayCard { card: blk_eq, targets: vec![1] });
        drain_pass(&mut ex, 16);
        assert_eq!(ex.state_core().player(1).hp, 3, "xiaoxi_hansui black equip as Sha");

        // `zymoucheng`: once per Play, black card as Jiedao `[weapon_holder, victim]` while natural black Sha `[t]` works!
        let mut ez = start_2p(2026_9409);
        ez.debug_grant(0, "zymoucheng");
        advance_to_phase(&mut ez, 0, Phase::Play, None);
        ez.debug_clear_hand(0);
        ez.debug_clear_hand(1);
        let blk_c = place_card_suit_rank(&mut ez, 0, Suit::Club, CardKind::Sha, 1, 13);
        let blk_sha = place_card_suit_rank(&mut ez, 0, Suit::Spade, CardKind::Sha, 1, 13);
        equip_weapon_range(&mut ez, 1, 2);
        let _ = submit(&mut ez, 0, GameAction::PlayCard { card: blk_c, targets: vec![1, 0] });
        assert_eq!(ez.open_choice().expect("jiedao prompt").spec.prompt, "jiedao_sha_or_give");
        let _ = answer(&mut ez, 1, vec![ChoiceOption::Tag(1)]);
        drain_pass(&mut ez, 16);
        assert!(ez.state_core().player(1).equips.is_empty() && !ez.state_core().player(0).equips.is_empty(), "zymoucheng Jiedao stole weapon");
        let hp1_before = ez.state_core().player(1).hp;
        let _ = submit(&mut ez, 0, GameAction::PlayCard { card: blk_sha, targets: vec![1] });
        drain_pass(&mut ez, 16);
        assert_eq!(ez.state_core().player(1).hp, hp1_before - 1, "natural black Sha [t] works alongside zymoucheng");
    }
}

#[test]
fn l11_vm_c_audit36_shared_vm_part2() {
    // Fixed `fuqi`, `lizhan`, `rewansha`, `reyingzi`, `weimu`, `xinfu_songsang`, `xinfu_zhanji`, `xinjuece`, `yjzhenlve`, `zhuiji`.
    {
        // `fuqi`: dist(target, owner) <= 1 makes Sha/Trick unrespondable!
        let mut ef = start_3p(2026_9410);
        ef.debug_grant(0, "fuqi");
        advance_to_phase(&mut ef, 0, Phase::Play, None);
        ef.debug_clear_hand(0);
        ef.debug_clear_hand(1);
        let sha = place_hand_kind_any(&mut ef, 0, CardKind::Sha);
        let _shan = place_hand_kind_any(&mut ef, 1, CardKind::Shan);
        let _ = submit(&mut ef, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(ef.open_choice().is_none(), "fuqi must make Sha to dist<=1 neighbor unrespondable");
        drain_pass(&mut ef, 8);
        assert_eq!(ef.state_core().player(1).hp, 3);

        // `fuqi`: covers ordinary tricks & damage basics, NOT Delayed Tricks (Le/Bing/Lightning) or Tao!
        let le_fq = place_hand_kind_any(&mut ef, 0, CardKind::Le);
        let dis_fq = place_hand_kind_any(&mut ef, 0, CardKind::Dismantle);
        assert!(
            !ef.state_core().is_card_use_unrespondable(
                ef.catalog(),
                0,
                1,
                le_fq,
                CardKind::Le,
                engine_native::event::DamageNature::Normal,
            ),
            "fuqi must only cover ordinary tricks, NOT Delayed Tricks like Le"
        );
        assert!(
            ef.state_core().is_card_use_unrespondable(
                ef.catalog(),
                0,
                1,
                dis_fq,
                CardKind::Dismantle,
                engine_native::event::DamageNature::Normal,
            ),
            "fuqi must make ordinary trick to dist<=1 neighbor unrespondable"
        );
        let _nulfq = place_hand_kind_any(&mut ef, 1, CardKind::Nullify);
        let _ = submit(&mut ef, 0, GameAction::PlayCard { card: dis_fq, targets: vec![1] });
        assert!(
            ef.open_choice().is_none_or(|c| c.spec.prompt != "nullify_window"),
            "fuqi must suppress nullify_window on ordinary trick Dismantle to dist<=1 neighbor"
        );
        drain_pass(&mut ef, 16);

        // `aoe` (`eval_card_unrespondable` uses `effective_kind_in`): taoluan + fuqi virtual SavageAttack makes dist=1 neighbor unable to respond!
        let mut eaoe = start_2p(2026_9416);
        eaoe.debug_grant(0, "taoluan");
        eaoe.debug_grant(0, "fuqi");
        advance_to_phase(&mut eaoe, 0, Phase::Play, None);
        eaoe.debug_clear_hand(0);
        eaoe.debug_clear_hand(1);
        let cost_armor = place_hand_kind_any(&mut eaoe, 0, CardKind::Armor);
        let _s_unused = place_hand_kind_any(&mut eaoe, 1, CardKind::Sha);
        let _ = activate(&mut eaoe, 0, "taoluan");
        let _ = answer(&mut eaoe, 0, vec![ChoiceOption::Card(cost_armor)]);
        let _ = answer(&mut eaoe, 0, vec![ChoiceOption::Tag(7)]); // 7 = SavageAttack in TAOLUAN_KINDS
        if let Some(ch_h) = eaoe.open_choice().cloned() {
            if ch_h.spec.prompt == "taoluan:choose_helper" {
                let _ = answer(&mut eaoe, 0, vec![ChoiceOption::Player(1)]);
            }
        }
        drain_pass(&mut eaoe, 16);
        assert_eq!(eaoe.state_core().player(1).hp, 3, "fuqi must see virtual SavageAttack effective_kind in AOE and prevent Sha response");

        // `lizhan`: End phase draws 1 for selected damaged players.
        let mut el = start_3p(2026_9411);
        el.debug_grant(0, "lizhan");
        el.debug_set_hp(1, 2);
        el.debug_set_hp(2, 3);
        el.debug_clear_hand(1);
        el.debug_clear_hand(2);
        advance_to_phase(&mut el, 0, Phase::End, Some("rvm_choice"));
        let mut el = roundtrip_restore(&el);
        let ch_lz = el.open_choice().cloned().expect("lizhan rvm_choice SelectPlayers");
        assert_eq!(ch_lz.spec.prompt, "rvm_choice");
        let _ = answer(&mut el, 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
        drain_pass(&mut el, 16);
        assert!(!el.state_core().player(1).hand.is_empty() && !el.state_core().player(2).hand.is_empty(), "lizhan drew for damaged players");

        // `rewansha`: during owner turn dying, seat 2 CANNOT save seat 1 with Tao; seat 1 self-saves.
        let mut erw = start_3p(2026_9412);
        erw.debug_grant(0, "rewansha");
        erw.debug_set_hp(1, 1);
        advance_to_phase(&mut erw, 0, Phase::Play, None);
        erw.debug_clear_hand(0);
        erw.debug_clear_hand(1);
        erw.debug_clear_hand(2);
        let _tao2 = place_hand_kind_any(&mut erw, 2, CardKind::Tao);
        let sha = place_hand_kind_any(&mut erw, 0, CardKind::Sha);
        let _ = submit(&mut erw, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        drain_pass(&mut erw, 16);
        assert!(!erw.state_core().player(1).alive, "rewansha blocks bystander seat 2 from saving seat 1");

        // `reyingzi` + `weimu` + `yjzhenlve` + `zhuiji`:
        let mut em = start_3p(2026_9413);
        em.debug_grant(0, "reyingzi");
        em.debug_grant(0, "zhuiji");
        em.debug_grant(1, "weimu");
        em.debug_grant(2, "yjzhenlve");
        em.debug_set_max_hp(0, 4);
        em.debug_set_hp(0, 2);
        assert_eq!(em.state_core().hand_limit(em.catalog(), 0), 4, "reyingzi fixes base hand limit = max_hp (4)");
        em.debug_clear_hand(0);
        em.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut em, 0, GameAction::NextPhase);
        drain_pass(&mut em, 16);
        assert_eq!(em.state_core().player(0).hand.len(), 3, "reyingzi draws +1 at DrawPhase");
        advance_to_phase(&mut em, 0, Phase::Play, None);
        em.debug_set_hp(0, 4);
        em.debug_set_hp(2, 2);
        equip_kind_suit(&mut em, 2, CardKind::HorsePlus, None);
        assert_eq!(em.state_core().computed_distance(em.catalog(), 0, 2), 1, "zhuiji sets dist=1 when owner.hp >= target.hp");
        em.debug_clear_hand(1);
        em.debug_clear_hand(2);
        let blk_dis = place_card_suit_rank(&mut em, 0, Suit::Spade, CardKind::Dismantle, 1, 13);
        let _p1 = place_hand_kind_any(&mut em, 1, CardKind::Sha);
        let _ = submit(&mut em, 0, GameAction::PlayCard { card: blk_dis, targets: vec![1] });
        drain_pass(&mut em, 16);
        assert_eq!(em.state_core().player(1).hand.len(), 1, "weimu blocks black trick on seat 1");
        let le = place_hand_kind_any(&mut em, 0, CardKind::Le);
        let _ = submit(&mut em, 0, GameAction::PlayCard { card: le, targets: vec![2] });
        drain_pass(&mut em, 8);
        assert!(em.state_core().player(2).judge.is_empty(), "yjzhenlve immune to Le");

        // `xinfu_songsang` + `xinfu_zhanji` + `xinjuece`:
        let mut exs = start_3p(2026_9414);
        exs.assign_general_named(0, "吕蒙").unwrap();
        exs.assign_general_named(1, "吕蒙").unwrap();
        exs.assign_general_named(2, "甘宁").unwrap();
        exs.debug_grant(0, "xinfu_songsang");
        exs.debug_grant(0, "xinjuece");
        exs.debug_set_hp(0, 2);
        exs.debug_set_hp(2, 1);
        advance_to_phase(&mut exs, 0, Phase::Play, None);
        exs.debug_clear_hand(0);
        exs.debug_clear_hand(1);
        exs.debug_clear_hand(2);
        let sha = place_hand_kind_any(&mut exs, 0, CardKind::Sha);
        let _ = submit(&mut exs, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
        drain_with_ai(&mut exs, 16);
        assert!(!exs.state_core().player(2).alive);
        assert_eq!(exs.state_core().player(0).hp, 3, "xinfu_songsang recovered 1 HP on death");
        assert!(exs.catalog().skill_by_name("xinfu_zhanji").is_some_and(|d| exs.state_core().has_skill(0, d)));
        // `xinfu_zhanji`: external Play-phase draw triggers +1 bonus draw (`XinfuZhanjiPlayDraw`, no self recursion)!
        let wz = place_hand_card_named(&mut exs, 0, "wuzhong");
        let _ = submit(&mut exs, 0, GameAction::PlayCard { card: wz, targets: vec![] });
        drain_with_ai(&mut exs, 16);
        assert_eq!(exs.state_core().player(0).hand.len(), 3, "xinfu_zhanji draws +1 (2 + 1 = 3) on external Play draw");
        // `xinjuece`: End phase when seat 1 has 0 hand cards deals 1 damage!
        exs.debug_clear_hand(1);
        let hp1 = exs.state_core().player(1).hp;
        advance_to_phase(&mut exs, 0, Phase::End, Some("trigger_confirm"));
        if exs.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
            let _ = answer(&mut exs, 0, vec![ChoiceOption::Confirm]);
        }
        if exs.open_choice().is_some_and(|c| c.spec.prompt == "rvm_choice") {
            let _ = answer(&mut exs, 0, vec![ChoiceOption::Player(1)]);
        }
        drain_with_ai(&mut exs, 16);
        assert_eq!(exs.state_core().player(1).hp, hp1 - 1, "xinjuece End damaged empty-hand player");
    }
}
