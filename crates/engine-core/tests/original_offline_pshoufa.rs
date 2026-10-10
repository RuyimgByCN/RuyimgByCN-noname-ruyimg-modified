//! Golden coverage for OFFLINE PS刘备「后发」rewritten from legacy JavaScript semantics.

use engine_api::{
    CardId, ChoiceId, Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey,
    PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects, EffectError};
use engine_core::state::{CardKind, ContinuationKind, GameState};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const PSHOUFA: u32 = 3_537_513_199;
const ZJ_FENXUN: u32 = 3_362_692_932;
const ZJ_FENXUN2: u32 = 3_251_234_145;
const SOURCE: &str = concat!(
    include_str!("../../../content/original/ported/offline/pshoufa.skill"),
    include_str!("../../../content/original/ported/offline/zj_fenxun.skill"),
);

fn setup() -> (GameState, PlayerId, PlayerId) {
    let package = content_compiler::compile_source_named("pshoufa-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_731, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let seats = tx.state().players.alive_by_seat();
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .chain(seats.iter().map(|player| EffectRequest::DiscardFromHand {
            player: *player,
            count: u32::MAX,
        }))
        .chain([EffectRequest::AddSkill {
            player: seats[1],
            skill_def: SkillDefId::new(PSHOUFA),
        }])
        .collect();
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().players.get_mut(seats[0]).unwrap().max_hp = 3;
    (tx.commit().state, seats[1], seats[0])
}

fn damage(
    state: GameState,
    source: PlayerId,
    target: PlayerId,
) -> (GameState, Result<Vec<engine_api::DomainEvent>, EffectError>) {
    let mut tx = Transaction::begin(state);
    let result = run_effects(
        &mut tx,
        vec![EffectRequest::DealCardDamage {
            source,
            target,
            amount: 1,
            is_lose_hp: false,
            nature: rule_api::damage_nature::NORMAL,
            card: CardId::NONE,
            card_kind: CardKind::Sha.tag(),
            card_flags: 0,
        }],
    );
    (tx.commit().state, result)
}

fn resolve(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state.clone(),
        Command {
            actor,
            expected_revision: ExpectedRevision(state.revision),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ResolveChoice {
                choice_id: ChoiceId::new(state.resolution.pending_choice_id),
                selections: vec![1],
                cancelled: false,
            },
        },
    )
}

#[test]
fn earlier_seat_draws_to_target_max_hp_only_once_per_turn() {
    let package = content_compiler::compile_source_named("pshoufa-test", SOURCE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 340);
    assert_eq!(
        package.trigger_index[0].filter,
        "owner_source_target_earlier_seat_hand_below_target_max_hp_once_per_turn"
    );

    let (state, owner, target) = setup();
    let (state, first) = damage(state, owner, target);
    assert_eq!(first, Err(EffectError::ChoiceRequested));
    assert_eq!(
        state.resolution.continuation_kind,
        ContinuationKind::OptionalTrigger
    );
    let (state, accepted) = resolve(state, owner, 1);
    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), 3);

    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::DiscardFromHand {
            player: owner,
            count: u32::MAX,
        }],
    )
    .unwrap();
    let (state, second) = damage(tx.commit().state, owner, target);
    assert!(second.is_ok());
    assert_eq!(state.hand_count(owner), 0);
}

#[test]
fn zj_fenxun_has_no_card_cost_and_binds_one_target_for_the_turn() {
    let package = content_compiler::compile_source_named("offline-small-rewrites", SOURCE).unwrap();
    let active = package
        .active_index
        .iter()
        .find(|active| active.skill_def_id == ZJ_FENXUN)
        .unwrap();
    assert_eq!(active.uses_per_turn, Some(1));
    assert!(active.selection.is_none());

    let (state, owner, target) = setup();
    let mut tx = Transaction::begin(state);
    run_effects(
        &mut tx,
        vec![EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(ZJ_FENXUN),
        }],
    )
    .unwrap();
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = engine_core::state::Phase::Play;
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == ZJ_FENXUN)
        .unwrap();
    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(10),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: Vec::new(),
            },
        },
    );
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert!(state
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .any(|skill| {
            state
                .skills
                .get(*skill)
                .is_some_and(|instance| instance.def.raw() == ZJ_FENXUN2)
        }));
}
