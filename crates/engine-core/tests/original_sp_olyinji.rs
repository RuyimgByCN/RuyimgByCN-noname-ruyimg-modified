//! Golden for SP 刘焉「殷积」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLYINJI: u32 = 2_633_842_541;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn isolated(owner_hp: i32, other_hp: i32) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20_260_716),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_716,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLYINJI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = owner_hp;
    tx.state_mut().players.get_mut(other).unwrap().hp = other_hp;
    (tx, owner)
}

fn phase_end(tx: &mut Transaction, owner: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    )
}

fn resolve(
    tx: &Transaction,
    owner: PlayerId,
    option: u32,
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
                selections: vec![option],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn committed_package_exposes_locked_phase_end_choice() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 70);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == OLYINJI && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLYINJI)
        .unwrap();
    assert_eq!(
        (
            trigger.event.as_str(),
            trigger.filter.as_str(),
            trigger.optional
        ),
        (
            "phase_end",
            "owner_is_target_not_unique_max_current_hp",
            false
        )
    );
}

#[test]
fn non_unique_owner_must_choose_recover_or_gain_max_hp() {
    let (mut recover, owner) = isolated(2, 3);
    assert_eq!(
        phase_end(&mut recover, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let max_hp = recover.state().players.get(owner).unwrap().max_hp;
    let recovered = resolve(&recover, owner, 1, 1);
    assert_eq!(recovered.players.get(owner).unwrap().hp, 3);
    assert_eq!(recovered.players.get(owner).unwrap().max_hp, max_hp);

    let (mut gain_max, owner) = isolated(2, 3);
    assert_eq!(
        phase_end(&mut gain_max, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let max_hp = gain_max.state().players.get(owner).unwrap().max_hp;
    let increased = resolve(&gain_max, owner, 2, 2);
    assert_eq!(increased.players.get(owner).unwrap().hp, 2);
    assert_eq!(increased.players.get(owner).unwrap().max_hp, max_hp + 1);

    let (mut tied, owner) = isolated(3, 3);
    assert_eq!(
        phase_end(&mut tied, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
}

#[test]
fn unique_highest_owner_gets_no_choice_or_effect() {
    let (mut tx, owner) = isolated(4, 3);
    let before = tx.state().clone();
    let effects = phase_end(&mut tx, owner).unwrap();
    assert!(effects.is_empty(), "{effects:?}");
    assert!(!tx.state().has_pending_choice());
    assert_eq!(tx.state().players.get(owner), before.players.get(owner));
}
