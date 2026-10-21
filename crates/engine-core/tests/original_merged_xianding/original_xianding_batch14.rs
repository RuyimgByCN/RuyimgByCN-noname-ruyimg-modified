//! Golden for XIANDING（限定包）批次十四：
//! 武神（dcwushen）、递仇（dcdichou）。

use engine_api::{CardDefId, CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCWUSHEN: u32 = 3_292_885_385;
const DCDICHOU: u32 = 3_304_378_096;
const DCDICHOU_MARK: u32 = DCDICHOU;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package())
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, first, second) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(players.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().revision = 1;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    (tx, owner, first, second)
}

fn add_skill(tx: &mut Transaction, player: PlayerId, def: u32) {
    run_effects(
        tx,
        vec![EffectRequest::AddSkill {
            player,
            skill_def: SkillDefId::new(def),
        }],
    )
    .unwrap();
}

fn inject_card(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: CardDefId::new(1),
        kind,
        suit,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

fn mark_count(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCDICHOU_MARK)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch14_skills_with_expected_contracts() {
    let package = package();

    let wushen: Vec<_> = package
        .modifier_index
        .iter()
        .filter(|modifier| modifier.skill_def_id == DCWUSHEN)
        .map(|modifier| modifier.target.as_str())
        .collect();
    assert_eq!(
        wushen,
        [
            "card_name",
            "card_nature",
            "card_distance_unlimited",
            "card_use_unlimited",
        ]
    );

    let mark_trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDICHOU && !trigger.optional)
        .expect("dcdichou mark trigger");
    assert_eq!(mark_trigger.event, "card_targeted");
    assert_eq!(mark_trigger.filter, "owner_is_target_other_source_alive");

    let draw_trigger = package
        .trigger_index
        .iter()
        .find(|trigger| {
            trigger.skill_def_id == DCDICHOU
                && trigger.filter == "owner_source_target_mark_positive_3304378096"
        })
        .expect("dcdichou draw trigger");
    assert!(!draw_trigger.optional);
}

#[test]
fn dcwushen_automatically_rewrites_heart_card_to_normal_sha() {
    let (mut tx, owner, _first, _second) = setup(20_260_832);
    add_skill(&mut tx, owner, DCWUSHEN);
    let heart = inject_card(&mut tx, owner, CardKind::Jiu, Suit::Heart);
    assert!(tx.state().hand_count(owner) >= 1);
    assert_eq!(
        tx.state().cards.get(heart).map(|card| card.kind),
        Some(CardKind::Jiu)
    );
    let physical = tx.state().cards.get(heart).unwrap();
    assert_eq!(
        engine_core::query::effective_card_kind(tx.state(), owner, physical),
        CardKind::Sha
    );
    assert_eq!(
        engine_core::query::effective_card_nature(tx.state(), owner, physical, CardKind::Sha),
        rule_api::damage_nature::NORMAL
    );
}

#[test]
fn dcdichou_marks_source_when_owner_targeted_and_draws_against_marked_target() {
    let (mut tx, owner, first, _second) = setup(20_260_833);
    add_skill(&mut tx, owner, DCDICHOU);

    let card = inject_card(&mut tx, first, CardKind::Sha, Suit::Spade);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    );
    let effects = dispatched.unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, first), 1);

    let sha = inject_card(&mut tx, owner, CardKind::Sha, Suit::Club);
    let before = tx.state().hand_count(owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: owner,
            card: sha,
            targets: vec![first],
        }],
        &mut TriggerFireRecord::new(),
    );
    run_effects(&mut tx, dispatched.unwrap()).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
    assert_eq!(
        tx.state()
            .players
            .get(first)
            .unwrap()
            .marks
            .iter()
            .find(|(mark, _)| *mark == DCDICHOU_MARK)
            .map_or(0, |(_, count)| *count),
        0
    );
}
