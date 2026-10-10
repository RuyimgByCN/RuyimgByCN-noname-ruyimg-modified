//! Golden for 一将成名廖化「伏枥」rewritten from the legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const FULI: u32 = 3_748_306_630;
const XINFULI: u32 = 3_062_292_376;
const WEI: u32 = 1;
const SHU: u32 = 8;
const WU: u32 = 15;
const QUN: u32 = 23;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

fn isolated(skill_def: u32, character_defs: [u32; 4]) -> (Transaction, PlayerId, Vec<PlayerId>) {
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
    for (player, character_def) in players.iter().copied().zip(character_defs) {
        tx.state_mut()
            .players
            .get_mut(player)
            .unwrap()
            .character_def = character_def;
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
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    (tx, owner, players)
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

fn has_skill(state: &engine_core::state::GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .into_iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn committed_package_exposes_optional_limited_dying_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 6);
    for skill_def in [FULI, XINFULI] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def && !skill.locked));
        let trigger = package
            .trigger_index
            .iter()
            .find(|trigger| trigger.skill_def_id == skill_def)
            .unwrap();
        assert_eq!(
            (
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional
            ),
            ("dying", "owner_is_target", true)
        );
    }
}

#[test]
fn duplicate_groups_count_once_and_acceptance_recovers_turns_over_and_consumes() {
    let (mut tx, owner, _) = isolated(FULI, [WEI, WEI, SHU, WU]);
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::AliveGroupCount,
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(3)
    );
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 1);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 3);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_skill(&accepted, owner, FULI));
}

#[test]
fn declining_preserves_state_and_dead_players_groups_do_not_count() {
    let (mut tx, owner, players) = isolated(FULI, [WEI, SHU, WU, QUN]);
    tx.state_mut().players.get_mut(players[3]).unwrap().alive = false;
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::AliveGroupCount,
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(3)
    );
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let declined = resolve(&tx, owner, false, 2);
    assert_eq!(declined.players.get(owner).unwrap().hp, 0);
    assert!(!declined.players.get(owner).unwrap().turned_over);
    assert!(has_skill(&declined, owner, FULI));
}

#[test]
fn xinfuli_recovers_and_draws_to_groups_then_turns_over_above_two() {
    let (mut tx, owner, _) = isolated(XINFULI, [WEI, WEI, SHU, WU]);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 3);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 3);
    assert_eq!(accepted.hand_count(owner), 3);
    assert!(accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_skill(&accepted, owner, XINFULI));
}

#[test]
fn xinfuli_does_not_reduce_hand_or_turn_over_at_two_groups() {
    let (mut tx, owner, _) = isolated(XINFULI, [WEI, WEI, SHU, SHU]);
    let hand = tx.state().hand_count(owner);
    assert!(hand > 2);
    assert_eq!(
        offer(&mut tx, owner).unwrap_err(),
        EffectError::ChoiceRequested
    );
    let accepted = resolve(&tx, owner, true, 4);
    assert_eq!(accepted.players.get(owner).unwrap().hp, 2);
    assert_eq!(accepted.hand_count(owner), hand);
    assert!(!accepted.players.get(owner).unwrap().turned_over);
    assert!(!has_skill(&accepted, owner, XINFULI));
}
