//! Golden coverage for 标刘表「自守」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const STDZISHOU: u32 = 2_683_293_404;
const STD_CAOZHEN: u32 = 2_623_502_245;
const STD_QUYI: u32 = 2_888_834_865;
const STD_DONGYUN: u32 = 2_711_327_184;
const STD_LIUZAN: u32 = 4_100_887_402;
const SOURCE: &str = include_str!("../../../content/original/ported/sixiang/stdzishou.skill");
const PACKAGE: &[u8] =
    include_bytes!("../../../content/original/ported/sixiang/sixiang.npack.json");

fn submit(
    state: GameState,
    actor: PlayerId,
    kind: CommandKind,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind,
        },
    )
}

#[test]
fn package_and_runtime_preserve_draw_then_skip_semantics() {
    let package = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(package.manifest.rule_vm_abi_version, 562);
    assert_eq!(package.skill_index.len(), 181);
    let compiled = content_compiler::compile_source_named("stdzishou", SOURCE).unwrap();
    let expected = &compiled.modules[0];
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == expected.name)
        .unwrap();
    assert_eq!(actual.logic_hash, expected.logic_hash);

    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 4, 2_026_074_401, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&package).unwrap();
    let players = tx.state().players.alive_by_seat();
    let owner = players[0];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(STDZISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    for (player, character) in players
        .iter()
        .zip([STD_CAOZHEN, STD_QUYI, STD_DONGYUN, STD_LIUZAN])
    {
        tx.state_mut()
            .players
            .get_mut(*player)
            .unwrap()
            .character_def = character;
    }
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Draw;
    tx.state_mut().revision = 1;
    let state = tx.commit().state;
    let before = state.hand_count(owner);

    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    let choice = offered.choice.as_ref().unwrap();
    assert_eq!(choice.kind, ChoiceKind::Confirm);
    let (state, accepted) = submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id: ChoiceId::new(choice.choice_id.raw()),
            selections: vec![1],
            cancelled: false,
        },
        2,
    );

    assert!(matches!(accepted.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(owner), before + 4);
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(accepted.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Play,
        } if *current == owner
    )));
}
