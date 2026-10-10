//! Golden coverage for REFRESH「界武圣」的红牌转化与方片【杀】距离规则。

use engine_api::{CardDefId, CardId, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::query::card_distance_unlimited_for_card;
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const NEW_REWUSHENG: u32 = 3_038_912_803;
const SOURCE: &str = include_str!("../../../content/original/ported/refresh/new_rewusheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/refresh/refresh.npack.json");

fn inject(tx: &mut Transaction, owner: PlayerId, kind: CardKind, suit: Suit, def: u32) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, owner).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn new_rewusheng_converts_red_personal_cards_and_only_diamond_sha_ignores_distance() {
    let compiled = content_compiler::compile_source_named("new-rewusheng-test", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 335);
    for window in ["play_phase", "response"] {
        assert!(compiled.view_as_index.iter().any(|view| {
            view.skill_def_id == NEW_REWUSHENG
                && view.as_kind == "sha"
                && view.window == window
                && view.filter == "red"
                && view.source_zone.as_deref() == Some("hand_equip_expansion")
        }));
    }

    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&compiled).unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, NEW_REWUSHENG as u64, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(NEW_REWUSHENG),
        }],
    )
    .unwrap();
    let diamond_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Diamond, 335_001);
    let heart_sha = inject(&mut tx, owner, CardKind::Sha, Suit::Heart, 335_002);
    let diamond_trick = inject(
        &mut tx,
        owner,
        CardKind::TrickJuedou,
        Suit::Diamond,
        335_003,
    );
    assert!(card_distance_unlimited_for_card(
        tx.state(),
        owner,
        diamond_sha
    ));
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        heart_sha
    ));
    assert!(!card_distance_unlimited_for_card(
        tx.state(),
        owner,
        diamond_trick
    ));

    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 723);
    assert!(package
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == NEW_REWUSHENG));
}
