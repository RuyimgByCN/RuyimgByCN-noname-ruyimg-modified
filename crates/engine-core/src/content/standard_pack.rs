//! Embed and load the official M1 standard skill package.

use super::ContentRegistry;
use content_compiler::compile_source_named;
use content_schema::ContentPackage;
use std::{collections::BTreeSet, sync::OnceLock};

const STANDARD_M1_SRC: &str = include_str!("../../../../content/standard/standard_m1.skill");

static STANDARD_PKG: OnceLock<ContentPackage> = OnceLock::new();

/// Compiled standard M1 package (lazy, process-wide).
pub fn standard_package() -> &'static ContentPackage {
    STANDARD_PKG.get_or_init(|| {
        let mut package = compile_source_named("standard", STANDARD_M1_SRC)
            .expect("standard_m1.skill must compile");
        package.character_index = super::characters::standard_character_entries();
        package.manifest.logic_hash = package.compute_logic_hash();
        package
    })
}

/// Ensure the standard package is loaded into a registry (idempotent by logic_hash).
pub fn ensure_standard_package(reg: &mut ContentRegistry) {
    if super::characters::STANDARD_CHARACTERS
        .iter()
        .all(|character| reg.character(character.id).is_some())
    {
        return;
    }
    let mut package = standard_package().clone();
    let provided: BTreeSet<_> = package
        .skill_index
        .iter()
        .filter(|skill| {
            reg.skill_metadata(engine_api::SkillDefId::new(skill.skill_def_id))
                .is_some()
        })
        .map(|skill| skill.skill_def_id)
        .collect();
    package
        .skill_index
        .retain(|skill| !provided.contains(&skill.skill_def_id));
    package
        .trigger_index
        .retain(|trigger| !provided.contains(&trigger.skill_def_id));
    package
        .modifier_index
        .retain(|modifier| !provided.contains(&modifier.skill_def_id));
    package
        .active_index
        .retain(|active| !provided.contains(&active.skill_def_id));
    package
        .view_as_index
        .retain(|view_as| !provided.contains(&view_as.skill_def_id));
    // Roster closure (ADR 0542 delta): gongsunzan / old_re_lidian reference
    // refresh-package skills (reyicong / xunxun / wangxi) cross-package, so the
    // embedded standard declares the refresh dependency and defers character
    // skill validation to the full content lock (like every other extension).
    package.manifest.dependencies = vec![content_schema::PackageDependency {
        namespace: "official".into(),
        package_id: "refresh".into(),
        version: "0.1.0".into(),
        logic_hash: [0u8; 32],
    }];
    package.manifest.logic_hash = package.compute_logic_hash();
    reg.load_package(&package)
        .expect("embedded standard content must be internally valid");
}

/// Combined guozhan skill DSL sources (ADR 0604).
const GUOZHAN_SKILL_SRC: &str = concat!(
    include_str!("../../../../content/guozhan/gz_jianxiong.skill"),
    "\n",
    include_str!("../../../../content/guozhan/gz_wei_skills.skill"),
    "\n",
    include_str!("../../../../content/guozhan/gz_other_skills.skill"),
    "\n",
    include_str!("../../../../content/guozhan/gz_batch_all.skill"),
    "\n",
    include_str!("../../../../content/guozhan/gz_batch_nounderscore.skill"),
    "\n",
    include_str!("../../../../content/guozhan/gz_batch_nongz.skill"),
);

static GUOZHAN_PKG: OnceLock<ContentPackage> = OnceLock::new();

/// Compiled guozhan skill package (lazy, process-wide).
pub fn guozhan_package() -> &'static ContentPackage {
    GUOZHAN_PKG.get_or_init(|| {
        let mut package = compile_source_named("guozhan", GUOZHAN_SKILL_SRC)
            .expect("guozhan skill DSL must compile");
        package.character_index = super::characters::guozhan_character_entries();
        package.card_index = super::characters::guozhan_card_entries();
        package.manifest.logic_hash = package.compute_logic_hash();
        package
    })
}

/// Ensure the guozhan package is loaded into a registry (idempotent by logic_hash).
///
/// Loads the 256 gz_ characters + their compiled skills so `bootstrap_guozhan` deals from
/// the guozhan pool. Identity matches never call this, so identity content is unchanged.
pub fn ensure_guozhan_package(reg: &mut ContentRegistry) {
    if super::characters::GUOZHAN_CHARACTERS
        .iter()
        .all(|character| reg.character(character.id).is_some())
    {
        return;
    }
    let mut package = guozhan_package().clone();
    let provided: BTreeSet<_> = package
        .skill_index
        .iter()
        .filter(|skill| {
            reg.skill_metadata(engine_api::SkillDefId::new(skill.skill_def_id))
                .is_some()
        })
        .map(|skill| skill.skill_def_id)
        .collect();
    package
        .skill_index
        .retain(|skill| !provided.contains(&skill.skill_def_id));
    package
        .trigger_index
        .retain(|trigger| !provided.contains(&trigger.skill_def_id));
    package
        .modifier_index
        .retain(|modifier| !provided.contains(&modifier.skill_def_id));
    package
        .active_index
        .retain(|active| !provided.contains(&active.skill_def_id));
    package
        .view_as_index
        .retain(|view_as| !provided.contains(&view_as.skill_def_id));
    package.manifest.logic_hash = package.compute_logic_hash();
    reg.load_package(&package)
        .expect("embedded guozhan content must be internally valid");
}
