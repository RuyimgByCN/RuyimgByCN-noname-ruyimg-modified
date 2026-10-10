//! Golden coverage for 缮甲 and old 曹纯 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SHANJIA: u32 = 3_876_659_100;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/shanjia.skill");

fn inject_weapon(tx: &mut Transaction, owner: PlayerId, rank: u8) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_200 + u32::from(rank)),
        kind: CardKind::Weapon,
        suit: Suit::Club,
        rank,
        weapon_range: 2,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn inject_equipped_weapon(tx: &mut Transaction, owner: PlayerId) -> CardId {
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_299),
        kind: CardKind::Weapon,
        suit: Suit::Spade,
        rank: 13,
        weapon_range: 2,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut()
        .zones
        .get_mut(equip)
        .unwrap()
        .cards
        .push(card);
    card
}

fn choose(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    state
}

#[test]
fn only_owner_equipment_uses_count_to_seven_and_equipment_discard_offers_far_sha() {
    let package = content_compiler::compile_source_named("shanjia-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 283);
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| { trigger.filter == "owner_source_equip_used_skill_mark_below_seven" }));
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 2_026_072_903, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let far = players[2];
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(SHANJIA),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();

    let foreign_equip = inject_weapon(&mut tx, other, 1);
    run_effects(
        &mut tx,
        vec![EffectRequest::UseCard {
            player: other,
            card: foreign_equip,
            targets: vec![other],
        }],
    )
    .unwrap();
    for rank in 2..=9 {
        let weapon = inject_weapon(&mut tx, owner, rank);
        run_effects(
            &mut tx,
            vec![EffectRequest::UseCard {
                player: owner,
                card: weapon,
                targets: vec![owner],
            }],
        )
        .unwrap();
    }
    let equipped = inject_equipped_weapon(&mut tx, owner);
    let marks = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == SHANJIA)
        .map_or(0, |(_, count)| *count);
    assert_eq!(marks, 7);

    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def == SkillDefId::new(SHANJIA))
        .unwrap()
        .id;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveShanjia {
                player: owner,
                count: marks as u32,
                stage: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let mut selected = vec![equipped.raw()];
    selected.extend(
        engine_core::query::discardable_hand_equip_cards(tx.state(), owner)
            .into_iter()
            .filter(|card| *card != equipped)
            .take(6)
            .map(CardId::raw),
    );
    assert_eq!(selected.len(), 7);
    let state = tx.commit().state;
    assert_eq!(state.resolution.continuation_payload[1], 7);
    assert_eq!(state.resolution.pending_observer, owner.raw());
    let legal = engine_core::query::discardable_hand_equip_cards(&state, owner);
    assert!(
        selected
            .iter()
            .all(|card| legal.contains(&CardId::new(*card))),
        "{selected:?} not within {legal:?}"
    );
    let state = choose(state, owner, selected, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
    assert_eq!(state.resolution.continuation_payload[2], 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(equipped).unwrap().zone, discard);
    let state = choose(state, owner, vec![far.raw()], 2);
    assert_ne!(
        state.cards.get(equipped).unwrap().zone,
        state.zones.find(ZoneKind::Equip, owner).unwrap()
    );
}
