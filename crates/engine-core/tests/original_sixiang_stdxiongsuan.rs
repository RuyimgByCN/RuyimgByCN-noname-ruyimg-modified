//! Golden coverage for 标李傕「凶算」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const STD_LIJUE: u32 = 2_852_318_654;
const STDXIONGSUAN: u32 = 3_084_450_928;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdxiongsuan.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn phase_prepare(player: PlayerId) -> DomainEvent {
    DomainEvent::TurnStateChanged {
        current: player,
        phase: TurnPhaseTag::Prepare,
    }
}

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_074_403, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, tied, lower) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDXIONGSUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    for (player, hp) in [(owner, 4), (tied, 4), (lower, 3)] {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = hp;
        participant.max_hp = participant.max_hp.max(hp as u32);
    }
    (tx, owner, tied, lower)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn package_and_runtime_force_same_hp_targets_then_damage_in_seat_order() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == STDXIONGSUAN && skill.name == "stdxiongsuan" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == STD_LIJUE)
        .unwrap();
    assert_eq!(character.name_key, "std_lijue");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 5);
    assert_eq!(character.skills, [STDXIONGSUAN]);

    let compiled = content_compiler::compile_source_named("stdxiongsuan", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 135);
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let (mut tx, owner, tied, lower) = setup();
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let committed = tx.commit();
    let choice = committed.output.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!((choice.min_count, choice.max_count), (1, 2));
    let options: Vec<_> = choice.options.iter().map(|option| option.token).collect();
    assert_eq!(options, [owner.raw(), tied.raw()]);
    assert!(!options.contains(&lower.raw()));

    let (state, output) = resolve(committed.state, owner, vec![tied.raw(), owner.raw()]);
    assert!(matches!(output.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(tied).unwrap().hp, 3);
    assert_eq!(state.players.get(lower).unwrap().hp, 3);
    let damaged: Vec<_> = output
        .events
        .iter()
        .filter_map(|event| match event {
            DomainEvent::DamageDealt { source, target, .. } if *source == owner => Some(*target),
            _ => None,
        })
        .collect();
    assert_eq!(damaged, [owner, tied]);
}

#[test]
fn trigger_does_not_fire_when_owner_is_below_the_table_maximum() {
    let (mut tx, owner, _, higher) = setup();
    tx.state_mut().players.get_mut(higher).unwrap().hp = 5;
    let effects = dispatch_for_events(
        &mut tx,
        &[phase_prepare(owner)],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
}
