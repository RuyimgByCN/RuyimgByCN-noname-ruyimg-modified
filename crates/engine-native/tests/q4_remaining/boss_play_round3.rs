use std::collections::BTreeMap;

use content_native_loader::{
    apply_douzhuan_request_to_engine, apply_douzhuan_to_engine, apply_shousha_to_engine,
    boss_play_extension_pack,
};
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, EquipSlot, Faction, Suit};
use engine_native::content::extension::{ExtensionDef, ExtensionGeneral, ExtensionPack, InitHook};
use engine_native::content::standard::Gender;
use engine_native::state::{BossStageState, GameMode, GameStateCore, SkillInstance, Winner};
use engine_native::view::{ChoiceView, PlayerView};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, Phase};
use serde_json::Value;

use super::helpers::{
    answer_choice, assert_accepted_or_pending, assert_archive_roundtrip, drain_default,
    drain_one_default, move_card_to_deck_top, move_card_to_hand, repo_asset, step_to_phase,
    submit,
};

fn total_cards_accounting(core: &GameStateCore) -> usize {
    let zones: usize = core
        .players
        .iter()
        .map(|p| p.hand.len() + p.equips.len() + p.judge.len() + p.off_court.len())
        .sum();
    core.deck.len() + core.discard.len() + core.table.len() + core.detached_cards.len() + zones
}

fn find_off_deck_or_deck_card_matching<F>(e: &Engine, pred: F) -> u32
where
    F: Fn(u32, &engine_native::catalog::CardDef) -> bool,
{
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(i, d)| !d.off_deck && pred(*i as u32, d))
        .map(|(i, _)| i as u32)
        .expect("matching card in catalog")
}

fn new_boss_4p(seed: u64, pack: &ExtensionPack, fellows: Vec<u8>) -> Engine {
    Engine::new_with_extensions_mode(
        seed,
        4,
        &[pack],
        GameMode::Boss {
            boss: 0,
            chongzheng: 0,
            fellows,
            stage: BossStageState::default_none(),
        },
    )
    .unwrap()
}

