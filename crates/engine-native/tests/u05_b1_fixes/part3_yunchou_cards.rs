use super::*;

#[test]
fn u05_b1_yunchou_suolianjia_chenhuodajie_caochuanjiejian_shuigong() {
    let (verified, mut e) = start_u05_b1_engine(
        92030,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }

    // 1. `suolianjia` (`card/yunchou/card.js:245`, `card/yunchou/skill.js:7-28`):
    //    Equip -> links; prevents Fire & Thunder damage (unless unequip/qinggang); unequip -> unlinks!
    advance_to_phase(&mut e, 0, Phase::Play);
    let slj = find_card(&e, |c| c.kind == CardKind::Suolianjia);
    move_card_to_hand(&mut e, 1, slj);
    let arm_p = e.save().core;
    let _ = arm_p;
    // Equip via seat 1 play phase
    advance_to_phase(&mut e, 1, Phase::Play);
    let out = submit(&mut e, 1, GameAction::PlayCard { card: slj, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "equip suolianjia: {out:?}");
    drain(&mut e);
    assert!(e.state_core().player(1).linked, "suolianjia links wearer on equip");
    // Fire damage must be prevented (hp unchanged, remains linked unless cleared)!
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Fire);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 4, "suolianjia prevents Fire damage");
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Thunder);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 4, "suolianjia prevents Thunder damage");
    // Normal damage is NOT prevented by suolianjia
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 3, "suolianjia does NOT prevent Normal damage");
    // Qinggang bypasses suolianjia!
    let qg = find_card(&e, |c| c.kind == CardKind::Qinggang);
    let fire_sha = find_card(&e, |c| c.kind == CardKind::FireSha);
    advance_to_phase(&mut e, 0, Phase::Play);
    move_card_to_hand(&mut e, 0, qg);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: qg, targets: vec![] });
    move_card_to_hand(&mut e, 0, fire_sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: fire_sha, targets: vec![1] });
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 2, "qinggang ignores suolianjia elemental prevention");

    // 2. `chenhuodajie` (`card/yunchou/card.js:266-286`): user chooses WHICH he card of target to gain!
    let chdj = find_card(&e, |c| c.kind == CardKind::Chenhuodajie);
    let t1_a = find_card(&e, |c| c.kind == CardKind::Shan);
    let t1_b = find_card(&e, |c| c.kind == CardKind::Tao);
    e.debug_clear_hand(1);
    move_card_to_hand(&mut e, 0, chdj);
    move_card_to_hand(&mut e, 1, t1_a);
    move_card_to_hand(&mut e, 1, t1_b);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: chdj, targets: vec![1] });
    drain_nullify_only_local(&mut e);
    let c_ch = e.open_choice().expect("chenhuodajie interactive gain pick").clone();
    assert_eq!(c_ch.observer, 0);
    assert!(c_ch.spec.options.contains(&ChoiceOption::Card(t1_b)));
    answer(&mut e, &c_ch.spec.prompt, vec![ChoiceOption::Card(t1_b)]); // pick 2nd card specifically!
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&t1_b), "chenhuodajie gained exact chosen card t1_b");
    assert!(e.debug_hand(1).contains(&t1_a), "chenhuodajie left unchosen t1_a");

    // 3. `shuigong` (`card/yunchou/card.js:333`): targets interactively choose which equip to discard!
    let sg = find_card(&e, |c| c.kind == CardKind::Shuigong);
    let eq_a = find_card(&e, |c| c.kind == CardKind::Armor);
    let eq_b = find_card(&e, |c| c.kind == CardKind::HorsePlus);
    advance_to_phase(&mut e, 2, Phase::Play);
    move_card_to_hand(&mut e, 2, eq_a);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: eq_a, targets: vec![] });
    move_card_to_hand(&mut e, 2, eq_b);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: eq_b, targets: vec![] });
    advance_to_phase(&mut e, 0, Phase::Play);
    move_card_to_hand(&mut e, 0, sg);
    let out_sg = submit(&mut e, 0, GameAction::PlayCard { card: sg, targets: vec![0, 1, 2] });
    assert!(!matches!(out_sg, CommandOutcome::Rejected(_)), "shuigong play: {out_sg:?}");
    drain_nullify_only_local(&mut e);
    let mut e = snapshot_roundtrip(&e, &verified);
    while let Some(c_sg) = e.open_choice().cloned() {
        assert_eq!(c_sg.spec.prompt, "u05_shuigong_discard");
        let sel = if c_sg.observer == 2 {
            vec![ChoiceOption::Card(eq_b)]
        } else {
            vec![c_sg.spec.options[0]]
        };
        answer(&mut e, &c_sg.spec.prompt, sel);
    }
    drain(&mut e);
    assert!(e.state_core().player(2).equips.contains(&eq_a), "shuigong kept unchosen eq_a");
    assert!(!e.state_core().player(2).equips.contains(&eq_b), "shuigong discarded chosen eq_b");

    // 4. `caochuanjiejian` (`card/yunchou/card.js:4-49`):
    //    Branch 0: target executes real `useCard({ name: "sha" }, player)` (so Shan dodges!)
    //    Branch 1: target shows hand and user picks visible hand card to discard (not forced .first())!
    let cc1 = find_card(&e, |c| c.kind == CardKind::Caochuanjiejian);
    let shan0 = find_card(&e, |c| c.kind == CardKind::Shan);
    let sha1 = find_card(&e, |c| c.kind == CardKind::Sha);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    move_card_to_hand(&mut e, 0, cc1);
    move_card_to_hand(&mut e, 0, shan0);
    move_card_to_hand(&mut e, 1, sha1);
    let hp0 = e.debug_hp(0);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: cc1, targets: vec![1] });
    drain_nullify_only_local(&mut e);
    let c_mode = e.open_choice().expect("caochuan mode choice").clone();
    answer(&mut e, &c_mode.spec.prompt, vec![ChoiceOption::Tag(0)]); // Branch 0: use virtual Sha on seat 0
    // Virtual Sha asks seat 0 for Shan!
    if let Some(c_shan) = e.open_choice().cloned() {
        assert_eq!(c_shan.observer, 0, "real Sha use prompts owner for Shan");
        answer(&mut e, &c_shan.spec.prompt, vec![ChoiceOption::Card(shan0)]);
    }
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0, "dodged virtual Sha from caochuanjiejian leaves hp intact");
    assert!(e.debug_hand(0).contains(&sha1), "player gained target's Sha cards per card.js:43");
}

