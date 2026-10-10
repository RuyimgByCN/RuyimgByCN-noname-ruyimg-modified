//! Golden coverage for MOBILE 十常侍组件「谣诼」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SCS_XIAYUN: u32 = 4_066_474_407;
const SCSYAOZHUO: u32 = 2_736_759_716;
const SCSYAOZHUO_WIN: u32 = 3_825_030_276;
const SCSYAOZHUO_LOSE: u32 = 3_028_413_936;
const SCSYAOZHUO_SKIP: u32 = 4_154_258_460;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/scsyaozhuo.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_compare: CardId,
    target_compare: CardId,
}

fn inject(state: &mut GameState, owner: PlayerId, def: u32, rank: u8) -> CardId {
    let zone = state.zones.find(ZoneKind::Hand, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Other,
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

fn setup(owner_rank: u8, target_rank: u8) -> Setup {
    let package = content_compiler::compile_source_named("scsyaozhuo-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 2_026_073_001, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let (owner, target) = (seats[0], seats[1]);
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
            player: owner,
            skill_def: SkillDefId::new(SCSYAOZHUO),
        },
        EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Play.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    let owner_compare = inject(tx.state_mut(), owner, 99_501, owner_rank);
    let target_compare = inject(tx.state_mut(), target, 99_502, target_rank);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        owner_compare,
        target_compare,
    }
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
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
}

fn command(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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
    command(
        state.clone(),
        actor,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(state.resolution.pending_choice_id),
            selections,
            cancelled: false,
        },
        key,
    )
}

fn compare(setup: &Setup) -> (GameState, PipelineResult) {
    let active = skill_id(&setup.state, setup.owner, SCSYAOZHUO).unwrap();
    let (state, activated) = command(
        setup.state.clone(),
        setup.owner,
        CommandKind::ActivateSkill {
            skill: active,
            targets: vec![setup.target],
            card_targets: Vec::new(),
        },
        1,
    );
    assert!(matches!(activated.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ActiveSkillSelection
    );
    let (state, owner_selected) = resolve(state, setup.owner, vec![setup.owner_compare.raw()], 2);
    assert!(matches!(
        owner_selected.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseCompareTargetHandCard
    );
    resolve(state, setup.target, vec![setup.target_compare.raw()], 3)
}

#[test]
fn package_publishes_four_rule_modules_without_exposing_hidden_xiayun() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for skill_def_id in [SCSYAOZHUO, SCSYAOZHUO_WIN, SCSYAOZHUO_LOSE, SCSYAOZHUO_SKIP] {
        assert!(package
            .skill_index
            .iter()
            .any(|skill| skill.skill_def_id == skill_def_id));
    }
    // 十常侍 roster: scs_xiayun publishes with 妖灼.
    assert!(package
        .character_index
        .iter()
        .any(|character| character.id == SCS_XIAYUN
            && character.skills.contains(&SCSYAOZHUO)));
}

#[test]
fn strict_win_skips_the_targets_next_draw_phase_once() {
    let setup = setup(13, 1);
    let (state, compared) = compare(&setup);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(skill_id(&state, setup.owner, SCSYAOZHUO_WIN).is_none());
    assert!(skill_id(&state, setup.target, SCSYAOZHUO_SKIP).is_some());

    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = setup.target;
    tx.state_mut().turn.phase = Phase::Draw;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.target,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let events = run_effects(&mut tx, effects).unwrap();
    assert!(events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Draw,
        } if *current == setup.target
    )));
    assert!(skill_id(tx.state(), setup.target, SCSYAOZHUO_SKIP).is_none());
}

#[test]
fn tie_discards_every_remaining_card_when_fewer_than_two_exist() {
    let mut setup = setup(7, 7);
    let spare = inject(&mut setup.state, setup.owner, 99_503, 4);
    let (state, compared) = compare(&setup);
    assert!(matches!(compared.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(skill_id(&state, setup.owner, SCSYAOZHUO_LOSE).is_none());
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(state.cards.get(spare).unwrap().zone, discard);
    assert_eq!(state.hand_count(setup.owner), 0);
}
