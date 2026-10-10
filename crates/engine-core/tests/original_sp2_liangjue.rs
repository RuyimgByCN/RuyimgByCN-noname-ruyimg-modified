//! 第一百六十五批：sp2「狼灭」（张横 liangjue）零引擎改动重写。
//! 上游：锁定技，当黑色牌进入或离开你的装备区/判定区后，你摸两张牌；若体力值
//! 大于 1，你失去 1 点体力。实现：cards_moved +
//! owner_black_card_moved_equip_judge（本批六点注册；进向经 to_zone ZoneId 反查
//! owner）→ draw_cards(self, 2) + if hp(self) > 1 lose_hp(self, 1)。

use engine_api::{
    CardDefId, CardId, CardMoveReason, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::dispatch_for_events;
use engine_core::Engine;
use rule_api::EffectRequest;

const LIANGJUE: u32 = 2_187_481_060;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp2/sp2.npack.json");

fn started(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 3,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    let other = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, other)
}

fn strip_and_arm(state: GameState, owner: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(
            tx.state()
                .players
                .alive_by_seat()
                .into_iter()
                .map(|player| EffectRequest::DiscardFromHand {
                    player,
                    count: u32::MAX,
                }),
        )
        .chain([EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(LIANGJUE),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn alloc_card(
    tx: &mut Transaction,
    owner: PlayerId,
    zone_kind: ZoneKind,
    suit: Suit,
) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(98_901),
        kind: CardKind::Weapon,
        suit,
        rank: 5,
        weapon_range: 1,
        zone,
        owner,
        face_up: true,
    })
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

#[test]
fn package_publishes_liangjue_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == LIANGJUE
            && trigger.event == "cards_moved"
            && trigger.filter == "owner_black_card_moved_equip_judge"
            && !trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == LIANGJUE && modifier.target == "hand_limit"));
}

#[test]
fn liangjue_draws_two_and_loses_one_when_black_card_enters_equip() -> Result<(), Box<dyn std::error::Error>> {
    let (state, owner, _other) = started(2_026_091_101);
    let state = strip_and_arm(state, owner);
    let hp_before = hp_of(&state, owner);
    assert!(hp_before > 1, "bootstrap hp must exceed 1 for the lose arm");

    // 黑色装备牌进入 owner 装备区（CardsMoved from 手牌区 to 装备区）。
    let mut tx = Transaction::begin(state);
    let from = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let to = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = alloc_card(&mut tx, owner, ZoneKind::Hand, Suit::Club);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::UseOrRespond,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;

    assert_eq!(hand_count(&state, owner), hand_before + 2, "draws two");
    assert_eq!(hp_of(&state, owner), hp_before - 1, "loses one when hp above 1");
    Ok(())
}
#[test]
fn liangjue_skips_red_card_and_non_owner_zones() -> Result<(), Box<dyn std::error::Error>> {
    let (state, owner, other) = started(2_026_091_102);
    let state = strip_and_arm(state, owner);

    // 红色装备牌进入 owner 装备区：不触发。
    let mut tx = Transaction::begin(state);
    let from = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let to = tx.state().zones.find(ZoneKind::Equip, owner).unwrap();
    let card = alloc_card(&mut tx, owner, ZoneKind::Hand, Suit::Heart);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::UseOrRespond,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;
    assert_eq!(hand_count(&state, owner), hand_before, "red card must not trigger");

    // 黑色牌进入**他人**装备区：不触发（区限定自己）。
    let mut tx = Transaction::begin(state);
    let from = tx.state().zones.find(ZoneKind::Hand, other).unwrap();
    let to = tx.state().zones.find(ZoneKind::Equip, other).unwrap();
    let card = alloc_card(&mut tx, other, ZoneKind::Hand, Suit::Spade);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::UseOrRespond,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;
    assert_eq!(hand_count(&state, owner), hand_before, "other's equip zone must not trigger");
    Ok(())
}
#[test]
fn liangjue_at_one_hp_draws_without_losing() -> Result<(), Box<dyn std::error::Error>> {
    let (state, owner, _other) = started(2_026_091_103);
    let state = strip_and_arm(state, owner);

    // hp 压到 1：摸 2 但不失体力。
    let mut tx = Transaction::begin(state);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 1;
    let from = tx.state().zones.find(ZoneKind::Judge, owner).unwrap();
    let to = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = alloc_card(&mut tx, owner, ZoneKind::Judge, Suit::Spade);
    tx.state_mut()
        .zones
        .get_mut(from)
        .unwrap()
        .cards
        .retain(|c| *c != card);
    tx.state_mut().zones.get_mut(to).unwrap().cards.push(card);
    let hand_before = hand_count(&tx.state(), owner);
    let requests = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardsMoved {
            cards: vec![card],
            from,
            to,
            reason: CardMoveReason::Discard,
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    )?;
    run_effects(&mut tx, requests)?;
    let state = tx.commit().state;

    assert_eq!(hand_count(&state, owner), hand_before + 2, "still draws two at 1 hp");
    assert_eq!(hp_of(&state, owner), 1, "no hp loss at 1 hp");
    Ok(())
}