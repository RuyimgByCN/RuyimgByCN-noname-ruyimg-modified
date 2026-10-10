//! Golden for 3D精选 曹髦【太思/驱车/潜龙】（ABI 624）。

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Identity, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DDDTAISI: u32 = 632_127_901;
const DDDQUCHE: u32 = 2_669_792_322;
const DDDQUCHE_EFFECT: u32 = 3_815_717_800;
const DDDQIANLONG: u32 = 3_550_796_651;
const SOURCE: &str = include_str!("../../../content/original/ported/ddd/caomao.skill");

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
        def: CardDefId::new(99_624_000 + serial),
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

fn setup(skill_defs: &[u32]) -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-caomao", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 20_260_803, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    tx.state_mut().players.get_mut(players[0]).unwrap().identity = Identity::Lord;
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
    effects.extend(skill_defs.iter().map(|skill_def| EffectRequest::AddSkill {
        player: players[0],
        skill_def: SkillDefId::new(*skill_def),
    }));
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, players)
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
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
fn package_exposes_abi624_active_modifier_and_three_triggers() {
    let package = content_compiler::compile_source_named("ddd-caomao", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 624);
    assert_eq!(package.skill_index.len(), 4);
    assert_eq!(package.trigger_index.len(), 3);
    assert_eq!(package.active_index.len(), 1);
    assert_eq!(package.modifier_index.len(), 1);
}

#[test]
fn taisi_gives_a_turn_entered_discard_card_and_draws_for_its_damage_source() {
    let (mut state, players) = setup(&[DDDTAISI]);
    let (owner, source, target) = (players[0], players[1], players[2]);
    let card = inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DiscardPile,
        CardKind::Other,
        Suit::Club,
        1,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Spade,
        2,
    );
    inject(
        &mut state,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Other,
        Suit::Spade,
        3,
    );
    state.turn.record_hp_change(owner);
    state.turn.record_entered_discard_cards([card]);
    state.turn.record_damage_pair(source, owner);
    let skill = skill_id(&state, owner, DDDTAISI);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaomao {
                player: owner,
                action: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, card_offer) = resolve(state, owner, vec![source.raw()], false, 1);
    assert_eq!(card_offer.choice.unwrap().kind, ChoiceKind::SelectCards);
    let (state, done) = resolve(state, owner, vec![card.raw()], false, 2);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert_eq!(state.cards.get(card).unwrap().owner, source);
    assert_eq!(state.hand_count(owner), 2);
    assert_eq!(state.hand_count(target), 0);
}

#[test]
fn quche_gives_red_uses_black_sha_without_count_and_can_stop_before_next_round() {
    let (mut state, players) = setup(&[DDDQUCHE, DDDQUCHE_EFFECT]);
    let (owner, gift_target, sha_target) = (players[0], players[1], players[2]);
    state.turn.current = owner;
    state.turn.phase = Phase::Play;
    let red = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Heart,
        10,
    );
    let black = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Club,
        11,
    );
    let next_red = inject(
        &mut state,
        owner,
        ZoneKind::Hand,
        CardKind::Other,
        Suit::Diamond,
        12,
    );
    let hp = state.players.get(sha_target).unwrap().hp;
    let skill = skill_id(&state, owner, DDDQUCHE);
    let mut tx = Transaction::begin(state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddCaomao {
                player: owner,
                action: 10,
                cards: vec![red],
                targets: vec![gift_target],
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(state.cards.get(red).unwrap().owner, gift_target);
    let (state, target_offer) = resolve(state, owner, vec![black.raw()], false, 10);
    assert_eq!(target_offer.choice.unwrap().kind, ChoiceKind::SelectTargets);
    let (state, red_offer) = resolve(state, owner, vec![sha_target.raw()], false, 11);
    assert_eq!(red_offer.choice.unwrap().kind, ChoiceKind::SelectCards);
    assert_eq!(state.players.get(sha_target).unwrap().hp, hp - 1);
    assert_eq!(engine_core::query::sha_play_phase_use_count(&state), 0);
    assert_eq!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == DDDQUCHE_EFFECT)
            .unwrap()
            .1,
        2
    );
    let (state, done) = resolve(state, owner, Vec::new(), true, 12);
    assert!(matches!(done.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.cards.get(next_red).unwrap().owner, owner);
}

#[test]
fn qianlong_lord_draws_three_after_killing_another_player() {
    let (mut state, players) = setup(&[DDDQIANLONG]);
    let owner = players[0];
    for serial in 20..23 {
        inject(
            &mut state,
            PlayerId::NONE,
            ZoneKind::DrawPile,
            CardKind::Other,
            Suit::Spade,
            serial,
        );
    }
    let qianlong = skill_id(&state, owner, DDDQIANLONG);
    assert!(state.skill_effectively_enabled(state.skills.get(qianlong).unwrap()));
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::PlayerKilled {
            player: players[1],
            killer: owner,
            hand_equip_cards: Vec::new(),
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert_eq!(state.hand_count(owner), before + 3);
}
