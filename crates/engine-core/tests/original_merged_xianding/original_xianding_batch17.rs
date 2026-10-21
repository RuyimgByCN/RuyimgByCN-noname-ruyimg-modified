//! Golden for XIANDING（限定包）批次十七：
//! 梦魇（dcwuhun，②）。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const DCWUHUN: u32 = 4_234_333_270;
const DCWUHUN_MARK: u32 = 3_698_112_918;
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

fn mark_count(tx: &Transaction, player: PlayerId) -> i32 {
    tx.state()
        .players
        .get(player)
        .unwrap()
        .marks
        .iter()
        .find(|(mark, _)| *mark == DCWUHUN_MARK)
        .map_or(0, |(_, count)| *count)
}

#[test]
fn package_exposes_batch17_skill_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let trigger = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCWUHUN)
        .expect("dcwuhun trigger");
    assert_eq!(trigger.event, "damage");
    assert_eq!(
        trigger.filter,
        "owner_is_source_or_target_other_alive_per_damage_point"
    );
    assert!(!trigger.optional);
}

#[test]
fn dcwuhun_marks_damage_source_when_owner_damaged() {
    let (mut tx, owner, first, _second) = setup(20_260_840);
    add_skill(&mut tx, owner, DCWUHUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: first,
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(err.is_ok(), "locked trigger must not open a choice");
    assert_eq!(mark_count(&tx, first), 1);
    assert_eq!(mark_count(&tx, owner), 0);
}

#[test]
fn dcwuhun_marks_once_per_damage_point() {
    let (mut tx, owner, first, _second) = setup(20_260_841);
    add_skill(&mut tx, owner, DCWUHUN);

    let err = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: first,
            target: owner,
            amount: 3,
            is_lose_hp: false,
            nature: 0,
        }],
    );
    assert!(err.is_ok());
    assert_eq!(mark_count(&tx, first), 3);
}
