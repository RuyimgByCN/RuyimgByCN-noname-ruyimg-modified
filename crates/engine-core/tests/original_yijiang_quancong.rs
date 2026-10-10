//! Golden coverage for 全琮「邀名」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::canonical_state_hash;
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const QUANCONG: u32 = 3_313_708_633;
const YAOMING: u32 = 2_608_199_273;
const YAOMING_SOURCE: &str = include_str!("../../../content/original/ported/yijiang/yaoming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    other: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
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
        skill_def: SkillDefId::new(YAOMING),
    });
    run_effects(&mut tx, effects).unwrap();
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 10;
        participant.max_hp = 10;
    }
    tx.state_mut().players.get_mut(owner).unwrap().character_def = QUANCONG;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
        target,
        other,
    }
}

fn inject_hand(state: &mut GameState, owner: PlayerId, def: u32) -> CardId {
    let hand = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
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

fn deal_damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    );
    (tx.commit().state, result)
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

fn open_yaoming(state: GameState, owner: PlayerId, target: PlayerId) -> GameState {
    let (state, result) = deal_damage(state, owner, target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    state
}

#[test]
fn package_publishes_complete_quancong_and_exact_abi60_module() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == QUANCONG)
        .unwrap();
    assert_eq!(character.name_key, "quancong");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "wu");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [YAOMING]);
    let compiled = content_compiler::compile_source_named("yaoming", YAOMING_SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 60);
    assert!(package.manifest.rule_vm_abi_version >= compiled.manifest.rule_vm_abi_version);
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
fn yaoming_requires_owner_damage_involvement_and_a_different_hand_count() {
    let equal = setup(2_026_074_001);
    let (state, result) = deal_damage(equal.state, equal.owner, equal.target);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut uninvolved = setup(2_026_074_002);
    inject_hand(&mut uninvolved.state, uninvolved.target, 99_001);
    let (state, result) = deal_damage(uninvolved.state, uninvolved.target, uninvolved.other);
    assert!(result.is_ok());
    assert!(!state.has_pending_choice());

    let mut source = setup(2_026_074_003);
    inject_hand(&mut source.state, source.target, 99_002);
    let (state, result) = deal_damage(source.state, source.owner, source.target);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    let mut damaged = setup(2_026_074_004);
    inject_hand(&mut damaged.state, damaged.target, 99_003);
    let (state, result) = deal_damage(damaged.state, damaged.target, damaged.owner);
    assert_eq!(result, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
}

#[test]
fn yaoming_lower_hand_target_draws_one() {
    let mut setup = setup(2_026_074_005);
    inject_hand(&mut setup.state, setup.owner, 99_011);
    inject_hand(&mut setup.state, setup.owner, 99_012);
    let state = open_yaoming(setup.state, setup.owner, setup.target);
    let (state, choice) = resolve(state, setup.owner, vec![1], false, 1);
    assert_eq!(
        choice.choice.as_ref().unwrap().kind,
        ChoiceKind::SelectTargets
    );
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseHandCountDifferentTargetDrawOrRandomDiscard
    );
    let before = state.hand_count(setup.target);
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), before + 1);
}

#[test]
fn yaoming_higher_hand_target_discards_one_with_authoritative_rng() {
    let mut setup = setup(2_026_074_006);
    let cards: Vec<_> = [99_021, 99_022, 99_023]
        .into_iter()
        .map(|def| inject_hand(&mut setup.state, setup.target, def))
        .collect();
    let state = open_yaoming(setup.state, setup.owner, setup.target);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 10);
    let calls = state.rng.call_count;
    let (state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, 11);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 2);
    assert_eq!(state.rng.call_count, calls + 1);
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(
        cards
            .iter()
            .filter(|card| state.cards.get(**card).unwrap().zone == discard)
            .count(),
        1
    );
}

#[test]
fn yaoming_rejects_forged_target_atomically_and_fires_once_per_turn() {
    let mut setup = setup(2_026_074_007);
    inject_hand(&mut setup.state, setup.owner, 99_031);
    inject_hand(&mut setup.state, setup.other, 99_032);
    let state = open_yaoming(setup.state, setup.owner, setup.target);
    let (state, _) = resolve(state, setup.owner, vec![1], false, 20);
    let before = canonical_state_hash(&state);
    let (state, forged) = resolve(state, setup.owner, vec![setup.other.raw()], false, 21);
    assert!(matches!(forged.result, CommandResult::Rejected { .. }));
    assert_eq!(canonical_state_hash(&state), before);
    let (mut state, result) = resolve(state, setup.owner, vec![setup.target.raw()], false, 22);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    inject_hand(&mut state, setup.target, 99_033);
    let (state, second) = deal_damage(state, setup.owner, setup.target);
    assert!(second.is_ok());
    assert!(!state.has_pending_choice());
}
