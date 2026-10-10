use legacy_analyzer::{analyze_source, grade_skill, CompatGrade};

#[test]
fn fixture_grades() {
    let src = include_str!("fixtures/sample_skills.js");
    let profiles = analyze_source(src).expect("extract");
    assert!(profiles.len() >= 4);

    let j = profiles.iter().find(|p| p.name == "jianxiong").unwrap();
    let gj = grade_skill(j);
    assert_eq!(gj.grade, CompatGrade::A, "{:?}", gj.reasons);

    let e = profiles.iter().find(|p| p.name == "evil_dom").unwrap();
    let ge = grade_skill(e);
    assert_eq!(ge.grade, CompatGrade::D);

    let w = profiles.iter().find(|p| p.name == "wusheng").unwrap();
    let gw = grade_skill(w);
    assert!(
        matches!(gw.grade, CompatGrade::B | CompatGrade::C),
        "{:?}",
        gw
    );
}
