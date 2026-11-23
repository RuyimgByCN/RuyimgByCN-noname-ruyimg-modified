use super::*;
use engine_native::machine::LogEvent;
use engine_native::view::HandView;
use engine_native::RejectReason;

#[test]
fn u05_b1_round2_fix01_02_03_jiaoxia_jieyan_zonghuo() {
    // Fix #1 (`diy_jiaoxia`): optional (`frequent: true`, `trigger_confirm` prompt) -> confirm draws 1, decline draws 0; black does NOT trigger!
    let (verified, mut e) = start_u05_b1_engine(
        92101,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(1, "diy_jiaoxia");
    advance_to_phase(&mut e, 0, Phase::Play);
    let red_sha1 = find_card(&e, |c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Heart | Suit::Diamond));
    let red_sha2 = find_card_excl(&e, &[red_sha1], |c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Heart | Suit::Diamond));
    let black_sha = find_card(&e, |c| c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade | Suit::Club));
    e.debug_grant(0, "咆哮");
    move_card_to_hand(&mut e, 0, red_sha1);
    move_card_to_hand(&mut e, 0, red_sha2);
    move_card_to_hand(&mut e, 0, black_sha);

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: red_sha1, targets: vec![1] });
    let c_opt = e.open_choice().expect("diy_jiaoxia must ask trigger_confirm").clone();
    assert_eq!((c_opt.observer, c_opt.spec.prompt), (1, "trigger_confirm"));
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "trigger_confirm", Vec::new()); // Decline!
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 0, "declining diy_jiaoxia draws 0 cards");

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: red_sha2, targets: vec![1] });
    answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]); // Confirm!
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "confirming diy_jiaoxia draws 1 card");

    // Black Sha must NOT prompt diy_jiaoxia!
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: black_sha, targets: vec![1] });
    if let Some(c) = e.open_choice() {
        assert_ne!(c.spec.prompt, "trigger_confirm", "black card must not trigger diy_jiaoxia");
    }
    drain(&mut e);

    // Fix #2 (`jieyan`): forward seat order `sortBySeat(owner)` -> when owner = 1, DamageDealt targets = [1, 2, 3, 0]!
    let (_, mut e2) = start_u05_b1_engine(
        92102,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e2.debug_clear_hand(p);
        e2.debug_set_hp(p, 4);
    }
    e2.debug_grant(1, "jieyan");
    advance_to_phase(&mut e2, 1, Phase::Play);
    e2.debug_clear_hand(1);
    let eq_red = find_card(&e2, |c| c.kind.is_equip() && matches!(c.suit, Suit::Heart | Suit::Diamond));
    move_card_to_hand(&mut e2, 1, eq_red);
    let _ = submit(&mut e2, 1, GameAction::PlayCard { card: eq_red, targets: vec![] });
    drain(&mut e2);
    // Equip-only red card cannot activate jieyan (hand-only!)
    assert!(
        matches!(activate(&mut e2, 1, "jieyan"), CommandOutcome::Rejected(_)),
        "jieyan requires red card in HAND, not equip"
    );
    let red_shan = find_card(&e2, |c| c.kind == CardKind::Shan && matches!(c.suit, Suit::Heart | Suit::Diamond));
    move_card_to_hand(&mut e2, 1, red_shan);
    let log_start = e2.log().len();
    assert!(!matches!(activate(&mut e2, 1, "jieyan"), CommandOutcome::Rejected(_)));
    answer(&mut e2, "u05_jieyan_red", vec![ChoiceOption::Card(red_shan)]);
    drain(&mut e2);
    let dmg_order: Vec<u8> = e2.log()[log_start..]
        .iter()
        .filter_map(|ev| match ev {
            LogEvent::DamageDealt { target, .. } => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(dmg_order, vec![1, 2, 3, 0], "jieyan must deal Fire damage in forward sortBySeat(owner) order!");

    // Fix #3 (`zonghuo`): hand-only (`position: "h"`), equips excluded from prompt options!
    let (_, mut e3) = start_u05_b1_engine(
        92103,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e3.debug_clear_hand(p);
        e3.debug_set_hp(p, 4);
    }
    e3.debug_grant(0, "zonghuo");
    advance_to_phase(&mut e3, 0, Phase::Play);
    let eq0 = find_card(&e3, |c| c.kind == CardKind::Weapon);
    let sha_last = find_card(&e3, |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e3, 0, eq0);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: eq0, targets: vec![] });
    drain(&mut e3);
    e3.state_core_mut().players[1].linked = true;
    e3.state_core_mut().players[2].linked = true;
    // Play Sha as ONLY hand card (equips has eq0, hand empty at damageBefore): zonghuo must NOT trigger on equip!
    move_card_to_hand(&mut e3, 0, sha_last);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: sha_last, targets: vec![1] });
    drain(&mut e3);
    assert_eq!(e3.debug_hp(1), 3);
    assert_eq!(e3.debug_hp(2), 4, "zonghuo must NOT trigger when only equips remain");
    assert_eq!(e3.debug_equips(0), vec![eq0], "equip untouched by zonghuo");

    // When hand has 1 card + equip has eq0, only hand card is offered in u05_zonghuo_discard!
    e3.debug_grant(0, "咆哮");
    let sha2 = find_card_excl(&e3, &[sha_last], |c| c.kind == CardKind::Sha);
    let hand_keep = find_card(&e3, |c| c.kind == CardKind::Tao);
    move_card_to_hand(&mut e3, 0, sha2);
    move_card_to_hand(&mut e3, 0, hand_keep);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    let c_zh = e3.open_choice().expect("zonghuo discard prompt").clone();
    assert_eq!(c_zh.spec.prompt, "u05_zonghuo_discard");
    assert!(c_zh.spec.options.contains(&ChoiceOption::Card(hand_keep)));
    assert!(!c_zh.spec.options.contains(&ChoiceOption::Card(eq0)), "equip must NOT be in zonghuo options");
    answer(&mut e3, "u05_zonghuo_discard", vec![ChoiceOption::Card(hand_keep)]);
    drain(&mut e3);
    assert_eq!(e3.debug_hp(2), 3, "confirmed hand discard spreads Fire damage via chain");
}

