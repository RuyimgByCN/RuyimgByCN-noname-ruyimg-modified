//! J2 author package lifecycle through the server's existing PackageStore adapter.
#[path = "../src/native.rs"]
#[allow(dead_code)]
mod native;
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand};
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};
static DEDUP: AtomicU64 = AtomicU64::new(1);

fn publish(root: &Path, version: &str, extra_draw: u8) {
    let source = include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill")
        .replace(
            "effect resolve_ol_jie_danshou(self);",
            &format!("effect resolve_ol_jie_danshou(self); effect draw_cards(self, {extra_draw});"),
        );
    let mut package = content_compiler::compile_source_named("j2-author", &source).unwrap();
    package.manifest.version = version.to_string();
    let characters = serde_json::to_vec_pretty(&serde_json::json!({
        "pack":{"name":"j2-author","version":1},"generals":[{"name":"j2-author","hp":4,"gender":"male","faction":"wu","skills":[{"name":"ol_jie_danshou"}]}]
    })).unwrap();
    package.asset_manifest = Some(serde_json::from_value(serde_json::json!({
        "schema_version":1,"package_id":"j2-author","total_compressed_size":0,"total_decoded_size":0,
        "entries":[{"asset_id":"j2-author:characters","source_hash":blake3::hash(&characters).as_bytes(),"kind":"Data","image_variants":[],"audio_variants":[],"priority":"Lazy","depends_on":[],"platforms":[]}]
    })).unwrap());
    package.asset_paths.insert(
        "j2-author:characters".into(),
        "assets/character.json".into(),
    );
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package.validate().unwrap();
    let folder = root.join("j2-author");
    std::fs::create_dir_all(folder.join("assets")).unwrap();
    std::fs::write(folder.join("assets/character.json"), &characters).unwrap();
    std::fs::write(
        folder.join("package.npack.json"),
        package.to_json_bytes().unwrap(),
    )
    .unwrap();
    if let Ok(dir) = std::env::var("J2_EVIDENCE_DIR") {
        let artifact = Path::new(&dir).join(version);
        std::fs::create_dir_all(&artifact).unwrap();
        std::fs::write(
            artifact.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        std::fs::write(artifact.join("character.json"), characters).unwrap();
    }
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn start(pack: &engine_native::content::extension::ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(99107867, 4, &[pack]).unwrap();
    e.assign_general_named(0, "j2-author").unwrap();
    for _ in 0..10 {
        if e.turn().phase == engine_native::Phase::Play {
            break;
        }
        assert!(!matches!(
            submit(&mut e, 0, GameAction::NextPhase),
            CommandOutcome::Rejected(_)
        ));
    }
    let skill = e.catalog().skill_by_name("ol_jie_danshou").unwrap();
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill }),
        CommandOutcome::Pending { .. }
    ));
    e
}

fn finish(e: &mut Engine) -> usize {
    let hand = e.state_core().player(0).hand.len();
    for _ in 0..10 {
        let Some(choice) = e.open_choice().cloned() else {
            return e.state_core().player(0).hand.len() - hand;
        };
        let selection = match choice.spec.prompt {
            "rvm_danshou_option" => vec![O::Tag(4)],
            "rvm_danshou_target" => vec![O::Player(0)],
            "rvm_danshou_discard" => choice
                .spec
                .options
                .iter()
                .take(choice.spec.min)
                .copied()
                .collect(),
            other => panic!("unexpected {other}"),
        };
        assert!(!matches!(
            submit(
                e,
                choice.observer,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection
                }
            ),
            CommandOutcome::Rejected(_)
        ));
    }
    panic!("settlement did not finish");
}

#[test]
fn author_install_start_activate_answer_archive_upgrade_rollback_disable_uninstall() {
    let root = tempfile::tempdir().unwrap();
    let packages = root.path().join("packages");
    std::fs::create_dir_all(&packages).unwrap();
    let content = native::NativeContent::open(packages.clone(), root.path().join("store")).unwrap();
    publish(&packages, "0.1.0", 1);
    content.import().unwrap();
    let v1 = content.load_scoped(Some("official"), "j2-author").unwrap();
    let mut original = start(&v1.pack);
    let archive = engine_native::archive::encode_archive(&original.save());
    let snapshot = engine_native::archive::decode_archive(&archive).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&v1.pack]).unwrap();
    assert_eq!(finish(&mut original), 1);
    assert_eq!(finish(&mut restored), 1);
    assert_eq!(original.state_core(), restored.state_core());
    assert_eq!(original.save().resolution, restored.save().resolution);
    publish(&packages, "0.1.1", 2);
    content.import().unwrap();
    let v2 = content.load_scoped(Some("official"), "j2-author").unwrap();
    assert!(
        Engine::restore_with_extensions(&snapshot, &[&v2.pack]).is_err(),
        "new package cannot replace the frozen old content lock"
    );
    let mut upgraded = start(&v2.pack);
    assert_eq!(finish(&mut upgraded), 2);
    // The old frozen package continues the old game after replacement of the installed active version.
    let mut frozen = Engine::restore_with_extensions(&snapshot, &[&v1.pack]).unwrap();
    assert_eq!(finish(&mut frozen), 1);
    content
        .activate_version("official", "j2-author", "0.1.0")
        .unwrap();
    let rolled_back = content.load_scoped(Some("official"), "j2-author").unwrap();
    assert_eq!(finish(&mut start(&rolled_back.pack)), 1);
    content.disable_package("official", "j2-author").unwrap();
    assert!(content.load_scoped(Some("official"), "j2-author").is_err());
    content.uninstall_package("official", "j2-author").unwrap();
    assert!(content.installed().unwrap().is_empty());
    assert_eq!(
        finish(&mut Engine::restore_with_extensions(&snapshot, &[&v1.pack]).unwrap()),
        1
    );
}
