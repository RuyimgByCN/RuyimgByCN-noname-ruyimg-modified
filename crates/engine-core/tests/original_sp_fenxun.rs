//! Golden coverage for old 奋迅 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const FENXUN: u32 = 3_362_200_043;
const FENXUN2: u32 = 3_152_146_054;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fenxun.skill");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_000 + kind.tag() as u32),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn discards_any_hand_card_and_unlocks_the_bound_target_for_this_turn() {
    let package = content_compiler::compile_source_named("fenxun-test", SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_072_937,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let target = seats[(owner_index + 2) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENXUN),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    assert!(matches!(
        query(
            tx.state(),
            &QueryRequest::Distance {
                from: owner,
                to: target,
            },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(distance) if distance > 1
    ));
    let cost = inject(&mut tx, owner, CardKind::Other);
    let sha = inject(&mut tx, owner, CardKind::Sha);
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == FENXUN)
        .unwrap();
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![cost],
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| { state.skills.get(*skill).unwrap().def.raw() == FENXUN2 }));
    assert_eq!(
        state
            .zones
            .get(state.cards.get(cost).unwrap().zone)
            .unwrap()
            .kind,
        ZoneKind::DiscardPile
    );

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.hand_cards(owner).contains(&sha));
}
