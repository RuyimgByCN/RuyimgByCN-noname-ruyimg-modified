//! Golden for 文鸯「铿锵」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDKENGQIANG: u32 = 3_956_037_825;
const SOURCE: &str = include_str!("../../../../content/original/ported/sixiang/stdkengqiang.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/sixiang/sixiang.npack.json");

fn inject(state: &mut GameState, owner: PlayerId, kind: CardKind, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn started(seed: u64) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let targets = players[1..].to_vec();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in &players {
        effects.push(EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(STDKENGQIANG),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.max_hp = 15;
        participant.hp = 10;
    }
    (tx.commit().state, owner, targets)
}

fn play(
    state: GameState,
    owner: PlayerId,
    card: CardId,
    targets: Vec<PlayerId>,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
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
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline
        .choice
        .expect("铿锵 should ask whether to activate");
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![1],
                cancelled: false,
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
    let choice = pipeline
        .choice
        .expect("铿锵 should ask which branch to use");
    assert_eq!(choice.kind, ChoiceKind::Custom);
    assert_eq!(
        choice
            .options
            .iter()
            .map(|option| option.token)
            .collect::<Vec<_>>(),
        [1, 2]
    );
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![branch],
                cancelled: false,
            },
        },
    )
}

fn next_owner_turn(state: GameState, owner: PlayerId, other: PlayerId) -> GameState {
    let mut state = state;
    state.turn.phase = Phase::End;
    let (state, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::EndPhase,
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.current, other);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        }],
    )
    .unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_optional_damage_card_trigger_at_abi_141() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.character_index.len(), 101);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == STDKENGQIANG && !skill.locked));
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == STDKENGQIANG)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(trigger.filter, "owner_source_damage_card_once_per_turn");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("stdkengqiang", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 141);
}

#[test]
fn non_damage_card_does_not_consume_trigger_and_draw_branch_draws_two() {
    let (mut state, owner, targets) = started(2_026_072_401);
    let wuzhong = inject(&mut state, owner, CardKind::TrickWuzhong, 141_001);
    let (state_after_wuzhong, result) = play(state, owner, wuzhong, Vec::new(), 1);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state_after_wuzhong.has_pending_choice());

    state = state_after_wuzhong;
    let sha = inject(&mut state, owner, CardKind::Sha, 141_002);
    let hand_before = state.hand_count(owner);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, optional) = play(state, owner, sha, vec![targets[0]], 2);
    let (state, branch) = resolve_optional(state, optional, owner, 3);
    let (state, result) = resolve_branch(state, branch, owner, 1, 4);

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 1);
}

#[test]
fn lose_hp_branch_increases_current_damage_once_per_turn_and_resets_next_turn() {
    let (mut state, owner, targets) = started(2_026_072_402);
    let sha = inject(&mut state, owner, CardKind::Sha, 141_011);
    let owner_hp = state.players.get(owner).unwrap().hp;
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, optional) = play(state, owner, sha, vec![targets[0]], 5);
    let (state, branch) = resolve_optional(state, optional, owner, 6);
    let (mut state, result) = resolve_branch(state, branch, owner, 2, 7);

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(owner).unwrap().hp, owner_hp - 1);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 2);

    let nanman = inject(&mut state, owner, CardKind::TrickNanman, 141_012);
    let before: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();
    let (state, result) = play(state, owner, nanman, Vec::new(), 8);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    for (target, hp) in targets.iter().zip(before) {
        assert_eq!(state.players.get(*target).unwrap().hp, hp - 1);
    }

    let mut state = next_owner_turn(state, owner, targets[0]);
    let sha = inject(&mut state, owner, CardKind::Sha, 141_013);
    let (state, result) = play(state, owner, sha, vec![targets[0]], 9);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}
