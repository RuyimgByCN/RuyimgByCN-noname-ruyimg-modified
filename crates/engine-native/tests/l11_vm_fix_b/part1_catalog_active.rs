use super::common::*;
use content_native_loader::taixu_vm_fix_b::{
    L11_VM_FIX_B_PRIMARY_SKILLS, L11_VM_FIX_B_SUB_SKILLS,
};
use content_native_loader::L11_TAIXU_CARDS;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::content::host::HostBehavior;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_vm_fix_b_catalog_bindings_rebiyue_vm_and_tiaojiyanmei_off_deck() {
    let _verified = taixu_verified();
    assert_eq!(L11_TAIXU_CARDS.len(), 32);
    assert_eq!(L11_VM_FIX_B_PRIMARY_SKILLS.len(), 34);
    assert_eq!(L11_VM_FIX_B_SUB_SKILLS.len(), 13);

    let e = start_4p_taixu(2026_8001);
    let cat = e.catalog();
    assert_eq!(engine_native::content::standard::STANDARD_RULESET_VERSION, 333);
    assert_eq!(engine_native::archive::ARCHIVE_FORMAT_VERSION, 107);
    assert_ne!(e.state_core().ruleset_version, 0);

    for &name in L11_VM_FIX_B_PRIMARY_SKILLS
        .iter()
        .chain(L11_VM_FIX_B_SUB_SKILLS.iter())
    {
        let sid = cat
            .skill_by_name(name)
            .unwrap_or_else(|| panic!("L11VmFixB skill {name} missing"));
        assert_eq!(
            cat.skills[sid as usize].behavior.as_host_behavior(),
            Some(HostBehavior::L11VmFixB),
            "skill {name} must bind to HostBehavior::L11VmFixB"
        );
    }

    let rebiyue_id = cat.skill_by_name("rebiyue").expect("rebiyue exists");
    assert!(
        cat.skills[rebiyue_id as usize]
            .behavior
            .as_host_behavior()
            .is_none(),
        "rebiyue must remain on taixu-upstream-liuli VM"
    );

    for dead_stub in ["quhu_win", "quhu_lose"] {
        assert!(
            cat.skill_by_name(dead_stub).is_none(),
            "stale VM stub {dead_stub} must stay stripped"
        );
    }

    for cname in [
        "tiaojiyanmei",
        "tiaojiyanmei_heart",
        "tiaojiyanmei_club",
        "tiaojiyanmei_diamond",
    ] {
        let cid = cat
            .card_by_name(cname)
            .unwrap_or_else(|| panic!("off-deck card {cname} missing"));
        assert!(cat.cards[cid as usize].off_deck);
        assert!(!e.state_core().deck.contains(&cid));
        assert!(e.state_core().detached_cards.iter().any(|&(id, _)| id == cid));
    }
}

#[test]
fn l11_vm_fix_b_rebiyue_vm_phase_end_empty_and_nonempty_plus_public_ai() {
    let mut e = start_4p_taixu(2026_8002);
    e.debug_grant(0, "rebiyue");

    // Empty hand at Phase::End -> triggers VM optional trigger_confirm -> draw 2
    e.debug_clear_hand(0);
    let out = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    if e.state_core().turn.phase == Phase::Discard {
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    }
    let _ = expect_prompt(&e, "trigger_confirm");
    // Public-info AI should confirm rebiyue!
    drain_open_with_ai(&mut e, 16);
    assert_eq!(e.debug_hand(0).len(), 2, "rebiyue on empty hand draws 2");

    // Next turn with non-empty hand -> confirm manually -> draws +1
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let keep = e.debug_place(0, "sha");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    if e.state_core().turn.phase == Phase::Discard {
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    let _ = expect_prompt(&e, "trigger_confirm");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hand(0).len(), 2, "rebiyue with 1 card draws +1 => 2");
    assert!(e.debug_hand(0).contains(&keep));
}

