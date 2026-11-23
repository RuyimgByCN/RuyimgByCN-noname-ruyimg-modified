use super::*;

#[test]
fn u05_b1_mtg_generals_static_metadata_and_host_bindings() {
    let (_, mut e) = start_u05_b1_engine(
        92020,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    // `character/mtg/character.js:2-18`
    let expected = [
        ("mtg_jiding", Faction::Qun, Gender::Male, 4, &["mbaizhan", "msilian"][..]),
        ("mtg_jiesi", Faction::Wei, Gender::Male, 3, &["mtongnian", "msuoling", "mhuanyi"][..]),
        ("mtg_lilianna", Faction::Qun, Gender::Female, 3, &["lingyong", "mduohun"][..]),
    ];
    for (gname, fac, gen, hp, sks) in expected {
        let gid = (0..e.catalog().general_count() as u16)
            .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.0 == gname))
            .unwrap_or_else(|| panic!("missing general {gname}"));
        let m = e.catalog().general_meta(gid).unwrap();
        assert_eq!(m.1, fac, "{gname} faction");
        assert_eq!(m.3, hp, "{gname} max_hp");
        assert_eq!(m.4, gen, "{gname} gender");
        for &sk in sks {
            assert!(m.2.contains(&sk), "{gname} missing skill {sk}: {:?}", m.2);
            let sdef = e.catalog().skill_by_name(sk).unwrap_or_else(|| panic!("missing skill {sk}"));
            assert_eq!(
                e.catalog().skills[sdef as usize].behavior.package_skill_id(),
                None,
                "MTG skill {sk} must bind real HostBehavior"
            );
        }
        e.assign_general_named(0, gname).unwrap();
    }
}

#[test]
fn u05_b1_mtg_char_skills_jiding_lilianna() {
    // 1. `mbaizhan` (damageEnd -> +hujia) & `msilian` (phaseEnd spend hujia -> extra turn with +storage-2 draw & loseHp if 0 damage)
    let (verified, mut e) = start_u05_b1_engine(
        92021,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    e.debug_grant(0, "mbaizhan");
    e.debug_grant(0, "msilian");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 5);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hujia, 2, "mbaizhan grants hujia equal to damage num");

    // Advance to seat 0's Turn End (`Phase::End` -> `msilian` trigger) with `hujia = 3` (`3 - 2 = +1` extra draw)
    e.state_core_mut().player_mut(0).hujia = 3;
    advance_to_phase(&mut e, 0, Phase::Discard);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    // Skip any non-msilian prompts until msilian prompt opens
    let c = e.open_choice().expect("msilian confirm at Phase::End").clone();
    assert_eq!(c.observer, 0);
    let e = snapshot_roundtrip(&e, &verified);
    let mut e = e;
    // Accept msilian
    answer(&mut e, &c.spec.prompt, vec![ChoiceOption::Confirm]);
    assert_eq!(e.state_core().player(0).hujia, 0, "msilian spends all hujia");
    // Extra turn should run next for seat 0 before seat 1! Advance through extra Draw phase:
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.debug_hand(0).len(),
        3,
        "msilian extra turn Phase::Draw draws 2 + (3 - 2) = 3 cards"
    );
    // End extra turn with 0 damage dealt -> loses 1 HP!
    let hp_before = e.debug_hp(0);
    advance_to_phase(&mut e, 1, Phase::Play);
    assert_eq!(e.debug_hp(0), hp_before - 1, "msilian phaseAfter 0 damage loses 1 HP");

    // 2. `mduohun` (`dyingAfter` source.loseHp) & `lingyong` (`dieAfter` capture fallen general skills + summon in Play)
    let (verified2, mut e2) = start_u05_b1_engine(
        92022,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.debug_grant(1, "mduohun");
    e2.debug_grant(0, "lingyong");
    e2.debug_set_hp(2, 4);
    e2.debug_set_hp(1, 1);
    // Give seat 1 a Tao so seat 1 survives dying, testing `mduohun` on `dyingAfter` (`source = seat 2`)!
    let tao = find_card(&e2, |c| c.kind == CardKind::Tao);
    move_card_to_hand(&mut e2, 1, tao);
    e2.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    let c_tao = e2.open_choice().expect("dying tao ask on seat 1").clone();
    assert_eq!(c_tao.observer, 1);
    answer(&mut e2, c_tao.spec.prompt, vec![ChoiceOption::Card(tao)]);
    drain(&mut e2);
    assert_eq!(e2.debug_hp(2), 3, "mduohun causes dying source to lose 1 HP on dyingAfter");

    // Now kill seat 3 (`mtg_jiding` has `["mbaizhan", "msilian"]`) -> `lingyong` stores subplayer for seat 0!
    e2.assign_general_named(3, "mtg_jiding").unwrap();
    e2.debug_set_hp(3, 1);
    e2.debug_deal_damage(Some(2), 3, 1, DamageNature::Normal);
    drain(&mut e2);
    assert!(!e2.state_core().player(3).alive);
    advance_to_phase(&mut e2, 0, Phase::Play);
    let tracked = total_tracked_cards(&e2);
    let out = activate(&mut e2, 0, "lingyong");
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "lingyong activate: {out:?}");
    let mut e2 = snapshot_roundtrip(&e2, &verified2);
    if let Some(c) = e2.open_choice().cloned() {
        answer(&mut e2, &c.spec.prompt, vec![c.spec.options[0]]);
    }
    drain(&mut e2);
    assert!(
        e2.state_core().has_skill(0, e2.catalog().skill_by_name("mbaizhan").unwrap()),
        "lingyong summons fallen mtg_jiding's mbaizhan skill"
    );
    assert_eq!(total_tracked_cards(&e2), tracked, "card conservation with lingyong subplayer");
}

