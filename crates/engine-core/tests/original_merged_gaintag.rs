//! O2 全量合并第二批（2026-10-14）：家族 gaintag 的 1 个独立测试文件逐字包裹为模块（小家族/名称冲突/注册表冲突全部纳入——mod 隔离下无碰撞）。证据：docs/operations/evidence/2026-10-14-o2-full-merge-batch1/README.md。

mod original_gaintag_primitive {
//! Coverage for the gaintag Host primitive cluster: `tag_event_cards`/`untag_event_cards`
//! effects and the `card_has_tag` query. These are the foundation primitives for clan skills
//! that mark cards (clanbaichu/clanjianbai/clanqingjue/clanshenjun/clanxsyingxiang/clanyuzhi),
//! which previously shipped as INTENTIONAL_EMPTY_NOOP awaiting this primitive.

use engine_api::{
    CardDefId, CardId, Command, CommandKind, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, Suit,
};
use engine_core::effect::run_effects;
use engine_core::query::{query, QueryContext};
use engine_core::state::{Card, CardKind, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::{EffectRequest, QueryRequest, QueryResponse};

/// A tiny skill that tags the used card with itself, then (in a second trigger) reports whether
/// a card carries that tag. The def id is reused as the tag value (matching the engine's
/// existing SHEQUAN_CARD_TAG/POTFUJI_CARD_TAG convention).
const TAGGER: u32 = 3_777_777_777;
const SOURCE: &str = concat!(
    // On card use, attach this skill's def-id tag to the event card.
    "skill gaintag_tagger(",
    "3777777777",
    ") locked {\n",
    "  trigger card_used always {\n",
    "    effect tag_event_cards(self_skill);\n",
    "  }\n",
    "}\n",
);

#[test]
fn tag_event_cards_compiles_and_publishes_trigger() {
    let pkg = content_compiler::compile_source(SOURCE).unwrap();
    assert!(pkg.trigger_index.iter().any(|trigger| {
        trigger.skill_def_id == TAGGER && trigger.event == "card_used"
    }));
}

#[test]
fn tag_card_then_query_reports_membership_and_untag_clears_it() {
    let pkg = content_compiler::compile_source(SOURCE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&pkg).unwrap();
    engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_260_809,
            ruleset: "identity-standard".into(),
        },
    });
    let owner = engine.state().turn.current;

    let mut tx = Transaction::begin(engine.state().clone());
    // Strip defaults and grant the tagger skill to the owner.
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TAGGER),
    });
    run_effects(&mut tx, effects).unwrap();

    // Resolve the live tagger instance + its def id (the tag value).
    let skill = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == TAGGER)
        .unwrap()
        .id;
    let tag_def = TAGGER;

    // Inject a hand card for the owner, then dispatch a CardUsed event → tagger tags it.
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(170_001),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card,
            targets: vec![],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, fired).unwrap();

    // The card now carries the tag: query via the live skill instance (resolves to the def id).
    let resp = query(
        tx.state(),
        &QueryRequest::CardHasTag { card, tag: skill.raw() },
        &mut QueryContext::new(),
    );
    assert!(matches!(resp, QueryResponse::Bool(true)), "tag should be set: {resp:?}");
    // And the lower-level state agrees.
    assert!(tx.state().card_has_content_tag(card, tag_def));

    // Untag via the primitive directly and confirm the query flips to false.
    run_effects(
        &mut tx,
        vec![EffectRequest::UntagCard {
            cards: vec![card],
            tag: tag_def,
        }],
    )
    .unwrap();
    let resp2 = query(
        tx.state(),
        &QueryRequest::CardHasTag { card, tag: skill.raw() },
        &mut QueryContext::new(),
    );
    assert!(matches!(resp2, QueryResponse::Bool(false)), "tag should be cleared: {resp2:?}");
    assert!(!tx.state().card_has_content_tag(card, tag_def));
}
}
