//! 第一百四十七批：offline 王战张绣「从谏」（sclcongjian）零新原语真实语义重写。
//! 上游：你于回合外造成伤害，或于回合内受到伤害时，此伤害+1。
//! 实现：damage_before 双臂 + current_phase_player() 比较 + modify_current_damage。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SCLCONGJIAN: u32 = 2_538_416_032;
const SCLCONGJIAN_SRC: &str =
    include_str!("../../../../content/original/ported/offline/sclcongjian.skill");

fn bootstrap() -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("sclcongjian", SCLCONGJIAN_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(SCLCONGJIAN), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in tx.state().players.alive_by_seat() {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLCONGJIAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn hp_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state().players.get(player).unwrap().hp
}

#[test]
fn congjian_increases_incoming_damage_during_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = owner;
    // bootstrap 武将面板随座位分配变化，断言一律用相对值
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before - 2,
        "own-turn incoming damage must be +1"
    );
}

#[test]
fn congjian_leaves_incoming_damage_unchanged_outside_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = other;
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before - 1,
        "outside-turn incoming damage must stay at base"
    );
}

#[test]
fn congjian_increases_outgoing_damage_outside_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = other;
    let before = hp_of(&tx, other);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, other),
        before - 2,
        "outside-turn outgoing damage must be +1"
    );
}

#[test]
fn congjian_leaves_outgoing_damage_unchanged_during_own_turn() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = owner;
    let before = hp_of(&tx, other);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, other),
        before - 1,
        "own-turn outgoing damage must stay at base"
    );
}
