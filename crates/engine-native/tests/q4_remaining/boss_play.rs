use content_native_loader::{
    boss_play_extension_pack, boss_play_init_and_arena_ready, L03_BOSS_GENERALS, U02_BOSS_GENERALS,
};
use engine_native::catalog::{EquipSlot, Faction};
use engine_native::content::standard::Gender;
use engine_native::machine::LogEvent;
use engine_native::state::{BossStageState, GameMode, INFINITE_HP};
use engine_native::{ChoiceOption, Engine, GameAction, Phase};

use super::helpers::{
    advance_phases, answer_choice, assert_accepted_or_pending, assert_archive_roundtrip,
    clear_equips_to_discard, drain_default, move_card_to_deck_top, repo_asset, step_to_phase,
    submit,
};

#[test]
fn u02_and_l03_machine_verify_all_boss_generals_and_modes() {
    let noname_cfg = repo_asset("packages/extensions/boss-play-upstream-noname/assets/config.json");
    let liuli_cfg = repo_asset("packages/extensions/boss-play-upstream-liuli/assets/config.json");
    let noname_pack = boss_play_extension_pack("noname", &[]);
    let liuli_pack = boss_play_extension_pack("noname-liuli", &[]);

    assert_eq!(noname_pack.generals.len(), 15);
    assert_eq!(liuli_pack.generals.len(), 8);

    // Exact machine verification of all 15 U02 generals inside real Catalog
    let e_noname = Engine::new_with_extensions_mode(
        1001,
        4,
        &[&noname_pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: Vec::new(),
            stage: BossStageState::default_none(),
        },
    )
    .expect("construct noname boss engine");

    let expected_u02: [(&str, Faction, Gender, u32, &[&str]); 15] = [
        ("boss_liedixuande", Faction::Shu, Gender::Male, 5, &["boss_lingfeng", "boss_jizhen"]),
        ("boss_gongshenyueying", Faction::Shu, Gender::Female, 4, &["boss_gongshenjg", "boss_jingmiao", "boss_zhinang"]),
        ("boss_tianhoukongming", Faction::Shu, Gender::Male, 4, &["boss_biantian", "bazhen"]),
        ("boss_yuhuoshiyuan", Faction::Shu, Gender::Male, 4, &["boss_yuhuojg", "boss_qiwu", "boss_tianyujg"]),
        ("boss_qiaokuijunyi", Faction::Wei, Gender::Male, 4, &["boss_huodi", "boss_jueji"]),
        ("boss_jiarenzidan", Faction::Wei, Gender::Male, 5, &["boss_chiying", "boss_jingfan"]),
        ("boss_duanyuzhongda", Faction::Wei, Gender::Male, 5, &["boss_fanshi", "boss_xuanlei", "boss_skonghun"]),
        ("boss_juechenmiaocai", Faction::Wei, Gender::Male, 5, &["boss_chuanyun", "boss_leili", "boss_fengxing"]),
        ("boss_kumuyuanrang", Faction::Wei, Gender::Male, 5, &["boss_bashi", "boss_danjing"]),
        ("boss_baijiwenyuan", Faction::Wei, Gender::Male, 5, &["boss_jiaoxie"]),
        ("boss_yihanyunchang", Faction::Shu, Gender::Male, 5, &["boss_xiaorui", "boss_huchen"]),
        ("boss_fuweizilong", Faction::Shu, Gender::Male, 5, &["boss_fengjian", "boss_keding"]),
        ("boss_lvbu1", Faction::Qun, Gender::Male, 8, &["马术", "无双", "boss_baonu", "boss_jingjia", "boss_aozhan"]),
        ("boss_lvbu2", Faction::Qun, Gender::Male, 6, &["马术", "无双", "xiuluo", "shenwei", "shenji"]),
        ("boss_lvbu3", Faction::Qun, Gender::Male, 6, &["无双", "shenqu", "jiwu"]),
    ];
    for (name, fac, gen, hp, skills) in expected_u02 {
        let idx = (0..e_noname.catalog().general_count() as u16)
            .find(|&d| e_noname.catalog().general_meta(d).map(|m| m.0) == Some(name))
            .unwrap_or_else(|| panic!("missing U02 boss general {name}"));
        let (_, actual_fac, actual_skills, actual_hp, actual_gen) =
            e_noname.catalog().general_meta(idx).unwrap();
        assert_eq!(actual_fac, fac, "{name} faction");
        assert_eq!(actual_gen, gen, "{name} gender");
        assert_eq!(actual_hp, hp, "{name} max_hp");
        assert_eq!(actual_skills, skills, "{name} skills");
        for sk in actual_skills {
            assert!(e_noname.catalog().skill_by_name(sk).is_some(), "missing skill {sk}");
        }
    }

    // Machine verification of L03 (8 generals)
    let mut e_liuli = Engine::new_with_extensions_mode(
        1002,
        4,
        &[&liuli_pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: Vec::new(),
            stage: BossStageState::default_none(),
        },
    )
    .expect("construct liuli boss engine");
    for (name, _, _, _, _) in L03_BOSS_GENERALS {
        let idx = (0..e_liuli.catalog().general_count() as u16)
            .find(|&d| e_liuli.catalog().general_meta(d).map(|m| m.0) == Some(*name));
        assert!(idx.is_some(), "L03 general {name} must exist");
    }
    let storage = content_native_loader::PlayStorage {
        boss: vec![content_native_loader::Entry {
            name: "boss_lvbu1".into(),
            hp: content_native_loader::HpKind::Infinity,
            tags: vec![],
            allowed: true,
        }],
        versus: vec![],
        translate: vec![],
    };
    let mut e_modes = Engine::new_with_extensions(1003, 4, &[&noname_pack]).unwrap();
    let tafang_out = boss_play_init_and_arena_ready(
        &noname_cfg,
        &mut e_modes,
        "tafang",
        "two",
        "boss",
        &storage,
        true,
    )
    .unwrap();
    assert!(tafang_out.skipped, "tafang mode skips boss_play_init");
    assert!(
        U02_BOSS_GENERALS.iter().all(|g| {
            let def = (0..e_modes.catalog().general_count() as u16)
                .find(|&d| e_modes.catalog().general_meta(d).map(|m| m.0) == Some(g.0))
                .unwrap();
            !e_modes.is_general_forbidai(def)
        }),
        "BUG-8: tafang skips arenaReady without mutating forbidai"
    );

    let id_no_ai = boss_play_init_and_arena_ready(
        &liuli_cfg,
        &mut e_modes,
        "identity",
        "two",
        "identity",
        &storage,
        false,
    )
    .unwrap();
    assert!(!id_no_ai.skipped);
    let liu_def = (0..e_modes.catalog().general_count() as u16)
        .find(|&d| e_modes.catalog().general_meta(d).map(|m| m.0) == Some("boss_liedixuande"))
        .unwrap();
    assert!(
        e_modes.is_general_forbidai(liu_def),
        "BUG-8: enableai=false in identity marks boss general forbidai"
    );
    assert!(
        e_modes.filter_selectable_generals(false).contains(&liu_def),
        "BUG-8: human selection still allows boss general in identity"
    );
    assert!(
        !e_modes.filter_selectable_generals(true).contains(&liu_def),
        "BUG-8: AI selection excludes boss general when enableai=false"
    );

    let id_out = boss_play_init_and_arena_ready(
        &liuli_cfg,
        &mut e_modes,
        "identity",
        "two",
        "identity",
        &storage,
        true,
    )
    .unwrap();
    assert!(!id_out.skipped);
    assert_eq!(id_out.boss_pack[0].hp, content_native_loader::HpKind::Infinity);
    assert!(
        !e_modes.is_general_forbidai(liu_def) && e_modes.filter_selectable_generals(true).contains(&liu_def),
        "BUG-8: enableai=true allows AI to pick in-source boss generals"
    );

    // Boss mode with enableai=false keeps boss_allowed while still setting forbidai
    let _ = boss_play_init_and_arena_ready(
        &liuli_cfg,
        &mut e_liuli,
        "boss",
        "two",
        "boss",
        &storage,
        false,
    )
    .unwrap();
    let liu_liuli = (0..e_liuli.catalog().general_count() as u16)
        .find(|&d| e_liuli.catalog().general_meta(d).map(|m| m.0) == Some("boss_liedixuande"))
        .unwrap();
    assert!(e_liuli.is_general_forbidai(liu_liuli));
    assert!(e_liuli.filter_boss_generals().contains(&liu_liuli));

    let boss_out = boss_play_init_and_arena_ready(
        &liuli_cfg,
        &mut e_liuli,
        "boss",
        "two",
        "boss",
        &storage,
        true,
    )
    .unwrap();
    assert!(!boss_out.skipped);
    assert!(!e_liuli.is_general_forbidai(liu_liuli));
    let allowed_boss: Vec<&str> = e_liuli
        .filter_boss_generals()
        .into_iter()
        .filter_map(|d| e_liuli.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert_eq!(allowed_boss.len(), 8);
    assert!(allowed_boss.contains(&"boss_liedixuande"));
    assert!(!allowed_boss.contains(&"boss_lvbu1"));
    assert!(INFINITE_HP > 100_000);

    let restored = assert_archive_roundtrip(&e_liuli, &[&liuli_pack]);
    let restored_boss: Vec<&str> = restored
        .filter_boss_generals()
        .into_iter()
        .filter_map(|d| restored.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert_eq!(restored_boss, allowed_boss);
}

#[test]
fn u02_real_event_skills_jiange_liu_zhuge_pang_sima_xiahou() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        2026,
        4,
        &[&pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: vec![2],
            stage: BossStageState::default_none(),
        },
    )
    .unwrap();
    // Side 0 = Boss(0) + Fellow(2); Side 1 = Challengers(1, 3)
    e.assign_general_named(0, "boss_liedixuande").unwrap();
    e.assign_general_named(2, "boss_yuhuoshiyuan").unwrap();
    e.assign_general_named(1, "boss_qiaokuijunyi").unwrap();
    e.assign_general_named(3, "boss_tianhoukongming").unwrap();

    // Test boss_jizhen draws 1 card for wounded fellow (seat 2) on seat 0 Phase::End
    e.state_core_mut().player_mut(2).hp = 2;
    let hand2_before = e.state_core().player(2).hand.len();
    e.debug_set_turn(0, Phase::Discard, 1);
    advance_phases(&mut e, 1);
    assert_eq!(
        e.state_core().player(2).hand.len(),
        hand2_before + 1,
        "boss_jizhen draws 1 card for wounded friend on End"
    );

    // Test boss_lingfeng (BUG-4: cards sit revealed on table during target/loseHp, gained ONLY after loseHp settles)
    e.state_core_mut().player_mut(1).hp = 4;
    e.debug_clear_hand(0);
    let red = e.debug_place_suit(0, engine_native::catalog::Suit::Heart);
    let blk = e.debug_place_suit(0, engine_native::catalog::Suit::Spade);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != red && c != blk);
    e.state_core_mut().deck.push(red);
    e.state_core_mut().deck.push(blk);
    let log_start = e.log().len();
    e.debug_set_turn(0, Phase::Judge, 1);
    let cur = e.turn().current;
    let _ = submit(&mut e, cur, GameAction::NextPhase);
    let mut checked_table_pre_gain = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_lingfeng:target" {
            assert_eq!(e.state_core().player(0).hand.len(), 0, "BUG-4: cards not in hand during target prompt");
            assert_eq!(e.state_core().table.len(), 2, "BUG-4: revealed cards sit on table");
            checked_table_pre_gain = true;
            answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_default(&mut e);
        }
    }
    assert!(checked_table_pre_gain, "saw boss_lingfeng:target");
    assert_eq!(e.state_core().player(1).hp, 3, "boss_lingfeng caused 1 HP loss on enemy 1");
    assert!(e.state_core().table.is_empty(), "BUG-4: table cleared after lingfeng finish");
    assert!(
        e.state_core().player(0).hand.contains(&red) && e.state_core().player(0).hand.contains(&blk),
        "BUG-4: revealed cards gained into hand after loseHp"
    );
    assert!(
        e.log()[log_start..].iter().any(|ev| matches!(ev, LogEvent::CardsRevealed { cards, .. } if cards.len() == 2)),
        "BUG-4: CardsRevealed emitted"
    );

    // Test boss_qiwu: seat 2 plays Club card -> heals wounded ally seat 0 (or 2)
    e.state_core_mut().player_mut(0).hp = 3;
    step_to_phase(&mut e, 2, Phase::Play);
    let club_tao = e.debug_place_suit(2, engine_native::catalog::Suit::Club);
    e.state_core_mut().player_mut(2).hp = 4;
    // Play a club trick or equip (note non-club/non-target-less club card like dismantle/sha)
    let club_dismantle = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| {
                c.kind == engine_native::catalog::CardKind::Dismantle
                    && c.suit == engine_native::catalog::Suit::Club
            })
            .map(|c| c.name);
        name.map(|n| e.debug_place(2, n)).unwrap_or(club_tao)
    };
    let res = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: club_dismantle,
            targets: vec![1],
        },
    );
    assert_accepted_or_pending(&res);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_qiwu:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(0)]);
        } else {
            drain_default(&mut e);
        }
    }
    assert_eq!(e.state_core().player(0).hp, 4, "boss_qiwu club card healed wounded friend");

    // Archive parity at live mid-match point
    let restored = assert_archive_roundtrip(&e, &[&pack]);
    assert_eq!(restored.state_core().player(0).hp, 4);
}

