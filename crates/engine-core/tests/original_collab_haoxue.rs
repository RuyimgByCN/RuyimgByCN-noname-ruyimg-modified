//! 第一百四十七批：collab 好学吕蒙「好学」（haoxue）零新原语真实语义重写。
//! 上游：出牌阶段内获得牌后回复 1 点体力（需已受伤）；出牌阶段外弃置牌后失去 1 点体力。
//! 实现：cards_moved 双臂 + owner_gained_cards_during_play_phase_wounded /
//! owner_discarded_outside_play_phase（本批注册过滤器）。

use engine_api::{PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const HAOXUE: u32 = 4_229_967_180;
const HAOXUE_SRC: &str = include_str!("../../../content/original/ported/collab/haoxue.skill");

fn bootstrap() -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("haoxue", HAOXUE_SRC).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(HAOXUE), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
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
        skill_def: SkillDefId::new(HAOXUE),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn hp_of(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state().players.get(player).unwrap().hp
}

fn max_hp_of(tx: &Transaction, player: PlayerId) -> u32 {
    tx.state().players.get(player).unwrap().max_hp
}

#[test]
fn haoxue_recovers_once_when_gaining_during_own_play_phase() {
    let (mut tx, owner, _other) = bootstrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let max_hp = max_hp_of(&tx, owner);
    tx.state_mut().players.get_mut(owner).unwrap().hp = max_hp as i32 - 2;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        max_hp as i32 - 1,
        "one gain event must recover exactly 1 (not per card)"
    );
}

#[test]
fn haoxue_no_recover_when_full_hp() {
    let (mut tx, owner, _other) = bootstrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(hp_of(&tx, owner), before, "full-HP gains must not recover");
}

#[test]
fn haoxue_loses_hp_when_discarding_outside_own_play_phase() {
    let (mut tx, owner, other) = bootstrap();
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before - 1,
        "discard outside own play phase must cost 1 HP"
    );
}

#[test]
fn haoxue_no_loss_when_discarding_during_own_play_phase() {
    let (mut tx, owner, _other) = bootstrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::DrawCards {
            player: owner,
            count: 2,
        }],
    )
    .unwrap();
    let before = hp_of(&tx, owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(
        hp_of(&tx, owner),
        before,
        "discard during own play phase must not cost HP"
    );
}
