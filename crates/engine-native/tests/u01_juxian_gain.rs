//! 聚贤置顶分支：真实发动路径中的私有手牌/公开装备选择。
//! 预期：archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:295-338。
use std::collections::BTreeSet;
use std::path::PathBuf;

use content_native_loader::{bridge_pack, decision34_pack9_for_coexistence};
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::rng::RuleRng;
use engine_native::snap::MachineSnapshot;
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, Phase};

fn packs() -> (ExtensionPack, ExtensionPack) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/d3-upstream-noname/assets/character.json");
    let text = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    let d3 = bridge_pack(&text).unwrap().pack;
    let p9 = decision34_pack9_for_coexistence(&[&d3]);
    (d3, p9)
}

fn command(e: &mut Engine, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1_000_000,
        action,
    })
}

fn submit(e: &mut Engine, action: GameAction) {
    let out = command(e, action);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}

fn answer(e: &mut Engine, selection: Vec<O>) {
    let c = e.open_choice().unwrap().clone();
    submit(
        e,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
}

fn give(e: &mut Engine, seat: u8, card: u32) {
    e.state_core_mut().deck.retain(|&id| id != card);
    e.state_core_mut().discard.retain(|&id| id != card);
    for p in &mut e.state_core_mut().players {
        p.hand.retain(|&id| id != card);
        p.equips.retain(|&id| id != card);
    }
    e.state_core_mut().player_mut(seat).hand.push(card);
}

fn fixture(packs: &[&ExtensionPack]) -> (Engine, [u32; 4]) {
    let mut e = Engine::new_with_extensions(2026111301, 8, packs).unwrap();
    e.state_core_mut().skills.clear();
    for p in 0..8 {
        e.debug_clear_hand(p);
    }
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, "dddjuxian");
    let find = |kind, suit| {
        e.catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(id, d)| *id != 0 && d.kind == kind && d.suit == suit)
            .unwrap()
            .0 as u32
    };
    let paid = find(CardKind::Sha, Suit::Spade);
    let black = find(CardKind::Sha, Suit::Club);
    let red = find(CardKind::Tao, Suit::Heart);
    let equip = find(CardKind::HorseMinus, Suit::Spade);
    give(&mut e, 0, paid);
    give(&mut e, 2, black);
    give(&mut e, 2, red);
    give(&mut e, 2, equip);
    e.state_core_mut()
        .player_mut(2)
        .hand
        .retain(|&id| id != equip);
    e.state_core_mut().player_mut(2).equips.push(equip);
    (e, [paid, black, red, equip])
}

fn open_gain(e: &mut Engine, paid: u32) {
    let skill = e.catalog().skill_by_name("dddjuxian").unwrap();
    submit(e, GameAction::ActivateSkill { skill });
    answer(e, vec![O::Tag(0)]);
    answer(e, vec![O::Card(paid)]);
    answer(e, vec![O::Player(2)]);
    assert_eq!(
        e.open_choice()
            .expect("gainPlayerCard must ask the owner")
            .spec
            .prompt,
        "dddjuxian_gain"
    );
}

fn flag(e: &Engine, key: &str) -> u32 {
    e.state_core()
        .vm_aux
        .flags
        .get(&(0, key.to_string()))
        .copied()
        .unwrap_or(0)
}

#[test]
fn juxian_gain_selects_each_hidden_card_or_public_equipment_and_resumes_archive() {
    let (d3, p9) = packs();
    let packs = [&d3, &p9];
    // Upstream :329-338: a forced gainPlayerCard chooses any he card;
    // only the chosen card's different color prohibits another activation.
    let mut hidden_received = BTreeSet::new();
    for branch in 0..3 {
        let (mut e, [paid, black, red, equip]) = fixture(&packs);
        open_gain(&mut e, paid);
        let c = e.open_choice().unwrap().clone();
        assert_eq!(c.observer, 0);
        assert_eq!(
            c.spec.options,
            vec![O::HiddenCard(0), O::HiddenCard(1), O::Card(equip)]
        );
        assert_eq!((c.spec.min, c.spec.max, c.spec.allow_cancel), (1, 1, false));
        assert_eq!(e.state_core().deck.last(), Some(&paid));
        assert!(e.state_core().player(0).hand.is_empty());

        // Neither cancellation nor a guessed foreign physical ID is legal.
        for selection in [vec![], vec![O::Card(black)], vec![O::HiddenCard(2)]] {
            let before = e.state_hash();
            assert!(matches!(
                command(
                    &mut e,
                    GameAction::Answer {
                        choice_id: c.id,
                        selection
                    }
                ),
                CommandOutcome::Rejected(_)
            ));
            assert_eq!(e.state_hash(), before);
        }

        let bytes = encode_archive(&e.save());
        let mut restored =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &packs).unwrap();
        assert_eq!(e.state_hash(), restored.state_hash());
        let selection = match branch {
            0 => O::HiddenCard(0),
            1 => O::HiddenCard(1),
            _ => O::Card(equip),
        };
        let log_begin = e.log().len();
        answer(&mut e, vec![selection]);
        answer(&mut restored, vec![selection]);
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(&e.log()[log_begin..], restored.log());
        assert!(e.open_choice().is_none());
        assert_eq!(e.state_core().player(0).hand.len(), 1);
        let chosen = e.state_core().player(0).hand[0];
        if branch < 2 {
            assert!([black, red].contains(&chosen));
            assert!(
                hidden_received.insert(chosen),
                "the two hidden slots must gain different cards"
            );
        } else {
            assert_eq!(chosen, equip);
        }
        assert_eq!(e.state_core().player(0).hand, vec![chosen]);
        assert!(!e.state_core().player(2).hand.contains(&chosen));
        assert!(!e.state_core().player(2).equips.contains(&chosen));
        assert_eq!(flag(&e, "dddjuxian_ban"), u32::from(chosen == red));
        assert_eq!(flag(&e, "dddjuxian_used"), 1);
    }
}

