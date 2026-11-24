use content_native_loader::{
    apply_coin_to_engine, apply_fool_to_engine, fool_extension_pack, martial_engine_from_config,
    CoinGate, CoinMode,
};
use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::content::extension::{ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook};
use engine_native::content::host::{HostBehavior, HostSkillDecl};
use engine_native::content::standard::Gender;
use engine_native::{
    ChoiceOption, Engine, FoolLiegongStyle, GameAction, Phase, U04SettleInput,
};
use serde_json::Value;

use super::helpers::{
    answer_choice, assert_accepted_or_pending, assert_archive_roundtrip, drain_default,
    move_card_to_deck_top, move_card_to_hand, move_card_to_pile, repo_asset, step_to_phase,
    submit,
};

#[test]
fn l21_martial_mode_configs_double_hp_opening_draw_death_replace_and_restore() {
    let raw_cfg = repo_asset("packages/extensions/martial-upstream-liuli/assets/config.json");
    let mut val: Value = serde_json::from_str(&raw_cfg).unwrap();

    // Seat 0 = Caocao(4) + XunYu?(4) / standard Caocao(4), Zhenji(3), Guojia(3), Xuzhu(4)
    let base = Engine::new(1, 2);
    let idx = |name: &str| {
        (0..base.catalog().general_count() as u16)
            .find(|&d| base.catalog().general_meta(d).map(|m| m.0) == Some(name))
            .unwrap()
    };
    let caocao = idx("曹操"); // 4 hp
    let zhenji = idx("甄姬"); // 3 hp
    let guojia = idx("郭嘉"); // 3 hp
    let liubei = idx("刘备"); // 4 hp
    let zhaoyun = idx("赵云"); // 4 hp
    let simayi = idx("司马懿"); // 3 hp

    let hp_expectations = [
        ("hejiansan", 4), // (4+3)-3 = 4
        ("pingjun", 3),   // floor((4+3)/2) = 3
        ("zuidazhi", 4),  // max(4,3) = 4
        ("zuixiaozhi", 3),// min(4,3) = 3
        ("zonghe", 7),    // 4+3 = 7
    ];
    for (rule, expected_hp) in hp_expectations {
        val["double_character"] = Value::Bool(true);
        val["double_hp"] = Value::String(rule.into());
        val["free_choose"] = Value::Bool(true);
        val["change_choice"] = Value::Bool(true);
        let cfg = serde_json::to_string(&val).unwrap();
        let e = martial_engine_from_config(
            901,
            &cfg,
            [vec![caocao, zhenji, guojia], vec![liubei, simayi, zhaoyun]],
            true,
            &[],
        )
        .unwrap();
        assert_eq!(e.state_core().player(0).max_hp, expected_hp, "double_hp={rule}");
        assert_eq!(e.state_core().player(0).hand.len(), 4, "zhu opening_draw = 4");
        assert_eq!(e.state_core().player(1).hand.len(), 5, "fan opening_draw = 5");
    }

    // Exercise real death replacement + draw 4 in single-general martial match (`double_character = false`)
    val["double_character"] = Value::Bool(false);
    val["double_hp"] = Value::String("hejiansan".into());
    val["free_choose"] = Value::Bool(false);
    val["change_choice"] = Value::Bool(true);
    let cfg = serde_json::to_string(&val).unwrap();
    let mut e = martial_engine_from_config(
        902,
        &cfg,
        [vec![caocao, guojia], vec![liubei, zhaoyun]],
        false,
        &[],
    )
    .unwrap();
    assert_eq!(e.state_core().player(0).hand.len(), 4);
    assert_eq!(e.state_core().player(1).hand.len(), 5);

    // free_choose rejected when disabled, change_choice swaps pool
    assert!(e.martial_free_choose(0, simayi).is_err(), "free_choose disabled");
    let swapped = e.martial_change_choice(0, 0).expect("change_choice swaps pool backup");
    assert_eq!(swapped, guojia);

    // Deal lethal Sha to seat 1 -> seat 1 dies -> backup zhaoyun pops in, revives, draws 4!
    step_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 1);
    let sha = e.debug_place(0, "sha");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_default(&mut e);
    assert!(e.outcome().is_none(), "backup zhaoyun kept match going");
    assert!(e.state_core().player(1).alive, "seat 1 revived with backup general");
    assert_eq!(e.state_core().player(1).general_slots[0], Some(zhaoyun));
    assert_eq!(e.state_core().player(1).hand.len(), 4, "replacement draws 4 cards");

    let restored = assert_archive_roundtrip(&e, &[]);
    assert_eq!(restored.state_core().player(1).general_slots[0], Some(zhaoyun));
}

