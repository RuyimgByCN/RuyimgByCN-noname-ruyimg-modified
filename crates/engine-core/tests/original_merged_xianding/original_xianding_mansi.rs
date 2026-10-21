//! 第一百六十三批：xianding「蛮肆」（花鬘 mansi）主臂近似重写。
//! 上游：【南蛮入侵】对任意角色造成伤害后，你摸一张牌（frequent）。
//! 实现：damage + owner_alive_damage_card_nanman（本批六点注册）→ draw_cards(self,1)。
//! 近似：计数标记（纯展示）省略；viewas 子技维持债。

use engine_api::{
    CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::effect::run_effects;
use engine_core::state::{CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::{damage_nature, EffectRequest};

const MANSI: u32 = 3_673_557_022;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

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
            skill_def: SkillDefId::new(MANSI),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    tx.commit().state
}

fn deal_nanman_damage(state: GameState, source: PlayerId, target: PlayerId) -> GameState {
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            card: CardId::NONE,
            card_kind: CardKind::TrickNanman.tag(),
            amount: 1,
            is_lose_hp: false,
            nature: damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    tx.commit().state
}

fn hand_count(state: &GameState, player: PlayerId) -> u32 {
    state
        .zones
        .find(ZoneKind::Hand, player)
        .and_then(|zone| state.zones.get(zone).map(|z| z.cards.len() as u32))
        .unwrap_or(0)
}

#[test]
fn package_publishes_mansi_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == MANSI
            && trigger.event == "damage"
            && trigger.filter == "owner_alive_damage_card_nanman"
            && !trigger.optional
    }));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| modifier.skill_def_id == MANSI && modifier.target == "hand_limit"));
}

#[test]
fn mansi_draws_when_nanman_damages_anyone() {
    let (state, owner, other) = started(2_026_091_001);
    let state = strip_and_arm(state, owner);

    // 南蛮入侵对他人造成伤害（owner 仅为观察者）：蛮肆摸 1。
    let before = hand_count(&state, owner);
    let state = deal_nanman_damage(state, other, other);
    assert_eq!(
        hand_count(&state, owner),
        before + 1,
        "nanman damage anywhere draws one"
    );
}

#[test]
fn mansi_not_triggered_by_plain_damage() {
    let (state, owner, other) = started(2_026_091_002);
    let state = strip_and_arm(state, owner);

    // 无牌面上下文的普通伤害：过滤器拦截，不摸牌。
    let before = hand_count(&state, owner);
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DealShaDamage {
            source: other,
            target: other,
            card: CardId::NONE,
            amount: 1,
            nature: damage_nature::NORMAL,
            card_flags: 0,
        }],
    )
    .unwrap();
    let state = tx.commit().state;
    assert_eq!(
        hand_count(&state, owner),
        before,
        "plain sha damage must not draw"
    );
}
