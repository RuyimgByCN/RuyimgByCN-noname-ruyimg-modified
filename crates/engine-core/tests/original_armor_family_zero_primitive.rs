//! Golden coverage for ADR 0643: armor family dissolves to zero primitives via
//! MARK_HUJIA (def 3) + `owner_mark_<def>` value source. 5 skills across
//! mobile / key / shahai, no ABI change (500 / 651 / 645).

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YUXIANG: u32 = 2_271_957_513;
const ERIKA_SHISONG: u32 = 2_366_255_007;
const TOMOYO_CHANGSHI: u32 = 3_488_868_404;
const ZHANHOU: u32 = 2_287_156_849;
const OLDZHANHOU: u32 = 3_706_511_387;

const MOBILE: &[u8] = include_bytes!("../../../content/original/ported/mobile/mobile.npack.json");
const KEY: &[u8] = include_bytes!("../../../content/original/ported/key/key.npack.json");
const SHAHAI: &[u8] = include_bytes!("../../../content/original/ported/shahai/shahai.npack.json");
const CHANGSHI_SRC: &str =
    include_str!("../../../content/original/ported/key/tomoyo_changshi.skill");

fn hujia(state: &GameState, player: PlayerId) -> i32 {
    state
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(def, _)| *def == 3)
        .map_or(0, |(_, n)| *n)
}

fn resolve(state: GameState, actor: PlayerId, selections: Vec<u32>, key: u64) -> GameState {
    let (state, pipe) = apply_command(
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
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    state
}

#[test]
fn armor_family_npack_contracts() {
    let mobile = content_schema::ContentPackage::from_json_bytes(MOBILE).unwrap();
    // 玉箱：出向 -护甲、入向 +护甲（无护甲时值为 0 自动失效）。
    assert!(mobile.modifier_index.iter().any(|m| {
        m.skill_def_id == YUXIANG
            && m.target == "distance_from"
            && m.value_source.as_deref() == Some("owner_mark_3")
    }));
    assert!(mobile.modifier_index.iter().any(|m| {
        m.skill_def_id == YUXIANG && m.target == "distance_to"
            && m.value_source.as_deref() == Some("owner_mark_3")
    }));
    assert_eq!(mobile.manifest.rule_vm_abi_version, 734);

    let key = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    // 诗诵：手牌上限 + 护甲值。
    assert!(key.modifier_index.iter().any(|m| {
        m.skill_def_id == ERIKA_SHISONG
            && m.target == "hand_limit"
            && m.value_source.as_deref() == Some("owner_mark_3")
    }));
    // 长视：任意回复触发授予护甲标记。
    assert!(key.trigger_index.iter().any(|t| {
        t.skill_def_id == TOMOYO_CHANGSHI && t.event == "hp_recovered" && !t.optional
    }));
    assert_eq!(key.manifest.rule_vm_abi_version, 691);

    let shahai = content_schema::ContentPackage::from_json_bytes(SHAHAI).unwrap();
    // 战吼双形态：限一次 / 不限。
    assert!(shahai
        .active_index
        .iter()
        .any(|a| a.skill_def_id == ZHANHOU));
    assert!(shahai
        .active_index
        .iter()
        .any(|a| a.skill_def_id == OLDZHANHOU));
    assert_eq!(shahai.manifest.rule_vm_abi_version, 657);
}

#[test]
fn changshi_recovery_grants_armor_mark() {
    let compiled = content_compiler::compile_source_named("changshi", CHANGSHI_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(TOMOYO_CHANGSHI), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|p| *p != owner)
        .unwrap();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(TOMOYO_CHANGSHI),
        }],
    )
    .unwrap();
    assert_eq!(hujia(tx.state(), owner), 0);
    // 其他角色回复体力 → 长视获得 1 点护甲（forced，无需确认）。
    let planned = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            player: other,
            source: PlayerId::NONE,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, planned).unwrap();
    let state = tx.commit().state;
    assert_eq!(hujia(&state, owner), 1, "recovery should grant one armor");
}
