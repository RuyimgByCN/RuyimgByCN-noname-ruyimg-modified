//! Golden for DIY「天机」的他人濒死、可选与限定技语义。

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

const NSTIANJI: u32 = 2_425_767_664;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/diy/diy.npack.json");

fn isolated_state() -> (Transaction, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NSTIANJI && !skill.locked));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: 20_260_715,
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
        skill_def: SkillDefId::new(NSTIANJI),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn offer(tx: &mut Transaction, dying: PlayerId) -> Result<Vec<EffectRequest>, EffectError> {
    dispatch_for_events(
        tx,
        &[DomainEvent::EnteredDying {
            player: dying,
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
            expected_revision: ExpectedRevision(0),
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

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == NSTIANJI)
}

#[test]
fn declining_preserves_the_skill_and_accepting_applies_all_effects_exactly_once() {
    let (mut tx, owner, other) = isolated_state();
    tx.state_mut().players.get_mut(other).unwrap().hp = -2;
    let owner_max_hp = tx.state().players.get(owner).unwrap().max_hp;
    let other_max_hp = tx.state().players.get(other).unwrap().max_hp;

    assert_eq!(
        offer(&mut tx, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 1);
    assert_eq!(declined.players.get(owner).unwrap().max_hp, owner_max_hp);
    assert_eq!(declined.players.get(other).unwrap().hp, -2);
    assert_eq!(declined.players.get(other).unwrap().max_hp, other_max_hp);
    assert!(has_skill(&declined, owner));

    let mut retry = Transaction::begin(declined);
    assert_eq!(
        offer(&mut retry, other).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&retry, owner, true, 2);
    assert_eq!(
        accepted.players.get(owner).unwrap().max_hp,
        owner_max_hp - 1
    );
    assert_eq!(accepted.players.get(other).unwrap().hp, 1);
    assert_eq!(
        accepted.players.get(other).unwrap().max_hp,
        other_max_hp + 1
    );
    assert!(!has_skill(&accepted, owner));
}

#[test]
fn self_dying_and_dead_owner_do_not_offer_tianji() {
    let (mut self_dying, owner, _) = isolated_state();
    self_dying.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert!(offer(&mut self_dying, owner).unwrap().is_empty());
    assert!(!self_dying.state().has_pending_choice());

    let (mut dead_owner, owner, other) = isolated_state();
    dead_owner.state_mut().players.get_mut(owner).unwrap().alive = false;
    dead_owner.state_mut().players.get_mut(other).unwrap().hp = 0;
    assert!(offer(&mut dead_owner, other).unwrap().is_empty());
    assert!(!dead_owner.state().has_pending_choice());
}
