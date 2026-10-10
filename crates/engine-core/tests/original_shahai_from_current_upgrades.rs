//! 第一百六十四批：shahai 三件「仅限其他角色」近似 upgrade 精确化。
//! gjqtwuxie / swd_wuxie（延时锦囊）与 moyao（普通锦囊）原用不分使用方的
//! card_target_cancel，现升级为 card_target_cancel_from_current（selector 参数化
//! 随本批放开 typecheck 的 any-only 限制）：其他角色使用时取消，自己对自己使用
//! 不受影响。

use engine_api::{Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, Suit};
use rule_api::{QueryRequest, QueryResponse};
use engine_core::query::{query, QueryContext};
use engine_core::state::{CardKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::Engine;

const GJQTWUXIE: u32 = 2_052_473_555;
const MOYAO: u32 = 2_399_830_529;
const SWD_WUXIE: u32 = 3_872_592_194;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/shahai/shahai.npack.json");

fn loaded(skill: u32, seed: u64) -> (Transaction, PlayerId, PlayerId) {
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
    let user = engine.state().turn.current;
    let owner = engine
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != user)
        .unwrap();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| rule_api::EffectRequest::RemoveSkill { skill: s.id })
        .chain(std::iter::once(rule_api::EffectRequest::AddSkill {
            player: owner,
            skill_def: engine_api::SkillDefId::new(skill),
        }))
        .collect();
    engine_core::effect::run_effects(&mut tx, effects).unwrap();
    (tx, user, owner)
}

fn target_legal(tx: &Transaction, user: PlayerId, card: engine_api::CardId, target: PlayerId) -> bool {
    match query(
        tx.state(),
        &QueryRequest::TargetLegal { user, card, target },
        &mut QueryContext::new(),
    ) {
        QueryResponse::Bool(value) => value,
        other => panic!("unexpected query response: {other:?}"),
    }
}

fn first_hand(tx: &Transaction, player: PlayerId) -> engine_api::CardId {
    *tx.state()
        .hand_cards(player)
        .first()
        .expect("user must hold a card")
}

fn set_kind(state: &mut GameState, card: engine_api::CardId, kind: CardKind, suit: Suit) {
    let c = state.cards.get_mut(card).unwrap();
    c.kind = kind;
    c.suit = suit;
}

#[test]
fn package_publishes_from_current_modifiers_for_the_three_upgrades() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    for (def, selector) in [
        (GJQTWUXIE, "delayed_trick"),
        (SWD_WUXIE, "delayed_trick"),
        (MOYAO, "ordinary_trick"),
    ] {
        assert!(
            package.modifier_index.iter().any(|m| {
                m.skill_def_id == def
                    && m.target == "card_target_cancel_from_current"
                    && m.selector.as_deref() == Some(selector)
            }),
            "skill {def} must publish card_target_cancel_from_current({selector})"
        );
    }
}

#[test]
fn others_delayed_tricks_cannot_target_the_owner() {
    // 乐不思蜀（延时锦囊）只能由当前回合角色主动使用——from_current 口径下
    // gjqtwuxie/swd_wuxie 取消「其他角色回合内」的指定。
    for (skill, seed) in [(GJQTWUXIE, 2_026_091_101), (SWD_WUXIE, 2_026_091_103)] {
        let (mut tx, user, owner) = loaded(skill, seed);
        let card = first_hand(&tx, user);
        set_kind(tx.state_mut(), card, CardKind::TrickLe, Suit::Spade);
        assert!(
            !target_legal(&tx, user, card, owner),
            "skill {skill}: others' delayed trick must be cancelled"
        );
    }
}

#[test]
fn others_ordinary_tricks_cannot_target_the_owner() {
    let (mut tx, user, owner) = loaded(MOYAO, 2_026_091_102);
    let card = first_hand(&tx, user);
    set_kind(tx.state_mut(), card, CardKind::TrickGuohe, Suit::Club);
    assert!(!target_legal(&tx, user, card, owner), "others' ordinary trick must be cancelled");
}

#[test]
fn without_the_skills_the_same_tricks_stay_legal() {
    // 对照组：不加载技能时同样的牌可以指定目标（证明取消来自 modifier）。
    let (mut tx, user, owner) = loaded(0, 2_026_091_104);
    let card = first_hand(&tx, user);
    set_kind(tx.state_mut(), card, CardKind::TrickGuohe, Suit::Club);
    assert!(target_legal(&tx, user, card, owner));
}
