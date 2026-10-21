//! 第一百八十六批：offline「魔魇」（魔姜维 psmoyan）零新原语重写。
//! 上游：锁定技——你造成和受到的伤害改为体力流失（damageBefore cancel + 等量
//! loseHp）；有角色失去体力后你摸一张牌（loseHpAfter）。
//! 实现：hp_lost owner_alive（任意角色失去体力）→ draw_cards(self, 1)；
//! damage_before owner_is_source / owner_is_target 双臂 → prevent_current_damage()
//! + lose_hp 等量（pxningzhou 转流失模板）。近似债：自伤事件双臂同中。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::{damage_nature, EffectRequest};

const PSMOYAN: u32 = 2_742_008_810;
const PS_DEVIL_JIANGWEI: u32 = 3_617_390_344;
const SOURCE: &str = include_str!("../../../../content/original/ported/offline/psmoyan.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("psmoyan-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(PSMOYAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn commit_damage(state: GameState, source: PlayerId, target: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    // damage_before 触发在 DealDamage 的 prepare 路径内规划（直驱 CommitDamage
    // 会绕过触发面）；转化产生的 lose_hp 事件由效果层回灌，驱动魔魇①摸牌臂。
    let effects = vec![EffectRequest::DealDamage {
        source,
        target,
        amount,
        is_lose_hp: false,
        nature: damage_nature::NORMAL,
    }];
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

/// 引擎统一体力变化事件：is_lose_hp=true 的 DamageDealt 即 hp_lost 触发面。
fn dispatch_hp_lost(state: GameState, who: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: PlayerId::NONE,
            target: who,
            amount,
            is_lose_hp: true,
            nature: damage_nature::NORMAL,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_psmoyan_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let arms: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == PSMOYAN)
        .collect();
    assert_eq!(arms.len(), 3, "hp_lost + damage_before×2 arms");
    assert!(arms
        .iter()
        .any(|t| t.event == "hp_lost" && t.filter == "owner_alive"));
    assert!(arms
        .iter()
        .any(|t| t.event == "damage_before" && t.filter == "owner_is_source"));
    assert!(arms
        .iter()
        .any(|t| t.event == "damage_before" && t.filter == "owner_is_target"));
    assert!(arms.iter().all(|t| !t.optional), "locked skill");
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == PS_DEVIL_JIANGWEI)
        .expect("ps_devil_jiangwei in roster");
    assert!(character.skills.contains(&PSMOYAN));
}

#[test]
fn owner_dealt_damage_converts_to_target_hp_loss_and_draws() {
    let (state, owner, other) = setup(20_260_911_01);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(other).unwrap().hp = 3;
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        tx.commit().state
    };
    let hand_before = state.hand_count(owner);

    let state = commit_damage(state, owner, other, 2);

    // 伤害被防止：目标体力只经体力流失扣 2（无护甲/伤害吸收面）。
    assert_eq!(
        state.players.get(other).unwrap().hp,
        1,
        "damage converted to 2-point hp loss"
    );
    // 目标失去体力 → 魔魇①：owner 摸 1。
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn owner_received_damage_converts_to_own_hp_loss_and_draws() {
    let (state, owner, other) = setup(20_260_911_02);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
        tx.state_mut().turn.current = owner;
        tx.state_mut().turn.phase = Phase::Play;
        tx.commit().state
    };
    let hand_before = state.hand_count(owner);
    let other_hp_before = state.players.get(other).unwrap().hp;

    let state = commit_damage(state, other, owner, 1);

    assert_eq!(
        state.players.get(owner).unwrap().hp,
        3,
        "incoming damage converted to 1-point hp loss"
    );
    assert_eq!(state.hand_count(owner), hand_before + 1);
    assert_eq!(
        state.players.get(other).unwrap().hp,
        other_hp_before,
        "source must not lose hp when owner converts incoming damage"
    );
}

#[test]
fn any_player_hp_loss_draws_owner_a_card() {
    let (state, owner, other) = setup(20_260_911_03);
    let mut state = {
        let mut tx = Transaction::begin(state);
        tx.state_mut().players.get_mut(other).unwrap().hp = 2;
        tx.commit().state
    };
    let hand_before = state.hand_count(owner);
    let state = dispatch_hp_lost(state, other, 1);
    assert_eq!(state.hand_count(owner), hand_before + 1);
}

#[test]
fn converted_chain_leaves_no_pending_choice() {
    // 防御性回归：转化链结束后无挂起选择（forced 技能无确认流）。
    let (state, owner, other) = setup(20_260_911_04);
    let state = commit_damage(state, owner, other, 1);
    assert!(!state.has_pending_choice());
}
