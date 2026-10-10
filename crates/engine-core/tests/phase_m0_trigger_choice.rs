//! Trigger VM choice barriers retain the fixed scheduler queue.

use content_compiler::compile_source;
use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::GameState;
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PACK: &str = r#"
skill choose_trigger(60) {
  trigger damage owner_is_target {
    let pick = choose(1, 2);
    if pick == 1 {
      effect draw_cards(self, 2);
    }
  }
}

skill later_trigger(61) {
  trigger damage owner_is_target {
    effect draw_cards(self, 3);
  }
}
"#;

const OPTIONAL_PACK: &str = r#"
skill optional_draw(62) {
  trigger damage owner_is_target optional {
    effect draw_cards(self, 2);
  }
}

skill later_draw(63) locked {
  trigger damage owner_is_target {
    effect draw_cards(self, 3);
  }
}
"#;

#[test]
fn trigger_choice_resumes_vm_then_remaining_trigger() {
    let package = compile_source(PACK).expect("compile trigger package");
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).expect("load package");
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 41, "identity-standard").expect("bootstrap");
    tx.state_mut().revision = 1;

    let owner = PlayerId::new(1);
    let old_skills: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .flat_map(|player| player.skills.iter().copied())
        .collect();
    run_effects(
        &mut tx,
        old_skills
            .into_iter()
            .map(|skill| EffectRequest::RemoveSkill { skill })
            .collect(),
    )
    .expect("remove bootstrap skills");
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(60),
            },
            EffectRequest::AddSkill {
                player: owner,
                skill_def: SkillDefId::new(61),
            },
        ],
    )
    .expect("grant trigger skills");

    let hand_before = tx.state().hand_count(owner);
    let error = run_effects(
        &mut tx,
        vec![EffectRequest::DealDamage {
            source: PlayerId::new(2),
            target: owner,
            amount: 1,
            is_lose_hp: false,
            nature: 0,
        }],
    )
    .expect_err("first trigger must pause");
    assert_eq!(error, EffectError::ChoiceRequested);
    assert!(tx.state().has_pending_choice());
    assert_eq!(tx.state().hand_count(owner), hand_before);
    assert_eq!(tx.state().resolution.deferred_triggers.len(), 1);

    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(!state.has_pending_choice());
    assert!(state.resolution.deferred_effects.is_empty());
    assert!(state.resolution.deferred_triggers.is_empty());
    assert_eq!(state.hand_count(owner), hand_before + 5);
}

#[test]
fn declining_optional_trigger_still_runs_remaining_trigger() {
    let package = compile_source(OPTIONAL_PACK).expect("compile optional package");
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content).load_package(&package).expect("load package");
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 2, 43, "identity-standard").expect("bootstrap");
    tx.state_mut().revision = 1;
    let owner = PlayerId::new(1);
    let old_skills: Vec<_> = tx
        .state()
        .players
        .iter_live()
        .flat_map(|player| player.skills.iter().copied())
        .collect();
    let mut setup_effects: Vec<_> = old_skills
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill })
        .collect();
    setup_effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(62),
        },
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(63),
        },
    ]);
    run_effects(&mut tx, setup_effects).expect("set up skills");
    let hand_before = tx.state().hand_count(owner);

    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::new(2),
                target: owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    assert_eq!(
        tx.state().resolution.continuation_kind,
        engine_core::state::ContinuationKind::OptionalTrigger
    );
    let committed = tx.commit();
    let choice_id = ChoiceId::new(committed.state.resolution.pending_choice_id);
    let (state, result) = apply_command(
        committed.state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id,
                selections: vec![0],
                cancelled: false,
            },
        },
    );

    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), hand_before + 3);
    assert!(
        state
            .content
            .skill_metadata(SkillDefId::new(63))
            .unwrap()
            .locked
    );
}
