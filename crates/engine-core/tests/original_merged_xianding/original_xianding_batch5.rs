//! Golden for XIANDING（限定包）批次五：
//! 逾制（dcyuzhi）、长姬（dczhangji）。

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId, Suit};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{Card, CardKind, GameState, Phase, ZoneKind};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCYUZHI: u32 = 3_271_796_686;
const DCZHANGJI: u32 = 2_178_808_868;
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

fn inject_sha(tx: &mut Transaction, player: PlayerId) -> CardId {
    let hand = tx.state().zones.find(ZoneKind::Hand, player).unwrap();
    let card = tx.state_mut().cards.alloc(Card {
        id: CardId::NONE,
        def: engine_api::CardDefId::new(1),
        kind: CardKind::Sha,
        suit: Suit::Spade,
        rank: 7,
        weapon_range: 0,
        zone: hand,
        owner: player,
        face_up: false,
    });
    tx.state_mut().zones.get_mut(hand).unwrap().cards.push(card);
    card
}

#[test]
fn package_exposes_batch5_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let yuzhi = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCYUZHI)
        .expect("dcyuzhi trigger");
    assert_eq!(yuzhi.event, "card_targeted");
    assert_eq!(yuzhi.filter, "owner_is_target_event_card_sha");
    assert!(!yuzhi.optional);

    let zhangji = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCZHANGJI)
        .expect("dczhangji trigger");
    assert_eq!(zhangji.event, "card_targeted");
    assert_eq!(zhangji.filter, "owner_alive_targeted_by_other");
    assert!(!zhangji.optional);
}

#[test]
fn dcyuzhi_plans_base_damage_increase_when_sha_targets_owner() {
    let (mut tx, owner, first, _second) = setup(20_260_811);
    add_skill(&mut tx, owner, DCYUZHI);
    let sha = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: sha,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    assert!(
        effects
            .iter()
            .any(|effect| matches!(effect, EffectRequest::IncreaseCurrentCardBaseDamage { .. })),
        "expected base-damage increase, got {effects:?}"
    );
}

#[test]
fn dczhangji_draws_extra_for_multi_target_card() {
    let (mut tx, owner, first, second) = setup(20_260_812);
    add_skill(&mut tx, owner, DCZHANGJI);
    let before = tx.state().hand_count(owner);
    let sha = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: sha,
            targets: vec![owner, second],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before + 1);
}

#[test]
fn dczhangji_no_draw_for_single_target_card() {
    let (mut tx, owner, first, _second) = setup(20_260_813);
    add_skill(&mut tx, owner, DCZHANGJI);
    let before = tx.state().hand_count(owner);
    let sha = inject_sha(&mut tx, first);
    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::CardUsed {
            player: first,
            card: sha,
            targets: vec![owner],
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    assert_eq!(tx.state().hand_count(owner), before);
}
