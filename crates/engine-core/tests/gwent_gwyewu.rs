//! Gwent 叶舞 (gwyewu) random-discard chain — ADR 0627, ABI 645.
//!
//! 叶舞：弃置一张手牌（费用），然后按种子 RNG 循环——随机敌人随机弃其一张「he」牌，
//! 颜色与费用相同则重复；每以此法弃敌方手牌得一张雾妖（784509644），每弃敌方装备牌
//! 得一张狼（2836710318）。雾妖/狼以技能近似（add_skill，造卡入口）。
//!
//! 确定性：测试通过「候选长度=1 / 牌长=1」迫使 RNG 选点唯一；重复分支用「两牌同色
//! 且与费用同色」迫使两轮后候选耗尽，均与 RNG 具体取值无关。

use engine_api::{
    CardDefId, CardId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const WUYAO_DEF: u32 = 784509644;
const LANG_DEF: u32 = 2836710318;

fn inject_hand_card(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let zone = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit,
        rank: 5,
        weapon_range: 0,
        zone,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn clear_hand(tx: &mut Transaction, player: PlayerId) {
    let cards = tx.state().hand_cards(player);
    if !cards.is_empty() {
        run_effects(tx, vec![EffectRequest::DiscardCards { player, cards }]).unwrap();
    }
}

fn boot(seed: u64) -> (PlayerId, PlayerId, Transaction) {
    let source = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../content/original/ported/shahai/shahai_impl.skill"
    ))
    .unwrap();
    let pkg = content_compiler::compile_source_named("shahai_impl", &source).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&pkg).unwrap();
    let start = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(seed),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: seed,
            ruleset: "guozhan-standard".into(),
        },
    });
    assert!(matches!(start, CommandResult::Accepted { .. }));
    let actor = engine.state().turn.current;
    let other = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != actor)
        .expect("second player");
    (actor, other, Transaction::begin(engine.state().clone()))
}

fn has_skill_def(tx: &Transaction, player: PlayerId, def: u32) -> bool {
    tx.state()
        .players
        .get(player)
        .into_iter()
        .flat_map(|p| p.skills.iter())
        .any(|sid| {
            tx.state()
                .skills
                .get(*sid)
                .is_some_and(|s| s.def.raw() == def)
        })
}

#[test]
fn gwyewu_color_mismatch_discards_one_enemy_hand_card_and_grants_wuyao() {
    let (actor, other, mut tx) = boot(1001);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 费用：红桃（红）。敌方唯一一张手牌：黑桃（黑）→ 颜色不同，循环一轮即止。
    let cost = inject_hand_card(&mut tx, actor, CardKind::Sha, Suit::Heart);
    inject_hand_card(&mut tx, other, CardKind::Shan, Suit::Spade);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveGwyewuChain {
            player: actor,
            skill: engine_api::SkillId::NONE,
            cost_cards: vec![cost],
            wuyao_def: engine_api::SkillDefId::new(WUYAO_DEF),
            lang_def: engine_api::SkillDefId::new(LANG_DEF),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(actor), 0, "cost card discarded");
    assert_eq!(tx.state().hand_count(other), 0, "enemy hand card discarded");
    assert!(
        has_skill_def(&tx, actor, WUYAO_DEF),
        "gained one 雾妖 skill for the enemy hand discard"
    );
    assert!(
        !has_skill_def(&tx, actor, LANG_DEF),
        "no 狼 (equip discard) granted"
    );
}

#[test]
fn gwyewu_color_match_repeats_until_enemy_exhausted() {
    let (actor, other, mut tx) = boot(2002);
    tx.state_mut().group_overrides.insert(actor, "wei".into());
    tx.state_mut().group_overrides.insert(other, "shu".into());
    clear_hand(&mut tx, actor);
    clear_hand(&mut tx, other);
    // 费用：黑桃（黑）。敌方两张手牌均为黑桃（黑）→ 两轮同色，第二轮后候选耗尽。
    let cost = inject_hand_card(&mut tx, actor, CardKind::Sha, Suit::Spade);
    inject_hand_card(&mut tx, other, CardKind::Shan, Suit::Spade);
    inject_hand_card(&mut tx, other, CardKind::Jiu, Suit::Club);
    run_effects(
        &mut tx,
        vec![EffectRequest::ResolveGwyewuChain {
            player: actor,
            skill: engine_api::SkillId::NONE,
            cost_cards: vec![cost],
            wuyao_def: engine_api::SkillDefId::new(WUYAO_DEF),
            lang_def: engine_api::SkillDefId::new(LANG_DEF),
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(actor), 0, "cost card discarded");
    assert_eq!(
        tx.state().hand_count(other),
        0,
        "both enemy hand cards discarded by the repeat"
    );
    assert!(
        has_skill_def(&tx, actor, WUYAO_DEF),
        "gained 雾妖 after the repeated discards"
    );
}
