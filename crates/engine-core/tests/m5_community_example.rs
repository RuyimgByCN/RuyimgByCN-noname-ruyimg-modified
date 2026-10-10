//! M5 gate 3: a non-official community example pack loads into a standard match.
//!
//! The `hero` community package (namespace `community`) declares a single
//! character with one active skill (出牌阶段限一次摸一张). This golden proves
//! the pack packs, installs, and its skill resolves inside a standard identity
//! match — the acceptance evidence for the M5 「至少一个非官方示例包装载进标准局」 gate.

use engine_api::{
    Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    SkillId,
};
use engine_core::command::apply_command;
use engine_core::effect::run_effects;
use engine_core::state::Phase;
use engine_core::transaction::Transaction;
use engine_core::Engine;
use rule_api::EffectRequest;

const HERO_EXAMPLE_SKILL: u32 = 4_200_000_901;
const HERO_EXAMPLE_CHARACTER: u32 = 4_200_000_902;
const PACKAGE: &[u8] = include_bytes!("../../../content/community/dist/hero.npack.json");

fn start() -> (Engine, PlayerId) {
    let mut engine = Engine::new();
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    engine.load_package(&pkg).unwrap();
    let result = engine.submit(Command {
        actor: PlayerId::NONE,
        expected_revision: ExpectedRevision(0),
        idempotency_key: IdempotencyKey(0),
        kind: CommandKind::StartGame {
            player_count: 2,
            rng_seed: 20_261_301,
            ruleset: "identity-standard".into(),
        },
    });
    assert!(
        matches!(result, CommandResult::Accepted { .. }),
        "community package must load into a standard match: {result:?}"
    );
    let owner = engine.state().turn.current;
    (engine, owner)
}

#[test]
fn community_pack_installs_with_character_and_skill() {
    let pkg = content_schema::ContentPackage::from_json_bytes(PACKAGE).unwrap();
    assert_eq!(pkg.manifest.package_id, "hero");
    assert_eq!(pkg.manifest.namespace, "community");
    assert!(pkg
        .skill_index
        .iter()
        .any(|skill| skill.skill_def_id == HERO_EXAMPLE_SKILL));
    assert!(pkg
        .character_index
        .iter()
        .any(|character| character.id == HERO_EXAMPLE_CHARACTER
            && character.skills == [HERO_EXAMPLE_SKILL]));
}

#[test]
fn community_skill_resolves_inside_the_standard_match() {
    let (engine, owner) = start();
    let mut tx = Transaction::begin(engine.state().clone());
    let effects: Vec<_> = tx
        .state()
        .skills
        .sorted_live()
        .into_iter()
        .map(|skill| EffectRequest::RemoveSkill { skill: skill.id })
        .collect();
    let mut effects = effects;
    effects.push(EffectRequest::AddSkill {
        player: owner,
        skill_def: SkillDefId::new(HERO_EXAMPLE_SKILL),
    });
    run_effects(&mut tx, effects).unwrap();
    let skill: SkillId = tx
        .state()
        .skills
        .sorted_live()
        .iter()
        .find(|skill| skill.def.raw() == HERO_EXAMPLE_SKILL)
        .unwrap()
        .id;
    tx.state_mut().turn.phase = Phase::Play;
    let hand_before = tx.state().hand_count(owner);

    let (state, result) = apply_command(
        tx.commit().state,
        Command {
            actor: owner,
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
        matches!(result.result, CommandResult::Accepted { .. }),
        "community active skill resolves: {:?}",
        result.result
    );
    assert_eq!(
        state.hand_count(owner),
        hand_before + 1,
        "示例技 draws 1 in the standard match"
    );
}
