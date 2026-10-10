//! Batch migration test (architecture §36) — 10 skills in one file.
//!
//! Verifies the analyzer can process a realistic batch of legacy skills,
//! grade each one, and generate DSL skeletons — the core §36.2 pipeline.

use legacy_analyzer::analyze_source;
use legacy_analyzer::{generate_dsl_skeleton, grade_skill, CompatGrade, SkillProfile};

const BATCH_JS: &str = include_str!("../../../tests/legacy_fixtures/batch_skills.js");

fn extract(src: &str) -> Vec<SkillProfile> {
    analyze_source(src).unwrap_or_default()
}

#[test]
fn batch_extracts_10_skills() {
    let profiles = extract(BATCH_JS);
    let names: Vec<&str> = profiles.iter().map(|p| p.name.as_str()).collect();
    assert!(
        profiles.len() >= 8,
        "should extract ≥8 of 10 skills, got {}: {:?}",
        profiles.len(),
        names
    );
    // Check key skills are found.
    for expected in &["yiji", "mashu2", "longdan2", "ganglie2"] {
        assert!(
            profiles.iter().any(|p| p.name == *expected),
            "must find {expected} in batch"
        );
    }
}

#[test]
fn batch_grades_and_skeletons_all_skills() {
    let profiles = extract(BATCH_JS);
    assert!(!profiles.is_empty(), "batch must yield skills");

    for p in &profiles {
        let detail = grade_skill(p);
        let skeleton = generate_dsl_skeleton(p, 1);

        // Every skill must produce a non-empty skeleton.
        assert!(
            !skeleton.is_empty(),
            "skeleton for {} must not be empty",
            p.name
        );
        assert!(
            skeleton.contains(&p.name) || skeleton.contains("skill"),
            "skeleton for {} must reference the skill",
            p.name
        );

        // Grade should be documented (not panic).
        let _ = format!("{:?}", detail.grade);
    }
}

#[test]
fn batch_trigger_skills_detected() {
    let profiles = extract(BATCH_JS);
    let trigger_skills: Vec<&SkillProfile> =
        profiles.iter().filter(|p| !p.triggers.is_empty()).collect();
    assert!(
        !trigger_skills.is_empty(),
        "batch must contain trigger skills"
    );
}

#[test]
fn batch_mod_skills_detected() {
    let profiles = extract(BATCH_JS);
    let mod_skills: Vec<&SkillProfile> = profiles.iter().filter(|p| p.has_mod).collect();
    assert!(!mod_skills.is_empty(), "batch must contain mod skills");
}

#[test]
fn batch_viewas_skills_detected() {
    let profiles = extract(BATCH_JS);
    let viewas_skills: Vec<&SkillProfile> = profiles.iter().filter(|p| p.has_view_as).collect();
    assert!(
        !viewas_skills.is_empty(),
        "batch must contain viewAs skills"
    );
}

#[test]
fn batch_ai_skills_detected() {
    let profiles = extract(BATCH_JS);
    let ai_skills: Vec<&SkillProfile> = profiles.iter().filter(|p| p.has_ai).collect();
    assert!(!ai_skills.is_empty(), "batch must contain AI skills");
}

#[test]
fn batch_group_skills_detected() {
    let profiles = extract(BATCH_JS);
    let group_skills: Vec<&SkillProfile> = profiles.iter().filter(|p| p.has_group).collect();
    assert!(!group_skills.is_empty(), "batch must contain group skills");
}