#[test]
fn round3_jiwu_u02_l03_variants_and_qiangxix_full_parity() {
    let u02 = boss_play_extension_pack("noname", &[]);
    let l03 = boss_play_extension_pack("noname-liuli", &[]);

    // 1) U02 registers ["qiangxix", "retieji", "olxuanfeng", "rewansha"] and jiwu offers them
    let e_u02 = new_boss_4p(7101, &u02, vec![]);
    for sk in ["qiangxix", "retieji", "olxuanfeng", "rewansha"] {
        assert!(
            e_u02.catalog().skill_by_name(sk).is_some(),
            "U02 missing jiwu skill {sk}"
        );
    }
    assert!(
        e_u02.catalog().skill_by_name("rexuanfeng").is_none(),
        "U02 must NOT register L03 rexuanfeng"
    );

    // 2) L03 registers ["qiangxix", "retieji", "rexuanfeng", "wansha"]
    let mut e = new_boss_4p(7102, &l03, vec![]);
    for sk in ["qiangxix", "retieji", "rexuanfeng", "wansha"] {
        assert!(
            e.catalog().skill_by_name(sk).is_some(),
            "L03 missing jiwu skill {sk}"
        );
    }
    assert!(
        e.catalog().skill_by_name("olxuanfeng").is_none(),
        "L03 must NOT register U02 olxuanfeng"
    );

    // Verify U02 jiwu:skill offers ["qiangxix", "retieji", "olxuanfeng", "rewansha"]
    let mut e_jw = new_boss_4p(7101, &u02, vec![]);
    e_jw.assign_general_named(0, "boss_lvbu3").unwrap();
    e_jw.assign_general_named(1, "曹操").unwrap();
    e_jw.assign_general_named(2, "刘备").unwrap();
    e_jw.assign_general_named(3, "孙权").unwrap();
    step_to_phase(&mut e_jw, 0, Phase::Play);
    let jiwu_def = e_jw.catalog().skill_by_name("jiwu").unwrap();
    let u02_qxx = e_jw.catalog().skill_by_name("qiangxix").unwrap();
    let u02_rtj = e_jw.catalog().skill_by_name("retieji").unwrap();
    let u02_olxf = e_jw.catalog().skill_by_name("olxuanfeng").unwrap();
    let u02_rews = e_jw.catalog().skill_by_name("rewansha").unwrap();
    let cost_c = e_jw.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(&mut e_jw, 0, GameAction::ActivateSkill { skill: jiwu_def }));
    let mut saw_jiwu_skills = Vec::new();
    for _ in 0..8 {
        let Some(c) = e_jw.open_choice().cloned() else { break };
        if c.spec.prompt == "jiwu:card" {
            answer_choice(&mut e_jw, vec![ChoiceOption::Card(cost_c)]);
        } else if c.spec.prompt == "jiwu:skill" {
            saw_jiwu_skills = c.spec.options.clone();
            answer_choice(&mut e_jw, vec![ChoiceOption::Skill(u02_qxx)]);
        } else {
            drain_one_default(&mut e_jw);
        }
    }
    for exp in [u02_qxx, u02_rtj, u02_olxf, u02_rews] {
        assert!(
            saw_jiwu_skills.contains(&ChoiceOption::Skill(exp)),
            "U02 jiwu must offer {exp:?}"
        );
    }

    // Exercise real qiangxix parity on L03 pack
    e.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e.assign_general_named(1, "曹操").unwrap();
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();
    let qiangxix_def = e.catalog().skill_by_name("qiangxix").unwrap();
    e.state_core_mut().skills.insert(
        (0, qiangxix_def),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    step_to_phase(&mut e, 0, Phase::Play);

    // Place a weapon in hand (not equipped) so distance stays 2 to seat 2; test no-range-limit + hp>=1 optional weapon vs lose 1 HP
    let weapon = find_off_deck_or_deck_card_matching(&e, |_, d| {
        d.kind.equip_slot() == Some(EquipSlot::Weapon)
    });
    move_card_to_hand(&mut e, 0, weapon);
    let hp0_before = e.state_core().player(0).hp;
    let hp2_before = e.state_core().player(2).hp;

    // Activate qiangxix #1 -> target seat 2 (distance 2, no weapon equipped -> proves no range limit!)
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::ActivateSkill { skill: qiangxix_def }));
    let c_tgt = e.open_choice().expect("qiangxix:target").clone();
    assert_eq!(c_tgt.spec.prompt, "qiangxix:target");
    assert!(c_tgt.spec.options.contains(&ChoiceOption::Player(2)));
    // Save / restore mid qiangxix:target
    let mut e_rt = assert_archive_roundtrip(&e, &[&l03]);
    let ai_pick_tgt = e_rt.plan_action(0, &BoundedPlanner::new(32)).action;
    assert!(matches!(ai_pick_tgt, GameAction::Answer { .. }));
    answer_choice(&mut e_rt, vec![ChoiceOption::Player(2)]);
    e = e_rt;

    // qiangxix:card (hp >= 1 -> min=0, allow_cancel=false -> empty selection chooses LoseHp(1)!)
    let c_card = e.open_choice().expect("qiangxix:card").clone();
    assert_eq!(c_card.spec.prompt, "qiangxix:card");
    assert_eq!(c_card.spec.min, 0);
    assert!(!c_card.spec.allow_cancel);
    assert!(c_card.spec.options.contains(&ChoiceOption::Card(weapon)));
    // Verify public-info AI prefers discarding weapon instead of losing 1 HP
    let ai_card = e.plan_action(0, &BoundedPlanner::new(32)).action;
    assert_eq!(
        ai_card,
        GameAction::Answer {
            choice_id: c_card.id,
            selection: vec![ChoiceOption::Card(weapon)],
        }
    );
    // Human chooses [] (pass weapon -> LoseHp(1) branch)
    answer_choice(&mut e, Vec::new());
    drain_default(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_before - 1);
    assert_eq!(e.state_core().player(2).hp, hp2_before - 1);
    assert!(e.state_core().player(0).hand.contains(&weapon));

    // Activate qiangxix #2 -> seat 2 MUST be excluded (once per target!), seat 1 allowed; discard weapon this time!
    let hp0_mid = e.state_core().player(0).hp;
    let hp1_before = e.state_core().player(1).hp;
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::ActivateSkill { skill: qiangxix_def }));
    let c_tgt2 = e.open_choice().expect("qiangxix:target #2").clone();
    assert!(!c_tgt2.spec.options.contains(&ChoiceOption::Player(2)), "once per target");
    assert!(c_tgt2.spec.options.contains(&ChoiceOption::Player(1)));
    answer_choice(&mut e, vec![ChoiceOption::Player(1)]);
    answer_choice(&mut e, vec![ChoiceOption::Card(weapon)]);
    drain_default(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0_mid, "discarded weapon avoided LoseHp");
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    assert!(!e.state_core().player(0).hand.contains(&weapon));

    // Activate qiangxix #3 in same Play phase -> rejected (`usable: 2`)
    let res3 = submit(&mut e, 0, GameAction::ActivateSkill { skill: qiangxix_def });
    assert!(matches!(res3, CommandOutcome::Rejected { .. }), "usable:2 limit enforced");

    // Counterexample (`hp < 1`): at next Play turn with hp=0, cannot activate without weapon;
    // with weapon, qiangxix:card has min=1, allow_cancel=true and cancelling deals NO damage / NO hp loss!
    let mut e_low = new_boss_4p(7103, &u02, vec![]);
    e_low.assign_general_named(0, "boss_lvbu3").unwrap();
    e_low.assign_general_named(1, "曹操").unwrap();
    e_low.assign_general_named(2, "刘备").unwrap();
    e_low.assign_general_named(3, "孙权").unwrap();
    let qxx_u02 = e_low.catalog().skill_by_name("qiangxix").unwrap();
    e_low.state_core_mut().skills.insert(
        (0, qxx_u02),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    step_to_phase(&mut e_low, 0, Phase::Play);
    e_low.debug_clear_hand(0);
    e_low.debug_set_hp(0, 0);
    let rej_no_weapon = submit(&mut e_low, 0, GameAction::ActivateSkill { skill: qxx_u02 });
    assert!(matches!(rej_no_weapon, CommandOutcome::Rejected { .. }));

    move_card_to_hand(&mut e_low, 0, weapon);
    // AI with hp <= 1 AND weapon DOES activate/discard weapon; if no weapon it cancels target
    assert_accepted_or_pending(&submit(&mut e_low, 0, GameAction::ActivateSkill { skill: qxx_u02 }));
    answer_choice(&mut e_low, vec![ChoiceOption::Player(1)]);
    let c_low = e_low.open_choice().expect("qiangxix:card hp<1").clone();
    assert_eq!(c_low.spec.min, 1);
    assert!(c_low.spec.allow_cancel);
    answer_choice(&mut e_low, Vec::new()); // cancel
    drain_default(&mut e_low);
    assert_eq!(e_low.state_core().player(0).hp, 0);
    assert_eq!(e_low.state_core().player(1).hp, 4, "cancelling weapon-only qiangxix deals no damage");
}