#[test]
fn u05_b1_round2_fix04_mhuanyi_expiry_and_target_enabled2() {
    let (verified, mut e) = start_u05_b1_engine(
        92104,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "mhuanyi");
    advance_to_phase(&mut e, 0, Phase::Discard);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    answer(&mut e, "u05_mhuanyi_target", vec![ChoiceOption::Player(2)]);
    answer(&mut e, "u05_mhuanyi_kind", vec![ChoiceOption::Tag(1)]); // 非基本牌
    drain(&mut e);
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 3);

    // Put Lebu on seat 2 -> targetEnabled2(lebu2, seat 1, seat 2) is FALSE -> no redirect, mark preserved!
    let le1 = find_card(&e, |c| c.kind == CardKind::Le);
    let le2 = find_card_excl(&e, &[le1], |c| c.kind == CardKind::Le);
    e.state_core_mut().deck.retain(|&c| c != le1);
    e.state_core_mut().discard.retain(|&c| c != le1);
    e.state_core_mut().player_mut(2).judge.push(le1);
    advance_to_phase(&mut e, 1, Phase::Play);
    move_card_to_hand(&mut e, 1, le2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: le2, targets: vec![0] });
    drain(&mut e);
    assert_eq!(e.state_core().player(0).judge, vec![le2], "le2 must hit seat 0 without illegal redirect");
    assert_eq!(e.state_core().player(2).judge, vec![le1], "seat 2 keeps existing le1");
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 3, "temp redirect mark preserved after illegal targetEnabled2!");

    // Now seat 1 uses Dismantle on seat 0 (with hand card on seat 0 and equip on seat 2) -> legal redirect consumes mark!
    let shan0 = find_card(&e, |c| c.kind == CardKind::Shan);
    let eq2 = find_card(&e, |c| c.kind == CardKind::Armor);
    let guohe = find_card(&e, |c| c.kind == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, shan0);
    let h0_before_guohe = e.debug_hand(0).len();
    e.state_core_mut().deck.retain(|&c| c != eq2);
    e.state_core_mut().player_mut(2).equips.push(eq2);
    move_card_to_hand(&mut e, 1, guohe);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: guohe, targets: vec![0] });
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), h0_before_guohe, "redirected guohe spared seat 0 hand");
    assert!(e.debug_hand(0).contains(&shan0), "redirected guohe spared shan0 on seat 0");
    assert!(e.debug_equips(2).is_empty(), "redirected guohe dismantled seat 2");
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 0, "temp mark cleared once legally redirected");

    // Step B: advance to round 3 (`round + 2 = 3`), trigger mhuanyi again -> advance to seat 0 next turn Phase::Prepare (`TurnBegin(0)`) -> must EXPIRE!
    advance_to_phase(&mut e, 2, Phase::Play);
    advance_to_phase(&mut e, 0, Phase::Discard);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
    let c_tgt = e.open_choice().expect("mhuanyi second trigger").clone();
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, &c_tgt.spec.prompt, vec![ChoiceOption::Player(2)]);
    answer(&mut e, "u05_mhuanyi_kind", vec![ChoiceOption::Tag(0)]); // 基本牌
    drain(&mut e);
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 3);
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 0, "mhuanyi temp mark MUST expire on owner phaseBegin / TurnBegin!");
    advance_to_phase(&mut e, 1, Phase::Play);
    let sha1 = find_card(&e, |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 1, sha1);
    let hp0 = e.debug_hp(0);
    let hp2 = e.debug_hp(2);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 - 1, "expired mhuanyi does not redirect next round");
    assert_eq!(e.debug_hp(2), hp2);
}

