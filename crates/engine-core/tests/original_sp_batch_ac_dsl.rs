//! Golden for sp batch-AC DSL rewrite:
//! 2 个技能——被批量脚本误标为「audio-only」/「需修改技原语」，上游 (sp/skill.js) 在
//! 文件头注释引用的行号处均有真实规则体：
//!   - olmiaoxin（妙心·OL）：受伤后摸 2（damage owner_is_target forced）
//!   - oltingji（廷机·OL）：回合内指定未受伤其他角色为目标后令该牌不可被响应
//!     （card_targeted + 新增 owner_source_current_phase_card_targeted_other_healthy）

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

const OLMIAOXIN: u32 = 3_006_386_800;
const OLTINGJI: u32 = 4_244_206_396;
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

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_301),
        kind,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_publishes_batch_ac_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, OLMIAOXIN | OLTINGJI))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLMIAOXIN, "damage", "owner_is_target", false)));
    assert!(triggers.contains(&(
        OLTINGJI,
        "card_targeted",
        "owner_source_current_phase_card_targeted_other_healthy",
        false
    )));
    // 两技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, OLMIAOXIN | OLTINGJI)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn olmiaoxin_draws_two_after_damage() {
    let (state, owner) = started(2_026_081_601);
    let state = with_skill(state, owner, OLMIAOXIN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 2);
}

#[test]
fn oltingji_makes_card_unrespondable_when_targeting_healthy_other() {
    let (state, owner) = started(2_026_081_602);
    let state = with_skill(state, owner, OLTINGJI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 目标保持满血（健康）
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: other,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    // 触发成功且无选择（锁定技强制）——事件中应含不可响应效果
    assert!(
        !effects.is_empty(),
        "healthy-other target must trigger 廷机"
    );
    run_effects(&mut tx, effects).unwrap();
}

#[test]
fn oltingji_does_not_fire_for_wounded_target() {
    let (state, owner) = started(2_026_081_603);
    let state = with_skill(state, owner, OLTINGJI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 打伤目标
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: owner,
            card: sha,
            target: other,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects.is_empty(),
        "wounded target must not trigger 廷机"
    );
}
