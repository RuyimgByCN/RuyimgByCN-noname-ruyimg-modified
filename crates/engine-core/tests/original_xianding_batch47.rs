//! Golden for XIANDING（限定包）批次四十七：
//! 巡使（xunshi）、王抗（dcsbwankang）、纫霜（old_dcrenshuang）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const XUNSHI: u32 = 4_189_180_642;
const DCSBWANKANG: u32 = 2_153_717_747;
const OLD_DCRENSHUANG: u32 = 3_716_618_560;
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

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    state
}

#[test]
fn package_exposes_batch47_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let name = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == XUNSHI && m.target == "card_name")
        .expect("xunshi card_name");
    assert_eq!(name.selector.as_deref(), Some("trick"));
    assert_eq!(name.value, 1);

    let wankang = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCSBWANKANG)
        .expect("dcsbwankang trigger");
    assert_eq!(wankang.event, "dying");

    let renshuang = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == OLD_DCRENSHUANG)
        .expect("old_dcrenshuang trigger");
    assert_eq!(renshuang.event, "dying");
}

#[test]
fn dcsbwankang_recovers_one_and_draws_two_on_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_902);
    add_skill(&mut tx, owner, DCSBWANKANG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let hp = tx.state().players.get(owner).unwrap().hp;
    let before = tx.state().hand_count(owner);

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, fired).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, hp + 1);
    assert_eq!(tx.state().hand_count(owner), before + 2);
}

#[test]
fn old_dcrenshuang_recovers_to_one_and_gains_max_hp_on_dying() {
    let (mut tx, owner, _first, _second) = setup(20_260_903);
    add_skill(&mut tx, owner, OLD_DCRENSHUANG);
    tx.state_mut().players.get_mut(owner).unwrap().hp = 0;
    let max_hp = tx.state().players.get(owner).unwrap().max_hp;

    let fired = dispatch_for_events(
        &mut tx,
        &[DomainEvent::EnteredDying {
            player: owner,
            source: PlayerId::NONE,
        }],
        &mut TriggerFireRecord::new(),
    )
    .unwrap();
    run_effects(&mut tx, fired).unwrap();
    assert_eq!(tx.state().players.get(owner).unwrap().hp, 1);
    assert_eq!(tx.state().players.get(owner).unwrap().max_hp, max_hp + 1);
}
