//! §7 技能夺取设施 S0（第一百四十三批，ABI 675）：potyinhui 音洄增益臂——
//! `choose_gain_other_skill` 两段选择链（选目标 → 选其可获得技能 → AddSkill）。

use engine_api::{
    ChoiceId, Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const POTYINHUI: u32 = 3_278_616_702;
const SOURCE: &str = include_str!("../../../content/original/ported/bingshi/potyinhui.skill");

fn started(source: &str, helper_defs: &[u32]) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("bingshi-potyinhui-test", source).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_843, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    // 第二个包提供「可被夺取」的技能 def（锁定技不可获得）。
    if !helper_defs.is_empty() {
        let helper = format!(
            "skill test_helper_a(3900000001) {{ trigger phase_end owner_is_target optional {{ effect draw_cards(self, 1); }} }}\nskill test_helper_locked(3900000002) locked non_gainable {{ modifier hand_limit add 0; }}\n"
        );
        let _ = helper_defs;
        let compiled2 =
            content_compiler::compile_source_named("bingshi-potyinhui-helpers", &helper).unwrap();
        std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled2).unwrap();
    }
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap()
        .raw();
    let clear: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, clear).unwrap();
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(POTYINHUI),
            },
            EffectRequest::AddSkill {
                player: PlayerId::new(other),
                skill_def: SkillDefId::new(3900000001),
            },
            EffectRequest::AddSkill {
                player: PlayerId::new(other),
                skill_def: SkillDefId::new(3900000002),
            },
        ],
    )
    .unwrap();
    (tx.commit().state, owner, PlayerId::new(other))
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let revision = state.revision;
    let choice_id = state.resolution.pending_choice_id;
    let (next, _pipeline) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(choice_id),
                selections,
                cancelled: false,
            },
        },
    );
    next
}

fn has_skill(state: &GameState, player: PlayerId, def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|s| s.owner == player && s.def.raw() == def)
}

#[test]
fn potyinhui_offers_then_gains_target_skill_via_staged_choice() {
    let source = SOURCE.to_string();
    let (state, owner, other) = started(&source, &[1]);
    // 驱动回合开始触发（optional confirm）。
    let mut tx = Transaction::begin(state);
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .map(|effects| run_effects(&mut tx, effects).unwrap_or_default());
    let asked = tx.state().has_pending_choice();
    let committed = tx.commit();
    assert!(asked, "optional potyinhui asks");
    // 段0 confirm（触发 optional）。
    let state = resolve(committed.state, owner, vec![1], 7_001);
    // 段1 选目标（options=其他存活者 raw；只有 other 一个候选）。
    let state = resolve(state, owner, vec![other.raw()], 7_002);
    // 段2 名单选技（只有 helper_a 可获得；locked non_gainable 已被过滤）。
    let state = resolve(state, owner, vec![0], 7_003);
    assert!(
        has_skill(&state, owner, 3900000001),
        "owner gained helper_a"
    );
    assert!(
        !has_skill(&state, owner, 3900000002),
        "locked non_gainable skill is not offered"
    );
}

