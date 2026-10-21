//! Golden coverage for 旧刘表「旧自守」rewritten from legacy JavaScript semantics.

use engine_api::{
    ChoiceId, ChoiceKind, Command, CommandKind, CommandResult, DomainEvent, ExpectedRevision,
    IdempotencyKey, PlayerId, SkillDefId, TurnPhaseTag,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const OLD_LIUBIAO: u32 = 3_231_055_413;
const OLDZISHOU: u32 = 2_477_599_277;
const ZONGSHI: u32 = 2_639_072_445;
const SOURCE: &str = include_str!("../../../../content/original/ported/old/oldzishou.skill");
const OLD: &[u8] = include_bytes!("../../../../content/original/ported/old/old.npack.json");
const YIJIANG: &[u8] =
    include_bytes!("../../../../content/original/ported/yijiang/yijiang.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
}

fn setup(seed: u64) -> Setup {
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    let yijiang = content_schema::ContentPackage::from_json_bytes(YIJIANG).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 3, seed, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&old)
        .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&yijiang)
        .unwrap();
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
        skill_def: SkillDefId::new(OLDZISHOU),
    });
    run_effects(&mut tx, effects).unwrap();
    let participant = tx.state_mut().players.get_mut(owner).unwrap();
    participant.character_def = OLD_LIUBIAO;
    participant.max_hp = 4;
    participant.hp = 2;
    tx.state_mut().turn.current = owner;
    tx.state_mut().turn.phase = Phase::Judge;
    tx.state_mut().revision = 1;
    Setup {
        state: tx.commit().state,
        owner,
    }
}

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

fn resolve(
    state: GameState,
    owner: PlayerId,
    accept: bool,
    key: u64,
) -> (GameState, PipelineResult) {
    let choice_id = ChoiceId::new(state.resolution.pending_choice_id);
    submit(
        state,
        owner,
        CommandKind::ResolveChoice {
            choice_id,
            selections: vec![u32::from(accept)],
            cancelled: false,
        },
        key,
    )
}

fn enter_draw(state: GameState, owner: PlayerId, accept: bool) -> (GameState, PipelineResult) {
    let (state, offered) = submit(state, owner, CommandKind::EndPhase, 1);
    assert_eq!(offered.choice.as_ref().unwrap().kind, ChoiceKind::Confirm);
    resolve(state, owner, accept, 2)
}

#[test]
fn package_publishes_exact_abi100_skill_and_character_closure() {
    let old = content_schema::ContentPackage::from_json_bytes(OLD).unwrap();
    assert_eq!(old.manifest.rule_vm_abi_version, 275);
    assert!(old.manifest.dependencies.iter().any(|dependency| {
        dependency.namespace == "official"
            && dependency.package_id == "yijiang"
            && dependency.version == "0.1.0"
    }));
    let character = old
        .character_index
        .iter()
        .find(|character| character.id == OLD_LIUBIAO)
        .unwrap();
    assert_eq!(character.name_key, "old_liubiao");
    assert_eq!(character.sex, "male");
    assert_eq!(character.group, "qun");
    assert_eq!(character.max_hp, 4);
    assert_eq!(character.skills, [OLDZISHOU, ZONGSHI]);

    let compiled = content_compiler::compile_source_named("oldzishou", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 100);
    for expected in compiled.modules {
        let actual = old
            .modules
            .iter()
            .find(|module| module.name == expected.name)
            .unwrap();
        assert_eq!(actual.logic_hash, expected.logic_hash);
    }
}

#[test]
fn declining_oldzishou_draws_normally_and_enters_play() {
    let setup = setup(2_026_074_201);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, false);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 2);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);

    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Play);
    assert_eq!(state.turn.skipped_phase_count(), 0);
}

#[test]
fn accepting_oldzishou_adds_lost_hp_and_skips_the_upcoming_play_phase() {
    let setup = setup(2_026_074_202);
    let before = setup.state.hand_count(setup.owner);
    let (state, result) = enter_draw(setup.state, setup.owner, true);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.hand_count(setup.owner), before + 4);
    assert_eq!(state.turn.flag(turn_flag::CURRENT_DRAW_COUNT_BONUS), 0);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 1);

    let (state, result) = submit(state, setup.owner, CommandKind::EndPhase, 3);
    assert!(matches!(result.result, CommandResult::Accepted { .. }));
    assert_eq!(state.turn.phase, Phase::Discard);
    assert_eq!(state.turn.flag(turn_flag::SKIP_PLAY), 0);
    assert_eq!(state.turn.skipped_phase_count(), 1);
    assert!(result.events.iter().any(|event| matches!(
        event,
        DomainEvent::PhaseSkipped {
            current,
            phase: TurnPhaseTag::Play,
        } if *current == setup.owner
    )));
}
