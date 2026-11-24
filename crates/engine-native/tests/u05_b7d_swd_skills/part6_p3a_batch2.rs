//! Phase 3 Track P3-A Batch 2 SWD parity & 8 unblocked generals real-match tests.
//! Zero `debug_execute_effect`; public-info AI checks only.

use super::common::*;
use content_native_loader::LockedStartMode;
use engine_native::ai::L01AiOptions;
use engine_native::content::host::{SWD_LANGEGUAIYI_SKILL_KEY, SWD_YUFULU_SKILL_KEY};
use engine_native::machine::LogEvent;

fn clear_all_hands(e: &mut Engine, seats: u8) {
    for p in 0..seats {
        e.debug_clear_hand(p);
    }
}

fn place_equip_on(
    e: &mut Engine,
    seat: u8,
    kind: CardKind,
) -> u32 {
    let cid = (0..e.catalog().cards.len() as u32)
        .find(|&id| e.catalog().cards[id as usize].kind == kind)
        .expect("equip card in catalog");
    let core = e.state_core_mut();
    core.deck.retain(|&x| x != cid);
    core.discard.retain(|&x| x != cid);
    core.table.retain(|&x| x != cid);
    for p in core.players.iter_mut() {
        p.hand.retain(|&x| x != cid);
        p.equips.retain(|&x| x != cid);
        p.judge.retain(|&x| x != cid);
    }
    core.players[seat as usize].equips.push(cid);
    cid
}

fn find_unassigned_card(
    e: &Engine,
    excl: &[u32],
    pred: impl Fn(&engine_native::catalog::CardDef) -> bool,
) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|&id| !excl.contains(&id) && pred(&e.catalog().cards[id as usize]))
        .expect("card in catalog")
}

fn move_to_hand(e: &mut Engine, seat: u8, cid: u32) {
    let core = e.state_core_mut();
    core.deck.retain(|&x| x != cid);
    core.discard.retain(|&x| x != cid);
    core.table.retain(|&x| x != cid);
    for p in core.players.iter_mut() {
        p.hand.retain(|&x| x != cid);
        p.equips.retain(|&x| x != cid);
        p.judge.retain(|&x| x != cid);
    }
    core.players[seat as usize].hand.push(cid);
}

#[test]
fn u05_b7d_p3a_yiluan_rng_playable_non_multitarget_and_rng_legal_target() {
    let mut sha_targets = std::collections::BTreeSet::new();
    let mut saw_sha = false;
    let mut saw_wuzhong = false;

    for seed in 90_100..90_120u64 {
        let (_v, mut e) = load_formal_swd_only(seed, 4);
        for p in 0..4 {
            e.debug_remove_general(p);
            e.debug_set_hp(p, 4);
        }
        advance_to_play(&mut e, 0);
        clear_all_hands(&mut e, 4);
        let sfm = place_equip_on(&mut e, 0, CardKind::Sifeizhenmian);
        let _ = sfm;
        e.debug_grant(0, "yiluan");
        let shan = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Shan);
        let aoe = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::SavageAttack);
        let sha = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Sha);
        let wuzhong = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Wuzhong);
        set_deck_top_matching(&mut e, |d| !matches!(d.suit, Suit::Heart));

        let yl_def = e.catalog().skill_by_name("yiluan").unwrap();
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: yl_def });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        let ch = e.open_choice().expect("yiluan_target").clone();
        assert_eq!(ch.spec.prompt, "yiluan_target");
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ch.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        settle_confirm(&mut e);

        let h1 = e.debug_hand(1);
        assert!(h1.contains(&shan), "yiluan must NEVER pick Shan");
        assert!(h1.contains(&aoe), "yiluan must NEVER pick multi-target SavageAttack");
        assert_ne!(e.debug_hp(3), 3, "seat 3 out of Sha range from seat 1 must never be targeted");
        if e.debug_hp(0) == 3 {
            sha_targets.insert(0u8);
            saw_sha = true;
            assert!(!h1.contains(&sha));
        } else if e.debug_hp(2) == 3 {
            sha_targets.insert(2u8);
            saw_sha = true;
            assert!(!h1.contains(&sha));
        } else {
            saw_wuzhong = true;
            assert!(!h1.contains(&wuzhong));
            assert_eq!(h1.len(), 3 + 2 - 1 + 1, "Wuzhong draws 2 into seat 1's hand");
        }
    }
    assert!(saw_sha && saw_wuzhong, "RNG should sample both Sha and Wuzhong across seeds");
    assert_eq!(
        sha_targets,
        [0u8, 2].into_iter().collect(),
        "RNG target selection must pick both legal distance-1 targets (0 and 2) across seeds"
    );
}