#[test]
fn u05_b1_round2_fix05_11_msuoling_and_mtongnian() {
    let (verified, mut e) = start_u05_b1_engine(
        92105,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname", "shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "msuoling");
    e.debug_grant(0, "mtongnian");
    e.debug_grant(2, "奇袭");

    // First verify Fix #11 (`mtongnian`): mirror link clears IMMEDIATELY when last enemy card is discarded mid-Play!
    let enemy_card = find_card(&e, |c| c.kind == CardKind::Shan);
    move_card_to_hand(&mut e, 1, enemy_card);
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(
        e.debug_hand(0).iter().filter(|&&c| flag(&e, 0, &format!("mtongnian_link:{c}")) > 0).count(),
        1,
        "mtongnian creates mirror link at phaseUseBegin"
    );
    let guohe0 = find_card(&e, |c| c.kind == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, guohe0);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: guohe0, targets: vec![1] });
    drain(&mut e);
    assert!(e.debug_hand(1).is_empty());
    assert_eq!(e.turn().phase, Phase::Play, "still inside seat 0 Play phase");
    assert_eq!(
        e.debug_hand(0).iter().filter(|&&c| flag(&e, 0, &format!("mtongnian_link:{c}")) > 0).count(),
        0,
        "mtongnian link cards must clear immediately when enemies empty hand!"
    );

    // Next test Fix #5 (`msuoling`):
    // 1) Delayed trick (Le) & viewAs converted trick (qixi Spade Nullify -> Dismantle) MUST NOT trigger msuoling!
    advance_to_phase(&mut e, 1, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let le_h = find_card(&e, |c| c.kind == CardKind::Le);
    let le_suit = e.catalog().cards[le_h as usize].suit;
    let same_le_0 = find_card_excl(&e, &[le_h], |c| c.suit == le_suit);
    move_card_to_hand(&mut e, 0, same_le_0);
    move_card_to_hand(&mut e, 1, le_h);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: le_h, targets: vec![2] });
    assert!(e.open_choice().is_none(), "delayed trick must NOT trigger msuoling");
    drain(&mut e);
    let cleared_j: Vec<_> = e.state_core_mut().player_mut(2).judge.drain(..).collect();
    e.state_core_mut().discard.extend(cleared_j);

    // qixi viewAs Spade Nullify as Dismantle (even with Spade card in seat 0 hand) -> use_as.is_some() -> NO msuoling!
    advance_to_phase(&mut e, 2, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let spade_nullify = find_card_excl(&e, &[le_h, same_le_0], |c| c.kind == CardKind::Nullify && c.suit == Suit::Spade);
    let spade_0 = find_card_excl(&e, &[le_h, same_le_0, spade_nullify], |c| c.suit == Suit::Spade && c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, spade_0);
    move_card_to_hand(&mut e, 2, spade_nullify);
    move_card_to_hand(&mut e, 3, same_le_0);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: spade_nullify, targets: vec![3] });
    assert!(e.open_choice().is_none(), "viewAs converted trick must NOT trigger msuoling");
    drain(&mut e);
    assert!(!e.debug_hand(3).contains(&same_le_0), "qixi converted Dismantle actually resolved and dismantled seat 3");

    // 2) Physical Snatch (`dist(0,1)=1`, `dist(0,2)=2`): seat 1 snatches seat 2 -> seat 0 steals -> ONLY seat 1 in range!
    advance_to_phase(&mut e, 1, Phase::Play);
    let sn = find_card(&e, |c| c.kind == CardKind::Snatch);
    let sn_suit = e.catalog().cards[sn as usize].suit;
    let same_sn_0 = find_card_excl(&e, &[sn], |c| c.suit == sn_suit && c.kind != CardKind::Nullify);
    let keep_p1 = find_card_excl(&e, &[sn, same_sn_0], |c| c.kind == CardKind::Shan);
    let keep_p2 = find_card_excl(&e, &[sn, same_sn_0, keep_p1], |c| c.kind == CardKind::Shan);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    move_card_to_hand(&mut e, 0, same_sn_0);
    move_card_to_hand(&mut e, 1, sn);
    move_card_to_hand(&mut e, 1, keep_p1);
    move_card_to_hand(&mut e, 2, keep_p2);
    let out_sn = submit(&mut e, 1, GameAction::PlayCard { card: sn, targets: vec![2] });
    assert!(!matches!(out_sn, CommandOutcome::Rejected(_)), "snatch rejected: {out_sn:?}");
    answer(&mut e, "u05_msuoling_discard", vec![ChoiceOption::Card(same_sn_0)]);
    let c_tgt = e.open_choice().expect("msuoling forced target choice").clone();
    assert_eq!(c_tgt.spec.prompt, "u05_msuoling_target");
    assert!(!c_tgt.spec.allow_cancel, "msuoling chooseUseTarget is forced");
    assert!(c_tgt.spec.options.contains(&ChoiceOption::Player(1)), "dist(0,1)=1 allowed");
    assert!(!c_tgt.spec.options.contains(&ChoiceOption::Player(2)), "dist(0,2)=2 out of Snatch range!");
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "u05_msuoling_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&keep_p1), "seat 0 snatched seat 1's card via stolen Snatch");
    assert!(e.debug_hand(2).contains(&keep_p2), "seat 2 untouched after original Snatch canceled");

    // 3) Advance round -> physical Geanguanhuo requires min=2, max=2 distinct targets!
    advance_to_phase(&mut e, 2, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Play);
    let ggh = find_card(&e, |c| c.kind == CardKind::Geanguanhuo);
    let ggh_suit = e.catalog().cards[ggh as usize].suit;
    let same_ggh_0 = find_card_excl(&e, &[ggh], |c| c.suit == ggh_suit && c.kind != CardKind::Nullify);
    let c2 = find_card_excl(&e, &[ggh, same_ggh_0], |c| c.rank <= 5);
    let c3 = find_card_excl(&e, &[ggh, same_ggh_0, c2], |c| c.rank >= 10);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    move_card_to_hand(&mut e, 0, same_ggh_0);
    move_card_to_hand(&mut e, 1, ggh);
    move_card_to_hand(&mut e, 2, c2);
    move_card_to_hand(&mut e, 3, c3);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: ggh, targets: vec![2, 3] });
    answer(&mut e, "u05_msuoling_discard", vec![ChoiceOption::Card(same_ggh_0)]);
    let c_ggh = e.open_choice().expect("msuoling geanguanhuo 2-target ask").clone();
    assert_eq!((c_ggh.spec.min, c_ggh.spec.max, c_ggh.spec.allow_cancel), (2, 2, false));
    answer(&mut e, "u05_msuoling_target", vec![ChoiceOption::Player(2), ChoiceOption::Player(3)]);
    drain(&mut e);
    let planner = engine_native::ai::BoundedPlanner::new(16);
    let _ = e.plan_action(1, &planner);
}

