//! Golden semantics for EXTRA 神诸葛亮「七星」「狂风」「大雾」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const QIXING: u32 = 2_771_819_420;
const KUANGFENG: u32 = 4_216_995_242;
const DAWU: u32 = 2_545_944_631;
const KUANGFENG2: u32 = 4_154_289_782;
const DAWU2: u32 = 3_348_547_572;
const SHEN_ZHUGELIANG: u32 = 4_215_109_649;
const EXTRA: &[u8] = include_bytes!("../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../content/standard/standard.npack.json");

fn setup() -> (GameState, Vec<PlayerId>, [SkillId; 3]) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(QIXING as u64),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: QIXING as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let players = engine.state().players.alive_by_seat();
    let owner = players[0];
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::SetTurnState {
                current: owner,
                phase: Phase::End.to_u8(),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(QIXING),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(KUANGFENG),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DAWU),
            },
        ],
    )
    .unwrap();
    let skill = |def| {
        tx.state()
            .players
            .get(owner)
            .unwrap()
            .skills
            .iter()
            .copied()
            .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == def)
            .unwrap()
    };
    let skills = [skill(QIXING), skill(KUANGFENG), skill(DAWU)];
    (tx.state().clone(), players, skills)
}

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    let revision = state.revision;
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
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

fn initialize_qixing(state: GameState, owner: PlayerId, skill: SkillId) -> GameState {
    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveQixing {
                player: owner,
                initialize: true,
                stage: 0,
                hand_cards: Vec::new(),
                pile_cards: Vec::new(),
                skill,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    tx.state().clone()
}

fn helper_skill(state: &GameState, target: PlayerId, def: u32) -> Option<SkillId> {
    state
        .players
        .get(target)?
        .skills
        .iter()
        .copied()
        .find(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == def)
        })
}

#[test]
fn package_closes_shen_zhugeliang_on_current_extra_abi() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_ZHUGELIANG)
        .unwrap();
    assert_eq!(character.skills, [QIXING, KUANGFENG, DAWU]);
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == DAWU2));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == KUANGFENG2));
}

#[test]
fn qixing_initializes_seven_public_stars_and_exchanges_equal_counts() {
    let (state, players, skills) = setup();
    let owner = players[0];
    let hand_before = state.hand_cards(owner);
    let state = initialize_qixing(state, owner, skills[0]);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseQixingHandCards
    );
    let stars = state.skill_pile_cards(owner, QIXING);
    assert_eq!(stars.len(), 7);
    assert!(stars
        .iter()
        .all(|card| state.cards.get(*card).unwrap().face_up));

    let (state, pile_choice) = resolve(state, owner, vec![hand_before[0].raw()], false, 1);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseQixingPileCards
    );
    assert_eq!(pile_choice.choice.unwrap().min_count, 1);
    let (state, result) = resolve(state, owner, vec![stars[0].raw()], false, 2);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .skill_pile_cards(owner, QIXING)
        .contains(&hand_before[0]));
    assert!(state.hand_cards(owner).contains(&stars[0]));
    assert_eq!(state.skill_pile_cards(owner, QIXING).len(), 7);
}

#[test]
fn kuangfeng_adds_fire_damage_and_dawu_only_allows_thunder_damage() {
    let (state, players, skills) = setup();
    let owner = players[0];
    let target = players[1];
    let state = initialize_qixing(state, owner, skills[0]);
    let (state, _) = resolve(state, owner, Vec::new(), true, 3);
    let stars = state.skill_pile_cards(owner, QIXING);

    let mut tx = Transaction::begin(state);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveQixingWeather {
                player: owner,
                qixing_skill_def: SkillDefId::new(QIXING),
                helper_skill_def: SkillDefId::new(KUANGFENG2),
                max_targets: 1,
                stage: 0,
                cards: Vec::new(),
                targets: Vec::new(),
                skill: skills[1],
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = tx.state().clone();
    let (state, _) = resolve(state, owner, vec![stars[0].raw()], false, 4);
    let (state, result) = resolve(state, owner, vec![target.raw()], false, 5);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let before = state.players.get(target).unwrap().hp;
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::FIRE,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 2);

    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveQixingWeather {
            player: owner,
            qixing_skill_def: SkillDefId::new(QIXING),
            helper_skill_def: SkillDefId::new(DAWU2),
            max_targets: u8::MAX,
            stage: 1,
            cards: vec![stars[1]],
            targets: vec![target],
            skill: skills[2],
        }],
    )
    .unwrap();
    let before = tx.state().players.get(target).unwrap().hp;
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::THUNDER,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().players.get(target).unwrap().hp, before - 1);
}

#[test]
fn weather_sources_expire_independently_at_each_sources_next_turn() {
    let (state, players, _skills) = setup();
    let first = players[0];
    let second = players[1];
    let target = players[2];
    let mut tx = Transaction::begin(state);
    for source in [first, second] {
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveQixingWeather {
                player: source,
                qixing_skill_def: SkillDefId::new(QIXING),
                helper_skill_def: SkillDefId::new(KUANGFENG2),
                max_targets: 1,
                stage: 1,
                cards: vec![engine_api::CardId::new(source.raw())],
                targets: vec![target],
                skill: SkillId::NONE,
            }],
        )
        .unwrap();
    }
    let helper = helper_skill(tx.state(), target, KUANGFENG2).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: players[2],
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let (state, result) = submit(state, PlayerId::NONE, CommandKind::EndPhase, 6);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.skills.get(helper).is_some());

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::SetTurnState {
            current: players[0],
            phase: Phase::End.to_u8(),
        }],
    )
    .unwrap();
    let (state, result) = submit(tx.state().clone(), PlayerId::NONE, CommandKind::EndPhase, 7);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state.skills.get(helper).is_none());
}
