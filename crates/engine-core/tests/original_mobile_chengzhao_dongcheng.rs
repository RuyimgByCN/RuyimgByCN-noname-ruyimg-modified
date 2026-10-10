//! Golden coverage for MOBILE 董承「承诏」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, ContinuationKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const CHENGZHAO: u32 = 3_452_131_931;
const CHENGZHAO_WIN: u32 = 2_503_452_290;
const CHENGZHAO_LOSE: u32 = 4_247_003_544;
const DONGCHENG: u32 = 3_178_823_789;
const TENGJIA4: u32 = 4_041_276_943;
const SOURCE: &str = include_str!("../../../content/original/ported/mobile/chengzhao.skill");
const ARMOR: &str = include_str!("../../../content/original/ported/mobile/rw_tengjia4.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    owner_card: CardId,
    target_card: CardId,
}

fn skill_id(state: &GameState, owner: PlayerId, def: u32) -> Option<engine_api::SkillId> {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .find_map(|id| {
            state
                .skills
                .get(*id)
                .is_some_and(|skill| skill.def.raw() == def)
                .then_some(*id)
        })
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

fn setup(owner_rank: u8, target_rank: u8, gains: u32, seed: u64) -> Setup {
    let chengzhao = content_compiler::compile_source_named("chengzhao-test", SOURCE).unwrap();
    let armor = content_compiler::compile_source_named("chengzhao-armor-test", ARMOR).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&chengzhao).unwrap();
    std::sync::Arc::make_mut(&mut state.content).load_package(&armor).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let ending = players[1];
    let target = players[2];
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
            skill_def: SkillDefId::new(CHENGZHAO),
        },
        EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(TENGJIA4),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = ending;
    tx.state_mut().turn.phase = Phase::End;
    tx.state_mut()
        .turn
        .set_flag(turn_flag::card_gained_by_player(owner), gains as i32);
    for player in &players {
        let participant = tx.state_mut().players.get_mut(*player).unwrap();
        participant.hp = 4;
        participant.max_hp = 4;
    }
    let owner_card = inject(tx.state_mut(), owner, 99_401, owner_rank);
    let target_card = inject(tx.state_mut(), target, 99_402, target_rank);
    Setup {
        state: tx.commit().state,
        owner,
        target,
        owner_card,
        target_card,
    }
}

fn trigger_end_phase(state: GameState) -> GameState {
    let current = state.turn.current;
    let mut tx = Transaction::begin(state);
    match dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    ) {
        Ok(effects) => {
            let result = run_effects(&mut tx, effects);
            assert!(matches!(result, Ok(_) | Err(EffectError::ChoiceRequested)));
        }
        Err(error) => assert_eq!(error, EffectError::ChoiceRequested),
    }
    tx.commit().state
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice {key}: {:?}",
        result.result,
    );
    state
}

fn compare(setup: Setup, key: u64) -> GameState {
    let state = trigger_end_phase(setup.state);
    assert!(state.has_pending_choice());
    assert_eq!(state.resolution.pending_observer, setup.owner.raw());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let state = resolve(state, setup.owner, vec![1], key);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseHandCompareTarget
    );
    let state = resolve(state, setup.owner, vec![setup.target.raw()], key + 1);
    let state = resolve(state, setup.owner, vec![setup.owner_card.raw()], key + 2);
    resolve(state, setup.target, vec![setup.target_card.raw()], key + 3)
}

#[test]
fn package_publishes_dongcheng_and_all_chengzhao_rules() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 325);
    assert_eq!(
        package
            .character_index
            .iter()
            .find(|character| character.id == DONGCHENG)
            .unwrap()
            .skills,
        [CHENGZHAO]
    );
    for skill in [CHENGZHAO, CHENGZHAO_WIN, CHENGZHAO_LOSE] {
        assert!(package
            .skill_index
            .iter()
            .any(|entry| entry.skill_def_id == skill));
    }
}

#[test]
fn trigger_requires_two_gained_cards_and_both_comparison_hands() {
    let setup = setup(13, 1, 1, 2_026_073_001);
    let state = trigger_end_phase(setup.state);
    assert!(!state.has_pending_choice());
}

#[test]
fn strict_win_uses_distance_unlimited_armor_ignoring_sha_and_cleans_helpers() {
    let setup = setup(13, 1, 2, 2_026_073_002);
    let owner = setup.owner;
    let target = setup.target;
    let state = compare(setup, 10);
    assert_eq!(state.players.get(target).unwrap().hp, 3);
    assert!(skill_id(&state, owner, CHENGZHAO_WIN).is_none());
    assert!(skill_id(&state, owner, CHENGZHAO_LOSE).is_none());
    assert!(!state.has_pending_choice());
}

#[test]
fn tie_or_loss_does_not_use_sha_and_cleans_helpers() {
    for (owner_rank, target_rank, key) in [(1, 13, 20), (7, 7, 30)] {
        let setup = setup(owner_rank, target_rank, 2, 2_026_073_000 + key);
        let owner = setup.owner;
        let target = setup.target;
        let state = compare(setup, key);
        assert_eq!(state.players.get(target).unwrap().hp, 4);
        assert!(skill_id(&state, owner, CHENGZHAO_WIN).is_none());
        assert!(skill_id(&state, owner, CHENGZHAO_LOSE).is_none());
        assert!(!state.has_pending_choice());
    }
}
