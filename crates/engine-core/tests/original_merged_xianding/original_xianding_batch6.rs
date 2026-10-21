//! Golden for XIANDING（限定包）批次六：
//! 沉勇（dcchenyong）、绞（dclingfang）。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCCHENYONG: u32 = 2_745_284_067;
const DCLINGFANG: u32 = 3_884_646_990;
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

fn inject(tx: &mut Transaction, player: PlayerId, kind: CardKind, suit: Suit) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(kind as u32),
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

fn mark_count(tx: &Transaction, owner: PlayerId) -> i32 {
    tx.state()
        .players
        .get(owner)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCLINGFANG)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch6_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let chenyong = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCCHENYONG)
        .expect("dcchenyong trigger");
    assert_eq!(chenyong.event, "phase_end");
    assert_eq!(
        chenyong.filter,
        "owner_is_target_play_phase_card_category_count_positive"
    );
    assert!(chenyong.optional);

    let lingfang: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| trigger.skill_def_id == DCLINGFANG)
        .map(|trigger| (trigger.event.as_str(), trigger.filter.as_str()))
        .collect();
    assert_eq!(
        lingfang,
        vec![
            ("phase_prepare", "owner_is_target"),
            ("card_targeted", "owner_is_target_nonred_card"),
        ]
    );
}

#[test]
fn dclingfang_gains_mark_at_prepare_phase() {
    let (mut tx, owner, _first, _second) = setup(20_260_814);
    add_skill(&mut tx, owner, DCLINGFANG);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, owner), 1);
}

#[test]
fn dclingfang_gains_mark_when_targeted_by_nonred_card() {
    let (mut tx, owner, first, _second) = setup(20_260_815);
    add_skill(&mut tx, owner, DCLINGFANG);
    let card = inject(&mut tx, first, CardKind::Sha, Suit::Spade);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(mark_count(&tx, owner), 1);
}
