//! Golden coverage for MOBILE 界董卓「酒池」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const REJIUCHI: u32 = 3_056_787_053;
const ROULIN: u32 = 3_168_437_611;
const BENGHUAI: u32 = 3_365_741_831;
const BAONUE: u32 = 3_659_854_073;
const RE_DONGZHUO: u32 = 3_597_733_258;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/mobile/rejiuchi.skill"),
    "\n",
    include_str!("../../../content/original/ported/shenhua/benghuai.skill")
);
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn setup(seed: u64) -> Setup {
    let package = content_compiler::compile_source_named("rejiuchi-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(REJIUCHI),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(BENGHUAI),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        target,
    }
}

fn inject_sha(state: &mut GameState, owner: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(191_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_abi191_and_complete_re_dongzhuo() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 252);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "shenhua"));
    assert!(package.view_as_index.iter().any(|view| {
        view.skill_def_id == REJIUCHI
            && view.as_kind == "jiu"
            && view.window == "play_phase"
            && view.filter == "spade"
    }));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REJIUCHI)
        .unwrap();
    assert_eq!(
        trigger.filter,
        "owner_source_jiu_boosted_sha_damage_skill_3365741831_enabled"
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_DONGZHUO)
        .unwrap();
    assert!(character.is_lord);
    assert_eq!(character.max_hp, 8);
    assert_eq!(character.skills, [REJIUCHI, ROULIN, BENGHUAI, BAONUE]);
}

#[test]
fn jiu_boosted_sha_damage_disables_benghuai_until_next_turn() {
    let mut setup = setup(2_026_072_505);
    let sha = inject_sha(&mut setup.state, setup.owner);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: setup.owner,
            target: setup.target,
            card: sha,
            amount: 2,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: rule_api::damage_card_flag::JIU_BOOSTED,
        }],
    )
    .unwrap();
    let benghuai = skill_id(tx.state(), setup.owner, BENGHUAI);
    assert!(tx.state().skills.get(benghuai).unwrap().turn_disabled);
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(benghuai).unwrap()));

    tx.state_mut().players.get_mut(setup.owner).unwrap().hp = 3;
    tx.state_mut().players.get_mut(setup.target).unwrap().hp = 2;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PhaseEnded {
            current: setup.owner,
            phase: TurnPhaseTag::End,
            damage_sources: 0,
            damaged_players: 0,
            card_targeted_other: false,
            current_dealt_damage_other: false,
            phase_targeted_players: 0,
            phase_hp_changed_players: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());

    tx.state_mut().turn.phase = Phase::End;
    let state = tx.commit().state;
    let (state, result) = apply_command(
        state,
        Command {
            actor: setup.owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let benghuai = skill_id(&state, setup.owner, BENGHUAI);
    assert!(!state.skills.get(benghuai).unwrap().turn_disabled);
    assert!(state.skill_effectively_enabled(state.skills.get(benghuai).unwrap()));
}

#[test]
fn ordinary_sha_damage_keeps_benghuai_enabled() {
    let mut setup = setup(2_026_072_506);
    let sha = inject_sha(&mut setup.state, setup.owner);
    let mut tx = Transaction::begin(setup.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: setup.owner,
            target: setup.target,
            card: sha,
            amount: 1,
            nature: rule_api::damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    let benghuai = skill_id(tx.state(), setup.owner, BENGHUAI);
    assert!(!tx.state().skills.get(benghuai).unwrap().turn_disabled);
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(benghuai).unwrap()));
}
