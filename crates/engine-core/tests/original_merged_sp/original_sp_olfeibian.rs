//! Golden coverage for SP OL 祢衡「飞辩」(olfeibian).
//! After the owner uses a card they must discard one hand card.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLFEIBIAN: u32 = 3_525_562_624;
const SOURCE: &str = include_str!("../../../../content/original/ported/sp/olfeibian.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, CardId) {
    let compiled = content_compiler::compile_source_named("olfeibian-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_43, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLFEIBIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_803),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    (tx.commit().state, owner, card)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
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
    )
}

#[test]
fn compiled_source_and_npack_publish_the_use_then_discard() {
    let compiled = content_compiler::compile_source_named("olfeibian", SOURCE).unwrap();
    let trigger = compiled
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFEIBIAN)
        .expect("olfeibian trigger");
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_is_source");
    assert!(!trigger.optional);

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLFEIBIAN)
        .expect("npack olfeibian trigger");
    assert_eq!(pack.event, "card_used");
    assert_eq!(pack.filter, "owner_is_source");
}

#[test]
fn using_a_card_forces_a_hand_discard() {
    let (state, owner, card) = setup();
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects),
        Err(EffectError::ChoiceRequested)
    );
    let (state, discarded) = resolve(tx.commit().state, owner, vec![card.raw()], 1);
    assert!(
        matches!(discarded.result, CommandResult::Accepted { .. }),
        "{:?}",
        discarded.result
    );
    assert_eq!(state.hand_count(owner), before - 1);
}
