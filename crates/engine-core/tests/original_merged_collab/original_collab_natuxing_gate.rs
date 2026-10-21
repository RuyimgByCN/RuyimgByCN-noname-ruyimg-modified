//! §8 S1（第一百一十八批）：`owner_source_hand_gaintag_absent_<tag>` 参数化
//! 过滤器——natuxing 兴门：手中无锁定兴 tag 牌时用牌不可被响应。
//! 兴 tag = blake2b("natuxing") = 297116519。

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{Card, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NATUXING: u32 = 4_091_854_877;
const XING_TAG: u32 = 297_116_519;

const SOURCES_DIR: &str = concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../content/original/ported/collab"
);

fn sources() -> String {
    let dir = std::path::Path::new(SOURCES_DIR);
    let mut files: Vec<_> = std::fs::read_dir(dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().map_or(false, |ext| ext == "skill"))
        .map(|e| e.path())
        .collect();
    files.sort();
    files
        .iter()
        .map(|p| {
            let mut s = std::fs::read_to_string(p).unwrap();
            s.push('\n');
            s
        })
        .collect()
}

fn drive(tx: &mut Transaction, events: &[engine_api::DomainEvent]) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        events,
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

fn started() -> (GameState, PlayerId) {
    let compiled = content_compiler::compile_source_named("collab-natuxing", &sources()).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_838, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    let owner = tx.state().turn.current;
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|s| EffectRequest::RemoveSkill { skill: s.id })
        .collect();
    run_effects(&mut tx, effects).unwrap();
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NATUXING),
        }],
    )
    .unwrap();
    (tx.commit().state, owner)
}

fn inject_card(tx: &mut Transaction, owner: PlayerId, tag: Option<u32>) -> engine_api::CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: engine_core::state::CardKind::Tao,
        suit: engine_api::Suit::Heart,
        rank: 5,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    if let Some(tag) = tag {
        tx.state_mut()
            .content_card_tags
            .entry(card)
            .or_default()
            .insert(tag);
    }
    card
}

fn drive_use_card(tx: &mut Transaction, holder: PlayerId, card: CardId) -> Result<(), EffectError> {
    match engine_core::trigger::dispatch_for_events(
        tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card,
            targets: vec![],
        }],
        &mut engine_core::trigger::TriggerFireRecord::new(),
    ) {
        Err(e) => Err(e),
        Ok(effects) => run_effects(tx, effects).map(|_| ()),
    }
}

#[test]
fn natuxing_unrespondable_gated_by_gaintag_absent() {
    let (state, holder) = started();
    let mut tx = Transaction::begin(state);

    // 手牌含兴 tag 牌 → 门关闭（无 unrespondable 效果）。
    let tagged = inject_card(&mut tx, holder, Some(XING_TAG));
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card: tagged,
            targets: vec![],
        }],
    );
    opened.unwrap();
    assert!(!tx.state().has_pending_choice(), "gate closed while tagged");
    let mut committed = tx.commit();

    // 手牌无兴 tag 牌 → 门开（不可被响应效果）。
    // 先移除场景一的兴 tag 牌（模拟其已被使用消耗）。
    {
        let hand = committed.state.zones.find(ZoneKind::Hand, holder).unwrap();
        committed
            .state
            .zones
            .get_mut(hand)
            .unwrap()
            .cards
            .retain(|&c| c != tagged);
    }
    let mut tx = Transaction::begin(committed.state);
    let plain = inject_card(&mut tx, holder, None);
    let opened = drive(
        &mut tx,
        &[engine_api::DomainEvent::CardUsed {
            player: holder,
            card: plain,
            targets: vec![],
        }],
    );
    opened.unwrap();
    let committed2 = tx.commit();
    let sid2 = committed2
        .state
        .skills
        .sorted_live()
        .iter()
        .find(|s| s.def.raw() == NATUXING)
        .unwrap()
        .id;
    let policy_armed = committed2
        .state
        .skills
        .sorted_live()
        .iter()
        .filter(|s| s.def.raw() == NATUXING)
        .any(|s| s.card_use_policy(plain) & 2 != 0);
    assert!(policy_armed, "gate open: unrespondable armed");
}
