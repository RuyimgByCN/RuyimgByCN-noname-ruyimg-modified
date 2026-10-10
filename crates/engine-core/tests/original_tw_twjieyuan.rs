//! Golden coverage for TW「竭缘」(twjieyuan).
//! Outgoing damage_before: optional confirm, then damage +1 and a forced hand discard.
//! Incoming damage_before from another source: optional confirm, then damage -1 and discard.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWJIEYUAN: u32 = 4_287_464_553;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twjieyuan.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    other: PlayerId,
    hand_card: CardId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("twjieyuan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_93, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
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
        skill_def: SkillDefId::new(TWJIEYUAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let hand = inject_hand(tx.state_mut(), owner, 170_301);
    Setup {
        state: tx.commit().state,
        owner,
        other,
        hand_card: hand,
    }
}

fn inject_hand(state: &mut GameState, player: PlayerId, def: u32) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, player).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: player,
        face_up: true,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
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
fn compiled_source_and_npack_publish_both_damage_before_windows() {
    let compiled = content_compiler::compile_source_named("twjieyuan", SOURCE).unwrap();
    let mut filters: Vec<_> = compiled
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == TWJIEYUAN)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str(), trigger.optional))
        .collect();
    filters.sort();
    assert_eq!(
        filters,
        vec![
            ("damage_before", "owner_is_target_other_source_alive", true),
            ("damage_before", "owner_source_other_damage_before", true),
        ]
    );

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == TWJIEYUAN && skill.name == "twjieyuan"));
    let pack_filters: Vec<_> = committed
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == TWJIEYUAN)
        .map(|trigger| trigger.filter.as_str())
        .collect();
    assert!(pack_filters.contains(&"owner_source_other_damage_before"));
    assert!(pack_filters.contains(&"owner_is_target_other_source_alive"));
}

#[test]
fn outgoing_damage_increases_then_discards_one_hand_card() {
    let setup = setup();
    let other_hp = setup.state.players.get(setup.other).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.owner,
                target: setup.other,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], 1);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    assert!(state.has_pending_choice());
    let before = state.hand_count(setup.owner);
    let (state, discarded) = resolve(state, setup.owner, vec![setup.hand_card.raw()], 2);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }), "{:?}", discarded.result);
    assert_eq!(state.hand_count(setup.owner), before - 1);
    assert_eq!(
        state.players.get(setup.other).unwrap().hp,
        other_hp - 2,
        "outgoing branch should deal 2; events={:?}",
        discarded.events
    );
}

#[test]
fn incoming_damage_decreases_to_zero_when_base_is_one() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.other,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let (state, confirmed) = resolve(tx.commit().state, setup.owner, vec![1], 3);
    assert!(matches!(confirmed.result, CommandResult::Accepted { .. }));
    let owner_hp = state.players.get(setup.owner).unwrap().hp;
    let (state, discarded) = resolve(state, setup.owner, vec![setup.hand_card.raw()], 4);
    assert!(matches!(discarded.result, CommandResult::Accepted { .. }), "{:?}", discarded.result);
    assert_eq!(state.players.get(setup.owner).unwrap().hp, owner_hp);
}
