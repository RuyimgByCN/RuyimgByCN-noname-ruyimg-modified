//! Golden coverage for sp 弘德 (hongde): optional skill that, when its owner gains
//! (or loses) at least two cards in one move, opens a choose-target dialog and
//! makes the chosen *other* player draw 1 card.
//!
//! Exercises the `choose_target_draw` dialog primitive end-to-end:
//!   - the DSL `choose_target_draw(self, 1, other)` compiles into `HOST_EFFECT(59)`,
//!   - an optional trigger first opens a Confirm choice,
//!   - confirming runs the effect, which opens a SelectTargets choice whose options
//!     are filtered by the `other` scope (self excluded),
//!   - resolving with the other player makes that player draw the card,
//!   - declining the trigger (selecting option 0) draws for nobody.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const HONGDE: u32 = 3_138_095_197;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_814,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    (engine, owner, other)
}

fn strip_and_add_hongde(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HONGDE),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == HONGDE)
        .unwrap()
        .id
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn committed_package_publishes_hongde_draw_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HONGDE
            && trigger.event == "cards_drawn"
            && trigger.filter == "owner_gained_or_lost_at_least_two"
            && trigger.optional
    }));
}

#[test]
fn confirming_opens_scope_filtered_choose_target_draw_and_target_draws() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let skill = strip_and_add_hongde(&mut tx, owner);
    let c1 = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let c2 = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let owner_hand_before = tx.state().hand_count(owner);
    let other_hand_before = tx.state().hand_count(other);

    // Owner gains two cards in one event → the optional trigger opens a Confirm choice.
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsDrawn {
            player: owner,
            cards: vec![c1, c2],
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(_) => panic!("optional trigger must open a confirm choice"),
        Err(engine_core::effect::EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch error: {e:?}"),
    }
    assert!(tx.state().has_pending_choice());
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );

    // Confirm the optional trigger → the skill effect runs and opens the
    // choose_target_draw dialog (scope = other, so only `other` is offered).
    let confirm_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, confirmed) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        engine_core::state::ContinuationKind::ChooseTargetDraw,
        "confirming runs the effect, which opens the target-draw dialog"
    );
    let draw_choice = confirmed
        .choice
        .as_ref()
        .expect("choose_target_draw must publish a choice");
    let tokens: Vec<u32> = draw_choice
        .options
        .iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(
        tokens,
        vec![other.raw()],
        "scope=other excludes the chooser"
    );
    assert_eq!(
        state.resolution.continuation_payload,
        vec![owner.raw(), 1, 0, skill.raw()],
        "payload encodes (player, count=1, scope=other, skill)"
    );

    // Pick the other player → they draw 1 card; the owner's hand is untouched.
    let draw_id = engine_api::ChoiceId::new(state.resolution.pending_choice_id);
    let (state, picked) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: draw_id,
                selections: vec![other.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(picked.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.hand_count(other),
        other_hand_before + 1,
        "chosen target draws the card"
    );
    assert_eq!(
        state.hand_count(owner),
        owner_hand_before,
        "owner's hand is unchanged"
    );
}

#[test]
fn declining_the_optional_trigger_draws_for_nobody() {
    let (engine, owner, _other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let _skill = strip_and_add_hongde(&mut tx, owner);
    let c1 = inject_hand(tx.state_mut(), owner, CardKind::Sha);
    let c2 = inject_hand(tx.state_mut(), owner, CardKind::Tao);
    let owner_hand_before = tx.state().hand_count(owner);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsDrawn {
            player: owner,
            cards: vec![c1, c2],
        }],
        &mut TriggerFireRecord::new(),
    );
    assert!(matches!(
        fired,
        Err(engine_core::effect::EffectError::ChoiceRequested)
    ));

    // Select option 0 → the optional trigger is declined, no dialog opens.
    let confirm_id = engine_api::ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, declined) = apply_command(
        tx.clone().commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: confirm_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert!(
        !state.has_pending_choice(),
        "declining the optional trigger must not open a draw dialog"
    );
    assert_eq!(state.hand_count(owner), owner_hand_before);
}
