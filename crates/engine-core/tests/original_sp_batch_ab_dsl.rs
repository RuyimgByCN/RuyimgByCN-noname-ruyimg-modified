//! Golden for sp batch-AB DSL rewrite:
//! 2 个州郡（px*）技能——被批量脚本误标为「audio-only」，上游 (sp/skill.js) 在文件头
//! 注释引用的行号处均有真实规则体：
//!   - pxliangzhoux（梁州）：成为他人【杀】目标后弃置其一张牌
//!   - pxchengdu（成都）：对自己使用牌后摸 1（新增 owner_source_card_used_targets_self）

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

const PXLIANGZHOUX: u32 = 2_317_213_965;
const PXCHENGDU: u32 = 2_164_643_288;
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

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: engine_api::CardId::NONE,
        def: engine_api::CardDefId::new(1_000_201),
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
fn package_publishes_batch_ab_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, PXLIANGZHOUX | PXCHENGDU))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(
        PXLIANGZHOUX,
        "card_targeted",
        "owner_targeted_by_other_sha",
        true
    )));
    assert!(triggers.contains(&(
        PXCHENGDU,
        "card_used",
        "owner_source_card_used_targets_self",
        true
    )));
    // 两技均不再保留手牌上限 +0 的诚实 noop modifier
    assert!(!package.modifier_index.iter().any(|modifier| {
        matches!(modifier.skill_def_id, PXLIANGZHOUX | PXCHENGDU)
            && modifier.target == "hand_limit"
    }));
}

#[test]
fn pxliangzhoux_discards_sha_user_card() {
    let (state, owner) = started(2_026_081_501);
    let state = with_skill(state, owner, PXLIANGZHOUX);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let sha = inject_hand(&mut tx, other, CardKind::Sha);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardTargetAdded {
            player: other,
            card: sha,
            target: owner,
            targets: vec![owner],
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
    // 确认后打开对使用者手牌/装备的弃置选择
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseDiscardTargetCard
    );
}

#[test]
fn pxchengdu_draws_after_self_targeted_card_use() {
    let (state, owner) = started(2_026_081_502);
    let state = with_skill(state, owner, PXCHENGDU);
    let mut tx = Transaction::begin(state);
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let before = tx.state().hand_count(owner);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![owner],
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
fn pxchengdu_ignores_card_use_not_targeting_self() {
    let (state, owner) = started(2_026_081_503);
    let state = with_skill(state, owner, PXCHENGDU);
    let mut tx = Transaction::begin(state);
    let other = tx
        .state()
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let sha = inject_hand(&mut tx, owner, CardKind::Sha);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![other],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects.is_empty(),
        "card use not targeting self must not trigger"
    );
}