#[test]
fn l11_vm_fix_b_quhu_win_lose_and_hp_gate() {
    let mut e = start_4p_taixu(2026_8003);
    e.debug_grant(0, "quhu");
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 3);
    e.debug_set_hp(2, 2);
    let c0 = place_suit_rank(&mut e, 0, Suit::Spade, 10);
    let _c1 = place_suit_rank(&mut e, 1, Suit::Heart, 5);
    let _c2 = place_suit_rank(&mut e, 2, Suit::Club, 4);

    // Negative gate: nobody has hp > owner.hp (3)
    assert!(matches!(
        activate_skill(&mut e, 0, "quhu"),
        CommandOutcome::Rejected(_)
    ));

    // Seat 1 gains hp=4 (> owner 3); win Pindian -> seat 1 deals 1 dmg to seat 2 (range 1)
    e.debug_set_hp(1, 4);
    let hp2_before = e.debug_hp(2);
    let out = activate_skill(&mut e, 0, "quhu");
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let _ = expect_prompt(&e, "l11_vm_b:quhu_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = expect_prompt(&e, "l11_vm_b:pindian_card");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c0)]);
    let _ = expect_prompt(&e, "l11_vm_b:quhu_damage_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    assert_eq!(e.debug_hp(2), hp2_before - 1, "quhu win: target damages victim");

    // Next turn: tie/lose Pindian -> target deals 1 damage to owner!
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 4);
    let low0 = place_suit_rank(&mut e, 0, Suit::Spade, 3);
    let _high1 = place_suit_rank(&mut e, 1, Suit::Heart, 9);
    let _ = activate_skill(&mut e, 0, "quhu");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(low0)]);
    assert_eq!(e.debug_hp(0), 2, "quhu lose: target deals 1 dmg to owner");
}

