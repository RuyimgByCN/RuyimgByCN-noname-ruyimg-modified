//! DSL `MoveCards` reaches the real engine with zone queries.

use content_compiler::compile_source;
use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PACK: &str = r#"
skill reclaim(100) {
  trigger card_used owner_is_source {
    effect move_card(event_card, zone(0, 2), card_zone(event_card));
  }
}
"#;

#[test]
fn trigger_moves_event_card_to_discard() {
    let package = compile_source(PACK).expect("compile package");
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .expect("load package");
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 17, "identity-standard").expect("bootstrap");
    let owner = PlayerId::new(1);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(100),
        }],
    )
    .expect("grant skill");

    let card: CardId = tx.state().hand_cards(owner)[0];
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    let mut fires = TriggerFireRecord::new();
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut fires,
    )
    .expect("dispatch");
    run_effects(&mut tx, effects).expect("apply move");

    assert_eq!(tx.state().cards.get(card).unwrap().zone, discard);
    assert!(tx.state().zones.get(discard).unwrap().cards.contains(&card));
}
