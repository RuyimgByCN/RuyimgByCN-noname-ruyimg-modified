//! Golden coverage for 链舟（huicui 蔡瑁张允）同体力横置（ABI 458）.
//!
//! 准备阶段：锁定横置自己，然后选择任意名体力值相同且未横置的其他角色并横置。

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

const LIANZHOU: u32 = 3_041_401_308;
const MARK_CHAIN: u32 = 2;
const SOURCE: &str = include_str!("../../../../content/original/ported/huicui/lianzhou.skill");

fn is_linked(state: &GameState, player: PlayerId) -> bool {
    state
        .players
        .get(player)
        .map(|participant| {
            participant
                .marks
                .iter()
                .any(|(mark, count)| *mark == MARK_CHAIN && *count > 0)
        })
        .unwrap_or(false)
}

#[test]
fn links_self_then_links_same_hp_unlinked_others() {
    let package = content_compiler::compile_source_named("lianzhou-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 458);
    let mut tx = Transaction::begin(GameState::empty());
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();
    bootstrap_match(&mut tx, 3, 2_026_080_115, "identity-standard").unwrap();
    let players = tx.state().players.alive_by_seat();
    let (owner, victim, other) = (players[0], players[1], players[2]);
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(LIANZHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    // Equalize HP so both other players qualify.
    for p in [owner, victim, other] {
        tx.state_mut().players.get_mut(p).unwrap().hp = 3;
    }
    // Pre-link the third player so only the victim is selectable.
    run_effects(
        &mut tx,
        vec![EffectRequest::AddMark {
            player: other,
            mark: engine_api::MarkDefId::new(MARK_CHAIN),
            delta: 1,
        }],
    )
    .unwrap();

    let mut ending = Transaction::begin(tx.commit().state);
    let fired = dispatch_for_events(
        &mut ending,
        &[engine_api::DomainEvent::TurnStateChanged {
            current: owner,
            phase: TurnPhaseTag::Prepare,
        }],
        &mut TriggerFireRecord::new(),
    );
    match fired {
        Ok(effects) => {
            if let Err(e) = run_effects(&mut ending, effects) {
                assert!(
                    matches!(e, EffectError::ChoiceRequested),
                    "target choice must pause: {e:?}"
                );
            }
        }
        Err(EffectError::ChoiceRequested) => {}
        Err(e) => panic!("dispatch failed: {e:?}"),
    }
    let mut state = ending.commit().state;
    assert!(is_linked(&state, owner), "owner links itself first");
    let revision = state.revision;
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    let (next, result) = apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(revision),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![victim.raw()],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "target choice must be accepted"
    );
    let state = next;
    assert!(
        is_linked(&state, victim),
        "selected same-HP unlinked target gets linked"
    );
    assert!(
        is_linked(&state, other),
        "pre-linked target stays linked and is not re-selected"
    );
    assert!(!state.has_pending_choice());
}
