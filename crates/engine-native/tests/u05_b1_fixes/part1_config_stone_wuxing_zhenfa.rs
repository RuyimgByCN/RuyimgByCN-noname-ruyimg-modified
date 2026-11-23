use super::*;
use content_native_loader::{resolved_shahai_subpackages_upstream, ShahaiWuxingCardRate};

#[test]
fn u05_b1_main_config_dependencies_wuxing_rate_and_stone_chenshui() {
    // 1. `main/precontent.js:28-75` dependency graph:
    //    - `gujian` requires `gwent`
    //    - `swd`, `hearth`, `ow`, `xianjian` require `gwent && yunchou`
    //    - `yunchou`, `mtg`, `zhenfa`, `wuxing` are standalone
    let mut cfg = ShahaiMainConfig::default();
    let all = resolved_shahai_subpackages_upstream(&cfg);
    assert_eq!(all.len(), 9);

    cfg.yunchou = false;
    let no_yc = resolved_shahai_subpackages_upstream(&cfg);
    assert!(no_yc.contains(&"shahai-gwent"));
    assert!(no_yc.contains(&"shahai-gujian"));
    assert!(!no_yc.contains(&"shahai-yunchou"));
    assert!(!no_yc.contains(&"shahai-swd"));
    assert!(!no_yc.contains(&"shahai-hearth"));
    assert!(!no_yc.contains(&"shaai-ow"));
    assert!(!no_yc.contains(&"shaai-xianjian"));

    cfg.yunchou = true;
    cfg.gwent = false;
    let no_gw = resolved_shahai_subpackages_upstream(&cfg);
    assert!(no_gw.contains(&"shahai-yunchou"));
    assert!(no_gw.contains(&"shahai-mtg"));
    assert!(no_gw.contains(&"shahai-zhenfa"));
    assert!(!no_gw.contains(&"shahai-gwent"));
    assert!(!no_gw.contains(&"shahai-gujian"));
    assert!(!no_gw.contains(&"shahai-swd"));
    assert!(!no_gw.contains(&"shahai-hearth"));

    cfg.wuxing = true;
    cfg.rand = ShahaiWuxingCardRate::Rate50;
    let with_wx = resolved_shahai_subpackages_upstream(&cfg);
    assert!(with_wx.contains(&"wuxing"));

    // 2. Runtime `wuxing` & `rand` rate (`play/wuxing/element.js:31`)
    let mut cfg_off = ShahaiMainConfig::default();
    cfg_off.wuxing = false;
    let (_, e_off) = start_u05_b1_engine(
        92001,
        3,
        LockedStartMode::Prototype,
        &cfg_off,
        &["wuxing-upstream-liuli", "shahai-zhenfa-upstream-noname"],
    );
    assert!(
        e_off.state_core().card_natures.is_empty(),
        "wuxing=false must suppress card_natures initialization"
    );
    assert!(
        e_off.state_core().players.iter().all(|p| p.wunature.is_none()),
        "wuxing=false must suppress player wunature initialization"
    );

    let mut cfg_10 = ShahaiMainConfig::default();
    cfg_10.wuxing = true;
    cfg_10.rand = ShahaiWuxingCardRate::Rate10;
    let (_, e_10) = start_u05_b1_engine(
        92002,
        3,
        LockedStartMode::Prototype,
        &cfg_10,
        &["wuxing-upstream-liuli"],
    );
    let mut cfg_50 = cfg_10.clone();
    cfg_50.rand = ShahaiWuxingCardRate::Rate50;
    let (_, e_50) = start_u05_b1_engine(
        92002,
        3,
        LockedStartMode::Prototype,
        &cfg_50,
        &["wuxing-upstream-liuli"],
    );
    assert!(
        e_50.state_core().players.iter().all(|p| p.wunature.is_some()),
        "wuxing=true assigns player wunature"
    );
    assert!(
        e_50.state_core().card_natures.len() > e_10.state_core().card_natures.len(),
        "rand=0.5 ({}) should assign more card natures than rand=0.1 ({})",
        e_50.state_core().card_natures.len(),
        e_10.state_core().card_natures.len()
    );

    // 3. `main/content.js:384-410` `stone_yisela` static metadata & `stone_chenshui` Phase::End trigger
    let (verified, mut e) = start_u05_b1_engine(
        92003,
        3,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-hearth-upstream-noname"],
    );
    let yisela_idx = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.0 == "stone_yisela"))
        .expect("stone_yisela in catalog");
    let meta = e.catalog().general_meta(yisela_idx).unwrap();
    assert_eq!(meta.1, Faction::Qun);
    assert_eq!(meta.3, 4);
    assert_eq!(meta.4, Gender::Female);
    assert!(meta.2.contains(&"stone_chenshui"));
    assert_eq!(
        e.catalog().skills[e.catalog().skill_by_name("stone_chenshui").unwrap() as usize]
            .behavior
            .package_skill_id(),
        None,
        "stone_chenshui must be bound as HostBehavior"
    );

    let tracked_before = total_tracked_cards(&e);
    e.debug_grant(0, "stone_chenshui");
    e.debug_clear_hand(0);
    // Target stone leader can be seat 1 via `u05_stone_leader:1` or default self (0).
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "u05_stone_leader".to_string()), 2); // 1-based leader = seat 1
    e.debug_clear_hand(1);
    advance_to_phase(&mut e, 0, Phase::Play);
    // Advance into Phase::End (结束阶段): `stone_chenshui` grants a `hsmengjing_*` card to leader seat 1!
    let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
    drain(&mut e);
    if e.turn().current == 0 && e.turn().phase == Phase::Discard {
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Discard -> End
        drain(&mut e);
    }
    assert_eq!(e.debug_hand(1).len(), 1, "stone_chenshui must grant 1 dream card to leader");
    let granted_name = e.catalog().cards[e.debug_hand(1)[0] as usize].name;
    assert!(
        matches!(
            granted_name,
            "hsmengjing_feicuiyoulong"
                | "hsmengjing_huanxiaojiemei"
                | "hsmengjing_suxing"
                | "hsmengjing_mengye"
                | "hsmengjing_mengjing"
        ) || granted_name.contains("hsmengjing_"),
        "stone_chenshui must grant one of the 5 dream cards, got {granted_name}"
    );
    assert_eq!(total_tracked_cards(&e), tracked_before, "card conservation");
    let _ = snapshot_roundtrip(&e, &verified);
}