#[test]
fn u05_b1_round2_fix06_g_chenhuodajie_and_fix07_08_09() {
    let (verified, mut e) = start_u05_b1_engine(
        92106,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname", "shahai-mtg-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_phase(&mut e, 0, Phase::Play);

    // Fix #6 (`g_chenhuodajie`): formal pack global trigger emits REAL CardUsed + nullify_window + cancelable!
    let chd = find_card(&e, |c| c.kind == CardKind::Chenhuodajie);
    let wx = find_card(&e, |c| c.kind == CardKind::Nullify);
    let loot1 = find_card(&e, |c| c.kind == CardKind::Tao);
    let eq3 = find_card(&e, |c| c.kind == CardKind::Armor);
    move_card_to_hand(&mut e, 0, chd);
    move_card_to_hand(&mut e, 1, loot1);
    move_card_to_hand(&mut e, 1, wx);
    e.state_core_mut().deck.retain(|&c| c != eq3);
    e.state_core_mut().player_mut(3).equips.push(eq3);
    e.debug_grant(0, "咆哮");

    // 6a. Cancel `g_chenhuodajie_use` -> card remains in seat 0 hand, seat 1 keeps loot1!
    let sha_a = find_card(&e, |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, sha_a);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    let c_use = e.open_choice().expect("g_chenhuodajie_use cancelable prompt").clone();
    assert_eq!((c_use.observer, c_use.spec.prompt, c_use.spec.allow_cancel), (0, "g_chenhuodajie_use", true));
    answer(&mut e, "g_chenhuodajie_use", Vec::new()); // Cancel!
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&chd), "canceled g_chenhuodajie keeps card in hand");
    assert!(e.debug_hand(1).contains(&loot1));

    // 6b. Accept `g_chenhuodajie_use` -> real CardUsed triggers `nullify_window` (wuxie) -> nullified, card discarded!
    let sha_b = find_card_excl(&e, &[sha_a], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, sha_b);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
    let c_use2 = e.open_choice().expect("g_chenhuodajie_use prompt 2").clone();
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, &c_use2.spec.prompt, vec![ChoiceOption::Card(chd)]);
    let c_wx = e.open_choice().expect("g_chenhuodajie must open real nullify_window").clone();
    assert_eq!((c_wx.observer, c_wx.spec.prompt), (1, "nullify_window"));
    answer(&mut e, "nullify_window", vec![ChoiceOption::Card(wx)]); // Wuxie nullifies!
    drain(&mut e);
    assert!(!e.debug_hand(0).contains(&chd), "used g_chenhuodajie removed from hand even when nullified");
    assert!(e.debug_hand(1).contains(&loot1), "wuxie prevented steal");

    // 6c. Accept without wuxie -> gains target equip via real CardUsed!
    let sha_c = find_card_excl(&e, &[sha_a, sha_b], |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, chd);
    move_card_to_hand(&mut e, 0, sha_c);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha_c, targets: vec![3] });
    answer(&mut e, "g_chenhuodajie_use", vec![ChoiceOption::Card(chd)]);
    answer(&mut e, "u05_chenhuodajie_gain", vec![ChoiceOption::Card(eq3)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&eq3));

    // Fix #7 (`mtg_du_*` & `toulianghuanzhu` swap): swapping du does NOT lose HP; discard DOES!
    let dz = find_card(&e, |c| c.name == "mtg_duzhao");
    move_card_to_hand(&mut e, 0, dz);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: dz, targets: vec![] });
    answer(&mut e, "u05_mtg_duzhao_target", vec![ChoiceOption::Player(0)]);
    drain(&mut e);
    let du0 = *e.debug_hand(0).iter().find(|&&c| e.catalog().cards[c as usize].name.starts_with("mtg_du_")).unwrap();
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: du0, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::WrongPhase)
    ));
    let tl = find_card(&e, |c| c.kind == CardKind::Toulianghuanzhu);
    move_card_to_hand(&mut e, 0, tl);
    let hp0 = e.debug_hp(0);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: tl, targets: vec![1] });
    answer(&mut e, "wd_touliang_own", vec![ChoiceOption::Card(du0)]);
    answer(&mut e, "wd_touliang_target", vec![ChoiceOption::Card(loot1)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0, "toulianghuanzhu face-down swap of du must cause 0 HP loss");
    assert!(e.debug_hand(1).contains(&du0));
    let hp1 = e.debug_hp(1);
    assert_eq!(e.debug_hand(1), vec![du0], "seat 1 holds only du0 before dismantle");
    let guohe = find_card(&e, |c| c.kind == CardKind::Dismantle);
    move_card_to_hand(&mut e, 0, guohe);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: guohe, targets: vec![1] });
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "visible discard of du deals 1 LoseHp via g_du");

    // Fix #8 (`geanguanhuo`): reject duplicate target [1, 1] AND allow canceling tie discard!
    let ggh = find_card(&e, |c| c.kind == CardKind::Geanguanhuo);
    let eq0 = find_card(&e, |c| c.kind == CardKind::Weapon);
    let r1 = find_card(&e, |c| c.rank == 7 && c.kind == CardKind::Shan);
    let r2 = find_card_excl(&e, &[r1], |c| c.rank == 7 && c.kind == CardKind::Sha);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    move_card_to_hand(&mut e, 0, ggh);
    move_card_to_hand(&mut e, 0, eq0);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: eq0, targets: vec![] });
    drain(&mut e);
    move_card_to_hand(&mut e, 1, r1);
    move_card_to_hand(&mut e, 2, r2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: ggh, targets: vec![1, 1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: ggh, targets: vec![1, 2] });
    answer(&mut e, "yc_gean_t1_pick", vec![ChoiceOption::Card(r1)]);
    answer(&mut e, "yc_gean_t2_pick", vec![ChoiceOption::Card(r2)]);
    let c_tie = e.open_choice().expect("u05_gean_tie_discard ask").clone();
    assert_eq!((c_tie.observer, c_tie.spec.prompt, c_tie.spec.min, c_tie.spec.allow_cancel), (1, "u05_gean_tie_discard", 0, true));
    answer(&mut e, "u05_gean_tie_discard", Vec::new()); // Cancel!
    drain(&mut e);
    assert_eq!(e.debug_equips(0), vec![eq0], "canceling geanguanhuo tie discard leaves caller equip intact");

    // Fix #9 (`toulianghuanzhu`): caller with 0 remaining hand cards views target's hand (CardsRevealed to caller only)!
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let p1_secret = find_card(&e, |c| c.kind == CardKind::Tao);
    move_card_to_hand(&mut e, 0, tl);
    move_card_to_hand(&mut e, 1, p1_secret);
    let log_idx = e.log().len();
    let out_tl = submit(&mut e, 0, GameAction::PlayCard { card: tl, targets: vec![1] });
    assert!(!matches!(out_tl, CommandOutcome::Rejected(_)));
    drain(&mut e);
    assert!(e.debug_hand(1).contains(&p1_secret), "0-card toulianghuanzhu does not swap");
    assert_eq!(flag(&e, 0, "u05_touliang_viewed:1"), 1, "caller recorded viewHandcards on target 1");
    assert!(
        e.log()[log_idx..].iter().any(|ev| matches!(ev, LogEvent::CardsRevealed { cards, to } if cards == &[p1_secret] && to == &[0])),
        "CardsRevealed logged only to caller seat 0"
    );
    assert!(matches!(e.view_for(2).players[1].hand, HandView::Hidden), "other seats keep target hand hidden");
}

