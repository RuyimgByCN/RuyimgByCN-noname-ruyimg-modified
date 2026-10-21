//! Golden coverage for KEY「远忆」rewritten from legacy JavaScript semantics.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{turn_flag, GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const GODAN_YUANYI: u32 = 3_610_550_580;
const GODAN_FEIQU: u32 = 3_490_863_084;
const GODAN_XIAOYUAN: u32 = 2_543_675_799;
const KEY_GODAN: u32 = 2_955_525_500;
const SOURCE: &str = include_str!("../../../../content/original/ported/key/godan_yuanyi.skill");
const KEY: &[u8] = include_bytes!("../../../../content/original/ported/key/key.npack.json");

struct Setup {
    state: GameState,
    owner: PlayerId,
    previous: PlayerId,
}

fn setup(round: u32) -> Setup {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(
        &mut tx,
        3,
        GODAN_YUANYI as u64 + u64::from(round),
        "identity-standard",
    )
    .unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content)
        .load_package(&package)
        .unwrap();

    let seats = tx.state().players.alive_by_seat();
    let previous = seats[0];
    let owner = seats[1];
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    for player in seats {
        effects.push(EffectRequest::DiscardFromHand {
            player,
            count: u32::MAX,
        });
    }
    effects.extend([
        EffectRequest::AddSkill {
            player: owner,
            skill_def: SkillDefId::new(GODAN_YUANYI),
        },
        EffectRequest::SetTurnState {
            current: previous,
            phase: Phase::End.to_u8(),
        },
    ]);
    run_effects(&mut tx, effects).unwrap();
    tx.state_mut().turn.round = round;
    Setup {
        state: tx.commit().state,
        owner,
        previous,
    }
}

fn end_phase(state: GameState, actor: PlayerId, key: u64) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::EndPhase,
        },
    )
}

#[test]
fn package_publishes_exact_skill_and_complete_character() {
    let package = content_schema::ContentPackage::from_json_bytes(KEY).unwrap();
    let compiled = content_compiler::compile_source_named("godan_yuanyi", SOURCE).unwrap();
    assert_eq!(compiled.manifest.rule_vm_abi_version, 75);
    assert_eq!(package.manifest.rule_vm_abi_version, 691);
    assert!(package.skill_index.iter().any(|skill| {
        skill.skill_def_id == GODAN_YUANYI && skill.name == "godan_yuanyi" && skill.locked
    }));
    let character = package
        .character_index
        .iter()
        .find(|character| character.id == KEY_GODAN)
        .unwrap();
    assert_eq!(
        character.skills,
        [GODAN_YUANYI, GODAN_FEIQU, GODAN_XIAOYUAN]
    );
    let actual = package
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .unwrap();
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn turn_begin_draws_current_round_capped_at_three_then_opens_extra_play() {
    for (round, expected) in [(1, 1), (2, 2), (5, 3)] {
        let setup = setup(round);
        let (state, result) = end_phase(setup.state, setup.previous, u64::from(round));
        assert!(matches!(result.result, CommandResult::Accepted { .. }));
        assert_eq!(state.turn.current, setup.owner);
        assert_eq!(state.turn.round, round);
        assert_eq!(state.turn.phase, Phase::Play);
        assert_eq!(state.turn.flag(turn_flag::OPENING_EXTRA_PLAY_ACTIVE), 1);
        assert_eq!(state.hand_count(setup.owner), expected);
    }
}