#[test]
fn round3_retieji_sha_begin_judge_seal_and_suit_give_parity() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = new_boss_4p(7104, &pack, vec![]);
    e.assign_general_named(0, "boss_lvbu3").unwrap();
    e.assign_general_named(1, "曹操").unwrap(); // 奸雄 (non-locked!)
    e.assign_general_named(2, "马超").unwrap(); // 马术 (locked!) + 铁骑 (non-locked!)
    e.assign_general_named(3, "孙权").unwrap();
    let rtj = e.catalog().skill_by_name("retieji").unwrap();
    e.state_core_mut().skills.insert(
        (0, rtj),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );

    step_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(1);
    let heart_judge = e.debug_place_suit(0, Suit::Heart);
    move_card_to_deck_top(&mut e, heart_judge);
    let heart_give = e.debug_place_suit(1, Suit::Heart);
    let shan_1 = find_off_deck_or_deck_card_matching(&e, |_, d| d.kind == CardKind::Shan && d.suit != Suit::Heart);
    let shan_2 = find_off_deck_or_deck_card_matching(&e, |i, d| d.kind == CardKind::Shan && d.suit != Suit::Heart && i != shan_1);
    move_card_to_hand(&mut e, 1, shan_1);
    move_card_to_hand(&mut e, 1, shan_2);
    let sha_a = e.debug_place(0, "sha");

    // Branch A: target 1 HAS matching Heart card and GIVES it -> sealed, NOT directHit, 1 damage only if no Shan (here has Shan -> dodges!)
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] }));
    let c_conf = e.open_choice().expect("retieji:confirm").clone();
    assert_eq!(c_conf.spec.prompt, "retieji:confirm");
    assert_eq!(e.plan_action(0, &BoundedPlanner::new(32)).action, GameAction::Answer {
        choice_id: c_conf.id,
        selection: vec![ChoiceOption::Confirm],
    });
    let mut e = assert_archive_roundtrip(&e, &[&pack]);
    answer_choice(&mut e, vec![ChoiceOption::Confirm]);

    // Target 1 must now be sealed on non-locked skills!
    assert!(e.catalog().is_tieji_sealed(e.state_core(), 1));
    let jx_def = e.catalog().skill_by_name("奸雄").unwrap();
    assert!(e.state_core().has_skill(1, jx_def));
    let c_give = e.open_choice().expect("retieji:give").clone();
    assert_eq!(c_give.spec.prompt, "retieji:give");
    assert!(c_give.spec.options.contains(&ChoiceOption::Card(heart_give)));
    assert!(!c_give.spec.options.contains(&ChoiceOption::Card(shan_1)));
    let e_rt = assert_archive_roundtrip(&e, &[&pack]);
    assert_eq!(
        e_rt.plan_action(1, &BoundedPlanner::new(32)).action,
        GameAction::Answer {
            choice_id: c_give.id,
            selection: vec![ChoiceOption::Card(heart_give)],
        }
    );
    e = e_rt;
    answer_choice(&mut e, vec![ChoiceOption::Card(heart_give)]);
    assert!(e.state_core().player(0).hand.contains(&heart_give));

    // Because target 1 gave matching suit, Sha is NOT directHit: target 1 gets asked to play Shan (2 due to boss_lvbu3 Wushuang) and dodges!
    let c_shan = e.open_choice().expect("shan response #1 after giving matching suit").clone();
    let pick_s1 = if c_shan.spec.options.contains(&ChoiceOption::Card(shan_1)) {
        shan_1
    } else {
        shan_2
    };
    answer_choice(&mut e, vec![ChoiceOption::Card(pick_s1)]);
    if let Some(c_shan2) = e.open_choice().cloned() {
        let pick_s2 = if c_shan2.spec.options.contains(&ChoiceOption::Card(shan_2)) {
            shan_2
        } else {
            shan_1
        };
        if c_shan2.spec.options.contains(&ChoiceOption::Card(pick_s2)) {
            answer_choice(&mut e, vec![ChoiceOption::Card(pick_s2)]);
        }
    }
    drain_default(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "dodged after giving retieji suit card");

    // Branch B: extra Sha capacity + target 1 declines (no matching suit or passes) -> directHit + +1 damage (2 damage!) AND 奸雄 sealed!
    e.state_core_mut().turn.sha_used_play_phase = false;
    let spade_judge = e.debug_place_suit(0, Suit::Spade);
    move_card_to_deck_top(&mut e, spade_judge);
    e.debug_clear_hand(1);
    let sha_b = e.debug_place(0, "sha");
    let diamond_shan = find_off_deck_or_deck_card_matching(&e, |_, d| d.kind == CardKind::Shan && d.suit != Suit::Spade);
    move_card_to_hand(&mut e, 1, diamond_shan);

    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] }));
    let mut saw_shan_prompt = false;
    let mut saw_jx_prompt = false;
    for _ in 0..10 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "retieji:confirm" {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt == "retieji:give" {
            answer_choice(&mut e, Vec::new());
        } else if c.spec.prompt.contains("shan") {
            saw_shan_prompt = true;
            drain_one_default(&mut e);
        } else if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Confirm) {
            saw_jx_prompt = true;
            drain_one_default(&mut e);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(!saw_shan_prompt, "declined/missing retieji suit card makes Sha directHit");
    assert!(!saw_jx_prompt, "retieji seals target 1 non-locked 奸雄");
    assert_eq!(e.state_core().player(1).hp, 2, "retieji adds +1 damage (2 total)");
    assert!(!e.state_core().player(1).hand.contains(&sha_b));

    // At turn end, seal clears!
    step_to_phase(&mut e, 1, Phase::Play);
    assert!(!e.catalog().is_tieji_sealed(e.state_core(), 1));
}