#[test]
fn u05_b1_round2_fix10_shepanzhen_enable_and_swapseat_order() {
    let (_, mut e5) = start_u05_b1_engine(
        92109,
        5,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-zhenfa-upstream-noname"],
    );
    let wei = (0..e5.catalog().general_count() as u16)
        .find(|&g| e5.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wei))
        .map(|g| e5.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let shu = (0..e5.catalog().general_count() as u16)
        .find(|&g| e5.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Shu))
        .map(|g| e5.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let wu = (0..e5.catalog().general_count() as u16)
        .find(|&g| e5.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wu))
        .map(|g| e5.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    for p in 0..5 {
        e5.debug_clear_hand(p);
    }
    // 1) Partial inline `[0(Wei), 1(Wei), 2(Shu), 3(Wei), 4(Wu)]`:
    //    0 IS inline with 1, yet 3 is separated by 2(Shu) & 4(Wu) -> MUST enable & swap!
    e5.debug_assemble_general(0, &wei);
    e5.debug_assemble_general(1, &wei);
    e5.debug_assemble_general(2, &shu);
    e5.debug_assemble_general(3, &wei);
    e5.debug_assemble_general(4, &wu);
    advance_to_phase(&mut e5, 0, Phase::Play);
    assert!(e5.state_core().inline(e5.catalog(), 0), "0 & 1 are adjacent");
    assert!(!e5.state_core().inline_with(e5.catalog(), 0, 3), "3 is NOT inline with 0");
    let sp5 = find_card(&e5, |c| c.name == "shepanzhen");
    move_card_to_hand(&mut e5, 0, sp5);
    let _ = submit(&mut e5, 0, GameAction::PlayCard { card: sp5, targets: vec![] });
    drain(&mut e5);
    assert_eq!(
        e5.state_core().seat_ring,
        vec![0, 1, 3, 2, 4],
        "shepanzhen swaps 3 with next_alive(1)=2 -> [0,1,3,2,4] (preserves 0,1,4!)"
    );
    assert!(e5.state_core().inline_with(e5.catalog(), 0, 3));

    // 2) CDP parity `[0(Shu), 1(Wei), 2(Shu), 3(Wei)]`, owner = 1 plays shepanzhen -> `[0, 1, 3, 2]`!
    let (_, mut e) = start_u05_b1_engine(
        92110,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-zhenfa-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sp = find_card(&e, |c| c.name == "shepanzhen");
    e.debug_assemble_general(0, &shu);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_assemble_general(3, &wei);
    advance_to_phase(&mut e, 1, Phase::Play);
    move_card_to_hand(&mut e, 1, sp);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sp, targets: vec![] });
    drain(&mut e);
    assert_eq!(
        e.state_core().seat_ring,
        vec![0, 1, 3, 2],
        "p1 plays shepanzhen -> swapSeat(p3, p1.next=p2) yields [0(Shu), 1(Wei), 3(Wei), 2(Shu)]!"
    );

    // 3) Now all friends ARE inline -> shepanzhen disables & recasts!
    move_card_to_hand(&mut e, 1, sp);
    let h1_before = e.debug_hand(1).len();
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: sp, targets: vec![] });
    drain(&mut e);
    assert_eq!(e.debug_hand(1).len(), h1_before, "fully-inline shepanzhen recasts (-1 +1 = net 0)");
    assert_eq!(e.state_core().seat_ring, vec![0, 1, 3, 2], "recast leaves seat_ring intact");
}