#[test]
fn u05_b7d_p3a_longfan_club_enemy_only_uniform_he_and_single_discard_log() {
    let (_v, mut e) = load_formal_swd_only_mode(90_201, 4, LockedStartMode::Identity);
    let roles = e.state_core().identity_roles().expect("Identity roles");
    let lord = roles.iter().position(|r| *r == Role::Lord).unwrap() as u8;
    let loyalist = roles.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
    let enemy = roles
        .iter()
        .position(|r| matches!(r, Role::Rebel | Role::Traitor))
        .unwrap() as u8;

    advance_to_play(&mut e, lord);
    for p in 0..4 {
        e.debug_remove_general(p);
    }
    clear_all_hands(&mut e, 4);
    place_equip_on(&mut e, lord, CardKind::Langeguaiyi);
    e.debug_grant(lord, SWD_LANGEGUAIYI_SKILL_KEY);
    let lf_def = e.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();

    // Case 1: Only Loyalist holds cards; Enemy is empty -> Club must NOT discard Loyalist!
    let loy_card = place_matching_card(&mut e, loyalist, |_, d| d.kind == CardKind::Tao);
    set_deck_top_matching(&mut e, |d| d.suit == Suit::Club);
    let out = submit(&mut e, lord, GameAction::ActivateSkill { skill: lf_def });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    settle_confirm(&mut e);
    assert_eq!(
        e.debug_hand(loyalist),
        vec![loy_card],
        "longfan Club must filter is_host_enemy and NEVER discard Loyalist ally"
    );

    // Case 2: Enemy holds both 1 hand card and 1 Armor equip -> uniform sampling hits equip across seeds, single CardDiscarded log!
    let mut hit_hand = false;
    let mut hit_equip = false;
    for seed in 90_210..90_235u64 {
        let (_v2, mut e2) = load_formal_swd_only_mode(seed, 4, LockedStartMode::Identity);
        let roles2 = e2.state_core().identity_roles().unwrap();
        let lord2 = roles2.iter().position(|r| *r == Role::Lord).unwrap() as u8;
        let loy2 = roles2.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
        let enemy2 = roles2
            .iter()
            .position(|r| matches!(r, Role::Rebel | Role::Traitor))
            .unwrap() as u8;
        advance_to_play(&mut e2, lord2);
        for p in 0..4 {
            e2.debug_remove_general(p);
        }
        clear_all_hands(&mut e2, 4);
        place_equip_on(&mut e2, lord2, CardKind::Langeguaiyi);
        e2.debug_grant(lord2, SWD_LANGEGUAIYI_SKILL_KEY);
        let def2 = e2.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
        let _loy_c = place_matching_card(&mut e2, loy2, |_, d| d.kind == CardKind::Shan);
        let enemy_h = place_matching_card(&mut e2, enemy2, |_, d| d.kind == CardKind::Tao);
        let enemy_eq = place_equip_on(&mut e2, enemy2, CardKind::Armor);
        set_deck_top_matching(&mut e2, |d| d.suit == Suit::Club);
        let log_start = e2.log().len();
        let _ = submit(&mut e2, lord2, GameAction::ActivateSkill { skill: def2 });
        settle_confirm(&mut e2);
        assert_eq!(e2.debug_hand(loy2).len(), 1, "Loyalist never targeted");
        let disc_logs: Vec<_> = e2.log()[log_start..]
            .iter()
            .filter_map(|ev| match ev {
                LogEvent::CardDiscarded { player, cards } if *player == enemy2 => Some(cards.clone()),
                _ => None,
            })
            .collect();
        assert_eq!(disc_logs.len(), 1, "must emit exactly 1 CardDiscarded log, got {disc_logs:?}");
        if !e2.debug_hand(enemy2).contains(&enemy_h) {
            hit_hand = true;
            assert!(e2.state_core().discard.contains(&enemy_h));
        } else {
            hit_equip = true;
            assert!(!e2.debug_equips(enemy2).contains(&enemy_eq));
            assert!(e2.state_core().discard.contains(&enemy_eq));
        }
    }
    let _ = enemy;
    assert!(hit_hand && hit_equip, "longfan Club must sample 'he' (both hand and equip) uniformly");

    // Case 3: settle_inline_judge_card yields BEFORE apply_longfan_suit runs!
    // Co-load taixu + swd and grant `_buff_txhj_huangtianzhimin`:
    // With deck top (last) = Diamond judge card, second-to-top = c_ht, third-to-top = c_lf:
    // settle_inline_judge_card triggers huangtianzhimin (draws c_ht FIRST) and yields to SelectNature,
    // THEN apply_longfan_suit draws c_lf SECOND -> hand == [c_ht, c_lf]!
    let freeze_ext = |dir: &str| {
        let pdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions")
            .join(dir);
        let raw = std::fs::read_to_string(pdir.join("package.npack.json"))
            .unwrap()
            .replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let mut assets = std::collections::BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pdir.join(rel))
                    .unwrap()
                    .into_iter()
                    .filter(|&b| b != b'\r')
                    .collect::<Vec<u8>>();
                assets.insert(rel.to_string(), bytes);
            }
        }
        content_native_loader::validate_and_freeze_raw_package(raw, assets).unwrap()
    };
    let v_tx = content_native_loader::prepare_locked_content_with_bridge(vec![
        freeze_ext("taixu-upstream-liuli"),
        freeze_ext("shahai-swd-upstream-noname"),
    ])
    .expect("prepare swd + taixu");
    let mut et = content_native_loader::start_locked_engine(
        90_240,
        2,
        LockedStartMode::Prototype,
        &v_tx,
    )
    .expect("start swd + taixu");
    for p in 0..2 {
        et.debug_remove_general(p);
    }
    advance_to_play(&mut et, 0);
    clear_all_hands(&mut et, 2);
    place_equip_on(&mut et, 0, CardKind::Langeguaiyi);
    et.debug_grant(0, SWD_LANGEGUAIYI_SKILL_KEY);
    et.debug_grant(0, "_buff_txhj_huangtianzhimin");
    let judge_d = (0..et.catalog().cards.len() as u32)
        .find(|&id| et.catalog().cards[id as usize].suit == Suit::Diamond)
        .unwrap();
    let (c_ht, c_lf) = {
        let deck = &et.state_core().deck;
        let mut picks = deck.iter().rev().copied().filter(|&id| id != judge_d);
        (picks.next().unwrap(), picks.next().unwrap())
    };
    {
        let core = et.state_core_mut();
        for x in [judge_d, c_ht, c_lf] {
            core.deck.retain(|c| *c != x);
            core.discard.retain(|c| *c != x);
            core.table.retain(|c| *c != x);
            for pl in &mut core.players {
                pl.hand.retain(|c| *c != x);
            }
        }
        core.deck.push(c_lf);
        core.deck.push(c_ht);
        core.deck.push(judge_d);
    }
    let lf_tx = et.catalog().skill_by_name(SWD_LANGEGUAIYI_SKILL_KEY).unwrap();
    let _ = submit(&mut et, 0, GameAction::ActivateSkill { skill: lf_tx });
    settle_confirm(&mut et);
    assert_eq!(
        et.debug_hand(0),
        vec![c_ht, c_lf],
        "settle_inline_judge_card (huangtianzhimin c_ht) must settle BEFORE longfan suit draw (c_lf)"
    );
}

#[test]
fn u05_b7d_p3a_touzhi_damage_before_gain_and_dead_target_discards() {
    // Case A: Dead target discards weapon instead of giving weapon to dead hand
    let (_v, mut e) = load_formal_swd_only(90_301, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
    }
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 3);
    place_equip_on(&mut e, 0, CardKind::Yufulu);
    e.debug_grant(0, SWD_YUFULU_SKILL_KEY);
    let yf_def = e.catalog().skill_by_name(SWD_YUFULU_SKILL_KEY).unwrap();
    let wp = find_unassigned_card(&e, &[], |d| d.kind == CardKind::Weapon);
    move_to_hand(&mut e, 0, wp);
    e.debug_set_hp(1, 1);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: yf_def });
    let ch_w = e.open_choice().unwrap().clone();
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_w.id,
            selection: vec![ChoiceOption::Card(wp)],
        },
    );
    let ch_t = e.open_choice().unwrap().clone();
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_t.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_confirm(&mut e);
    assert!(!e.state_core().player(1).alive, "seat 1 died from touzhi 1 dmg");
    assert!(e.debug_hand(1).is_empty(), "weapon must NOT go to dead target's hand");
    assert!(e.state_core().discard.contains(&wp), "weapon must go to discard when target died");

    // Case B: Alive target receives Damage(1) first; taking wp from owner's hand triggers owner's anlianying!
    let (_v2, mut e2) = load_formal_swd_only(90_302, 3);
    for p in 0..3 {
        e2.debug_remove_general(p);
    }
    advance_to_play(&mut e2, 0);
    clear_all_hands(&mut e2, 3);
    place_equip_on(&mut e2, 0, CardKind::Yufulu);
    e2.debug_grant(0, SWD_YUFULU_SKILL_KEY);
    e2.debug_grant(0, "anlianying");
    let yf_def2 = e2.catalog().skill_by_name(SWD_YUFULU_SKILL_KEY).unwrap();
    let wp2 = find_unassigned_card(&e2, &[], |d| d.kind == CardKind::Weapon);
    move_to_hand(&mut e2, 0, wp2);
    e2.debug_set_hp(1, 4);
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: yf_def2 });
    let ch_w2 = e2.open_choice().unwrap().clone();
    submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: ch_w2.id,
            selection: vec![ChoiceOption::Card(wp2)],
        },
    );
    let ch_t2 = e2.open_choice().unwrap().clone();
    submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: ch_t2.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    settle_confirm(&mut e2);
    assert_eq!(e2.debug_hp(1), 3, "target took 1 damage");
    assert!(e2.debug_hand(1).contains(&wp2), "surviving target gained weapon");
    assert_eq!(e2.debug_hand(0).len(), 2, "take_hand_card fired anlianying when wp2 left hand");
}