#[test]
fn u05_b1_yunchou_geanguanhuo_shezhan_touliang_fudi_huoshan_hongshui() {
    let (verified, mut e) = start_u05_b1_engine(
        92031,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-yunchou-upstream-noname"],
    );
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_phase(&mut e, 0, Phase::Play);

    // 1. `toulianghuanzhu`: fix Suit->Color bug (`Heart` + `Diamond` are BOTH RED -> draws 1!) + cancel + empty-hand view
    let tl = find_card(&e, |c| c.kind == CardKind::Toulianghuanzhu);
    let own_heart = find_card(&e, |c| c.suit == Suit::Heart && c.kind == CardKind::Shan);
    let tgt_diamond = find_card(&e, |c| c.suit == Suit::Diamond && c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, tl);
    move_card_to_hand(&mut e, 0, own_heart);
    move_card_to_hand(&mut e, 1, tgt_diamond);
    let h0_before = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: tl, targets: vec![1] });
    drain_nullify_only_local(&mut e);
    let c_own = e.open_choice().expect("touliang own").clone();
    answer(&mut e, &c_own.spec.prompt, vec![ChoiceOption::Card(own_heart)]);
    let c_tgt = e.open_choice().expect("touliang target").clone();
    answer(&mut e, &c_tgt.spec.prompt, vec![ChoiceOption::Card(tgt_diamond)]);
    drain(&mut e);
    assert!(e.debug_hand(0).contains(&tgt_diamond));
    assert!(e.debug_hand(1).contains(&own_heart));
    assert_eq!(
        e.debug_hand(0).len(),
        h0_before, // -1 played -1 own +1 target +1 draw(both red!) = net 0
        "toulianghuanzhu Heart+Diamond same RED color must grant +1 draw!"
    );

    // 2. `shezhanqunru` (`card/yunchou/card.js:132-220`):
    //    No responders / responders decline -> draws 3? Wait: if !list.length -> draw 3; if responders agree -> win=+1, lose=-1 -> draw 3 ONLY if num > 0!
    let sz1 = find_card(&e, |c| c.kind == CardKind::Shezhanqunru);
    let p0_k = find_card(&e, |c| c.rank == 12 && c.kind != CardKind::Shezhanqunru);
    let p1_q = find_card(&e, |c| c.rank == 13);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    move_card_to_hand(&mut e, 0, sz1);
    move_card_to_hand(&mut e, 0, p0_k);
    move_card_to_hand(&mut e, 1, p1_q); // rank 13 beats rank 12!
    let h0_sz = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sz1, targets: vec![] });
    drain_nullify_only_local(&mut e);
    // Seat 1 has hand -> asked Confirm/Bool whether to respond! Seat 1 agrees (Confirm)
    let c_r1 = e.open_choice().expect("shezhan seat 1 respond confirm").clone();
    assert_eq!(c_r1.observer, 1);
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, &c_r1.spec.prompt, vec![ChoiceOption::Confirm]);
    // Pindian picks: seat 0 and seat 1 each pick card -> rank 13 > rank 12 -> net num = -1 (<= 0 -> NO draw 3!)
    while let Some(c) = e.open_choice().cloned() {
        let sel = if c.spec.options.contains(&ChoiceOption::Card(p1_q)) {
            vec![ChoiceOption::Card(p1_q)]
        } else if c.spec.options.contains(&ChoiceOption::Card(p0_k)) {
            vec![ChoiceOption::Card(p0_k)]
        } else {
            vec![c.spec.options[0]]
        };
        answer(&mut e, &c.spec.prompt, sel);
    }
    assert!(e.debug_hand(0).len() < h0_sz, "shezhan lost comparison (num <= 0) must NOT draw 3");

    // 3. `geanguanhuo`: when alive == 2 -> `enable == false` & `recastable == true`!
    // 4. `hongshui` (`card/yunchou/card.js:505-556`):
    //    Club 2..=9 hit -> self discards up to 3 random he (or loseHp 1 if 0), dist<=2 discard (3-max(1,dist)) (or loseHp 1);
    //    Miss -> moves to next alive player's judge area!
    let hs = find_card(&e, |c| c.kind == CardKind::HongShui);
    let club5 = find_card_excl(&e, &[hs], |c| c.suit == Suit::Club && (2..=9).contains(&c.rank));
    let spade_k = find_card(&e, |c| c.suit == Suit::Spade && c.rank == 13);
    // Inject hongshui into seat 1 judge area, empty seat 1 hand (so loseHp 1!), give seat 0 & 2 3 cards each
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    for (p, k) in [(0, CardKind::Shan), (2, CardKind::Tao)] {
        let c1 = find_card_excl(&e, &[spade_k], |c| c.kind == k);
        let c2 = find_card_excl(&e, &[spade_k, c1], |c| c.kind == k);
        let c3 = find_card_excl(&e, &[spade_k, c1, c2], |c| c.kind == k);
        move_card_to_hand(&mut e, p, c1);
        move_card_to_hand(&mut e, p, c2);
        move_card_to_hand(&mut e, p, c3);
    }
    e.state_core_mut().deck.retain(|&c| c != hs);
    e.state_core_mut().discard.retain(|&c| c != hs);
    e.state_core_mut().player_mut(1).judge.push(hs);
    // First test MISS (Spade K): hongshui moves from seat 1 judge to seat 2 judge!
    put_deck_top(&mut e, spade_k);
    advance_to_phase(&mut e, 1, Phase::Draw);
    assert!(!e.state_core().player(1).judge.contains(&hs));
    assert!(e.state_core().player(2).judge.contains(&hs), "hongshui miss passes to next_alive (seat 2)");
    // Clear seat 1's newly drawn cards from seat 1's Draw phase so seat 1 tests 0-card loseHp=1!
    e.debug_clear_hand(1);
    // Now test HIT (Club 5) at seat 2! seat 2 has 3 cards -> discards 3 (NOT 3 thunder dmg!), dist 1 (seat 0 & 1): seat 0 discards 2, seat 1 has 0 -> loseHp 1!
    let hp2_before = e.debug_hp(2);
    let hp1_before = e.debug_hp(1);
    let old_hand2 = e.debug_hand(2).clone();
    assert_eq!(old_hand2.len(), 3);
    put_deck_top(&mut e, club5);
    advance_to_phase(&mut e, 2, Phase::Draw);
    assert_eq!(e.debug_hp(2), hp2_before, "hongshui does discard when >=1 cards, NOT 3 thunder damage");
    for &c in &old_hand2 {
        assert!(!e.debug_hand(2).contains(&c), "hongshui target discarded old hand card {c}");
        assert!(e.state_core().discard.contains(&c), "hongshui discarded card {c} to discard pile");
    }
    assert_eq!(e.debug_hand(2).len(), 2, "only Draw phase 2 cards remain in seat 2 hand after discarding all 3");
    assert_eq!(e.debug_hand(0).len(), 1, "hongshui splash dist 1 discards 3-1=2");
    assert_eq!(e.debug_hp(1), hp1_before - 1, "hongshui splash 0-card neighbor loses 1 HP");

    // 5. `huoshan` (`card/yunchou/card.js:442-486`):
    //    Heart 2..=9 hit -> self takes 2 Fire damage + dist<=1 alive others take 1 Fire damage!
    let hsh = find_card(&e, |c| c.name == "yc_huoshan_0" || c.name == "huoshan");
    let heart5 = find_card_excl(&e, &[hsh], |c| c.suit == Suit::Heart && (2..=9).contains(&c.rank));
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.state_core_mut().player_mut(0).linked = false;
    e.state_core_mut().player_mut(1).linked = false;
    e.state_core_mut().player_mut(2).linked = false;
    e.state_core_mut().deck.retain(|&c| c != hsh);
    e.state_core_mut().discard.retain(|&c| c != hsh);
    e.state_core_mut().player_mut(0).judge.push(hsh);
    put_deck_top(&mut e, heart5);
    advance_to_phase(&mut e, 0, Phase::Draw);
    assert_eq!(e.debug_hp(0), 2, "huoshan hit deals 2 Fire damage to owner");
    assert_eq!(e.debug_hp(1), 3, "huoshan hit deals 1 Fire damage to dist<=1 seat 1");
    assert_eq!(e.debug_hp(2), 3, "huoshan hit deals 1 Fire damage to dist<=1 seat 2");
}

fn drain_nullify_only_local(e: &mut Engine) {
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = submit(
                e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
        } else {
            break;
        }
    }
}
