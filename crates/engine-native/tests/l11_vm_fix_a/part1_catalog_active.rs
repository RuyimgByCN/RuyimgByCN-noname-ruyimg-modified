use super::common::*;
use content_native_loader::taixu_vm_fix_a::L11_TAIXU_VM_FIX_A_SKILLS;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::content::host::HostBehavior;
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, Phase};

fn start_clean_4p_sans_general(seed: u64) -> Engine {
    let verified = taixu_verified();
    let mut e = content_native_loader::start_locked_engine(
        seed,
        4,
        content_native_loader::LockedStartMode::Identity,
        verified,
    )
    .expect("start 4p");
    e.debug_set_identity_roles(vec![
        engine_native::state::Role::Lord,
        engine_native::state::Role::Rebel,
        engine_native::state::Role::Traitor,
        engine_native::state::Role::Loyalist,
    ]);
    for s in 0..4 {
        e.debug_remove_general(s);
        e.debug_set_max_hp(s, 5);
        e.debug_set_hp(s, 5);
        e.debug_clear_hand(s);
    }
    e.debug_sweep_judge_zones();
    advance_to_phase(&mut e, 0, Phase::Play);
    for s in 0..4 {
        e.debug_clear_hand(s);
    }
    e
}

#[test]
fn l11_vm_fix_a_catalog_routing_and_locked_flags() {
    let e = start_clean_4p_sans_general(910_001);
    let cat = e.catalog();
    let xiangle_id = cat.skill_by_name("xiangle").expect("xiangle registered");
    assert_eq!(cat.skills[xiangle_id as usize].behavior.as_host_behavior(), Some(HostBehavior::U01DddBatchD));

    let jsrgbahu_id = cat.skill_by_name("jsrgbahu").expect("jsrgbahu stays in catalog");
    assert_eq!(cat.skills[jsrgbahu_id as usize].behavior.as_host_behavior(), None);
    assert!(engine_native::catalog::Catalog::is_locked_skill_name("jsrgbahu", None));

    assert!(cat.skill_by_name("nzry_longnu_1").is_none());
    assert!(cat.skill_by_name("nzry_longnu_2").is_none());

    for sk in L11_TAIXU_VM_FIX_A_SKILLS {
        let sid = cat.skill_by_name(sk).unwrap_or_else(|| panic!("{sk} missing"));
        assert_eq!(
            cat.skills[sid as usize].behavior.as_host_behavior(),
            Some(HostBehavior::L11VmFixA),
            "{sk} must bind HostBehavior::L11VmFixA"
        );
    }
    for locked in ["fulin", "nzry_longnu", "jiezi", "kunfen", "manji", "jsrgbahu", "wushen"] {
        assert!(
            engine_native::catalog::Catalog::is_locked_skill_name(locked, None),
            "{locked} must be recognized as locked"
        );
    }
}