#[test]
fn juxian_top_mode_requires_other_he_and_mode_cancel_keeps_payment() {
    let (d3, p9) = packs();
    let (mut e, [paid, ..]) = fixture(&[&d3, &p9]);
    e.debug_clear_hand(2);
    let equip = std::mem::take(&mut e.state_core_mut().player_mut(2).equips);
    e.state_core_mut().discard.extend(equip);
    let skill = e.catalog().skill_by_name("dddjuxian").unwrap();
    submit(&mut e, GameAction::ActivateSkill { skill });
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.options, vec![O::Tag(1)]);
    assert!(matches!(
        command(
            &mut e,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![O::Tag(0)]
            }
        ),
        CommandOutcome::Rejected(_)
    ));
    answer(&mut e, vec![]);
    assert_eq!(e.state_core().player(0).hand, vec![paid]);
    assert_eq!(flag(&e, "dddjuxian_used"), 0);
    assert!(e.open_choice().is_none());
}

#[test]
fn juxian_gain_planner_does_not_inspect_foreign_hand_faces() {
    let (d3, p9) = packs();
    let packs = [&d3, &p9];
    let (mut e, [paid, ..]) = fixture(&packs);
    open_gain(&mut e, paid);
    let mut swapped = Engine::restore_with_extensions(&e.save(), &packs).unwrap();
    swapped.state_core_mut().player_mut(2).hand.swap(0, 1);
    let planner = BoundedPlanner::new(8);
    let first = e.plan_action(0, &planner);
    let second = swapped.plan_action(0, &planner);
    assert_eq!(first.action, second.action);
    assert!(
        matches!(first.action, GameAction::Answer { ref selection, .. }
        if e.open_choice().unwrap().spec.accepts(selection))
    );
}

#[test]
fn juxian_hidden_order_uses_rng_is_archived_and_rejects_invalid_mapping() {
    let (d3, p9) = packs();
    let packs = [&d3, &p9];
    let mut first_cards = BTreeSet::new();
    // Upstream core/noname/library/element/content.ts:8962 shuffles hs.
    // An opaque slot must not consistently identify a recently given card.
    for seed in 1..=8 {
        let (mut e, [paid, black, red, _]) = fixture(&packs);
        e.state_core_mut().rng = RuleRng::from_seed(seed);
        open_gain(&mut e, paid);
        assert!(
            e.state_core().rng.calls() > 0,
            "hidden candidates must use rule RNG"
        );
        let snapshot = e.save();
        let private_cards = snapshot
            .resolution
            .parks
            .iter()
            .find_map(|park| match &park.machine {
                MachineSnapshot::Host { def_key, cards, .. } if def_key == "dddjuxian" => {
                    Some(cards.clone())
                }
                _ => None,
            })
            .expect("pending juxian machine");
        assert_eq!(private_cards.len(), 3);
        assert_eq!(private_cards[0], paid);
        assert_eq!(
            private_cards[1..].iter().copied().collect::<BTreeSet<_>>(),
            BTreeSet::from([black, red])
        );
        first_cards.insert(private_cards[1]);
        let restored = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
        assert_eq!(restored.save().resolution.parks, snapshot.resolution.parks);
        assert_eq!(restored.state_core().rng, e.state_core().rng);

        let mut corrupt = snapshot;
        for park in &mut corrupt.resolution.parks {
            if let MachineSnapshot::Host { def_key, cards, .. } = &mut park.machine {
                if def_key == "dddjuxian" {
                    cards[2] = cards[1];
                }
            }
        }
        assert!(Engine::restore_with_extensions(&corrupt, &packs).is_err());
    }
    assert_eq!(
        first_cards.len(),
        2,
        "different seeds must not expose one stable hand order"
    );
}
