//! Golden for legacy `charlotte: true` placeholders with no rule behavior.
//!
//! The 1.11.5 delta round replaced every empty DSL body with a non-empty one
//! (catalog placeholder = 0), giving most former charlotte placeholders
//! approximate rule surfaces (documented in each `.skill` as 最小近似 /
//! intentional no-op). Only the embedded standard package still ships pure
//! metadata-only skills, so this guard now applies to standard only.
const CASES: &[(&[u8], &[u32])] = &[
    (
        include_bytes!("../../../content/standard/standard.npack.json"),
        &[
            2_475_334_928,
            2_543_104_437,
            3_282_332_367,
            2_804_212_292,
            3_648_794_780,
        ],
    ),
];

#[test]
fn metadata_only_skills_are_packaged_without_rule_indexes() {
    for (bytes, ids) in CASES {
        let package = content_schema::ContentPackage::from_json_bytes(bytes).unwrap();
        for id in *ids {
            let skill = package
                .skill_index
                .iter()
                .find(|skill| skill.skill_def_id == *id)
                .unwrap_or_else(|| panic!("skill {id} must be packaged"));
            assert!(!skill.locked);
            assert!(!package
                .trigger_index
                .iter()
                .any(|entry| entry.skill_def_id == *id));
            assert!(!package
                .modifier_index
                .iter()
                .any(|entry| entry.skill_def_id == *id));
            assert!(!package
                .active_index
                .iter()
                .any(|entry| entry.skill_def_id == *id));
            assert!(!package
                .view_as_index
                .iter()
                .any(|entry| entry.skill_def_id == *id));
        }
    }
}
