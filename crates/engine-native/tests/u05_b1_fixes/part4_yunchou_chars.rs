use super::*;

#[test]
fn u05_b1_yunchou_generals_static_metadata_and_remaining_skills() {
    let (verified, mut e) = start_u05_b1_engine(
        92040,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    // 1. All 10 yunchou generals static metadata (`character/yunchou/character.js:4-73`)
    let expected: [(&str, Faction, Gender, u32); 10] = [
        ("ycn_diy_caocao", Faction::Wei, Gender::Male, 4),
        ("ycn_diy_weiyan", Faction::Shu, Gender::Male, 4),
        ("ycn_diy_sp_zhouyu", Faction::Wu, Gender::Male, 3),
        ("ycn_diy_luxun", Faction::Wu, Gender::Male, 3),
        ("ycn_diy_yuji", Faction::Qun, Gender::Male, 3),
        ("ycn_diy_zhouyu", Faction::Wu, Gender::Male, 3),
        ("ycn_diy_caiwenji", Faction::Qun, Gender::Female, 3),
        ("ycn_diy_lukang", Faction::Wu, Gender::Male, 4),
        ("ycn_diy_zhenji", Faction::Wei, Gender::Female, 3),
        ("ycn_old_majun", Faction::Wei, Gender::Male, 3),
    ];
    for (gname, fac, gen, hp) in expected {
        let gid = (0..e.catalog().general_count() as u16)
            .find(|&g| {
                e.catalog().general_meta(g).is_some_and(|m| {
                    m.0 == gname
                        || m.0 == gname.strip_prefix("ycn_").unwrap_or(gname)
                })
            })
            .unwrap_or_else(|| panic!("yunchou general {gname} missing"));
        let m = e.catalog().general_meta(gid).unwrap();
        assert_eq!(m.1, fac, "{gname} faction");
        assert_eq!(m.3, hp, "{gname} max_hp");
        assert_eq!(m.4, gen, "{gname} gender");
        for sk in m.2 {
            assert_eq!(
                e.catalog().skills[e.catalog().skill_by_name(sk).unwrap() as usize]
                    .behavior
                    .package_skill_id(),
                None,
                "yunchou skill {sk} on {} must bind native HostBehavior",
                m.0
            );
        }
    }

    // 2. `old_majun`: `xinfu_qiaosi` (dice 1..=6 -> draw num -> chooseControl give vs discard!) + `xinfu_jingxie`
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_grant(0, "ycnxinfu_qiaosi");
    e.debug_grant(0, "xinfu_jingxie");
    let arm = find_card(&e, |c| c.kind == CardKind::Armor);
    move_card_to_hand(&mut e, 0, arm);
    // 2a. `xinfu_jingxie`: upgrade Armor in Play phase + dying armor recast to 1 HP!
    let out = activate(&mut e, 0, "xinfu_jingxie");
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "xinfu_jingxie play: {out:?}");
    let c_jx = e.open_choice().expect("xinfu_jingxie card pick").clone();
    answer(&mut e, &c_jx.spec.prompt, vec![ChoiceOption::Card(arm)]);
    drain(&mut e);
    assert!(
        engine_native::rvm_bridge::modifiers::d34_l28_mods::is_jingxie_rewritten(e.state_core(), arm),
        "xinfu_jingxie marks armor rewritten/upgraded"
    );

    // 2b. `ycnxinfu_qiaosi`: throws dice `1..=6`, draws `num`, then control give vs discard!
    let tracked = total_tracked_cards(&e);
    let out = activate(&mut e, 0, "ycnxinfu_qiaosi");
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "ycnxinfu_qiaosi activate: {out:?}");
    let mut e = snapshot_roundtrip(&e, &verified);
    let c_ctrl = e.open_choice().expect("qiaosi control give vs discard").clone();
    let h0_after_draw = e.debug_hand(0).len();
    assert!(h0_after_draw >= 2, "qiaosi drew 1..=6 cards before give/discard control");
    // Pick Choice 0 (Give `num` cards to chosen target)
    answer(&mut e, &c_ctrl.spec.prompt, vec![c_ctrl.spec.options[0]]);
    let c_cards = e.open_choice().expect("qiaosi pick cards to give").clone();
    let give_n = c_cards.spec.min;
    let give_sel: Vec<ChoiceOption> = c_cards.spec.options.iter().take(give_n).copied().collect();
    answer(&mut e, &c_cards.spec.prompt, give_sel);
    let c_tgt = e.open_choice().expect("qiaosi pick target").clone();
    answer(&mut e, &c_tgt.spec.prompt, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), give_n, "qiaosi gave exact dice num cards to seat 1");
    assert_eq!(total_tracked_cards(&e), tracked, "card conservation");

    // 3. `ycndiyjianxiong` (`character/yunchou/skill.js:29-107`):
    //    `dieBefore` on own turn in Identity sets non-Lord dying player's role to Rebel before death rewards!
    let (verified_id, mut e_id) = start_u05_b1_engine(
        92041,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e_id.debug_clear_hand(p);
    }
    e_id.state_core_mut().mode = GameMode::Identity {
        roles: vec![Role::Lord, Role::Loyalist, Role::Rebel, Role::Traitor],
    };
    e_id.debug_grant(0, "ycndiyjianxiong");
    e_id.debug_set_hp(1, 1);
    advance_to_phase(&mut e_id, 0, Phase::Play);
    // Save/restore verified before conversion while Identity role invariants hold (1 Loyalist)
    let mut e_id = snapshot_roundtrip(&e_id, &verified_id);
    // Lord seat 0 kills Loyalist seat 1 on seat 0's turn -> diyjianxiong flips seat 1 to Rebel BEFORE rewards,
    // so Lord draws 3 (rebel kill reward) instead of losing all cards (loyalist kill penalty)!
    let h0_before_kill = e_id.debug_hand(0).len();
    e_id.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain(&mut e_id);
    assert_eq!(
        e_id.state_core().identity_roles().and_then(|r| r.get(1).copied()),
        Some(Role::Rebel),
        "diyjianxiong changed dying role to Rebel"
    );
    assert!(
        e_id.debug_hand(0).len() >= h0_before_kill + 3,
        "Lord killed converted Rebel and drew 3 rewards instead of loyalist discard!"
    );
}

