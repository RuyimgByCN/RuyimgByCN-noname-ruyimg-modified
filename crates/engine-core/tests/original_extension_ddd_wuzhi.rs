//! Golden for 3D精选 吴质【灵涌/虚孝】（ABI 622）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DDDLINGYONG: u32 = 2_946_205_848;
const DDDXUXIAO: u32 = 3_539_713_105;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/wuzhi.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    suit: Suit,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_622_000 + serial),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-wuzhi", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend(
        [DDDLINGYONG, DDDXUXIAO].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_exposes_exact_abi_and_two_trigger_contracts() {
    let package = content_compiler::compile_source_named("ddd-wuzhi", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 622);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 2);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "phase_any_skipped" && trigger.filter == "owner_alive" && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_discarded_black_basic_to_discard"
            && !trigger.optional
    }));
}

#[test]
fn lingyong_observes_non_draw_skips_repeats_after_non_sha_and_stops_on_sha() {
    let (mut state, players) = setup();
    let owner = players[0];
    let sha = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Spade,
        1,
    );
    let other = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Heart,
        2,
    );
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::PhaseSkipped {
                current: players[1],
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(committed.output.choice.unwrap().kind, ChoiceKind::Confirm);
    let state = committed.state;

    let (state, use_offer) = resolve(state, owner, vec![1], false, 1);
    assert_eq!(use_offer.choice.unwrap().kind, ChoiceKind::Confirm);
    let (state, repeat_offer) = resolve(state, owner, vec![0], false, 2);
    assert_eq!(repeat_offer.choice.unwrap().kind, ChoiceKind::Confirm);
    let (state, finished) = resolve(state, owner, vec![1], false, 3);
    assert!(matches!(finished.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(other).unwrap().zone, discard);
    assert_eq!(state.cards.get(sha).unwrap().zone, discard);
}

#[test]
fn xuxiao_observes_another_players_discard_and_directly_places_bingliang_then_draws_two() {
    let (mut state, players) = setup();
    let owner = players[0];
    let discarder = players[1];
    let recipient = players[2];
    let black = inject(
        &mut state,
        discarder,
        ZoneKind::Hand,
        CardKind::Sha,
        Suit::Club,
        3,
    );
    let before = state.hand_count(recipient);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DiscardCards {
                player: discarder,
                cards: vec![black],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.unwrap().kind,
        ChoiceKind::SelectCards
    );
    let state = committed.state;
    let (state, target_offer) = resolve(state, owner, vec![black.raw()], false, 4);
    assert_eq!(target_offer.choice.unwrap().kind, ChoiceKind::SelectTargets);
    let (state, result) = resolve(state, owner, vec![recipient.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(recipient), before + 2);
    let judge = state.zones.find(ZoneKind::Judge, recipient).unwrap();
    assert_eq!(state.cards.get(black).unwrap().zone, judge);
    assert_eq!(
        state.cards.get(black).unwrap().kind,
        CardKind::TrickBingliang
    );
}
