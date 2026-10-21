//! Golden for sp batch-W DSL rewrite:
//! 灵杉 (huamu) card-resolved → store used card onto the owner's skill pile (ABI 607).

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const HUAMU: u32 = 3_881_736_086;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut state = engine.state().clone();
    // v1.11.6 增量 sp roster +1（王皑）平移种子：开局 optional 触发可能挂起 choice，
    // 以 cancelled=true 排空（original_sp_qimen_zniaoxiang 范例）。
    let mut drain_key = 1u64;
    while state.has_pending_choice() {
        let choice = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        let (next, output) = apply_command(
            state,
            Command {
                actor: PlayerId::NONE,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(seed ^ drain_key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice,
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(output.result, CommandResult::Accepted { .. }),
            "drain rejected: {:?}",
            output.result
        );
        state = next;
        drain_key += 1;
    }
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_huamu_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HUAMU
            && trigger.event == "card_resolved"
            && trigger.filter == "owner_source"
            && trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| { modifier.skill_def_id == HUAMU }));
}

#[test]
fn huamu_stores_discarded_used_card_on_skill_pile() {
    let (state, owner) = started(2_026_081_701);
    let state = with_skill(state, owner, HUAMU);
    // Manually place a card in the discard pile and fire a CardResolved event whose
    // card is that discard-pile card; the skill should move it to the owner's pile.
    let mut tx = Transaction::begin(state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_200),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: discard,
        owner: PlayerId::NONE,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(discard)
        .unwrap()
        .cards
        .push(card);
    let card_id = card;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: owner,
            card: card_id,
            kind: CardKind::Sha.tag(),
            targets: Vec::new(),
            damaged_targets: Vec::new(),
            source_cards: Vec::new(),
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // The used card should now sit in the owner's skill pile (expansion zone).
    let pile_cards = state.skill_pile_cards(owner, HUAMU);
    assert!(
        pile_cards.contains(&card_id),
        "used card must be stored on the skill pile"
    );
    assert_eq!(
        state.cards.get(card_id).unwrap().zone,
        state.zones.find(ZoneKind::Expansion, owner).unwrap()
    );
}

#[test]
fn huamu_skips_card_not_in_discard() {
    let (state, owner) = started(2_026_081_702);
    let state = with_skill(state, owner, HUAMU);
    let mut tx = Transaction::begin(state);
    // Card in the hand, not the discard pile: the store effect must no-op.
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_201),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let card_id = card;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardResolved {
            source: owner,
            card: card_id,
            kind: CardKind::Sha.tag(),
            targets: Vec::new(),
            damaged_targets: Vec::new(),
            source_cards: Vec::new(),
            all_sources_were_hand: false,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let pile_cards = state.skill_pile_cards(owner, HUAMU);
    assert!(
        !pile_cards.contains(&card_id),
        "card outside the discard pile must not be stored"
    );
}
