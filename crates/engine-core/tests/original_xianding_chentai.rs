//! Golden for XIANDING 陈泰【矜贤】（ABI 597）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const DCCTJIUXIAN: u32 = 3_598_325_442;
const SOURCE: &str = include_str!("../../../content/original/ported/xianding/dcctjiuxian.skill");

fn setup() -> (GameState, PlayerId, PlayerId, PlayerId, engine_api::SkillId) {
    let package = content_compiler::compile_source_named("dcctjiuxian-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, damaged, other) = (players[0], players[1], players[2]);
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
            skill_def: SkillDefId::new(DCCTJIUXIAN),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let initial_hand = tx.state().hand_cards(owner);
    if !initial_hand.is_empty() {
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: owner,
                cards: initial_hand,
            }],
        )
        .unwrap();
    }
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == DCCTJIUXIAN)
        .unwrap()
        .id;
    (tx.commit().state, owner, damaged, other, skill)
}

fn alloc(tx: &mut Transaction, owner: PlayerId, zone: ZoneKind, rank: u8) -> CardId {
    let zone = tx.state().zones.find(zone, owner).unwrap_or_else(|| {
        tx.state()
            .zones
            .find(zone, PlayerId::NONE)
            .expect("shared zone")
    });
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_700 + u32::from(rank)),
        kind: CardKind::Other,
        suit: Suit::Club,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, cancelled: bool) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(90_000 + u64::from(actor.raw())),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn active_recasts_ceil_half_then_opens_an_exact_tracked_duel() {
    let (state, owner, damaged, _other, skill) = setup();
    let mut tx = Transaction::begin(state);
    let hand = [
        alloc(&mut tx, owner, ZoneKind::Hand, 1),
        alloc(&mut tx, owner, ZoneKind::Hand, 2),
        alloc(&mut tx, owner, ZoneKind::Hand, 3),
    ];
    alloc(&mut tx, PlayerId::NONE, ZoneKind::DrawPile, 4);
    alloc(&mut tx, PlayerId::NONE, ZoneKind::DrawPile, 5);
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(result.choice.unwrap().min_count, 2);

    let state = resolve(state, owner, vec![hand[0].raw(), hand[1].raw()], false);
    assert_eq!(state.hand_count(owner), 3);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseUseVirtualCard
    );
    assert_eq!(state.resolution.continuation_payload.last(), Some(&1));
    assert!(state.resolution.continuation_payload[5..].contains(&damaged.raw()));
}

#[test]
fn damage_branch_can_recover_a_wounded_character_in_the_target_range() {
    let (state, owner, damaged, other, skill) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(damaged).unwrap().hp = 2;
    tx.state_mut().players.get_mut(other).unwrap().hp = 2;
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDcctjiuxian {
                player: owner,
                event_player: damaged,
                target: PlayerId::NONE,
                card: CardId::NONE,
                action: 2,
                cards: Vec::new(),
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let before = tx.state().players.get(other).unwrap().hp;
    let state = resolve(tx.commit().state, owner, vec![other.raw()], false);
    assert_eq!(state.players.get(other).unwrap().hp, before + 1);
}
