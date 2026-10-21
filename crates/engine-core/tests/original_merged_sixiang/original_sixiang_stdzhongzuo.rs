//! Golden coverage for 标贾逵「忠佐」turn-scoped damage history.

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZHONGZUO: u32 = 3_465_971_059;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdzhongzuo.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("stdzhongzuo-test", SOURCE).unwrap();
    let mut state = engine_core::state::GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_717, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, ending_player, third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDZHONGZUO),
        },
        EffectRequest::SetTurnState {
            current: ending_player,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, ending_player, third)
}

fn end_turn(tx: &mut Transaction, next: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: next,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
}

#[test]
fn package_exposes_locked_turn_damage_participant_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == STDZHONGZUO
            && trigger.event == "turn_ended"
            && trigger.filter == "owner_alive_dealt_or_received_damage_this_turn"
            && !trigger.optional
    }));
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDZHONGZUO && skill.locked));
}

#[test]
fn noncurrent_owner_dealing_damage_draws_after_the_ending_player_in_seat_order() {
    let (mut tx, owner, ending_player, third) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let ending_hand = tx.state().hand_count(ending_player);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: third,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(tx.state().turn.player_dealt_damage(owner));

    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let cards = &tx.state().zones.get(draw).unwrap().cards;
    let first = cards[cards.len() - 1];
    let second = cards[cards.len() - 2];
    end_turn(&mut tx, third);

    assert_eq!(tx.state().hand_count(ending_player), ending_hand + 1);
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    assert!(tx.state().hand_cards(ending_player).contains(&first));
    assert!(tx.state().hand_cards(owner).contains(&second));
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGE_SOURCES), 0);
    assert_eq!(tx.state().turn.flag(turn_flag::DAMAGED_PLAYERS), 0);
}

#[test]
fn receiving_damage_triggers_but_hp_loss_and_a_clean_following_turn_do_not() {
    let (mut tx, owner, ending_player, third) = setup();
    let owner_hand = tx.state().hand_count(owner);
    let ending_hand = tx.state().hand_count(ending_player);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: ending_player,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert!(tx.state().turn.player_received_damage(owner));
    end_turn(&mut tx, third);
    assert_eq!(tx.state().hand_count(owner), owner_hand + 1);
    assert_eq!(tx.state().hand_count(ending_player), ending_hand + 1);

    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: third,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: owner,
                amount: 1,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .unwrap();
    let clean_owner_hand = tx.state().hand_count(owner);
    let clean_ending_hand = tx.state().hand_count(third);
    end_turn(&mut tx, ending_player);
    assert_eq!(tx.state().hand_count(owner), clean_owner_hand);
    assert_eq!(tx.state().hand_count(third), clean_ending_hand);
}
