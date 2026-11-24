use content_native_loader::boss_play_extension_pack;
use engine_native::catalog::{EquipSlot, Suit};
use engine_native::content::standard::BOSS_XIONGSHOU;
use engine_native::state::{BossStageState, GameMode, SkillInstance};
use engine_native::{ChoiceOption, Engine, GameAction, Phase};

use super::helpers::{
    answer_choice, assert_accepted_or_pending, drain_default, drain_one_default,
    move_card_to_hand, step_to_phase, submit,
};

#[test]
fn round2_boss_baonu_tag1_shenqu_immediate_and_bashi_turnoverbefore_cancel() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        7071,
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
    e.assign_general_named(0, "boss_lvbu1").unwrap();
    e.assign_general_named(1, "boss_kumuyuanrang").unwrap();
    e.assign_general_named(2, "boss_yuhuoshiyuan").unwrap();
    e.assign_general_named(3, "刘备").unwrap();

    // BUG-7 (`boss_baonu`): choosing Tag(1) -> boss_lvbu3 immediately enters Phase::Prepare (shenqu prompt opens!)
    e.state_core_mut().vm_aux.flags.insert((0, "q4_jingjia_done".to_string()), 1);
    e.debug_set_hp(0, 4);
    e.debug_clear_hand(0);
    step_to_phase(&mut e, 3, Phase::Play);
    let sha = e.debug_place(3, "sha");
    assert_accepted_or_pending(&submit(
        &mut e,
        3,
        GameAction::PlayCard { card: sha, targets: vec![0] },
    ));
    let mut picked_lvbu3 = false;
    let mut saw_immediate_shenqu = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_baonu:form" {
            picked_lvbu3 = true;
            answer_choice(&mut e, vec![ChoiceOption::Tag(1)]);
        } else if picked_lvbu3 && c.observer == 0 && c.spec.options.contains(&ChoiceOption::Confirm) {
            saw_immediate_shenqu = true;
            assert_eq!(e.turn().current, 0);
            assert_eq!(e.turn().phase, Phase::Prepare);
            assert!(e.turn().turn_opened, "BUG-7: boss_baonu opened Prepare phase immediately");
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(picked_lvbu3 && saw_immediate_shenqu, "BUG-7: Tag(1) fired shenqu Prepare immediately");
    assert!(e.state_core().player(0).hand.len() >= 2, "shenqu Prepare drew 2 cards immediately");

    // BUG-7 (`boss_bashi` counterexample): emits TurnOverBefore, so BOSS_XIONGSHOU cancels face_down flip!
    let xs_def = e.catalog().skill_by_name(BOSS_XIONGSHOU).expect("凶兽 in catalog");
    e.state_core_mut().skills.insert(
        (1, xs_def),
        SkillInstance {
            owner: 1,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    let hp1_before = e.state_core().player(1).hp;
    step_to_phase(&mut e, 0, Phase::Play);
    let sha_bashi = e.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard { card: sha_bashi, targets: vec![1] },
    ));
    let mut saw_bashi = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_bashi:confirm" {
            saw_bashi = true;
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(saw_bashi, "boss_bashi prompted Confirm");
    assert!(!e.state_core().player(1).face_down, "BUG-7: BOSS_XIONGSHOU cancelled TurnOverBefore emitted by boss_bashi");
    assert_eq!(e.state_core().player(1).hp, hp1_before, "target still excluded from Sha");
}

