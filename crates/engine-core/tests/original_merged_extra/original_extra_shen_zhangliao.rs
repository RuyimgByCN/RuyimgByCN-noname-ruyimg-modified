//! Golden semantics for EXTRA 神张辽「夺锐」「止啼」。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::query::{query, QueryContext};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

const DUORUI: u32 = 4_266_461_274;
const DUORUI_HELPER: u32 = 4_193_490_776;
const ZHITI: u32 = 3_298_844_877;
const POXI: u32 = 3_008_932_273;
const SHEN_ZHANGLIAO: u32 = 3_439_287_180;
const SHEN_GANNING: u32 = 3_481_630_433;
const EXTRA: &[u8] = include_bytes!("../../../../content/original/ported/extra/extra.npack.json");
const STANDARD: &[u8] = include_bytes!("../../../../content/standard/standard.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId) {
    let standard = content_schema::ContentPackage::from_json_bytes(STANDARD).unwrap();
    let extra = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&standard).unwrap();
    engine.load_package(&extra).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(DUORUI as u64),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: DUORUI as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let mut key = 1;
    while engine.state().has_pending_choice() {
        let choice = engine.last_choice().unwrap().clone();
        assert!(matches!(
            engine.submit(Command {
                actor: choice.observer,
                expected_revision: ExpectedRevision(engine.state().revision),
                idempotency_key: IdempotencyKey(key),
                kind: CommandKind::ResolveChoice {
                    choice_id: choice.choice_id,
                    selections: Vec::new(),
                    cancelled: true,
                },
            }),
            CommandResult::Accepted { .. }
        ));
        key += 1;
    }
    let players = engine.state().players.alive_by_seat();
    let mut tx = Transaction::begin(engine.state().clone());
    let old = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    run_effects(&mut tx, old).unwrap();
    tx.state_mut()
        .players
        .get_mut(players[0])
        .unwrap()
        .character_def = SHEN_ZHANGLIAO;
    tx.state_mut()
        .players
        .get_mut(players[1])
        .unwrap()
        .character_def = SHEN_GANNING;
    (tx.state().clone(), players[0], players[1])
}

fn skill(state: &GameState, owner: PlayerId, def: u32) -> SkillId {
    state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| state.skills.get(*skill).unwrap().def.raw() == def)
        .unwrap()
}

fn resolve_choice(state: GameState, actor: PlayerId, selection: u32, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_closes_shen_zhangliao_and_marks_helper_non_gainable() {
    let package = content_schema::ContentPackage::from_json_bytes(EXTRA).unwrap();
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SHEN_ZHANGLIAO)
        .unwrap();
    assert_eq!(
        (character.max_hp, character.skills.as_slice()),
        (4, [DUORUI, ZHITI].as_slice())
    );
    content_runtime::check_engine_compat(
        &package.manifest,
        &content_runtime::tooling_host_versions(),
    )
    .unwrap();
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == DUORUI_HELPER && skill.non_gainable));
}

#[test]
fn duorui_disables_slot_borrows_skill_and_cleans_up_by_reason() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(DUORUI),
            },
            EffectRequest::AddSkill {
                player: target,
                skill_def: SkillDefId::new(POXI),
            },
        ],
    )
    .unwrap();
    let duorui = skill(tx.state(), owner, DUORUI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltDuorui {
                player: owner,
                target,
                helper_skill_def: SkillDefId::new(DUORUI_HELPER),
                stage: 0,
                selected_skill_def: SkillDefId::NONE,
                skill: duorui,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::ChooseDrltDuoruiDisableSlot
    );
    let state = resolve_choice(tx.state().clone(), owner, 1, 20);
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDrltDuoruiSkill
    );
    let state = resolve_choice(state, owner, POXI, 21);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 1);
    let target_skill = skill(&state, target, POXI);
    assert!(!state.skill_effectively_enabled(state.skills.get(target_skill).unwrap()));
    let borrowed = skill(&state, owner, POXI);
    let helper = skill(&state, target, DUORUI_HELPER);

    let mut tx = Transaction::begin(state);
    tx.state_mut()
        .skills
        .get_mut(target_skill)
        .unwrap()
        .disabled_by_skills
        .push(SkillId::new(999));
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveDrltDuorui {
            player: target,
            target,
            helper_skill_def: SkillDefId::new(DUORUI_HELPER),
            stage: 3,
            selected_skill_def: SkillDefId::NONE,
            skill: helper,
        }],
    )
    .unwrap();
    assert!(tx.state().skills.get(borrowed).is_none());
    assert!(!tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(target_skill).unwrap()));
    assert_eq!(
        tx.state()
            .skills
            .get(target_skill)
            .unwrap()
            .disabled_by_skills,
        [SkillId::new(999)]
    );
    assert!(tx
        .state_mut()
        .skills
        .remove_disable_reason(SkillId::new(999)));
    tx.state_mut().rebuild_derived_indexes();
    assert!(tx
        .state()
        .skill_effectively_enabled(tx.state().skills.get(target_skill).unwrap()));
    assert!(tx.state().skills.get(helper).is_none());
}

#[test]
fn zhiti_reduces_wounded_in_range_hand_limit_and_restores_a_slot() {
    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(target).unwrap().hp = 3;
    tx.state_mut().players.get_mut(target).unwrap().max_hp = 4;
    tx.state_mut()
        .players
        .get_mut(owner)
        .unwrap()
        .disabled_equip_slots[0] = 1;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZHITI),
        }],
    )
    .unwrap();
    assert_eq!(
        query(
            tx.state(),
            &QueryRequest::HandLimit { player: target },
            &mut QueryContext::new(),
        ),
        QueryResponse::Int(2)
    );
    let zhiti = skill(tx.state(), owner, ZHITI);
    assert!(matches!(
        run_effects(
            &mut tx,
            vec![EffectRequest::ResolveDrltZhiti {
                player: owner,
                event_player: owner,
                event_source: target,
                event_amount: 1,
                mode: 2,
                skill: zhiti,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    ));
    let state = resolve_choice(tx.state().clone(), owner, 1, 30);
    assert_eq!(state.players.get(owner).unwrap().disabled_equip_slots[0], 0);
}
