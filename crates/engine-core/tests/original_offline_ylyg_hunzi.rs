//! 第一百八十八批：offline「魂姿」（雁翎孙策 ylyg_hunzi）觉醒零新原语重写。
//! 上游：当你的体力为 1 时：加 1 点体力上限，获得〖英姿〗与〖英魂〗。
//! 实现：hp_lost owner_is_target + body hp==1 门控 + once 标记 → change_max_hp(+1)
//! + 跨包 add_skill 字面量（refresh 英姿、shenhua 英魂——均为 offline 声明依赖）。
//! 近似（登记债）：触发面收窄为体力流失（上游 changeHp 含回复至 1 的路径）。

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YLYG_HUNZI: u32 = 2_262_123_725;
const REYINGZI: u32 = 2_412_932_912; // refresh 英姿
const YINGHUN: u32 = 3_072_887_319; // shenhua 英魂
const SOURCE: &str = include_str!("../../../content/original/ported/offline/ylyg_hunzi.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64, hp: i32) -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("ylyg-hunzi-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YLYG_HUNZI),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(owner).unwrap().hp = hp;
    (tx.commit().state, owner)
}

fn lose_hp(state: GameState, who: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: PlayerId::NONE,
            target: who,
            amount,
            is_lose_hp: true,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn has_skill(state: &GameState, owner: PlayerId, skill_def: u32) -> bool {
    state
        .skills
        .sorted_live()
        .iter()
        .any(|skill| skill.owner == owner && skill.def.raw() == skill_def)
}

#[test]
fn package_publishes_ylyg_hunzi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let trigger = package
        .trigger_index
        .iter()
        .find(|t| t.skill_def_id == YLYG_HUNZI)
        .unwrap();
    assert_eq!(trigger.event, "hp_lost");
    assert_eq!(trigger.filter, "owner_is_target");
    assert!(!trigger.optional);
}

#[test]
fn awakening_at_one_hp_grants_yingzi_and_yinghun_once() {
    // hp=1 时流失 1 体力 → 触发：上限+1、获得双技。
    let (state, owner) = setup(20_260_912_02, 1);
    let max_before = state.players.get(owner).unwrap().max_hp;

    let state = lose_hp(state, owner, 1);
    assert_eq!(
        state.players.get(owner).unwrap().max_hp,
        max_before + 1,
        "gain one max hp"
    );
    assert!(has_skill(&state, owner, REYINGZI), "英姿 granted");
    assert!(has_skill(&state, owner, YINGHUN), "英魂 granted");

    // once 门控：再次流失不再触发（上限不再增长）。
    let max_after_awaken = state.players.get(owner).unwrap().max_hp;
    let state = lose_hp(state, owner, 1);
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_after_awaken);
}

#[test]
fn hp_above_one_does_not_awaken() {
    let (state, owner) = setup(20_260_912_03, 3);
    let max_before = state.players.get(owner).unwrap().max_hp;
    let state = lose_hp(state, owner, 1);
    // 合成事件只跑触发链不应用体力变化：hp 保持 3，觉醒门控不成立。
    assert_eq!(state.players.get(owner).unwrap().hp, 3);
    assert_eq!(state.players.get(owner).unwrap().max_hp, max_before);
    assert!(!has_skill(&state, owner, REYINGZI));
}
