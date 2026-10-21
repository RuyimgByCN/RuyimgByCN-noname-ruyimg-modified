//! Golden coverage for 于禁「镇军」rewritten from legacy JavaScript semantics.

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

const YUJIN: u32 = 2_365_095_387;
const REZHENJUN: u32 = 4_131_625_144;
const SOURCE: &str = include_str!("../../../../content/original/ported/yijiang/rezhenjun.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
    skill: engine_api::SkillId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
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
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(REZHENJUN),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = YUJIN;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Prepare;
    tx.state_mut().revision = 1;
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|instance| instance.owner == owner && instance.def.raw() == REZHENJUN)
        .unwrap()
        .id;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
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

fn open_effect(setup: &Setup) -> GameState {
    let mut tx = Transaction::begin(setup.state.clone());
    assert_eq!(
        run_effects(
            &mut tx,
            vec![
                EffectRequest::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw {
                    player: setup.owner,
                    skill: setup.skill,
                },
            ],
        ),
        Err(EffectError::ChoiceRequested)
    );
    tx.commit().state
}

fn select_target(state: GameState, setup: &Setup, key: u64) -> GameState {
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, key);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    state
}

#[test]
fn package_publishes_complete_yujin_and_exact_abi62_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 637);
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YUJIN)
        .unwrap();
    assert_eq!(character.name_key, "yujin");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wei");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [REZHENJUN]);
    let compiled = content_compiler::compile_source_named("rezhenjun", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 62);
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
fn prepare_filter_requires_a_hand_card_and_target_choice_includes_self() {
    let mut empty = setup(2_026_074_301);
    let effects = dispatch_for_events(
        &mut Transaction::begin(empty.state.clone()),
        &[DomainEvent::TurnStateChanged {
            current: empty.owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty());

    inject_hand(&mut empty.state, empty.owner, 99_301, CardKind::Other);
    let mut tx = Transaction::begin(empty.state);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::TurnStateChanged {
                current: empty.owner,
                phase: TurnPhaseTag::Prepare,
            }],
            &mut TriggerFireRecord::new(),
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (state, result) = resolve(state, empty.owner, vec![1], false, 1);
    assert!(result
        .choice
        .as_ref()
        .unwrap()
        .options
        .iter()
        .any(|option| option.token == empty.owner.raw()));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDiscardExcessThenSelfDiscardOrTargetDraw
    );
}

#[test]
fn discarding_only_equipment_finishes_without_a_second_branch() {
    let mut setup = setup(2_026_074_302);
    let equipment = inject_hand(&mut setup.state, setup.target, 99_311, CardKind::Weapon);
    setup.state.players.get_mut(setup.target).unwrap().hp = 4;
    let state = select_target(open_effect(&setup), &setup, 10);
    let (state, result) = resolve(state, setup.owner, vec![equipment.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(setup.target), 0);
}

#[test]
fn non_equipment_count_becomes_the_exact_optional_self_discard_cost() {
    let mut setup = setup(2_026_074_303);
    setup.state.players.get_mut(setup.target).unwrap().hp = 1;
    let ordinary = inject_hand(&mut setup.state, setup.target, 99_321, CardKind::Other);
    let equipment = inject_hand(&mut setup.state, setup.target, 99_322, CardKind::Armor);
    inject_hand(&mut setup.state, setup.target, 99_323, CardKind::Other);
    let cost = inject_hand(&mut setup.state, setup.owner, 99_324, CardKind::Other);
    let state = select_target(open_effect(&setup), &setup, 20);
    let (state, result) = resolve(
        state,
        setup.owner,
        vec![ordinary.raw(), equipment.raw()],
        false,
        21,
    );
    assert_eq!(
        result.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectCards
    );
    assert_eq!(state.resolution.continuation_payload[0], 2);
    assert_eq!(state.resolution.continuation_payload[4], 1);
    let target_before = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![cost.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), target_before);
    assert_eq!(state.hand_count(setup.owner), 0);
}

#[test]
fn cancel_or_insufficient_cost_draws_and_forged_cost_is_atomic() {
    let mut cancel = setup(2_026_074_304);
    cancel.state.players.get_mut(cancel.target).unwrap().hp = 4;
    let target_card = inject_hand(&mut cancel.state, cancel.target, 99_331, CardKind::Other);
    inject_hand(&mut cancel.state, cancel.owner, 99_332, CardKind::Other);
    let state = select_target(open_effect(&cancel), &cancel, 30);
    let (state, _) = resolve(state, cancel.owner, vec![target_card.raw()], false, 31);
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, cancel.owner, vec![9_999_999], false, 32);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let target_before = state.hand_count(cancel.target);
    let (state, result) = resolve(state, cancel.owner, Vec::new(), true, 33);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(cancel.target), target_before + 1);

    let mut insufficient = setup(2_026_074_305);
    insufficient
        .state
        .players
        .get_mut(insufficient.target)
        .unwrap()
        .hp = 1;
    let first = inject_hand(
        &mut insufficient.state,
        insufficient.target,
        99_341,
        CardKind::Other,
    );
    let second = inject_hand(
        &mut insufficient.state,
        insufficient.target,
        99_342,
        CardKind::Other,
    );
    inject_hand(
        &mut insufficient.state,
        insufficient.target,
        99_343,
        CardKind::Other,
    );
    inject_hand(
        &mut insufficient.state,
        insufficient.owner,
        99_344,
        CardKind::Other,
    );
    let state = select_target(open_effect(&insufficient), &insufficient, 40);
    let before = state.hand_count(insufficient.target);
    let (state, result) = resolve(
        state,
        insufficient.owner,
        vec![first.raw(), second.raw()],
        false,
        41,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert_eq!(state.hand_count(insufficient.target), before);
    assert_ne!(insufficient.other, insufficient.target);
}
