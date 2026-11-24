use content_native_loader::boss_play_extension_pack;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::state::{BossStageState, GameMode, SkillInstance, Winner};
use engine_native::{ChoiceOption, Engine, GameAction, Phase};

use super::helpers::{
    answer_choice, assert_accepted_or_pending, assert_archive_roundtrip, drain_default,
    drain_one_default, move_card_to_deck_top, move_card_to_hand, remove_card_from_all_zones,
    step_to_phase, submit,
};

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

fn new_boss_4p_packs(seed: u64, packs: &[&ExtensionPack], fellows: Vec<u8>) -> Engine {
    Engine::new_with_extensions_mode(
        seed,
        4,
        packs,
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
fn round4_retieji_real_judge_guicai_tiandu_and_seal_after_judge() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = new_boss_4p_packs(7401, &[&pack], vec![]);
    e.assign_general_named(0, "boss_lvbu3").unwrap();
    e.assign_general_named(1, "司马懿").unwrap(); // 鬼才 + 反馈 (non-locked!)
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();
    let rtj = e.catalog().skill_by_name("retieji").unwrap();
    let td = e.catalog().skill_by_name("天妒").unwrap();
    for def in [rtj, td] {
        e.state_core_mut().skills.insert(
            (0, def),
            SkillInstance {
                owner: 0,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
    }

    step_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);

    // Top of deck is Heart, but 司马懿 holds a Club card to rewrite judge -> Club,
    // plus another Club card to choose whether to give, plus a non-Club Shan!
    let heart_judge = find_off_deck_or_deck_card_matching(&e, |_, d| d.suit == Suit::Heart);
    move_card_to_deck_top(&mut e, heart_judge);

    let club_rewrite = find_off_deck_or_deck_card_matching(&e, |i, d| {
        d.suit == Suit::Club && i != heart_judge
    });
    let club_give = find_off_deck_or_deck_card_matching(&e, |i, d| {
        d.suit == Suit::Club && i != heart_judge && i != club_rewrite
    });
    let diamond_shan = find_off_deck_or_deck_card_matching(&e, |_, d| {
        d.kind == CardKind::Shan && d.suit == Suit::Diamond
    });
    let heart_hand = find_off_deck_or_deck_card_matching(&e, |i, d| {
        d.suit == Suit::Heart && i != heart_judge
    });
    move_card_to_hand(&mut e, 1, club_rewrite);
    move_card_to_hand(&mut e, 1, club_give);
    move_card_to_hand(&mut e, 1, diamond_shan);
    move_card_to_hand(&mut e, 1, heart_hand);
    let sha_a = e.debug_place(0, "sha");

    // Branch A: Confirm retieji -> real JudgeMachine opens gui_dao_rewrite (target unsealed!),
    // rewrites Heart -> Club, opens tian_du_gain (target STILL unsealed!), THEN seals target
    // and asks retieji:give matching rewritten Club suit!
    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_a,
            targets: vec![1],
        },
    ));
    let c_conf = e.open_choice().expect("retieji:confirm").clone();
    assert_eq!(c_conf.spec.prompt, "retieji:confirm");
    answer_choice(&mut e, vec![ChoiceOption::Confirm]);

    let c_rw = e.open_choice().expect("gui_dao_rewrite during retieji judge").clone();
    assert_eq!(c_rw.spec.prompt, "gui_dao_rewrite");
    assert_eq!(c_rw.observer, 1);
    assert!(
        !e.catalog().is_tieji_sealed(e.state_core(), 1),
        "target MUST NOT be sealed during retieji judge"
    );
    let mut e = assert_archive_roundtrip(&e, &[&pack]);
    answer_choice(&mut e, vec![ChoiceOption::Card(club_rewrite)]);

    let c_td = e.open_choice().expect("tian_du_gain on retieji judge").clone();
    assert_eq!(c_td.spec.prompt, "tian_du_gain");
    assert_eq!(c_td.observer, 0);
    assert!(
        !e.catalog().is_tieji_sealed(e.state_core(), 1),
        "target MUST stay unsealed until judge finishes"
    );
    answer_choice(&mut e, vec![ChoiceOption::Confirm]);
    assert!(
        e.state_core().player(0).hand.contains(&club_rewrite),
        "owner gained rewritten judge card via tiandu"
    );

    assert!(
        e.catalog().is_tieji_sealed(e.state_core(), 1),
        "target MUST be sealed AFTER retieji judge completes"
    );
    let c_give = e.open_choice().expect("retieji:give after judge").clone();
    assert_eq!(c_give.spec.prompt, "retieji:give");
    assert!(
        c_give.spec.options.contains(&ChoiceOption::Card(club_give)),
        "effective rewritten suit Club must be offered"
    );
    assert!(
        !c_give.spec.options.contains(&ChoiceOption::Card(heart_hand)),
        "pre-rewrite Heart suit must NOT be offered after guicai rewrite"
    );
    // Cancel giving -> directHit + +1 damage (2 total) AND 反馈 sealed!
    answer_choice(&mut e, Vec::new());
    let mut saw_fankui = false;
    let mut saw_shan = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt.contains("shan") {
            saw_shan = true;
        } else if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Confirm) {
            saw_fankui = true;
        }
        drain_one_default(&mut e);
    }
    assert!(!saw_shan, "declining retieji:give makes Sha directHit");
    assert!(!saw_fankui, "retieji seal blocks target non-locked 反馈");
    assert_eq!(e.state_core().player(1).hp, 1, "司马懿 3 HP - 2 dmg = 1 HP");
}

