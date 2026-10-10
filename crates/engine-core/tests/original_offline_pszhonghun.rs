//! Golden for OFFLINE ★关羽「忠魂」的牌堆顶印刷颜色语义。

use engine_api::{DomainEvent, PlayerId, Suit, RULE_VM_ABI_VERSION};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, ZoneKind};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PS_GUANYU: u32 = 2_687_854_414;
const PSZHONGHUN: u32 = 4_055_252_724;
const OFFLINE: &[u8] =
    include_bytes!("../../../content/original/ported/offline/offline.npack.json");

#[test]
fn package_closes_ps_guanyu_and_top_card_only_enters_hand_on_matching_color() {
    let package = content_schema::ContentPackage::from_json_bytes(OFFLINE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.character_index.iter().any(|character| {
        character.id == PS_GUANYU && character.skills == [16, PSZHONGHUN, 2_974_830_679]
    }));
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == PSZHONGHUN)
        .collect();
    assert_eq!(triggers.len(), 2);
    assert!(triggers
        .iter()
        .all(|trigger| trigger.filter == "owner_source_red" && trigger.optional));

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, PSZHONGHUN as u64, "identity-standard").unwrap();
    let owner = tx.state().players.alive_by_seat()[0];
    let draw = tx
        .state()
        .zones
        .find(ZoneKind::DrawPile, PlayerId::NONE)
        .unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let red = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(red).unwrap().suit = Suit::Heart;
    let hand = tx.state().hand_count(owner);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::RevealTopGainIfColor {
            player: owner,
            color_mask: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand + 1);
    assert!(events.iter().any(|event| {
        matches!(event, DomainEvent::CardsRevealed { cards, to } if cards == &[red] && to.is_empty())
    }));

    let black = *tx.state().zones.get(draw).unwrap().cards.last().unwrap();
    tx.state_mut().cards.get_mut(black).unwrap().suit = Suit::Spade;
    let hand = tx.state().hand_count(owner);
    run_effects(
        &mut tx,
        vec![EffectRequest::RevealTopGainIfColor {
            player: owner,
            color_mask: 1,
        }],
    )
    .unwrap();
    assert_eq!(tx.state().hand_count(owner), hand);
    assert_eq!(tx.state().cards.get(black).unwrap().zone, discard);
}