#[test]
fn u05_b1_zhenfa_all_11_cards_and_recastable_parity() {
    // Upstream `card/zhenfa/card.js:4-368`:
    // Verify fengyangzhen, yunchuizhen, shepanzhen, longfeizhen, huyizhen, recastable on all 11!
    let (verified, mut e) = start_u05_b1_engine(
        92010,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["shahai-zhenfa-upstream-noname", "wuxing-upstream-liuli"],
    );
    let wei = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wei))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let shu = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Shu))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    let wu = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).is_some_and(|m| m.1 == Faction::Wu))
        .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
        .unwrap();
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    advance_to_phase(&mut e, 0, Phase::Play);
    // Seats: 0=Wei, 1=Shu, 2=Wei, 3=Wu ->
    // Seat 1 (Shu) is flanked by Wei (0 and 2), so 0 and 2 siege 1!
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &shu);
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &wu);

    // A. `fengyangzhen` (`card/zhenfa/card.js:123-143`): sieged() targets gain temp `feiying` until damageAfter
    let fy = find_card(&e, |c| c.name == "fengyangzhen");
    move_card_to_hand(&mut e, 0, fy);
    let dist_before = e.state_core().computed_distance(e.catalog(), 0, 1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: fy, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "fengyangzhen play: {out:?}");
    drain(&mut e);
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        dist_before + 1,
        "fengyangzhen grants +1 defensive distance (`feiying`) to sieged seat 1"
    );
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        dist_before,
        "fengyangzhen temp `feiying` must expire on target damageAfter"
    );

    // B. `yunchuizhen` (`card/zhenfa/card.js:144-164`): siege() players gain temp `wushuang` until source:damageAfter
    let yc = find_card(&e, |c| c.name == "yunchuizhen");
    move_card_to_hand(&mut e, 0, yc);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: yc, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "yunchuizhen play: {out:?}");
    drain(&mut e);
    let sha = find_card(&e, |c| c.kind == CardKind::Sha);
    assert!(
        engine_native::rvm_bridge::modifiers::eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            1,
            sha,
            CardKind::Sha,
            DamageNature::Normal,
        ) || flag(&e, 0, "u05_yunchuizhen_wushuang") > 0,
        "yunchuizhen grants `wushuang` to siege players (0 & 2)"
    );
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(
        flag(&e, 0, "u05_yunchuizhen_wushuang"),
        0,
        "yunchuizhen temp `wushuang` must clear on source damageAfter"
    );

    // C. `huyizhen` (`card/zhenfa/card.js:269-319`): seat 0 flanks seat 1 with seat 2;
    //    seat 0 & seat 2 each interactively pick 1 non-basic he card to use as virtual Sha vs seat 1!
    e.debug_set_hp(1, 4);
    let hy = find_card(&e, |c| c.name == "huyizhen");
    let armor0 = find_card(&e, |c| c.kind == CardKind::Armor);
    let armor2 = find_card(&e, |c| c.kind == CardKind::Weapon);
    let sha_unused = find_card(&e, |c| c.kind == CardKind::Sha);
    move_card_to_hand(&mut e, 0, hy);
    move_card_to_hand(&mut e, 0, armor0);
    move_card_to_hand(&mut e, 0, sha_unused);
    move_card_to_hand(&mut e, 2, armor2);
    let hp1_before = e.debug_hp(1);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: hy, targets: vec![1, 3] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "huyizhen play: {out:?}");
    drain_nullify_only(&mut e);
    let mut e = snapshot_roundtrip(&e, &verified);
    // Only non-basic (`armor0`) should be selectable (not `sha_unused`!).
    let c0 = e.open_choice().expect("huyizhen player 0 card pick").clone();
    assert_eq!(c0.observer, 0);
    assert!(!c0.spec.options.contains(&ChoiceOption::Card(sha_unused)), "huyizhen excludes basic cards");
    answer(&mut e, c0.spec.prompt, vec![ChoiceOption::Card(armor0)]);
    while let Some(next_c) = e.open_choice().cloned() {
        if next_c.spec.prompt == "u05_zhenfa_huyizhen_card" {
            break;
        }
        let sel = if next_c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if next_c.spec.allow_cancel {
            Vec::new()
        } else {
            next_c.spec.options.iter().take(next_c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            &mut e,
            next_c.observer,
            GameAction::Answer {
                choice_id: next_c.id,
                selection: sel,
            },
        );
    }
    let c2 = e.open_choice().expect("huyizhen player 2 card pick").clone();
    assert_eq!(c2.observer, 2);
    answer(&mut e, c2.spec.prompt, vec![ChoiceOption::Card(armor2)]);
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1_before - 2, "both flanking allies dealt virtual Sha via huyizhen");

    // D. `longfeizhen` (`card/zhenfa/card.js:237-260`): from seat 1 (sieged by 0 & 2, next=2 sieges 1),
    //    player 1 picks which card of each adjacent `next`/`previous` (`0`, `2`) to discard + draws 1 on last!
    advance_to_phase(&mut e, 1, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let lf = find_card(&e, |c| c.name == "longfeizhen");
    let t0_c = find_card(&e, |c| c.kind == CardKind::Shan);
    let t2_c = find_card(&e, |c| c.kind == CardKind::Tao);
    move_card_to_hand(&mut e, 1, lf);
    move_card_to_hand(&mut e, 0, t0_c);
    move_card_to_hand(&mut e, 2, t2_c);
    let h1_before = e.debug_hand(1).len();
    let out = submit(&mut e, 1, GameAction::PlayCard { card: lf, targets: vec![2, 0] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "longfeizhen play: {out:?}");
    drain_nullify_only(&mut e);
    let c_pick1 = e.open_choice().expect("longfeizhen target 1 discard pick").clone();
    assert_eq!(c_pick1.observer, 1);
    answer(&mut e, c_pick1.spec.prompt, vec![c_pick1.spec.options[0]]);
    let c_pick2 = e.open_choice().expect("longfeizhen target 2 discard pick").clone();
    assert_eq!(c_pick2.observer, 1);
    answer(&mut e, c_pick2.spec.prompt, vec![c_pick2.spec.options[0]]);
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), 0);
    assert_eq!(e.debug_hand(2).len(), 0);
    assert_eq!(e.debug_hand(1).len(), h1_before, "-1 played +1 drawn = net 0");

    // E. `shepanzhen` (`card/zhenfa/card.js:204-236`): seats 0(Wei), 1(Shu), 2(Wei), 3(Wu) -> 0 & 2 not inline!
    //    Playing `shepanzhen` swaps Wei seats so 0 & 2 become adjacent (inline)!
    advance_to_phase(&mut e, 0, Phase::Play);
    assert!(!e.state_core().inline(e.catalog(), 0), "Wei seats 0 & 2 initially separated");
    let sp = find_card(&e, |c| c.name == "shepanzhen");
    move_card_to_hand(&mut e, 0, sp);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sp, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "shepanzhen play: {out:?}");
    drain(&mut e);
    assert!(e.state_core().inline(e.catalog(), 0), "shepanzhen makes Wei seats contiguous inline");

    // F. `recastable: true` on ALL 11 zhenfa cards when enable==false (e.g., `shepanzhen` now inline -> enable false!)
    let pz = find_card(&e, |c| c.name == "pozhenjue");
    // Ensure a deck card exists for recast draw
    let filler = find_card(&e, |c| c.kind == CardKind::Jiu);
    put_deck_top(&mut e, filler);
    // Put shepanzhen back from discard if needed, since all 11 zhenfa cards have single entities
    move_card_to_hand(&mut e, 0, sp);
    let h_before_recast = e.debug_hand(0).len();
    // Since Wei is already inline, `shepanzhen.enable == false`, so `PlayCard { targets: vec![] }` recasts!
    let out = submit(&mut e, 0, GameAction::PlayCard { card: sp, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "shepanzhen recast when !enable: {out:?}");
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), h_before_recast, "recast -1 +1 = net 0");
    assert!(e.state_core().discard.contains(&sp), "recasted zhenfa went to discard");

    // Also verify active recast skill `u05_zhenfa_recast` works even when `pozhenjue.enable == true`!
    move_card_to_hand(&mut e, 0, pz);
    let out = activate(&mut e, 0, "u05_zhenfa_recast");
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "u05_zhenfa_recast activate: {out:?}");
    let c_rec = e.open_choice().expect("u05_zhenfa_recast pick").clone();
    assert!(c_rec.spec.options.contains(&ChoiceOption::Card(pz)));
    let ring_before = e.state_core().seat_ring.clone();
    answer(&mut e, c_rec.spec.prompt, vec![ChoiceOption::Card(pz)]);
    drain(&mut e);
    assert_eq!(e.state_core().seat_ring, ring_before, "recasting pozhenjue does NOT shuffle seats");
    assert!(e.state_core().discard.contains(&pz), "recasted pozhenjue discarded");
}

