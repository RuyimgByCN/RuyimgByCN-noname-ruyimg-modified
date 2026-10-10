//! Golden for mobile 莲花 (shhlianhua): locked — the owner draws 1 card when
//! targeted by a 【杀】.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const SHHLIANHUA: u32 = 4_132_145_816;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

fn start() -> (Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_601,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(matches!(result, CommandResult::Accepted { .. }));
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

fn add_skill(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(SHHLIANHUA),
    });
    run_effects(tx, effects).unwrap();
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
fn committed_package_publishes_shhlianhua_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|t| {
        t.skill_def_id == SHHLIANHUA
            && t.event == "card_targeted"
            && t.filter == "owner_is_target_event_card_sha"
    }));
}

#[test]
fn being_targeted_by_sha_draws_one_card() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner);
    let sha = inject_hand(tx.state_mut(), other, CardKind::Sha);
    let hand_before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "被【杀】指定为目标 → 摸一张"
    );
}

#[test]
fn being_targeted_by_a_trick_does_not_draw() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_skill(&mut tx, owner);
    let trick = inject_hand(tx.state_mut(), other, CardKind::TrickWuzhong);
    let hand_before = tx.state().hand_count(owner);

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: trick,
            target: owner,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(&mut tx, effects).unwrap();

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "非【杀】目标不触发"
    );
}