#[test]
fn round3_olxuanfeng_vs_rexuanfeng_and_rewansha_wansha_parity() {
    let u02 = boss_play_extension_pack("noname", &[]);
    let l03 = boss_play_extension_pack("noname-liuli", &[]);

    // 1) U02 `olxuanfeng`: triggers on Play-phase `zhiheng` >=2 discard (`rexuanfeng` must NOT!)
    let mut e_ol = new_boss_4p(7105, &u02, vec![]);
    e_ol.assign_general_named(0, "boss_lvbu3").unwrap();
    e_ol.assign_general_named(1, "曹操").unwrap();
    e_ol.assign_general_named(2, "刘备").unwrap();
    e_ol.assign_general_named(3, "孙权").unwrap();
    let olxf = e_ol.catalog().skill_by_name("olxuanfeng").unwrap();
    let zhiheng = e_ol.catalog().skill_by_name("制衡").unwrap();
    e_ol.state_core_mut().skills.insert(
        (0, olxf),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    e_ol.state_core_mut().skills.insert(
        (0, zhiheng),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    step_to_phase(&mut e_ol, 0, Phase::Play);
    e_ol.debug_clear_hand(0);
    let zh1 = e_ol.debug_place(0, "sha");
    let zh2 = e_ol.debug_place(0, "shan");
    let tgt_card = e_ol.debug_place(1, "tao");
    assert_accepted_or_pending(&submit(&mut e_ol, 0, GameAction::ActivateSkill { skill: zhiheng }));
    let mut saw_olxf_on_play_discard = false;
    for _ in 0..12 {
        let Some(c) = e_ol.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Confirm) {
            answer_choice(&mut e_ol, vec![ChoiceOption::Confirm]);
        } else if c.spec.options.contains(&ChoiceOption::Card(zh1)) && c.spec.options.contains(&ChoiceOption::Card(zh2)) {
            answer_choice(&mut e_ol, vec![ChoiceOption::Card(zh1), ChoiceOption::Card(zh2)]);
        } else if c.spec.prompt == "xuanfeng:target" {
            saw_olxf_on_play_discard = true;
            answer_choice(&mut e_ol, vec![ChoiceOption::Player(1)]);
        } else if c.spec.prompt == "xuanfeng:card" {
            let pick = c
                .spec
                .options
                .iter()
                .find(|o| **o == ChoiceOption::Card(tgt_card))
                .copied()
                .unwrap_or(c.spec.options[0]);
            answer_choice(&mut e_ol, vec![pick]);
        } else {
            drain_one_default(&mut e_ol);
        }
    }
    assert!(saw_olxf_on_play_discard, "olxuanfeng triggers on >=2 Play-phase discard");
    assert!(!e_ol.state_core().player(1).hand.contains(&tgt_card));

    // Counterexample: L03 `rexuanfeng` must NOT trigger on Play-phase `zhiheng` >=2 discard, ONLY Discard>=2 or Equip loss!
    let mut e_re = new_boss_4p(7106, &l03, vec![]);
    e_re.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e_re.assign_general_named(1, "曹操").unwrap();
    e_re.assign_general_named(2, "刘备").unwrap();
    e_re.assign_general_named(3, "孙权").unwrap();
    let rexf = e_re.catalog().skill_by_name("rexuanfeng").unwrap();
    let zh_re = e_re.catalog().skill_by_name("制衡").unwrap();
    e_re.state_core_mut().skills.insert(
        (0, rexf),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    e_re.state_core_mut().skills.insert(
        (0, zh_re),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    step_to_phase(&mut e_re, 0, Phase::Play);
    e_re.debug_clear_hand(0);
    let rzh1 = e_re.debug_place(0, "sha");
    let rzh2 = e_re.debug_place(0, "shan");
    assert_accepted_or_pending(&submit(&mut e_re, 0, GameAction::ActivateSkill { skill: zh_re }));
    let mut saw_rexf_on_play = false;
    for _ in 0..8 {
        let Some(c) = e_re.open_choice().cloned() else { break };
        if c.spec.prompt.starts_with("rexuanfeng") || c.spec.prompt.starts_with("xuanfeng") {
            saw_rexf_on_play = true;
        }
        if c.spec.options.contains(&ChoiceOption::Card(rzh1)) && c.spec.options.contains(&ChoiceOption::Card(rzh2)) {
            answer_choice(&mut e_re, vec![ChoiceOption::Card(rzh1), ChoiceOption::Card(rzh2)]);
        } else {
            drain_one_default(&mut e_re);
        }
    }
    assert!(!saw_rexf_on_play, "rexuanfeng must NOT trigger on non-Discard Play-phase card discard");

    // 2) `rexuanfeng` equip-move mode (`Tag(1)`) on equip loss between two other seats + AI & save/restore!
    let w0 = find_off_deck_or_deck_card_matching(&e_re, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    let w1 = find_off_deck_or_deck_card_matching(&e_re, |i, d| d.kind.equip_slot() == Some(EquipSlot::Weapon) && i != w0);
    let armor1 = find_off_deck_or_deck_card_matching(&e_re, |_, d| d.kind.equip_slot() == Some(EquipSlot::Armor));
    move_card_to_hand(&mut e_re, 0, w0);
    move_card_to_hand(&mut e_re, 0, w1);
    move_card_to_hand(&mut e_re, 1, armor1);
    // Seat 1 equips armor1, Seat 0 equips w0 then replaces with w1
    e_re.state_core_mut().player_mut(1).hand.retain(|&c| c != armor1);
    e_re.state_core_mut().player_mut(1).equips.push(armor1);
    assert_accepted_or_pending(&submit(&mut e_re, 0, GameAction::PlayCard { card: w0, targets: vec![] }));
    drain_default(&mut e_re);
    assert_accepted_or_pending(&submit(&mut e_re, 0, GameAction::PlayCard { card: w1, targets: vec![] }));
    if let Some(c) = e_re.open_choice().cloned() {
        if c.spec.prompt == "trigger_confirm" {
            answer_choice(&mut e_re, vec![ChoiceOption::Confirm]);
        }
    }

    let c_mode = e_re.open_choice().expect("rexuanfeng:mode").clone();
    assert_eq!(c_mode.spec.prompt, "rexuanfeng:mode");
    assert!(c_mode.spec.options.contains(&ChoiceOption::Tag(1)));
    let mut e_re = assert_archive_roundtrip(&e_re, &[&l03]);
    let ai_mode = e_re.plan_action(0, &BoundedPlanner::new(32)).action;
    assert!(matches!(ai_mode, GameAction::Answer { .. }));
    answer_choice(&mut e_re, vec![ChoiceOption::Tag(1)]);

    let c_from = e_re.open_choice().expect("rexuanfeng:move_from").clone();
    assert_eq!(c_from.spec.prompt, "rexuanfeng:move_from");
    assert!(c_from.spec.options.contains(&ChoiceOption::Player(1)));
    let _ = e_re.plan_action(0, &BoundedPlanner::new(32));
    answer_choice(&mut e_re, vec![ChoiceOption::Player(1)]);

    let c_card = e_re.open_choice().expect("rexuanfeng:move_card").clone();
    assert_eq!(c_card.spec.prompt, "rexuanfeng:move_card");
    assert_eq!(c_card.spec.options, vec![ChoiceOption::Card(armor1)]);
    let _ = e_re.plan_action(0, &BoundedPlanner::new(32));
    answer_choice(&mut e_re, vec![ChoiceOption::Card(armor1)]);

    let c_to = e_re.open_choice().expect("rexuanfeng:move_to").clone();
    assert_eq!(c_to.spec.prompt, "rexuanfeng:move_to");
    assert!(!c_to.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(!c_to.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(c_to.spec.options.contains(&ChoiceOption::Player(3)));
    let _ = e_re.plan_action(0, &BoundedPlanner::new(32));
    answer_choice(&mut e_re, vec![ChoiceOption::Player(3)]);
    drain_default(&mut e_re);
    assert!(!e_re.state_core().player(1).equips.contains(&armor1));
    assert!(e_re.state_core().player(3).equips.contains(&armor1));

    // 3) `rewansha` (`U02`) seals non-locked skills on other non-dying seats + blocks ally Tao until Dying finishes;
    // `wansha` (`L03`) ALSO blocks ally Tao/jijiu!
    let mut e_ws = new_boss_4p(7107, &u02, vec![]);
    e_ws.assign_general_named(0, "boss_lvbu3").unwrap();
    e_ws.assign_general_named(1, "刘备").unwrap();
    e_ws.assign_general_named(2, "曹操").unwrap();
    e_ws.assign_general_named(3, "孙权").unwrap();
    let rewansha = e_ws.catalog().skill_by_name("rewansha").unwrap();
    e_ws.state_core_mut().skills.insert(
        (0, rewansha),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    e_ws.state_core_mut().player_mut(1).hp = 1;
    let tao_self = e_ws.debug_place(1, "tao");
    let tao_ally = e_ws.debug_place(2, "tao");
    step_to_phase(&mut e_ws, 0, Phase::Play);
    let sha_d = e_ws.debug_place(0, "sha");
    e_ws.debug_clear_hand(1);
    move_card_to_hand(&mut e_ws, 1, tao_self);
    assert_accepted_or_pending(&submit(&mut e_ws, 0, GameAction::PlayCard { card: sha_d, targets: vec![1] }));
    let mut asked_ally_during_rewansha = false;
    let mut saw_sealed_during_dying = false;
    let mut saw_dying_self_tao = false;
    for _ in 0..12 {
        let Some(c) = e_ws.open_choice().cloned() else { break };
        if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(tao_ally)) {
            asked_ally_during_rewansha = true;
            drain_one_default(&mut e_ws);
        } else if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(tao_self)) {
            saw_dying_self_tao = true;
            saw_sealed_during_dying = e_ws.catalog().is_tieji_sealed(e_ws.state_core(), 2)
                && !e_ws.catalog().is_tieji_sealed(e_ws.state_core(), 1)
                && !e_ws.catalog().is_tieji_sealed(e_ws.state_core(), 0);
            answer_choice(&mut e_ws, vec![ChoiceOption::Card(tao_self)]);
        } else {
            drain_one_default(&mut e_ws);
        }
    }
    assert!(saw_dying_self_tao, "dying player CAN self-Tao under rewansha");
    assert!(saw_sealed_during_dying, "rewansha sealed other non-dying seat 2 during Dying");
    assert!(!asked_ally_during_rewansha, "rewansha blocked non-dying ally seat 2 Tao save");
    assert!(!e_ws.catalog().is_tieji_sealed(e_ws.state_core(), 2), "rewansha seal cleared after Dying settled");

    // Verify L03 `wansha` blocks `华佗` `急救` and ally `Tao`
    let mut e_l03ws = new_boss_4p(7108, &l03, vec![]);
    e_l03ws.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e_l03ws.assign_general_named(1, "刘备").unwrap();
    e_l03ws.assign_general_named(2, "华佗").unwrap();
    e_l03ws.assign_general_named(3, "孙权").unwrap();
    let wansha_l03 = e_l03ws.catalog().skill_by_name("wansha").unwrap();
    e_l03ws.state_core_mut().skills.insert(
        (0, wansha_l03),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    e_l03ws.state_core_mut().player_mut(1).hp = 1;
    step_to_phase(&mut e_l03ws, 0, Phase::Play);
    e_l03ws.debug_clear_hand(1);
    e_l03ws.debug_clear_hand(2);
    let red_ht = e_l03ws.debug_place_suit(2, Suit::Heart);
    let sha_w = e_l03ws.debug_place(0, "sha");
    let _ = red_ht;
    assert_accepted_or_pending(&submit(&mut e_l03ws, 0, GameAction::PlayCard { card: sha_w, targets: vec![1] }));
    let mut ht_asked = false;
    for _ in 0..10 {
        let Some(c) = e_l03ws.open_choice().cloned() else { break };
        if c.observer == 2 {
            ht_asked = true;
        }
        drain_one_default(&mut e_l03ws);
    }
    assert!(!ht_asked, "wansha blocked Huatuo ally rescue");
    assert!(!e_l03ws.state_core().player(1).alive);
}

#[test]
fn round3_boss_baonu_full_unwind_and_baonuwash_win_condition() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = new_boss_4p(7109, &pack, vec![]);
    e.assign_general_named(0, "boss_lvbu1").unwrap();
    e.assign_general_named(1, "曹操").unwrap();
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();

    // Seat 1 is challenger in Play with an extra turn queued AND an expires_end_of_turn temp skill,
    // and plays `Nanman` AOE hitting Boss (seat 0 at hp=5) before seat 2/3!
    let temp_def = e.catalog().skill_by_name("奸雄").unwrap();
    step_to_phase(&mut e, 1, Phase::Play);
    e.state_core_mut().player_mut(0).hp = 5;
    e.state_core_mut().turn.extra_phases.push(1);
    e.state_core_mut().vm_aux.flags.insert((2, "extra_turn_insert_queued".to_string()), 1);
    e.state_core_mut().skills.insert(
        (1, temp_def),
        SkillInstance { owner: 1, enabled: true, once_used: false, expires_end_of_turn: true, rounds: 0 },
    );
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let hp2_before = e.state_core().player(2).hp;
    let total_before = total_cards_accounting(e.state_core());
    let nanman = find_off_deck_or_deck_card_matching(&e, |_, d| d.kind == CardKind::SavageAttack);
    move_card_to_hand(&mut e, 1, nanman);

    assert_accepted_or_pending(&submit(&mut e, 1, GameAction::PlayCard { card: nanman, targets: vec![] }));
    let mut saw_baonu = false;
    let mut saw_nanman_after_boss = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_baonu:form" {
            saw_baonu = true;
            answer_choice(&mut e, vec![ChoiceOption::Tag(0)]); // boss_lvbu2
        } else {
            if c.observer == 2 || c.observer == 3 {
                saw_nanman_after_boss = true;
            }
            drain_one_default(&mut e);
        }
    }
    assert!(saw_baonu, "boss_baonu triggered mid-Nanman");
    assert!(!saw_nanman_after_boss, "BUG-1: boss_baonu unwound remaining Nanman AOE targets");
    assert_eq!(e.state_core().player(2).hp, hp2_before, "seat 2 never suffered Nanman damage after unwind");
    assert_eq!(e.turn().current, 0, "Boss gets immediate TurnBegin");
    assert!(e.state_core().table.is_empty(), "stranded Nanman moved to discard");
    assert!(e.state_core().discard.contains(&nanman), "Nanman conserved into discard");
    assert!(e.state_core().turn.extra_phases.is_empty(), "interrupted extra phases cleared");
    assert!(!e.state_core().has_skill(1, temp_def), "expires_end_of_turn cleaned on unwind");
    assert_eq!(total_cards_accounting(e.state_core()), total_before, "exact card conservation on unwind");

    // 2) `boss_baonuwash`: 1st wash inside a 2-card draw triggers `boss_baonu` (and conserves held_cards!);
    // 2nd wash sets `q4_boss_baonuwash_game_over` -> Boss wins at TurnPhaseAfter!
    let mut ew = new_boss_4p(7110, &pack, vec![]);
    ew.assign_general_named(0, "boss_lvbu1").unwrap();
    ew.assign_general_named(1, "曹操").unwrap();
    ew.assign_general_named(2, "刘备").unwrap();
    ew.assign_general_named(3, "孙权").unwrap();
    let total_w = total_cards_accounting(ew.state_core());
    // Move all but 1 card from deck into discard so seat 1's DrawPhase (2 cards) forces a deck wash mid-Draw!
    step_to_phase(&mut ew, 1, Phase::Judge);
    {
        let core = ew.state_core_mut();
        while core.deck.len() > 1 {
            let c = core.deck.pop().unwrap();
            core.discard.push(c);
        }
    }
    assert_accepted_or_pending(&submit(&mut ew, 1, GameAction::NextPhase)); // -> Draw -> washes deck!
    let mut saw_wash_baonu = false;
    for _ in 0..16 {
        let Some(c) = ew.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_baonu:form" {
            saw_wash_baonu = true;
            answer_choice(&mut ew, vec![ChoiceOption::Tag(1)]); // boss_lvbu3
        } else {
            drain_one_default(&mut ew);
        }
    }
    assert!(saw_wash_baonu, "1st deck wash triggered boss_baonu even after transforming");
    assert_eq!(ew.turn().current, 0, "boss_baonuwash gave Boss immediate turn");
    assert_eq!(
        total_cards_accounting(ew.state_core()),
        total_w,
        "held_cards inside unwound Draw conserved to discard!"
    );

    // Trigger 2nd deck wash during Boss's turn -> Boss wins at TurnPhaseAfter!
    step_to_phase(&mut ew, 0, Phase::Play);
    {
        let core = ew.state_core_mut();
        while !core.deck.is_empty() {
            let c = core.deck.pop().unwrap();
            core.discard.push(c);
        }
    }
    let wuzhong = find_off_deck_or_deck_card_matching(&ew, |_, d| d.kind == CardKind::Wuzhong);
    move_card_to_hand(&mut ew, 0, wuzhong);
    assert_accepted_or_pending(&submit(&mut ew, 0, GameAction::PlayCard { card: wuzhong, targets: vec![] }));
    drain_default(&mut ew);
    assert!(ew.outcome().is_none(), "win waits until TurnPhaseAfter");
    while ew.outcome().is_none() && ew.turn().current == 0 {
        let _ = submit(&mut ew, 0, GameAction::NextPhase);
        drain_default(&mut ew);
    }
    assert_eq!(ew.outcome().map(|o| o.winner), Some(Winner::Boss { boss_won: true }), "2nd deck wash gives Boss victory");
}

#[test]
fn round3_boss_fengxing_kongcheng_and_keding_multiple_target_legality() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = new_boss_4p(7111, &pack, vec![]);
    e.assign_general_named(0, "boss_juechenmiaocai").unwrap(); // boss_fengxing
    e.assign_general_named(1, "诸葛亮").unwrap(); // 空城
    e.assign_general_named(2, "曹操").unwrap();
    e.assign_general_named(3, "孙权").unwrap();
    e.debug_clear_hand(1); // kongcheng active!
    let _c2 = e.debug_place(2, "shan");
    e.debug_clear_hand(3);
    e.debug_set_turn(3, Phase::End, 1);
    // Advance across TurnBegin into Prepare (when boss_fengxing asks target)
    let _ = submit(&mut e, 3, GameAction::NextPhase);
    let mut c_fx = None;
    for _ in 0..8 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "boss_fengxing:target" {
                c_fx = Some(c);
                break;
            }
            drain_one_default(&mut e);
        } else {
            let _ = submit(&mut e, 0, GameAction::NextPhase);
        }
    }
    let c_fx = c_fx.expect("boss_fengxing:target");
    assert_eq!(c_fx.spec.prompt, "boss_fengxing:target");
    assert!(
        !c_fx.spec.options.contains(&ChoiceOption::Player(1)),
        "boss_fengxing must reject kongcheng with 0 hand cards"
    );
    assert!(
        c_fx.spec.options.contains(&ChoiceOption::Player(2)),
        "boss_fengxing keeps no-range-limit targeting against distance-2 seat 2"
    );
    drain_default(&mut e);

    // 2) `boss_keding` + `fengjian` + `kongcheng` + `allows_multiple_targets` gate
    let mut ek = new_boss_4p(7112, &pack, vec![]);
    ek.assign_general_named(0, "boss_fuweizilong").unwrap(); // boss_keding
    ek.assign_general_named(1, "诸葛亮").unwrap(); // 空城 (0 hand -> illegal for Sha!)
    ek.assign_general_named(2, "曹操").unwrap();
    ek.assign_general_named(3, "孙权").unwrap();
    let fj_def = ek.catalog().skill_by_name("boss_fengjian").unwrap();
    ek.state_core_mut().skills.insert(
        (3, fj_def),
        SkillInstance { owner: 3, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    // Seat 3 (holding boss_fengjian) damages seat 0 with Sha -> sets q4_fengjian_ban:3 on seat 0!
    step_to_phase(&mut ek, 3, Phase::Play);
    ek.debug_clear_hand(0);
    let sha_fj = ek.debug_place(3, "sha");
    assert_accepted_or_pending(&submit(&mut ek, 3, GameAction::PlayCard { card: sha_fj, targets: vec![0] }));
    drain_default(&mut ek);
    assert_eq!(
        ek.state_core().vm_aux.flags.get(&(0, "q4_fengjian_ban:3".to_string())).copied(),
        Some(1),
        "seat 3 boss_fengjian bans victim seat 0 from targeting seat 3"
    );

    step_to_phase(&mut ek, 0, Phase::Play);
    let k1 = ek.debug_place(0, "sha");
    let k2 = ek.debug_place(0, "shan");
    let k3 = ek.debug_place(0, "tao");
    let xr_def = ek.catalog().skill_by_name("boss_xiaorui").unwrap();
    ek.state_core_mut().skills.insert(
        (0, xr_def),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    // Equip a range>=2 weapon so seat 0 can reach distance-2 seat 2
    let long_wep = find_off_deck_or_deck_card_matching(&ek, |_, d| {
        d.kind.equip_slot() == Some(EquipSlot::Weapon) && d.range >= 2
    });
    move_card_to_hand(&mut ek, 0, long_wep);
    assert_accepted_or_pending(&submit(&mut ek, 0, GameAction::PlayCard { card: long_wep, targets: vec![] }));
    drain_default(&mut ek);
    // Seat 3 is fengjian_banned and seat 1 (诸葛亮) has 0 hand -> playing Sha targeting seat 2 has 0 extra targets!
    ek.debug_clear_hand(1);
    let _card2 = ek.debug_place(2, "shan");
    ek.state_core_mut().turn.sha_used_play_phase = false;
    let sha_no_extra = ek.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(&mut ek, 0, GameAction::PlayCard { card: sha_no_extra, targets: vec![2] }));
    let mut saw_keding_with_zero_extra = false;
    for _ in 0..8 {
        let Some(c) = ek.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_keding:cards" {
            saw_keding_with_zero_extra = true;
        }
        drain_one_default(&mut ek);
    }
    assert!(!saw_keding_with_zero_extra, "0 legal extra targets -> boss_keding must not open");

    // Now give seat 1 1 hand card -> 1 legal extra target (seat 1, NOT fengjian-banned seat 3!),
    // while Boss holds 3 cards -> `boss_keding:cards` cap MUST be 1 (matching ASK_KEDING_TARGETS options!), NOT 3!
    let _c1 = ek.debug_place(1, "tao");
    ek.state_core_mut().turn.sha_used_play_phase = false;
    let _ = ek.debug_place(0, "sha");
    let _ = ek.debug_place(0, "shan");
    let sha_1extra = ek.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(&mut ek, 0, GameAction::PlayCard { card: sha_1extra, targets: vec![2] }));
    let c_kcards = ek.open_choice().expect("boss_keding:cards with 1 legal extra target").clone();
    assert_eq!(c_kcards.spec.prompt, "boss_keding:cards");
    assert_eq!(c_kcards.spec.max, 1, "cap equals exact count of legal extra targets (seat 1 only, excluding seat 3 fengjian_banned)");
    let pick_c = match c_kcards.spec.options[0] {
        ChoiceOption::Card(c) => c,
        _ => panic!("expected card option"),
    };
    answer_choice(&mut ek, vec![ChoiceOption::Card(pick_c)]);
    let c_ktgts = ek.open_choice().expect("boss_keding:targets").clone();
    assert_eq!(c_ktgts.spec.prompt, "boss_keding:targets");
    assert_eq!(c_ktgts.spec.options, vec![ChoiceOption::Player(1)]);
    answer_choice(&mut ek, vec![ChoiceOption::Player(1)]);
    drain_default(&mut ek);

    // Delayed tricks (Le/Bing), Jiedao, Wuzhong must NEVER trigger boss_keding (`allows_multiple_targets == false`)
    // even when seats 1/2/3 are legal targets, while Dismantle (`Guohe`) DOES trigger `boss_keding`!
    let _ = (k1, k2, k3);
    for kind in [CardKind::Le, CardKind::Wuzhong, CardKind::Jiedao] {
        let card = find_off_deck_or_deck_card_matching(&ek, |_, d| d.kind == kind);
        move_card_to_hand(&mut ek, 0, card);
        let targets = match kind {
            CardKind::Wuzhong => vec![],
            CardKind::Jiedao => {
                let wp = find_off_deck_or_deck_card_matching(&ek, |_, d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
                ek.state_core_mut().player_mut(1).equips.push(wp);
                vec![1, 2]
            }
            _ => vec![2],
        };
        let res = submit(&mut ek, 0, GameAction::PlayCard { card, targets });
        assert_accepted_or_pending(&res);
        for _ in 0..8 {
            let Some(c) = ek.open_choice().cloned() else { break };
            assert_ne!(c.spec.prompt, "boss_keding:cards", "{kind:?} must NOT allow multiple targets via boss_keding");
            drain_one_default(&mut ek);
        }
    }
    let dismantle = find_off_deck_or_deck_card_matching(&ek, |_, d| d.kind == CardKind::Dismantle);
    move_card_to_hand(&mut ek, 0, dismantle);
    let _pad1 = ek.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(&mut ek, 0, GameAction::PlayCard { card: dismantle, targets: vec![2] }));
    let c_dis = ek.open_choice().expect("Dismantle should trigger boss_keding:cards").clone();
    assert_eq!(c_dis.spec.prompt, "boss_keding:cards");
    drain_default(&mut ek);
}

