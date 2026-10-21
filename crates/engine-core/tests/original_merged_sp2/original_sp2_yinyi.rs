//! 第一百六十批：sp2「隐翼」（yinyi）重写——新过滤器 + 既有 prevent 原语。
//! 上游：锁定技，每回合限一次，当你受到伤害时，若来源体力值与你不同且其手牌数
//! 与你不同，防止此伤害。实现：damage_before +
//! owner_targeted_source_hp_hand_differ_once_per_turn（本批六点注册）→
//! prevent_current_damage()。
//! 近似：上游 linked-nature 排除（铁索传导）未建模。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const YINYI: u32 = 3_441_826_579;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp2/sp2.npack.json");

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
    let source = state
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    (state, owner, source)
}

fn strip_and_setup(state: GameState, owner: PlayerId) -> GameState {
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
            skill_def: SkillDefId::new(YINYI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn deal_damage(state: GameState, source: PlayerId, target: PlayerId, amount: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source,
            target,
            card: CardId::NONE,
            amount,
            nature: damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn hp_of(state: &GameState, player: PlayerId) -> i32 {
    state.players.get(player).unwrap().hp
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

#[test]
fn package_publishes_yinyi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == YINYI
            && trigger.event == "damage_before"
            && trigger.filter == "owner_targeted_source_hp_hand_differ_once_per_turn"
            && !trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == YINYI && modifier.target == "hand_limit"));
}

#[test]
fn yinyi_prevents_first_differing_damage_once_per_turn() {
    let (state, owner, source) = started(2_026_090_701);
    let state = strip_and_setup(state, owner);
    let owner_hp = hp_of(&state, owner);

    // 体力与手牌均不同（source 手牌 0、owner 也 0？——先保证手牌不同：给 owner 一张）。
    let mut tx = Transaction::begin(state);
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(98_601),
        kind: CardKind::Tao,
        suit: engine_api::Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    let state = tx.commit().state;
    assert_ne!(
        hp_of(&state, owner),
        hp_of(&state, source),
        "bootstrap hp differs"
    );
    assert_ne!(hand_count(&state, owner), hand_count(&state, source));

    // 第一次伤害：防止（hp 不变，锁定技无确认）。
    let state = deal_damage(state, source, owner, 1);
    assert_eq!(
        hp_of(&state, owner),
        owner_hp,
        "first differing damage prevented"
    );

    // 第二次伤害：每回合限一次，照常结算。
    let state = deal_damage(state, source, owner, 1);
    assert_eq!(
        hp_of(&state, owner),
        owner_hp - 1,
        "second damage lands (once per turn)"
    );
}

#[test]
fn yinyi_not_triggered_when_hand_counts_equal() {
    let (state, owner, source) = started(2_026_090_702);
    let state = strip_and_setup(state, owner);
    let owner_hp = hp_of(&state, owner);
    // 双方手牌均空（相等）→ 过滤不过，伤害照常。
    assert_eq!(hand_count(&state, owner), 0);
    assert_eq!(hand_count(&state, source), 0);
    let state = deal_damage(state, source, owner, 1);
    assert_eq!(
        hp_of(&state, owner),
        owner_hp - 1,
        "equal hand counts bypass yinyi"
    );
}