#[test]
fn l11_vm_fix_b_rexianzhen_and_tianyi_subskills_armor_discard_cleanup() {
    let mut e = start_4p_taixu(2026_8004);
    e.debug_grant(0, "rexianzhen");
    e.debug_set_hp(0, 2);
    // Give seat 2 Renwang armor via real PlayCard
    advance_to_play(&mut e, 2);
    let rw = e.debug_place(2, "renwang");
    let _ = play_card(&mut e, 2, rw, vec![]);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    // Win Pindian with a Sha (rank >= 9) -> grants rexianzhen2 + rexianzhen4
    let sha_hi = place_matching(&mut e, 0, |d| d.kind == CardKind::Sha && d.rank >= 9);
    let _low2 = place_suit_rank(&mut e, 2, Suit::Diamond, 2); // distance 2 from seat 0

    // Clear seat 0 extra except sha_hi, sha_extra_1 (black), sha_extra_2 (black)
    let _ = activate_skill(&mut e, 0, "rexianzhen");
    let _ = expect_prompt(&e, "l11_vm_b:rexianzhen_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    let _ = expect_prompt(&e, "l11_vm_b:pindian_card");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(sha_hi)]);
    assert!(e.debug_has_skill(0, "rexianzhen2"));
    assert!(e.debug_has_skill(0, "rexianzhen4"));

    // Unlimited distance + unlimited count + ignore Renwang armor vs bound target 2!
    let black_sha1 = place_matching(&mut e, 0, |d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let black_sha2 = place_matching(&mut e, 0, |d| {
        d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
    });
    let hp2_before = e.debug_hp(2);
    assert!(!matches!(
        play_card(&mut e, 0, black_sha1, vec![2]),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e, 16);
    assert!(!matches!(
        play_card(&mut e, 0, black_sha2, vec![2]),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e, 16);
    assert_eq!(e.debug_hp(2), hp2_before - 2);

    // rexianzhen4: holding extra Shas (> hp=2), Phase::Discard must NOT count/discard Shas!
    let s_keep1 = e.debug_place(0, "sha");
    let s_keep2 = e.debug_place(0, "sha");
    let s_keep3 = e.debug_place(0, "sha");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    // With hp=2 and 3 Shas ignored by rexianzhen4, effective hand count = 0 <= 2 -> Discard asks nothing!
    assert_eq!(e.state_core().turn.phase, Phase::Discard);
    assert!(e.open_choice().is_none(), "rexianzhen4 ignores Shas in Discard");
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut e, 0, GameAction::NextPhase); // End -> TurnPhaseAfter
    assert!(e.debug_hand(0).contains(&s_keep1));
    assert!(e.debug_hand(0).contains(&s_keep2));
    assert!(e.debug_hand(0).contains(&s_keep3));
    assert!(!e.debug_has_skill(0, "rexianzhen2"));
    assert!(!e.debug_has_skill(0, "rexianzhen4"));

    // Lose Pindian with rexianzhen -> rexianzhen3 blocks Sha!
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let low = place_suit_rank(&mut e, 0, Suit::Heart, 2);
    let extra_sha = e.debug_place(0, "sha");
    let _hi = place_suit_rank(&mut e, 1, Suit::Spade, 13);
    let _ = activate_skill(&mut e, 0, "rexianzhen");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(low)]);
    assert!(e.debug_has_skill(0, "rexianzhen3"));
    assert!(matches!(
        play_card(&mut e, 0, extra_sha, vec![1]),
        CommandOutcome::Rejected(_)
    ));

    // Verify tianyi win -> extra target (2 targets with Sha) & lose -> tianyi3 blocks Sha
    advance_to_play(&mut e, 0);
    e.debug_remove_skill(0, "rexianzhen");
    e.debug_grant(0, "tianyi");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let t_hi = place_suit_rank(&mut e, 0, Suit::Heart, 13);
    let sha_a = e.debug_place(0, "sha");
    let sha_b = e.debug_place(0, "sha");
    let _t_lo = place_suit_rank(&mut e, 1, Suit::Club, 2);
    let _ = activate_skill(&mut e, 0, "tianyi");
    let _ = expect_prompt(&e, "l11_vm_b:tianyi_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(t_hi)]);
    assert!(e.debug_has_skill(0, "tianyi2"));
    assert!(!matches!(
        play_card(&mut e, 0, sha_a, vec![1, 2]),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e, 16);
    assert!(!matches!(
        play_card(&mut e, 0, sha_b, vec![3]),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e, 16);
}

#[test]
fn l11_vm_fix_b_refanjian_reluanwu_sbyangwei_shanxie_shenfen_tiaoxin() {
    // 1. refanjian: suit discard vs loseHp + empty hand direct loseHp
    let mut e = start_4p_taixu(2026_8005);
    e.debug_grant(0, "refanjian");
    let give_h = place_suit_rank(&mut e, 0, Suit::Heart, 6);
    let t1_h = place_matching(&mut e, 1, |d| d.suit == Suit::Heart && d.kind == CardKind::Shan);
    let t1_s = place_matching(&mut e, 1, |d| !d.off_deck && d.suit == Suit::Spade);
    let _ = activate_skill(&mut e, 0, "refanjian");
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_card");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(give_h)]);
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_mode");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Tag(0)]); // show hand & discard all Hearts!
    assert!(!e.debug_hand(1).contains(&give_h));
    assert!(!e.debug_hand(1).contains(&t1_h));
    assert!(e.debug_hand(1).contains(&t1_s));

    // Next turn: choose loseHp (Tag(1))
    advance_to_play(&mut e, 0);
    let give2 = place_suit_rank(&mut e, 0, Suit::Diamond, 4);
    let hp2_before = e.debug_hp(2);
    let _ = activate_skill(&mut e, 0, "refanjian");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(give2)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_mode");
    let _ = answer(&mut e, 2, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e.debug_hp(2), hp2_before - 1);

    // 2. sbyangwei: 2-turn cooldown + ignore armor + cleanup
    let mut e2 = start_4p_taixu(2026_8006);
    e2.debug_grant(0, "sbyangwei");
    assert!(!matches!(
        activate_skill(&mut e2, 0, "sbyangwei"),
        CommandOutcome::Rejected(_)
    ));
    assert_eq!(e2.debug_hand(0).len(), 2);
    assert!(e2.debug_has_skill(0, "sbyangwei_effect"));
    assert!(e2.debug_has_skill(0, "sbyangwei_counter"));
    // Advance to seat 0's Turn 2 Play -> effect gone, counter STILL present -> blocked!
    advance_to_play(&mut e2, 0);
    assert!(!e2.debug_has_skill(0, "sbyangwei_effect"));
    assert!(e2.debug_has_skill(0, "sbyangwei_counter"));
    assert!(matches!(
        activate_skill(&mut e2, 0, "sbyangwei"),
        CommandOutcome::Rejected(_)
    ));
    // Advance to seat 0's Turn 3 Play -> counter removed on 2nd Phase::End -> usable again!
    advance_to_play(&mut e2, 0);
    assert!(!e2.debug_has_skill(0, "sbyangwei_counter"));
    assert!(!matches!(
        activate_skill(&mut e2, 0, "sbyangwei"),
        CommandOutcome::Rejected(_)
    ));

    // 3. shenfen + reluanwu + shanxie + tiaoxin
    let mut e3 = start_4p_taixu(2026_8007);
    e3.debug_grant(0, "shenfen");
    e3.debug_grant(0, "shanxie");
    e3.debug_grant(0, "tiaoxin");
    // Rejected below 6 baonu marks
    e3.state_core_mut().vm_aux.marks.insert((0, 1_252_226_867), 5);
    assert!(matches!(
        activate_skill(&mut e3, 0, "shenfen"),
        CommandOutcome::Rejected(_)
    ));
    // At 6 marks: deals 1 damage to seats 1,2,3 and discards 4 from each!
    e3.state_core_mut().vm_aux.marks.insert((0, 1_252_226_867), 6);
    for s in 1..=3 {
        for _ in 0..5 {
            e3.debug_place(s, "sha");
        }
    }
    assert!(!matches!(
        activate_skill(&mut e3, 0, "shenfen"),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e3, 32);
    assert_eq!(e3.state_core().vm_aux.marks.get(&(0, 1_252_226_867)).copied().unwrap_or(0), 0);
    for s in 1..=3 {
        assert_eq!(e3.debug_hp(s), 3);
        assert_eq!(e3.debug_hand(s).len(), 1, "seat {s} kept 1 of 5 after 4 discarded");
    }
    // Once per game blocked on next turn
    advance_to_play(&mut e3, 0);
    e3.state_core_mut().vm_aux.marks.insert((0, 1_252_226_867), 6);
    assert!(matches!(
        activate_skill(&mut e3, 0, "shenfen"),
        CommandOutcome::Rejected(_)
    ));

    // shanxie gain weapon + shanxie_exclude voids low-rank Shan while high-rank Shan succeeds
    e3.debug_clear_hand(0);
    assert!(!matches!(
        activate_skill(&mut e3, 0, "shanxie"),
        CommandOutcome::Rejected(_)
    ));
    assert!(
        e3.debug_hand(0)
            .iter()
            .any(|&c| e3.catalog().card_slot_of(e3.state_core(), c) == Some(EquipSlot::Weapon)),
        "shanxie must gain a weapon from deck"
    );
    // Equip a weapon range 2 (qinggang), attack_range=2 -> threshold 2*2=4 -> rank 2 Shan voided!
    let _ = e3.debug_clear_hand(1);
    let shan_low = place_matching(&mut e3, 1, |d| d.kind == CardKind::Shan && d.rank <= 4);
    let qg = e3.debug_place(0, "qinggang");
    let _ = play_card(&mut e3, 0, qg, vec![]);
    let sha_atk = e3.debug_place(0, "sha");
    let hp1_b = e3.debug_hp(1);
    let _ = play_card(&mut e3, 0, sha_atk, vec![1]);
    drain_open_with_ai(&mut e3, 12);
    assert_eq!(e3.debug_hp(1), hp1_b - 1, "shanxie_exclude must void low-rank Shan");
    let _ = shan_low;

    // tiaoxin: target with range can use Sha OR owner discards 1 HiddenCard
    advance_to_play(&mut e3, 0);
    e3.debug_clear_hand(1);
    e3.debug_place(1, "shan");
    assert!(!matches!(
        activate_skill(&mut e3, 0, "tiaoxin"),
        CommandOutcome::Rejected(_)
    ));
    let _ = expect_prompt(&e3, "l11_vm_b:tiaoxin_target");
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
    let _ = expect_prompt(&e3, "l11_vm_b:tiaoxin_discard");
    let _ = answer(&mut e3, 0, vec![ChoiceOption::HiddenCard(0)]);
    assert!(e3.debug_hand(1).is_empty());

    // reluanwu: limited, nearest Sha or loseHp(1) + owner free Sha
    e3.debug_grant(0, "reluanwu");
    e3.debug_clear_hand(1);
    e3.debug_clear_hand(2);
    e3.debug_clear_hand(3);
    let hp2_before_lw = e3.debug_hp(2);
    assert!(!matches!(
        activate_skill(&mut e3, 0, "reluanwu"),
        CommandOutcome::Rejected(_)
    ));
    drain_open_with_ai(&mut e3, 24);
    assert_eq!(e3.debug_hp(2), hp2_before_lw - 1, "no Sha loses 1 HP in reluanwu");
    assert!(matches!(
        activate_skill(&mut e3, 0, "reluanwu"),
        CommandOutcome::Rejected(_)
    ), "reluanwu must stay once_per_game");
}

#[test]
fn l11_vm_fix_b_r2_refanjian_effective_suit_with_hongyan() {
    let mut e = start_4p_taixu(2026_8211);
    e.debug_grant(0, "refanjian");
    e.debug_grant(1, "hongyan");
    let give_spade = place_suit_rank(&mut e, 0, Suit::Spade, 7);
    let t1_heart = place_matching(&mut e, 1, |d| !d.off_deck && d.suit == Suit::Heart);
    let t1_club = place_matching(&mut e, 1, |d| !d.off_deck && d.suit == Suit::Club);

    assert!(!matches!(
        activate_skill(&mut e, 0, "refanjian"),
        CommandOutcome::Rejected(_)
    ));
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_card");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(give_spade)]);
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_target");
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = expect_prompt(&e, "l11_vm_b:refanjian_mode");
    let _ = answer(&mut e, 1, vec![ChoiceOption::Tag(0)]); // show & discard effective suit (Heart due to hongyan!)

    assert!(
        !e.debug_hand(1).contains(&give_spade),
        "hongyan turns given Spade into Heart -> must be discarded"
    );
    assert!(
        !e.debug_hand(1).contains(&t1_heart),
        "Heart in target hand matches hongyan-converted Heart -> must be discarded"
    );
    assert!(
        e.debug_hand(1).contains(&t1_club),
        "Club in target hand must stay"
    );
}