#[test]
fn round4_l03_retieji_choose_to_discard_and_direct_hit_without_plus_one() {
    let l03 = boss_play_extension_pack("noname-liuli", &[]);
    let mut e = new_boss_4p_packs(7402, &[&l03], vec![]);
    e.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e.assign_general_named(1, "夏侯惇").unwrap(); // 刚烈 (non-locked!)
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();
    let rtj = e.catalog().skill_by_name("retieji").unwrap();
    e.state_core_mut().skills.insert(
        (0, rtj),
        SkillInstance {
            owner: 0,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );

    step_to_phase(&mut e, 0, Phase::Play);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);

    // Branch A: matching Heart equip in target equip zone -> L03 prompts retieji:discard (NOT :give!),
    // target discards equip to discard pile (NOT owner hand), and can still dodge Sha with Shan!
    let heart_judge = find_off_deck_or_deck_card_matching(&e, |_, d| d.suit == Suit::Heart);
    let heart_equip = find_off_deck_or_deck_card_matching(&e, |i, d| {
        d.suit == Suit::Heart && d.kind.equip_slot().is_some() && i != heart_judge
    });
    let diamond_shan = find_off_deck_or_deck_card_matching(&e, |_, d| {
        d.kind == CardKind::Shan && d.suit == Suit::Diamond
    });
    move_card_to_deck_top(&mut e, heart_judge);
    remove_card_from_all_zones(&mut e, heart_equip);
    e.state_core_mut().player_mut(1).equips.push(heart_equip);
    move_card_to_hand(&mut e, 1, diamond_shan);
    let sha_a = e.debug_place(0, "sha");

    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_a,
            targets: vec![1],
        },
    ));
    let c_conf = e.open_choice().expect("retieji:confirm").clone();
    assert_eq!(c_conf.spec.prompt, "retieji:confirm");
    answer_choice(&mut e, vec![ChoiceOption::Confirm]);

    assert!(e.catalog().is_tieji_sealed(e.state_core(), 1));
    let c_disc = e.open_choice().expect("L03 retieji:discard").clone();
    assert_eq!(c_disc.spec.prompt, "retieji:discard");
    assert_eq!(c_disc.observer, 1);
    assert!(c_disc.spec.options.contains(&ChoiceOption::Card(heart_equip)));
    let mut e = assert_archive_roundtrip(&e, &[&l03]);
    assert_eq!(
        e.plan_action(1, &BoundedPlanner::new(32)).action,
        GameAction::Answer {
            choice_id: c_disc.id,
            selection: vec![ChoiceOption::Card(heart_equip)],
        }
    );
    answer_choice(&mut e, vec![ChoiceOption::Card(heart_equip)]);
    assert!(
        e.state_core().discard.contains(&heart_equip),
        "L03 chooseToDiscard moves chosen card to discard"
    );
    assert!(
        !e.state_core().player(0).hand.contains(&heart_equip),
        "L03 retieji must NOT give card to owner"
    );
    assert!(
        !e.state_core().player(1).equips.contains(&heart_equip),
        "discarded equip removed from target equips"
    );
    let c_shan = e.open_choice().expect("target can dodge after L03 discard").clone();
    assert_eq!(c_shan.spec.prompt, "respond_shan");
    answer_choice(&mut e, vec![ChoiceOption::Card(diamond_shan)]);
    drain_default(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "dodged Sha -> 0 damage");

    // Branch B1: matching suit exists -> target cancels retieji:discard -> directHit ONLY (1 dmg, NO +1!)
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(1);
    let club_judge = find_off_deck_or_deck_card_matching(&e, |_, d| d.suit == Suit::Club);
    let club_hand = find_off_deck_or_deck_card_matching(&e, |i, d| {
        d.suit == Suit::Club && i != club_judge
    });
    let diamond_shan = find_off_deck_or_deck_card_matching(&e, |_, d| {
        d.kind == CardKind::Shan && d.suit == Suit::Diamond
    });
    move_card_to_deck_top(&mut e, club_judge);
    move_card_to_hand(&mut e, 1, club_hand);
    move_card_to_hand(&mut e, 1, diamond_shan);
    let sha_b = e.debug_place(0, "sha");

    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_b,
            targets: vec![1],
        },
    ));
    let mut saw_shan_b1 = false;
    let mut saw_jx_b1 = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "retieji:confirm" {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt == "retieji:discard" {
            answer_choice(&mut e, Vec::new());
        } else if c.spec.prompt.contains("shan") {
            saw_shan_b1 = true;
            drain_one_default(&mut e);
        } else if c.observer == 1 && c.spec.options.contains(&ChoiceOption::Confirm) {
            saw_jx_b1 = true;
            drain_one_default(&mut e);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(!saw_shan_b1, "L03 retieji cancel makes Sha directHit");
    assert!(!saw_jx_b1, "L03 retieji still seals target 刚烈");
    assert_eq!(e.state_core().player(1).hp, 3, "L03 retieji has NO +1 baseDamage (4 -> 3)");
    assert!(e.state_core().player(1).hand.contains(&club_hand));

    // Branch B2: target has NO matching suit card -> directHit ONLY (3 -> 2, NO +1!)
    e.state_core_mut().turn.sha_used_play_phase = false;
    e.debug_clear_hand(1);
    let spade_judge = find_off_deck_or_deck_card_matching(&e, |_, d| d.suit == Suit::Spade);
    let heart_shan = find_off_deck_or_deck_card_matching(&e, |_, d| {
        d.kind == CardKind::Shan && d.suit == Suit::Heart
    });
    move_card_to_deck_top(&mut e, spade_judge);
    move_card_to_hand(&mut e, 1, heart_shan);
    let sha_c = e.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![1],
        },
    ));
    let mut saw_shan_b2 = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "retieji:confirm" {
            answer_choice(&mut e, vec![ChoiceOption::Confirm]);
        } else if c.spec.prompt.contains("shan") {
            saw_shan_b2 = true;
            drain_one_default(&mut e);
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(!saw_shan_b2, "L03 retieji with no matching card makes Sha directHit");
    assert_eq!(e.state_core().player(1).hp, 2, "L03 empty-match retieji has NO +1 damage (3 -> 2)");
}

