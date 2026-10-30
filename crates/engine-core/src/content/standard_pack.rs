//! Embed and load the official M1 standard skill package.

use super::ContentRegistry;
use content_compiler::compile_source_named;
use content_schema::ContentPackage;
use std::{
    collections::BTreeSet,
    sync::{Arc, OnceLock},
};

const STANDARD_M1_SRC: &str = include_str!("../../../../content/standard/standard_m1.skill");

static STANDARD_PKG: OnceLock<ContentPackage> = OnceLock::new();
static STANDARD_CONTENT: OnceLock<Arc<ContentRegistry>> = OnceLock::new();

pub(crate) fn ensure_standard_content(content: &mut Arc<ContentRegistry>) {
    if content.as_ref() == &ContentRegistry::new() {
        // Only the exact empty registry can reuse the fully validated immutable catalog.
        *content = STANDARD_CONTENT
            .get_or_init(|| {
                let mut registry = ContentRegistry::new();
                ensure_standard_package(&mut registry);
                Arc::new(registry)
            })
            .clone();
    } else {
        ensure_standard_package(Arc::make_mut(content));
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Engine;
    use engine_api::{
        Command, CommandKind, CommandResult, ExpectedRevision, IdempotencyKey, PlayerId, SkillDefId,
    };
    use std::sync::Arc;

    fn start(seed: u64, players: u8) -> Command {
        Command {
            actor: PlayerId::NONE,
            expected_revision: ExpectedRevision(0),
            idempotency_key: IdempotencyKey(0),
            kind: CommandKind::StartGame {
                player_count: players,
                rng_seed: seed,
                ruleset: "identity-standard".into(),
            },
        }
    }

    #[test]
    fn shared_standard_catalog_preserves_full_start_and_replay() {
        let mut reference = ContentRegistry::new();
        ensure_standard_package(&mut reference);
        for (players, seed) in [(2, 1), (4, 42), (8, 99)] {
            let mut shared = Engine::new();
            let mut rebuilt = Engine::new();
            rebuilt.state_mut_for_tests().content = Arc::new(reference.clone());
            let result = shared.submit(start(seed, players));
            assert!(matches!(result, CommandResult::Accepted { .. }));
            assert_eq!(result, rebuilt.submit(start(seed, players)));
            assert_eq!(shared.state(), rebuilt.state());
            assert_eq!(shared.last_events(), rebuilt.last_events());
            assert_eq!(shared.last_choice(), rebuilt.last_choice());
            assert_eq!(shared.manifest(), rebuilt.manifest());
            assert_eq!(
                shared.export_replay().unwrap(),
                rebuilt.export_replay().unwrap()
            );
            let snapshot = shared.snapshot();
            let mut restored = Engine::new();
            restored.restore_snapshot(&snapshot).unwrap();
            assert_eq!(restored.state(), shared.state());
            assert_eq!(restored.state_hash(), shared.state_hash());
        }
    }

    #[test]
    fn standard_catalog_sharing_keeps_match_state_and_package_writes_independent() {
        let mut a = Engine::new();
        let mut b = Engine::new();
        a.submit(start(42, 4));
        b.submit(start(42, 4));
        assert!(Arc::ptr_eq(&a.state().content, &b.state().content));
        let before = b.state().clone();
        a.state_mut_for_tests()
            .players
            .get_mut(PlayerId::new(1))
            .unwrap()
            .hp -= 1;
        let package = compile_source_named(
            "private-extra",
            "skill private_extra(4000000000) { modifier distance_from add 1; }",
        )
        .unwrap();
        a.load_package(&package).unwrap();
        assert!(!Arc::ptr_eq(&a.state().content, &b.state().content));
        assert!(a
            .content()
            .skill_metadata(SkillDefId::new(4_000_000_000))
            .is_some());
        assert_eq!(b.state(), &before);
        let mut c = Engine::new();
        c.submit(start(42, 4));
        assert_eq!(c.state(), &before);
        assert!(Arc::ptr_eq(&b.state().content, &c.state().content));
    }

    #[test]
    fn dynamic_and_noncanonical_empty_catalogs_keep_existing_merge_semantics() {
        let package = compile_source_named(
            "override",
            "skill jianxiong(1) locked { trigger damage owner_is_target { effect draw_cards(self, 2); } }",
        ).unwrap();
        let mut dynamic = ContentRegistry::new();
        dynamic.load_package(&package).unwrap();
        let mut altered_empty = ContentRegistry::new();
        altered_empty.combined_logic_hash = [7; 32];
        for registry in [dynamic, ContentRegistry::default(), altered_empty] {
            let mut reference = registry.clone();
            ensure_standard_package(&mut reference);
            let mut shared = Arc::new(registry);
            ensure_standard_content(&mut shared);
            assert_eq!(*shared, reference);
        }
        let mut overridden = Engine::new();
        overridden.load_package(&package).unwrap();
        assert!(matches!(
            overridden.submit(start(42, 4)),
            CommandResult::Accepted { .. }
        ));
        assert!(
            overridden
                .content()
                .skill_metadata(SkillDefId::new(1))
                .unwrap()
                .locked
        );
        assert_eq!(overridden.content().package_count(), 2);
    }
}
