//! Golden for sp batch-U DSL rewrites (zero new primitives):
//! 小西·OL (olxiaoxi) round-start virtual Sha;
//! 逆乱·OL (olniluan) other's end phase with higher HP → virtual Sha;
//! 淑慎·国战 (gzshushen) recover → other draws.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use engine_core::Engine;
use rule_api::EffectRequest;

const OLXIAOXI: u32 = 3_429_531_288;
const OLNILUAN: u32 = 3_803_313_156;
const GZSHUSHEN: u32 = 2_220_815_073;
const PACKAGE: &[u8] = include_bytes!("../../../../content/original/ported/sp/sp.npack.json");

fn started(seed: u64) -> (GameState, PlayerId) {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let mut engine = Engine::new();
    engine.load_package(&package).unwrap();
    assert!(matches!(
        engine.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(seed),
            kind: CommandKind::StartGame {
                player_count: 2,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }),
        CommandResult::Accepted { .. }
    ));
    let state = engine.state().clone();
    let owner = state.turn.current;
    (state, owner)
}

fn with_skill(state: GameState, owner: PlayerId, def: u32) -> GameState {
    let mut tx = Transaction::begin(state);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(def),
    });
    run_effects(&mut tx, effects).unwrap();
    tx.commit().state
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, engine_core::command::PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections,
                cancelled,
            },
        },
    )
}

#[test]
fn package_publishes_batch_u_contract() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert!(package.manifest.rule_vm_abi_version >= 647);
    let triggers: Vec<_> = package
        .trigger_index
        .iter()
        .filter(|trigger| matches!(trigger.skill_def_id, OLXIAOXI | OLNILUAN | GZSHUSHEN))
        .map(|trigger| {
            (
                trigger.skill_def_id,
                trigger.event.as_str(),
                trigger.filter.as_str(),
                trigger.optional,
            )
        })
        .collect();
    assert!(triggers.contains(&(OLXIAOXI, "round_started", "owner_alive", true)));
    assert!(triggers.contains(&(OLNILUAN, "phase_end", "owner_alive_not_target", true)));
    assert!(triggers.contains(&(GZSHUSHEN, "hp_recovered", "owner_is_target", true)));
    assert!(!package
        .modifier_index
        .iter()
        .any(|modifier| { matches!(modifier.skill_def_id, OLXIAOXI | OLNILUAN | GZSHUSHEN) }));
}

#[test]
fn olxiaoxi_opens_virtual_sha_on_round_start() {
    let (state, owner) = started(2_026_081_501);
    let state = with_skill(state, owner, OLXIAOXI);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::RoundStarted {
            round: 1,
            current: owner,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
}

#[test]
fn olniluan_opens_virtual_sha_when_event_player_hp_gt_owner() {
    let (state, owner) = started(2_026_081_502);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLNILUAN);
    let mut tx = Transaction::begin(state);
    // Boost the event player's HP above the owner.
    tx.state_mut().players.get_mut(other).unwrap().hp = 10;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 10;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::VirtualShaTargets
    );
}

#[test]
fn olniluan_skipped_when_event_player_hp_not_above_owner() {
    let (state, owner) = started(2_026_081_503);
    let other = state
        .players
        .iter_live()
        .find(|player| player.id != owner)
        .unwrap()
        .id;
    let state = with_skill(state, owner, OLNILUAN);
    let mut tx = Transaction::begin(state);
    // Owner has max HP (lord bonus); the event player is not strictly higher.
    tx.state_mut().players.get_mut(other).unwrap().hp = 1;
    tx.state_mut().players.get_mut(other).unwrap().max_hp = 1;
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::TurnStateChanged {
            current: other,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(
        opened.unwrap_err(),
        EffectError::ChoiceRequested,
        "the optional trigger still opens a confirm choice"
    );
    let tx = tx.commit();
    // Confirming the optional trigger runs the body; the HP guard skips the
    // virtual Sha, so no VirtualShaTargets choice follows.
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
}

#[test]
fn gzshushen_opens_other_choice_on_recover() {
    let (state, owner) = started(2_026_081_504);
    let state = with_skill(state, owner, GZSHUSHEN);
    let mut tx = Transaction::begin(state);
    let opened = dispatch_for_events(
        &mut tx,
        &[DomainEvent::HpRecovered {
            source: owner,
            player: owner,
            amount: 1,
        }],
        &mut TriggerFireRecord::new(),
    );
    assert_eq!(opened.unwrap_err(), EffectError::ChoiceRequested);
    assert_eq!(
        tx.state().resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let tx = tx.commit();
    let (state, accepted) = resolve(tx.state, owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    // choose_other_draw_or_recover opens a target-selection choice.
    assert!(state.has_pending_choice());
}
