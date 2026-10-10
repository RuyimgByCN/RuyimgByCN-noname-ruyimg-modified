//! Golden coverage for TW 李遗「教化」.

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId, Suit,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{turn_flag, Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const TWJIAOHUA: u32 = 3_924_813_285;
const TW_LIWEI: u32 = 2_988_712_070;
const SOURCE: &str = include_str!("../../../content/original/ported/tw/twjiaohua.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/tw/tw.npack.json");

fn inject(tx: &mut Transaction, zone_kind: ZoneKind, owner: PlayerId, kind: CardKind) -> CardId {
    let zone = tx.state().zones.find(zone_kind, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(90_000),
        kind,
        suit: Suit::Club,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind == ZoneKind::DiscardPile,
    });
    tx.state_mut().zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("twjiaohua-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 363);
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, 2_026_080_104, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let target = players[1];
    let other = players[2];
    tx.state_mut().players.get_mut(owner).unwrap().hp = 4;
    tx.state_mut().players.get_mut(target).unwrap().hp = 1;
    tx.state_mut().players.get_mut(other).unwrap().hp = 3;
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let removed = tx
        .state()
        .zones
        .find(ZoneKind::Removed, PlayerId::NONE)
        .unwrap();
    for kind in [ZoneKind::DrawPile, ZoneKind::DiscardPile] {
        let zone = tx.state().zones.find(kind, PlayerId::NONE).unwrap();
        let cards = tx.state().zones.get(zone).unwrap().cards.clone();
        if !cards.is_empty() {
            effects.push(EffectRequest::MoveCards {
                cards,
                to: removed,
                from: Some(zone),
            });
        }
    }
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(TWJIAOHUA),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, target)
}

fn fire(tx: &mut Transaction, target: PlayerId, cards: Vec<CardId>) -> bool {
    // 教化当前为锁定自动执行（无确认 choice）：有触发计划即视为触发。
    let effects = dispatch_for_events(
        tx,
        &[DomainEvent::CardsDrawn {
            player: target,
            cards,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    if effects.is_empty() {
        return false;
    }
    run_effects(tx, effects).is_ok()
}


#[test]
fn twjiaohua_excludes_drawn_and_turn_used_categories_then_stops() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(committed.manifest.rule_vm_abi_version >= 363);
    assert_eq!(
        committed
            .character_index
            .iter()
            .find(|character| character.id == TW_LIWEI)
            .unwrap()
            .skills,
        [TWJIAOHUA]
    );

    let (mut tx, owner, target) = setup();
    let drawn_basic = inject(&mut tx, ZoneKind::Hand, target, CardKind::Shan);
    let discard_trick = inject(
        &mut tx,
        ZoneKind::DiscardPile,
        PlayerId::NONE,
        CardKind::TrickWuzhong,
    );
    assert!(fire(&mut tx, target, vec![drawn_basic]));
    let state = tx.commit().state;
    assert_eq!(
        state.cards.get(discard_trick).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert_eq!(
        state
            .turn
            .flag(turn_flag::twjiaohua_category_by_player(target)),
        0b010
    );

    let mut tx = Transaction::begin(state);
    let draw_basic = inject(&mut tx, ZoneKind::DrawPile, PlayerId::NONE, CardKind::Sha);
    let drawn_equip = inject(&mut tx, ZoneKind::Hand, target, CardKind::Armor);
    assert!(fire(&mut tx, target, vec![drawn_equip]));
    let state = tx.commit().state;
    assert_eq!(
        state.cards.get(draw_basic).unwrap().zone,
        state.zones.find(ZoneKind::Hand, target).unwrap()
    );
    assert_eq!(
        state
            .turn
            .flag(turn_flag::twjiaohua_category_by_player(target)),
        0b011
    );
    assert!(!state.has_pending_choice());
    assert!(state.resolution.optional_trigger.is_none());
    assert!(state.resolution.deferred_effects.is_empty());
    assert!(state.resolution.deferred_triggers.is_empty());

    let mut tx = Transaction::begin(state);
    let drawn_equip = inject(&mut tx, ZoneKind::Hand, target, CardKind::Armor);
    assert!(tx.state().cards.get(drawn_equip).unwrap().kind.is_equip());
    assert_eq!(
        tx.state()
            .turn
            .flag(turn_flag::twjiaohua_category_by_player(target)),
        0b011
    );
    assert!(!fire(&mut tx, target, vec![drawn_equip]));
}
