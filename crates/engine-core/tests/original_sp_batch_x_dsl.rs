//! Golden for sp batch-X DSL rewrite:
//! 5 个被批量脚本误标为「audio-only」的州郡（px*）技能——上游 (sp/skill.js) 在
//! 文件头注释引用的行号处均有真实规则体（archive 与同步源一致），全部零新原语移植：
//!   - pxqiaoxian（谯县）：受伤后可选摸 1（damage + owner_is_target optional）
//!   - pxdunhuang（敦煌）：锁定，【杀】被抵消后摸 1（sha_dodged + owner_source_sha_dodged）
//!   - pxchenliu（陈留）：锁定，跳过弃牌阶段（modifier skip_discard_phase override 1）
//!   - pxyanzhou（兖州）：回复体力后可选摸 1（hp_recovered + owner_is_target optional）
//!   - pxdianchi（滇池）：造成火焰伤害后可令受伤角色失去 1 点体力
//!     （damage + owner_source_fire optional）

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, ContinuationKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const PXQIAOXIAN: u32 = 2_589_677_698;
const PXDUNHUANG: u32 = 3_965_660_786;
const PXCHENLIU: u32 = 4_259_220_755;
const PXYANZHOU: u32 = 4_070_969_920;
const PXDIANCHI: u32 = 4_289_301_860;
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_x_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| {
            matches!(
                trigger.skill_def_id,
                PXQIAOXIAN | PXDUNHUANG | PXYANZHOU | PXDIANCHI
            )
        })
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(PXQIAOXIAN, "damage", "owner_is_target", true)));
    assert!(triggers.contains(&(PXDUNHUANG, "sha_dodged", "owner_source_sha_dodged", false)));
    assert!(triggers.contains(&(PXYANZHOU, "hp_recovered", "owner_is_target", true)));
    assert!(triggers.contains(&(PXDIANCHI, "damage", "owner_source_fire", true)));
    // 陈留以 modifier 实现，不应出现在 trigger_index
    assert!(!package
        .trigger_index
        .iter()
        .any(|trigger| trigger.skill_def_id == PXCHENLIU));
    // 五技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXQIAOXIAN | PXDUNHUANG | PXYANZHOU | PXDIANCHI
        ) && modifier.target == "hand_limit"
    }));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == PXCHENLIU
            && modifier.target == "skip_discard_phase"
            && modifier.mode == "override"
            && modifier.value == 1));
}

#[test]
fn pxqiaoxian_draws_after_damage_when_accepted() {
    let (state, owner) = started(2_026_081_101);
    let state = with_skill(state, owner, PXQIAOXIAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn pxqiaoxian_cancels_without_draw() {
    let (state, owner) = started(2_026_081_102);
    let state = with_skill(state, owner, PXQIAOXIAN);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![], true, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before);
}

#[test]
fn pxdunhuang_draws_when_owner_sha_dodged() {
    let (state, owner) = started(2_026_081_103);
    let state = with_skill(state, owner, PXDUNHUANG);
    let mut tx = Transaction::begin(state);
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_000),
        kind: CardKind::Sha,
        suit: engine_api::Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let before = tx.state().hand_count(owner);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::ShaDodged {
            source: owner,
            target: other,
            card,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn pxchenliu_skips_discard_phase() {
    let (state, owner) = started(2_026_081_104);
    let state = with_skill(state, owner, PXCHENLIU);
    // 陈留无触发，通过 modifier 跳过弃牌阶段：以查询校验
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::skip_discard_phase(tx.state(), owner),
        "skip_discard_phase modifier must force skip"
    );
}

#[test]
fn pxyanzhou_draws_after_recover() {
    let (state, owner) = started(2_026_081_105);
    let state = with_skill(state, owner, PXYANZHOU);
    let before = state.hand_count(owner);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: owner,
            player: owner,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 1);
}

#[test]
fn pxdianchi_makes_fire_damage_target_lose_hp() {
    let (state, owner) = started(2_026_081_106);
    let state = with_skill(state, owner, PXDIANCHI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let other_hp = tx.state().players.get(other).unwrap().hp;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.players.get(other).unwrap().hp, other_hp - 1);
}

#[test]
fn pxdianchi_ignores_non_fire_damage() {
    let (state, owner) = started(2_026_081_107);
    let state = with_skill(state, owner, PXDIANCHI);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::DamageDealt {
            source: owner,
            target: other,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(effects.is_empty(), "non-fire damage must not trigger");
}