#[test]
fn l11_vm_fix_a_xiangle_and_jsrgbahu_vm_and_seals() {
    // 1. xiangle: opponent Sha must discard 1 Basic card or Sha is cancelled
    let mut e = start_clean_4p_sans_general(910_002);
    e.debug_grant(1, "xiangle");
    let sha1 = e.debug_place(0, "sha");
    let peach0 = e.debug_place(0, "tao");
    let trick0 = e.debug_place(0, "wuzhong");
    let shan1 = e.debug_place(1, "shan");
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert_eq!(e.open_choice().map(|c| c.observer), Some(0));
    let opts = e.open_choice().unwrap().spec.options.clone();
    assert!(opts.contains(&ChoiceOption::Card(peach0)));
    assert!(!opts.contains(&ChoiceOption::Card(trick0)), "xiangle rejects non-basic");
    let _ = answer(&mut e, 0, Vec::new()); // Decline -> Sha cancelled
    drain_ai(&mut e, 16);
    assert_eq!(e.debug_hp(1), 5);
    assert!(e.debug_hand(1).contains(&shan1));

    // Confirm discarding basic -> Sha resolves (equip Zhuge to allow 2nd Sha in same Play phase)
    let wq = place_kind(&mut e, 0, CardKind::Zhuge);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wq, targets: vec![] });
    let sha2 = e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(peach0)]);
    drain_ai(&mut e, 16);
    assert_eq!(e.debug_hp(1), 4);

    // 2. jsrgbahu VM: phase_prepare draw 1, play limit +1, bypasses tieji seal, blocked by chanyuan
    let mut e2 = start_clean_4p_sans_general(910_003);
    e2.debug_grant(1, "jsrgbahu");
    advance_to_phase(&mut e2, 0, Phase::End);
    e2.state_core_mut().vm_aux.flags.insert((1, "hs_fengyin_xuetu".to_string()), 1);
    assert!(e2.catalog().is_tieji_sealed(e2.state_core(), 1));
    let h_before = e2.debug_hand(1).len();
    advance_to_phase(&mut e2, 1, Phase::Play);
    assert!(e2.catalog().is_tieji_sealed(e2.state_core(), 1));
    assert!(e2.debug_hand(1).len() >= h_before + 3, "locked jsrgbahu draw 1 at Prepare + 2 at Draw even under tieji");

    let s1 = e2.debug_place(1, "sha");
    let s2 = e2.debug_place(1, "sha");
    e2.debug_clear_hand(2);
    let _ = submit(&mut e2, 1, GameAction::PlayCard { card: s1, targets: vec![2] });
    drain_ai(&mut e2, 16);
    let out2 = submit(&mut e2, 1, GameAction::PlayCard { card: s2, targets: vec![2] });
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "jsrgbahu +1 Sha limit must allow 2nd Sha");
    drain_ai(&mut e2, 16);
    assert_eq!(e2.debug_hp(2), 3);

    // chanyuan seal (hp == 1) blocks jsrgbahu even though tieji does not
    e2.debug_grant(1, "chanyuan");
    e2.debug_set_hp(1, 1);
    advance_to_phase(&mut e2, 0, Phase::End);
    let h_pre = e2.debug_hand(1).len();
    advance_to_phase(&mut e2, 1, Phase::Play);
    assert_eq!(e2.debug_hand(1).len(), h_pre + 2, "chanyuan hp==1 blocks jsrgbahu prepare draw (only +2 DrawPhase)");
    let s3 = e2.debug_place(1, "sha");
    let s4 = e2.debug_place(1, "sha");
    let _ = submit(&mut e2, 1, GameAction::PlayCard { card: s3, targets: vec![2] });
    drain_ai(&mut e2, 16);
    assert!(
        matches!(
            submit(&mut e2, 1, GameAction::PlayCard { card: s4, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ),
        "chanyuan hp==1 must block jsrgbahu +1 Sha limit"
    );
}

#[test]
fn l11_vm_fix_a_wurong_all_branches_and_archive() {
    // Branch 1: Sha vs !Shan -> discard c1 + 1 damage
    let mut e = start_clean_4p_sans_general(910_004);
    e.debug_grant(0, "wurong");
    let wr = e.catalog().skill_by_name("wurong").unwrap();
    let c_sha = e.debug_place(0, "sha");
    let c_tao1 = e.debug_place(1, "tao");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: wr });
    assert!(prompt_is(&e, "l11_vm_a_wurong_target"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    assert!(prompt_is(&e, "l11_vm_a_wurong_card"));
    // NGSA roundtrip mid-choice
    let mut e = roundtrip_ngsa(&e, taixu_verified());
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_sha)]);
    assert!(prompt_is(&e, "l11_vm_a_wurong_card"));
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(c_tao1)]);
    assert_eq!(e.debug_hp(1), 4);
    assert!(!e.debug_hand(0).contains(&c_sha));
    assert!(e.debug_hand(1).contains(&c_tao1));

    // Branch 2: !Sha vs Shan -> discard c1 + steal he
    advance_to_phase(&mut e, 0, Phase::Discard);
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let c_peach = e.debug_place(0, "tao");
    let c_shan = e.debug_place(1, "shan");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: wr });
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_peach)]);
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(c_shan)]);
    assert!(prompt_is(&e, "l11_vm_a_wurong_gain"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_shan)]);
    assert_eq!(e.debug_hp(1), 4);
    assert!(!e.debug_hand(0).contains(&c_peach));
    assert!(e.debug_hand(0).contains(&c_shan));

    // Branch 3: Sha vs Shan -> neither discarded, no damage
    advance_to_phase(&mut e, 0, Phase::Discard);
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let c_sha2 = e.debug_place(0, "sha");
    let c_shan2 = e.debug_place(1, "shan");
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: wr });
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(c_sha2)]);
    let _ = answer(&mut e, 1, vec![ChoiceOption::Card(c_shan2)]);
    assert_eq!(e.debug_hp(1), 4);
    assert!(e.debug_hand(0).contains(&c_sha2));
    assert!(e.debug_hand(1).contains(&c_shan2));
}