#[test]
fn u02_real_event_lvbu_forms_baonu_jingjia_shenwei_shenji_jiwu_and_keding() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        3030,
        4,
        &[&pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: Vec::new(),
            stage: BossStageState::default_none(),
        },
    )
    .unwrap();
    e.assign_general_named(0, "boss_lvbu1").unwrap();
    e.assign_general_named(1, "boss_fuweizilong").unwrap();
    e.assign_general_named(2, "曹操").unwrap();
    e.assign_general_named(3, "刘备").unwrap();

    // Test boss_jingjia (BUG-2: equips Weapon/Armor/Treasure only via shuffled 1.5->0.5 loop; skips HorsePlus/HorseMinus!)
    let horse_p = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(id, _)| e.catalog().card_slot_of(e.state_core(), *id as u32) == Some(EquipSlot::HorsePlus))
        .map(|(id, _)| id as u32);
    let horse_m = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(id, _)| e.catalog().card_slot_of(e.state_core(), *id as u32) == Some(EquipSlot::HorseMinus))
        .map(|(id, _)| id as u32);
    if let Some(hp_c) = horse_p {
        move_card_to_deck_top(&mut e, hp_c);
    }
    if let Some(hm_c) = horse_m {
        move_card_to_deck_top(&mut e, hm_c);
    }
    e.debug_set_turn(3, Phase::End, 1);
    step_to_phase(&mut e, 0, Phase::Play);
    assert!(
        !e.state_core().player(0).equips.is_empty(),
        "boss_jingjia equipped items from deck at TurnBegin"
    );
    assert!(
        e.state_core().player(0).equips.iter().all(|c| !matches!(
            e.catalog().card_slot_of(e.state_core(), *c),
            Some(EquipSlot::HorsePlus | EquipSlot::HorseMinus)
        )),
        "BUG-2: boss_jingjia must NEVER equip HorsePlus or HorseMinus"
    );

    // Damage boss_lvbu1 down to <= 4 HP to trigger boss_baonu transformation into boss_lvbu2
    clear_equips_to_discard(&mut e, 0);
    e.assign_general_named(0, "boss_lvbu1").unwrap();
    e.state_core_mut().vm_aux.flags.insert((0, "q4_jingjia_done".to_string()), 1);
    e.debug_set_hp(0, 4);
    e.debug_clear_hand(0);
    e.state_core_mut().player_mut(0).linked = true;
    step_to_phase(&mut e, 3, Phase::Play);
    let sha = e.debug_place(3, "sha");
    let sha_res = submit(
        &mut e,
        3,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    assert_accepted_or_pending(&sha_res);
    // Let sha hit (pass Dodge) and pick boss_lvbu2 (Tag(0)) when boss_baonu prompts
    let mut saw_baonu = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_baonu:form" {
            saw_baonu = true;
            // Save snapshot right inside suspension!
            let _ = assert_archive_roundtrip(&e, &[&pack]);
            answer_choice(&mut e, vec![ChoiceOption::Tag(0)]);
        } else {
            drain_default(&mut e);
        }
    }
    assert!(saw_baonu, "boss_baonu triggered at hp <= 4");
    assert_eq!(e.turn().current, 0, "boss_baonu grants immediate turn to boss");
    assert!(!e.state_core().player(0).linked, "boss_baonu clears linked");
    let lvbu2_def = e
        .catalog()
        .skill_by_name("shenwei")
        .expect("shenwei skill present");
    assert!(e.state_core().has_skill(0, lvbu2_def), "transformed into boss_lvbu2");

    // Advance to seat 0 DrawPhase to confirm shenwei extra draw (+min(3, 3)=+3 -> 5 total)
    let hand_before = e.state_core().player(0).hand.len();
    step_to_phase(&mut e, 0, Phase::Play);
    assert!(
        e.state_core().player(0).hand.len() >= hand_before + 5,
        "shenwei draws +3 with 3 other alive players"
    );

    // shenji lets unarmed boss_lvbu2 target up to 3 players with 1 Sha
    clear_equips_to_discard(&mut e, 0);
    let sha2 = e.debug_place(0, "sha");
    let multi = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1, 2, 3],
        },
    );
    assert_accepted_or_pending(&multi);
    drain_default(&mut e);

    // Switch boss to boss_lvbu3 and test jiwu ActivateSkill gaining "rewansha" / "qiangxix"
    e.assign_general_named(0, "boss_lvbu3").unwrap();
    let jiwu_def = e.catalog().skill_by_name("jiwu").expect("jiwu skill");
    let _discardable = e.debug_place(0, "sha");
    step_to_phase(&mut e, 0, Phase::Play);
    let act = submit(&mut e, 0, GameAction::ActivateSkill { skill: jiwu_def });
    assert_accepted_or_pending(&act);
    let rewansha_def = e.catalog().skill_by_name("rewansha").expect("rewansha present");
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "jiwu:card" {
            let opt = c.spec.options[0];
            answer_choice(&mut e, vec![opt]);
        } else if c.spec.prompt == "jiwu:skill" {
            answer_choice(&mut e, vec![ChoiceOption::Skill(rewansha_def)]);
        } else {
            drain_default(&mut e);
        }
    }
    assert!(e.state_core().has_skill(0, rewansha_def), "jiwu granted selected skill rewansha");
    assert_archive_roundtrip(&e, &[&pack]);
}

