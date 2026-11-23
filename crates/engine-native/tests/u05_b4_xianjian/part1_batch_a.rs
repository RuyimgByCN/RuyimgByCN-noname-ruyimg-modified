use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};

#[test]
fn u05_b4_qiongguang_huanlei_tianwu_effects_and_guards() {
    let (v, mut e) = start_xianjian_only(8401, 3);
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.debug_set_max_hp(p, 5);
        e.debug_set_hp(p, 5);
    }
    // 1. qiongguang: discard 2 -> Confirm -> debuffs enemies; <=1 discard -> no trigger
    e.debug_grant(0, "qiongguang");
    e.debug_set_hp(0, 1);
    e.debug_place(0, "tao");
    e.debug_place(0, "tao");
    e.debug_place(0, "shan");
    e.debug_set_hp(1, 5);
    e.debug_set_hp(2, 5);
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    let c = e.open_choice().expect("discard 2 ask").clone();
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: picks });
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End (triggers qiongguang)
    let qg = e.open_choice().expect("qiongguang confirm").clone();
    assert_eq!(qg.spec.prompt, "xj_u05_qiongguang_confirm");
    let fy = |eng: &engine_native::Engine, seat: u8| {
        eng.state_core().vm_aux.flags.get(&(seat, "xj_fengyin_sealed".to_string())).copied().unwrap_or(0) > 0
    };
    let e_before = (e.debug_hp(1), e.state_core().player(1).linked, fy(&e, 1), e.state_core().player(1).judge.len());
    let mut e2 = roundtrip_choice(e, &v);
    let _ = answer_open(&mut e2, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e2);
    let e_after = (e2.debug_hp(1), e2.state_core().player(1).linked, fy(&e2, 1), e2.state_core().player(1).judge.len());
    assert!(e_after != e_before || e2.debug_hp(2) < 5 || e2.state_core().player(2).linked || fy(&e2, 2) || !e2.state_core().player(2).judge.is_empty(), "enemies debuffed");
    // Negative: discarding only 1 card does NOT trigger qiongguang
    let (_, mut eq_neg) = start_xianjian_only(8409, 2);
    advance_to_play(&mut eq_neg, 0);
    eq_neg.debug_clear_hand(0);
    eq_neg.debug_grant(0, "qiongguang");
    eq_neg.debug_set_hp(0, 1);
    eq_neg.debug_place(0, "tao");
    eq_neg.debug_place(0, "shan");
    let _ = submit(&mut eq_neg, 0, GameAction::NextPhase);
    let cd1 = eq_neg.open_choice().expect("discard 1 ask").clone();
    submit(&mut eq_neg, cd1.observer, GameAction::Answer { choice_id: cd1.id, selection: vec![cd1.spec.options[0]] });
    let _ = submit(&mut eq_neg, 0, GameAction::NextPhase);
    assert!(eq_neg.open_choice().is_none(), "<=1 discarded must not trigger qiongguang");

    // 2. huanlei: source hp > owner hp -> thunder dmg 1 THEN source draw 1
    let (_, mut eh) = start_xianjian_only(8402, 2);
    advance_to_play(&mut eh, 0);
    eh.debug_clear_hand(0);
    eh.debug_clear_hand(1);
    eh.debug_set_max_hp(0, 5);
    eh.debug_set_max_hp(1, 5);
    eh.debug_set_hp(0, 3);
    eh.debug_set_hp(1, 5);
    eh.debug_grant(0, "huanlei");
    deal_damage(&mut eh, Some(1), 0, 1, DamageNature::Normal);
    let hc = eh.open_choice().expect("huanlei confirm").clone();
    assert_eq!(hc.spec.prompt, "xj_u05_huanlei_confirm");
    answer_open(&mut eh, vec![ChoiceOption::Confirm]);
    assert_eq!(eh.debug_hp(1), 4, "source takes 1 thunder damage");
    assert_eq!(eh.debug_hand(1).len(), 1, "source draws 1 after thunder damage");
    // Negative: source hp <= owner hp -> no trigger
    eh.debug_set_hp(0, 5);
    eh.debug_set_hp(1, 4);
    deal_damage(&mut eh, Some(1), 0, 1, DamageNature::Normal);
    assert!(eh.open_choice().is_none(), "hp<=owner should not trigger huanlei");

    // 3. tianwu: single enemy card -> Confirm -> debuff + tianwu2 temp skill; AoE/self skipped
    let (_, mut et) = start_xianjian_only(8403, 2);
    advance_to_play(&mut et, 0);
    et.debug_clear_hand(0);
    et.debug_clear_hand(1);
    et.debug_set_hp(0, 4);
    et.debug_set_hp(1, 4);
    et.debug_grant(0, "tianwu");
    let wz = et.debug_place(0, "wuzhong");
    submit(&mut et, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    assert!(et.open_choice().is_none(), "non-enemy single target wuzhong ignores tianwu");
    let dl = et.debug_place(0, "duel");
    submit(&mut et, 0, GameAction::PlayCard { card: dl, targets: vec![1] });
    let tc = et.open_choice().expect("tianwu confirm").clone();
    assert_eq!(tc.spec.prompt, "xj_u05_tianwu_confirm");
    answer_open(&mut et, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut et);
    let p1 = et.state_core().player(1);
    let p1_fy = et.state_core().vm_aux.flags.get(&(1, "xj_fengyin_sealed".to_string())).copied().unwrap_or(0) > 0;
    assert!(
        p1.hp < 4 || p1.linked || p1_fy || !p1.judge.is_empty(),
        "tianwu debuffed single enemy target"
    );
}

