//! Golden coverage for STANDARD「天妒」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TIANDU: u32 = 6;
const SOURCE: &str = include_str!("../../../../content/standard/tiandu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn inject_discard(tx: &mut Transaction) -> CardId {
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(24_101),
        kind: CardKind::Other,
        suit: Suit::Spade,
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
    card
}

#[test]
fn owner_may_gain_the_final_physical_judgment_card() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == TIANDU)
        .unwrap();
    assert_eq!(trigger.event, "judgment_resolved");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(trigger.optional);
    let compiled = content_compiler::compile_source_named("tiandu-test", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_725, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TIANDU),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;

    let result_card = inject_discard(&mut tx);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::EmitJudgmentResolved {
                player: owner,
                result_card,
                suit: Suit::Spade,
                rank: 7,
                cause_card: CardId::NONE,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );

    let state = tx.state().clone();
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, accepted) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert_eq!(state.cards.get(result_card).unwrap().zone, hand);
}
