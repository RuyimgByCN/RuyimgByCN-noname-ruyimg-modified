//! Golden for 3D精选 许靖【虚誉/实荐】（ABI 623）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DDDXUYU: u32 = 3_210_825_357;
const DDDSHIJIAN: u32 = 2_262_569_305;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/xujing.skill");

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    kind: CardKind,
    serial: u32,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_623_000 + serial),
        kind,
        suit: Suit::Club,
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
    let package = content_compiler::compile_source_named("ddd-xujing", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
        [DDDXUYU, DDDSHIJIAN].map(|skill_def| EffectRequest::AddSkill {
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
                cancelled: false,
            },
        },
    )
}

fn resolved(source: PlayerId, card: CardId, targets: Vec<PlayerId>) -> DomainEvent {
    DomainEvent::CardResolved {
        source,
        card,
        kind: CardKind::Other.tag(),
        targets,
        damaged_targets: Vec::new(),
        source_cards: vec![card],
        all_sources_were_hand: true,
    }
}

#[test]
fn package_exposes_exact_abi_and_six_trigger_contracts() {
    let package = content_compiler::compile_source_named("ddd-xujing", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 623);
    assert_eq!(package.skill_index.len(), 2);
    assert_eq!(package.trigger_index.len(), 6);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_resolved"
            && trigger.filter
                == "owner_alive_other_source_second_play_card_resolved_hand_equip_nonempty"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.event == "card_used"
            && trigger.filter == "ddd_shijian_watched_source_card_used_targeted"
            && !trigger.optional
    }));
}

#[test]
fn xuyu_draws_then_forces_one_discard_after_the_next_resolved_card() {
    let (mut state, players) = setup();
    let owner = players[0];
    let first = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 1);
    let second = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 2);
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        3,
    );
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[resolved(owner, first, Vec::new())],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, accepted) = resolve(state, owner, vec![1], 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);

    let mut tx = Transaction::begin(state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[resolved(owner, second, Vec::new())],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let committed = tx.commit();
    assert_eq!(
        committed.output.choice.unwrap().kind,
        ChoiceKind::SelectCards
    );
    let (state, next_offer) = resolve(committed.state, owner, vec![first.raw()], 2);
    assert_eq!(next_offer.choice.unwrap().kind, ChoiceKind::Confirm);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(first).unwrap().zone, discard);
    let (state, done) = resolve(state, owner, vec![0], 3);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn shijian_gifts_after_second_play_card_and_replays_next_targeted_sha() {
    let (mut state, players) = setup();
    let (owner, source, target) = (players[0], players[1], players[2]);
    state.turn.current = source;
    state.turn.phase = Phase::Play;
    let gift = inject(&mut state, owner, ZoneKind::Hand, CardKind::Other, 4);
    let first = inject(&mut state, source, ZoneKind::Hand, CardKind::Other, 5);
    let second = inject(&mut state, source, ZoneKind::Hand, CardKind::Other, 6);
    let sha = inject(&mut state, source, ZoneKind::Hand, CardKind::Sha, 7);
    let mut tx = Transaction::begin(state);
    dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card: first,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: source,
            card: second,
            targets: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[resolved(source, second, Vec::new())],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, card_offer) = resolve(state, owner, vec![1], 10);
    assert_eq!(card_offer.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, armed) = resolve(state, owner, vec![gift.raw()], 11);
    assert!(matches!(armed.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(gift).unwrap().owner, source);

    let hp = state.players.get(target).unwrap().hp;
    let revision = state.revision;
    let (state, offered) = apply_command(
        state,
        Command {
            actor: source,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::PlayCard {
                card: sha,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert_eq!(offered.choice.unwrap().kind, ChoiceKind::Custom);
    let (state, result) = resolve(state, owner, vec![0], 13);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 2);
}
