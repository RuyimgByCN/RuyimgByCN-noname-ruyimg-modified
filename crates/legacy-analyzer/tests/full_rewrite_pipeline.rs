//! Full rewrite pipeline test (architecture §36).
//!
//! Verifies: legacy JS → analyze → grade → skeleton → compile → engine load.
//! The generated DSL is compilable, not just a skeleton.

use content_compiler::compile_source;
use engine_api::{Command, CommandKind, ExpectedRevision, IdempotencyKey, PlayerId};
use engine_core::{invariants::check_invariants, Engine};
use legacy_analyzer::analyze_source;
use legacy_analyzer::{generate_dsl_skeleton, grade_skill, CompatGrade};

const JIANXIONG_JS: &str = include_str!("../../../tests/legacy_fixtures/jianxiong_legacy.js");
const MASHU_JS: &str = include_str!("../../../tests/legacy_fixtures/mashu_legacy.js");

fn extract(src: &str) -> Vec<legacy_analyzer::SkillProfile> {
    analyze_source(src).unwrap_or_default()
}

#[test]
fn jianxiong_skeleton_compiles() {
    let profiles = extract(JIANXIONG_JS);
    let p = profiles.iter().find(|p| p.name == "jianxiong").unwrap();
    let skeleton = generate_dsl_skeleton(p, 1);
    let pkg = compile_source(&skeleton).expect("jianxiong skeleton must compile");
    assert!(!pkg.trigger_index.is_empty(), "must have triggers");
}

#[test]
fn mashu_skeleton_compiles() {
    let profiles = extract(MASHU_JS);
    let p = profiles.iter().find(|p| p.name == "mashu").unwrap();
    let skeleton = generate_dsl_skeleton(p, 2);
    let pkg = compile_source(&skeleton).expect("mashu skeleton must compile");
    assert!(!pkg.modifier_index.is_empty(), "must have modifiers");
}

#[test]
fn full_pipeline_loads_into_engine() {
    for (source, name, def_id) in [(JIANXIONG_JS, "jianxiong", 1u32), (MASHU_JS, "mashu", 2u32)] {
        // Step 1: Analyze
        let profiles = extract(source);
        let p = profiles.iter().find(|p| p.name == name).unwrap();
        // Step 2: Grade
        let detail = grade_skill(p);
        assert!(
            !matches!(detail.grade, CompatGrade::D),
            "{name} should not be D"
        );
        // Step 3: Skeleton
        let skeleton = generate_dsl_skeleton(p, def_id);
        // Step 4: Compile
        let pkg = compile_source(&skeleton).unwrap_or_else(|e| panic!("{name} compile: {e}"));
        // Step 5: Load into engine
        let mut eng = Engine::new();
        eng.load_package(&pkg)
            .unwrap_or_else(|e| panic!("{name} load: {e}"));
        // Step 6: Start game
        eng.submit(Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: 4,
                rng_seed: 42,
                ruleset: "identity-standard".into(),
            },
        });
        check_invariants(eng.state()).unwrap();
    }
}

#[test]
fn pipeline_logic_hash_reproducible() {
    let profiles = extract(JIANXIONG_JS);
    let p = profiles.iter().find(|p| p.name == "jianxiong").unwrap();
    let skel = generate_dsl_skeleton(p, 1);
    let pkg1 = compile_source(&skel).unwrap();
    let pkg2 = compile_source(&skel).unwrap();
    assert_eq!(pkg1.manifest.logic_hash, pkg2.manifest.logic_hash);
}