#[test]
fn u05_b7d_p3a_pojian_and_anlianying_all_lose_paths_and_equip_negative() {
    let (_v, mut e) = load_formal_swd_only(90_401, 4);
    for p in 0..4 {
        e.debug_remove_general(p);
    }
    e.assign_general_named(0, "swd_qi").unwrap();
    e.assign_general_named(1, "swd_anka").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 4);

    // 1) PlayCard empties seat 0 (swd_qi, pojian) hand -> equips first equip from deck!
    let tao0 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Tao);
    e.debug_set_hp(0, 2);
    let out = submit(&mut e, 0, GameAction::PlayCard { card: tao0, targets: vec![] });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    settle_confirm(&mut e);
    assert!(!e.debug_equips(0).is_empty(), "PlayCard emptying hand must trigger pojian");

    // 2) RespondShan empties seat 1 (swd_anka, anlianying) hand -> draws 2!
    let sha0 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Sha);
    let shan1 = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Shan);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    let ch = e.open_choice().expect("respond_shan").clone();
    assert_eq!(ch.observer, 1);
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: ch.id,
            selection: vec![ChoiceOption::Card(shan1)],
        },
    );
    settle_confirm(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "CardResponded emptying hand must trigger anlianying (+2)");

    // 3) Snatch (CardsMoved) empties seat 1's single hand card -> draws 2!
    advance_to_play(&mut e, 2);
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 4);
    let snatch0 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Snatch);
    let _only1 = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Tao);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: snatch0, targets: vec![1] });
    settle_confirm(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "Snatch (CardsMoved) emptying hand must trigger anlianying");

    // 4) Negative: equip loss when hand is ALREADY empty must NOT falsely trigger pojian/anlianying!
    advance_to_play(&mut e, 2);
    clear_all_hands(&mut e, 4);
    e.state_core_mut().players[0].equips.clear();
    e.state_core_mut().players[1].equips.clear();
    let eq0 = place_equip_on(&mut e, 0, CardKind::Armor);
    let eq1 = place_equip_on(&mut e, 1, CardKind::Armor);
    let _ = eq0;
    let dis1 = place_matching_card(&mut e, 2, |_, d| d.kind == CardKind::Dismantle);
    let dis2 = find_unassigned_card(&e, &[dis1], |d| d.kind == CardKind::Dismantle);
    move_to_hand(&mut e, 2, dis2);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: dis1, targets: vec![0] });
    settle_confirm(&mut e);
    assert!(e.debug_equips(0).is_empty(), "seat 0 equip discarded without re-equipping via pojian");
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: dis2, targets: vec![1] });
    settle_confirm(&mut e);
    assert!(e.debug_equips(1).is_empty(), "seat 1 equip discarded");
    assert!(e.debug_hand(1).is_empty(), "equip-only discard when hand already empty must NOT trigger anlianying");
    let _ = eq1;

    // 5) Anti-double-fire: Dismantle emptying seat 0's hand when equips in deck/discard only fires pojian ONCE!
    e.debug_clear_hand(0);
    e.state_core_mut().players[0].equips.clear();
    let _h0 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Tao);
    let dis3 = find_unassigned_card(&e, &[], |d| d.kind == CardKind::Dismantle);
    move_to_hand(&mut e, 2, dis3);
    let _ = submit(&mut e, 2, GameAction::PlayCard { card: dis3, targets: vec![0] });
    settle_confirm(&mut e);
    assert_eq!(
        e.debug_equips(0).len(),
        1,
        "pojian on CardsDiscarded must fire ONCE, not double-fire into two equips"
    );
}

#[test]
fn u05_b7d_p3a_xuanyuan_ai_public_avoidance_and_self_allowed() {
    let (_v, mut e) = load_formal_swd_only(90_501, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
        e.debug_set_hp(p, 4);
    }
    e.assign_general_named(1, "swd_xuanyuanjianxian").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 3);
    let xuan = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Xuanyuanjian);

    let mut planner1 = BoundedPlanner::new(32);
    planner1.policy_variant = 1;
    let plan1 = e.plan_action(0, &planner1);
    assert_eq!(
        plan1.action,
        GameAction::NextPhase,
        "public-info AI must avoid equipping Xuanyuanjian when hostile player visibly has 'xuanyuan'"
    );

    let mut planner2 = BoundedPlanner::new(32);
    planner2.policy_variant = 2;
    planner2.l01_options = Some(L01AiOptions::default());
    let plan2 = e.plan_action(0, &planner2);
    assert_ne!(
        plan2.action,
        GameAction::PlayCard { card: xuan, targets: vec![] },
        "L01 AI must also avoid equipping Xuanyuanjian against hostile 'xuanyuan'"
    );

    // Control: when actor itself has 'xuanyuan' (or no enemy has it), AI equips Xuanyuanjian!
    e.assign_general_named(0, "swd_xuanyuanjianxian").unwrap();
    let plan_self = e.plan_action(0, &planner1);
    assert_eq!(
        plan_self.action,
        GameAction::PlayCard { card: xuan, targets: vec![] },
        "actor holding 'xuanyuan' should equip Xuanyuanjian normally"
    );
}

#[test]
fn u05_b7d_p3a_jqimou_draw_then_choose_use_roundtrip_ai_and_decline() {
    let (v, mut e) = load_formal_swd_only(90_601, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
        e.debug_set_hp(p, 4);
    }
    e.assign_general_named(1, "swd_jiliang").unwrap();
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 3);

    let sha0 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Sha);
    let sha1_draw = set_deck_top_matching(&mut e, |d| d.kind == CardKind::Sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha0, targets: vec![1] });
    // seat 1 has 0 hand cards so take 1 dmg -> jqimou draws sha1_draw -> opens swd_jqimou_use!
    let ch_use = e.open_choice().expect("swd_jqimou_use").clone();
    assert_eq!(ch_use.observer, 1);
    assert_eq!(ch_use.spec.prompt, "swd_jqimou_use");
    assert!(ch_use.spec.allow_cancel);
    assert!(ch_use.spec.options.contains(&ChoiceOption::Card(sha1_draw)));
    assert!(e.debug_hand(1).contains(&sha1_draw), "draw(1) settled before swd_jqimou_use");

    // Public-info AI picks sha1_draw to strike back
    assert_eq!(
        e.plan_action(1, &BoundedPlanner::new(16)).action,
        GameAction::Answer {
            choice_id: ch_use.id,
            selection: vec![ChoiceOption::Card(sha1_draw)],
        }
    );

    // NGSA 107 save/restore mid swd_jqimou_use
    let mut er = roundtrip_bytes(&v, &e);
    submit(
        &mut er,
        1,
        GameAction::Answer {
            choice_id: ch_use.id,
            selection: vec![ChoiceOption::Card(sha1_draw)],
        },
    );
    let ch_tgt = er.open_choice().expect("swd_jqimou_target").clone();
    assert_eq!(ch_tgt.observer, 1);
    assert_eq!(ch_tgt.spec.prompt, "swd_jqimou_target");
    assert!(ch_tgt.spec.allow_cancel);

    // Decline path check on clone (allow_cancel: true -> empty selection cancels)
    let mut e_decline = e;
    let out_decl = submit(
        &mut e_decline,
        1,
        GameAction::Answer {
            choice_id: ch_use.id,
            selection: vec![],
        },
    );
    assert!(!matches!(out_decl, CommandOutcome::Rejected(_)), "{out_decl:?}");
    assert!(e_decline.open_choice().is_none());
    assert!(e_decline.debug_hand(1).contains(&sha1_draw), "decline leaves drawn Sha in hand");

    // NGSA 107 save/restore mid swd_jqimou_target + real damage against seat 0
    let mut er2 = roundtrip_bytes(&v, &er);
    let hp0_before = er2.debug_hp(0);
    submit(
        &mut er2,
        1,
        GameAction::Answer {
            choice_id: ch_tgt.id,
            selection: vec![ChoiceOption::Player(0)],
        },
    );
    settle_confirm(&mut er2);
    assert_eq!(er2.debug_hp(0), hp0_before - 1, "jqimou counter-Sha hits seat 0");
    assert!(!er2.debug_hand(1).contains(&sha1_draw), "jqimou consumed Sha");
}