#[test]
fn round2_boss_danjing_lose_hp_before_tao_and_wansha_counterexample() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        7072,
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
    e.assign_general_named(0, "boss_kumuyuanrang").unwrap();
    e.assign_general_named(2, "曹操").unwrap();
    e.assign_general_named(1, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();

    // Counterexample: enemy seat 1 has rewansha during its Play phase -> boss_danjing loses 1 HP,
    // then card_savable_tao returns false so seat 2 is NOT rescued!
    let wansha_def = e.catalog().skill_by_name("rewansha").expect("rewansha in U02 catalog");
    e.state_core_mut().skills.insert(
        (1, wansha_def),
        SkillInstance {
            owner: 1,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    e.state_core_mut().player_mut(0).hp = 2;
    e.state_core_mut().player_mut(2).hp = 1;
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    step_to_phase(&mut e, 1, Phase::Play);
    let sha_w = e.debug_place(1, "sha");
    assert_accepted_or_pending(&submit(
        &mut e,
        1,
        GameAction::PlayCard { card: sha_w, targets: vec![2] },
    ));
    let mut confirmed_dj_blocked = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_danjing:confirm" {
            confirmed_dj_blocked = true;
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(confirmed_dj_blocked);
    assert_eq!(e.state_core().player(0).hp, 1, "BUG-3: boss_danjing still lost 1 HP before cardSavable check");
    assert!(!e.state_core().player(2).alive, "BUG-3 counterexample: wansha blocked virtual Tao rescue");

    // Normal case without wansha: boss_danjing loses 1 HP FIRST then virtual Tao saves ally seat 2!
    let mut e2 = Engine::new_with_extensions_mode(
        7073,
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
    e2.assign_general_named(0, "boss_kumuyuanrang").unwrap();
    e2.assign_general_named(2, "曹操").unwrap();
    e2.assign_general_named(1, "刘备").unwrap();
    e2.assign_general_named(3, "孙权").unwrap();
    e2.state_core_mut().player_mut(0).hp = 3;
    e2.state_core_mut().player_mut(2).hp = 1;
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(2);
    step_to_phase(&mut e2, 1, Phase::Play);
    let sha_ok = e2.debug_place(1, "sha");
    assert_accepted_or_pending(&submit(
        &mut e2,
        1,
        GameAction::PlayCard { card: sha_ok, targets: vec![2] },
    ));
    let mut confirmed_dj_ok = false;
    for _ in 0..8 {
        let Some(c) = e2.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_danjing:confirm" {
            confirmed_dj_ok = true;
            answer_choice(&mut e2, vec![ChoiceOption::Confirm]);
        } else {
            drain_one_default(&mut e2);
        }
    }
    assert!(confirmed_dj_ok);
    assert_eq!(e2.state_core().player(0).hp, 2, "boss_danjing lost 1 HP");
    assert!(e2.state_core().player(2).alive && e2.state_core().player(2).hp == 1, "virtual Tao saved dying ally");
}

#[test]
fn round2_boss_xuanlei_order_lingfeng_mid_dying_and_xuanfeng_jiwu_discard() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = Engine::new_with_extensions_mode(
        7074,
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
    e.assign_general_named(1, "曹操").unwrap();
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();

    // BUG-2 (`boss_xuanlei`): judge on enemies 1 & 3, both hp=1 with Tao -> Dying opens in forward seat order [1, 3]!
    let j1 = e.debug_place(1, "le");
    let j3 = e.debug_place(3, "le");
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.state_core_mut().player_mut(1).judge.push(j1);
    e.state_core_mut().player_mut(3).judge.push(j3);
    e.state_core_mut().player_mut(1).hp = 1;
    e.state_core_mut().player_mut(3).hp = 1;
    let t1 = e.debug_place(1, "tao");
    let t3 = e.debug_place(3, "tao");
    e.debug_set_turn(3, Phase::End, 1);
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let mut xl_order = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(t1)) {
            xl_order.push(1u8);
            answer_choice(&mut e, vec![ChoiceOption::Card(t1)]);
        } else if c.observer == 3 && c.spec.options.contains(&ChoiceOption::Card(t3)) {
            xl_order.push(3u8);
            answer_choice(&mut e, vec![ChoiceOption::Card(t3)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert_eq!(xl_order, vec![1, 3], "BUG-2: boss_xuanlei resolved in forward seat order 1 -> 3");

    // BUG-4 (`boss_lingfeng`): during enemy 1 Dying pause inside LoseHp, p0.hand stays 0 until LoseHp settles!
    e.assign_general_named(0, "boss_liedixuande").unwrap();
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.state_core_mut().player_mut(1).hp = 1;
    let tao1 = e.debug_place(1, "tao");
    let red = e.debug_place_suit(0, Suit::Heart);
    let blk = e.debug_place_suit(0, Suit::Spade);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != red && c != blk);
    e.state_core_mut().deck.push(red);
    e.state_core_mut().deck.push(blk);
    e.debug_set_turn(0, Phase::Judge, 1);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mut saw_mid_dying_zero_hand = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_lingfeng:target" {
            answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(tao1)) {
            assert_eq!(e.state_core().player(0).hand.len(), 0, "BUG-4: p0.hand stays empty during mid-LoseHp Dying");
            saw_mid_dying_zero_hand = true;
            answer_choice(&mut e, vec![ChoiceOption::Card(tao1)]);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(saw_mid_dying_zero_hand, "observed Dying pause during boss_lingfeng LoseHp");
    assert!(e.state_core().player(0).hand.contains(&red) && e.state_core().player(0).hand.contains(&blk));

    // BUG-6 (`olxuanfeng` via equip-loss + non-hostile allowed + 2 interactive :card picks, AND Phase::Discard >=2)
    let mut e3 = Engine::new_with_extensions_mode(
        7075,
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
    e3.assign_general_named(0, "boss_lvbu3").unwrap();
    let xf_def = e3.catalog().skill_by_name("olxuanfeng").expect("olxuanfeng in U02 catalog");
    e3.state_core_mut().skills.insert(
        (0, xf_def),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    let weapons: Vec<u32> = e3
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(i, c)| !c.off_deck && e3.catalog().card_slot_of(e3.state_core(), *i as u32) == Some(EquipSlot::Weapon))
        .map(|(i, _)| i as u32)
        .take(3)
        .collect();
    assert!(weapons.len() >= 3);
    step_to_phase(&mut e3, 0, Phase::Play);
    move_card_to_hand(&mut e3, 0, weapons[0]);
    move_card_to_hand(&mut e3, 0, weapons[1]);
    move_card_to_hand(&mut e3, 1, weapons[2]);
    let f2_card = e3.debug_place(2, "shan");
    assert_accepted_or_pending(&submit(&mut e3, 0, GameAction::PlayCard { card: weapons[0], targets: vec![] }));
    drain_default(&mut e3);
    // Replace equipped weapon with weapons[1] -> triggers xuanfeng via CardsLost(Equip)!
    assert_accepted_or_pending(&submit(&mut e3, 0, GameAction::PlayCard { card: weapons[1], targets: vec![] }));
    let mut xf_step = 0u8;
    for _ in 0..12 {
        let Some(c) = e3.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e3, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt == "xuanfeng:target" {
            assert!(c.spec.options.contains(&ChoiceOption::Player(2)), "BUG-6: fellow seat 2 is legal xuanfeng target");
            if xf_step == 0 {
                xf_step = 1;
                answer_choice(&mut e3, vec![ChoiceOption::Player(1)]);
            } else {
                xf_step = 2;
                answer_choice(&mut e3, vec![ChoiceOption::Player(2)]);
            }
        } else if c.spec.prompt == "xuanfeng:card" {
            assert!(!c.spec.allow_cancel, "BUG-6: xuanfeng:card pick is mandatory");
            if c.spec.options.contains(&ChoiceOption::Card(weapons[2])) {
                answer_choice(&mut e3, vec![ChoiceOption::Card(weapons[2])]);
            } else {
                answer_choice(&mut e3, vec![ChoiceOption::Card(f2_card)]);
            }
        } else {
            drain_one_default(&mut e3);
        }
    }
    assert_eq!(xf_step, 2, "BUG-6: xuanfeng picked 2 targets and cards interactively on equip loss");
    assert!(!e3.state_core().player(1).hand.contains(&weapons[2]) && !e3.state_core().player(2).hand.contains(&f2_card));

    // BUG-6b: jiwu discarding equip emits CardsLost(Equip) -> triggers olxuanfeng, AND Phase::Discard >=2 triggers olxuanfeng!
    let jiwu_def = e3.catalog().skill_by_name("jiwu").expect("jiwu present");
    let wansha_def = e3.catalog().skill_by_name("rewansha").expect("rewansha present");
    move_card_to_hand(&mut e3, 0, weapons[0]);
    assert_accepted_or_pending(&submit(&mut e3, 0, GameAction::PlayCard { card: weapons[0], targets: vec![] }));
    drain_default(&mut e3);
    let pick_c = e3.debug_place(1, "sha");
    assert_accepted_or_pending(&submit(&mut e3, 0, GameAction::ActivateSkill { skill: jiwu_def }));
    let mut saw_xf_after_jiwu_eq = false;
    for _ in 0..10 {
        let Some(c) = e3.open_choice().cloned() else { break };
        if c.spec.prompt == "jiwu:card" {
            assert!(c.spec.options.contains(&ChoiceOption::Card(weapons[0])));
            answer_choice(&mut e3, vec![ChoiceOption::Card(weapons[0])]);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e3, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt == "xuanfeng:target" {
            if !saw_xf_after_jiwu_eq {
                saw_xf_after_jiwu_eq = true;
                answer_choice(&mut e3, vec![ChoiceOption::Player(1)]);
            } else {
                answer_choice(&mut e3, Vec::new());
            }
        } else if c.spec.prompt == "xuanfeng:card" {
            let pick = c
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(id) if *id == pick_c))
                .copied()
                .unwrap_or(c.spec.options[0]);
            answer_choice(&mut e3, vec![pick]);
        } else if c.spec.prompt == "jiwu:skill" {
            answer_choice(&mut e3, vec![ChoiceOption::Skill(wansha_def)]);
        } else {
            drain_one_default(&mut e3);
        }
    }
    assert!(saw_xf_after_jiwu_eq, "BUG-6: jiwu equip discard emitted CardsLost(Equip) triggering xuanfeng");
    assert!(!e3.state_core().player(1).hand.contains(&pick_c));

    // Phase::Discard >= 2 triggers xuanfeng
    e3.debug_set_hp(0, 1);
    e3.debug_clear_hand(0);
    let _d1 = e3.debug_place(0, "sha");
    let _d2 = e3.debug_place(0, "shan");
    let _d3 = e3.debug_place(0, "sha");
    let c_disc_tgt = e3.debug_place(1, "shan");
    let _ = submit(&mut e3, 0, GameAction::NextPhase);
    let mut saw_xf_discard_phase = false;
    for _ in 0..16 {
        let Some(c) = e3.open_choice().cloned() else {
            if e3.turn().current == 0 && e3.turn().phase == Phase::Discard {
                let _ = submit(&mut e3, 0, GameAction::NextPhase);
                continue;
            }
            break;
        };
        if c.spec.prompt == "discard_to_limit" {
            let picks: Vec<ChoiceOption> = c.spec.options.iter().take(c.spec.min).copied().collect();
            answer_choice(&mut e3, picks);
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e3, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt == "xuanfeng:target" {
            if !saw_xf_discard_phase {
                saw_xf_discard_phase = true;
                answer_choice(&mut e3, vec![ChoiceOption::Player(1)]);
            } else {
                answer_choice(&mut e3, Vec::new());
            }
        } else if c.spec.prompt == "xuanfeng:card" {
            let pick = c
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Card(id) if *id == c_disc_tgt))
                .copied()
                .unwrap_or(c.spec.options[0]);
            answer_choice(&mut e3, vec![pick]);
        } else {
            drain_one_default(&mut e3);
        }
    }
    assert!(saw_xf_discard_phase, "BUG-6: Phase::Discard >=2 triggered xuanfeng");
    assert!(!e3.state_core().player(1).hand.contains(&c_disc_tgt));
}
