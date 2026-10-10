//! M0 slice5: DSL `choose{}` end-to-end — VM pause → ChoiceRequest → resume.
//!
//! An active skill offers a choice between two option tokens; the engine pauses
//! at HOST_CHOICE, opens a ChoiceRequest, and on ResolveChoice pushes the chosen
//! token back into the VM and resumes, applying the branch's effect.

use content_compiler::compile_source;
use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::Phase;
use engine_core::Engine;
use rule_api::EffectRequest;

// Active skill 60: choose to draw (token 1) or recover (token 2).
const PACK: &str = r#"
skill xuanze(60) {
  active play_phase {
    let pick = choose(1, 2);
    if pick == 1 {
      effect draw_cards(self, 2);
    }
    if pick == 2 {
      effect recover_hp(self, 1);
    }
  }
}
"#;

fn start(seed: u64, n: u8) -> Command {
    Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: n,
            rng_seed: seed,
            ruleset: "identity-standard".into(),
        },
    }
}

/// Grant skill 60 to the current player and return (engine-state, actor, skill id).
fn setup() -> (Engine, PlayerId) {
    let pkg = compile_source(PACK).unwrap();
    let mut eng = Engine::new();
    eng.load_package(&pkg).unwrap();
    eng.submit(start(42, 2));
    let actor = eng.state().turn.current;
    (eng, actor)
}

#[test]
fn choose_draw_branch_resumes_and_draws() {
    let (eng, actor) = setup();
    // Grant skill 60 + set play phase via a transaction, then persist.
    let mut tx = engine_core::transaction::Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: actor,
            skill_def: SkillDefId::new(60),
        }],
    )
    .unwrap();
    let state = tx.state().clone();
    let skill = *state.players.get(actor).unwrap().skills.last().unwrap();
    let before = state.hand_count(actor);

    // Activate → should pause on choose.
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(state.has_pending_choice(), "choose should open a choice");
    let choice = pipe.choice.expect("choice request");
    assert_eq!(choice.options.len(), 2, "two option tokens offered");

    // Resolve choosing token 1 (draw branch).
    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![1],
                cancelled: false,
            },
        },
    );
    assert!(
        matches!(pipe.result, CommandResult::Accepted { .. }),
        "{:?}",
        pipe.result
    );
    assert!(!state.has_pending_choice(), "choice resolved");
    assert_eq!(state.hand_count(actor), before + 2, "draw branch drew 2");
}

#[test]
fn choose_recover_branch_resumes_and_recovers() {
    let (eng, actor) = setup();
    let mut tx = engine_core::transaction::Transaction::begin(eng.state().clone());
    tx.state_mut().revision = 1;
    tx.state_mut().turn.phase = Phase::Play;
    // Wound the actor so recover is observable.
    run_effects(
        &mut tx,
        vec![
            EffectRequest::AddSkill {
                player: actor,
                skill_def: SkillDefId::new(60),
            },
            EffectRequest::ChangeMaxHp {
                player: actor,
                delta: 2,
            },
            EffectRequest::DealDamage {
                source: actor,
                target: actor,
                amount: 2,
                is_lose_hp: true,
                nature: 0,
            },
        ],
    )
    .unwrap();
    let state = tx.state().clone();
    let skill = *state.players.get(actor).unwrap().skills.last().unwrap();
    let hp_before = state.players.get(actor).unwrap().hp;

    let (state, pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(1),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![],
                card_targets: vec![],
            },
        },
    );
    assert!(matches!(pipe.result, CommandResult::Accepted { .. }));
    let choice = pipe.choice.expect("choice request");

    // Choose token 2 (recover branch).
    let (state, _pipe) = apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(2),
            kind: CommandKind::ResolveChoice {
                choice_id: choice.choice_id,
                selections: vec![2],
                cancelled: false,
            },
        },
    );
    assert_eq!(
        state.players.get(actor).unwrap().hp,
        hp_before + 1,
        "recover branch healed 1"
    );
}
