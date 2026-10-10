//! Golden for sp batch-Y DSL rewrite:
//! 4 个州郡（px*）技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头
//! 注释引用的行号处均有真实规则体（archive 与同步源一致），全部零新原语移植：
//!   - pxfuling（涪陵）：装备牌不能被其他角色弃置（modifier card_discard_protected）
//!   - pxwudu（武都）：使用牌无次数限制（modifier card_use_unlimited）
//!   - pxanding（安定）：锁定，传导属性伤害防止（damage_before + chain_damage_linked）
//!   - pxzhuti（朱提）：属性伤害改为回复等量体力（damage_before 双触发 prevent+recover）

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

const PXFULING: u32 = 3_616_870_464;
const PXWUDU: u32 = 4_254_976_850;
const PXANDING: u32 = 3_330_093_204;
const PXZHUTI: u32 = 2_771_412_426;
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
fn package_publishes_batch_y_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXANDING | PXZHUTI))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(PXANDING, "damage_before", "owner_is_target_chain_damage_linked", false)));
    assert!(triggers.contains(&(PXZHUTI, "damage_before", "owner_target_fire", true)));
    assert!(triggers.contains(&(PXZHUTI, "damage_before", "owner_target_thunder", true)));
    // 涪陵/武都以 modifier 实现，不应出现在 trigger_index
    assert!(!package
        .trigger_index
        .iter()
        .any(|trigger| matches!(trigger.skill_def_id, PXFULING | PXWUDU)));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == PXFULING
            && modifier.target == "card_discard_protected"
            && modifier.mode == "override"
            && modifier.value == 1));
    assert!(package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == PXWUDU
            && modifier.target == "card_use_unlimited"
            && modifier.mode == "override"
            && modifier.value == 1));
    // 四技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(
            modifier.skill_def_id,
            PXFULING | PXWUDU | PXANDING | PXZHUTI
        ) && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxfuling_protects_equip_from_other_player_discard() {
    let (state, owner) = started(2_026_081_201);
    let state = with_skill(state, owner, PXFULING);
    let mut tx = Transaction::begin(state);
    // 给 owner 装一件装备（防具）
    let equip = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(2_000_001),
        kind: CardKind::Armor,
        suit: engine_api::Suit::Club,
        rank: 2,
        weapon_range: 0,
        zone: equip,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(equip).unwrap().cards.push(card);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 其他角色不能弃置该装备
    assert!(
        engine_core::query::card_discard_protected_from(tx.state(), other, owner, card),
        "other player must not be able to discard owner's equip"
    );
    // 本人仍可弃置自己的装备
    assert!(
        !engine_core::query::card_discard_protected_from(tx.state(), owner, owner, card),
        "owner can still discard own equip"
    );
}

#[test]
fn pxwudu_unlimits_card_use() {
    let (state, owner) = started(2_026_081_202);
    let state = with_skill(state, owner, PXWUDU);
    let tx = Transaction::begin(state);
    assert!(
        engine_core::query::card_use_unlimited_for(tx.state(), owner),
        "card_use_unlimited modifier must lift use limits"
    );
}

#[test]
fn pxanding_prevents_chain_damage() {
    let (state, owner) = started(2_026_081_203);
    let state = with_skill(state, owner, PXANDING);
    let mut tx = Transaction::begin(state);
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // 构造一个带传导标记的 DamageBefore：经 CommitDamage 管线派发
    let effects = run_prevent_check(&mut tx, source, owner, rule_api::damage_nature::THUNDER);
    assert!(
        effects
            .iter()
            .any(|event| matches!(event, DomainEvent::DamagePrevented { .. })),
        "chain elemental damage must be prevented"
    );
}

#[test]
fn pxzhuti_replaces_elemental_damage_with_recover() {
    let (state, owner) = started(2_026_081_204);
    let state = with_skill(state, owner, PXZHUTI);
    let mut tx = Transaction::begin(state);
    // 先把 owner 打伤（2 人局 owner 为主公，满血 5/5，回满无空间），再验证回复
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let source = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    // damage_before 触发链：火焰伤害打开可选确认（prevent + recover）
    let opened = run_effects(
        &mut tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target: owner,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::FIRE,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // 伤害被防止，改为回复等量体力
    assert_eq!(state.players.get(owner).unwrap().hp, hp_before + 1);
}

/// 通过 CommitDamage 管线派发带传导标记的属性伤害，返回产出事件。
fn run_prevent_check(
    tx: &mut Transaction,
    source: PlayerId,
    target: PlayerId,
    nature: u8,
) -> Vec<DomainEvent> {
    // CommitDamage 直接派发 damage_before 触发链
    let effects = run_effects(
        tx,
        vec![EffectRequest::CommitDamage {
            damage_id: 0,
            source,
            target,
            amount: 1,
            base_amount: 1,
            is_lose_hp: false,
            nature,
            card: engine_api::CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: rule_api::damage_card_flag::CHAIN_PROPAGATED,
        }],
    );
    match effects {
        Ok(events) => events,
        Err(EffectError::ChoiceRequested) => Vec::new(),
        Err(other) => panic!("unexpected effect error: {other:?}"),
    }
}