#[test]
fn l11_vm_fix_a_yeyan_small_and_great_with_forcedie() {
    // Small Yeyan: up to 3 targets, 1 fire damage each, once per game
    let mut e = start_clean_4p_sans_general(910_005);
    e.debug_grant(0, "yeyan");
    let yy = e.catalog().skill_by_name("yeyan").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: yy });
    assert!(prompt_is(&e, "l11_vm_a_yeyan_mode"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Tag(0)]);
    assert!(prompt_is(&e, "l11_vm_a_yeyan_small_targets"));
    let _ = answer(
        &mut e,
        0,
        vec![ChoiceOption::Player(3), ChoiceOption::Player(1), ChoiceOption::Player(2)],
    );
    drain_ai(&mut e, 16);
    assert_eq!((e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)), (4, 4, 4));
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: yy }),
        CommandOutcome::Rejected(_)
    ));

    // Great Yeyan: 4 distinct suits, owner HP = 2 -> dies from LoseHp(3), fire damage STILL lands on both targets (2 + 1)
    let mut e2 = start_clean_4p_sans_general(910_006);
    e2.debug_grant(0, "yeyan");
    e2.debug_set_hp(0, 2);
    let yy2 = e2.catalog().skill_by_name("yeyan").unwrap();
    let c_s = e2.debug_place_suit(0, Suit::Spade);
    let c_h = e2.debug_place_suit(0, Suit::Heart);
    let c_c = e2.debug_place_suit(0, Suit::Club);
    let c_d = e2.debug_place_suit(0, Suit::Diamond);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: yy2 });
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Tag(1)]);
    for c in [c_s, c_h, c_c, c_d] {
        assert!(prompt_is(&e2, "l11_vm_a_yeyan_card"));
        let _ = answer(&mut e2, 0, vec![ChoiceOption::Card(c)]);
    }
    assert!(prompt_is(&e2, "l11_vm_a_yeyan_great_targets"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    assert!(prompt_is(&e2, "l11_vm_a_yeyan_2dmg_target"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(2)]);
    drain_ai(&mut e2, 32);
    assert_eq!(e2.debug_hp(1), 4, "secondary great yeyan target takes 1 fire damage");
    assert_eq!(e2.debug_hp(2), 3, "primary great yeyan target takes 2 fire damage even after owner dies");

    // Great Yeyan 1-target mode: choose 3 damage on single target
    let mut e3 = start_clean_4p_sans_general(910_010);
    e3.debug_grant(0, "yeyan");
    let yy3 = e3.catalog().skill_by_name("yeyan").unwrap();
    let s1 = e3.debug_place_suit(0, Suit::Spade);
    let h1 = e3.debug_place_suit(0, Suit::Heart);
    let c1 = e3.debug_place_suit(0, Suit::Club);
    let d1 = e3.debug_place_suit(0, Suit::Diamond);
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: yy3 });
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Tag(1)]);
    for c in [s1, h1, c1, d1] {
        let _ = answer(&mut e3, 0, vec![ChoiceOption::Card(c)]);
    }
    assert!(prompt_is(&e3, "l11_vm_a_yeyan_great_targets"));
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
    assert!(prompt_is(&e3, "l11_vm_a_yeyan_dmg_amount"));
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Tag(3)]);
    drain_ai(&mut e3, 32);
    assert_eq!(e3.debug_hp(0), 2);
    assert_eq!(e3.debug_hp(1), 2);

    // Great Yeyan hand-only: 4th suit only in equip zone must NOT satisfy Great Yeyan or consume limit
    let mut e4 = start_clean_4p_sans_general(910_011);
    e4.debug_grant(0, "yeyan");
    let yy4 = e4.catalog().skill_by_name("yeyan").unwrap();
    let _hs = e4.debug_place_suit(0, Suit::Spade);
    let _hh = e4.debug_place_suit(0, Suit::Heart);
    let _hc = e4.debug_place_suit(0, Suit::Club);
    let eq_diamond = e4
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(i, cd)| {
            let id = i as u32;
            let free = !e4
                .state_core()
                .players
                .iter()
                .any(|p| p.hand.contains(&id) || p.equips.contains(&id) || p.judge.contains(&id))
                && !e4.state_core().table.contains(&id);
            (cd.suit == Suit::Diamond && cd.kind.equip_slot().is_some() && free).then_some(id)
        })
        .expect("free diamond equip");
    e4.state_core_mut().deck.retain(|&c| c != eq_diamond);
    e4.state_core_mut().player_mut(0).hand.push(eq_diamond);
    let _ = submit(&mut e4, 0, GameAction::PlayCard { card: eq_diamond, targets: vec![] });
    assert!(e4.debug_equips(0).contains(&eq_diamond));
    let _ = submit(&mut e4, 0, GameAction::ActivateSkill { skill: yy4 });
    assert!(prompt_is(&e4, "l11_vm_a_yeyan_mode"));
    let _ = answer(&mut e4, 0, vec![ChoiceOption::Tag(1)]);
    assert!(e4.open_choice().is_none(), "equip suit must not unlock Great Yeyan");
    assert_eq!(e4.debug_hp(0), 5);
    assert!(!matches!(
        submit(&mut e4, 0, GameAction::ActivateSkill { skill: yy4 }),
        CommandOutcome::Rejected(_)
    ), "failed Great Yeyan must not consume once_used");
    let _ = answer(&mut e4, 0, vec![ChoiceOption::Tag(0)]);
    let _ = answer(&mut e4, 0, vec![ChoiceOption::Player(1)]);
    drain_ai(&mut e4, 16);
    assert_eq!(e4.debug_hp(1), 4);
}

