//! 第一百八十八批：offline「激昂」（雁翎孙策 ylyg_jiang）零新原语重写 + 全局过滤器。
//! 上游：任意角色使用【决斗】或红色【杀】指定目标后——使用者是你可以摸两张，
//! 否则你与其各摸一张。实现：card_used + owner_alive_(other_)global_juedou_or_
//! red_sha_used 双过滤器（5 点注册零 ABI，sbjiang 过滤器去 owner 介入版）。
//! 近似（登记债）：应答窗口与点数 13 当杀 mod 未建模。

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const YLYG_JIANG: u32 = 4_118_108_424;
const YLYG_SUNCE: u32 = 3_809_908_961;
const SOURCE: &str = include_str!("../../../content/original/ported/offline/ylyg_jiang.skill");
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

fn setup(seed: u64) -> (GameState, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("ylyg-jiang-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let other = players[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(YLYG_JIANG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, owner, other)
}

fn alloc_card(state: &mut GameState, kind: CardKind, suit: Suit) -> CardId {
    let zone = state.zones.find(ZoneKind::Processing, PlayerId::NONE).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(9_999),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner: PlayerId::NONE,
        face_up: true,
    });
    state
        .zones
        .get_mut(zone)
        .unwrap()
        .cards
        .push(card);
    card
}

fn card_used(state: GameState, user: PlayerId, kind: CardKind, suit: Suit) -> (GameState, usize) {
    let mut tx = Transaction::begin(state);
    let card = alloc_card(tx.state_mut(), kind, suit);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: user,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    let n = effects.len();
    run_effects(&mut tx, effects).unwrap();
    (tx.commit().state, n)
}

#[test]
fn package_publishes_ylyg_jiang_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 682);
    let arms: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|t| t.skill_def_id == YLYG_JIANG)
        .collect();
    assert_eq!(arms.len(), 2, "self + other global arms");
    assert!(
        arms.iter()
            .any(|t| t.filter == "owner_alive_global_juedou_or_red_sha_used")
    );
    assert!(
        arms.iter()
            .any(|t| t.filter == "owner_alive_other_global_juedou_or_red_sha_used")
    );
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == YLYG_SUNCE)
        .expect("ylyg_sunce in roster");
    assert!(character.skills.contains(&YLYG_JIANG));
}

#[test]
fn own_juedou_draws_two_and_other_juedou_draws_one_each() {
    let (state, owner, other) = setup(20_260_912_01);
    let hand_before = state.hand_count(owner);

    // 自己使用【决斗】：摸 2。
    let (state, _) = card_used(state, owner, CardKind::TrickJuedou, Suit::Spade);
    assert_eq!(state.hand_count(owner), hand_before + 2, "own juedou: draw 2");

    // 他人使用红【杀】：各摸 1（owner 净 +1）。
    let (state, n) = card_used(state, other, CardKind::Sha, Suit::Heart);
    assert!(n > 0, "other red sha must fire");
    assert_eq!(state.hand_count(owner), hand_before + 3, "each draws one");

    // 他人使用黑【杀】：不触发。
    let (state, n) = card_used(state, other, CardKind::Sha, Suit::Spade);
    assert_eq!(n, 0, "black sha must not fire");
    assert_eq!(state.hand_count(owner), hand_before + 3);
}
