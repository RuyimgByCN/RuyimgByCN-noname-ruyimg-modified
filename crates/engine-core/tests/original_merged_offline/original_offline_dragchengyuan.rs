//! Golden for OFFLINE「承缘」的目标性别、可选与限定技语义。

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

const DRAGCHENGYUAN: u32 = 2_540_695_647;
const TYFULI: u32 = 3_185_405_820;
const MALE_CHARACTER: u32 = 1;
const FEMALE_CHARACTER: u32 = 7;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn isolated_state(skill_def: u32) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == skill_def && !skill.locked }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: 20_260_715,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));

    let owner = engine.state().turn.current;
    let others: Vec<_> = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let male = others[0];
    let female = others[1];
    let mut tx = Transaction::begin(engine.state().clone());
    for (player, character) in [
        (owner, MALE_CHARACTER),
        (male, MALE_CHARACTER),
        (female, FEMALE_CHARACTER),
    ] {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character;
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, male, female)
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

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn female_target_can_be_restored_to_max_and_declining_does_not_consume_the_skill() {
    let (mut tx, owner, male, female) = isolated_state(DRAGCHENGYUAN);
    tx.state_mut().players.get_mut(male).unwrap().hp = 0;
    assert!(offer(&mut tx, male).unwrap().is_empty());
    assert!(!tx.state().has_pending_choice());

    tx.state_mut().players.get_mut(female).unwrap().hp = -1;
    assert_eq!(
        offer(&mut tx, female).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 1);
    assert_eq!(declined.players.get(female).unwrap().hp, -1);
    assert!(has_skill(&declined, owner, DRAGCHENGYUAN));

    let mut retry = Transaction::begin(declined);
    assert_eq!(
        offer(&mut retry, female).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&retry, owner, true, 2);
    let target = accepted.players.get(female).unwrap();
    assert_eq!(target.hp, target.max_hp as i32);
    assert!(!has_skill(&accepted, owner, DRAGCHENGYUAN));
}

#[test]
fn male_owner_is_an_eligible_target_but_a_dead_owner_cannot_trigger() {
    let (mut tx, owner, _, _) = isolated_state(DRAGCHENGYUAN);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 3);
    let owner_state = accepted.players.get(owner).unwrap();
    assert_eq!(owner_state.hp, owner_state.max_hp as i32);
    assert!(!has_skill(&accepted, owner, DRAGCHENGYUAN));

    let (mut dead_owner, owner, _, female) = isolated_state(DRAGCHENGYUAN);
    dead_owner.state_mut().players.get_mut(owner).unwrap().alive = false;
    assert!(offer(&mut dead_owner, female).unwrap().is_empty());
    assert!(!dead_owner.state().has_pending_choice());
}

#[test]
fn tyfuli_recovers_and_draws_to_two_only_when_accepted() {
    let (mut tx, owner, _, _) = isolated_state(TYFULI);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = -1;
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );

    let declined = resolve(&tx, owner, false, 4);
    assert_eq!(declined.players.get(owner).unwrap().hp, -1);
    assert_eq!(declined.hand_count(owner), 0);
    assert!(has_skill(&declined, owner, TYFULI));

    let mut retry = Transaction::begin(declined);
    assert_eq!(
        offer(&mut retry, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&retry, owner, true, 5);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert_eq!(accepted.hand_count(owner), 2);
    assert!(!has_skill(&accepted, owner, TYFULI));

    let (mut enough_cards, owner, _, _) = isolated_state(TYFULI);
    enough_cards.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let hand = enough_cards.state().hand_count(owner);
    assert!(hand > 2);
    assert_eq!(
        offer(&mut enough_cards, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&enough_cards, owner, true, 6);
    assert_eq!(accepted.hand_count(owner), hand);
}
