//! Golden coverage for sp 狭赏 (olxiasheng): locked skill whose useCard branch
//! draws 1 when the owner uses a black card while holding more red than black
//! hand cards.
//!
//! Exercises `hand_color_count`, the suit-based black/red check via
//! `card_attribute`, and `draw_cards`. The useCard2 add-target branch is not
//! ported (see the .skill comment).

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLXIASHENG: u32 = 2_354_381_667;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_819,
            ruleset: "identity-standard".into(),
        },
    });
    engine.load_package(&pkg).unwrap();
    let owner = engine.state().turn.current;
    (engine, owner)
}

fn add_olxiasheng(tx: &mut Transaction, owner: PlayerId) {
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
        skill_def: SkillDefId::new(OLXIASHENG),
    });
    run_effects(tx, effects).unwrap();
}

fn inject_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_100 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn replace_hand(
    state: &mut engine_core::state::GameState,
    player: PlayerId,
    cards: &[(CardKind, Suit)],
) -> Vec<CardId> {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let old: Vec<_> = state.hand_cards(player);
    let discard = state.zones.find(ZoneKind::DiscardPile, PlayerId::NONE).unwrap();
    state.zones.get_mut(discard).unwrap().cards.extend(old.clone());
    for card in &old {
        state.cards.get_mut(*card).unwrap().zone = discard;
    }
    state.zones.get_mut(hand).unwrap().cards.clear();
    cards.iter().map(|(kind, suit)| inject_hand(state, player, *kind, *suit)).collect()
}

fn used_card(tx: &mut Transaction, owner: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap_or_else(|e| panic!("dispatch error: {e:?}"));
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_olxiasheng_card_used_trigger() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == OLXIASHENG
            && trigger.event == "card_used"
            && trigger.filter == "owner_source"
    }));
}

#[test]
fn red_majority_black_card_draws_one() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_olxiasheng(&mut tx, owner);
    // Hand: 2 red (桃 heart, 酒 diamond), 1 black (杀 spade).
    let black_sha = replace_hand(
        tx.state_mut(),
        owner,
        &[
            (CardKind::Tao, Suit::Heart),
            (CardKind::Jiu, Suit::Diamond),
            (CardKind::Sha, Suit::Spade),
        ],
    )[2];
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, black_sha);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before + 1,
        "black card with red hand majority draws 1"
    );
}

#[test]
fn black_majority_black_card_draws_nothing() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_olxiasheng(&mut tx, owner);
    // Hand: 2 black (杀 spade, 闪 club), 1 red (桃 heart).
    let black_sha = replace_hand(
        tx.state_mut(),
        owner,
        &[
            (CardKind::Sha, Suit::Spade),
            (CardKind::Shan, Suit::Club),
            (CardKind::Tao, Suit::Heart),
        ],
    )[0];
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, black_sha);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "black-card use with black hand majority draws nothing"
    );
}

#[test]
fn red_majority_red_card_draws_nothing() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    add_olxiasheng(&mut tx, owner);
    // Hand: 2 red, 1 black — use the red card.
    let red_tao = replace_hand(
        tx.state_mut(),
        owner,
        &[
            (CardKind::Tao, Suit::Heart),
            (CardKind::Jiu, Suit::Diamond),
            (CardKind::Sha, Suit::Spade),
        ],
    )[0];
    let hand_before = tx.state().hand_count(owner);

    used_card(&mut tx, owner, red_tao);

    assert_eq!(
        tx.state().hand_count(owner),
        hand_before,
        "red-card use draws nothing"
    );
}