#[test]
fn u05_b7d_p3a_xingdian_identity_enemies_rng_seat_order_uniform_he_and_single_log() {
    let mut hit_combos = std::collections::BTreeSet::new();
    let mut saw_equip_hit = false;

    for seed in 90_700..90_725u64 {
        let (_v, mut e) = load_formal_swd_only_mode(seed, 5, LockedStartMode::Identity);
        let roles = e.state_core().identity_roles().unwrap();
        let lord = roles.iter().position(|r| *r == Role::Lord).unwrap() as u8;
        let loyalist = roles.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
        let enemies: Vec<u8> = (0..5)
            .filter(|&p| matches!(roles[p as usize], Role::Rebel | Role::Traitor))
            .collect();
        assert_eq!(enemies.len(), 3);

        for p in 0..5 {
            e.debug_remove_general(p);
        }
        e.assign_general_named(lord, "swd_lanyin").unwrap();
        advance_to_play(&mut e, lord);
        clear_all_hands(&mut e, 5);
        let cost_c = place_matching_card(&mut e, lord, |_, d| d.kind == CardKind::Tao);
        let loy_c = place_matching_card(&mut e, loyalist, |_, d| d.kind == CardKind::Shan);
        let e0_h = find_unassigned_card(&e, &[cost_c, loy_c], |d| d.kind == CardKind::Sha);
        let e0_eq = find_unassigned_card(&e, &[cost_c, loy_c, e0_h], |d| d.kind == CardKind::Armor);
        let e1_h = find_unassigned_card(&e, &[cost_c, loy_c, e0_h, e0_eq], |d| d.kind == CardKind::Sha);
        let e2_h = find_unassigned_card(&e, &[cost_c, loy_c, e0_h, e0_eq, e1_h], |d| d.kind == CardKind::Tao);
        move_to_hand(&mut e, enemies[0], e0_h);
        e.state_core_mut().deck.retain(|x| *x != e0_eq);
        e.state_core_mut().discard.retain(|x| *x != e0_eq);
        e.state_core_mut().players[enemies[0] as usize].equips.push(e0_eq);
        move_to_hand(&mut e, enemies[1], e1_h);
        move_to_hand(&mut e, enemies[2], e2_h);

        let xd_def = e.catalog().skill_by_name("xingdian").unwrap();
        let log_start = e.log().len();
        let _ = submit(&mut e, lord, GameAction::ActivateSkill { skill: xd_def });
        let ch = e.open_choice().unwrap().clone();
        submit(
            &mut e,
            lord,
            GameAction::Answer {
                choice_id: ch.id,
                selection: vec![ChoiceOption::Card(cost_c)],
            },
        );
        settle_confirm(&mut e);

        assert_eq!(e.debug_hand(loyalist), vec![loy_c], "xingdian NEVER hits Loyalist ally");
        let victims_hit: Vec<u8> = e.log()[log_start..]
            .iter()
            .filter_map(|ev| match ev {
                LogEvent::CardDiscarded { player, .. } if *player != lord => Some(*player),
                _ => None,
            })
            .collect();
        assert_eq!(victims_hit.len(), 2, "xingdian must pick exactly 2 enemies with 1 CardDiscarded log each");
        assert_eq!(
            e.log()[log_start..]
                .iter()
                .filter(|ev| matches!(ev, LogEvent::CardDiscarded { .. }))
                .count(),
            3,
            "1 owner discard + 2 victim discards = 3 CardDiscarded logs (no double logs)"
        );
        // Check seat order from lord!
        let step0 = e.state_core().seat_steps_from(lord, victims_hit[0]);
        let step1 = e.state_core().seat_steps_from(lord, victims_hit[1]);
        assert!(step0 < step1, "victims must be discarded in seat order from lord");
        hit_combos.insert(victims_hit);
        if !e.debug_equips(enemies[0]).contains(&e0_eq) {
            saw_equip_hit = true;
        }
    }
    assert!(hit_combos.len() >= 2, "xingdian RNG must select different enemy pairs across seeds");
    assert!(saw_equip_hit, "xingdian must sample equips uniformly in 'he'");
}

#[test]
fn u05_b7d_p3a_lanzhi_optional_decline_and_public_ai() {
    let (_v, mut e) = load_formal_swd_only_mode(90_801, 4, LockedStartMode::Identity);
    let roles = e.state_core().identity_roles().unwrap();
    let lord = roles.iter().position(|r| *r == Role::Lord).unwrap() as u8;
    let loyalist = roles.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
    let enemy = roles
        .iter()
        .position(|r| matches!(r, Role::Rebel | Role::Traitor))
        .unwrap() as u8;
    for p in 0..4 {
        e.debug_remove_general(p);
    }
    e.assign_general_named(lord, "swd_huanyuanzhi").unwrap();
    advance_to_play(&mut e, lord);
    clear_all_hands(&mut e, 4);

    // Case A: Lord full HP (4/4), only enemy injured (2 <= 4) -> optional prompt opens & AI declines (Pass)!
    e.debug_set_hp(lord, 4);
    e.debug_set_hp(loyalist, 4);
    e.debug_set_hp(enemy, 2);
    let club1 = place_matching_card(&mut e, lord, |_, d| d.suit == Suit::Club && d.kind == CardKind::Jiu);
    let _ = submit(&mut e, lord, GameAction::PlayCard { card: club1, targets: vec![] });
    let ch = e.open_choice().expect("lanzhi optional trigger_confirm").clone();
    assert_eq!(ch.observer, lord);
    assert_eq!(ch.spec.prompt, "trigger_confirm");
    assert!(ch.spec.allow_cancel);
    assert_eq!(
        e.plan_action(lord, &BoundedPlanner::new(8)).action,
        GameAction::Answer {
            choice_id: ch.id,
            selection: vec![ChoiceOption::Pass],
        },
        "AI must decline lanzhi when only public enemies/neutrals benefit"
    );
    submit(
        &mut e,
        lord,
        GameAction::Answer {
            choice_id: ch.id,
            selection: vec![ChoiceOption::Pass],
        },
    );
    assert_eq!(e.debug_hp(enemy), 2, "declined lanzhi leaves enemy unhealed");

    // Case B: Lord & Loyalist injured (<= lord.hp), enemy full -> AI confirms and heals both allies!
    let club2 = place_matching_card(&mut e, lord, |_, d| d.suit == Suit::Club && d.kind.is_equip());
    e.debug_set_hp(lord, 2);
    e.debug_set_hp(loyalist, 2);
    e.debug_set_hp(enemy, 4);
    let _ = submit(&mut e, lord, GameAction::PlayCard { card: club2, targets: vec![] });
    let ch2 = e.open_choice().expect("lanzhi trigger_confirm").clone();
    let plan2 = e.plan_action(lord, &BoundedPlanner::new(8));
    assert_eq!(
        plan2.action,
        GameAction::Answer {
            choice_id: ch2.id,
            selection: vec![ChoiceOption::Confirm],
        },
        "AI confirms lanzhi when self/allies gain net healing"
    );
    submit(&mut e, lord, plan2.action);
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(lord), 3);
    assert_eq!(e.debug_hp(loyalist), 3);
    assert_eq!(e.debug_hp(enemy), 4);

    // Case C: No player is damaged (all full HP) -> lanzhi filter MUST NOT open trigger_confirm!
    e.debug_set_hp(lord, 4);
    e.debug_set_hp(loyalist, 4);
    e.debug_set_hp(enemy, 4);
    let club3 = place_matching_card(&mut e, lord, |_, d| d.suit == Suit::Club && d.kind == CardKind::Jiu);
    let _ = submit(&mut e, lord, GameAction::PlayCard { card: club3, targets: vec![] });
    assert!(
        e.open_choice().is_none(),
        "lanzhi must NOT prompt when no damaged player has hp <= owner.hp"
    );
}

