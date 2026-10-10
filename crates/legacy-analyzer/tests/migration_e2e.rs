//! Legacy migration end-to-end test (architecture §36).

use legacy_analyzer::analyze_source;
use legacy_analyzer::{generate_dsl_skeleton, grade_skill, CompatGrade, SkillProfile};

const JIANXIONG_JS: &str = include_str!("../../../tests/legacy_fixtures/jianxiong_legacy.js");
const MASHU_JS: &str = include_str!("../../../tests/legacy_fixtures/mashu_legacy.js");
const WUSHENG_JS: &str = include_str!("../../../tests/legacy_fixtures/wusheng_legacy.js");

fn find_skill<'a>(profiles: &'a [SkillProfile], name: &str) -> &'a SkillProfile {
    profiles
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("must find {name}"))
}

fn extract(src: &str) -> Vec<SkillProfile> {
    analyze_source(src).unwrap_or_default()
}

#[test]
fn analyze_jianxiong_legacy() {
    let profiles = extract(JIANXIONG_JS);
    let p = find_skill(&profiles, "jianxiong");
    assert!(p.forced, "jianxiong should be forced");
    assert!(!p.triggers.is_empty(), "should have triggers");
}

#[test]
fn analyze_mashu_legacy_has_mod() {
    let profiles = extract(MASHU_JS);
    let p = find_skill(&profiles, "mashu");
    assert!(p.has_mod, "mashu should have mod");
}

#[test]
fn analyze_wusheng_legacy_has_viewas() {
    let profiles = extract(WUSHENG_JS);
    let p = find_skill(&profiles, "wusheng");
    assert!(p.has_view_as, "wusheng should have viewAs");
}

#[test]
fn grade_jianxiong_is_migratable() {
    let profiles = extract(JIANXIONG_JS);
    let p = find_skill(&profiles, "jianxiong");
    let detail = grade_skill(p);
    assert!(
        detail.grade != CompatGrade::D,
        "should not be D: {:?}",
        detail.reasons
    );
}

#[test]
fn skeleton_generation_produces_valid_dsl() {
    let profiles = extract(JIANXIONG_JS);
    let p = find_skill(&profiles, "jianxiong");
    let skel = generate_dsl_skeleton(p, 1);
    assert!(skel.contains("skill"));
    assert!(skel.contains("jianxiong"));
}

#[test]
fn full_pipeline_all_three_fixtures() {
    let cases: &[(&str, &str)] = &[
        (JIANXIONG_JS, "jianxiong"),
        (MASHU_JS, "mashu"),
        (WUSHENG_JS, "wusheng"),
    ];
    for (src, name) in cases {
        let profiles = extract(src);
        let p = find_skill(&profiles, name);
        let detail = grade_skill(p);
        let skel = generate_dsl_skeleton(p, 1);
        assert!(
            !skel.is_empty() && skel.contains(name),
            "skeleton for {name}"
        );
        assert!(
            detail.grade != CompatGrade::D,
            "{name} should not be D: {:?}",
            detail.reasons
        );
    }
}
