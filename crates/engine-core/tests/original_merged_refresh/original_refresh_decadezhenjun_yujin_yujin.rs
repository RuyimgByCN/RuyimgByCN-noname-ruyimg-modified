//! Golden coverage for 界于禁「镇军」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YUJIN_YUJIN: u32 = 3_873_317_599;
const DECADEZHENJUN: u32 = 2_634_318_954;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/refresh/decadezhenjun.skill");
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const LEGACY_REZHENJUN_SOURCE: &str =
    include_str!("../../../../content/original/ported/yijiang/rezhenjun.skill");
const LEGACY_REZHENJUN_LOGIC_HASH: [u8; 32] = [
    111, 13, 76, 138, 108, 127, 148, 182, 111, 104, 225, 58, 129, 205, 156, 80, 118, 110, 160, 1,
    127, 135, 68, 171, 236, 217, 252, 46, 248, 205, 119, 131,
];
const PREPARE_LOGIC_HASH: [u8; 32] = [
    67, 78, 214, 35, 141, 175, 103, 224, 23, 144, 140, 58, 237, 221, 126, 20, 185, 154, 185, 71,
    96, 39, 131, 225, 17, 76, 168, 77, 167, 234, 8, 131,
];
const END_LOGIC_HASH: [u8; 32] = [
    179, 87, 145, 107, 121, 213, 84, 251, 60, 113, 203, 232, 177, 247, 173, 173, 187, 152, 252,
    151, 223, 98, 215, 32, 184, 29, 214, 229, 62, 249, 232, 90,
];

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: engine_api::SkillId,
}

fn setup(seed: u64, phase: Phase) -> Setup {
    let package = content_compiler::compile_source_named("decadezhenjun-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
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
        skill_def: SkillDefId::new(DECADEZHENJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in players {
        let participant = tx.state_mut().players.get_mut(player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = phase;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == owner && instance.def.raw() == DECADEZHENJUN)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        skill,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: u8::from(kind == CardKind::Weapon) * 2,
        zone: hand,
        owner,
        face_up: false,
    });
    state.zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let command = Command {
        actor,
        expected_revision: ExpectedRevision(state.revision),
        idempotency_key: IdempotencyKey(key),
        kind: CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled,
        },
    };
    apply_command(state, command)
}

fn open_effect(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDecadezhenjun {
                player: setup.owner,
                target: PlayerId::NONE,
                stage: 0,
                count: 0,
                skill: setup.skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw
    );
    assert_eq!(state.resolution.continuation_payload[0], 3);
    state
}

fn select_target(state: GameState, setup: &Setup, key: u64) -> GameState {
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        result.choice.as_ref().map(|choice| choice.kind),
        Some(ChoiceKind::SelectCards)
    );
    assert_eq!(state.resolution.continuation_payload[0], 4);
    state
}

fn fire_phase(mut setup: Setup, phase: TurnPhaseTag, key: u64) -> GameState {
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: setup.owner,
                phase,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    setup.state = tx.commit().state;
    let (state, result) = resolve(setup.state, setup.owner, vec![1], false, key);
    assert!(result
        .choice
        .as_ref()
        .is_some_and(|choice| choice.kind == ChoiceKind::SelectTargets));
    state
}

#[test]
fn package_publishes_complete_character_and_freezes_legacy_rezhenjun_contract() {
    let compiled = content_compiler::compile_source_named("decadezhenjun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 298);
    assert_eq!(compiled.trigger_index.len(), 2);
    assert!(compiled.trigger_index.iter().all(|trigger| {
        trigger.optional
            && trigger.filter == "owner_is_target_any_alive_discardable_by_owner_hand_equip"
    }));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_prepare"));
    assert!(compiled
        .trigger_index
        .iter()
        .any(|trigger| trigger.event == "phase_end"));
    let prepare = compiled
        .modules
        .iter()
        .find(|module| module.name == "decadezhenjun_phase_prepare")
        .unwrap();
    assert_eq!(prepare.code, [83, 90, 29, 1, 1]);
    assert_eq!(prepare.logic_hash, PREPARE_LOGIC_HASH);
    let end = compiled
        .modules
        .iter()
        .find(|module| module.name == "decadezhenjun_phase_end")
        .unwrap();
    assert_eq!(end.code, [83, 90, 29, 1, 1]);
    assert_eq!(end.logic_hash, END_LOGIC_HASH);

    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 298);
    assert!(package
        .manifest
        .dependencies
        .iter()
        .any(|dependency| dependency.package_id == "yijiang"));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YUJIN_YUJIN)
        .unwrap();
    assert_eq!(character.name_key, "yujin_yujin");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [DECADEZHENJUN]);
    assert!(compiled.modules.iter().all(|expected| {
        package
            .modules
            .iter()
            .any(|actual| actual.name == expected.name && actual.logic_hash == expected.logic_hash)
    }));

    let legacy =
        content_compiler::compile_source_named("legacy-rezhenjun", LEGACY_REZHENJUN_SOURCE)
            .unwrap();
    assert_eq!(legacy.manifest.rule_vm_abi_version, 62);
    assert_eq!(legacy.modules.len(), 1);
    assert_eq!(legacy.modules[0].code, [83, 80, 91, 1]);
    assert_eq!(legacy.modules[0].logic_hash, LEGACY_REZHENJUN_LOGIC_HASH);
}