#[test]
fn u05_b7d_p3a_yueren_once_per_turn_and_nonblack_recovers_exact_sha() {
    let (_v, mut e) = load_formal_swd_only(90_901, 3);
    for p in 0..3 {
        e.debug_remove_general(p);
        e.state_core_mut().player_mut(p).max_hp = 6;
        e.debug_set_hp(p, 6);
    }
    e.assign_general_named(0, "swd_kama").unwrap();
    e.debug_grant(0, "咆哮");
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 3);

    let sha1 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Sha);
    let sha2 = find_unassigned_card(&e, &[sha1], |d| d.kind == CardKind::Sha);
    move_to_hand(&mut e, 0, sha2);
    let p1_card = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Tao);

    // 1) Optional prompt: declining swd_yueren_confirm leaves usable:1 (yueren2) unconsumed!
    set_deck_top_matching(&mut e, |d| d.suit == Suit::Spade);
    let deck_before_decl = e.state_core().deck.len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    let ch_decl = e.open_choice().expect("swd_yueren_confirm prompt").clone();
    assert_eq!(ch_decl.spec.prompt, "swd_yueren_confirm");
    assert_eq!(ch_decl.observer, 0);
    assert!(ch_decl.spec.allow_cancel);
    assert_eq!(
        e.plan_action(0, &BoundedPlanner::new(2)).action,
        GameAction::Answer {
            choice_id: ch_decl.id,
            selection: vec![ChoiceOption::Confirm],
        }
    );
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: ch_decl.id,
            selection: Vec::new(),
        },
    );
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 5, "declined yueren still resolves Sha (6 -> 5)");
    assert!(e.debug_hand(1).contains(&p1_card), "declined yueren did not discard p1_card");
    assert_eq!(e.state_core().deck.len(), deck_before_decl, "declined yueren drew no judge card");

    // 2) Same turn sha2 STILL prompts swd_yueren_confirm (save/restore + Heart judge recovers exact sha2)!
    let judge_h = set_deck_top_matching(&mut e, |d| d.suit == Suit::Heart);
    let deck_before = e.state_core().deck.len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    let ch_conf = e.open_choice().expect("swd_yueren_confirm after decline").clone();
    assert_eq!(ch_conf.spec.prompt, "swd_yueren_confirm");
    let snap = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save())).unwrap();
    let mut er = content_native_loader::restore_locked_engine(&snap, &_v).expect("restore on swd_yueren_confirm");
    let ch_r = er.open_choice().expect("restored swd_yueren_confirm").clone();
    assert_eq!(ch_r.spec.prompt, "swd_yueren_confirm");
    let _ = submit(
        &mut er,
        0,
        GameAction::Answer {
            choice_id: ch_r.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    e = er;
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 4, "2nd Sha dealt 1 dmg");
    assert!(e.debug_hand(0).contains(&sha2), "non-black judge recovers exact sha2 card");
    assert_eq!(e.debug_hand(0).len(), 1, "hand has only recovered sha2");
    assert_eq!(e.state_core().deck.len(), deck_before - 1, "only judge_h left deck");
    let _ = judge_h;

    // 3) Re-using recovered sha2 in SAME turn -> capped at usable: 1, must NOT prompt swd_yueren_confirm!
    set_deck_top_matching(&mut e, |d| d.suit == Suit::Spade);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert!(
        e.open_choice().is_none_or(|c| c.spec.prompt != "swd_yueren_confirm"),
        "yueren2 usable:1 prevents 2nd trigger in same turn"
    );
    settle_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "3rd Sha dealt 1 dmg");
    assert!(e.debug_hand(1).contains(&p1_card), "yueren usable:1 prevented discard");

    // 4) Next turn of seat 0 -> yueren triggers cleanly again!
    advance_to_play(&mut e, 1);
    advance_to_play(&mut e, 0);
    clear_all_hands(&mut e, 3);
    let p1_card2 = place_matching_card(&mut e, 1, |_, d| d.kind == CardKind::Tao);
    set_deck_top_matching(&mut e, |d| d.suit == Suit::Spade);
    let sha3 = place_matching_card(&mut e, 0, |_, d| d.kind == CardKind::Sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha3, targets: vec![1] });
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("swd_yueren_confirm"));
    settle_confirm(&mut e);
    assert!(!e.debug_hand(1).contains(&p1_card2), "yueren resets on next turn and discards p1_card2");

    // 5) settle_inline_judge_card runs BEFORE apply_yueren_suit (order probe via _buff_txhj_huangtianzhimin):
    let freeze_ext = |dir: &str| {
        let pdir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions")
            .join(dir);
        let raw = std::fs::read_to_string(pdir.join("package.npack.json")).unwrap().replace("\r\n", "\n");
        let parsed: serde_json::Value = serde_json::from_str(&raw).unwrap();
        let mut assets = std::collections::BTreeMap::new();
        if let Some(paths) = parsed.get("asset_paths").and_then(|v| v.as_object()) {
            for rel in paths.values().filter_map(|v| v.as_str()) {
                let bytes = std::fs::read(pdir.join(rel)).unwrap().into_iter().filter(|&b| b != b'\r').collect::<Vec<u8>>();
                assets.insert(rel.to_string(), bytes);
            }
        }
        content_native_loader::validate_and_freeze_raw_package(raw, assets).unwrap()
    };
    let v_tx = content_native_loader::prepare_locked_content_with_bridge(vec![
        freeze_ext("taixu-upstream-liuli"),
        freeze_ext("shahai-swd-upstream-noname"),
    ])
    .unwrap();
    let mut et = content_native_loader::start_locked_engine(90_940, 2, LockedStartMode::Prototype, &v_tx).unwrap();
    for p in 0..2 {
        et.debug_remove_general(p);
    }
    advance_to_play(&mut et, 0);
    clear_all_hands(&mut et, 2);
    et.debug_grant(0, "yueren");
    et.debug_grant(0, "_buff_txhj_huangtianzhimin");
    let sha_tx = place_matching_card(&mut et, 0, |_, d| d.kind == CardKind::Sha);
    let judge_ht = (0..et.catalog().cards.len() as u32)
        .find(|&id| id != sha_tx && et.catalog().cards[id as usize].suit == Suit::Heart)
        .unwrap();
    let c_ht = et
        .state_core()
        .deck
        .iter()
        .rev()
        .copied()
        .find(|&id| id != sha_tx && id != judge_ht)
        .unwrap();
    {
        let core = et.state_core_mut();
        for x in [c_ht, judge_ht] {
            core.deck.retain(|c| *c != x);
            core.discard.retain(|c| *c != x);
            core.table.retain(|c| *c != x);
            for pl in &mut core.players {
                pl.hand.retain(|c| *c != x);
            }
        }
        core.deck.push(c_ht);
        core.deck.push(judge_ht);
    }
    let _ = submit(&mut et, 0, GameAction::PlayCard { card: sha_tx, targets: vec![1] });
    settle_confirm(&mut et);
    assert_eq!(
        et.debug_hand(0),
        vec![c_ht, sha_tx],
        "yueren must settle_inline_judge_card (c_ht drawn FIRST) BEFORE apply_yueren_suit (sha_tx recovered SECOND)"
    );

    // 6) yueren is NOT locked: flag-based seals (tieji/ow_qinru/hs_fengyin_xuetu) and rechanyuan must block swd_yueren_confirm!
    let (_vs, mut es) = load_formal_swd_only(90_941, 2);
    for p in 0..2 {
        es.debug_remove_general(p);
        es.state_core_mut().player_mut(p).max_hp = 6;
        es.debug_set_hp(p, 6);
    }
    es.assign_general_named(0, "swd_kama").unwrap();
    advance_to_play(&mut es, 0);
    clear_all_hands(&mut es, 2);

    for seal_flag in [
        "sbtieji_target_sealed",
        "fz_new_tieji_target_sealed",
        "ow_qinru_fengyin",
        "hs_fengyin_xuetu",
    ] {
        es.state_core_mut().turn.sha_used_play_phase = false;
        let hp1 = es.debug_hp(1);
        let p1_keep = place_matching_card(&mut es, 1, |_, d| d.kind == CardKind::Tao);
        let sha_s = place_matching_card(&mut es, 0, |_, d| d.kind == CardKind::Sha);
        set_deck_top_matching(&mut es, |d| d.suit == Suit::Spade);
        es.state_core_mut().vm_aux.flags.insert((0, seal_flag.to_string()), 1);
        let _ = submit(&mut es, 0, GameAction::PlayCard { card: sha_s, targets: vec![1] });
        assert!(
            es.open_choice().is_none_or(|c| c.spec.prompt != "swd_yueren_confirm"),
            "{seal_flag} must prevent yueren prompt"
        );
        settle_confirm(&mut es);
        assert_eq!(es.debug_hp(1), hp1 - 1, "sealed Sha still resolves damage normally");
        assert!(es.debug_hand(1).contains(&p1_keep), "{seal_flag} must prevent yueren discard");
        es.state_core_mut().vm_aux.flags.remove(&(0, seal_flag.to_string()));
        clear_all_hands(&mut es, 2);
    }

    // rechanyuan seal (hp <= 1 + rechanyuan): blocks yueren as well!
    es.state_core_mut().vm_aux.flags.insert((0, "l10_rechanyuan".to_string()), 1);
    es.debug_set_hp(0, 1);
    es.state_core_mut().turn.sha_used_play_phase = false;
    let p1_ch = place_matching_card(&mut es, 1, |_, d| d.kind == CardKind::Tao);
    let sha_ch = place_matching_card(&mut es, 0, |_, d| d.kind == CardKind::Sha);
    let hp1_ch = es.debug_hp(1);
    let _ = submit(&mut es, 0, GameAction::PlayCard { card: sha_ch, targets: vec![1] });
    assert!(
        es.open_choice().is_none_or(|c| c.spec.prompt != "swd_yueren_confirm"),
        "rechanyuan at hp<=1 must prevent yueren prompt"
    );
    settle_confirm(&mut es);
    assert_eq!(es.debug_hp(1), hp1_ch - 1);
    assert!(es.debug_hand(1).contains(&p1_ch));

    // Unsealed control (recover hp above 1 so rechanyuan no longer seals): yueren prompts again!
    es.debug_set_hp(0, 4);
    es.state_core_mut().turn.sha_used_play_phase = false;
    let sha_ok = place_matching_card(&mut es, 0, |_, d| d.kind == CardKind::Sha);
    let _ = submit(&mut es, 0, GameAction::PlayCard { card: sha_ok, targets: vec![1] });
    assert_eq!(
        es.open_choice().map(|c| c.spec.prompt),
        Some("swd_yueren_confirm"),
        "clearing seal restores yueren prompt"
    );
    settle_confirm(&mut es);
}