#[test]
fn round4_u02_vs_l03_coexistence_respects_first_registered_retieji_variant() {
    let l03 = boss_play_extension_pack("noname-liuli", &[]);
    let u02_after_l03 = boss_play_extension_pack("noname", &[&l03]);
    let mut e_l03_first = new_boss_4p_packs(7403, &[&l03, &u02_after_l03], vec![]);
    e_l03_first.assign_general_named(0, "boss_duanyuzhongda").unwrap();
    e_l03_first.assign_general_named(1, "曹操").unwrap();
    e_l03_first.assign_general_named(2, "刘备").unwrap();
    e_l03_first.assign_general_named(3, "孙权").unwrap();
    let rtj = e_l03_first.catalog().skill_by_name("retieji").unwrap();
    e_l03_first.state_core_mut().skills.insert(
        (0, rtj),
        SkillInstance {
            owner: 0,
            enabled: true,
            once_used: false,
            expires_end_of_turn: false,
            rounds: 0,
        },
    );
    step_to_phase(&mut e_l03_first, 0, Phase::Play);
    e_l03_first.debug_clear_hand(1);
    let heart_j = find_off_deck_or_deck_card_matching(&e_l03_first, |_, d| d.suit == Suit::Heart);
    let heart_c = find_off_deck_or_deck_card_matching(&e_l03_first, |i, d| {
        d.suit == Suit::Heart && i != heart_j
    });
    move_card_to_deck_top(&mut e_l03_first, heart_j);
    move_card_to_hand(&mut e_l03_first, 1, heart_c);
    let sha = e_l03_first.debug_place(0, "sha");
    assert_accepted_or_pending(&submit(
        &mut e_l03_first,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    ));
    answer_choice(&mut e_l03_first, vec![ChoiceOption::Confirm]);
    assert_eq!(
        e_l03_first.open_choice().map(|c| c.spec.prompt),
        Some("retieji:discard"),
        "L03 pack loaded first must preserve L03 chooseToDiscard retieji even with U02 coexisting"
    );
}

