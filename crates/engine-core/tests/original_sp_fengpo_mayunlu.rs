//! Golden coverage for SP Ma Yunlu's 凤魄 rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const FENGPO: u32 = 4_276_310_203;
const MASHU: u32 = 2;
const MAYUNLU: u32 = 3_591_117_734;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/fengpo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn started(seed: u64) -> (GameState, PlayerId, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("fengpo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
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
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(FENGPO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 20;
        participant.hp = 15;
    }
    (tx.commit().state, owner, target, other)
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::PlayCard {
                card,
                targets,
                card_targets: Vec::new(),
            },
        },
    )
}

fn resolve_optional(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline.choice.expect("凤魄应询问是否发动");
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: accept.then_some(1).into_iter().collect(),
                cancelled: !accept,
            },
        },
    )
}

fn resolve_branch(
    state: GameState,
    pipeline: PipelineResult,
    owner: PlayerId,
    branch: u32,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VmResume
    );
    let choice = pipeline.choice.expect("凤魄应询问分支");
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![branch],
                cancelled: false,
            },
        },
    )
}

fn cancel_remaining_choices(
    mut state: GameState,
    mut pipeline: PipelineResult,
    mut key: u64,
) -> (GameState, PipelineResult) {
    while let Some(choice) = pipeline.choice.take() {
        assert_eq!(choice.kind, ChoiceKind::SelectCards);
        let revision = state.revision;
        (state, pipeline) = apply_command(
            state,
            Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: ChoiceId::new(choice.choice_id.raw()),
                    selections: Vec::new(),
                    cancelled: true,
                },
            },
        );
        key += 1;
    }
    (state, pipeline)
}

#[test]
fn package_publishes_fengpo_and_mayunlu_without_javascript() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 302);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == FENGPO && skill.name == "fengpo" && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == FENGPO)
        .unwrap();
    assert_eq!(trigger.event, "card_targeted");
    assert_eq!(
        trigger.filter,
        "owner_source_sha_or_juedou_single_target_first_same_name_this_turn"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == MAYUNLU)
        .unwrap();
    assert_eq!(character.skills, [FENGPO, MASHU]);
    assert!(!serde_json::to_string(&package).unwrap().contains(".js"));
}

#[test]
fn first_sha_and_first_juedou_are_tracked_independently() {
    let (mut state, owner, target, _) = started(2_026_072_706);
    let first_sha = inject(&mut state, owner, CardKind::Sha, Suit::Spade, 229_001);
    let (state, optional) = play(state, owner, first_sha, vec![target], 1);
    let (mut state, declined) = resolve_optional(state, optional, owner, false, 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));

    let second_sha = inject(&mut state, owner, CardKind::Sha, Suit::Heart, 229_002);
    let mut tx = Transaction::begin(state);
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: owner,
            card: second_sha,
            targets: vec![target],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    state = tx.commit().state;
    let juedou = inject(
        &mut state,
        owner,
        CardKind::TrickJuedou,
        Suit::Club,
        229_003,
    );
    let (state, result) = play(state, owner, juedou, vec![target], 3);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(result.choice.unwrap().kind, ChoiceKind::Confirm);
}

#[test]
fn sha_draw_x_branch_adds_one_damage_and_multi_target_is_rejected_by_filter() {
    let (mut state, owner, target, other) = started(2_026_072_707);
    inject(
        &mut state,
        target,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        229_011,
    );
    inject(&mut state, target, CardKind::Tao, Suit::Diamond, 229_012);
    let sha = inject(&mut state, owner, CardKind::Sha, Suit::Spade, 229_013);
    let hand_before = state.hand_count(owner);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, optional) = play(state, owner, sha, vec![target], 10);
    let (state, branch) = resolve_optional(state, optional, owner, true, 11);
    let (state, result) = resolve_branch(state, branch, owner, 1, 12);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 2);

    let package = content_compiler::compile_source_named("fengpo-multi", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_072_708, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: players[0],
            skill_def: SkillDefId::new(FENGPO),
        }],
    )
    .unwrap();
    let card = inject(
        tx.state_mut(),
        players[0],
        CardKind::Sha,
        Suit::Spade,
        229_014,
    );
    let effects = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: players[0],
            card,
            targets: vec![players[1], players[2]],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());
    assert_ne!(target, other);
}

#[test]
fn juedou_draw_one_branch_adds_current_diamond_count_to_damage() {
    let (mut state, owner, target, _) = started(2_026_072_709);
    inject(
        &mut state,
        target,
        CardKind::TrickWuzhong,
        Suit::Diamond,
        229_021,
    );
    inject(&mut state, target, CardKind::Tao, Suit::Diamond, 229_022);
    let juedou = inject(
        &mut state,
        owner,
        CardKind::TrickJuedou,
        Suit::Club,
        229_023,
    );
    let hand_before = state.hand_count(owner);
    let hp_before = state.players.get(target).unwrap().hp;
    let (state, optional) = play(state, owner, juedou, vec![target], 20);
    let (state, branch) = resolve_optional(state, optional, owner, true, 21);
    let (state, result) = resolve_branch(state, branch, owner, 2, 22);
    let (state, result) = cancel_remaining_choices(state, result, 23);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before);
    assert_eq!(state.players.get(target).unwrap().hp, hp_before - 3);
}