#[test]
fn u05_b7d_p3a_all_8_swd_generals_and_jiangwu_yijue_dangping_match() {
    let (_v, mut e) = load_formal_swd_only(91_001, 4);
    for (name, expected_skills) in [
        ("swd_qi", &["yaotong", "heihuo", "pojian"][..]),
        ("swd_anka", &["songci", "anlianying"][..]),
        ("swd_jiliang", &["yunchou", "gongxin", "jqimou"][..]),
        ("swd_lanyin", &["xingdian", "yulin", "luomei"][..]),
        ("swd_huanyuanzhi", &["swdtianshu", "lanzhi", "mufeng"][..]),
        ("swd_kama", &["yueren", "shangshi"][..]),
        ("swd_weida", &["yueren", "zhenlie", "duijue"][..]),
        ("swd_jiangwu", &["yijue", "dangping"][..]),
    ] {
        e.debug_remove_general(0);
        e.assign_general_named(0, name).unwrap_or_else(|err| panic!("assign {name}: {err:?}"));
        for sk in expected_skills {
            assert!(e.debug_has_skill(0, sk), "{name} missing {sk}");
        }
    }

    // Real match exercising swd_jiangwu (yijue + dangping) in Identity mode for AI + real Damage checks
    let (_vi, mut ei) = load_formal_swd_only_mode(91_002, 4, LockedStartMode::Identity);
    let roles = ei.state_core().identity_roles().unwrap();
    let lord = roles.iter().position(|r| *r == Role::Lord).unwrap() as u8;
    let loyalist = roles.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
    let enemy = roles
        .iter()
        .position(|r| matches!(r, Role::Rebel | Role::Traitor))
        .unwrap() as u8;
    let other_neighbor = if (enemy + 1) % 4 != lord {
        (enemy + 1) % 4
    } else {
        (enemy + 3) % 4
    };
    for p in 0..4 {
        ei.debug_remove_general(p);
        ei.debug_set_hp(p, 4);
    }
    ei.assign_general_named(lord, "swd_jiangwu").unwrap();
    // Grant enemy a non-locked skill (`gongxin`) so yijue fengyin disables it:
    ei.debug_grant(enemy, "gongxin");
    advance_to_play(&mut ei, lord);
    clear_all_hands(&mut ei, 4);

    let hi = place_matching_card(&mut ei, lord, |_, d| d.rank >= 12);
    let sha_real = find_unassigned_card(&ei, &[hi], |d| d.kind == CardKind::Sha);
    let cost_dp = find_unassigned_card(&ei, &[hi, sha_real], |d| d.kind == CardKind::Tao);
    move_to_hand(&mut ei, lord, sha_real);
    move_to_hand(&mut ei, lord, cost_dp);
    let lo = place_matching_card(&mut ei, enemy, |_, d| d.rank <= 3);
    let shan_enemy = find_unassigned_card(&ei, &[hi, sha_real, cost_dp, lo], |d| d.kind == CardKind::Shan);
    move_to_hand(&mut ei, enemy, shan_enemy);

    let yj_def = ei.catalog().skill_by_name("yijue").unwrap();
    let out = submit(&mut ei, lord, GameAction::ActivateSkill { skill: yj_def });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    let ch_tgt = ei.open_choice().unwrap().clone();
    assert_eq!(ch_tgt.spec.prompt, "yijue_target");
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_tgt.id, selection: vec![ChoiceOption::Player(enemy)] });
    let ch_p1 = ei.open_choice().unwrap().clone();
    assert_eq!(ch_p1.spec.prompt, "yijue_target_pick");
    submit(&mut ei, enemy, GameAction::Answer { choice_id: ch_p1.id, selection: vec![ChoiceOption::Card(lo)] });
    let ch_p0 = ei.open_choice().unwrap().clone();
    assert_eq!(ch_p0.spec.prompt, "yijue_player_pick");
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_p0.id, selection: vec![ChoiceOption::Card(hi)] });

    // Win yijue -> NO 1 dmg! Applies fengyin + yijue2 hand-card lock!
    assert!(ei.open_choice().is_none(), "yijue win must finish cleanly with no prompt");
    assert_eq!(ei.debug_hp(enemy), 4, "yijue win deals NO 1 damage (4 -> 4)");
    assert!(ei.catalog().is_xj_fengyin_sealed(ei.state_core(), enemy), "yijue win applies fengyin");
    assert!(ei.catalog().blocks_hand_cards(ei.state_core(), enemy), "yijue win applies yijue2 hand lock");
    assert!(!ei.debug_has_skill(enemy, "gongxin"), "fengyin seals enemy's non-locked skill");

    // Lord attacks enemy with real Sha -> enemy holds Shan in hand but CANNOT respond with it!
    // (For non-adjacent enemy seat, make enemy distance 1 using HorseMinus or skip distance if needed; pick adjacent enemy):
    let adj_enemy = (lord + 1) % 4;
    ei.debug_set_hp(adj_enemy, 4);
    let _ = other_neighbor;
    let out_sha = if adj_enemy == enemy {
        submit(&mut ei, lord, GameAction::PlayCard { card: sha_real, targets: vec![enemy] })
    } else {
        // Place enemy-locked seat as Sha target via direct distance override (HorseMinus equip)
        place_equip_on(&mut ei, lord, CardKind::HorseMinus);
        submit(&mut ei, lord, GameAction::PlayCard { card: sha_real, targets: vec![enemy] })
    };
    assert!(!matches!(out_sha, CommandOutcome::Rejected(_)), "sha failed: {out_sha:?}");
    // yijue2 locks enemy's hand Shan (`blocks_hand_cards`), so Sha auto-hits immediately -> triggers dangping!
    let neighbor = (enemy + 1) % 4;
    let nb_hp_before = ei.debug_hp(neighbor);
    let ch_dp_conf = ei.open_choice().expect("dangping trigger_confirm (yijue2 blocked Shan)").clone();
    assert_eq!(ch_dp_conf.spec.prompt, "trigger_confirm");
    assert!(
        ei.debug_hand(enemy).contains(&shan_enemy),
        "yijue2 MUST prevent target from playing hand Shan"
    );
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_dp_conf.id, selection: vec![ChoiceOption::Confirm] });
    let ch_dp_c = ei.open_choice().expect("dangping_discard").clone();
    assert_eq!(ch_dp_c.spec.prompt, "dangping_discard");
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_dp_c.id, selection: vec![ChoiceOption::Card(cost_dp)] });
    let ch_dp_t = ei.open_choice().expect("dangping_target").clone();
    assert_eq!(ch_dp_t.spec.prompt, "dangping_target");
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_dp_t.id, selection: vec![ChoiceOption::Player(neighbor)] });
    settle_confirm(&mut ei);
    assert_eq!(ei.debug_hp(enemy), 3, "real Sha dealt 1 dmg (4 -> 3)");
    assert_eq!(ei.debug_hp(neighbor), nb_hp_before - 1, "dangping dealt 1 dmg to enemy's neighbor");

    // Turn ends -> fengyin + yijue2 hand lock clear AND gongxin is restored!
    advance_to_play(&mut ei, (lord + 1) % 4);
    assert!(!ei.catalog().blocks_hand_cards(ei.state_core(), enemy), "yijue2 wears off at turn end");
    assert!(!ei.catalog().is_xj_fengyin_sealed(ei.state_core(), enemy), "fengyin wears off at turn end");
    assert!(ei.debug_has_skill(enemy, "gongxin"), "gongxin restored after turn end");

    // Next turn of lord: yijue LOSS vs damaged enemy -> chooseBool prompt ("swd_yijue_recover_confirm"),
    // public-info AI declines enemy (vec![]), save/restore works, and AI confirms damaged ally!
    advance_to_play(&mut ei, lord);
    clear_all_hands(&mut ei, 4);
    ei.debug_set_hp(loyalist, 4);
    let lo0 = place_matching_card(&mut ei, lord, |_, d| d.rank <= 3);
    let hi_e = place_matching_card(&mut ei, enemy, |_, d| d.rank >= 12);
    let _ = submit(&mut ei, lord, GameAction::ActivateSkill { skill: yj_def });
    let ch_t2 = ei.open_choice().unwrap().clone();
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_t2.id, selection: vec![ChoiceOption::Player(enemy)] });
    let ch_tp2 = ei.open_choice().unwrap().clone();
    submit(&mut ei, enemy, GameAction::Answer { choice_id: ch_tp2.id, selection: vec![ChoiceOption::Card(hi_e)] });
    let ch_pp2 = ei.open_choice().unwrap().clone();
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_pp2.id, selection: vec![ChoiceOption::Card(lo0)] });
    let ch_rec = ei.open_choice().expect("swd_yijue_recover_confirm").clone();
    assert_eq!(ch_rec.spec.prompt, "swd_yijue_recover_confirm");
    assert_eq!(
        ei.plan_action(lord, &BoundedPlanner::new(2)).action,
        GameAction::Answer { choice_id: ch_rec.id, selection: Vec::new() },
        "AI must decline recovering damaged enemy"
    );
    submit(&mut ei, lord, GameAction::Answer { choice_id: ch_rec.id, selection: Vec::new() });
    assert_eq!(ei.debug_hp(enemy), 3, "declined yijue recover leaves enemy at 3 HP");

    // Public-info AI check (`Loyalist` knows `Lord` role):
    // 1) `yijue_target` picks enemy over publicly known `Lord` (`options[0]` if lord_a < enemy_a),
    // 2) `Loyalist` losing clash to injured `enemy_a` WHILE `Lord` (`lord_a`) is ALSO injured declines recover (`swd_yijue_recover_confirm` carries `[Player(enemy_a)]`, NOT blind `has_injured_ally`),
    // 3) `Loyalist` losing clash to injured `lord_a` recovers `lord_a` cleanly across NGSA save/restore!
    let (_va, mut ea) = load_formal_swd_only_mode(91_003, 4, LockedStartMode::Identity);
    let roles_a = ea.state_core().identity_roles().unwrap();
    let lord_a = roles_a.iter().position(|r| *r == Role::Lord).unwrap() as u8;
    let loy_a = roles_a.iter().position(|r| *r == Role::Loyalist).unwrap() as u8;
    let enemy_a = roles_a
        .iter()
        .position(|r| matches!(r, Role::Rebel | Role::Traitor))
        .unwrap() as u8;
    for p in 0..4 {
        ea.debug_remove_general(p);
        ea.debug_set_hp(p, 4);
    }
    ea.assign_general_named(loy_a, "swd_jiangwu").unwrap();
    advance_to_play(&mut ea, loy_a);
    clear_all_hands(&mut ea, 4);
    ea.debug_set_hp(lord_a, 2);
    ea.debug_set_hp(enemy_a, 2);
    let lo_loy = place_matching_card(&mut ea, loy_a, |_, d| d.rank <= 3);
    let hi_enemy = find_unassigned_card(&ea, &[lo_loy], |d| d.rank >= 12);
    move_to_hand(&mut ea, enemy_a, hi_enemy);
    let lord_keep = find_unassigned_card(&ea, &[lo_loy, hi_enemy], |d| d.rank >= 12);
    move_to_hand(&mut ea, lord_a, lord_keep);
    let yj_a = ea.catalog().skill_by_name("yijue").unwrap();
    let _ = submit(&mut ea, loy_a, GameAction::ActivateSkill { skill: yj_a });
    let ch_tgt_ai = ea.open_choice().expect("yijue_target AI").clone();
    assert_eq!(ch_tgt_ai.spec.prompt, "yijue_target");
    assert!(
        ch_tgt_ai.spec.options.contains(&ChoiceOption::Player(lord_a))
            && ch_tgt_ai.spec.options.contains(&ChoiceOption::Player(enemy_a)),
        "both lord_a and enemy_a must be target options"
    );
    let picked_tgt = match ea.plan_action(loy_a, &BoundedPlanner::new(2)).action {
        GameAction::Answer { choice_id, selection } if choice_id == ch_tgt_ai.id => selection,
        other => panic!("unexpected yijue_target AI plan: {other:?}"),
    };
    assert_eq!(picked_tgt.len(), 1);
    let ChoiceOption::Player(chosen_p) = picked_tgt[0] else {
        panic!("expected Player choice: {picked_tgt:?}");
    };
    assert_ne!(chosen_p, lord_a, "yijue_target AI must NOT target publicly known ally Lord");

    // Clash against injured enemy_a WHILE ally lord_a IS ALSO injured -> must decline recovering enemy_a!
    submit(&mut ea, loy_a, GameAction::Answer { choice_id: ch_tgt_ai.id, selection: vec![ChoiceOption::Player(enemy_a)] });
    let ch_tp_e = ea.open_choice().unwrap().clone();
    submit(&mut ea, enemy_a, GameAction::Answer { choice_id: ch_tp_e.id, selection: vec![ChoiceOption::Card(hi_enemy)] });
    let ch_pp_e = ea.open_choice().unwrap().clone();
    submit(&mut ea, loy_a, GameAction::Answer { choice_id: ch_pp_e.id, selection: vec![ChoiceOption::Card(lo_loy)] });
    let ch_rec_e = ea.open_choice().expect("swd_yijue_recover_confirm enemy").clone();
    assert_eq!(ch_rec_e.spec.prompt, "swd_yijue_recover_confirm");
    assert_eq!(ch_rec_e.spec.options, vec![ChoiceOption::Player(enemy_a)]);
    assert_eq!(
        ea.plan_action(loy_a, &BoundedPlanner::new(2)).action,
        GameAction::Answer { choice_id: ch_rec_e.id, selection: Vec::new() },
        "AI must decline recovering injured enemy even when third-seat Lord ally is also damaged"
    );
    submit(&mut ea, loy_a, GameAction::Answer { choice_id: ch_rec_e.id, selection: Vec::new() });
    assert_eq!(ea.debug_hp(enemy_a), 2, "declined yijue recover leaves enemy_a at 2 HP");

    // Next turn of loy_a: clash loss vs injured Lord ally -> save/restore + AI confirms recovering Lord!
    advance_to_play(&mut ea, (loy_a + 1) % 4);
    advance_to_play(&mut ea, loy_a);
    clear_all_hands(&mut ea, 4);
    ea.debug_set_hp(lord_a, 2);
    let lo_loy2 = place_matching_card(&mut ea, loy_a, |_, d| d.rank <= 3);
    let hi_lord = place_matching_card(&mut ea, lord_a, |_, d| d.rank >= 12);
    let _ = submit(&mut ea, loy_a, GameAction::ActivateSkill { skill: yj_a });
    let ch_t3 = ea.open_choice().unwrap().clone();
    submit(&mut ea, loy_a, GameAction::Answer { choice_id: ch_t3.id, selection: vec![ChoiceOption::Player(lord_a)] });
    let ch_tp3 = ea.open_choice().unwrap().clone();
    submit(&mut ea, lord_a, GameAction::Answer { choice_id: ch_tp3.id, selection: vec![ChoiceOption::Card(hi_lord)] });
    let ch_pp3 = ea.open_choice().unwrap().clone();
    submit(&mut ea, loy_a, GameAction::Answer { choice_id: ch_pp3.id, selection: vec![ChoiceOption::Card(lo_loy2)] });
    let ch_rec_ally = ea.open_choice().expect("swd_yijue_recover_confirm ally").clone();
    assert_eq!(ch_rec_ally.spec.prompt, "swd_yijue_recover_confirm");
    assert_eq!(ch_rec_ally.spec.options, vec![ChoiceOption::Player(lord_a)]);
    let snap2 = engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&ea.save())).unwrap();
    let mut ea_r = content_native_loader::restore_locked_engine(&snap2, &_va).expect("restore on swd_yijue_recover_confirm");
    let ch_rec_r = ea_r.open_choice().unwrap().clone();
    assert_eq!(ch_rec_r.spec.prompt, "swd_yijue_recover_confirm");
    assert_eq!(ch_rec_r.spec.options, vec![ChoiceOption::Player(lord_a)]);
    assert_eq!(
        ea_r.plan_action(loy_a, &BoundedPlanner::new(2)).action,
        GameAction::Answer { choice_id: ch_rec_r.id, selection: vec![ChoiceOption::Player(lord_a)] },
        "AI must confirm recovering publicly known damaged Lord ally"
    );
    submit(&mut ea_r, loy_a, GameAction::Answer { choice_id: ch_rec_r.id, selection: vec![ChoiceOption::Player(lord_a)] });
    assert_eq!(ea_r.debug_hp(lord_a), 3, "confirmed yijue recover heals Lord 2 -> 3");

    // Next turn of loy_a: when ONLY Lord ally is a candidate (`options = [Player(lord_a)]`):
    // - healthy/low-hand Lord -> `yijue_target` AI cancels (`Vec::new()`),
    // - injured excess-hand Lord (`hand_count > hp + 1`) -> `yijue_target` AI selects Lord (`refresh/skill.js:16780`)!
    advance_to_play(&mut ea_r, (loy_a + 1) % 4);
    advance_to_play(&mut ea_r, loy_a);
    clear_all_hands(&mut ea_r, 4);
    ea_r.debug_set_hp(lord_a, 4);
    let _loy_card = place_matching_card(&mut ea_r, loy_a, |_, d| d.rank <= 3);
    let lord_only = find_unassigned_card(&ea_r, &[_loy_card], |d| d.rank >= 12);
    move_to_hand(&mut ea_r, lord_a, lord_only);
    let _ = submit(&mut ea_r, loy_a, GameAction::ActivateSkill { skill: yj_a });
    let ch_only_ally = ea_r.open_choice().expect("yijue_target ally-only").clone();
    assert_eq!(ch_only_ally.spec.options, vec![ChoiceOption::Player(lord_a)]);
    assert_eq!(
        ea_r.plan_action(loy_a, &BoundedPlanner::new(2)).action,
        GameAction::Answer { choice_id: ch_only_ally.id, selection: Vec::new() },
        "yijue_target AI must cancel when only a non-beneficial ally exists"
    );
    submit(&mut ea_r, loy_a, GameAction::Answer { choice_id: ch_only_ally.id, selection: Vec::new() });

    advance_to_play(&mut ea_r, (loy_a + 1) % 4);
    advance_to_play(&mut ea_r, loy_a);
    clear_all_hands(&mut ea_r, 4);
    ea_r.debug_set_hp(lord_a, 2); // excess threshold: hand_count > 2 + 1 = 3 -> 4 cards
    let _loy_card2 = place_matching_card(&mut ea_r, loy_a, |_, d| d.rank <= 3);
    let mut used_ids = vec![_loy_card2];
    for _ in 0..4 {
        let extra = find_unassigned_card(&ea_r, &used_ids, |_| true);
        used_ids.push(extra);
        move_to_hand(&mut ea_r, lord_a, extra);
    }
    let _ = submit(&mut ea_r, loy_a, GameAction::ActivateSkill { skill: yj_a });
    let ch_heal_ally = ea_r.open_choice().expect("yijue_target injured excess-hand ally").clone();
    assert_eq!(ch_heal_ally.spec.options, vec![ChoiceOption::Player(lord_a)]);
    assert_eq!(
        ea_r.plan_action(loy_a, &BoundedPlanner::new(2)).action,
        GameAction::Answer { choice_id: ch_heal_ally.id, selection: vec![ChoiceOption::Player(lord_a)] },
        "yijue_target AI should select injured ally with hand_count > hp + 1"
    );
    submit(&mut ea_r, loy_a, GameAction::Answer { choice_id: ch_heal_ally.id, selection: Vec::new() });
}