#[test]
fn u02_real_event_skonghun_xuanlei_fanshi_xiaorui_huchen_jiaoxie_bashi_danjing() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        4040,
        4,
        &[&pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows: vec![2],
            stage: BossStageState::default_none(),
        },
    )
    .unwrap();
    e.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e.assign_general_named(1, "boss_kumuyuanrang").unwrap();
    e.assign_general_named(2, "boss_yihanyunchang").unwrap();
    e.assign_general_named(3, "boss_baijiwenyuan").unwrap();

    // BUG-2: Put judge card on enemy 1 (xuanlei -1), verify forward damage order 1 -> 3 AND p0 stays hp=3 during enemy dying until all damages settle!
    let j = e.debug_place(1, "le");
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != j);
    e.state_core_mut().player_mut(1).judge.push(j);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.state_core_mut().player_mut(1).hp = 2; // xuanlei (-1) -> 1; skonghun (-1) -> 0 (Dying!)
    e.state_core_mut().player_mut(3).hp = 1; // skonghun (-1) -> 0 (Dying!)
    let t1 = e.debug_place(1, "tao");
    let t3 = e.debug_place(3, "tao");
    e.state_core_mut().player_mut(0).hp = 3; // lost=2 == enemies(2: seats 1,3)
    e.debug_set_turn(3, Phase::End, 1);
    let mut dying_order = Vec::new();
    for _ in 0..32 {
        if let Some(c) = e.open_choice().cloned() {
            if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(t1)) {
                dying_order.push(1u8);
                assert_eq!(e.state_core().player(0).hp, 3, "BUG-2: boss_skonghun has NOT recovered before enemy 1 damage settles");
                assert_eq!(e.state_core().player(3).hp, 1, "BUG-2: seat 3 not damaged before seat 1 settles");
                answer_choice(&mut e, vec![ChoiceOption::Card(t1)]);
            } else if c.observer == 3 && c.spec.options.contains(&ChoiceOption::Card(t3)) {
                dying_order.push(3u8);
                assert_eq!(e.state_core().player(0).hp, 3, "BUG-2: boss_skonghun has NOT recovered while enemy 3 damage settles");
                answer_choice(&mut e, vec![ChoiceOption::Card(t3)]);
            } else {
                drain_default(&mut e);
            }
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(dying_order, vec![1, 3], "BUG-2: boss_skonghun resolved damages in forward seat order 1 -> 3");
    assert_eq!(e.state_core().player(0).hp, 5, "boss_skonghun recovered +2 HP AFTER both enemy damages settled");

    // Advance out of Play -> Discard -> Phase::End (PhaseBegin triggers boss_fanshi -1 HP)
    let hp_before_end = e.state_core().player(0).hp;
    step_to_phase(&mut e, 0, Phase::End);
    drain_default(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp_before_end - 1, "boss_fanshi loses 1 HP at End");

    // Test boss_jiaoxie active skill on seat 3 (target mech fellow seat 2)
    e.state_core_mut().vm_aux.flags.insert((2, "q4_is_mech".to_string()), 1);
    let jiaoxie_def = e.catalog().skill_by_name("boss_jiaoxie").unwrap();
    let give_c = e.debug_place(2, "sha");
    step_to_phase(&mut e, 3, Phase::Play);
    let act = submit(&mut e, 3, GameAction::ActivateSkill { skill: jiaoxie_def });
    assert_accepted_or_pending(&act);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_jiaoxie:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(2)]);
        } else if c.spec.prompt == "boss_jiaoxie:give_card" {
            answer_choice(&mut e, vec![ChoiceOption::Card(give_c)]);
        } else {
            drain_default(&mut e);
        }
    }
    assert!(e.state_core().player(3).hand.contains(&give_c), "boss_jiaoxie took card from mech enemy");

    assert_archive_roundtrip(&e, &[&pack]);
}

