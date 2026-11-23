//! Batch U05-4 (`U05 杀海拾遗` `xianjian` 子包 = 36 将 120 技) 真实事件链与审修验收测试。

#[path = "u05_b4_xianjian/common.rs"]
mod common;
#[path = "u05_b4_xianjian/part1_batch_a.rs"]
mod part1_batch_a;
#[path = "u05_b4_xianjian/part2_batch_b.rs"]
mod part2_batch_b;
#[path = "u05_b4_xianjian/part3_batch_c_and_d.rs"]
mod part3_batch_c_and_d;
#[path = "u05_b4_xianjian/part4_audited_fixes.rs"]
mod part4_audited_fixes;
#[path = "u05_b4_xianjian/part5_round3_fixes.rs"]
mod part5_round3_fixes;
#[path = "u05_b4_xianjian/part6_round4_fixes.rs"]
mod part6_round4_fixes;

use common::*;
use content_native_loader::{U05_XIANJIAN_ALL_SKILLS, U05_XIANJIAN_GENERALS_FULL};

#[test]
fn u05_b4_all_36_generals_and_120_skills_bound_without_vm_stubs() {
    let (_verified, e) = start_xianjian_only(8400, 2);
    let cat = e.catalog();
    assert_eq!(U05_XIANJIAN_GENERALS_FULL.len(), 36);
    assert_eq!(U05_XIANJIAN_ALL_SKILLS.len(), 120);

    for (gen_name, _, _, _, sks) in U05_XIANJIAN_GENERALS_FULL {
        let found = (0..cat.generals_count_compat())
            .filter_map(|i| cat.general_meta(i as u16))
            .find(|(n, ..)| *n == gen_name);
        assert!(
            found.is_some() || cat.extension_generals.iter().any(|g| g.name == gen_name),
            "general missing: {gen_name}"
        );
        for &sk in sks {
            assert!(cat.skill_by_name(sk).is_some(), "general {gen_name} skill {sk} missing");
        }
    }

    for &sk in &U05_XIANJIAN_ALL_SKILLS {
        let def = cat.skill_by_name(sk).unwrap_or_else(|| panic!("skill missing: {sk}"));
        let s = &cat.skills[def as usize];
        assert!(
            s.behavior.package_skill_id().is_none(),
            "xianjian skill {sk} still using coarse VM dispatch instead of native HostBehavior"
        );
    }
}
