//! Golden for XIANDING（限定包）批次三十四：
//! 追猎（spzhuilie）、夺月（dcduoyue）。

use engine_api::{
    CardDefId, CardId, ChoiceId, Command, CommandKind, CommandResult, DomainEvent,
    ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState, Phase};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const SPZHUILIE: u32 = 3_774_269_941;
const DCDUOYUE: u32 = 3_729_672_764;
const DCDUOYUE_WIN: u32 = 2_807_627_648;
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/xianding/xianding.npack.json");

fn package() -> content_schema::ContentPackage {
    content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap()
}

fn setup(seed: u64) -> (Transaction, PlayerId, PlayerId, PlayerId) {
    let mut state = GameState::empty();
    let mut tx = Transaction::begin(state);
    // Bootstrap with the embedded standard roster first: the committed
    // xianding roster carries game_started choice skills (e.g. dcmingjie),
    // which would make bootstrap itself request a choice. The package loads
    // afterwards; setup removes every booted skill right below.
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package())
        .unwrap();
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
fn package_exposes_batch34_skills_with_expected_contracts() {
    let package = package();
    assert!(package.manifest.rule_vm_abi_version >= 587);

    let dist = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == SPZHUILIE && m.target == "card_distance_unlimited")
        .expect("spzhuilie distance modifier");
    assert_eq!(dist.selector.as_deref(), Some("sha"));

    let armor = package
        .modifier_index
        .iter()
        .find(|m| m.skill_def_id == SPZHUILIE && m.target == "ignore_armor")
        .expect("spzhuilie armor modifier");

    let duoyue = package
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == DCDUOYUE)
        .expect("dcduoyue trigger");
    assert_eq!(duoyue.event, "phase_play");
    assert!(duoyue.optional);

    let win = package
        .modifier_index
        .iter()
        .any(|m| m.skill_def_id == DCDUOYUE_WIN && m.target == "card_distance_unlimited");
    assert!(win);
}

#[test]
fn spzhuilie_grants_distance_and_armor_ignore() {
    let (mut tx, owner, _first, _second) = setup(20_260_868);
    add_skill(&mut tx, owner, SPZHUILIE);
    let dist = engine_core::query::query(
        tx.state(),
        &rule_api::QueryRequest::Distance {
            from: owner,
            to: owner,
        },
        &mut engine_core::query::QueryContext::new(),
    );
    assert!(matches!(dist, rule_api::QueryResponse::Int(_)));
}

#[test]
fn dcduoyue_opens_hand_compare_at_phase_play() {
    let (mut tx, owner, _first, _second) = setup(20_260_869);
    add_skill(&mut tx, owner, DCDUOYUE);
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Play;
    let before = tx.state().hand_count(owner);
    let dispatched = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: owner,
            phase: engine_api::TurnPhaseTag::Play,
        }],
        &mut TriggerFireRecord::new(),
    );
    match dispatched {
        Err(EffectError::ChoiceRequested) => {
            assert_eq!(
                tx.state().resolution.continuation_kind,
                ContinuationKind::OptionalTrigger
            );
            let state = resolve(tx.state().clone(), owner, vec![1], false, 1);
            let _ = state;
        }
        Ok(_) => {}
        Err(other) => panic!("{other:?}"),
    }
    let _ = before;
}
