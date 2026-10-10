//! Golden for SHIJI 神马超「驭雳 / 霆威」语义（ABI 409 / 410）。
//!
//! 驭雳：造成伤害前非雷改为雷、已雷则 +1（prevent + 同源同量同牌雷重放）；
//! 自己受伤前仅雷伤害被取消并摸伤害点数张。
//! 霆威：杀指定首个目标后加 4 霆标记并选一个目标四选一，取消则其横置。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const YULI: u32 = 3_155_821_166;
const TINGWEI: u32 = 4_145_075_227;
/// 霆标记（与寂灭 jimie 共用）。
const TING_MARK: u32 = 4_145_075_227;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shiji/shiji.npack.json");

fn setup(skill_def: u32, players: u8) -> (Transaction, PlayerId, Vec<PlayerId>) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(skill_def as u64),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: 20_260_802 + skill_def as u64,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let owner = engine.state().turn.current;
    let seats = engine.state().players.alive_by_seat();
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
        skill_def: SkillDefId::new(skill_def),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, seats)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

/// 驭雳：普通伤害被改为雷属性——目标若同为驭雳拥有者，改雷后其防御分支
/// （受伤前取消并摸牌）被触发，证明伤害确实被改成了雷。
#[test]
fn yuli_converts_normal_damage_to_thunder() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: target,
            skill_def: SkillDefId::new(YULI),
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let hand_before = tx.state().hand_count(target);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let state = tx.state();
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp_before,
        "改雷后目标应通过驭雳防御取消该伤害"
    );
    assert_eq!(
        state.hand_count(target),
        hand_before + 1,
        "目标应摸伤害点数张（1）"
    );
}

/// 驭雳：已是雷属性伤害时伤害 +1。
#[test]
fn yuli_increments_thunder_damage() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let target = seats.into_iter().find(|p| *p != owner).unwrap();
    let before = tx.state().players.get(target).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: owner,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    let after = tx.state().players.get(target).unwrap().hp;
    assert_eq!(after, before - 2, "雷伤害应 +1");
}

/// 驭雳：自己受到普通伤害不触发（照常受伤）。
#[test]
fn yuli_ignores_incoming_normal_damage() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let source = seats.into_iter().find(|p| *p != owner).unwrap();
    let before = tx.state().players.get(owner).unwrap().hp;

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .unwrap();
    let after = tx.state().players.get(owner).unwrap().hp;
    assert_eq!(after, before - 1, "普通伤害不触发驭雳防御");
}

/// 驭雳：自己受到雷伤害前取消并摸伤害点数张。
#[test]
fn yuli_deflects_incoming_thunder_and_draws() {
    let (mut tx, owner, seats) = setup(YULI, 2);
    let source = seats.into_iter().find(|p| *p != owner).unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let hand_before = tx.state().hand_count(owner);

    run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source,
            target: owner,
            amount: 2,
            is_lose_hp: false,
            nature: rule_api::damage_nature::THUNDER,
        }],
    )
    .unwrap();
    let state = tx.state();
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before,
        "雷伤害应被取消"
    );
    assert_eq!(state.hand_count(owner), hand_before + 2, "应摸伤害点数张");
}

/// 霆威：杀指定首个目标后加 4 霆标记并打开选目标 choice。
#[test]
fn tingwei_grants_marks_and_offers_target_choice() {
    let (mut tx, owner, seats) = setup(TINGWEI, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);

    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card: sha,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    // 确认 optional 发动 → module 执行 → 加 4 霆标记并打开选目标 choice。
    let state = confirm_optional(&mut tx, owner, 81);
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == TING_MARK && *count == 4),
        "应获得 4 枚霆标记"
    );
    assert!(state.has_pending_choice(), "应打开选目标 choice");
}

/// 霆威：目标四选一时取消 → 目标横置。
#[test]
fn tingwei_cancel_chains_target() {
    let (mut tx, owner, seats) = setup(TINGWEI, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card: sha,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    // 阶段 0：拥有者选择第一个目标。
    let mut state = confirm_optional(&mut tx, owner, 90);
    assert!(state.has_pending_choice(), "应打开选目标 choice");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(94),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![targets[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let mut state = state;
    assert!(state.has_pending_choice(), "应打开目标四选一 choice");
    // 阶段 1：目标取消 → 横置。
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor: targets[0],
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(91),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![],
                cancelled: true,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(
        next.players
            .get(targets[0])
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == engine_core::content::MARK_CHAIN && *count > 0),
        "取消应使目标横置"
    );
}

/// 霆威：目标选择「非锁定技失效」选项 → 目标非锁定技被压制。
#[test]
fn tingwei_suppress_option() {
    let (mut tx, owner, seats) = setup(TINGWEI, 3);
    let targets: Vec<_> = seats
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    assert_eq!(
        dispatch_for_events(
            &mut tx,
            &[DomainEvent::CardUsed {
                player: owner,
                card: sha,
                targets: targets.clone(),
            }],
            &mut TriggerFireRecord::new(),
        )
        .unwrap_err(),
        EffectError::ChoiceRequested
    );
    let mut state = confirm_optional(&mut tx, owner, 92);
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state.clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(95),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![targets[0].raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    let state = state;
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor: targets[0],
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(93),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(
        next.turn
            .non_locked_skill_suppressed_players
            .contains(&targets[0]),
        "目标非锁定技应被压制至其回合结束"
    );
}

/// 确认挂起的 optional 发动（selections=[1]=发动）。
fn confirm_optional(tx: &mut Transaction, actor: PlayerId, key: u64) -> GameState {
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

fn confirm(tx: &mut Transaction, owner: PlayerId, key: u64, target: PlayerId) -> GameState {
    let revision = tx.state().revision;
    let choice_id = ChoiceId::new(tx.state().resolution.pending_choice_id);
    let (state, result) = apply_command(
        tx.state().clone(),
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![target.raw()],
                cancelled: false,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}
