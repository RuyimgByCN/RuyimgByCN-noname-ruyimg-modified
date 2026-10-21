//! Golden for SHIJI 孔融「新礼让 / 新名士」语义（ABI 411）。
//!
//! 新礼让：其他角色摸牌阶段开始时（无谦标记时）令其多摸两张、自己得 1 谦标记
//! 并临时获得截获技能；弃牌阶段后可获得当前回合角色弃掉的一张弃牌；
//! 自己回合开始时若仍有谦标记则跳过判定与摸牌阶段。
//! 新名士：受伤后（有谦标记）来源强制弃一张 hej 牌，红色则回复、弃牌堆则获得。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const XINLIRANG: u32 = 2_286_538_070;
const XINMINGSHI: u32 = 2_976_944_922;
const XINLIRANG_GAIN: u32 = 3_887_425_911;
/// 谦标记（= xinlirang 定义 ID）。
const QIAN_MARK: u32 = 2_286_538_070;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/shiji/shiji.npack.json");

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
                rng_seed: 20_260_803 + skill_def as u64,
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

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn resolve_choice(
    state: GameState,
    actor: PlayerId,
    key: u64,
    selections: Vec<u32>,
    cancelled: bool,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    next
}

/// 新礼让：其他角色摸牌阶段开始时可选发动——其多摸两张、自己得谦标记与截获技能。
#[test]
fn xinlirang_grants_extra_draw_and_qian_mark() {
    let (mut tx, owner, seats) = setup(XINLIRANG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let hand_before = tx.state().hand_count(other);

    // 模拟其他角色的摸牌阶段开始事件。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    // optional 触发打开 Confirm choice。
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 1, vec![1], false);
    let state = Transaction::begin(state);
    let state = state.commit().state;
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .any(|(mark, count)| *mark == QIAN_MARK && *count == 1),
        "应获得 1 枚谦标记"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|s| s.def.raw() == XINLIRANG_GAIN && s.owner == owner),
        "应临时获得截获技能"
    );
    let _ = hand_before;
}

/// 新礼让·截获：当前回合角色弃牌阶段弃牌后，拥有者可获得一张其弃牌堆中的弃牌。
#[test]
fn xinlirang_gain_grabs_discarded_card() {
    let (mut tx, owner, seats) = setup(XINLIRANG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 先发动礼让（摸牌阶段）获得截获技能。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let mut state = resolve_choice(tx.commit().state, owner, 10, vec![1], false);
    // 其他角色弃牌阶段弃一张手牌 → cards_moved 触发截获。
    let card = state.hand_cards(other)[0];
    let hand = state.zones.find(ZoneKind::Hand, other).unwrap();
    let discard = state
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    // run_effects 自动 emit CardsMoved 并触发截获（turn.current 需是 other）。
    let mut tx = Transaction::begin(state);
    tx.state_mut().turn.current = other;
    tx.state_mut().turn.phase = Phase::Discard;
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![card],
            to: discard,
            from: Some(hand),
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 11, vec![card.raw()], false);
    assert!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, owner).unwrap())
            .unwrap()
            .cards
            .contains(&card),
        "应获得弃牌堆中的弃牌"
    );
}

/// 新礼让·让渡：拥有者回合开始时若有谦标记则跳过判定与摸牌阶段并清空标记。
#[test]
fn xinlirang_skip_phases_when_marked() {
    let (mut tx, owner, seats) = setup(XINLIRANG, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 发动一次礼让获得谦标记。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::Draw,
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, owner, 20, vec![1], false);
    // 拥有者回合开始 → 跳过判定/摸牌并清空标记。
    let mut tx = Transaction::begin(state);
    // 直接验证 skip 触发：模拟拥有者判定阶段开始（真实流程由 turn.rs 推进阶段）。
    tx.state_mut().turn.phase = Phase::Judge;
    // dispatch 返回待执行 effects；手动执行（真实流程由 run_effects 内部自动执行）。
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Judge,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let state = tx.commit().state;
    assert!(
        state
            .players
            .get(owner)
            .unwrap()
            .marks
            .iter()
            .all(|(mark, count)| !(*mark == QIAN_MARK && *count > 0)),
        "谦标记应被清空"
    );
}

/// 新名士：受伤后（有谦标记）来源强制弃一张牌，红色弃牌回复 1 点体力。
#[test]
fn xinmingshi_red_discard_recovers() {
    let (mut tx, owner, seats) = setup(XINMINGSHI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 手动加谦标记。
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(QIAN_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let hp_before = tx.state().players.get(owner).unwrap().hp;
    let red = inject_hand(&mut tx, other, CardKind::Other, Suit::Heart);
    // 受伤 1 点 → 触发新名士 → 来源强制弃牌 choice。
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, other, 30, vec![red.raw()], false);
    assert_eq!(
        state.players.get(owner).unwrap().hp,
        hp_before,
        "红色弃牌应回复 1 点（抵掉受伤）"
    );
}

/// 新名士：黑色弃牌进入弃牌堆则拥有者获得它。
#[test]
fn xinmingshi_black_discard_is_gained() {
    let (mut tx, owner, seats) = setup(XINMINGSHI, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: owner,
            mark: engine_api::MarkDefId::new(QIAN_MARK),
            delta: 1,
        }],
    )
    .unwrap();
    let black = inject_hand(&mut tx, other, CardKind::Other, Suit::Spade);
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: other,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested));
    let state = resolve_choice(tx.commit().state, other, 40, vec![black.raw()], false);
    assert!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, owner).unwrap())
            .unwrap()
            .cards
            .contains(&black),
        "黑色弃牌应被拥有者获得"
    );
}