#[test]
fn u05_b1_round3_mhuanyi_cancel_jiedao_and_du_parity() {
    // 1. `mhuanyi` (`character/mtg/skill.js:108,133-135`):
    //    - 1a: stage-2 `u05_mhuanyi_kind` MUST be cancelable (`allow_cancel: true`) and leave flags clear.
    //    - 1b: when `u.player == t` (e.g. `p1` plays `tiesuo` on `p0`), MUST NOT redirect back onto `p1`; keeps `mhuanyi` mark!
    let (verified, mut e) = start_u05_b1_engine(
        92111,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname", "shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_grant(0, "mhuanyi");
    advance_to_phase(&mut e, 0, Phase::Discard);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "u05_mhuanyi_target", vec![ChoiceOption::Player(1)]);
    let c_kind = e.open_choice().expect("mhuanyi kind choice").clone();
    assert_eq!((c_kind.observer, c_kind.spec.prompt, c_kind.spec.allow_cancel), (0, "u05_mhuanyi_kind", true));
    let mut e = snapshot_roundtrip(&e, &verified);
    let planner = engine_native::ai::BoundedPlanner::new(16);
    let _ = e.plan_action(0, &planner);
    answer(&mut e, "u05_mhuanyi_kind", Vec::new()); // Cancel stage 2 (`cancel2`)!
    drain(&mut e);
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 0, "cancel2 clears mhuanyi target");
    assert_eq!(flag(&e, 0, "u05_mhuanyi_cls"), 0, "cancel2 clears mhuanyi class");

    // Re-arm mhuanyi [p1, trick(Tag(1))] on next round:
    advance_to_phase(&mut e, 0, Phase::Discard);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    answer(&mut e, "u05_mhuanyi_target", vec![ChoiceOption::Player(1)]);
    answer(&mut e, "u05_mhuanyi_kind", vec![ChoiceOption::Tag(1)]);
    drain(&mut e);
    assert_eq!((flag(&e, 0, "u05_mhuanyi_tgt"), flag(&e, 0, "u05_mhuanyi_cls")), (2, 2));

    // Seat 1 (who IS `t`!) uses Chain (`tiesuo`) on Seat 0 -> MUST NOT redirect to Seat 1!
    advance_to_phase(&mut e, 1, Phase::Play);
    let ts1 = find_card(&e, |c| c.kind == CardKind::Chain);
    move_card_to_hand(&mut e, 1, ts1);
    let out_ts1 = submit(&mut e, 1, GameAction::PlayCard { card: ts1, targets: vec![0] });
    assert!(!matches!(out_ts1, CommandOutcome::Rejected(_)));
    drain(&mut e);
    assert!(e.state_core().player(0).linked, "p0 takes p1's tiesuo (caster == mhuanyi_target blocked)");
    assert!(!e.state_core().player(1).linked, "p1 must NOT be chained by own tiesuo");
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 2, "mhuanyi remains armed when caster == target blocked it");

    // Seat 2 (`u.player = 2 != t = 1`) uses Chain (`tiesuo`) on Seat 0 -> DOES redirect onto Seat 1!
    advance_to_phase(&mut e, 2, Phase::Play);
    let ts2 = find_card_excl(&e, &[ts1], |c| c.kind == CardKind::Chain);
    move_card_to_hand(&mut e, 2, ts2);
    let out_ts2 = submit(&mut e, 2, GameAction::PlayCard { card: ts2, targets: vec![0] });
    assert!(!matches!(out_ts2, CommandOutcome::Rejected(_)));
    drain(&mut e);
    assert!(e.state_core().player(1).linked, "p2's tiesuo legally redirects from p0 to p1");
    assert_eq!(flag(&e, 0, "u05_mhuanyi_tgt"), 0, "mhuanyi mark cleared after legal redirect");

    // 2. `msuoling` + `Jiedao` (`card/standard.js:2984-2997`):
    //    - 2a: named weapon (`Zhuge` / `EquipSlot::Weapon`) qualifies as `Jiedao` holder.
    //    - 2b: `msuoling` owner (`actor == 0`) is legal `Jiedao` victim!
    let (verified2, mut e2) = start_u05_b1_engine(
        92112,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname"],
    );
    for p in 0..4 {
        e2.debug_clear_hand(p);
        e2.debug_set_hp(p, 4);
    }
    e2.debug_grant(0, "msuoling");
    advance_to_phase(&mut e2, 3, Phase::Play);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let zhuge = find_card(&e2, |c| c.kind == CardKind::Zhuge);
    e2.state_core_mut().deck.retain(|&c| c != zhuge);
    e2.state_core_mut().discard.retain(|&c| c != zhuge);
    e2.state_core_mut().player_mut(1).equips.push(zhuge);
    let jd = find_card(&e2, |c| c.kind == CardKind::Jiedao);
    let jd_suit = e2.catalog().cards[jd as usize].suit;
    let same_jd_0 = find_card_excl(&e2, &[jd, zhuge], |c| c.suit == jd_suit && c.kind != CardKind::Nullify);
    move_card_to_hand(&mut e2, 0, same_jd_0);
    move_card_to_hand(&mut e2, 3, jd);
    let out_jd = submit(&mut e2, 3, GameAction::PlayCard { card: jd, targets: vec![1, 2] });
    assert!(!matches!(out_jd, CommandOutcome::Rejected(_)), "play jiedao with Zhuge holder: {out_jd:?}");
    answer(&mut e2, "u05_msuoling_discard", vec![ChoiceOption::Card(same_jd_0)]);
    let c_holder = e2.open_choice().expect("msuoling jiedao holder ask").clone();
    assert_eq!(c_holder.spec.prompt, "u05_msuoling_target");
    assert!(c_holder.spec.options.contains(&ChoiceOption::Player(1)), "Zhuge equips seat 1 as EquipSlot::Weapon holder!");
    answer(&mut e2, "u05_msuoling_target", vec![ChoiceOption::Player(1)]);
    let c_victim = e2.open_choice().expect("msuoling jiedao victim ask").clone();
    assert_eq!(c_victim.spec.prompt, "u05_msuoling_target");
    assert!(c_victim.spec.options.contains(&ChoiceOption::Player(0)), "actor=0 in range(1,0)=1 MUST be eligible Jiedao victim!");
    let mut e2 = snapshot_roundtrip(&e2, &verified2);
    answer(&mut e2, "u05_msuoling_target", vec![ChoiceOption::Player(0)]);
    let hp0_before = e2.debug_hp(0);
    answer(&mut e2, "jiedao_sha_or_give", vec![ChoiceOption::Tag(0)]); // Seat 1 strikes Seat 0
    drain(&mut e2);
    assert_eq!(e2.debug_hp(0), hp0_before - 1, "stolen Jiedao struck msuoling owner seat 0 via seat 1's Zhuge!");

    // 3. `mtg_du_*` (`card/yongjian.js:595-624`):
    //    - 3a: hidden `Snatch` taking `du` from hand -> 0 HP loss!
    //    - 3b: `fudichouxin` compare with `du` (`winner gains via move_cards_to_hand` or keeps `du`) -> 1 HP loss!
    let (verified3, mut e3) = start_u05_b1_engine(
        92113,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-mtg-upstream-noname", "shahai-yunchou-upstream-noname"],
    );
    for p in 0..4 {
        e3.debug_clear_hand(p);
        e3.debug_set_hp(p, 4);
    }
    advance_to_phase(&mut e3, 0, Phase::Play);
    e3.debug_clear_hand(0);
    let dz = find_card(&e3, |c| c.name == "mtg_duzhao");
    move_card_to_hand(&mut e3, 0, dz);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: dz, targets: vec![] });
    answer(&mut e3, "u05_mtg_duzhao_target", vec![ChoiceOption::Player(1)]);
    drain(&mut e3);
    let du1 = *e3.debug_hand(1).iter().find(|&&c| e3.catalog().cards[c as usize].name.starts_with("mtg_du_")).expect("du1 dealt");
    let shun = find_card(&e3, |c| c.kind == CardKind::Snatch);
    move_card_to_hand(&mut e3, 0, shun);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: shun, targets: vec![1] });
    drain(&mut e3);
    assert!(e3.debug_hand(0).contains(&du1), "p0 snatched du1 into hand");
    assert_eq!(e3.debug_hp(1), 4, "hidden Snatch taking du from hand MUST NOT cause LoseHp!");

    // Arm `du2` into seat 1 via `mtg_duzhao_skill` at `Phase::End(1)`, then test `fudichouxin` win taking `du2`!
    advance_to_phase(&mut e3, 2, Phase::Play);
    let du2 = *e3.debug_hand(1).iter().find(|&&c| e3.catalog().cards[c as usize].name.starts_with("mtg_du_")).expect("du2 dealt at seat 1 phaseEnd");
    let du2_rank = e3.catalog().cards[du2 as usize].rank;
    advance_to_phase(&mut e3, 0, Phase::Play);
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    move_card_to_hand(&mut e3, 1, du2);
    let king = find_card(&e3, |c| c.rank > du2_rank && !c.name.starts_with("mtg_du_"));
    let fd = find_card(&e3, |c| c.kind == CardKind::Fudichouxin);
    move_card_to_hand(&mut e3, 0, king);
    move_card_to_hand(&mut e3, 0, fd);
    let out_fd = submit(&mut e3, 0, GameAction::PlayCard { card: fd, targets: vec![1] });
    assert!(!matches!(out_fd, CommandOutcome::Rejected(_)));
    let mut e3 = snapshot_roundtrip(&e3, &verified3);
    answer(&mut e3, "wd_fudi_target_pick", vec![ChoiceOption::Card(du2)]);
    answer(&mut e3, "wd_fudi_player_pick", vec![ChoiceOption::Card(king)]);
    drain(&mut e3);
    assert!(e3.debug_hand(0).contains(&du2), "fudichouxin winner gained target's du2");
    assert!(e3.debug_hand(0).contains(&king), "fudichouxin winner retained king");
    assert_eq!(e3.debug_hp(1), 3, "fudichouxin compare with du2 (`card2 == du`) MUST cost target 1 HP even when gained!");
    assert_eq!(e3.debug_hp(0), 4);

    // Also test `card1 == du` (`pr > tr` where winner 0 uses `du1` rank >= 4 against target's low card rank <= 3)!
    let low_c = find_card(&e3, |c| c.rank < e3.catalog().cards[du1 as usize].rank && !c.name.starts_with("mtg_du_"));
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    move_card_to_hand(&mut e3, 0, du1);
    move_card_to_hand(&mut e3, 0, fd);
    move_card_to_hand(&mut e3, 1, low_c);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: fd, targets: vec![1] });
    answer(&mut e3, "wd_fudi_target_pick", vec![ChoiceOption::Card(low_c)]);
    answer(&mut e3, "wd_fudi_player_pick", vec![ChoiceOption::Card(du1)]);
    drain(&mut e3);
    assert!(e3.debug_hand(0).contains(&du1), "winner keeps du1");
    assert_eq!(e3.debug_hp(0), 3, "fudichouxin compare with du1 (`card1 == du`) costs winner 1 HP too!");
    assert_eq!(e3.debug_hp(1), 3);
}
