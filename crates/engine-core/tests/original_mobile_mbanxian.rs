//! Golden coverage for mobile 曹性「暗弦」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const MBANXIAN: u32 = 2_708_680_147;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup() -> (Transaction, PlayerId, Vec<PlayerId>) {
    let mut engine = Engine::new();
    engine.load_package(&package()).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_073_103,
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
        skill_def: SkillDefId::new(MBANXIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::SHA_PLAYED_IN_PLAY_PHASE, 99);
    tx.state_mut()
        .turn
        .set_flag(turn_flag::hand_discard_event_by_player(owner), 0);
    (tx, owner, others)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, Vec<DomainEvent>) {
    let (state, output) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(output.result, CommandResult::Accepted { .. }),
        "{:?}",
        output.result
    );
    (state, output.events)
}

#[test]
fn package_publishes_abi333_and_first_hand_discard_trigger() {
    let package = package();
    assert_eq!(package.manifest.rule_vm_abi_version, 734);
    assert_eq!(package.skill_index.len(), 440);
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == MBANXIAN)
        .unwrap();
    assert_eq!(trigger.event, "cards_moved");
    assert_eq!(
        trigger.filter,
        "owner_first_hand_discard_has_sha_in_discard"
    );
    assert!(trigger.optional);
}

#[test]
fn first_discarded_hand_sha_is_gained_then_used_ignoring_limits() {
    let (mut tx, owner, others) = setup();
    let cards = tx.state().hand_cards(owner);
    let sha = cards[0];
    tx.state_mut().cards.get_mut(sha).unwrap().kind = CardKind::Sha;
    assert!(engine_core::query::effective_card_kind(
        tx.state(),
        owner,
        tx.state().cards.get(sha).unwrap()
    )
    .is_sha());
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::hand_discard_event_by_player(owner)),
        0
    );
    let offered = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![sha],
        }],
    );
    assert!(engine_core::query::effective_card_kind(
        tx.state(),
        owner,
        tx.state().cards.get(sha).unwrap()
    )
    .is_sha());
    assert_eq!(offered.unwrap_err(), EffectError::ChoiceRequested);

    let (activated, _) = resolve(tx.state().clone(), owner, vec![1], 1);
    assert_eq!(activated.resolution.pending_observer, owner.raw());
    let (gained, _) = resolve(activated, owner, vec![sha.raw()], 2);
    assert!(gained.hand_cards(owner).contains(&sha));
    let distant_target = others[1];
    let (used, events) = resolve(gained, owner, vec![distant_target.raw()], 3);
    assert!(!used.hand_cards(owner).contains(&sha));
    assert!(events.iter().any(
        |event| matches!(event, DomainEvent::CardUsed { player, card, targets }
            if *player == owner && *card == sha && targets.contains(&distant_target))
    ));
}

#[test]
fn a_first_hand_discard_without_sha_blocks_later_sha() {
    let (mut tx, owner, _) = setup();
    let cards = tx.state().hand_cards(owner);
    let first = cards[0];
    let later_sha = cards[1];
    tx.state_mut().cards.get_mut(first).unwrap().kind = CardKind::TrickWuzhong;
    tx.state_mut().cards.get_mut(later_sha).unwrap().kind = CardKind::Sha;

    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![first],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards: vec![later_sha],
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::hand_discard_event_by_player(owner)),
        2
    );
}