#[test]
fn u05_b1_mtg_char_skills_jiesi_mtongnian_msuoling_mhuanyi() {
    let (verified, mut e) = start_u05_b1_engine(
        92023,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "mtongnian");
    e.debug_grant(0, "msuoling");
    e.debug_grant(0, "mhuanyi");
    // Give enemy seat 1 a Spade Sha for `mtongnian`
    let enemy_sha = find_card(&e, |c| c.kind == CardKind::Sha && c.suit == Suit::Spade);
    move_card_to_hand(&mut e, 1, enemy_sha);

    advance_to_phase(&mut e, 0, Phase::Play);
    // `mtongnian`: at `phaseUseBegin`, seat 0 receives a linked mirror card of enemy hand card!
    let link_cards: Vec<u32> = e
        .debug_hand(0)
        .iter()
        .copied()
        .filter(|&cid| flag(&e, 0, &format!("mtongnian_link:{cid}")) > 0)
        .collect();
    assert_eq!(link_cards.len(), 1, "mtongnian must mirror 1 enemy hand card in hand");

    // Playing the linked mirror card uses and removes the enemy's real card (`enemy_sha`)!
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: link_cards[0],
            targets: vec![1],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play mtongnian card: {out:?}");
    drain(&mut e);
    assert!(
        !e.debug_hand(1).contains(&enemy_sha),
        "mtongnian useCardBefore swaps & consumes enemy real card"
    );

    // Next: Play -> End triggers `mhuanyi` (`round: 2` -> pick target 2 & card class tag 0=Basic, 1=NonBasic)
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let c_tgt = e.open_choice().expect("mhuanyi target choice").clone();
    assert_eq!(c_tgt.observer, 0);
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, &c_tgt.spec.prompt, vec![ChoiceOption::Player(2)]);
    let c_cls = e.open_choice().expect("mhuanyi basic/nonbasic control").clone();
    // Pick Choice 0 ("基本牌")
    answer(&mut e, &c_cls.spec.prompt, vec![c_cls.spec.options[0]]);
    drain(&mut e);

    // Seat 1 uses a Basic `Sha` targeting Seat 0 -> redirected to Seat 2!
    advance_to_phase(&mut e, 1, Phase::Play);
    let sha1 = find_card_excl(&e, &[enemy_sha], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 1, sha1);
    let hp0 = e.debug_hp(0);
    let hp2 = e.debug_hp(2);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "sha play into mhuanyi: {out:?}");
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0, "mhuanyi shields seat 0");
    assert_eq!(e.debug_hp(2), hp2 - 1, "mhuanyi redirects basic card to seat 2");

    // `msuoling` (`round: 1`): Seat 1 uses Heart `Wuzhong` -> Seat 0 discards Heart card to steal & retarget!
    let wz_h = find_card(&e, |c| c.kind == CardKind::Wuzhong);
    let wz_suit = e.catalog().cards[wz_h as usize].suit;
    let same_suit = find_card_excl(&e, &[wz_h], |c| c.suit == wz_suit && c.kind == CardKind::Shan);
    move_card_to_hand(&mut e, 1, wz_h);
    move_card_to_hand(&mut e, 0, same_suit);
    let h0_before = e.debug_hand(0).len();
    let h1_before = e.debug_hand(1).len();
    let out = submit(&mut e, 1, GameAction::PlayCard { card: wz_h, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play trick for msuoling: {out:?}");
    let c_sl = e.open_choice().expect("msuoling same-suit discard prompt").clone();
    assert_eq!(c_sl.observer, 0);
    answer(&mut e, &c_sl.spec.prompt, vec![ChoiceOption::Card(same_suit)]);
    if let Some(c_tgt) = e.open_choice().cloned() {
        if c_tgt.observer == 0 {
            answer(&mut e, &c_tgt.spec.prompt, vec![c_tgt.spec.options[0]]);
        }
    }
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), h1_before - 1, "original caster canceled (-1 card, 0 draws)");
    assert_eq!(e.debug_hand(0).len(), h0_before - 1 + 2, "msuoling owner discarded 1 & drew 2 from stolen wuzhong");
}