#[test]
fn u04_coin_engine_lifecycle_gates_change_bet_settle_and_restore() {
    let raw_cfg = repo_asset("packages/extensions/coin-upstream-noname/assets/config.json");
    let gate_ok = CoinGate {
        mode: CoinMode::Identity,
        chess_leader: false,
        video: false,
        connect: false,
        online: false,
        phase_started: false,
    };

    let mut e = Engine::new_identity(515, 4);
    apply_coin_to_engine(&raw_cfg, gate_ok, &mut e).expect("apply coin");
    assert_eq!(e.u04_coin_balance(), Some(0));
    assert_eq!(e.u04_match_coin(), Some(0));

    // Pre-turn bet deducts 20 even into negative balance, blocked once phase starts
    assert_eq!(e.u04_place_bet().unwrap(), -20);
    assert!(e.u04_place_bet().is_err(), "cannot bet twice");

    e.u04_mark_phase_started();
    // changeCoin still adjusts balance while persistent
    assert_eq!(e.u04_change_coin(55).unwrap(), 35);

    let mut e_closed = Engine::new_identity(516, 4);
    apply_coin_to_engine(&raw_cfg, gate_ok, &mut e_closed).unwrap();
    e_closed.u04_mark_phase_started();
    assert!(e_closed.u04_place_bet().is_err(), "bet blocked after phase started");

    // Disabled when online or video or chess leader
    let mut e_online = Engine::new(517, 2);
    apply_coin_to_engine(
        &raw_cfg,
        CoinGate {
            online: true,
            connect: true,
            ..gate_ok
        },
        &mut e_online,
    )
    .unwrap();
    assert!(e_online.u04_change_coin(10).is_err(), "online disables coin persistence");
    assert!(e_online.u04_match_coin().is_none());

    // Settle victory on Identity with deterministic unit random & restore parity
    let payout = e
        .u04_settle_game(&U04SettleInput {
            victory: true,
            identity_zhu_or_zhong: true,
            identity_fan: false,
            identity_nei: false,
            enhance_zhu: false,
            guozhan_ye: false,
            guozhan_population: 0,
            versus_friends: None,
            chess_combat_extra: false,
            additional_reward: 0,
            coin_coeff: None,
            unit_random_override: Some(0.5),
        })
        .unwrap();
    // coeff=1.0, base_id=20*4/8=10, win=+20, bet=+10 -> payout=40, bet_return=20 -> bal=35+40+20=95
    assert_eq!(payout, 40);
    assert_eq!(e.u04_coin_balance(), Some(95));
    assert_eq!(e.u04_match_coin(), Some(40));

    let restored = assert_archive_roundtrip(&e, &[]);
    assert_eq!(restored.u04_coin_balance(), Some(95));
    assert_eq!(restored.u04_match_coin(), Some(40));
}

