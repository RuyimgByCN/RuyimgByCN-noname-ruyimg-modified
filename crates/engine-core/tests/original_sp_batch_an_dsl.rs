//! Golden for sp batch-AN DSL rewrite:
//! 肃州 (pxsizhou) on-gain optional: gain one hand card of another player via the
//! new two-stage `gain_target_hand_card` primitive (ABI 647, Cont 267):
//! target selection (cancellable) → owner picks one of the target's hand cards
//! → the card moves to the owner's hand.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const PXSIZHOU: u32 = 3_079_253_303;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

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
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
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

fn inject_hand(state: &mut GameState, player: PlayerId, tag: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(92_100 + tag),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Club,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_an_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 745);
    let entry = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == PXSIZHOU)
        .expect("pxsizhou trigger");
    assert_eq!(entry.event.as_str(), "skill_added");
    assert_eq!(entry.filter.as_str(), "owner_skill_added_self_3079253303");
    assert!(entry.optional);
}

#[test]
fn pxsizhou_gains_one_target_hand_card() {
    let (mut state, owner) = started(2_026_081_550);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let target_card = inject_hand(&mut state, other, 1);
    inject_hand(&mut state, other, 2);
    let owner_before = state.hand_count(owner);
    let other_before = state.hand_count(other);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSIZHOU),
        }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    // Confirm → target selection opens.
    let (state, result) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GainTargetHandCard
    );
    // Pick the target, then the owner picks one of its hand cards.
    let (state, result) = resolve(state, owner, vec![other.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GainTargetHandCard
    );
    let (state, result) = resolve(state, owner, vec![target_card.raw()], false, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_before + 1);
    assert_eq!(state.hand_count(other), other_before - 1);
    assert!(state.hand_cards(owner).contains(&target_card));
}

#[test]
fn pxsizhou_cancel_keeps_hands_intact() {
    let (mut state, owner) = started(2_026_081_551);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    inject_hand(&mut state, other, 3);
    let owner_before = state.hand_count(owner);
    let other_before = state.hand_count(other);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSIZHOU),
        }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    let tx = tx.commit();
    // Confirm, then cancel the target selection entirely.
    let (state, _) = resolve(tx.state, owner, vec![1], false, 4);
    let (state, result) = resolve(state, owner, vec![], true, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), owner_before);
    assert_eq!(state.hand_count(other), other_before);
}

#[test]
fn pxsizhou_decline_keeps_hands_intact() {
    let (mut state, owner) = started(2_026_081_552);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    inject_hand(&mut state, other, 4);
    let owner_before = state.hand_count(owner);

    let mut tx = Transaction::begin(state);
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PXSIZHOU),
        }],
    );
    assert!(matches!(effects, Err(EffectError::ChoiceRequested)));
    let tx = tx.commit();
    // Decline at the optional-trigger confirm.
    let (state, result) = resolve(tx.state, owner, vec![], true, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), owner_before);
}
