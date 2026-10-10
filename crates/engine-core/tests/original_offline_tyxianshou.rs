//! Golden for 桃张达「献首」rewritten from legacy JavaScript.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::ContinuationKind;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const TYXIANSHOU: u32 = 4_281_024_996;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/tyxianshou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (Transaction, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let seats = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let existing = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, existing).unwrap();
    (tx, seats)
}

fn grant(tx: &mut Transaction, player: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(TYXIANSHOU),
        }],
    )
    .unwrap();
}

fn resolve(
    tx: &Transaction,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(tx.state().revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(tx.state().resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_exposes_forced_source_death_recovery_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert_eq!(package.skill_index.len(), 838);
    assert_eq!(package.character_index.len(), 382);
    assert_eq!(package.trigger_index.len(), 587);
    assert_eq!(package.active_index.len(), 137);
    assert_eq!(package.modifier_index.len(), 128);
    assert_eq!(package.modules.len(), 724);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TYXIANSHOU && skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TYXIANSHOU)
        .unwrap();
    assert_eq!(trigger.event, "death");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(!trigger.optional);

    let compiled = content_compiler::compile_source_named("tyxianshou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 160);
    assert_eq!(
        package
            .modules
            .iter()
            .find(|module| module.name == "tyxianshou_death"),
        compiled.modules.first()
    );
}

#[test]
fn killer_must_choose_another_living_player_to_recover_two() {
    let (mut tx, seats) = setup(2_210_204_269);
    let (killer, victim, recipient) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, killer);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: killer,
            target: recipient,
            amount: 2,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    let before = tx.state().players.get(recipient).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseOtherDrawOrRecover
    );

    let (_, rejected) = resolve(&tx, killer, Vec::new(), true);
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let (state, accepted) = resolve(&tx, killer, vec![recipient.raw()], false);
    assert!(
        matches!(accepted.result, CommandResult::Accepted { .. }),
        "{:?}",
        accepted.result
    );
    assert_eq!(state.players.get(recipient).unwrap().hp, before + 2);
    assert!(!state.has_pending_choice());
}

#[test]
fn full_hp_target_remains_selectable_and_other_killer_does_not_trigger() {
    let (mut tx, seats) = setup(2_210_204_270);
    let (killer, victim, recipient) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, killer);
    let before = tx.state().players.get(recipient).unwrap().hp;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::KillPlayer {
                player: victim,
                killer,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let (state, accepted) = resolve(&tx, killer, vec![recipient.raw()], false);
    assert!(
        matches!(accepted.result, CommandResult::Accepted { .. }),
        "{:?}",
        accepted.result
    );
    assert_eq!(state.players.get(recipient).unwrap().hp, before);

    let (mut tx, seats) = setup(2_210_204_271);
    let (owner, victim, other_killer) = (seats[0], seats[1], seats[2]);
    grant(&mut tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::KillPlayer {
            player: victim,
            killer: other_killer,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
}
