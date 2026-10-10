//! Golden coverage for old 短兵 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ContinuationKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const DUANBING: u32 = 4_181_445_466;
const UNLIMITED_SHA: u32 = 4_290_000_277;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/duanbing.skill");

#[test]
fn additional_sha_target_is_limited_to_final_distance_one() {
    let package = content_compiler::compile_source_named("duanbing-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 277);
    let fixture = content_compiler::compile_source_named(
        "duanbing-unlimited-fixture",
        "skill unlimited_sha(4290000277) { modifier card_distance_unlimited(sha) override 1; }",
    )
    .unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.load_package(&fixture).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 2_026_072_938,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
    let owner_index = seats.iter().position(|player| *player == owner).unwrap();
    let first = seats[(owner_index + 1) % seats.len()];
    let far = seats[(owner_index + 2) % seats.len()];
    let near = seats[(owner_index + 3) % seats.len()];
    let mut tx = Transaction::begin(engine.state().clone());
    let effects = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain([
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DUANBING),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(UNLIMITED_SHA),
            },
        ])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    // Fixed HP so damage assertions are independent of the booted roster.
    for seat in &seats {
        let participant = tx.state_mut().players.get_mut(*seat).unwrap();
        participant.max_hp = 4;
        participant.hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let sha = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(93_277),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(sha);

    let (state, offered) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![first],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(offered.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseAddCurrentShaTarget
    );
    let choice = offered.choice.unwrap();
    assert!(choice
        .options
        .iter()
        .any(|option| option.token == near.raw()));
    assert!(choice
        .options
        .iter()
        .all(|option| option.token != far.raw()));

    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![near.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardTargetAdded { target, .. } if *target == near
    )));
    // The 杀 now resolves through its response window: drive every pending
    // choice (each target declines 闪) until the damage commits.
    let mut state = state;
    let mut key = 3u64;
    while state.has_pending_choice() {
        let (next, r) = apply_command(
            state.clone(),
            Command {
                actor: PlayerId::new(state.resolution.pending_observer),
                expected_revision: ExpectedRevision(0),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                    selections: vec![],
                    cancelled: true,
                },
            },
        );
        assert!(
            matches!(r.result, CommandResult::Accepted { .. }),
            "{:?}",
            r.result
        );
        state = next;
        key += 1;
    }
    assert_eq!(state.players.get(first).unwrap().hp, 3);
    assert_eq!(state.players.get(near).unwrap().hp, 3);
    assert_eq!(state.players.get(far).unwrap().hp, 4);
}
