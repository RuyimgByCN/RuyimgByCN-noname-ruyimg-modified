//! Golden for sp batch-Z DSL rewrite:
//! 3 个州郡（px*）技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头
//! 注释引用的行号处均有真实规则体（archive 与同步源一致），全部零新原语移植：
//!   - pxdaifang（带方）：属性【杀】不可被响应（modifier card_use_unrespondable）
//!   - pxpeixian（沛县）：锦囊不可被响应（modifier card_use_unrespondable）
//!   - pxningzhou（宁州）：【杀】不可被抵消 + 杀伤害视为体力流失
//!     （card_use_unrespondable + damage_before owner_source_sha prevent+lose_hp）

use engine_api::{
    Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXDAIFANG: u32 = 2_175_017_211;
const PXPEIXIAN: u32 = 4_115_824_913;
const PXNINGZHOU: u32 = 2_289_560_906;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

#[test]
fn package_publishes_batch_z_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    // 三技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXDAIFANG | PXPEIXIAN | PXNINGZHOU
        ) && modifier.target == "hand_limit"
    }));
    // 带方/沛县/宁州均为 card_use_unrespondable 修饰
    assert_eq!(
        package
            .modifier_index
            .iter()
            .filter(|modifier| matches!(
                modifier.skill_def_id,
                PXDAIFANG | PXPEIXIAN | PXNINGZHOU
            ))
            .filter(|modifier| modifier.target == "card_use_unrespondable"
                && modifier.mode == "override"
                && modifier.value == 1)
            .count(),
        3
    );
    // 宁州另有 damage_before 触发
    assert!(package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXNINGZHOU
            && trigger.event == "damage_before"
            && trigger.filter == "owner_source_sha"
            && !trigger.optional));
}

#[test]
fn pxdaifang_makes_elemental_sha_unrespondable() {
    let (state, owner) = started(2_026_081_301);
    let state = with_skill(state, owner, PXDAIFANG);
    let tx = Transaction::begin(state);
    // 属性【杀】不可被响应
    assert!(
        engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::ShaFire,
            engine_api::Suit::Heart,
        ),
        "elemental sha must be unrespondable"
    );
    // 普通【杀】不受影响
    assert!(
        !engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::Sha,
            engine_api::Suit::Spade,
        ),
        "ordinary sha must remain respondable"
    );
}

#[test]
fn pxpeixian_makes_trick_unrespondable() {
    let (state, owner) = started(2_026_081_302);
    let state = with_skill(state, owner, PXPEIXIAN);
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::TrickGuohe,
            engine_api::Suit::Spade,
        ),
        "trick must be unrespondable"
    );
    // 基本牌不受影响
    assert!(
        !engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::Sha,
            engine_api::Suit::Spade,
        ),
        "basic card must remain respondable"
    );
}

#[test]
fn pxningzhou_makes_sha_unrespondable() {
    let (state, owner) = started(2_026_081_303);
    let state = with_skill(state, owner, PXNINGZHOU);
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::card_use_unrespondable_for_kind(
            tx.state(),
            owner,
            CardKind::Sha,
            engine_api::Suit::Spade,
        ),
        "sha must be unrespondable"
    );
}

#[test]
fn pxningzhou_turns_sha_damage_into_hp_loss() {
    let (state, owner) = started(2_026_081_304);
    let state = with_skill(state, owner, PXNINGZHOU);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    // 杀伤害在 damage_before 被防止，改为等量体力流失
    let effects = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source: owner,
            target: other,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    // 触发链会先经过 prevent；若无选择，则事件中应含体力流失
    match effects {
        Ok(events) => {
            assert!(
                events
                    .iter()
                    .any(|event| matches!(event, DomainEvent::DamagePrevented { .. })),
                "sha damage must be prevented"
            );
            assert!(
                events.iter().any(|event| matches!(
                    event,
                    DomainEvent::DamageDealt {
                        is_lose_hp: true,
                        ..
                    }
                )),
                "sha damage must convert to hp loss"
            );
            assert_eq!(tx.state().players.get(other).unwrap().hp, other_hp - 1);
        }
        Err(EffectError::ChoiceRequested) => {
            panic!("pxningzhou is a locked forced skill and must not open choices");
        }
        Err(other) => panic!("unexpected effect error: {other:?}"),
    }
}