#[test]
fn u05_b1_play_wuxing_and_remaining_zhenfa_recast_parity() {
    // 1. `play/wuxing` (`play/wuxing/card.js`, `play/wuxing/extension.js`):
    //    `wuxingpan` + `wuxingpan_skill` (`wuxing_select_card`, `wuxing_select_nature`),
    //    `_shengke` (`KE`: `shengke_discard` forced he discard; `SHENG`: draw 1),
    //    executed via `start_u05_b1_engine` with `wuxing = true`.
    let (verified, mut e) = start_u05_b1_engine(
        92011,
        4,
        LockedStartMode::Prototype,
        &ShahaiMainConfig::default(),
        &["wuxing-upstream-liuli", "shahai-zhenfa-upstream-noname"],
    );
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    let pan = find_card(&e, |c| c.kind == CardKind::WuxingPan);
    let sha_ke = find_card(&e, |c| c.kind == CardKind::Sha);
    let sha_sheng = find_card_excl(&e, &[sha_ke], |c| c.kind == CardKind::Sha);
    let keep1 = find_card(&e, |c| c.kind == CardKind::Tao);
    let top_draw = find_card(&e, |c| c.kind == CardKind::Shan);
    move_card_to_hand(&mut e, 0, pan);
    move_card_to_hand(&mut e, 0, sha_ke);
    move_card_to_hand(&mut e, 0, sha_sheng);
    move_card_to_hand(&mut e, 1, keep1);
    put_deck_top(&mut e, top_draw);
    e.state_core_mut().card_natures.remove(&sha_ke);
    e.state_core_mut().card_natures.remove(&sha_sheng);
    e.state_core_mut().players[1].wunature = Some(1); // 木

    let out_pan = submit(&mut e, 0, GameAction::PlayCard { card: pan, targets: vec![] });
    assert!(!matches!(out_pan, CommandOutcome::Rejected(_)), "equip wuxingpan: {out_pan:?}");
    drain(&mut e);
    assert!(e.debug_has_skill(0, "wuxingpan_skill"), "equipping wuxingpan grants wuxingpan_skill");

    let out_sk = activate(&mut e, 0, "wuxingpan_skill");
    assert!(!matches!(out_sk, CommandOutcome::Rejected(_)), "activate wuxingpan_skill: {out_sk:?}");
    answer(&mut e, "wuxing_select_card", vec![ChoiceOption::Card(sha_ke)]);
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "wuxing_select_nature", vec![ChoiceOption::Tag(0)]); // 金 (KE[0]=1 木, SHENG[0]=2 水)
    assert_eq!(e.state_core().card_natures.get(&sha_ke), Some(&0));
    assert!(
        matches!(activate(&mut e, 0, "wuxingpan_skill"), CommandOutcome::Rejected(_)),
        "wuxingpan_skill is once per play phase"
    );

    // `_shengke` 克 (金0 克 木1 -> forced `shengke_discard` on target 1)
    let out_ke = submit(&mut e, 0, GameAction::PlayCard { card: sha_ke, targets: vec![1] });
    assert!(!matches!(out_ke, CommandOutcome::Rejected(_)), "play sha_ke: {out_ke:?}");
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "shengke_discard", vec![ChoiceOption::Card(keep1)]);
    drain(&mut e);
    assert!(!e.debug_hand(1).contains(&keep1), "KE branch discarded seat 1 card");

    // `_shengke` 生 (金0 生 水2 -> seat 1 draws 1)
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.state_core_mut().players[1].wunature = Some(2); // 水
    e.state_core_mut().card_natures.insert(sha_sheng, 0);
    e.debug_clear_hand(1);
    put_deck_top(&mut e, top_draw);
    let out_sheng = submit(&mut e, 0, GameAction::PlayCard { card: sha_sheng, targets: vec![1] });
    assert!(!matches!(out_sheng, CommandOutcome::Rejected(_)), "play sha_sheng: {out_sheng:?}");
    drain(&mut e);
    assert!(e.debug_hand(1).contains(&top_draw), "SHENG branch draws 1 card");

    // 2. Remaining 6 zhenfa cards (`card/zhenfa/card.js`):
    //    `pozhenjue`, `changshezhen`, `dizaizhen`, `tianfuzhen`, `qixingzhen`, `niaoxiangzhen`
    //    verify `!enable -> recast` AND `enable -> play` parity with snapshot_roundtrip.
    let pick_no_va = |f: Faction| -> String {
        (0..e.catalog().general_count() as u16)
            .find(|&g| {
                e.catalog().general_meta(g).is_some_and(|m| {
                    m.1 == f
                        && !m
                            .2
                            .iter()
                            .any(|s| matches!(*s, "武圣" | "国色" | "奇袭" | "龙胆"))
                })
            })
            .map(|g| e.catalog().general_meta(g).unwrap().0.to_string())
            .unwrap()
    };
    let wei = pick_no_va(Faction::Wei);
    let shu = pick_no_va(Faction::Shu);
    let wu = pick_no_va(Faction::Wu);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let keep_defs: Vec<u16> = ["wuxing_shengke", "wuxingpan_skill", "u05_zhenfa_recast"]
        .iter()
        .filter_map(|n| e.catalog().skill_by_name(n))
        .collect();
    e.state_core_mut()
        .skills
        .retain(|(_, d), _| keep_defs.contains(d));
    // Config A: 0=Wei, 1=Shu, 2=Wei, 3=Wu (0 & 2 siege 1; Wei is major faction; 3 Wu is isolated)
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &shu);
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &wu);

    let pz = find_card(&e, |c| c.kind == CardKind::Pozhenjue);
    let cs = find_card(&e, |c| c.kind == CardKind::Changshezhen);
    let dz = find_card(&e, |c| c.kind == CardKind::Dizaizhen);
    let tf = find_card(&e, |c| c.kind == CardKind::Tianfuzhen);
    let qx = find_card(&e, |c| c.kind == CardKind::Qixingzhen);
    let nx = find_card(&e, |c| c.kind == CardKind::Niaoxiangzhen);

    // pozhenjue (`enable: true`, also recastable via `u05_zhenfa_recast`)
    move_card_to_hand(&mut e, 0, pz);
    let out_pz = submit(&mut e, 0, GameAction::PlayCard { card: pz, targets: vec![] });
    assert!(!matches!(out_pz, CommandOutcome::Rejected(_)), "pozhenjue play: {out_pz:?}");
    drain(&mut e);
    assert!(e.state_core().discard.contains(&pz));

    // Restore canonical ring `[0, 1, 2, 3]` after `pozhenjue` shuffle
    e.state_core_mut().seat_ring = vec![0, 1, 2, 3];
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &shu);
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &wu);

    // qixingzhen (`enable == true` when siege/sieged: seat 0 sieges 1)
    let filler1 = find_card(&e, |c| c.kind == CardKind::Jiu);
    move_card_to_hand(&mut e, 0, qx);
    move_card_to_hand(&mut e, 1, filler1);
    let out_qx = submit(&mut e, 0, GameAction::PlayCard { card: qx, targets: vec![0] });
    assert!(!matches!(out_qx, CommandOutcome::Rejected(_)), "qixingzhen play: {out_qx:?}");
    let mut e = snapshot_roundtrip(&e, &verified);
    drain(&mut e);
    assert!(e.state_core().discard.contains(&qx));

    // changshezhen (`enable == true` for Wei seat 0 because Wei seat 2 exists -> swaps inline)
    move_card_to_hand(&mut e, 0, cs);
    let out_cs = submit(&mut e, 0, GameAction::PlayCard { card: cs, targets: vec![] });
    assert!(!matches!(out_cs, CommandOutcome::Rejected(_)), "changshezhen play: {out_cs:?}");
    drain(&mut e);
    assert!(e.state_core().inline(e.catalog(), 0), "changshezhen swaps Wei seats inline");

    // Re-map seats in ring: now Config B: 0=Wei, 1=Wei, 2=Shu, 3=Wu (0/1 inline Wei; nobody sieges 3; Shu/Wu non-major)
    e.state_core_mut().seat_ring = vec![0, 1, 2, 3];
    e.debug_assemble_general(0, &wei);
    e.debug_assemble_general(1, &wei);
    e.debug_assemble_general(2, &shu);
    e.debug_assemble_general(3, &wu);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // tianfuzhen (`enable == true`: Wei is major, seats 0 & 1 have cards) -> forced `swd_tianfuzhen_discard`
    let tf_self = find_card(&e, |c| c.kind == CardKind::Sha);
    let tf_fodder = find_card(&e, |c| c.kind == CardKind::Shan);
    move_card_to_hand(&mut e, 0, tf_self);
    move_card_to_hand(&mut e, 1, tf_fodder);
    move_card_to_hand(&mut e, 0, tf);
    let out_tf = submit(&mut e, 0, GameAction::PlayCard { card: tf, targets: vec![0, 1] });
    assert!(!matches!(out_tf, CommandOutcome::Rejected(_)), "tianfuzhen play: {out_tf:?}");
    drain_nullify_only(&mut e);
    let mut e = snapshot_roundtrip(&e, &verified);
    answer(&mut e, "swd_tianfuzhen_discard", vec![ChoiceOption::Card(tf_self)]);
    answer(&mut e, "swd_tianfuzhen_discard", vec![ChoiceOption::Card(tf_fodder)]);
    drain(&mut e);
    assert!(!e.debug_hand(0).contains(&tf_self), "tianfuzhen forced discard seat 0");
    assert!(!e.debug_hand(1).contains(&tf_fodder), "tianfuzhen forced discard seat 1");

    // niaoxiangzhen from seat 2 (Shu): adjacent Wei seats 0 & 1 are valid targets -> respond Shan or 1 dmg
    advance_to_phase(&mut e, 2, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    move_card_to_hand(&mut e, 2, nx);
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);
    let out_nx = submit(&mut e, 2, GameAction::PlayCard { card: nx, targets: vec![0, 1] });
    assert!(!matches!(out_nx, CommandOutcome::Rejected(_)), "niaoxiangzhen play: {out_nx:?}");
    drain(&mut e);
    assert_eq!((e.debug_hp(0), e.debug_hp(1)), (hp0 - 1, hp1 - 1));

    // `!enable -> recast` on `changshezhen`, `qixingzhen`, `tianfuzhen`, `dizaizhen`
    // Seat 3 (Wu, isolated): `changshezhen.enable == false` -> recast!
    advance_to_phase(&mut e, 3, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let rec_draw = find_card(&e, |c| c.kind == CardKind::Tao);
    put_deck_top(&mut e, rec_draw);
    move_card_to_hand(&mut e, 3, cs);
    let out_cs_rec = submit(&mut e, 3, GameAction::PlayCard { card: cs, targets: vec![] });
    assert!(!matches!(out_cs_rec, CommandOutcome::Rejected(_)), "changshezhen !enable recast: {out_cs_rec:?}");
    drain(&mut e);
    assert!(e.debug_hand(3).contains(&rec_draw), "changshezhen recast drew replacement");

    // Seat 3 (`0=Wei,1=Wei,2=Shu,3=Wu`) is not siege/sieged -> `qixingzhen.enable == false` -> recast!
    let rec_draw2 = find_card_excl(&e, &[rec_draw], |c| c.kind == CardKind::Tao);
    put_deck_top(&mut e, rec_draw2);
    move_card_to_hand(&mut e, 3, qx);
    let out_qx_rec = submit(&mut e, 3, GameAction::PlayCard { card: qx, targets: vec![] });
    assert!(!matches!(out_qx_rec, CommandOutcome::Rejected(_)), "qixingzhen !enable recast: {out_qx_rec:?}");
    drain(&mut e);
    assert!(e.debug_hand(3).contains(&rec_draw2), "qixingzhen recast drew replacement");

    // Config C: all 4 Wei (`0..=3` all Wei) -> NO non-major -> `dizaizhen.enable == false` -> recast!
    // Also when all 4 unrevealed (no revealed faction), `tianfuzhen.enable == false` -> recast!
    e.debug_assemble_general(2, &wei);
    e.debug_assemble_general(3, &wei);
    let rec_draw3 = find_card(&e, |c| c.kind == CardKind::Shan);
    put_deck_top(&mut e, rec_draw3);
    move_card_to_hand(&mut e, 3, dz);
    let out_dz_rec = submit(&mut e, 3, GameAction::PlayCard { card: dz, targets: vec![] });
    assert!(!matches!(out_dz_rec, CommandOutcome::Rejected(_)), "dizaizhen !enable recast: {out_dz_rec:?}");
    drain(&mut e);
    assert!(e.debug_hand(3).contains(&rec_draw3), "dizaizhen recast drew replacement");

    // Clear major faction (hide all generals -> no major faction -> tianfuzhen !enable -> recast!)
    for p in 0..4 {
        e.state_core_mut().player_mut(p).general_slots = [None, None];
    }
    let rec_draw4 = find_card_excl(&e, &[rec_draw3], |c| c.kind == CardKind::Shan);
    put_deck_top(&mut e, rec_draw4);
    move_card_to_hand(&mut e, 3, tf);
    let out_tf_rec = submit(&mut e, 3, GameAction::PlayCard { card: tf, targets: vec![] });
    assert!(!matches!(out_tf_rec, CommandOutcome::Rejected(_)), "tianfuzhen !enable recast: {out_tf_rec:?}");
    drain(&mut e);
    assert!(e.debug_hand(3).contains(&rec_draw4), "tianfuzhen recast drew replacement");
}

fn drain_nullify_only(e: &mut Engine) {
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
