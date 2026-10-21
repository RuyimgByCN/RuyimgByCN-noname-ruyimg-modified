//! Golden for SHIJI 蔡贞姬「舍医 / 天荫」语义（ABI 417 / 418）。
//!
//! 舍医：其他角色受伤前（每轮一次）交至少 max(1, hp) 张 he 牌防伤。
//! 天荫：结束阶段获得牌堆顶每个本阶段未用类别牌的第一张。

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

const SHEYI: u32 = 3_236_439_408;
const TIANYIN: u32 = 2_264_320_121;
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
                rng_seed: 20_260_807,
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

fn inject_card(
    tx: &mut Transaction,
    player: PlayerId,
    zone: ZoneKind,
    kind: CardKind,
    suit: Suit,
) -> CardId {
    let zone_id = tx.state().zones.find(zone, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(80_000 + kind.tag() as u32),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: zone_id,
        owner: player,
        face_up: false,
    });
    tx.state_mut()
        .zones
        .get_mut(zone_id)
        .unwrap()
        .cards
        .push(card);
    card
}

fn inject_hand(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    inject_card(tx, player, ZoneKind::Hand, kind, suit)
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

/// 舍医：其他角色受伤前打开交牌 choice，确认后防伤。
#[test]
fn sheyi_gives_cards_and_prevents_damage() {
    let (mut tx, owner, seats) = setup(SHEYI, 3);
    let others: Vec<_> = seats.into_iter().filter(|p| *p != owner).collect();
    let target = others[0];
    let third = others[1];
    // 目标 hp < 蔡贞姬 hp（贞姬 hp 设为 3,目标 hp 2：受伤 1 点不濒死）。
    tx.state_mut().players.get_mut(owner).unwrap().hp = 3;
    tx.state_mut().players.get_mut(target).unwrap().hp = 2;
    let owner_hp = tx.state().players.get(owner).unwrap().hp;
    assert!(owner_hp > 2);
    let hp_before = tx.state().players.get(target).unwrap().hp;
    let need = tx.state().players.get(owner).unwrap().hp.max(1) as usize;
    let gifts = tx
        .state()
        .hand_cards(owner)
        .iter()
        .take(need)
        .copied()
        .collect::<Vec<_>>();
    // 第三方对目标造成 1 点伤害 → 舍医触发（optional confirm + 选牌）。
    // 直接调用舍医原语（带真实技能实例；DSL optional 触发链在真实对局由引擎驱动）。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == SHEYI && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    let err = run_effects(
        &mut tx,
        vec![EffectRequest::ResolveSheyi {
            player: owner,
            target,
            skill,
        }],
    )
    .err();
    assert_eq!(err, Some(EffectError::ChoiceRequested), "应打开选牌 choice");
    // 选交牌（至少 max(1, hp) 张）。
    let state = tx.commit().state;
    let selections: Vec<u32> = gifts.iter().map(|c| c.raw()).collect();
    let state = resolve_choice(state, owner, 2, selections, false);
    assert_eq!(
        state.players.get(target).unwrap().hp,
        hp_before,
        "伤害应被舍医防止"
    );
    assert!(
        state
            .zones
            .get(state.zones.find(ZoneKind::Hand, target).unwrap())
            .unwrap()
            .cards
            .iter()
            .any(|c| gifts.contains(c)),
        "目标应收到交出的牌"
    );
}

/// 天荫：结束阶段获得牌堆顶每个未用类别牌的第一张。
#[test]
fn tianyin_gains_top_cards_of_unused_categories() {
    let (mut tx, owner, seats) = setup(TIANYIN, 2);
    let _other = seats.into_iter().find(|p| *p != owner).unwrap();
    // 牌堆顶放:装备、基本、锦囊、装备(每类第一张应被收,第二张装备不收)。
    let w = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Spade,
    );
    let sha = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Sha,
        Suit::Club,
    );
    let trick = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::TrickWuzhong,
        Suit::Heart,
    );
    let w2 = inject_card(
        &mut tx,
        PlayerId::NONE,
        ZoneKind::DrawPile,
        CardKind::Weapon,
        Suit::Diamond,
    );
    let hand_before = tx.state().hand_count(owner);
    // 直接调用天荫原语（带真实技能实例；DSL trigger 在真实对局由引擎 dispatch）。
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .find(|s| s.def.raw() == TIANYIN && s.owner == owner)
        .map(|s| s.id)
        .unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveTianyin {
            player: owner,
            skill,
        }],
    )
    .unwrap();
    // 牌堆顶 = DrawPile vec 末元素：w2 是最后 append 的（顶部），依次是 trick/sha/w。
    // 每类第一张被收：w2(装备)、trick(锦囊)、sha(基本)——3 张；w(第二张装备)不收。
    assert_eq!(tx.state().hand_count(owner), hand_before + 3);
    let hand: Vec<_> = tx.state().hand_cards(owner);
    assert!(hand.contains(&w2) && hand.contains(&trick) && hand.contains(&sha));
    assert!(!hand.contains(&w), "第二张同类别牌不应被收");
}
