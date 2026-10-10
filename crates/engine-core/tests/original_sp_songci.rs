//! Golden coverage for SP 陈琳「颂词」(songci).
//! Play-phase active: selected target draws 2.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::{apply_command, PipelineResult};
use engine_core::effect::{bootstrap_match, run_effects};
use engine_core::state::{GameState, Phase};
use engine_core::transaction::Transaction;
use rule_api::EffectRequest;

const SONGCI: u32 = 2_662_892_097;
const SOURCE: &str = include_str!("../../../content/original/ported/sp/songci.skill");
const PACKAGE: &[u8] = include_bytes!("../../../content/original/ported/sp/sp.npack.json");

fn setup() -> (GameState, PlayerId, PlayerId, SkillId) {
    let compiled = content_compiler::compile_source_named("songci-test", SOURCE).unwrap();
    let mut tx = Transaction::begin(GameState::empty());
    bootstrap_match(&mut tx, 2, 20_260_815_13, "identity-standard").unwrap();
    std::sync::Arc::make_mut(&mut tx.state_mut().content).load_package(&compiled).unwrap();
    tx.state_mut().turn.phase = Phase::Play;
    let owner = tx.state().turn.current;
    let other = tx
        .state()
        .players
        .alive_by_seat()
        .into_iter()
        .find(|player| *player != owner)
        .unwrap();
    let mut effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(SONGCI),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill = tx
        .state()
        .players
        .get(owner)
        .unwrap()
        .skills
        .iter()
        .copied()
        .find(|skill| tx.state().skills.get(*skill).unwrap().def.raw() == SONGCI)
        .unwrap();
    (tx.commit().state, owner, other, skill)
}

fn activate(
    state: GameState,
    owner: PlayerId,
    target: PlayerId,
    skill: SkillId,
    key: u64,
) -> (GameState, PipelineResult) {
    apply_command(
        state,
        Command {
            actor: owner,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(key),
            kind: CommandKind::ActivateSkill {
                skill,
                targets: vec![target],
                card_targets: vec![],
            },
        },
    )
}

#[test]
fn compiled_source_and_npack_publish_the_active_draw() {
    let compiled = content_compiler::compile_source_named("songci", SOURCE).unwrap();
    assert!(compiled
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == SONGCI && skill.name == "songci" && !skill.locked));
    let active = compiled
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SONGCI)
        .expect("songci active");
    assert_eq!(active.window, "play_phase");
    let targets = active.targets.as_ref().unwrap();
    assert_eq!((targets.min, targets.max), (1, 1));

    let committed = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    let pack = committed
        .active_index
        .iter()
        .find(|active| active.skill_def_id == SONGCI)
        .expect("npack songci active");
    assert_eq!(pack.window, "play_phase");
    let actual = committed
        .modules
        .iter()
        .find(|module| module.name == compiled.modules[0].name)
        .expect("npack module for songci");
    assert_eq!(actual.logic_hash, compiled.modules[0].logic_hash);
}

#[test]
fn selected_target_draws_two() {
    let (state, owner, other, skill) = setup();
    let before = state.hand_count(other);
    let (state, result) = activate(state, owner, other, skill, 1);
    assert!(
        matches!(result.result, CommandResult::Accepted { .. }),
        "{:?}",
        result.result
    );
    assert_eq!(state.hand_count(other), before + 2);
}
