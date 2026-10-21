//! Golden coverage for JSRG「乘虚」rewritten from an honest noop.

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::{card_use_unrespondable_for_kind, sha_unrespondable_for_card};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JSRGCHENGXU: u32 = 2_667_096_470;
const SOURCE: &str = include_str!("../../../../content/original/ported/jsrg/jsrgchengxu.skill");
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/jsrg/jsrg.npack.json");

fn bootstrap() -> (Transaction, PlayerId, PlayerId) {
    let compiled = content_compiler::compile_source_named("chengxu-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, u64::from(JSRGCHENGXU), "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&compiled)
        .unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JSRGCHENGXU),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn inject_hand(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(266_700 + u32::from(suit as u8)),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: true,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn source_and_npack_publish_unrespondable_modifiers() {
    let compiled = content_compiler::compile_source_named("jsrgchengxu", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JSRGCHENGXU && skill.locked));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU
            && modifier.target == "sha_unrespondable"
            && modifier.selector.as_deref() == Some("any")
    }));
    assert!(compiled.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU
            && modifier.target == "card_use_unrespondable"
            && modifier.selector.as_deref() == Some("ordinary_trick")
    }));

    let pack = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU && modifier.target == "sha_unrespondable"
    }));
    assert!(pack.modifier_index.iter().any(|modifier| {
        modifier.skill_def_id == JSRGCHENGXU && modifier.target == "card_use_unrespondable"
    }));
}

#[test]
fn owners_sha_and_ordinary_tricks_are_unrespondable() {
    let (mut tx, owner, other) = bootstrap();
    let sha = inject_hand(&mut tx, owner, CardKind::Sha, Suit::Spade);
    assert!(sha_unrespondable_for_card(tx.state(), owner, sha));
    assert!(!sha_unrespondable_for_card(tx.state(), other, sha));
    assert!(card_use_unrespondable_for_kind(
        tx.state(),
        owner,
        CardKind::TrickJuedou,
        Suit::Heart
    ));
    assert!(!card_use_unrespondable_for_kind(
        tx.state(),
        other,
        CardKind::TrickJuedou,
        Suit::Heart
    ));
}