#[test]
fn round3_l17_douzhuan_full_engine_bridge_and_save_restore() {
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "l17-bridge-gen",
            version: 1,
            description: "L17 JinJiang & miaoshu test generals",
        },
        generals: vec![
            ExtensionGeneral {
                name: "caocao",
                faction: Faction::Wei,
                skills: &["奸雄"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "liubei",
                faction: Faction::Shu,
                skills: &["仁德"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
            ExtensionGeneral {
                name: "sunquan",
                faction: Faction::Wu,
                skills: &["制衡"],
                max_hp: 4,
                gender: Gender::Male,
                presentation_binding: None,
            },
        ],
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };

    let raw_dz = repo_asset("packages/extensions/douzhuan-xingyi-upstream-liuli/assets/config.json");
    let mut dz_val: Value = serde_json::from_str(&raw_dz).unwrap();
    {
        let obj = dz_val.as_object_mut().unwrap();
        obj.insert("jinJiang".into(), Value::Bool(true));
        obj.insert("jinJiangType".into(), Value::String("forbidai".into()));
        obj.insert("jinJiangRuZhen".into(), Value::String("online".into()));
        obj.insert("miaoshuXiuGai".into(), Value::Bool(true));
        obj.insert("forbidReload".into(), Value::Bool(false));
        obj.insert("modePlans".into(), serde_json::json!({"identity": 1}));
        obj.insert("plans".into(), serde_json::json!({"1": ["caocao"], "2": ["sunquan"]}));
    }
    let cfg_json = serde_json::to_string(&dz_val).unwrap();
    let rarities = BTreeMap::new();

    // Preserving an existing L15 forbidai ban on "sunquan" when L17 later clears/switches plans
    let mut e = Engine::new_with_extensions_mode(
        7113,
        4,
        &[&pack],
        GameMode::Identity {
            roles: vec![
                engine_native::Role::Lord,
                engine_native::Role::Loyalist,
                engine_native::Role::Rebel,
                engine_native::Role::Traitor,
            ],
        },
    )
    .unwrap();
    let raw_l15 = repo_asset("packages/extensions/shousha-full-upstream-liuli/assets/config.json");
    let _ = apply_shousha_to_engine(&raw_l15, "identity", false, &[], &BTreeMap::new(), &BTreeMap::new(), &mut e);
    e.set_general_forbidai_named("liubei", true);

    // Apply L17 online with JinJiangType=forbidai -> "caocao" is AI-banned ONLY, human keeps "caocao"!
    apply_douzhuan_to_engine(&cfg_json, true, &rarities, &mut e).unwrap();
    assert!(e.douzhuan_jinjiang_gate());
    assert_eq!(e.douzhuan_jinjiang_forbid_type(), "forbidai");
    assert_eq!(e.douzhuan_plan_index(), Some(1));
    assert!(e.is_douzhuan_miaoshu_enabled());
    assert_eq!(
        e.format_skill_description("距离不大于1且不小于0，阴转阳"),
        "距离≤1且≥0，阳转阴"
    );
    assert!(!e.is_douzhuan_forbid_reload());
    assert!(e.douzhuan_would_reload(Some("doudizhu")));
    assert!(!e.douzhuan_would_reload(Some("identity")));

    let human_names: Vec<&str> = e
        .filter_selectable_generals(false)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    let ai_names: Vec<&str> = e
        .filter_selectable_generals(true)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(human_names.contains(&"caocao"), "forbidai allows human");
    assert!(!ai_names.contains(&"caocao"), "forbidai blocks AI");
    assert!(!ai_names.contains(&"liubei"), "prior L15 forbidai preserved");

    // Save/Restore preserves ALL L17 flags and filtered pools!
    let restored = assert_archive_roundtrip(&e, &[&pack]);
    assert!(restored.douzhuan_jinjiang_gate());
    assert_eq!(restored.douzhuan_jinjiang_forbid_type(), "forbidai");
    assert_eq!(restored.douzhuan_plan_index(), Some(1));
    assert_eq!(
        restored.format_skill_description("不大于不小于阴阳"),
        "≤≥阳阴"
    );
    assert!(restored.douzhuan_would_reload(Some("doudizhu")));
    let restored_ai: Vec<&str> = restored
        .filter_selectable_generals(true)
        .into_iter()
        .filter_map(|d| restored.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(!restored_ai.contains(&"caocao"));
    assert!(!restored_ai.contains(&"liubei"));

    // Interactive banCharF request: add "sunquan" and remove "caocao" (keeping prior L15 "liubei"), plus unseen type + offline gate!
    let req_json = serde_json::json!({
        "mode": "identity",
        "network": "online",
        "editIndex": 1,
        "banIds": ["sunquan"],
        "openIds": ["caocao"],
        "characters": [
            {"id": "caocao", "tags": []},
            {"id": "liubei", "tags": []},
            {"id": "sunquan", "tags": []}
        ],
        "skills": []
    })
    .to_string();
    let mut dz_unseen = dz_val.clone();
    dz_unseen.as_object_mut().unwrap().insert("jinJiangType".into(), Value::String("unseen".into()));
    let rep = apply_douzhuan_request_to_engine(&serde_json::to_string(&dz_unseen).unwrap(), &req_json, &rarities, &mut e).unwrap();
    assert_eq!(rep.banned, vec!["sunquan".to_string()]);
    assert_eq!(e.douzhuan_jinjiang_forbid_type(), "unseen");
    let human_after_edit: Vec<&str> = e
        .filter_selectable_generals(false)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    let ai_after_edit: Vec<&str> = e
        .filter_selectable_generals(true)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(human_after_edit.contains(&"caocao"), "L17 ban on caocao cleanly removed");
    assert!(!human_after_edit.contains(&"sunquan"), "unseen hides sunquan even from human");
    assert!(!ai_after_edit.contains(&"liubei"), "prior L15 ban on liubei untouched by clear_douzhuan_l17_bans");

    // Offline gate: JinJiangRuZhen="online" -> offline clears L17 gate & L17 bans!
    apply_douzhuan_to_engine(&serde_json::to_string(&dz_unseen).unwrap(), false, &rarities, &mut e).unwrap();
    assert!(!e.douzhuan_jinjiang_gate());
    let human_offline: Vec<&str> = e
        .filter_selectable_generals(false)
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| m.0))
        .collect();
    assert!(human_offline.contains(&"sunquan"), "offline network disables online-gated JinJiang bans");
}