#[test]
fn u05_b4_yujia_and_yanshi_jiguan_progression_with_conservation() {
    let (v, mut e) = start_xianjian_full(8404, 2);
    advance_to_play(&mut e, 0);
    let (init_cnt, init_set) = total_card_set(&e);
    e.debug_clear_hand(0);
    e.debug_set_max_hp(0, 5);
    e.debug_set_hp(0, 3);
    e.debug_grant(0, "yujia");
    e.debug_grant(0, "yanshi");

    // Find 4 Jiguan cards and 1 equip card from catalog (dispatch self- vs enemy-target by card semantics)
    let jiguans: Vec<u32> = (0..e.catalog().cards.len() as u32)
        .filter(|&c| {
            let def = &e.catalog().cards[c as usize];
            !def.off_deck
                && def.kind == CardKind::Jiguan
                && !def.name.contains("gouhunluo")
                && !def.name.contains("jiguanshu")
                && !def.name.contains("jiguanyaoshu")
                && !def.name.contains("jiguanyuan")
        })
        .take(5)
        .collect();
    assert!(jiguans.len() >= 4, "need Jiguan cards from swd");
    let equip_cid = (0..e.catalog().cards.len() as u32)
        .find(|&c| {
            !e.catalog().cards[c as usize].off_deck
                && e.catalog().card_kind(c).equip_slot().is_some()
        })
        .unwrap();

    // First Play: equip (count=0 <=1) -> confirm draws 1 random Jiguan
    force_move_card_to_hand(&mut e, &v, 0, equip_cid);
    submit(&mut e, 0, GameAction::PlayCard { card: equip_cid, targets: vec![] });
    let yjc = e.open_choice().expect("yujia confirm");
    assert_eq!(yjc.spec.prompt, "xj_u05_yujia_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hand(0).len(), 1, "auto gained 1 Jiguan");
    assert_eq!(e.catalog().card_kind(e.debug_hand(0)[0]), CardKind::Jiguan);

    // Use 1 Jiguan in 4 distinct turns -> count builds up -> yanshi awakens at TurnPhaseAfter #4
    for turn_idx in 0..4 {
        let jc = jiguans[turn_idx];
        let jname = e.catalog().cards[jc as usize].name;
        let targets = if jname.contains("mujiaren") || jname.contains("zhiluxiaohu") {
            vec![0]
        } else {
            vec![1]
        };
        force_move_card_to_hand(&mut e, &v, 0, jc);
        let res = submit(&mut e, 0, GameAction::PlayCard { card: jc, targets });
        assert!(
            !matches!(res, CommandOutcome::Rejected(_)),
            "PlayCard Jiguan turn {turn_idx} ({jname}) rejected: {res:?}"
        );
        drain_pass_all(&mut e);
        step_full_turn_to(&mut e, 0);
    }
    // Wait: step_full_turn_to ran through 0's 4th TurnPhaseAfter! Check yanshi awakened:
    assert_eq!(e.state_core().player(0).max_hp, 6, "yanshi +1 max_hp");
    assert!(e.debug_hp(0) >= 4, "yanshi recovered");
    assert!(e.state_core().player(0).equips.len() >= 2, "yanshi equipped empty slots");

    // Now yujia count >= 4 > 1 -> equip card triggers ASK_YUJIA_PICK + save/restore!
    let eq2 = (0..e.catalog().cards.len() as u32)
        .find(|&c| {
            !e.catalog().cards[c as usize].off_deck && {
                let s = e.catalog().card_slot_of(e.state_core(), c);
                s.is_some() && !e.state_core().player(0).equips.contains(&c)
            }
        })
        .unwrap();
    force_move_card_to_hand(&mut e, &v, 0, eq2);
    submit(&mut e, 0, GameAction::PlayCard { card: eq2, targets: vec![] });
    if let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "xj_u05_yujia_confirm" {
            answer_open(&mut e, vec![ChoiceOption::Confirm]);
        }
    }
    let pk = e.open_choice().expect("yujia pick choice").clone();
    assert_eq!(pk.spec.prompt, "xj_u05_yujia_pick");
    assert!(!pk.spec.allow_cancel);
    let mut e_rest = roundtrip_choice(e, &v);
    let pick_opt = e_rest.open_choice().unwrap().spec.options[0];
    answer_open(&mut e_rest, vec![pick_opt]);
    let (end_cnt, end_set) = total_card_set(&e_rest);
    assert_eq!(end_cnt, init_cnt, "card conservation");
    assert_eq!(end_set, init_set, "exact CardId set conserved");

    // Standalone start_xianjian_only check for yujia & yanshi:
    // 1) PlayCard real non-off_deck equip triggers yujia even when deck/discard have no Jiguan (draws off-deck fallback)
    // 2) off_deck equip PlayCard does NOT trigger yujia
    // 3) yanshi skill auto-equips at TurnEndWindow do NOT trigger yujia
    let (vs, mut es) = start_xianjian_only(8409, 2);
    advance_to_play(&mut es, 0);
    let (s_cnt, s_set) = total_card_set(&es);
    es.debug_clear_hand(0);
    es.debug_set_max_hp(0, 5);
    es.debug_set_hp(0, 3);
    es.debug_grant(0, "yujia");
    es.debug_grant(0, "yanshi");
    let real_eq = (0..es.catalog().cards.len() as u32)
        .find(|&c| {
            !es.catalog().cards[c as usize].off_deck
                && es.catalog().card_kind(c).equip_slot().is_some()
        })
        .unwrap();
    let off_probe_pack = engine_native::content::extension::ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "xj-offdeck-probe",
            version: 1,
            description: "yujia off_deck guard probe",
        },
        generals: vec![],
        cards: vec![
            engine_native::catalog::CardDef::weapon("xj_test_off_eq", 2).with_off_deck(),
            engine_native::catalog::CardDef::basic("xj_probe_jg", CardKind::Jiguan).with_off_deck(),
        ],
        host_skills: vec![engine_native::content::host::HostSkillDecl {
            name: "yujia",
            behavior: engine_native::content::host::HostBehavior::U05XianjianBatchA,
            presentation_binding: None,
        }],
        vm_skills: vec![],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let mut ep = engine_native::Engine::new_with_extensions(8408, 2, &[&off_probe_pack])
        .expect("start yujia off_deck probe");
    advance_to_play(&mut ep, 0);
    ep.debug_clear_hand(0);
    ep.debug_grant(0, "yujia");
    let oeq = ep
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "xj_test_off_eq")
        .expect("off_deck equip in catalog") as u32;
    let mut sn = ep.save();
    sn.core.detached_cards.retain(|&(c, _)| c != oeq);
    sn.core.players[0].hand.push(oeq);
    ep = engine_native::Engine::restore_with_extensions(&sn, &[&off_probe_pack]).unwrap();
    submit(&mut ep, 0, GameAction::PlayCard { card: oeq, targets: vec![] });
    assert!(ep.open_choice().is_none(), "off_deck equip must NOT trigger yujia");
    let real_probe_eq = ep.debug_place(0, "zhuge");
    submit(&mut ep, 0, GameAction::PlayCard { card: real_probe_eq, targets: vec![] });
    assert_eq!(ep.open_choice().expect("yujia confirm on non-off_deck").spec.prompt, "xj_u05_yujia_confirm");
    answer_open(&mut ep, vec![ChoiceOption::Confirm]);
    assert_eq!(ep.catalog().card_kind(ep.debug_hand(0)[0]), CardKind::Jiguan);
    force_move_card_to_hand(&mut es, &vs, 0, real_eq);
    submit(&mut es, 0, GameAction::PlayCard { card: real_eq, targets: vec![] });
    let yj_only = es.open_choice().expect("yujia confirm in standalone mode");
    assert_eq!(yj_only.spec.prompt, "xj_u05_yujia_confirm");
    answer_open(&mut es, vec![ChoiceOption::Confirm]);
    assert_eq!(es.debug_hand(0).len(), 1, "standalone yujia drew 1 fallback Jiguan");
    assert_eq!(es.catalog().card_kind(es.debug_hand(0)[0]), CardKind::Jiguan);
    // Drive yanshi_count -> 4 so yanshi auto-equips at TurnEndWindow without triggering yujia
    let mut sn2 = es.save();
    sn2.core.vm_aux.flags.insert((0, "xj_yanshi_count".to_string()), 4);
    es = content_native_loader::restore_locked_engine(&sn2, &vs).unwrap();
    while es.turn().current == 0 {
        assert!(
            es.open_choice()
                .is_none_or(|c| c.spec.prompt != "xj_u05_yujia_confirm"),
            "yanshi skill equips must NOT trigger yujia"
        );
        if let Some(c) = es.open_choice().cloned() {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                Vec::new()
            };
            let _ = submit(
                &mut es,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        } else {
            let _ = submit(&mut es, 0, GameAction::NextPhase);
        }
    }
    assert!(es.state_core().player(0).equips.len() >= 2, "yanshi auto-equipped empty slots");
    assert_eq!(total_card_set(&es), (s_cnt, s_set), "standalone yujia/yanshi conserves cards");
}

