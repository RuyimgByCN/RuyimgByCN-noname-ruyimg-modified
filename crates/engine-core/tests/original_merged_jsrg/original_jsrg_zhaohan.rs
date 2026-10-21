//! Golden for JSRG 杨彪「昭汉」的牌堆重洗分支语义。

use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId};
use engine_core::effect::{reshuffle_discard_into_draw, run_effects};
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::{canonical_state_hash, Engine};
use rule_api::EffectRequest;

const ZHAOHAN: u32 = 3_529_273_369;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");

fn loaded(seed: u64) -> (Transaction, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 730);
    assert_eq!(package.skill_index.len(), 181);
    assert_eq!(package.trigger_index.len(), 56);
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHAOHAN
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_draw_reshuffle_zero_wounded"
            && !trigger.optional
    }));
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == ZHAOHAN
            && trigger.event == "phase_prepare"
            && trigger.filter == "owner_is_target_draw_reshuffle_positive"
            && !trigger.optional
    }));

    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;
    let mut tx = Transaction::begin(engine.state().clone());
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(ZHAOHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner)
}

fn enter_prepare(tx: &mut Transaction, owner: PlayerId) {
    run_effects(
        tx,
        vec![EffectRequest::SetTurnState {
            current: owner,
            phase: Phase::Prepare.to_u8(),
        }],
    )
    .unwrap();
}

#[test]
fn before_first_reshuffle_only_a_wounded_owner_recovers() {
    let (mut wounded, owner) = loaded(20_260_725);
    let max_hp = wounded.state().players.get(owner).unwrap().max_hp as i32;
    wounded.state_mut().players.get_mut(owner).unwrap().hp = max_hp - 1;
    enter_prepare(&mut wounded, owner);
    assert_eq!(wounded.state().players.get(owner).unwrap().hp, max_hp);

    let (mut healthy, owner) = loaded(20_260_726);
    let hp = healthy.state().players.get(owner).unwrap().hp;
    enter_prepare(&mut healthy, owner);
    assert_eq!(healthy.state().players.get(owner).unwrap().hp, hp);
}

#[test]
fn after_a_reshuffle_the_owner_loses_hp_even_when_healthy() {
    let (mut tx, owner) = loaded(20_260_727);
    tx.state_mut().draw_reshuffle_count = 1;
    let hp = tx.state().players.get(owner).unwrap().hp;
    enter_prepare(&mut tx, owner);
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp - 1);
}

#[test]
fn actual_nonempty_discard_reshuffle_increments_once() {
    let (mut tx, owner) = loaded(20_260_728);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, 0);

    reshuffle_discard_into_draw(&mut tx).unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, 1);

    reshuffle_discard_into_draw(&mut tx).unwrap();
    assert_eq!(tx.state().draw_reshuffle_count, 1);
}

#[test]
fn reshuffle_count_is_canonical_and_old_zero_state_still_decodes() {
    let zero = engine_core::state::GameState::empty();
    let encoded = serde_json::to_value(&zero).unwrap();
    assert!(encoded.get("draw_reshuffle_count").is_none());
    assert_eq!(
        serde_json::from_value::<engine_core::state::GameState>(encoded)
            .unwrap()
            .draw_reshuffle_count,
        0
    );

    let mut one = zero.clone();
    one.draw_reshuffle_count = 1;
    assert_ne!(canonical_state_hash(&zero), canonical_state_hash(&one));
}
