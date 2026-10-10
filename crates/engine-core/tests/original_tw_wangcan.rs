//! Golden coverage for TW 王粲「应机/商和」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const TWSHANGHE: u32 = 4_227_896_164;
const TWYINGJI: u32 = 2_583_490_638;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twwangcan.skill");

fn shanghe_case(kind: CardKind, expected_hp: i32, key: u64) {
    let package = content_compiler::compile_source_named("tw-wangcan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 385);
    assert_eq!(package.view_as_index.len(), 2);
    assert!(package
        .view_as_index
        .iter()
        .all(|rule| { rule.cost.as_deref() == Some("draw_one_outside_turn_empty_hand") }));

    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_126 + key, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, giver) = (players[0], players[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWSHANGHE),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx.state().players.get(owner).unwrap().skills[0];
    let hand = tx.state().zones.find(ZoneKind::Hand, giver).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_860 + key as u32),
        kind,
        suit: Suit::Diamond,
        rank: 9,
        weapon_range: 0,
        zone: hand,
        owner: giver,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
    owner_state.hp = 0;
    owner_state.dying = true;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveTwshanghe {
                player: owner,
                stage: 0,
                remaining: Vec::new(),
                got_jiu: false,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(state.resolution.pending_observer, giver.raw());
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor: giver,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![card.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, expected_hp);
    assert_eq!(
        state.cards.get(card).unwrap().zone,
        state.zones.find(ZoneKind::Hand, owner).unwrap()
    );
}

#[test]
fn shanghe_recovers_without_jiu_and_does_not_recover_with_jiu() {
    shanghe_case(CardKind::Sha, 1, 1);
    shanghe_case(CardKind::Jiu, 0, 2);
}

#[test]
fn yingji_draws_then_supplies_a_source_free_shan_outside_turn() {
    let package = content_compiler::compile_source_named("tw-wangcan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 2, 2_026_080_129, "identity-standard").unwrap();
    let actor = tx.state().turn.current;
    let responder = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let responder_hand = tx.state().zones.find(ZoneKind::Hand, responder).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, engine_api::PlayerId::NONE)
        .unwrap();
    let old_hand = tx.state().hand_cards(responder);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::MoveCards {
                cards: old_hand,
                to: discard,
                from: Some(responder_hand),
            },
            EffectRequest::AddSkill {
                player: responder,
                skill_def: SkillDefId::new(TWYINGJI),
            },
        ],
    )
    .unwrap();
    let actor_hand = tx.state().zones.find(ZoneKind::Hand, actor).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_870),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: actor_hand,
        owner: actor,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(actor_hand)
        .unwrap()
        .cards
        .push(sha);
    let hp = tx.state().players.get(responder).unwrap().hp;
    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(20),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![responder],
                card_targets: vec![],
            },
        },
    );
    let choice = offered.choice.unwrap();
    assert_eq!(choice.observer, responder);
    let token = choice.options[0].token;
    let (state, resolved) = apply_command(
        state,
        Command {
            actor: responder,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(21),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![token],
                cancelled: false,
            },
        },
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(responder).unwrap().hp, hp);
    assert_eq!(state.hand_count(responder), 1);
}