#[test]
fn l11_vm_fix_a_qiangwu_xiongluan_mingce_nsyangwu() {
    // 1. qiangwu: Judge rank 7 -> rank < 7 unlimited distance, rank > 7 unlimited count, expires phaseUseEnd
    let mut e = start_clean_4p_sans_general(910_007);
    e.debug_grant(0, "qiangwu");
    let qw = e.catalog().skill_by_name("qiangwu").unwrap();
    let _j7 = e.debug_set_deck_top_by(|cd| cd.rank == 7);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: qw });
    drain_ai(&mut e, 8);
    let sha_far = place_by_rank(&mut e, 0, "sha", 1, 6);
    let sha_hi1 = place_by_rank(&mut e, 0, "sha", 8, 13);
    let sha_hi2 = place_by_rank(&mut e, 0, "sha", 8, 13);
    e.debug_clear_hand(2);
    e.debug_clear_hand(1);
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_far, targets: vec![2] }),
        CommandOutcome::Rejected(_)
    ), "< judge rank Sha ignores distance (seat 2 at dist 2)");
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_hi1, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ));
    assert!(!matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha_hi2, targets: vec![1] }),
        CommandOutcome::Rejected(_)
    ), "> judge rank Sha ignores Sha count limit");
    // Advance out of Play phase -> flags expire
    advance_to_phase(&mut e, 0, Phase::Discard);
    assert_eq!(e.state_core().vm_aux.flags.get(&(0, "l11_qiangwu_active".to_string())).copied().unwrap_or(0), 0);

    // 2. drlt_xiongluan: disables equip & judge slots, blocks target hand cards, unlimited targeting, expires phaseUseEnd
    let mut e2 = start_clean_4p_sans_general(910_008);
    e2.debug_grant(0, "drlt_xiongluan");
    let xl = e2.catalog().skill_by_name("drlt_xiongluan").unwrap();
    let _eq_old = place_kind(&mut e2, 0, CardKind::Qinggang);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: _eq_old, targets: vec![] });
    let _j_old = e2.debug_place_judge(0, "le");
    let shan2 = e2.debug_place(2, "shan");
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: xl });
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(2)]);
    assert!(e2.debug_equips(0).is_empty(), "xiongluan discards existing equips");
    assert!(e2.debug_judge(0).is_empty(), "xiongluan discards existing judge cards");
    assert!(!e2.catalog().is_equip_slot_enabled(e2.state_core(), 0, EquipSlot::Weapon));
    assert!(e2.catalog().blocks_hand_cards(e2.state_core(), 2));
    assert!(e2.catalog().blocks_delayed_trick(e2.state_core(), 0));

    let s_a = e2.debug_place(0, "sha");
    let s_b = e2.debug_place(0, "sha");
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: s_a, targets: vec![2] });
    // Target cannot respond with hand Shan (auto-hits immediately!)
    assert_eq!(e2.debug_hp(2), 4);
    assert!(e2.debug_hand(2).contains(&shan2));
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: s_b, targets: vec![2] });
    assert_eq!(e2.debug_hp(2), 3);

    advance_to_phase(&mut e2, 0, Phase::Discard);
    assert!(!e2.catalog().blocks_hand_cards(e2.state_core(), 2), "xiongluan hand lock must expire at phaseUseEnd");

    // 3. mingce & nsyangwu
    let mut e3 = start_clean_4p_sans_general(910_009);
    e3.debug_grant(0, "mingce");
    e3.debug_grant(0, "nsyangwu");
    let mc = e3.catalog().skill_by_name("mingce").unwrap();
    let yw = e3.catalog().skill_by_name("nsyangwu").unwrap();
    let mc_sha = e3.debug_place(0, "sha");
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: mc });
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Card(mc_sha)]);
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(3)]);
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(2)]);
    assert!(prompt_is(&e3, "l11_vm_a_mingce_choice"));
    let _ = answer(&mut e3, 3, vec![ChoiceOption::Tag(1)]); // Virtual Sha on seat 2
    drain_ai(&mut e3, 8);
    assert!(e3.debug_hand(3).contains(&mc_sha));
    assert_eq!(e3.debug_hp(2), 4);

    // nsyangwu: owner has 1 Heart card, seat 1 has 4 cards -> after discarding Heart (owner 0, seat 1 = 4) -> gain ceil(4/2)=2
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    let heart_c = place_by_suit(&mut e3, 0, "shan", Suit::Heart);
    for _ in 0..4 {
        e3.debug_place(1, "tao");
    }
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: yw });
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Card(heart_c)]);
    assert!(prompt_is(&e3, "l11_vm_a_nsyangwu_gain"));
    let gain_opts: Vec<_> = e3.open_choice().unwrap().spec.options.clone().into_iter().take(2).collect();
    assert_eq!(gain_opts.len(), 2);
    let _ = answer(&mut e3, 0, gain_opts);
    assert_eq!(e3.debug_hand(0).len(), 2);
    assert_eq!(e3.debug_hand(1).len(), 2);
    let _ = DamageNature::Fire;
}
