//! Golden coverage for COLLAB「亥猪」rewritten from legacy JavaScript semantics.

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit, TurnPhaseTag};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const OLHAIZHU: u32 = 2_513_379_804;
const OL_HAIZHU: u32 = 3_508_419_110;
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/collab/collab.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    others: Vec<PlayerId>,
}

fn setup(seed: u64) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let owner = tx.state().turn.current;
    let others = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .filter(|player| *player != owner)
        .collect();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(OLHAIZHU),
    });
    run_effects(&mut tx, effects).unwrap();
    Setup {
        state: tx.commit().state,
        owner,
        others,
    }
}

fn inject(
    state: &mut GameState,
    owner: PlayerId,
    zone_kind: ZoneKind,
    def: u32,
    suit: Suit,
) -> CardId {
    let zone = state.zones.find(zone_kind, owner).unwrap();
    let card = state.cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(def),
        kind: CardKind::Sha,
        suit,
        rank: 7,
        weapon_range: 0,
        zone,
        owner,
        face_up: zone_kind != ZoneKind::Hand,
    });
    state.zones.get_mut(zone).unwrap().cards.push(card);
    card
}

fn dispatch(tx: &mut Transaction, events: &[DomainEvent]) {
    let effects = dispatch_for_events(tx, events, &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_retains_abi112_skill_and_character_closure() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 694);
    assert_eq!(package.skill_index.len(), 152);
    assert_eq!(package.character_index.len(), 88);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == OLHAIZHU)
        .collect();
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "cards_moved"
            && trigger.filter == "owner_alive_other_discarded_black_to_discard"
    }));
    assert!(triggers.iter().any(|trigger| {
        trigger.event == "phase_prepare" && trigger.filter == "owner_is_target_tied_max_hand"
    }));
    assert!(package.character_index.iter().any(|character| {
        character.id == OL_HAIZHU
            && character.name_key == "ol_haizhu"
            && character.sex == "male"
            && character.group == "qun"
            && character.max_hp == 5
            && character.skills == [OLHAIZHU]
    }));
}

#[test]
fn gains_only_other_players_black_cards_from_the_exact_discard_batch() {
    let mut setup = setup(2_026_072_312);
    let other = setup.others[0];
    let black = inject(&mut setup.state, other, ZoneKind::Hand, 99_201, Suit::Spade);
    let red = inject(&mut setup.state, other, ZoneKind::Hand, 99_202, Suit::Heart);
    let mut tx = Transaction::begin(setup.state);
    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: other,
            cards: vec![black, red],
        }],
    )
    .unwrap();
    dispatch(&mut tx, &events);

    let owner_hand = tx.state().zones.find(ZoneKind::Hand, setup.owner).unwrap();
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();
    assert_eq!(tx.state().cards.get(black).unwrap().zone, owner_hand);
    assert_eq!(tx.state().cards.get(black).unwrap().owner, setup.owner);
    assert_eq!(tx.state().cards.get(red).unwrap().zone, discard);
}

#[test]
fn own_discards_and_non_discard_movements_are_not_reclaimed() {
    let mut setup = setup(2_026_072_313);
    let own_black = inject(
        &mut setup.state,
        setup.owner,
        ZoneKind::Hand,
        99_211,
        Suit::Club,
    );
    let other_black = inject(
        &mut setup.state,
        setup.others[0],
        ZoneKind::Hand,
        99_212,
        Suit::Spade,
    );
    let mut tx = Transaction::begin(setup.state);
    let discard = tx
        .state()
        .zones
        .find(ZoneKind::DiscardPile, PlayerId::NONE)
        .unwrap();

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::DiscardCards {
            player: setup.owner,
            cards: vec![own_black],
        }],
    )
    .unwrap();
    dispatch(&mut tx, &events);
    assert_eq!(tx.state().cards.get(own_black).unwrap().zone, discard);

    let events = run_effects(
        &mut tx,
        vec![EffectRequest::MoveCards {
            cards: vec![other_black],
            to: discard,
            from: None,
        }],
    )
    .unwrap();
    dispatch(&mut tx, &events);
    assert_eq!(tx.state().cards.get(other_black).unwrap().zone, discard);
}

#[test]
fn preparation_hp_loss_counts_ties_but_not_a_smaller_hand() {
    let tied = setup(2_026_072_314);
    let before = tied.state.players.get(tied.owner).unwrap().hp;
    let mut tx = Transaction::begin(tied.state);
    dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: tied.owner,
            phase: TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(tx.state().players.get(tied.owner).unwrap().hp, before - 1);

    let mut setup = setup(2_026_072_315);
    let other = setup.others[0];
    let needed = setup
        .state
        .hand_count(setup.owner)
        .saturating_sub(setup.state.hand_count(other))
        + 1;
    for offset in 0..needed {
        inject(
            &mut setup.state,
            other,
            ZoneKind::Hand,
            99_221 + offset,
            Suit::Heart,
        );
    }
    let before = setup.state.players.get(setup.owner).unwrap().hp;
    let mut tx = Transaction::begin(setup.state);
    dispatch(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: setup.owner,
            phase: TurnPhaseTag::Prepare,
        }],
    );
    assert_eq!(tx.state().players.get(setup.owner).unwrap().hp, before);
}
