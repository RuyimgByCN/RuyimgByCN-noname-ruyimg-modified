//! Golden coverage for standard 刚烈 rewritten from the legacy JavaScript semantics.

use content_compiler::compile_source_named;
use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::{damage_nature, EffectRequest};

const GANGLIE: u32 = 7;
const SOURCE: &str = include_str!("../../../content/standard/ganglie.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = compile_source_named("standard-ganglie-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 20_260_725_7, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let source = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(GANGLIE),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 20;
        participant.max_hp = 20;
    }
    (tx.commit().state, owner, source)
}

fn put_judgment(state: &mut GameState, suit: Suit, def: u32) -> CardId {
    let zone = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn damage(state: GameState, source: PlayerId, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source,
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: damage_nature::NORMAL,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_exact_optional_abi171_ganglie() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 261);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == GANGLIE
            && trigger.event == "damage"
            && trigger.filter == "owner_is_target"
            && trigger.optional
    }));
    let compiled = compile_source_named("standard-ganglie", SOURCE).unwrap();
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));
}

#[test]
fn nonheart_source_discards_exactly_two_hand_cards_or_takes_damage() {
    let (mut state, owner, source) = setup();
    put_judgment(&mut state, Suit::Spade, 171_001);
    let source_hp = state.players.get(source).unwrap().hp;
    let hand = state.hand_cards(source);
    let hand_count = state.hand_count(source);
    assert!(hand.len() >= 2);
    let state = damage(state, source, owner);
    let (state, accepted) = resolve(state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::GanglieSourceDiscardOrDamage
    );
    let (state, discarded) = resolve(state, source, vec![hand[0].raw(), hand[1].raw()], false, 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(source), hand_count - 2);
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);

    let (mut state, owner, source) = setup();
    put_judgment(&mut state, Suit::Club, 171_002);
    let source_hp = state.players.get(source).unwrap().hp;
    let state = damage(state, source, owner);
    let (state, _) = resolve(state, owner, vec![1], false, 3);
    let (state, declined) = resolve(state, source, Vec::new(), true, 4);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}

#[test]
fn heart_missing_source_or_missing_two_card_cost_opens_no_source_choice() {
    let (mut state, owner, source) = setup();
    put_judgment(&mut state, Suit::Heart, 171_010);
    let source_hp = state.players.get(source).unwrap().hp;
    let state = damage(state, source, owner);
    let (state, heart) = resolve(state, owner, vec![1], false, 5);
    assert!(matches!(heart.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, source_hp);

    let (mut state, owner, _) = setup();
    let judgment = put_judgment(&mut state, Suit::Spade, 171_011);
    let state = damage(state, PlayerId::NONE, owner);
    let (state, missing_source) = resolve(state, owner, vec![1], false, 6);
    assert!(matches!(
        missing_source.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(judgment).unwrap().zone, discard);

    let (mut state, owner, source) = setup();
    let hand_zone = state.zones.find(ZoneKind::Hand, source).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    for card in state.hand_cards(source).into_iter().skip(1) {
        state
            .zones
            .get_mut(hand_zone)
            .unwrap()
            .cards
            .retain(|id| *id != card);
        state.zones.get_mut(discard).unwrap().cards.push(card);
        let card_state = state.cards.get_mut(card).unwrap();
        card_state.zone = discard;
        card_state.owner = PlayerId::NONE;
    }
    put_judgment(&mut state, Suit::Diamond, 171_012);
    let source_hp = state.players.get(source).unwrap().hp;
    let state = damage(state, source, owner);
    let (state, insufficient) = resolve(state, owner, vec![1], false, 7);
    assert!(matches!(
        insufficient.result,
        CommandResult::Accepted { .. }
    ));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(source).unwrap().hp, source_hp - 1);
}