#[test]
fn round3_ai_public_info_isolation_on_jiwu_choices() {
    let pack = boss_play_extension_pack("noname-liuli", &[]);
    let mut e = new_boss_4p(7114, &pack, vec![]);
    e.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e.assign_general_named(1, "曹操").unwrap();
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();
    let rtj = e.catalog().skill_by_name("retieji").unwrap();
    e.state_core_mut().skills.insert(
        (0, rtj),
        SkillInstance { owner: 0, enabled: true, once_used: false, expires_end_of_turn: false, rounds: 0 },
    );
    step_to_phase(&mut e, 0, Phase::Play);
    let sha = e.debug_place(0, "sha");
    let hidden_1 = e.debug_place(1, "tao");
    assert_accepted_or_pending(&submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }));

    // Seat 1 cannot see Seat 0's open choice options (`ChoiceView::Other`), nor can Seat 0 see Seat 1's hidden hand card
    let v0: PlayerView = e.view_for(0);
    assert!(!v0.visible_card_ids().contains(&hidden_1), "seat 0 must not see opponent hidden hand card");
    let sess1 = e.ai_session(1);
    assert!(matches!(sess1.choice(), Some(ChoiceView::Other { observer: 0, prompt: "retieji:confirm", .. })));
    assert_eq!(e.plan_action(1, &BoundedPlanner::new(16)).action, GameAction::NextPhase);
}