#[test]
fn potyinhui_lose_arm_offers_and_removes_a_gained_skill() {
    let source = SOURCE.to_string();
    let (state, owner, other) = started(&source, &[1]);
    let mut tx = Transaction::begin(state);
    // 第一轮：confirm → 选目标 → 选技能 → 获得 helper_a。
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .map(|effects| run_effects(&mut tx, effects).unwrap_or_default());
    let committed = tx.commit();
    assert!(committed.state.has_pending_choice(), "first chain asks");
    let state = resolve(committed.state, owner, vec![1], 8_101);
    let state = resolve(state, owner, vec![other.raw()], 8_102);
    let state = resolve(state, owner, vec![0], 8_103);
    assert!(has_skill(&state, owner, 3_900_000_001), "gained helper_a");
    assert!(
        state
            .facility_gained_skills
            .get(&owner.raw())
            .is_some_and(|v| v.contains(&3_900_000_001)),
        "gain registered in facility list"
    );

    // 直接驱动 S1 失去臂：清单列出 helper_a；确认后 RemoveSkill+出清登记。
    let potyinhui_instance = state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def.raw() == POTYINHUI)
        .unwrap()
        .id;
    let mut tx = Transaction::begin(state);
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::LoseGainedSkill {
            player: owner,
            skill: potyinhui_instance,
        }],
    );
    assert!(tx.state().has_pending_choice(), "lose arm offers list");
    let committed = tx.commit();
    let state = resolve(committed.state, owner, vec![0], 8_104);
    assert!(
        !has_skill(&state, owner, 3_900_000_001),
        "gained skill removed"
    );
    assert!(
        state.facility_gained_skills.get(&owner.raw()).is_none(),
        "facility registration cleared"
    );
}

#[test]
fn potyinhui_once_per_round_lock_engages() {
    let source = SOURCE.to_string();
    let (state, owner, _other) = started(&source, &[1]);
    let mut tx = Transaction::begin(state);
    let _ = engine_core::trigger::dispatch_for_events(
        &mut tx,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )
    .map(|effects| run_effects(&mut tx, effects).unwrap_or_default());
    let asked = tx.state().has_pending_choice();
    let _committed = tx.commit();
    assert!(asked, "first round asks");
}

// 第一百四十五批：合韵延伸（S2）——potheyun 三段链。
const POTHEYUN: u32 = 2_439_155_128;
const POTHEYUN_SOURCE: &str = include_str!("../../../content/original/ported/bingshi/potheyun.skill");

fn started_potheyun() -> (GameState, PlayerId, PlayerId) {
    let compiled =
        content_compiler::compile_source_named("bingshi-potheyun-test", POTHEYUN_SOURCE).unwrap();
    let helper = "skill shared_a(3900000011) { trigger phase_end owner_is_target optional { effect draw_cards(self, 1); } }\nskill shared_b(3900000012) { modifier hand_limit add 0; }\nskill locked_c(3900000013) locked { modifier hand_limit add 0; }\n";
    let compiled2 =
        content_compiler::compile_source_named("bingshi-potheyun-helpers", helper).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_844, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled2).unwrap();
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let clear: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, clear).unwrap();
    // 双方共享 shared_a/shared_b；owner 独有 locked_c（不可共享/不可失去）。
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(POTHEYUN),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(3_900_000_011),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(3_900_000_013),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(3_900_000_011),
            },
            EffectRequest::AddSkill {
                player: other,
                skill_def: SkillDefId::new(3_900_000_012),
            },
        ],
    )
    .unwrap();
    (tx.commit().state, owner, other)
}

#[test]
fn potheyun_loses_shared_skill_and_target_draws_two() {
    let (state, owner, other) = started_potheyun();
    let mut tx = Transaction::begin(state);
    let other_defs_before = tx.state().hand_count(other);
    let potheyun_instance = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.owner == owner && s.def.raw() == POTHEYUN)
        .unwrap()
        .id;
    let _ = run_effects(
        &mut tx,
        vec![EffectRequest::ResolvePotheyun {
            player: owner,
            skill: potheyun_instance,
        }],
    );
    assert!(tx.state().has_pending_choice(), "stage0 target choice");
    let committed = tx.commit();
    let state = resolve(committed.state, owner, vec![other.raw()], 9_101);
    let state = resolve(state, owner, vec![0], 9_102);
    // 共同技能只有 shared_a（locked_c 被过滤、shared_b 是对方独有）。
    assert!(
        !has_skill(&state, owner, 3_900_000_011),
        "owner lost shared_a"
    );
    assert!(
        has_skill(&state, owner, 3_900_000_013),
        "locked skill untouched"
    );
    assert_eq!(
        state.hand_count(other),
        other_defs_before + 2,
        "target drew 2"
    );
}
