//! Golden coverage for 顾雍「秉壹」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, ChoiceRequest, Command, CommandKind, CommandResult,
    DomainEvent, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, SkillId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GUYONG: u32 = 3_099_069_622;
const SHENXING: u32 = 3_283_042_324;
const OLBINGYI: u32 = 2_703_324_677;
const OLBINGYI_SOURCE: &str =
    include_str!("../../../content/original/ported/yijiang/olbingyi.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: [PlayerId; 3],
}

struct Step {
    state: GameState,
    events: Vec<DomainEvent>,
    choice: Option<ChoiceRequest>,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLBINGYI),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 8;
        participant.max_hp = 8;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        others: [players[1], players[2], players[3]],
    }
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
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

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled,
        },
        key,
    )
}

fn discard(state: GameState, player: PlayerId, card: CardId) -> Step {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player,
            cards: vec![card],
        }],
    );
    assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
    let committed = tx.commit();
    Step {
        state: committed.state,
        events: committed.output.events,
        choice: committed.output.choice,
    }
}

fn olbingyi_instance(state: &GameState, owner: PlayerId) -> SkillId {
    state
        .skills
        .sorted_live()
        .into_iter()
        .find(|skill| skill.owner == owner && skill.def.raw() == OLBINGYI)
        .unwrap()
        .id
}

fn assert_optional(choice: &Option<ChoiceRequest>) {
    let choice = choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [0, 1]
    );
}

#[test]
fn package_publishes_complete_guyong_and_exact_abi56_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == GUYONG)
        .unwrap();
    assert_eq!(character.name_key, "guyong");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 3);
    assert_eq!(character.skills, [SHENXING, OLBINGYI]);

    let compiled = content_compiler::compile_source_named("olbingyi", OLBINGYI_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 56);
    for expected in compiled.modules {
        let actual = package
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn non_uniform_hand_is_publicly_revealed_without_drawing() {
    let mut setup = setup(2_026_073_601);
    let discarded = inject(&mut setup.state, setup.owner, 99_601, Suit::Club);
    let red = inject(&mut setup.state, setup.owner, 99_602, Suit::Heart);
    let black = inject(&mut setup.state, setup.owner, 99_603, Suit::Spade);
    let step = discard(setup.state, setup.owner, discarded);
    assert_optional(&step.choice);

    let before = step.state.hand_count(setup.owner);
    let (state, accepted) = resolve(step.state, setup.owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.owner), before);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red, black] && to.is_empty()
    )));
    let skill = olbingyi_instance(&state, setup.owner);
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 1);
}

#[test]
fn uniform_hand_draws_selected_participants_by_seat_after_atomic_validation() {
    let mut setup = setup(2_026_073_602);
    let discarded = inject(&mut setup.state, setup.owner, 99_611, Suit::Club);
    let red_a = inject(&mut setup.state, setup.owner, 99_612, Suit::Heart);
    let red_b = inject(&mut setup.state, setup.owner, 99_613, Suit::Diamond);
    let step = discard(setup.state, setup.owner, discarded);
    let (state, accepted) = resolve(step.state, setup.owner, vec![1], false, 10);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsRevealed { cards, to }
            if cards == &vec![red_a, red_b] && to.is_empty()
    )));
    let choice = accepted.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::SelectTargets);
    assert_eq!(
        (choice.min_count, choice.max_count, choice.allow_cancel),
        (1, 2, true)
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseOthersAfterUniformHandReveal
    );

    let before_forged = canonical_state_hash(&state);
    let (unchanged, forged) = resolve(state, setup.owner, vec![setup.owner.raw()], false, 11);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forged);
    let (unchanged, duplicate) = resolve(
        unchanged,
        setup.owner,
        vec![setup.others[0].raw(), setup.others[0].raw()],
        false,
        12,
    );
    assert!(matches!(duplicate.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&unchanged), before_forged);

    let owner_before = unchanged.hand_count(setup.owner);
    let first_before = unchanged.hand_count(setup.others[0]);
    let middle_before = unchanged.hand_count(setup.others[1]);
    let last_before = unchanged.hand_count(setup.others[2]);
    let (state, drawn) = resolve(
        unchanged,
        setup.owner,
        vec![setup.others[2].raw(), setup.others[0].raw()],
        false,
        13,
    );
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), owner_before + 1);
    assert_eq!(state.hand_count(setup.others[0]), first_before + 1);
    assert_eq!(state.hand_count(setup.others[1]), middle_before);
    assert_eq!(state.hand_count(setup.others[2]), last_before + 1);
    assert_eq!(
        drawn
            .events
            .iter()
            .filter_map(|event| match event {
                DomainEvent::CardsDrawn { player, .. } => Some(*player),
                _ => None,
            })
            .collect::<Vec<_>>(),
        [setup.owner, setup.others[0], setup.others[2]]
    );
}

#[test]
fn decline_does_not_consume_limit_and_acceptance_resets_on_next_phase() {
    let mut setup = setup(2_026_073_603);
    let first = inject(&mut setup.state, setup.owner, 99_621, Suit::Club);
    let second = inject(&mut setup.state, setup.owner, 99_622, Suit::Heart);
    let _red = inject(&mut setup.state, setup.owner, 99_623, Suit::Diamond);
    let _black = inject(&mut setup.state, setup.owner, 99_624, Suit::Spade);

    let step = discard(setup.state, setup.owner, first);
    assert_optional(&step.choice);
    let (state, declined) = resolve(step.state, setup.owner, vec![0], false, 20);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    let skill = olbingyi_instance(&state, setup.owner);
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 0);

    let step = discard(state, setup.owner, second);
    assert_optional(&step.choice);
    let (state, accepted) = resolve(step.state, setup.owner, vec![1], false, 21);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 1);

    let mut state = state;
    let third = inject(&mut state, setup.owner, 99_625, Suit::Club);
    let same_phase = discard(state, setup.owner, third);
    assert!(same_phase.choice.is_none());
    assert!(same_phase.events.iter().any(|event| matches!(
        event,
        DomainEvent::CardsMoved { cards, .. } if cards == &vec![third]
    )));

    let mut tx = Transaction::begin(same_phase.state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: setup.owner,
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let mut state = tx.commit().state;
    assert_eq!(state.skills.get(skill).unwrap().play_phase_fire_count, 0);
    let fourth = inject(&mut state, setup.owner, 99_626, Suit::Club);
    let next_phase = discard(state, setup.owner, fourth);
    assert_optional(&next_phase.choice);
}