#[test]
fn round4_boss_lvbu1_post_transform_first_and_second_deck_wash_wins() {
    let pack = boss_play_extension_pack("noname", &[]);
    let mut e = new_boss_4p_packs(7404, &[&pack], vec![]);
    e.assign_general_named(0, "boss_lvbu1").unwrap();
    e.assign_general_named(1, "曹操").unwrap();
    e.assign_general_named(2, "刘备").unwrap();
    e.assign_general_named(3, "孙权").unwrap();

    // Seat 1 drops Boss below 5 HP via a real Sha BEFORE any deck wash -> Boss transforms
    // into boss_lvbu2 and removes `boss_baonu` (`!has_baonu`).
    step_to_phase(&mut e, 1, Phase::Play);
    e.state_core_mut().player_mut(0).hp = 5;
    e.debug_clear_hand(0);
    let sha = e.debug_place(1, "sha");
    assert_accepted_or_pending(&submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    ));
    let mut transformed = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_baonu:form" {
            transformed = true;
            answer_choice(&mut e, vec![ChoiceOption::Tag(0)]); // boss_lvbu2
        } else {
            drain_one_default(&mut e);
        }
    }
    assert!(transformed, "boss_lvbu1 transformed at <= 4 HP before any wash");
    let baonu_def = e.catalog().skill_by_name("boss_baonu").unwrap();
    assert!(!e.state_core().has_skill(0, baonu_def), "boss_baonu removed after transform");
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, "q4_boss_baonuwash_triggered".to_string())).copied().unwrap_or(0),
        0,
        "no wash has occurred yet"
    );

    // 1st deck wash AFTER transform (during Seat 1's Draw phase): must mark
    // `q4_boss_baonuwash_triggered = 1` unconditionally without reopening boss_baonu:form!
    step_to_phase(&mut e, 1, Phase::Judge);
    {
        let core = e.state_core_mut();
        while core.deck.len() > 1 {
            let c = core.deck.pop().unwrap();
            core.discard.push(c);
        }
    }
    assert_accepted_or_pending(&submit(&mut e, 1, GameAction::NextPhase)); // -> Draw -> wash #1
    let mut saw_form_on_wash1 = false;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "boss_baonu:form" {
            saw_form_on_wash1 = true;
        }
        drain_one_default(&mut e);
    }
    assert!(!saw_form_on_wash1, "already transformed Boss must not re-prompt boss_baonu:form");
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, "q4_boss_baonuwash_triggered".to_string())).copied().unwrap_or(0),
        1,
        "1st post-transform deck wash must mark q4_boss_baonuwash_triggered = 1"
    );
    assert!(e.outcome().is_none(), "1st wash must not end the game");

    // Archive save/restore between wash 1 and wash 2 preserves wash state!
    let mut e = assert_archive_roundtrip(&e, &[&pack]);

    // 2nd deck wash -> sets `q4_boss_baonuwash_game_over`, Boss wins at TurnPhaseAfter!
    step_to_phase(&mut e, 1, Phase::Play);
    assert!(e.outcome().is_none());
    {
        let core = e.state_core_mut();
        while !core.deck.is_empty() {
            let c = core.deck.pop().unwrap();
            core.discard.push(c);
        }
    }
    let wuzhong = find_off_deck_or_deck_card_matching(&e, |_, d| d.kind == CardKind::Wuzhong);
    move_card_to_hand(&mut e, 1, wuzhong);
    assert_accepted_or_pending(&submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    ));
    drain_default(&mut e);
    assert!(e.outcome().is_none(), "victory waits until TurnPhaseAfter");
    while e.outcome().is_none() && e.turn().current == 1 {
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        drain_default(&mut e);
    }
    assert_eq!(
        e.outcome().map(|o| o.winner),
        Some(Winner::Boss { boss_won: true }),
        "2nd deck wash after HP-transform must grant Boss victory at TurnPhaseAfter"
    );
}
