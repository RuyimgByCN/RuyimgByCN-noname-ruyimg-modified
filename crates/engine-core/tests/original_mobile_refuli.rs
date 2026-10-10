//! Golden for mobile 廖化「伏枥」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const REFULI: u32 = 2_837_551_482;
const WU: u32 = 4_065_000_621;
const SHU: u32 = 3_601_555_604;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn isolated(other_hp: [i32; 3]) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let mut players = vec![owner];
    players.extend(
        engine
            .state()
            .players
            .alive_by_seat()
            .into_iter()
            .filter(|player| *player != owner),
    );
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, character_def) in players.iter().copied().zip([WU, WU, SHU, SHU]) {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character_def;
    }
    for (player, hp) in players.iter().copied().skip(1).zip(other_hp) {
        tx.state_mut().players.get_mut(player).unwrap().hp = hp;
    }
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REFULI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    (tx, owner)
}

fn offer(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> engine_core::state::GameState {
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn has_refuli(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == REFULI)
}

#[test]
fn committed_package_exposes_optional_limited_dying_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 7);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == REFULI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        ("dying", "owner_is_target", true)
    );
    assert!(package
        .modules
        .iter()
        .any(|module| module.code.windows(2).any(|bytes| {
            bytes
                == [
                    rule_vm::bytecode::op::HOST_QUERY,
                    rule_vm::bytecode::query_tag::UNIQUE_MAX_CURRENT_HP_AT,
                ]
        })));
}

#[test]
fn accepted_refuli_recovers_then_turns_over_only_when_strictly_highest() {
    let (mut tx, owner) = isolated([1, 1, 1]);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_refuli(&accepted, owner));
}

#[test]
fn tied_current_hp_does_not_turn_over_but_still_consumes_refuli() {
    let (mut tx, owner) = isolated([2, 1, 1]);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 2);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert!(!accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_refuli(&accepted, owner));
}

#[test]
fn declining_refuli_preserves_hp_orientation_and_limited_skill() {
    let (mut tx, owner) = isolated([1, 1, 1]);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 3);
    assert_eq!(declined.players.get(owner).unwrap().hp, 0);
    assert!(!declined.players.get(owner).unwrap().turned_over);
    assert!(has_refuli(&declined, owner));
}
