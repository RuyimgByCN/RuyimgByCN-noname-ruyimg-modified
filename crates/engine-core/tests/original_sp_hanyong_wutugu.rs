//! Golden for SP 兀突骨「悍勇」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HANYONG: u32 = 3_661_917_608;
const RANSHANG: u32 = 3_675_767_094;
const RANSHANG_MARK: u32 = 638_947_751;
const WUTUGU: u32 = 4_039_499_432;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/hanyong.skill");
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

fn started(seed: u64, round: u32, hp: i32) -> (GameState, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
            skill_def: SkillDefId::new(HANYONG),
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
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    tx.state_mut().turn.round = round;
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
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let choice = pipeline.choice.expect("optional Hanyong choice");
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice.choice_id.raw()),
                selections: vec![u32::from(accept)],
                cancelled: false,
            },
        },
    )
}

fn mark_count(state: &GameState, owner: PlayerId) -> i32 {
    state
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find_map(|(mark, count)| (*mark == RANSHANG_MARK).then_some(*count))
        .unwrap_or(0)
}

#[test]
fn package_publishes_optional_hanyong_and_wutugu_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    assert_eq!(
        content_compiler::compile_source_named("hanyong", SOURCE)
            .unwrap()
            .manifest
            .rule_vm_abi_version,
        104
    );
    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == HANYONG)
        .unwrap();
    assert_eq!(trigger.event, "card_used");
    assert_eq!(
        trigger.filter,
        "owner_source_wounded_nanman_wanjian_or_normal_spade_sha"
    );
    assert!(trigger.optional);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == WUTUGU)
        .unwrap();
    assert_eq!(character.name_key, "wutugu");
    assert_eq!(character.max_hp, 15);
    assert_eq!(character.skills, [RANSHANG, HANYONG]);
}

#[test]
fn accepted_nanman_increases_every_hit_and_adds_burn_when_round_not_above_hp() {
    let (mut state, owner, targets) = started(2_026_072_303, 10, 14);
    let nanman = inject(
        &mut state,
        owner,
        CardKind::TrickNanman,
        Suit::Heart,
        99_101,
    );
    let before: Vec<_> = targets
        .iter()
        .map(|target| state.players.get(*target).unwrap().hp)
        .collect();

    let (state, pipeline) = play(state, owner, nanman, Vec::new(), 1);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    let (state, pipeline) = resolve_optional(state, pipeline, owner, true, 2);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    for (target, hp) in targets.iter().zip(before) {
        assert_eq!(state.players.get(*target).unwrap().hp, hp - 2);
    }
    assert_eq!(mark_count(&state, owner), 1);
}

#[test]
fn declined_or_ineligible_cards_keep_normal_damage_and_round_gate_only_blocks_burn() {
    let (mut state, owner, targets) = started(2_026_072_304, 10, 14);
    let nanman = inject(&mut state, owner, CardKind::TrickNanman, Suit::Club, 99_201);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, nanman, Vec::new(), 3);
    let (state, _) = resolve_optional(state, pipeline, owner, false, 4);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 1);
    assert_eq!(mark_count(&state, owner), 0);

    let (mut state, owner, targets) = started(2_026_072_305, 15, 14);
    let sha = inject(&mut state, owner, CardKind::Sha, Suit::Spade, 99_202);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, sha, vec![targets[0]], 5);
    let (state, _) = resolve_optional(state, pipeline, owner, true, 6);
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 2);
    assert_eq!(mark_count(&state, owner), 0);

    let (mut state, owner, targets) = started(2_026_072_306, 1, 14);
    let fire_sha = inject(&mut state, owner, CardKind::ShaFire, Suit::Spade, 99_203);
    let target_hp = state.players.get(targets[0]).unwrap().hp;
    let (state, pipeline) = play(state, owner, fire_sha, vec![targets[0]], 7);
    assert!(matches!(pipeline.result, CommandResult::Accepted { .. }));
    assert_ne!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    assert_eq!(state.players.get(targets[0]).unwrap().hp, target_hp - 1);
    assert_eq!(mark_count(&state, owner), 0);
}
