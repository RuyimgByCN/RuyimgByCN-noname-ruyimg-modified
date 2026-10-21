//! Golden coverage for KEY「天妒」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NAGISA_TIANDU: u32 = 4_094_577_149;
const SOURCE: &str = include_str!("../../../../content/original/ported/key/nagisa_tiandu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

fn inject_discard(tx: &mut Transaction, suit: Suit, rank: u8) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(24_100 + rank as u32),
        kind: CardKind::Other,
        suit,
        rank,
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
    card
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("nagisa-tiandu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_725, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(NAGISA_TIANDU),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    (tx, owner, other)
}

#[test]
fn gains_only_the_owners_final_physical_judgment_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 691);
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == NAGISA_TIANDU)
        .unwrap();
    assert_eq!(trigger.event, "judgment_resolved");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);

    let (mut tx, owner, other) = setup();
    let result_card = inject_discard(&mut tx, Suit::Spade, 7);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: owner,
            result_card,
            suit: Suit::Spade,
            rank: 7,
            cause_card: CardId::NONE,
        }],
    );
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(tx.state().resolution.pending_observer, owner.raw());

    let state = tx.state().clone();
    let (state, accepted) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(result_card).unwrap().zone, hand);

    let mut tx = Transaction::begin(state);
    let other_result = inject_discard(&mut tx, Suit::Heart, 8);
    run_effects(
        &mut tx,
        vec![EffectRequest::EmitJudgmentResolved {
            player: other,
            result_card: other_result,
            suit: Suit::Heart,
            rank: 8,
            cause_card: CardId::NONE,
        }],
    )
    .unwrap();
    assert!(!tx.state().has_pending_choice());
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(other_result).unwrap().zone, discard);
}
