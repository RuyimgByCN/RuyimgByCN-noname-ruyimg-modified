//! Golden coverage for 备战（SP 审配）目标摸牌+授予技能（ABI 455）.
//!
//! 审配结束阶段可选确认 → 选择目标摸至 min(5, 体力上限) → 目标获得「备战·待战」；
//! 其回合开始时若手牌数为全场最多（可并列），本回合获得「自守」
//! （card_targets_self_only，yijiang zishou2），随后移除自身。

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId, TurnPhaseTag,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use engine_core::trigger::{dispatch_for_events, TriggerFireRecord};
use rule_api::EffectRequest;

const BEIZHAN: u32 = 2_840_635_277;
const BEIZHAN2: u32 = 4_111_010_111;
const SOURCE: &str = include_str!("../../../content/original/ported/sp2/beizhan.skill");

fn drive_choice(
    state: engine_core::state::GameState,
    actor: PlayerId,
    selection: u32,
) -> GameState {
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![selection],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "choice must be accepted"
    );
    next
}

#[test]
fn draws_target_to_max_hp_then_grants_waiting_skill() {
    let package = content_compiler::compile_source_named("beizhan-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 455);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, target, _third) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(BEIZHAN),
    });
    run_effects(&mut tx, effects).unwrap();
    let target_hand = tx.state().hand_count(target);
    let target_max_hp = tx.state().players.get(target).unwrap().max_hp;

    // Owner's end phase: optional confirm, then the draw-target choice.
    let mut ending = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut ending,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::End,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut ending, effects) {
                assert!(
                    matches!(e, EffectError::ChoiceRequested),
                    "optional confirm must pause: {e:?}"
                );
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch failed: {e:?}"),
    }
    let mut state = ending.commit().state;
    let mut choices = 0;
    while state.has_pending_choice() && choices < 4 {
        let selection = if matches!(
            state.resolution.continuation_kind,
            engine_core::state::ContinuationKind::OptionalTrigger
        ) {
            1
        } else {
            target.raw()
        };
        state = drive_choice(state, owner, selection);
        choices += 1;
    }

    assert_eq!(
        state.hand_count(target),
        (target_max_hp.min(5)).max(target_hand),
        "target draws to min(5, max HP)"
    );
    assert!(
        state
            .skills
            .sorted_live()
            .iter()
            .any(|skill| skill.owner == target && skill.def.raw() == BEIZHAN2),
        "target gains beizhan2"
    );
    assert!(!state.has_pending_choice());
}
