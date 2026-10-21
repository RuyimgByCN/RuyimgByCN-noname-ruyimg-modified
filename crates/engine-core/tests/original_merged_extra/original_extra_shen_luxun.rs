//! Golden semantics for EXTRA 神陆逊「摧克」「绽火」。

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    MarkDefId, PlayerId, SkillDefId, SkillId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{CardKind, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const CUIKE: u32 = 2_516_554_182;
const DINGHUO: u32 = 3_907_113_145;
const JUNLVE_MARK: u32 = 1_681_567_279;
const CHAIN_MARK: u32 = 2;
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
    marks: u32,
    player_count: u8,
) -> (engine_core::state::GameState, PlayerId, SkillId) {
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
                player_count,
                rng_seed: skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    cancel_startup_choices(&mut engine);
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
                phase: Phase::Play.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(skill_def),
            },
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(JUNLVE_MARK),
                delta: marks as i32,
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
    (tx.state().clone(), owner, skill)
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
    let choice = ChoiceId::new(state.resolution.pending_choice_id);
    engine_core::command::apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: choice,
                selections,
                cancelled: false,
            },
        },
    )
}

fn offer_cuike(
    state: engine_core::state::GameState,
    owner: PlayerId,
    skill: SkillId,
) -> (engine_core::state::GameState, engine_api::ChoiceRequest) {
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveMarkParityControl {
                player: owner,
                mark: MarkDefId::new(JUNLVE_MARK),
                mass_threshold: 7,
                damage_amount: 1,
                stage: 0,
                index: 0,
                target: PlayerId::NONE,
                targets: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    (committed.state, committed.output.choice.unwrap())
}

fn mark_count(state: &engine_core::state::GameState, player: PlayerId, mark: u32) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(definition, _)| *definition == mark)
        .map(|(_, count)| *count)
        .unwrap_or(0)
}

#[test]
fn cuike_is_offered_at_play_phase_begin() {
    let (state, owner, _) = skill_state(CUIKE, 1, 2);
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.phase = Phase::Draw;
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::Play.to_u8(),
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let committed = tx.commit();
    let choice = committed.output.choice.unwrap();
    assert_eq!(choice.observer, owner);
    assert_eq!(choice.kind, ChoiceKind::Confirm);
}

#[test]
fn cuike_odd_damages_even_links_and_discards_hej() {
    let (state, owner, skill) = skill_state(CUIKE, 1, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let hp = state.players.get(target).unwrap().hp;
    let (state, target_choice) = offer_cuike(state, owner, skill);
    assert_eq!(target_choice.kind, ChoiceKind::SelectTargets);
    let (state, result) = choose(state, owner, vec![target.raw()], 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.players.get(target).unwrap().hp, hp - 1);

    let (mut state, owner, skill) = skill_state(CUIKE, 2, 2);
    let target = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let card = state.hand_cards(target)[0];
    state.cards.get_mut(card).unwrap().kind = CardKind::Other;
    let (state, _) = offer_cuike(state, owner, skill);
    let (state, result) = choose(state, owner, vec![target.raw()], 2);
    let card_choice = result.choice.unwrap();
    assert_eq!(card_choice.kind, ChoiceKind::SelectCards);
    assert!(card_choice
        .options
        .iter()
        .any(|option| option.token == card.raw()));
    let (state, result) = choose(state, owner, vec![card.raw()], 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(mark_count(&state, target, CHAIN_MARK), 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(card).unwrap().zone, discard);
}

#[test]
fn cuike_above_seven_can_clear_marks_and_damage_all_other_players() {
    let (state, owner, skill) = skill_state(CUIKE, 9, 3);
    let others: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let hp: Vec<_> = others
        .iter()
        .map(|player| state.players.get(*player).unwrap().hp)
        .collect();
    let (state, _) = offer_cuike(state, owner, skill);
    let (state, result) = choose(state, owner, vec![others[0].raw()], 4);
    let confirm = result.choice.unwrap();
    assert_eq!(confirm.kind, ChoiceKind::Custom);
    assert_eq!(
        confirm
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
    let (state, result) = choose(state, owner, vec![1], 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(mark_count(&state, owner, JUNLVE_MARK), 0);
    assert_eq!(state.players.get(others[0]).unwrap().hp, hp[0] - 2);
    assert_eq!(state.players.get(others[1]).unwrap().hp, hp[1] - 1);
}

#[test]
fn dinghuo_enforces_linked_mark_capped_targets_and_resolves_in_order() {
    let (mut state, owner, skill) = skill_state(DINGHUO, 2, 3);
    let others: Vec<_> = state
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: others[0],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: others[1],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    state = tx.state().clone();

    let revision = state.revision;
    let (_, rejected) = engine_core::command::apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner, others[0], others[1]],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::RemoveMark {
            player: owner,
            mark: MarkDefId::new(CHAIN_MARK),
            amount: 0,
        }],
    )
    .unwrap();
    state = tx.state().clone();
    let revision = state.revision;
    let (_, rejected) = engine_core::command::apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(11),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![owner],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(rejected.result, CommandResult::Rejected { .. }));

    let mut tx = Transaction::begin(state);
    let mut equipment = Vec::new();
    for target in &others {
        let card = tx.state().hand_cards(*target)[0];
        tx.state_mut().cards.get_mut(card).unwrap().kind = CardKind::Weapon;
        let hand = tx.state().zones.find(ZoneKind::Hand, *target).unwrap();
        let equip = tx.state().zones.find(ZoneKind::Equip, *target).unwrap();
        run_effects(
            &mut tx,
            vec![EffectRequest::MoveCards {
                cards: vec![card],
                to: equip,
                from: Some(hand),
            }],
        )
        .unwrap();
        equipment.push(card);
    }
    state = tx.state().clone();
    let hp = state.players.get(others[1]).unwrap().hp;
    let revision = state.revision;
    let (state, activated) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(12),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: others.clone(),
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    let choice = activated.choice.unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(mark_count(&state, owner, JUNLVE_MARK), 0);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert!(equipment
        .iter()
        .all(|card| state.cards.get(*card).unwrap().zone == discard));

    let (state, resolved) = choose(state, owner, vec![others[1].raw()], 13);
    assert!(matches!(resolved.result, CommandResult::Accepted { .. }));
    assert!(state.players.get(others[1]).unwrap().hp < hp);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddMark {
                player: owner,
                mark: MarkDefId::new(JUNLVE_MARK),
                delta: 2,
            },
            EffectRequest::AddMark {
                player: others[0],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
            EffectRequest::AddMark {
                player: others[1],
                mark: MarkDefId::new(CHAIN_MARK),
                delta: 1,
            },
        ],
    )
    .unwrap();
    let state = tx.state().clone();
    let revision = state.revision;
    let (_, repeated) = engine_core::command::apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(14),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: others,
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(repeated.result, CommandResult::Rejected { .. }));
}
