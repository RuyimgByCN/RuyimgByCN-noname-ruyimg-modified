//! Golden coverage for clan 中流 (clanzhongliu): locked clan skill that refreshes its own
//! use counts when its owner uses a card and no other member of 太原王氏 lost hand cards.
//!
//! This exercises the three Host primitives added to unlock the clan skill family:
//!   - character `clans` attribute (registry lookup, like `group`),
//!   - `refresh_skill` effect (single-skill per-turn counter reset),
//!   - `owner_is_event_player_no_<clan>_lost_hand_this_turn` trigger filter.
//! The card-loss check is a per-turn approximation of upstream's per-useEvent correlation
//! (the engine tracks per-turn card loss, not per-event-parent loss).

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

const CLANZHONGLIU: u32 = 3_016_047_578;
const CLAN_WANGLING: u32 = 2_803_794_166;
/// Another 太原王氏 character used for the opposing seat so the clan-loss filter has a
/// same-clan player to observe.
const CLAN_WANGCHANG: u32 = 3_580_915_327;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/clan/clan.npack.json");

fn start() -> (engine_core::Engine, PlayerId, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_809,
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

fn add_clanzhongliu(tx: &mut Transaction, owner: PlayerId) -> engine_api::SkillId {
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(CLANZHONGLIU),
    });
    run_effects(tx, effects).unwrap();
    tx.state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == CLANZHONGLIU)
        .unwrap()
        .id
}

/// Assign two 太原王氏 clan characters (from the loaded clan package) to the seats, since the
/// default identity bootstrap draws characters from the standard pool which carries no clans.
fn assign_clan_seats(tx: &mut Transaction, owner: PlayerId, other: PlayerId) {
    tx.state_mut().players.get_mut(owner).unwrap().character_def = CLAN_WANGLING;
    tx.state_mut().players.get_mut(other).unwrap().character_def = CLAN_WANGCHANG;
}

fn inject_hand(state: &mut engine_core::state::GameState, player: PlayerId) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_001),
        kind: CardKind::Sha,
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

fn dispatch_card_used(tx: &mut Transaction, user: PlayerId, card: CardId) {
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn committed_package_publishes_clanzhongliu_trigger_and_clan_roster() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    // The locked trigger is published with the clan-loss filter on card_used.
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == CLANZHONGLIU
            && trigger.event == "card_used"
            && trigger
                .filter
                .ends_with("_lost_hand_this_turn")
            && trigger.filter.contains("太原王氏")
    }));
    // clan_wangling carries the 太原王氏 clan membership.
    let wangling = pkg
        .character_index
        .iter()
        .find(|character| character.id == CLAN_WANGLING)
        .unwrap();
    assert_eq!(wangling.clans, ["太原王氏"]);
}

#[test]
fn refresh_skill_clears_use_counts_when_no_clan_member_lost_hand() {
    let (engine, owner, other) = start();
    // Assign two 太原王氏 clan characters to the seats; the owner uses a card while no one has
    // lost hand cards this turn → clanzhongliu fires refresh_skill.
    let mut tx = Transaction::begin(engine.state().clone());
    assign_clan_seats(&mut tx, owner, other);
    let skill = add_clanzhongliu(&mut tx, owner);
    // Simulate prior activations so use_count is non-zero before the refresh.
    {
        let instance = tx.state_mut().skills.get_mut(skill).unwrap();
        instance.use_count = 2;
        instance.turn_fire_count = 1;
    }
    let card = inject_hand(tx.state_mut(), owner);
    assert_eq!(tx.state().skills.get(skill).unwrap().use_count, 2);
    dispatch_card_used(&mut tx, owner, card);
    // refresh_skill zeroed the per-turn counters.
    let refreshed = tx.state().skills.get(skill).unwrap();
    assert_eq!(refreshed.use_count, 0);
    assert_eq!(refreshed.turn_fire_count, 0);
}

#[test]
fn filter_blocks_when_a_clan_member_lost_hand_this_turn() {
    let (engine, owner, other) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    assign_clan_seats(&mut tx, owner, other);
    let skill = add_clanzhongliu(&mut tx, owner);
    // Record that the other (clan-member) player lost a hand card this turn.
    let other_card = inject_hand(tx.state_mut(), other);
    tx.state_mut().turn.record_card_loss(other, other_card);
    assert!(tx.state().turn.card_loss_count(other) > 0);
    // Bump the owner's counts so we can detect that refresh did NOT fire.
    {
        let instance = tx.state_mut().skills.get_mut(skill).unwrap();
        instance.use_count = 2;
    }
    let card = inject_hand(tx.state_mut(), owner);
    dispatch_card_used(&mut tx, owner, card);
    // The filter rejected the trigger → counts unchanged (no refresh_skill).
    assert_eq!(tx.state().skills.get(skill).unwrap().use_count, 2);
}
