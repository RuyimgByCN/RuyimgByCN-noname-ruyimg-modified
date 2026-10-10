//! Golden for SHIJI 许靖「博明 / 恶荐」语义（ABI 413 / 414）。
//!
//! 博明：出牌阶段限两次，将一张手牌/装备交给一名其他角色。
//! 恶荐：赠牌后目标若另有同类牌则二选一——受 1 点伤害 / 展示手牌并弃所有同类牌。

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

const BOMING: u32 = 4_154_330_954;
const EJIAN: u32 = 4_172_706_421;
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
                rng_seed: 20_260_805,
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

/// 博明：主动技赠牌——选中的手牌进入目标手牌。
#[test]
fn boming_gives_selected_card() {
    let (mut tx, owner, seats) = setup(BOMING, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let before = tx.state().hand_count(other);

    // 手动构造主动技调用(选牌+选目标)。
    let boming_skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == BOMING && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveBoming {
            player: owner,
            target: other,
            card: gift,
            skill: boming_skill,
        }],
    )
    .unwrap();
    assert_eq!(
        tx.state().hand_count(other),
        before + 1,
        "博明应把选中的牌交给目标"
    );
    assert!(
        tx.state()
            .zones
            .get(tx.state().zones.find(ZoneKind::Hand, other).unwrap())
            .unwrap()
            .cards
            .contains(&gift),
        "目标手牌应含赠牌"
    );
}

/// 恶荐：赠牌后目标有同类牌 → 二选一 choice。
#[test]
fn ejian_offers_choice_when_same_category_exists() {
    let (mut tx, owner, seats) = setup(EJIAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 目标已有 1 张杀,再被赠 1 张杀 → 有同类。
    let existing = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    // 赠牌触发 CardsGiven。
    let err = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .map(|_| ())
    .err();
    // 手动 dispatch 只返回 effects;执行它打开二选一 choice。
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let result = run_effects(&mut tx, effects);
    assert_eq!(result.err(), Some(EffectError::ChoiceRequested));
    assert!(tx.state().has_pending_choice(), "应打开二选一 choice");
    let _ = existing;
    let _ = err;
}

/// 恶荐：目标选"受 1 点伤害"。
#[test]
fn ejian_damage_option() {
    let (mut tx, owner, seats) = setup(EJIAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let existing = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let hp_before = tx.state().players.get(other).unwrap().hp;
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).err(),
        Some(EffectError::ChoiceRequested)
    );
    let state = resolve_choice(tx.commit().state, other, 1, vec![0], false);
    assert_eq!(
        state.players.get(other).unwrap().hp,
        hp_before - 1,
        "选受伤应受 1 点伤害"
    );
    let _ = existing;
}

/// 恶荐：目标选"展示手牌并弃所有同类牌"——目标弃掉全部同类牌。
#[test]
fn ejian_discard_same_category_option() {
    let (mut tx, owner, seats) = setup(EJIAN, 2);
    let other = seats.into_iter().find(|p| *p != owner).unwrap();
    let existing = inject_hand(&mut tx, other, CardKind::Sha, Suit::Club);
    let gift = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    let hand_before = tx.state().hand_count(other);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsGiven {
            giver: owner,
            recipient: other,
            cards: vec![gift],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert_eq!(
        run_effects(&mut tx, effects).err(),
        Some(EffectError::ChoiceRequested)
    );
    // 选"展示弃同类"(option 1)。
    let state = resolve_choice(tx.commit().state, other, 2, vec![1], false);
    // 目标弃掉了同类(杀)牌;赠牌(杀)本身是 gift——它在目标手上也同类,会被弃。
    assert!(state.hand_count(other) < hand_before, "目标应弃掉同类牌");
    let _ = existing;
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
