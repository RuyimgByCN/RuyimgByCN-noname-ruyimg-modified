//! Golden coverage for SCL 骆统「勤政」rewritten from legacy JavaScript semantics.

use engine_api::{CardId, DomainEvent, PlayerId, SkillDefId};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{CardKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SCLS_QINZHENG: u32 = 3_270_401_551;
const SCL_LUOTONG: u32 = 2_314_837_306;
const SOURCE: &str =
    include_str!("../../../../content/original/ported/offline/scls_qinzheng.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/offline/offline.npack.json");

fn setup() -> (Transaction, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("scls-qinzheng-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, SCLS_QINZHENG as u64, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let other = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SCLS_QINZHENG),
    });
    run_effects(&mut tx, effects).unwrap();
    (tx, owner, other)
}

fn emit_action(tx: &mut Transaction, player: PlayerId, other: PlayerId, responded: bool) {
    let event = if responded {
        DomainEvent::CardResponded {
            player,
            respond_to: other,
            card: CardId::NONE,
            response_kind: CardKind::Shan.tag(),
        }
    } else {
        DomainEvent::CardUsed {
            player,
            card: CardId::NONE,
            targets: vec![other],
        }
    };
    let effects = dispatch_for_events(tx, &[event], &mut TriggerFireRecord::new()).unwrap();
    run_effects(tx, effects).unwrap();
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let compiled = content_compiler::compile_source_named("scls_qinzheng", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 78);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == SCLS_QINZHENG && skill.name == "scls_qinzheng" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == SCL_LUOTONG)
        .unwrap();
    assert_eq!(character.skills, [SCLS_QINZHENG]);
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn lifetime_use_and_response_count_draws_for_each_matching_divisor_across_turns() {
    let (mut tx, owner, other) = setup();

    let before_other = tx.state().hand_cards(owner).len();
    emit_action(&mut tx, other, owner, false);
    assert_eq!(tx.state().hand_cards(owner).len(), before_other);
    assert_eq!(tx.state().card_action_totals.get(&owner), None);
    assert_eq!(tx.state().card_action_totals.get(&other), Some(&1));

    for total in 1_u32..=15 {
        let before = tx.state().hand_cards(owner).len();
        emit_action(&mut tx, owner, other, total % 2 == 0);
        let expected = [3, 5, 8]
            .into_iter()
            .filter(|divisor| total % divisor == 0)
            .count();
        assert_eq!(
            tx.state().hand_cards(owner).len(),
            before + expected,
            "wrong reward at lifetime action {total}"
        );
        assert_eq!(tx.state().card_action_totals.get(&owner), Some(&total));

        if total == 8 {
            run_effects(
                &mut tx,
                vec![
                    EffectRequest::SetTurnState {
                        current: other,
                        phase: Phase::Prepare.to_u8(),
                    },
                    EffectRequest::SetTurnState {
                        current: owner,
                        phase: Phase::Play.to_u8(),
                    },
                ],
            )
            .unwrap();
            assert_eq!(tx.state().card_action_totals.get(&owner), Some(&8));
        }
    }
}
