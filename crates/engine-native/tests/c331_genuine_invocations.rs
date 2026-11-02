use super::c331_source_predicates::{finish, restore, same_final};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::state::{BossStageState, GameMode};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
pub(super) fn emitter() -> ExtensionPack {
    let p = content_compiler::compile_source_named(
        "c9g-emission",
        "skill c9g_emit(99109601) { active play_phase targets 0 0 { effect deal_damage(3,2,1); } }",
    )
    .unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c9g-emission",
            version: 1,
            description: "real command cause",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&p)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
pub(super) fn fixture(p: &ExtensionPack, name: &str) -> Engine {
    let mut e = Engine::new_with_extensions(15001, 4, &[p]).unwrap();
    for id in 0..4 {
        e.debug_clear_hand(id);
        e.debug_set_hp(id, 4);
    }
    e.debug_set_turn(0, Phase::Play, 1);
    e.state_core_mut().mode = GameMode::Boss {
        boss: 0,
        chongzheng: 3,
        fellows: vec![1],
        stage: BossStageState::default_none(),
    };
    e.debug_grant(0, "c9g_emit");
    e.debug_grant(if name == "章武" { 1 } else { 0 }, name);
    e
}
pub(super) fn fire(e: &mut Engine) {
    let skill = e.catalog().skill_by_name("c9g_emit").unwrap();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::ActivateSkill { skill },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
#[test]
fn c9g_zhangwu_holder_cancel_has_real_invocation_without_activation_log() {
    let p = emitter();
    let mut e = fixture(&p, "章武");
    e.debug_place(1, "sha");
    fire(&mut e);
    let c = e
        .open_choice()
        .expect("actual optional holder cost")
        .clone();
    assert_eq!(c.spec.prompt, "c331_zhangwu_holder");
    assert!(!e.log().iter().any(|x| matches!(
        x,
        engine_native::LogEvent::SkillTriggered {
            skill: "章武", ..
        }
    )));
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        let out = branch.submit(GameCommand {
            actor: c.observer,
            expected_revision: branch.revision(),
            dedup_id: branch.save().last_dedup + 1,
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        finish(branch);
    }
    same_final(&e, &copy);
    assert_eq!(e.debug_hand(1).len(), 1);
}
#[test]
fn c9g_jiding_dodge_finishes_card_then_removes_actual_definition() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    e.debug_place(2, "shan");
    let def = e.catalog().skill_by_name("急叮").unwrap();
    fire(&mut e);
    let c = e.open_choice().expect("real virtual Sha Dodge").clone();
    let card = c
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Card(_)))
        .copied()
        .unwrap();
    assert!(e.state_core().has_skill(0, def));
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![card],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    finish(&mut e);
    assert!(
        !e.state_core().has_skill_any_state(0, def),
        "remove after complete dodged card subtree"
    );
}
#[test]
fn c9g_jiding_has_persisted_named_invocation_before_card() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    e.set_time_slice(Some(1));
    fire(&mut e);
    let mut found = false;
    for _ in 0..500 {
        let s = e.save();
        found |= format!("{:?}", s.resolution.stack).contains("SkillInvocation");
        if s.resolution.stack.is_empty() && !s.segment_open {
            break;
        }
        e.resume();
    }
    assert!(found, "actual invocation frame, not scalar activation log");
}
#[test]
fn c9g_invocation_every_phase_and_dodge_restore() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    e.debug_place(2, "shan");
    e.set_time_slice(Some(1));
    fire(&mut e);
    let mut phases = std::collections::BTreeSet::new();
    let mut checked = 0;
    for _ in 0..800 {
        let s = e.save();
        for f in &s.resolution.stack {
            if let engine_native::event::EventData::SkillInvocation(d) = &f.data {
                phases.insert(d.phase.tag());
            }
        }
        if s.resolution.stack.is_empty() && !s.segment_open {
            break;
        }
        let mut copy = restore(&e, &[&p]);
        finish(&mut copy);
        let mut control = e.clone();
        finish(&mut control);
        same_final(&control, &copy);
        checked += 1;
        if let Some(c) = e.open_choice().cloned() {
            let card = c
                .spec
                .options
                .iter()
                .find(|x| matches!(x, ChoiceOption::Card(_)))
                .copied();
            let selection = card.map(|v| vec![v]).unwrap_or_default();
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: e.save().last_dedup + 1,
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection,
                },
            });
            assert!(!matches!(out, CommandOutcome::Rejected(_)));
        } else {
            e.resume();
        }
    }
    assert!(checked > 10);
    assert!(phases.is_superset(&[0, 1, 2, 3, 4].into_iter().collect()));
}
#[test]
fn c9g_zhangwu_actual_two_card_equipment_cost_and_source_cancel() {
    let p = emitter();
    let mut e = fixture(&p, "章武");
    e.debug_set_max_hp(2, 20);
    e.debug_set_hp(2, 20);
    let hand = e.debug_place(1, "sha");
    let equip = e.debug_place(1, "zhuge");
    e.state_core_mut()
        .player_mut(1)
        .hand
        .retain(|c| *c != equip);
    e.state_core_mut().player_mut(1).equips.push(equip);
    e.debug_place(2, "tao");
    e.debug_place(2, "shan");
    fire(&mut e);
    let c = e.open_choice().unwrap().clone();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(hand), ChoiceOption::Card(equip)],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.prompt, "c331_zhangwu_source");
    assert_eq!((c.spec.min, c.spec.max), (2, 2));
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        let out = branch.submit(GameCommand {
            actor: 2,
            expected_revision: branch.revision(),
            dedup_id: branch.save().last_dedup + 1,
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        finish(branch);
    }
    same_final(&e, &copy);
    assert_eq!(e.debug_hp(2), 18);
    assert!(e.state_core().discard.contains(&equip));
    assert_eq!(
        e.log()
            .iter()
            .filter(|x| matches!(
                x,
                engine_native::LogEvent::SkillTriggered {
                    skill: "章武", ..
                }
            ))
            .count(),
        1
    );
}
#[test]
fn c9g_kingdoms_revealed_wei_shu_are_source_enemies() {
    let mut e = Engine::new(15004, 3);
    let a = (0..e.catalog().general_count())
        .find(|id| {
            e.catalog()
                .general_meta(*id as u16)
                .is_some_and(|g| g.1 == engine_native::catalog::Faction::Wei)
        })
        .unwrap() as u16;
    let b = (0..e.catalog().general_count())
        .find(|id| {
            e.catalog()
                .general_meta(*id as u16)
                .is_some_and(|g| g.1 == engine_native::catalog::Faction::Shu)
        })
        .unwrap() as u16;
    e.debug_set_general_def(0, a);
    e.debug_set_general_def(1, b);
    e.state_core_mut().mode = GameMode::Kingdoms;
    for actor in [0, 1] {
        e.debug_set_turn(actor, Phase::Play, 1);
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: e.save().last_dedup + 1,
            action: GameAction::RevealGeneral { slot: 0 },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    assert_eq!(
        e.state_core().source_is_enemy_of(e.catalog(), 0, 1),
        Some(true)
    );
}
#[test]
fn c9g_yuance_opens_two_real_options_without_activation_log() {
    let p = emitter();
    let mut e = fixture(&p, "远策");
    fire(&mut e);
    let c = e.open_choice().expect("actual options1/2/cancel");
    assert_eq!(c.spec.prompt, "c331_yuance");
    assert_eq!(c.spec.options.len(), 2);
    assert!(!e.log().iter().any(|x| matches!(
        x,
        engine_native::LogEvent::SkillTriggered {
            skill: "远策", ..
        }
    )));
}
#[test]
fn c9g_jidian_opens_actual_target_choice() {
    let p = emitter();
    let mut e = fixture(&p, "极点");
    e.debug_grant(2, "极点");
    fire(&mut e);
    let c = e.open_choice().expect("actual optional target");
    assert_eq!(c.spec.prompt, "c331_jidian_target");
    assert!(
        c.spec.options.contains(&ChoiceOption::Player(2)),
        "source allows owner when distance from damaged actor permits"
    );
}
#[test]
fn c9g_jiding_genuine_card_damage_heals_actual_named_rushou() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    let id = (e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len()
        + 15) as u16;
    e.debug_set_general_def(3, id);
    e.debug_set_hp(3, 2);
    fire(&mut e);
    finish(&mut e);
    assert_eq!(
        e.debug_hp(3),
        3,
        "one genuine invocation/CardUsed damageEnd heals actual living boss_jinshenrushou"
    );
}
pub(super) fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
#[test]
fn c9g_yuance_four_colours_options_and_cancel() {
    use engine_native::catalog::Suit;
    use engine_native::event::YuanceOption as O;
    for (option, suit, expected) in [
        (O::LoseHp, Suit::Spade, (2, 3)),
        (O::LoseHp, Suit::Heart, (3, 2)),
        (O::Recover, Suit::Spade, (3, 4)),
        (O::Recover, Suit::Heart, (4, 3)),
    ] {
        let p = emitter();
        let mut e = fixture(&p, "远策");
        e.debug_set_hp(2, 3);
        e.debug_set_deck_top_by(|c| c.suit == suit);
        fire(&mut e);
        let mut copy = restore(&e, &[&p]);
        for branch in [&mut e, &mut copy] {
            answer(branch, vec![ChoiceOption::Control(option)]);
            finish(branch);
        }
        same_final(&e, &copy);
        assert_eq!((e.debug_hp(1), e.debug_hp(2)), expected);
    }
    let p = emitter();
    let mut e = fixture(&p, "远策");
    fire(&mut e);
    let before = e.state_core().rng.clone();
    answer(&mut e, vec![]);
    finish(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(format!("{:?}", e.state_core().rng), format!("{before:?}"));
}
#[test]
fn c9g_ordinary_tiandao_nonfirst_equipment_flips_colour_and_restores() {
    use engine_native::catalog::Suit;
    let p = emitter();
    let mut e = fixture(&p, "远策");
    e.debug_grant(3, "天道");
    e.debug_place(3, "tao");
    let equip = e.debug_place(3, "armor");
    assert!(matches!(
        e.catalog().cards[equip as usize].suit,
        Suit::Spade | Suit::Club
    ));
    e.state_core_mut()
        .player_mut(3)
        .hand
        .retain(|c| *c != equip);
    e.state_core_mut().player_mut(3).equips.push(equip);
    e.debug_set_deck_top_by(|c| c.suit == Suit::Heart);
    let old = *e.state_core().deck.last().unwrap();
    fire(&mut e);
    answer(
        &mut e,
        vec![ChoiceOption::Control(
            engine_native::event::YuanceOption::LoseHp,
        )],
    );
    let c = e.open_choice().expect("genuine Tiandao rewriter");
    assert_eq!(c.observer, 3);
    assert_ne!(
        c.spec
            .options
            .iter()
            .position(|o| *o == ChoiceOption::Card(equip)),
        Some(1)
    );
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![ChoiceOption::Card(equip)]);
        finish(branch);
    }
    same_final(&e, &copy);
    assert_eq!(e.debug_hp(1), 2);
    assert_eq!(e.debug_hp(2), 4);
    assert!(e.debug_hand(3).contains(&old));
    assert!(e.state_core().discard.contains(&equip));
}
#[test]
fn c9g_jidian_red_does_not_mute_next_unrelated_reaction() {
    let p = emitter();
    let mut e = fixture(&p, "极点");
    e.debug_grant(2, "极点");
    e.debug_set_deck_top_by(|c| c.suit == engine_native::catalog::Suit::Heart);
    fire(&mut e);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    finish(&mut e);
    assert!(!e.state_core().turn.jidian_chain);
    let skill = e.catalog().skill_by_name("c9g_emit").unwrap();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: GameAction::ActivateSkill { skill },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "c331_jidian_target");
}
