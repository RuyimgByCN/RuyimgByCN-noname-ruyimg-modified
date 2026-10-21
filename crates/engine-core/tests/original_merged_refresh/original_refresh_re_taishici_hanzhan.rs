//! Golden coverage for 界太史慈「酣战」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const RE_TAISHICI: u32 = 4_231_532_233;
const TIANYI: u32 = 3_117_995_535;
const HANZHAN: u32 = 4_071_802_825;
const HANZHAN_GAIN: u32 = 3_019_387_228;
const REFRESH: &[u8] =
    include_bytes!("../../../../content/original/ported/refresh/refresh.npack.json");
const SHENHUA: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    initiator: PlayerId,
    target: PlayerId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, kind: CardKind, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit: Suit::Spade,
        rank,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup(seed: u64, hanzhan_owner_is_initiator: bool) -> Setup {
    let shenhua = content_schema::ContentPackage::from_json_bytes(SHENHUA).unwrap();
    let refresh = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&shenhua)
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&refresh)
        .unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (initiator, target) = (seats[0], seats[1]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.extend([
        EffectRequest::AddSkill {
            player: initiator,
            skill_def: SkillDefId::new(TIANYI),
        },
        EffectRequest::AddSkill {
            player: if hanzhan_owner_is_initiator {
                initiator
            } else {
                target
            },
            skill_def: SkillDefId::new(HANZHAN),
        },
        EffectRequest::SetTurnState {
            current: initiator,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        initiator,
        target,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> engine_api::SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
                .then_some(*skill)
        })
        .unwrap()
}

fn command(
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
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    command(
        state,
        actor,
        CommandKind::ResolveChoice {
            choice_id,
            selections,
            cancelled: false,
        },
        key,
    )
}

fn activate(state: GameState, initiator: PlayerId, target: PlayerId, card: CardId) -> GameState {
    let tianyi = skill_id(&state, initiator, TIANYI);
    let (state, result) = command(
        state,
        initiator,
        CommandKind::ActivateSkill {
            skill: tianyi,
            targets: vec![target],
            card_targets: vec![card],
        },
        1,
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_publishes_complete_re_taishici_dependency_and_hooks() {
    let package = content_schema::ContentPackage::from_json_bytes(REFRESH).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 251);
    assert!(package.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "shenhua"
            && dependency.version == "0.1.0"
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == RE_TAISHICI)
        .unwrap();
    assert_eq!(character.skills, [TIANYI, HANZHAN]);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| { skill.skill_def_id == HANZHAN_GAIN && skill.name == "hanzhan_gain" }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HANZHAN
            && trigger.event == "hand_compare_started"
            && trigger.filter == "owner_hand_compare_participant"
            && trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == HANZHAN
            && trigger.event == "hand_compare_finished"
            && trigger.filter == "owner_hand_compare_participant_has_sha"
            && trigger.optional
    }));
}

#[test]
fn initiator_can_randomize_target_without_opening_target_card_choice() {
    let mut setup = setup(2_026_072_201, true);
    let initiator_card = inject(
        &mut setup.state,
        setup.initiator,
        98_001,
        CardKind::Other,
        9,
    );
    let target_a = inject(&mut setup.state, setup.target, 98_002, CardKind::Other, 2);
    let target_b = inject(&mut setup.state, setup.target, 98_003, CardKind::Other, 3);
    let state = activate(setup.state, setup.initiator, setup.target, initiator_card);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, setup.initiator, vec![1], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(
        !state.has_pending_choice(),
        "unexpected pending choice: {:?} {:?}",
        state.resolution.continuation_kind,
        accepted.choice
    );
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(initiator_card).unwrap().zone, discard);
    assert_eq!(
        [target_a, target_b]
            .into_iter()
            .filter(|card| state.cards.get(*card).unwrap().zone == discard)
            .count(),
        1
    );
}

#[test]
fn target_can_randomize_initiator_then_choose_own_card() {
    let mut setup = setup(2_026_072_202, false);
    let initiator_a = inject(
        &mut setup.state,
        setup.initiator,
        98_010,
        CardKind::Other,
        4,
    );
    let initiator_b = inject(
        &mut setup.state,
        setup.initiator,
        98_011,
        CardKind::Other,
        5,
    );
    let target_card = inject(&mut setup.state, setup.target, 98_012, CardKind::Other, 6);
    let state = activate(setup.state, setup.initiator, setup.target, initiator_a);
    let (state, accepted) = resolve(state, setup.target, vec![1], 2);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let randomized = CardId::new(state.resolution.continuation_payload[2]);
    assert!([initiator_a, initiator_b].contains(&randomized));
    let (state, compared) = resolve(state, setup.target, vec![target_card.raw()], 3);
    assert!(
        matches!(compared.result, CommandResult::Accepted { .. }),
        "comparison failed: {:?}",
        compared.result
    );
    assert!(!state.has_pending_choice());
}

#[test]
fn declining_randomization_then_accepting_gain_collects_all_tied_highest_sha() {
    let mut setup = setup(2_026_072_203, true);
    let initiator_sha = inject(&mut setup.state, setup.initiator, 98_020, CardKind::Sha, 11);
    let target_sha = inject(&mut setup.state, setup.target, 98_021, CardKind::Sha, 11);
    let state = activate(setup.state, setup.initiator, setup.target, initiator_sha);
    let (state, declined) = resolve(state, setup.initiator, vec![0], 2);
    assert!(matches!(declined.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    let (state, compared) = resolve(state, setup.target, vec![target_sha.raw()], 3);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, gained) = resolve(state, setup.initiator, vec![1], 4);
    assert!(matches!(gained.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    let hand = state.zones.find(ZoneKind::Hand, setup.initiator).unwrap();
    for card in [initiator_sha, target_sha] {
        let card = state.cards.get(card).unwrap();
        assert_eq!(card.owner, setup.initiator);
        assert_eq!(card.zone, hand);
    }
}