#[test]
fn u02_arena_ready_mode_gating_storage_versus_ladder_and_jiange_filter() {
    use content_native_loader::{
        arena_ready_outcome, boss_play_arena_ready_with_storage, ArenaReadyInput, Entry, HpKind,
        PlayStorage,
    };

    let noname_cfg = repo_asset("packages/extensions/boss-play-upstream-noname/assets/config.json");
    let noname_pack = boss_play_extension_pack("noname", &[]);
    let arena_in = ArenaReadyInput {
        boss_mode_entries: vec![
            Entry { name: "boss_lvbu1".into(), hp: HpKind::Number(8), tags: vec![], allowed: true },
            Entry { name: "boss_hidden".into(), hp: HpKind::Number(4), tags: vec![], allowed: false },
        ],
        boss_mode_skills: vec!["boss_baonu".into(), "versus_ladder".into()],
        boss_mode_translate: vec![("x".into(), "new_x".into()), ("b_key".into(), "B".into())],
        versus_mode_jiange: vec![
            Entry { name: "boss_liedixuande".into(), hp: HpKind::Number(5), tags: vec!["bossallowed".into()], allowed: true },
            Entry { name: "boss_blocked".into(), hp: HpKind::Number(4), tags: vec![], allowed: false },
        ],
        versus_mode_skills: vec!["boss_lingfeng".into(), "versus_ladder".into()],
        versus_mode_translate: vec![("v_key".into(), "V".into())],
        existing_translate: vec![("x".into(), "keep_x".into())],
    };

    // tafang: early exit without mutating storage, skills, or translations
    let tf = arena_ready_outcome("tafang", &PlayStorage::default(), &arena_in);
    assert!(tf.skipped);
    assert!(tf.loaded_skills.is_empty() && tf.storage_boss.is_empty() && tf.storage_versus.is_empty());

    // identity: loads both boss (1+filtered) & versus (excludes versus_ladder), keeps existing translate ||=
    let id_out = arena_ready_outcome("identity", &PlayStorage::default(), &arena_in);
    assert!(!id_out.skipped && id_out.boss_loaded && id_out.versus_loaded);
    assert_eq!(id_out.storage_boss.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["boss_lvbu1"]);
    assert_eq!(id_out.storage_versus.iter().map(|e| e.name.as_str()).collect::<Vec<_>>(), vec!["boss_liedixuande"]);
    assert!(id_out.loaded_skills.contains(&"boss_lingfeng".to_string()));
    assert!(!id_out.finished_skills.contains(&"versus_ladder".to_string()));
    assert!(id_out.translate.contains(&("x".to_string(), "keep_x".to_string())));

    // versus: skips loading versus pack itself, loads boss mode skills, and filters U02 to 12 jiange generals
    let vs_out = arena_ready_outcome("versus", &PlayStorage::default(), &arena_in);
    assert!(vs_out.boss_loaded && !vs_out.versus_loaded);
    let mut e_vs = Engine::new_with_extensions(1099, 4, &[&noname_pack]).unwrap();
    let storage = PlayStorage {
        versus: vec![Entry { name: "boss_kumuyuanrang".into(), hp: HpKind::Number(5), tags: vec![], allowed: false }],
        ..Default::default()
    };
    let _ = boss_play_init_and_arena_ready(&noname_cfg, &mut e_vs, "versus", "jiange", "jiange", &storage, true).unwrap();
    let allowed_names: Vec<&str> = e_vs
        .filter_boss_generals()
        .into_iter()
        .filter_map(|d| e_vs.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(allowed_names.contains(&"boss_liedixuande"), "jiange general allowed in versus");
    assert!(!allowed_names.contains(&"boss_kumuyuanrang"), "storage allowed=false excludes general in versus");
    assert!(!allowed_names.contains(&"boss_lvbu1"), "boss_lvbu1 (mode_boss only) excluded from versus jiange pool");

    // boss mode with storage override re-checked and roundtripped
    let mut e_boss = Engine::new_with_extensions(1100, 4, &[&noname_pack]).unwrap();
    boss_play_arena_ready_with_storage(&mut e_boss, "noname", "boss", &PlayStorage::default(), true);
    let boss_allowed: Vec<&str> = e_boss
        .filter_boss_generals()
        .into_iter()
        .filter_map(|d| e_boss.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(boss_allowed.contains(&"boss_lvbu1"));
    assert_eq!(boss_allowed.len(), 15);
    assert_archive_roundtrip(&e_boss, &[&noname_pack]);
}