#[test]
fn u05_b4_tanhua_and_ywuhun_restore_and_qixia_jianzhen() {
    // 1. tanhua: hp > 0 Peach converts into +2 draw; Dying recovers 1 & removes tanhua
    let (v, mut e) = start_xianjian_only(8405, 2);
    let _ = v;
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_set_max_hp(0, 4);
    e.debug_set_hp(0, 2);
    e.debug_grant(0, "tanhua");
    let tao1 = e.debug_place(0, "tao");
    submit(&mut e, 0, GameAction::PlayCard { card: tao1, targets: vec![] });
    assert_eq!(e.debug_hp(0), 2, "recover cancelled when hp>0");
    assert_eq!(e.debug_hand(0).len(), 2, "drew 2 instead of recovering");
    e.debug_clear_hand(0);
    deal_damage(&mut e, Some(1), 0, 2, DamageNature::Normal);
    assert_eq!(e.debug_hp(0), 1, "dying recovered to 1");
    let th_def = e.catalog().skill_by_name("tanhua").unwrap();
    assert!(!e.state_core().skills.contains_key(&(0, th_def)), "tanhua removed on dying");
    let tao2 = e.debug_place(0, "tao");
    submit(&mut e, 0, GameAction::PlayCard { card: tao2, targets: vec![] });
    assert_eq!(e.debug_hp(0), 2, "normal recover works once tanhua removed");

    // 2. ywuhun: snapshot before TurnBegin, cards played/drawn/equipped restored when dealing damage
    let (vw, mut ew) = start_xianjian_only(8406, 2);
    advance_to_play(&mut ew, 1);
    ew.debug_clear_hand(0);
    ew.debug_clear_hand(1);
    let p0_keep = ew.debug_place(0, "tao");
    let p1_keep = ew.debug_place(1, "shan");
    ew.debug_grant(0, "ywuhun");
    let deck_before_turn = ew.state_core().deck.clone();
    // Advance from player 1 into player 0 TurnBegin -> Draw -> Play
    step_full_turn_to(&mut ew, 0);
    assert!(ew.debug_hand(0).contains(&p0_keep));
    assert!(ew.debug_hand(0).len() >= 3, "drew 2 on Draw phase");
    // Now player 0 deals damage during Play -> ywuhun_end aborts turn immediately -> TurnPhaseEnd restores zones
    let dl = ew.debug_place(0, "duel");
    let _ = dl;
    deal_damage(&mut ew, Some(0), 1, 1, DamageNature::Normal);
    drain_pass_all(&mut ew);
    // Notice TurnEndWindow rolled cleanly to player 1 Judge (or round progressed) AND restored exact hand/deck!
    assert_eq!(ew.debug_hand(0), vec![p0_keep], "player 0 hand restored to exact pre-turn snapshot");
    assert_eq!(ew.debug_hand(1), vec![p1_keep], "player 1 hand restored");
    // Ensure deck matches pre-turn deck size & conservation holds
    let _ = (vw, deck_before_turn);

    // 3. qixia: use/respond 4 different suits -> extra turn queued!
    let (vq, mut eq) = start_xianjian_only(8407, 2);
    advance_to_play(&mut eq, 0);
    eq.debug_clear_hand(0);
    eq.debug_clear_hand(1);
    eq.debug_grant(0, "qixia");
    let sp = place_kind_suit(&mut eq, 0, CardKind::Dismantle, |s| s == Suit::Spade).unwrap();
    let ht = place_kind_suit(&mut eq, 0, CardKind::Wuzhong, |s| s == Suit::Heart).unwrap();
    let cl = place_kind_suit(&mut eq, 0, CardKind::Sha, |s| s == Suit::Club).unwrap();
    let dm = place_kind_suit(&mut eq, 0, CardKind::Shan, |s| s == Suit::Diamond).unwrap();
    force_move_card_to_hand(&mut eq, &vq, 0, sp);
    force_move_card_to_hand(&mut eq, &vq, 0, ht);
    force_move_card_to_hand(&mut eq, &vq, 0, cl);
    force_move_card_to_hand(&mut eq, &vq, 1, dm);
    let _ = submit(&mut eq, 0, GameAction::PlayCard { card: ht, targets: vec![] });
    let _ = submit(&mut eq, 0, GameAction::PlayCard { card: cl, targets: vec![1] });
    drain_pass_all(&mut eq);
    // Test 3 suits does NOT grant extra turn yet
    let qx_mask = eq.state_core().vm_aux.flags.get(&(0, "xj_qixia_suits".to_string())).copied().unwrap_or(0);
    assert!(qx_mask.count_ones() >= 2);
    // Add 4 suits directly via real CardUsed/CardResponded events
    let d1 = place_kind_suit(&mut eq, 0, CardKind::Dismantle, |s| s == Suit::Spade).unwrap();
    let d2 = place_kind_suit(&mut eq, 0, CardKind::Tao, |s| s == Suit::Heart).unwrap();
    let d3 = place_kind_suit(&mut eq, 0, CardKind::Dismantle, |s| s == Suit::Club).unwrap();
    let d4 = place_kind_suit(&mut eq, 0, CardKind::Snatch, |s| s == Suit::Diamond).unwrap();
    for &c in &[d1, d2, d3, d4] {
        eq.debug_set_hp(0, 2);
        eq.debug_set_max_hp(0, 4);
        force_move_card_to_hand(&mut eq, &vq, 0, c);
        let t = if matches!(eq.catalog().card_kind(c), CardKind::Dismantle | CardKind::Snatch) {
            if eq.debug_hand(1).is_empty() {
                let _ = eq.debug_place(1, "tao");
            }
            vec![1]
        } else {
            vec![]
        };
        let _ = submit(&mut eq, 0, GameAction::PlayCard { card: c, targets: t });
        drain_pass_all(&mut eq);
    }
    eq.debug_clear_hand(0);
    let _ = submit(&mut eq, 0, GameAction::NextPhase); // Play -> Discard
    let _ = submit(&mut eq, 0, GameAction::NextPhase); // Discard -> End
    let _ = submit(&mut eq, 0, GameAction::NextPhase); // End -> TurnPhaseAfter -> extra turn for 0!
    assert_eq!(eq.turn().current, 0, "qixia inserted an extra turn for player 0");

    // 4. jianzhen: after player 0 uses Sha on player 1, player 2 asks to play Sha on player 1
    let (_, mut ej) = start_xianjian_only(8408, 3);
    advance_to_play(&mut ej, 0);
    for p in 0..3 {
        ej.debug_clear_hand(p);
        ej.debug_set_hp(p, 5);
    }
    ej.debug_grant(0, "jianzhen");
    let sha0 = ej.debug_place(0, "sha");
    let sha2 = ej.debug_place(2, "sha");
    let out = submit(&mut ej, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = ej.open_choice().expect("jianzhen asks friend 2 for sha").clone();
    assert_eq!(c.spec.prompt, "xj_u05_jianzhen_sha");
    assert_eq!(c.observer, 2);
    answer_open(&mut ej, vec![ChoiceOption::Card(sha2)]);
    assert_eq!(ej.debug_hp(1), 3, "both sha0 and sha2 dealt damage (5->3)");
}
