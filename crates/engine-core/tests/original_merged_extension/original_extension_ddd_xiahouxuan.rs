//! Golden for 3D精选 夏侯玄【朗怀/玄论】（ABI 638）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const LANGHUAI: u32 = 4_025_119_557;
const XUANLUN: u32 = 3_478_177_951;
const SOURCE: &str = include_str!("../../../../content/original/ported/ddd/xiahouxuan.skill");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>) {
    let package = content_compiler::compile_source_named("ddd-xiahouxuan", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 638);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap())
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 20_260_804, "identity-standard").unwrap();
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
        [LANGHUAI, XUANLUN].map(|skill_def| EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(skill_def),
        }),
    );
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

fn inject(state: &mut GameState, owner: PlayerId, serial: u32, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(99_638_000 + serial),
        kind: CardKind::Other,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch(state: GameState, event: DomainEvent) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(&mut tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    match run_effects(&mut tx, effects) {
        Ok(_) | Err(EffectError::ChoiceRequested) => tx.commit().state,
        Err(error) => panic!("dispatch failed: {error:?}"),
    }
}

fn choose(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
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

fn begin_xuanlun_settlement(state: GameState, owner: PlayerId) -> GameState {
    let skill = skill_id(&state, owner, XUANLUN);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDddXiahouxuan {
            player: owner,
            action: 1,
            skill,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), 4);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDddXiahouxuan {
                player: owner,
                action: 2,
                skill,
            }],
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    tx.commit().state
}

#[test]
fn langhuai_draws_present_then_missing_suit_counts_and_toggles() {
    let (mut state, players) = setup();
    let owner = players[0];
    inject(&mut state, owner, 1, Suit::Spade);
    inject(&mut state, owner, 2, Suit::Heart);
    inject(&mut state, owner, 3, Suit::Heart);
    state.turn.current = owner;
    state.turn.phase = Phase::Draw;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    assert!(state.has_pending_choice());
    let (state, result) = choose(state, owner, vec![1], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 5);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, owner, LANGHUAI))
            .unwrap()
            .mark,
        1
    );

    let mut state = state;
    state.turn.clear_draw_phase_fixed_count();
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    let before = state.hand_count(owner);
    let mut suits = 0u8;
    for card in state.hand_cards(owner) {
        let card = state.cards.get(card).unwrap();
        suits |= match engine_core::query::effective_card_suit(&state, owner, card) {
            Suit::Spade => 1,
            Suit::Heart => 2,
            Suit::Club => 4,
            Suit::Diamond => 8,
            Suit::None => 0,
        };
    }
    let missing = 4 - suits.count_ones();
    let (state, result) = choose(state, owner, vec![1], 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + missing);
    assert_eq!(
        state
            .skills
            .get(skill_id(&state, owner, LANGHUAI))
            .unwrap()
            .mark,
        0
    );
}

#[test]
fn xuanlun_can_force_the_next_langhuai_then_forced_activation_clears_it() {
    let (state, players) = setup();
    let owner = players[0];
    let state = begin_xuanlun_settlement(state, owner);
    let (mut state, result) = choose(state, owner, vec![2], 10);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let xuanlun = skill_id(&state, owner, XUANLUN);
    assert_eq!(state.skills.get(xuanlun).unwrap().mark & 2, 2);

    state.turn.current = owner;
    state.turn.phase = Phase::Draw;
    let state = dispatch(
        state,
        DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Draw,
        },
    );
    assert!(!state.has_pending_choice());
    assert_eq!(state.skills.get(xuanlun).unwrap().mark & 2, 0);
    assert_eq!(state.turn.draw_phase_fixed_count(), Some(0));
}

#[test]
fn xuanlun_places_four_selected_cards_on_top_and_bottom_in_requested_order() {
    let (state, players) = setup();
    let owner = players[0];
    let state = begin_xuanlun_settlement(state, owner);
    let (state, result) = choose(state, owner, vec![1], 20);
    assert!(result.choice.is_some());
    let cards = state.hand_cards(owner);
    assert_eq!(cards.len(), 4);
    let order = vec![
        cards[0].raw(),
        cards[1].raw(),
        0,
        cards[2].raw(),
        cards[3].raw(),
    ];
    let (state, result) = choose(state, owner, order, 21);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let pile = &state.zones.get(draw).unwrap().cards;
    assert_eq!(&pile[..2], &[cards[2], cards[3]]);
    assert_eq!(&pile[pile.len() - 2..], &[cards[1], cards[0]]);
}
