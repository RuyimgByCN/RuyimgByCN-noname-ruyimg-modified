//! Golden semantics for EXTRA 神吕蒙「涉猎」「攻心」。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const SHELIE: u32 = 4_211_343_786;
const GONGXIN: u32 = 3_100_453_257;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn cancel_startup_choices(engine: &mut Engine) {
    let mut key = 90_000;
    while engine.state().resolution.pending_choice_id != 0 {
        let state = engine.state();
        let actor = PlayerId::new(state.resolution.pending_observer);
        let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
        let revision = state.revision;
        assert!(matches!(
            engine.submit(Command {
                actor,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
}

fn skill_state(
    skill_def: u32,
    phase: Phase,
) -> (engine_core::state::GameState, PlayerId, PlayerId, SkillId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let owner = engine.state().turn.current;
    let target = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let old: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: phase.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
        ],
    )
    .unwrap();
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == skill_def)
        .unwrap()
        .id;
    (tx.state().clone(), owner, target, skill)
}

fn choose(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    )
}

#[test]
fn shelie_replaces_draw_and_requires_exactly_one_card_per_distinct_suit() {
    let (mut state, owner, _, skill) = skill_state(SHELIE, Phase::Draw);
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let top: Vec<_> = {
        let zone = state.zones.get(draw).unwrap();
        zone.cards[zone.cards.len() - 5..].to_vec()
    };
    for (card, suit) in top.iter().zip([
        Suit::Heart,
        Suit::Heart,
        Suit::Diamond,
        Suit::Club,
        Suit::Spade,
    ]) {
        state.cards.get_mut(*card).unwrap().suit = suit;
    }
    let hand_before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ReplaceDrawWithDistinctSuitSelection {
                player: owner,
                count: 5,
                stage: 0,
                cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(tx.state().turn.flag(turn_flag::DRAW_PHASE_DRAW_ZEROED), 1);
    let committed = tx.commit();
    assert!(committed.output.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.len() == 5 && to.is_empty())
    }));
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(choice.min_count, 4);
    assert_eq!(choice.max_count, 4);

    let (same, rejected) = choose(
        committed.state.clone(),
        owner,
        vec![top[0].raw(), top[1].raw(), top[2].raw(), top[3].raw()],
        1,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(same, committed.state);

    let selected = vec![top[0], top[2], top[3], top[4]];
    let (state, accepted) = choose(
        committed.state,
        owner,
        selected.iter().map(|card| card.raw()).collect(),
        2,
    );
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 4);
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    assert!(selected
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == hand));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(top[1]).unwrap().zone, discard);
}

fn activate_gongxin(
    state: engine_core::state::GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let revision = state.revision;
    engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn gongxin_can_put_one_inspected_heart_on_draw_pile_top() {
    let (mut state, owner, target, skill) = skill_state(GONGXIN, Phase::Play);
    let cards = state.hand_cards(target);
    let heart = cards[0];
    state.cards.get_mut(heart).unwrap().suit = Suit::Heart;
    for card in &cards[1..] {
        state.cards.get_mut(*card).unwrap().suit = Suit::Spade;
    }
    let (state, activated) = activate_gongxin(state, owner, target, skill, 3);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectCards);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [heart.raw()]
    );
    assert!(activated.events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to } if cards.contains(&heart) && to == &[owner])
    }));

    let (state, destination) = choose(state, owner, vec![heart.raw()], 4);
    assert_eq!(destination.choice.unwrap().kind, ChoiceKind::Custom);
    let (state, resolved) = choose(state, owner, vec![1], 5);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    let draw = state
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(heart).unwrap().zone, draw);
    assert_eq!(
        state.zones.get(draw).unwrap().cards.last().copied(),
        Some(heart)
    );

    let (_, repeated) = activate_gongxin(state, owner, target, skill, 6);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn gongxin_can_discard_the_selected_heart_and_no_heart_only_reveals() {
    let (mut state, owner, target, skill) = skill_state(GONGXIN, Phase::Play);
    let heart = state.hand_cards(target)[0];
    state.cards.get_mut(heart).unwrap().suit = Suit::Heart;
    let (state, _) = activate_gongxin(state, owner, target, skill, 7);
    let (state, _) = choose(state, owner, vec![heart.raw()], 8);
    let (state, resolved) = choose(state, owner, vec![0], 9);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(heart).unwrap().zone, discard);

    let (mut state, owner, target, skill) = skill_state(GONGXIN, Phase::Play);
    for card in state.hand_cards(target) {
        state.cards.get_mut(card).unwrap().suit = Suit::Club;
    }
    let hand = state.hand_cards(target);
    let (state, activated) = activate_gongxin(state, owner, target, skill, 10);
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert!(activated.choice.is_none());
    assert_eq!(state.hand_cards(target), hand);
}