#[test]
fn prepare_and_end_filters_require_a_legal_target_and_include_self() {
    let empty = setup(2_026_073_001, Phase::Prepare);
    let mut tx = Transaction::begin(empty.state.clone());
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: empty.owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());

    let mut prepare = empty;
    inject_hand(&mut prepare.state, prepare.owner, 99_801, CardKind::Other);
    let owner = prepare.owner;
    let state = fire_phase(prepare, TurnPhaseTag::Prepare, 1);
    assert_eq!(state.resolution.continuation_payload[0], 3);
    assert!(state
        .resolution
        .continuation_payload
        .get(4..)
        .is_some_and(|targets| targets.contains(&owner.raw())));

    let mut end = setup(2_026_073_002, Phase::End);
    inject_hand(&mut end.state, end.owner, 99_802, CardKind::Other);
    let state = fire_phase(end, TurnPhaseTag::End, 2);
    assert_eq!(state.resolution.continuation_payload[0], 3);
}

#[test]
fn dynamic_target_count_and_any_equipment_finish_without_owner_branch() {
    let mut setup = setup(2_026_073_003, Phase::Prepare);
    setup.state.players.get_mut(setup.target).unwrap().hp = 1;
    let ordinary = inject_hand(&mut setup.state, setup.target, 99_811, CardKind::Other);
    let equipment = inject_hand(&mut setup.state, setup.target, 99_812, CardKind::Weapon);
    let remaining = inject_hand(&mut setup.state, setup.target, 99_813, CardKind::Other);
    let state = select_target(open_effect(&setup), &setup, 10);
    assert_eq!(state.resolution.continuation_payload[4], 2);
    let (state, result) = resolve(
        state,
        setup.owner,
        vec![ordinary.raw(), equipment.raw()],
        false,
        11,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_cards(setup.target), [remaining]);
}

#[test]
fn all_non_equipment_requires_one_owner_card_while_cancel_or_inability_draws_n() {
    let mut paid = setup(2_026_073_004, Phase::End);
    paid.state.players.get_mut(paid.target).unwrap().hp = 1;
    let first = inject_hand(&mut paid.state, paid.target, 99_821, CardKind::Other);
    let second = inject_hand(&mut paid.state, paid.target, 99_822, CardKind::Other);
    inject_hand(&mut paid.state, paid.target, 99_823, CardKind::Other);
    let cost = inject_hand(&mut paid.state, paid.owner, 99_824, CardKind::Other);
    let state = select_target(open_effect(&paid), &paid, 20);
    let (state, result) = resolve(
        state,
        paid.owner,
        vec![first.raw(), second.raw()],
        false,
        21,
    );
    assert_eq!(
        result.choice.as_ref().map(|choice| (
            choice.kind,
            choice.min_count,
            choice.max_count,
            choice.allow_cancel
        )),
        Some((ChoiceKind::SelectCards, 1, 1, true))
    );
    assert_eq!(state.resolution.continuation_payload[0], 5);
    assert_eq!(state.resolution.continuation_payload[4], 2);
    let (state, result) = resolve(state, paid.owner, vec![cost.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(paid.owner), 0);
    assert_eq!(state.hand_count(paid.target), 1);

    let mut cancelled = setup(2_026_073_005, Phase::Prepare);
    cancelled
        .state
        .players
        .get_mut(cancelled.target)
        .unwrap()
        .hp = 1;
    let first = inject_hand(
        &mut cancelled.state,
        cancelled.target,
        99_831,
        CardKind::Other,
    );
    let second = inject_hand(
        &mut cancelled.state,
        cancelled.target,
        99_832,
        CardKind::Other,
    );
    inject_hand(
        &mut cancelled.state,
        cancelled.target,
        99_833,
        CardKind::Other,
    );
    inject_hand(
        &mut cancelled.state,
        cancelled.owner,
        99_834,
        CardKind::Other,
    );
    let state = select_target(open_effect(&cancelled), &cancelled, 30);
    let (state, _) = resolve(
        state,
        cancelled.owner,
        vec![first.raw(), second.raw()],
        false,
        31,
    );
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, cancelled.owner, vec![9_999_999], false, 32);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (state, result) = resolve(state, cancelled.owner, Vec::new(), true, 33);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(cancelled.target), 3);

    let mut unable = setup(2_026_073_006, Phase::Prepare);
    unable.state.players.get_mut(unable.target).unwrap().hp = 1;
    let first = inject_hand(&mut unable.state, unable.target, 99_841, CardKind::Other);
    let second = inject_hand(&mut unable.state, unable.target, 99_842, CardKind::Other);
    inject_hand(&mut unable.state, unable.target, 99_843, CardKind::Other);
    let state = select_target(open_effect(&unable), &unable, 40);
    let (state, result) = resolve(
        state,
        unable.owner,
        vec![first.raw(), second.raw()],
        false,
        41,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(unable.target), 3);
}
