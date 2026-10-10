use legacy_analyzer::CapabilityMatrix;

#[test]
fn committed_matrix_covers_standard_and_nonstandard_samples() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../docs/content-sdk/skill-capability-matrix.json"
    );
    let matrix: CapabilityMatrix =
        serde_json::from_slice(&std::fs::read(path).expect("read capability matrix"))
            .expect("parse capability matrix");
    assert!(matrix.total_skills >= 100);
    assert_eq!(matrix.total_skills, matrix.skills.len());
    assert!(matrix.source_counts.get("standard").copied().unwrap_or(0) >= 60);
    assert!(matrix.source_counts.get("shenhua").copied().unwrap_or(0) >= 30);
    assert!(matrix
        .skills
        .iter()
        .all(|row| !row.required_surfaces.is_empty() || !row.gaps.is_empty()));
}