#[test]
fn u05_b1_yunchou_guihan_zonghuo_shaoying_luweiyan2_diyguhuo_yiesheng() {
    let (verified, mut e) = start_u05_b1_engine(
        92042,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((p, "effective_gender_override".to_string()), 1);
    }

    // 1. `guihan`: dying player chooses WHICH male target (can pick seat 2 instead of seat 0, or cancel!)
    e.debug_grant(1, "guihan");
    e.debug_set_hp(1, 1);
    e.debug_set_hp(2, 2);
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let c_gh = e.open_choice().expect("guihan target selection").clone();
    assert_eq!(c_gh.observer, 1);
    assert!(c_gh.spec.allow_cancel, "guihan target choice is cancelable");
    assert!(c_gh.spec.options.contains(&ChoiceOption::Player(2)), "seat 2 male selectable");
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, &c_gh.spec.prompt, vec![ChoiceOption::Player(2)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 1, "guihan heals dying player by 1");
    assert_eq!(e.debug_hp(2), 3, "guihan heals chosen male seat 2 (2 -> 3)");
    assert_eq!(e.debug_hand(1).len(), 2);
    assert_eq!(e.debug_hand(2).len(), 2);
    assert_eq!(e.debug_hand(0).len(), 0, "unpicked male seat 0 gets nothing");

    // 2. `yiesheng` conservation + effective black color (`effective_is_red`)
    advance_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_grant(0, "yiesheng");
    let black1 = find_card(&e, |c| matches!(c.suit, Suit::Spade | Suit::Club) && c.kind == CardKind::Sha);
    let black2 = find_card_excl(&e, &[black1], |c| matches!(c.suit, Suit::Spade | Suit::Club) && c.kind == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, black1);
    move_card_to_hand(&mut e, 0, black2);
    let tracked = total_tracked_cards(&e);
    let out = activate(&mut e, 0, "yiesheng");
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "yiesheng activate: {out:?}");
    let c_ys = e.open_choice().expect("yiesheng pick").clone();
    answer(&mut e, &c_ys.spec.prompt, vec![ChoiceOption::Card(black1), ChoiceOption::Card(black2)]);
    drain(&mut e);
    assert_eq!(
        total_tracked_cards(&e),
        tracked,
        "yiesheng discarded cards MUST be moved to discard pile (card conservation!)"
    );
    assert!(e.state_core().discard.contains(&black1) && e.state_core().discard.contains(&black2));

    // 3. `zonghuo`: prompt cancelable discard before setting nature=Fire (cancel keeps Normal!)
    e.debug_grant(0, "zonghuo");
    let c_keep = find_card(&e, |c| c.kind == CardKind::Tao);
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, c_keep);
    e.state_core_mut().players[1].linked = true;
    e.state_core_mut().players[2].linked = true;
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let c_zh = e.open_choice().expect("zonghuo discard prompt").clone();
    assert!(c_zh.spec.allow_cancel);
    answer(&mut e, &c_zh.spec.prompt, Vec::new()); // Cancel!
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(e.debug_hp(2), 4, "canceling zonghuo keeps damage Normal (no chain spread)");

    // Confirm with chosen card -> Fire damage chains!
    let _ = e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let c_zh2 = e.open_choice().expect("zonghuo discard prompt 2").clone();
    answer(&mut e, &c_zh2.spec.prompt, vec![ChoiceOption::Card(c_keep)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(2), 3, "confirmed zonghuo spreads Fire damage through chain");

    // 4. `shaoying`: asks target choice (<= distance 1, cancelable) instead of deterministic .first()
    e.debug_grant(0, "shaoying");
    let red_top = find_card(&e, |c| matches!(c.suit, Suit::Heart | Suit::Diamond));
    put_deck_top(&mut e, red_top);
    let hp3 = e.debug_hp(3);
    let _ = e.debug_deal_damage(Some(0), 0, 1, DamageNature::Fire);
    let c_sy = e.open_choice().expect("shaoying target choice").clone();
    assert!(c_sy.spec.allow_cancel);
    assert!(c_sy.spec.options.contains(&ChoiceOption::Player(3)), "seat 3 is dist 1 from seat 0");
    answer(&mut e, &c_sy.spec.prompt, vec![ChoiceOption::Player(3)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(3), hp3 - 1, "shaoying deals 1 Fire dmg to chosen target 3");
}