#[test]
fn l26_fool_liegong_and_fool_skill_real_engine_and_restore() {
    let raw_cfg = repo_asset("packages/extensions/fool-plugin-upstream-liuli/assets/config.json");
    let fool_base = fool_extension_pack(&[]);
    let general_pack = ExtensionPack {
        def: ExtensionDef {
            name: "l26-sp-generals-for-fool",
            version: 1,
            description: "mizhu / wangrong / re_zhangliang / huangzhong_lg",
        },
        generals: vec![
            ExtensionGeneral {
                name: "mizhu",
                faction: Faction::Shu,
                skills: &["ziyuan"],
                max_hp: 3,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "wangrong",
                faction: Faction::Qun,
                skills: &["minsi"],
                max_hp: 3,
                gender: Gender::Female,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "re_zhangliang",
                faction: Faction::Qun,
                skills: &["xinfu_fangtong"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "huangzhong_lg",
                faction: Faction::Shu,
                skills: &["liegong"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "jintao_tester",
                faction: Faction::Wei,
                skills: &["jintao", "咆哮"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
        ],
        cards: Vec::new(),
        host_skills: {
            let mut hs = fool_base.host_skills.clone();
            hs.push(HostSkillDecl {
                name: "liegong",
                behavior: HostBehavior::FzLiegong,
                presentation_binding: None,
            });
            hs.push(HostSkillDecl {
                name: "jintao",
                behavior: HostBehavior::Q4FoolSkill,
                presentation_binding: None,
            });
            hs
        },
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };

    let mut e = Engine::new_with_extensions(606, 2, &[&general_pack]).expect("fool engine");
    apply_fool_to_engine(&raw_cfg, &mut e).expect("apply fool config");
    assert_eq!(e.fool_liegong_style(), FoolLiegongStyle::Shousha);
    assert!(e.fool_skill_enabled());

    // 1. Fool_liegong live query on huangzhong ("烈弓") vs close style
    e.assign_general_named(0, "huangzhong_lg").unwrap();
    e.assign_general_named(1, "曹操").unwrap();
    step_to_phase(&mut e, 0, Phase::Play);
    // Ensure player 0 hp=4 <= target 1 hand len -> liegong triggers!
    while e.state_core().player(1).hand.len() < 4 {
        let _ = e.debug_place(1, "sha");
    }
    let hint = e.query_fool_sha_hints(0, 1, None, 0);
    assert_eq!(hint.as_deref(), Some("可烈弓"), "live engine Fool_liegong detects 烈弓");
    e.set_fool_liegong_style(FoolLiegongStyle::Close);
    assert_eq!(e.query_fool_sha_hints(0, 1, None, 0), None);
    e.set_fool_liegong_style(FoolLiegongStyle::Shousha);

    // 1b. BUG-10: jintao q4_play_sha_count progression (0th -> "加伤", 1st -> "不可响应", 2nd -> "")
    e.assign_general_named(0, "jintao_tester").unwrap();
    step_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(e.query_fool_sha_hints(0, 1, None, 0).as_deref(), Some("加伤"));
    let jt_sha1 = e.debug_place(0, "sha");
    let jt_sha2 = e.debug_place(0, "sha");
    e.debug_clear_hand(1);
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: jt_sha1, targets: vec![1] }));
    drain_default(&mut e);
    assert_eq!(e.query_fool_sha_hints(0, 1, None, 0).as_deref(), Some("不可响应"));
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: jt_sha2, targets: vec![1] }));
    drain_default(&mut e);
    assert_eq!(e.query_fool_sha_hints(0, 1, None, 0).as_deref(), Some(""));

    // 2. Fool_skill progressive candidate filtering & real ziyuan (5 -> live ChoiceSpec hides 10 when enabled, keeps when disabled)
    e.assign_general_named(0, "mizhu").unwrap();
    e.debug_clear_hand(0);
    let cards_13 = {
        let c5 = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.rank == 5 && !c.off_deck)
            .unwrap() as u32;
        let c8 = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.rank == 8 && !c.off_deck)
            .unwrap() as u32;
        let c10 = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.rank == 10 && !c.off_deck)
            .unwrap() as u32;
        move_card_to_hand(&mut e, 0, c5);
        move_card_to_hand(&mut e, 0, c8);
        move_card_to_hand(&mut e, 0, c10);
        (c5, c8, c10)
    };
    let legal_after_5 = e.fool_skill_filter_candidates(0, "ziyuan", &[cards_13.0]);
    assert!(legal_after_5.contains(&cards_13.1), "5 + 8 <= 13 allowed");
    assert!(!legal_after_5.contains(&cards_13.2), "5 + 10 = 15 > 13 filtered out");

    e.state_core_mut().player_mut(1).hp = 2;
    let ziyuan_def = e.catalog().skill_by_name("ziyuan").unwrap();
    let act_zy = submit(&mut e, 0, GameAction::ActivateSkill { skill: ziyuan_def });
    assert_accepted_or_pending(&act_zy);
    let mut picked_first_zy = false;
    let mut checked_live_filter_zy = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "ziyuan:cards" {
            if !picked_first_zy {
                picked_first_zy = true;
                answer_choice(&mut e, vec![ChoiceOption::Card(cards_13.0)]);
            } else {
                checked_live_filter_zy = true;
                assert!(!c.spec.options.contains(&ChoiceOption::Card(cards_13.2)), "BUG-10: progressive live ChoiceSpec excludes 10 after picking 5");
                assert!(c.spec.options.contains(&ChoiceOption::Card(cards_13.1)), "8 remains selectable");
                answer_choice(&mut e, vec![ChoiceOption::Card(cards_13.1)]);
            }
        } else if c.spec.prompt == "ziyuan:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
        } else {
            drain_default(&mut e);
        }
    }
    assert!(checked_live_filter_zy, "ziyuan used progressive two-step selection");
    assert_eq!(e.state_core().player(1).hp, 3, "ziyuan recovered wounded target +1 HP");
    // Counterexample: !fool_skill_enabled keeps c10 in progressive ziyuan options
    e.set_fool_skill_enabled(false);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    step_to_phase(&mut e, 0, Phase::Play);
    move_card_to_hand(&mut e, 0, cards_13.0);
    move_card_to_hand(&mut e, 0, cards_13.2);
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::ActivateSkill { skill: ziyuan_def }));
    answer_choice(&mut e, vec![ChoiceOption::Card(cards_13.0)]);
    let c_unfiltered = e.open_choice().expect("ziyuan step 2 without fool filter").clone();
    assert!(c_unfiltered.spec.options.contains(&ChoiceOption::Card(cards_13.2)), "BUG-10 counterexample: !fool_skill_enabled keeps c10");
    answer_choice(&mut e, Vec::new());
    e.set_fool_skill_enabled(true);

    // 3. Test minsi "he" (equip + hand) + minsi2 (red exempt from hand-limit/discard, black unlimited distance)
    let mut e4 = Engine::new_with_extensions(607, 4, &[&general_pack]).expect("fool 4-seat");
    apply_fool_to_engine(&raw_cfg, &mut e4).unwrap();
    e4.assign_general_named(0, "wangrong").unwrap();
    step_to_phase(&mut e4, 0, Phase::Play);
    let (eq_c, partner_c) = e4
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find_map(|(ei, ec)| {
            if !ec.off_deck && ec.kind.is_equip() && ec.rank <= 12 {
                let need = 13 - ec.rank;
                e4.catalog()
                    .cards
                    .iter()
                    .enumerate()
                    .find(|(hi, hc)| *hi != ei && !hc.off_deck && hc.rank == need)
                    .map(|(hi, _)| (ei as u32, hi as u32))
            } else {
                None
            }
        })
        .expect("equip + rank complement summing to 13");
    move_card_to_hand(&mut e4, 0, eq_c);
    assert_accepted_or_pending(&submit(&mut e4, 0, GameAction::PlayCard { card: eq_c, targets: vec![] }));
    drain_default(&mut e4);
    assert!(e4.state_core().player(0).equips.contains(&eq_c));
    e4.debug_clear_hand(0);
    move_card_to_hand(&mut e4, 0, partner_c);
    let red_m = e4
        .catalog()
        .cards
        .iter()
        .enumerate()
        .position(|(i, c)| i as u32 != eq_c && i as u32 != partner_c && !c.off_deck && matches!(c.suit, Suit::Heart | Suit::Diamond))
        .unwrap() as u32;
    let blk_sha = e4
        .catalog()
        .cards
        .iter()
        .enumerate()
        .position(|(i, c)| i as u32 != eq_c && i as u32 != partner_c && i as u32 != red_m && !c.off_deck && c.kind == CardKind::Sha && matches!(c.suit, Suit::Spade | Suit::Club))
        .unwrap() as u32;
    let extra_1 = e4.debug_place(0, "sha");
    let extra_2 = e4.debug_place(0, "shan");
    let extra_3 = e4.debug_place(0, "sha");
    let extra_4 = e4.debug_place(0, "shan");
    // Draw order pops from deck end: put blk_sha, red_m on top
    move_card_to_deck_top(&mut e4, blk_sha);
    move_card_to_deck_top(&mut e4, red_m);
    let minsi_def = e4.catalog().skill_by_name("minsi").unwrap();
    assert!(e4.fool_skill_filter_candidates(0, "minsi", &[]).contains(&eq_c));
    assert_accepted_or_pending(&submit(&mut e4, 0, GameAction::ActivateSkill { skill: minsi_def }));
    let mut picked_eq = false;
    for _ in 0..8 {
        let Some(c) = e4.open_choice().cloned() else { break };
        if c.spec.prompt == "minsi:cards" {
            if !picked_eq {
                picked_eq = true;
                assert!(c.spec.options.contains(&ChoiceOption::Card(eq_c)), "minsi allows equip card");
                answer_choice(&mut e4, vec![ChoiceOption::Card(eq_c)]);
            } else {
                answer_choice(&mut e4, vec![ChoiceOption::Card(partner_c)]);
            }
        } else {
            drain_default(&mut e4);
        }
    }
    assert!(!e4.state_core().player(0).equips.contains(&eq_c), "minsi discarded equip");
    assert!(e4.state_core().discard.contains(&eq_c));
    assert!(e4.state_core().player(0).hand.contains(&red_m) && e4.state_core().player(0).hand.contains(&blk_sha));
    // Seat 2 is at distance 2 (beyond range 1 without weapons); black minsi2 Sha ignores distance!
    e4.debug_clear_hand(2);
    let hp2_before = e4.state_core().player(2).hp;
    assert_accepted_or_pending(&submit(&mut e4, 0, GameAction::PlayCard { card: blk_sha, targets: vec![2] }));
    drain_default(&mut e4);
    assert_eq!(e4.state_core().player(2).hp, hp2_before - 1, "BUG-10: black minsi2 Sha ignored distance");
    // Advance to Discard: red_m is exempt from hand limit and excluded from discard choices!
    let _ = (extra_1, extra_2, extra_3, extra_4);
    let _ = submit(&mut e4, 0, GameAction::NextPhase);
    let mut saw_discard = false;
    for _ in 0..6 {
        let Some(c) = e4.open_choice().cloned() else { break };
        saw_discard = true;
        assert!(!c.spec.options.contains(&ChoiceOption::Card(red_m)), "BUG-10: red minsi2 not discardable in DiscardPhase");
        let n = c.spec.min.max(1);
        answer_choice(&mut e4, c.spec.options.iter().take(n).copied().collect());
    }
    assert!(saw_discard, "BUG-10: DiscardPhase prompted even with red minsi2 exempt");
    assert!(e4.state_core().player(0).hand.contains(&red_m), "BUG-10: red minsi2 kept past discard");
    step_to_phase(&mut e4, 1, Phase::Judge);
    assert!(!e4.state_core().vm_aux.flags.contains_key(&(0, format!("q4_minsi2_card:{red_m}"))), "minsi2 cleared at turn end");

    // 4. Test xinfu_fangtong: 2-stage ("he" then fang_pile) + progressive 36 filtering + restore
    e.assign_general_named(0, "re_zhangliang").unwrap();
    let r10 = e.catalog().cards.iter().position(|c| c.rank == 10 && !c.off_deck).unwrap() as u32;
    let r12: Vec<u32> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.rank == 12 && !c.off_deck)
        .map(|(i, _)| i as u32)
        .take(2)
        .collect();
    let r5_bad = e.catalog().cards.iter().enumerate().position(|(i, c)| i as u32 != cards_13.0 && c.rank == 5 && !c.off_deck).unwrap() as u32;
    let r2_ok = e.catalog().cards.iter().position(|c| c.rank == 2 && !c.off_deck).unwrap() as u32;
    e.debug_clear_hand(0);
    move_card_to_hand(&mut e, 0, r10);
    move_card_to_pile(&mut e, 0, "xinfu_jijun", r12[0]);
    move_card_to_pile(&mut e, 0, "xinfu_jijun", r12[1]);
    move_card_to_pile(&mut e, 0, "xinfu_jijun", r5_bad);
    move_card_to_pile(&mut e, 0, "xinfu_jijun", r2_ok);
    assert_eq!(e.recommend_fangtong_combo(0).unwrap().iter().map(|&id| e.catalog().cards[id as usize].rank as u32).sum::<u32>(), 36);
    let hp1_before = e.state_core().player(1).hp;
    e.debug_set_turn(0, Phase::Discard, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mut step_ft = 0u8;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "xinfu_fangtong:he" {
            assert!(!c.spec.options.contains(&ChoiceOption::Card(r12[0])), "fang_pile excluded from :he");
            answer_choice(&mut e, vec![ChoiceOption::Card(r10)]);
        } else if c.spec.prompt == "xinfu_fangtong:cards" {
            assert!(!c.spec.options.contains(&ChoiceOption::Card(r10)), "chosen :he card excluded from :cards");
            if step_ft == 0 {
                step_ft = 1;
                answer_choice(&mut e, vec![ChoiceOption::Card(r12[0]), ChoiceOption::Card(r12[1])]);
            } else {
                step_ft = 2;
                assert!(!c.spec.options.contains(&ChoiceOption::Card(r5_bad)), "BUG-10: sum 34 excludes rank 5 (>36)");
                assert!(c.spec.options.contains(&ChoiceOption::Card(r2_ok)), "BUG-10: rank 2 completes 36");
                answer_choice(&mut e, vec![ChoiceOption::Card(r2_ok)]);
            }
        } else if c.spec.prompt == "xinfu_fangtong:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
        } else {
            drain_default(&mut e);
        }
    }
    assert_eq!(step_ft, 2, "xinfu_fangtong progressive step reached");
    assert_eq!(
        e.state_core().player(1).hp,
        hp1_before - 3,
        "xinfu_fangtong dealt 3 thunder damage"
    );

    let restored = assert_archive_roundtrip(&e, &[&general_pack]);
    assert!(restored.fool_skill_enabled());
    assert_eq!(restored.fool_liegong_style(), FoolLiegongStyle::Shousha);
    let _ = (CardKind::Sha, Suit::Heart);
}
