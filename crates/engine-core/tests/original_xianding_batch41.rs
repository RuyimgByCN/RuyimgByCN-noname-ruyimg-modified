//! Golden for XIANDING（限定包）批次四十一：
//! 天匠（pytianjiang，PY）。

use engine_api::{DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const PYTIANJIANG: u32 = 2_167_112_704;
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package()).unwrap();
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

#[test]
fn package_exposes_batch41_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == PYTIANJIANG)
        .expect("pytianjiang trigger");
    assert_eq!(trigger.event, "game_started");
    assert!(!trigger.optional);
}

#[test]
fn pytianjiang_gains_equipment_on_game_start() {
    let (mut tx, owner, _first, _second) = setup(20_260_878);
    let before_hand = tx.state().hand_count(owner) as u32;
    let before_equip = tx
        .state()
        .zones
        .get(tx.state().zones.find(engine_core::state::ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .len() as u32;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(PYTIANJIANG),
        }],
    )
    .unwrap();

    let effects = dispatch_for_events(
        &mut tx,
        &[DomainEvent::GameStarted { player_count: 3 }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, effects).unwrap();
    let after_hand = tx.state().hand_count(owner) as u32;
    let after_equip = tx
        .state()
        .zones
        .get(tx.state().zones.find(engine_core::state::ZoneKind::Equip, owner).unwrap())
        .unwrap()
        .cards
        .len() as u32;
    assert!(
        after_hand + after_equip >= before_hand + before_equip + 1u32,
        "should have gained one equipment card"
    );
}
