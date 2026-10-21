//! Golden semantics for EXTRA 神周瑜「业炎」「琴音」。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{turn_flag, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const YEYAN: u32 = 3_293_607_680;
const QINYIN: u32 = 3_841_403_199;
const EXTRA: &[u8] = include_bytes!("../../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

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
) -> (engine_core::state::GameState, Vec<PlayerId>, SkillId) {
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
                player_count: 3,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
    let players = engine.state().players.alive_by_seat();
    let owner = engine.state().turn.current;
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
    (tx.state().clone(), players, skill)
}

fn resolve_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    key: u64,
) -> (
    engine_core::state::GameState,
    engine_core::command::PipelineResult,
) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let revision = state.revision;
    engine_core::command::apply_command(
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

fn activate_yeyan(
    state: engine_core::state::GameState,
    owner: PlayerId,
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
                targets: Vec::new(),
                card_targets: Vec::new(),
            },
        },
    )
}

#[test]
fn small_yeyan_damages_one_to_three_selected_players_with_fire() {
    let (mut state, players, skill) = skill_state(YEYAN, Phase::Play);
    let owner = state.turn.current;
    for card in state.hand_cards(owner) {
        state.cards.get_mut(card).unwrap().suit = Suit::Spade;
    }
    let targets: Vec<_> = players
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let before: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();

    let (state, activated) = activate_yeyan(state, owner, skill, 1);
    assert_eq!(activated.choice.unwrap().kind, ChoiceKind::Custom);
    let (state, targets_choice) = resolve_choice(state, owner, vec![0], 2);
    assert_eq!(
        targets_choice.choice.unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, resolved) = resolve_choice(
        state,
        owner,
        targets.iter().map(|target| target.raw()).collect(),
        3,
    );
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    for (target, hp) in targets.iter().zip(before) {
        assert_eq!(state.players.get(*target).unwrap().hp, hp - 1);
    }
    let (_, repeated) = activate_yeyan(state, owner, skill, 4);
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}

#[test]
fn big_yeyan_discards_four_suits_loses_three_hp_and_allocates_three_fire_damage() {
    let (mut state, players, skill) = skill_state(YEYAN, Phase::Play);
    let owner = state.turn.current;
    let cards = state.hand_cards(owner);
    assert!(cards.len() >= 4);
    for (card, suit) in
        cards
            .iter()
            .take(4)
            .zip([Suit::Spade, Suit::Heart, Suit::Club, Suit::Diamond])
    {
        state.cards.get_mut(*card).unwrap().suit = suit;
    }
    let owner_state = state.players.get_mut(owner).unwrap();
    owner_state.max_hp = 7;
    owner_state.hp = 7;
    let targets: Vec<_> = players
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let first_hp = state.players.get(targets[0]).unwrap().hp;
    let second_hp = state.players.get(targets[1]).unwrap().hp;

    let (state, mode) = activate_yeyan(state, owner, skill, 5);
    assert_eq!(
        mode.choice
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, cards_choice) = resolve_choice(state, owner, vec![1], 6);
    assert_eq!(cards_choice.choice.unwrap().kind, ChoiceKind::SelectCards);
    let cost: Vec<_> = cards.iter().take(4).map(|card| card.raw()).collect();
    let (same, rejected) = resolve_choice(
        state.clone(),
        owner,
        vec![cost[0], cost[0], cost[2], cost[3]],
        7,
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));
    assert_eq!(same, state);
    let (state, targets_choice) = resolve_choice(state, owner, cost.clone(), 8);
    assert_eq!(
        targets_choice.choice.unwrap().kind,
        ChoiceKind::SelectTargets
    );
    let (state, allocation) = resolve_choice(
        state,
        owner,
        targets.iter().map(|target| target.raw()).collect(),
        9,
    );
    assert_eq!(state.players.get(owner).unwrap().hp, 4);
    assert_eq!(allocation.choice.unwrap().kind, ChoiceKind::SelectTargets);
    let (state, resolved) = resolve_choice(state, owner, vec![targets[0].raw()], 10);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(targets[0]).unwrap().hp, first_hp - 2);
    assert_eq!(state.players.get(targets[1]).unwrap().hp, second_hp - 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(cost.iter().all(|card| state
        .cards
        .get(engine_api::CardId::new(*card))
        .unwrap()
        .zone
        == discard));
}

#[test]
fn single_target_big_yeyan_can_choose_three_fire_damage() {
    let (state, players, skill) = skill_state(YEYAN, Phase::Play);
    let owner = state.turn.current;
    let target = players.into_iter().find(|player| *player != owner).unwrap();
    let hp = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveYeyan {
                player: owner,
                mode: 1,
                stage: 3,
                index: 0,
                targets: vec![target],
                data: 0,
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    assert_eq!(
        committed
            .output
            .choice
            .as_ref()
            .unwrap()
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [2, 3]
    );
    let (state, resolved) = resolve_choice(committed.state, owner, vec![3], 11);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(target).unwrap().hp, hp - 3);
}

fn end_discard_after(
    state: engine_core::state::GameState,
    owner: PlayerId,
    count: usize,
) -> (
    engine_core::state::GameState,
    Option<engine_api::ChoiceRequest>,
) {
    let cards: Vec<_> = state.hand_cards(owner).into_iter().take(count).collect();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: owner,
            cards,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().turn.flag(turn_flag::DISCARD_PHASE_DISCARDED),
        count as i32
    );
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::End.to_u8(),
        }],
    );
    if count >= 2 {
        assert!(matches!(result, Err(EffectError::ChoiceRequested)));
    } else {
        result.unwrap();
    }
    let committed = tx.commit();
    (committed.state, committed.output.choice)
}

#[test]
fn qinyin_requires_two_discards_and_supports_recover_or_lose_hp() {
    let (mut state, players, _) = skill_state(QINYIN, Phase::Discard);
    let owner = state.turn.current;
    for player in &players {
        let state = state.players.get_mut(*player).unwrap();
        state.hp -= 1;
    }
    let before: Vec<_> = players
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, prompt) = end_discard_after(state, owner, 2);
    assert_eq!(prompt.unwrap().kind, ChoiceKind::Custom);
    let (state, recovered) = resolve_choice(state, owner, vec![1], 12);
    assert!(matches!(recovered.result, CommandResult::Accepted { .. }));
    for (player, hp) in players.iter().zip(before) {
        assert_eq!(state.players.get(*player).unwrap().hp, hp + 1);
    }

    let (state, players, _) = skill_state(QINYIN, Phase::Discard);
    let owner = state.turn.current;
    let before: Vec<_> = players
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, _) = end_discard_after(state, owner, 2);
    let (state, lost) = resolve_choice(state, owner, vec![0], 13);
    assert!(matches!(lost.result, CommandResult::Accepted { .. }));
    for (player, hp) in players.iter().zip(before) {
        assert_eq!(state.players.get(*player).unwrap().hp, hp - 1);
    }

    let (state, _, _) = skill_state(QINYIN, Phase::Discard);
    let owner = state.turn.current;
    let (_, no_prompt) = end_discard_after(state, owner, 1);
    assert!(no_prompt.is_none());
}
