//! Golden coverage for 神话「节命」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId,
    SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const JIEMING: u32 = 3_579_069_878;
const SOURCE: &str = include_str!("../../../../content/original/ported/shenhua/jieming.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../../content/original/ported/shenhua/shenhua.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    source: PlayerId,
    target: PlayerId,
}

fn setup() -> Setup {
    let package = content_compiler::compile_source_named("jieming-test", SOURCE).unwrap();
    let mut state = GameState::empty();
    std::sync::Arc::make_mut(&mut state.content)
        .load_package(&package)
        .unwrap();
    let mut tx = Transaction::begin(state);
    bootstrap_match(&mut tx, 4, 20_260_722_84, "identity-standard").unwrap();
    let seats = tx.state().players.alive_by_seat();
    let owner = seats[0];
    let source = seats[1];
    let target = seats[2];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.extend(seats.iter().map(|player| EffectRequest::DiscardFromHand {
        player: *player,
        count: u32::MAX,
    }));
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(JIEMING),
    });
    run_effects(&mut tx, effects).unwrap();
    {
        let owner_state = tx.state_mut().players.get_mut(owner).unwrap();
        owner_state.max_hp = 10;
        owner_state.hp = 10;
        let target_state = tx.state_mut().players.get_mut(target).unwrap();
        target_state.max_hp = 7;
        target_state.hp = 7;
    }
    Setup {
        state: tx.commit().state,
        owner,
        source,
        target,
    }
}

fn resolve(
    state: GameState,
    actor: PlayerId,
    selections: Vec<u32>,
    cancelled: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
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
fn package_exposes_exact_per_damage_point_contract() {
    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(committed.manifest.rule_vm_abi_version, 671);
    assert!(committed
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == JIEMING && skill.name == "jieming" && !skill.locked));
    let trigger = committed
        .trigger_index
        .iter()
        .find(|trigger| trigger.skill_def_id == JIEMING)
        .unwrap();
    assert_eq!(trigger.event, "damage");
    assert_eq!(trigger.filter, "owner_is_target_per_damage_point");
    assert!(trigger.optional);

    let compiled = content_compiler::compile_source_named("jieming", SOURCE).unwrap();
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn two_damage_points_offer_independent_targets_and_cap_draw_at_five() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: setup.source,
                target: setup.owner,
                amount: 2,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    assert_eq!(state.players.get(setup.owner).unwrap().hp, 8);

    // The first damage point's optional confirm opens; accepting it commits the first
    // point's effect before the second point's prompt (confirmed triggers run first so
    // replacement/exclusion filters can invalidate stale queued work).
    let (state, accepted) = resolve(state, setup.owner, vec![1], false, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let options: Vec<_> = accepted
        .choice
        .unwrap()
        .options
        .into_iter()
        .map(|option| option.token)
        .collect();
    assert_eq!(
        options,
        vec![setup.owner.raw(), setup.source.raw(), setup.target.raw(), 4]
    );

    // Draw the first point's target up to five; the second point's confirm then opens.
    let (state, drawn) = resolve(state, setup.owner, vec![setup.target.raw()], false, 2);
    assert!(matches!(drawn.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 5);
    assert!(state.has_pending_choice());
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );

    // Accept the second point's confirm and draw the same target again (already capped).
    let (state, accepted_again) = resolve(state, setup.owner, vec![1], false, 3);
    assert!(matches!(
        accepted_again.result,
        CommandResult::Accepted { .. }
    ));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::ChooseTargetDraw
    );
    let (state, no_op) = resolve(state, setup.owner, vec![setup.target.raw()], false, 4);
    assert!(matches!(no_op.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.target), 5);
    assert!(!state.has_pending_choice());
}

#[test]
fn source_free_damage_triggers_and_dead_target_is_rejected_at_resume() {
    let setup = setup();
    let mut tx = Transaction::begin(setup.state);
    assert_eq!(
        run_effects(
            &mut tx,
            vec![EffectRequest::DealDamage {
                source: PlayerId::NONE,
                target: setup.owner,
                amount: 1,
                is_lose_hp: false,
                nature: 0,
            }],
        ),
        Err(EffectError::ChoiceRequested)
    );
    let state = tx.commit().state;
    let (mut state, accepted) = resolve(state, setup.owner, vec![1], false, 5);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    state.players.get_mut(setup.target).unwrap().alive = false;
    let (_, stale) = resolve(state, setup.owner, vec![setup.target.raw()], false, 6);
    assert!(matches!(stale.result, CommandResult::Rejected { .. }));
}