#[test]
fn u05_b1_mtg_duzhao_cangbaohaiwan_lindixiliu_skill_and_bingheyaosai() {
    let (verified, mut e) = start_u05_b1_engine(
        92024,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname", "shahai-swd-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_phase(&mut e, 0, Phase::Play);

    // 1. `mtg_duzhao`: pick target to gain `"du"` + `phaseEnd` triggers `mtg_duzhao_skill` giving `"du"`
    let dz = find_card(&e, |c| c.name == "mtg_duzhao");
    move_card_to_hand(&mut e, 0, dz);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: dz, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play mtg_duzhao: {out:?}");
    let mut e = snapshot_roundtrip(&e, &verified);
    let c_dz = e.open_choice().expect("mtg_duzhao target choice").clone();
    // AI public-info check: hostile AI targets seat 1 or 2, never friendly
    answer(&mut e, &c_dz.spec.prompt, vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert!(
        e.debug_hand(1).iter().any(|&c| e.catalog().cards[c as usize].name.starts_with("mtg_du_")),
        "target gains du card from mtg_duzhao"
    );

    // Playing `du` or discarding `du` deals 1 HP loss to holder (`LoseHp { amount: 1 }`)
    let du_cid = *e.debug_hand(1).iter().find(|&&c| e.catalog().cards[c as usize].name.starts_with("mtg_du_")).unwrap();
    // Advance to seat 0 End to check `mtg_duzhao_skill` gives seat 0 a `du` at `phaseEnd`
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain(&mut e);
    assert!(
        e.debug_hand(0).iter().any(|&c| e.catalog().cards[c as usize].name.starts_with("mtg_du_")),
        "mtg_duzhao_skill gives active player du at phaseEnd"
    );

    // Next turn (`seat 1`): upstream `yongjian.js` `du` has no `enable` (`normalCanPlayDu: false`) — active play rejected!
    advance_to_phase(&mut e, 1, Phase::Play);
    let hp1 = e.debug_hp(1);
    let play_du = submit(&mut e, 1, GameAction::PlayCard { card: du_cid, targets: vec![] });
    assert!(matches!(play_du, CommandOutcome::Rejected(_)), "mtg_du must reject normal PlayCard: {play_du:?}");
    assert_eq!(e.debug_hp(1), hp1, "rejected du play deals 0 HP loss");

    let cb = find_card(&e, |c| c.name == "mtg_cangbaohaiwan");
    move_card_to_hand(&mut e, 1, cb);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: cb, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play mtg_cangbaohaiwan: {out:?}");
    if let Some(c_pack) = e.open_choice().cloned() {
        answer(&mut e, &c_pack.spec.prompt, vec![c_pack.spec.options[0]]);
    }
    drain(&mut e);
    assert!(!e.debug_hand(1).is_empty(), "mtg_cangbaohaiwan grants 1 derivation card");

    // Trigger seat 1 Discard phase with >hp hand cards and visibly discard `du_cid` -> `g_du` causes LoseHp 1!
    for _ in 0..5 {
        let filler = find_card_excl(&e, &[du_cid], |c| c.kind == CardKind::Shan);
        move_card_to_hand(&mut e, 1, filler);
    }
    if !e.debug_hand(1).contains(&du_cid) {
        move_card_to_hand(&mut e, 1, du_cid);
    }
    let _ = submit(&mut e, 1, GameAction::NextPhase); // Play -> Discard
    if let Some(c_disc) = e.open_choice().cloned() {
        if c_disc.spec.prompt == "discard_to_limit" && c_disc.spec.options.contains(&ChoiceOption::Card(du_cid)) {
            let mut sel = vec![ChoiceOption::Card(du_cid)];
            for opt in &c_disc.spec.options {
                if sel.len() >= c_disc.spec.min {
                    break;
                }
                if !sel.contains(opt) {
                    sel.push(*opt);
                }
            }
            answer(&mut e, "discard_to_limit", sel);
        }
    }
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "visible discard of du causes LoseHp 1");

    // 2. `mtg_lindixiliu_skill` (`card/mtg/skill.js:223-245`): `phaseBegin` optional discard 1 Heart he -> discard own judge!
    advance_to_phase(&mut e, 2, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Play);
    let lx = find_card(&e, |c| c.kind == CardKind::Lindixiliu);
    let heart_c = find_card(&e, |c| c.suit == Suit::Heart && c.kind == CardKind::Shan);
    let le_c = find_card(&e, |c| c.kind == CardKind::Le);
    move_card_to_hand(&mut e, 1, lx);
    let out_lx = submit(&mut e, 1, GameAction::PlayCard { card: lx, targets: vec![] });
    assert!(!matches!(out_lx, CommandOutcome::Rejected(_)), "play mtg_lindixiliu: {out_lx:?}");
    drain(&mut e);
    move_card_to_hand(&mut e, 2, heart_c);
    e.state_core_mut().deck.retain(|&c| c != le_c);
    e.state_core_mut().discard.retain(|&c| c != le_c);
    e.state_core_mut().player_mut(2).judge.push(le_c);
    let lx_sk = e.catalog().skill_by_name("mtg_lindixiliu_skill").unwrap();
    assert_eq!(e.state_core().current_land, Some(CardKind::Lindixiliu));
    assert!(e.state_core().has_skill(2, lx_sk), "seat 2 granted mtg_lindixiliu_skill");
    assert!(e.catalog().skills[lx_sk as usize].trigger.is_some(), "mtg_lindixiliu_skill has TriggerMeta");
    for _ in 0..80 {
        if e.open_choice()
            .as_ref()
            .is_some_and(|c| c.spec.prompt == "u05_mtg_lindixiliu_heart")
        {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
    }
    let c_lx = e.open_choice().expect("mtg_lindixiliu_skill heart discard at phaseBegin").clone();
    assert_eq!(c_lx.observer, 2);
    answer(&mut e, &c_lx.spec.prompt, vec![ChoiceOption::Card(heart_c)]);
    if let Some(c_j) = e.open_choice().cloned() {
        answer(&mut e, &c_j.spec.prompt, vec![ChoiceOption::Card(le_c)]);
    }
    drain(&mut e);
    assert!(e.state_core().player(2).judge.is_empty(), "mtg_lindixiliu_skill discards judge card");

    // 3. `mtg_bingheyaosai_skill` (`card/mtg/skill.js:5-21`): interactive player-chosen `"he"` discard on Sha!
    advance_to_phase(&mut e, 2, Phase::Play);
    let bh = find_card(&e, |c| c.kind == CardKind::Bingheyaosai);
    move_card_to_hand(&mut e, 2, bh);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: bh, targets: vec![] });
    drain(&mut e);
    let sha2 = find_card(&e, |c| c.kind == CardKind::Sha);
    let keep_c = find_card(&e, |c| c.kind == CardKind::Tao);
    let toss_c = find_card(&e, |c| c.kind == CardKind::Shan);
    e.debug_clear_hand(2);
    move_card_to_hand(&mut e, 2, sha2);
    move_card_to_hand(&mut e, 2, keep_c);
    move_card_to_hand(&mut e, 2, toss_c);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: sha2, targets: vec![0] });
    let c_bh = e.open_choice().expect("bingheyaosai interactive he discard on Sha").clone();
    assert_eq!(c_bh.observer, 2);
    assert!(c_bh.spec.options.contains(&ChoiceOption::Card(toss_c)));
    assert!(c_bh.spec.options.contains(&ChoiceOption::Card(keep_c)));
    answer(&mut e, &c_bh.spec.prompt, vec![ChoiceOption::Card(toss_c)]);
    drain(&mut e);
    assert!(e.debug_hand(2).contains(&keep_c), "bingheyaosai keeps unchosen card");
    assert!(!e.debug_hand(2).contains(&toss_c), "bingheyaosai discards player-chosen card");

    // Verify AI public-info choice on `mtg_duzhao` doesn't crash or read private hands
    let planner = engine_native::ai::BoundedPlanner::new(16);
    let _ = e.plan_action(2, &planner);
}
