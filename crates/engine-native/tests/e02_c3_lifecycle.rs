//! E02/C3 生命周期跨层场景（演进提案 C3、台账 E02/G1）：
//! 包升级与内容锁绑定的正反例——
//! ① 安装 v1 → 建局保存（内容锁绑定 v1 集合）；
//! ② 升级 v2 后用 v2 集合恢复原快照：**明确拒绝**（错版不静默兼容）；
//! ③ 回滚（旧版保留/重取）后原快照恢复成功。
//!
//! 包为测试自构造的最小自描述 cards.json 包（ADR 0778），两版仅
//! `pack.version` 不同；npack 的资产/逻辑哈希按同源 API 计算，随内容更新。

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use engine_native::content::extension::ExtensionPack;
use engine_native::{Engine, SnapshotError};

const NAMESPACE: &str = "test-ns";
const PACKAGE_ID: &str = "life-pack";

fn cards_json(version: u32) -> String {
    format!(
        r#"{{
  "source": "test",
  "pack": {{ "name": "life-pack-upstream", "version": {version}, "description": "生命周期场景包", "card_prefix": "lf_" }},
  "cards": [
    {{ "name": "sha", "kind": "sha", "suit": "spade", "rank": 7 }}
  ]
}}
"#
    )
}

fn npack_bytes(version: u32, json: &str) -> Vec<u8> {
    let mut pkg = ContentPackage {
        manifest: PackageManifest::new(NAMESPACE, PACKAGE_ID, &format!("{version}.0.0")),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: Default::default(),
        asset_manifest: None,
        asset_paths: Default::default(),
        compiler_version: "test".into(),
    };
    let mut assets = AssetManifest::new(PACKAGE_ID);
    assets.add_entry(AssetEntry {
        asset_id: format!("{PACKAGE_ID}:cards"),
        source_hash: *blake3::hash(json.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    pkg.asset_manifest = Some(assets);
    pkg.asset_paths
        .insert(format!("{PACKAGE_ID}:cards"), "assets/cards.json".into());
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    pkg.to_json_bytes().unwrap()
}

fn write_ext(root: &std::path::Path, version: u32) {
    let dir = root.join(PACKAGE_ID);
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    let manifest = format!(
        "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
    );
    std::fs::write(dir.join("manifest.json"), manifest).unwrap();
    let json = cards_json(version);
    std::fs::write(dir.join("assets/cards.json"), &json).unwrap();
    std::fs::write(dir.join("package.npack.json"), npack_bytes(version, &json)).unwrap();
}

fn bridge(json: &str) -> ExtensionPack {
    content_native_loader::bridge_pack(json).expect("装载").pack
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

#[test]
fn e02_c3_upgrade_rebinds_lock_and_rollback_restores_v1() {
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_ext(root, 1);
    let v1_json = cards_json(1);
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let discovered = discover_extensions(root).unwrap();
    assert_eq!(discovered.len(), 1, "测试包可发现");
    let _ = import_extensions(&store, root, &options()).unwrap();
    let pack_v1 = bridge(&v1_json);

    // ① v1 建局并保存：内容锁绑定 v1 集合。
    let e = Engine::new_with_extensions(5201, 3, &[&pack_v1]).expect("v1 建局");
    let snapshot = e.save();

    // ② 升级 v2：内容锁随 name:version 变化；错版恢复明确拒绝。
    write_ext(root, 2);
    let v2_json = cards_json(2);
    let _ = import_extensions(&store, root, &options()).unwrap();
    let pack_v2 = bridge(&v2_json);
    assert_eq!(pack_v2.def.version, 2, "v2 已激活");
    assert!(
        matches!(
            Engine::restore_with_extensions(&snapshot, &[&pack_v2]),
            Err(SnapshotError::RuleVersionMismatch { .. })
        ),
        "内容锁/规则版本必须拒绝错版恢复"
    );

    // ③ 回滚（旧版保留/重取）：激活集回到 v1，原快照恢复成功并继续对局。
    let rolled = store.rollback().unwrap();
    assert!(
        rolled.packages.values().any(|v| v == "1.0.0"),
        "回滚应回到 v1 激活集"
    );
    let active_bytes = store
        .load_active_package_bytes(NAMESPACE, PACKAGE_ID)
        .expect("回滚后 v1 npack 仍可重取");
    assert!(String::from_utf8_lossy(&active_bytes).contains("1.0.0"));
    let mut e2 = Engine::restore_with_extensions(&snapshot, &[&pack_v1]).expect("回滚后恢复");
    // 恢复后继续对局：推进一次出牌阶段收尾。
    let actor = e2.turn().current;
    let _ = e2.submit(engine_native::GameCommand {
        actor,
        expected_revision: e2.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
}

/// C3 卸载进行中对局（演进提案 C3 剩余场景）：卸载只影响后续开局，
/// 进行中对局在内容锁下自洽——
/// ① 装载建局（锁绑定包集合）→ 推进到对局中；
/// ② 宿主注册表卸载：活局不受影响（引擎持有 Arc 目录，无注册表回写
///    通道），扩展内容仍可查、命令照常提交、锁指纹不变；
/// ③ 挂起对局恢复的锁绑定：卸载后（空集合/标准目录）恢复被拒；
///    带原包集合恢复成功（存档字节自包含）——宿主须保留包字节直到
///    无对局引用（登记宿主策略，引擎不感知宿主引用计数）；
/// ④ 卸载后的新开局（按卸载后集合建局）不含该扩展内容。
#[test]
fn e03_c3_uninstall_during_active_game_lock_pinned() {
    use engine_native::content::extension::{content_lock_id, ExtensionDef, ExtensionRegistry};
    let tmp = tempfile::tempdir().unwrap();
    let root = tmp.path();
    write_ext(root, 1);
    let v1_json = cards_json(1);
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let _ = import_extensions(&store, root, &options()).unwrap();
    let pack_v1 = bridge(&v1_json);

    // 宿主注册表：安装（默认启用）。
    let mut reg = ExtensionRegistry::new();
    reg.install(ExtensionDef {
        name: "life-pack-upstream",
        version: 1,
        description: "生命周期场景包",
    })
    .unwrap();

    // ① 建局并推进到对局中（锁绑定 v1 集合）。
    let mut e = Engine::new_with_extensions(5301, 3, &[&pack_v1]).expect("建局");
    let lock_pinned = content_lock_id(&[&pack_v1]);
    let actor = e.turn().current;
    let _ = e.submit(engine_native::GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });
    let snapshot = e.save();

    // ② 宿主卸载：活局不受影响。
    reg.uninstall("life-pack-upstream").unwrap();
    assert!(!reg.is_installed("life-pack-upstream"), "注册表已卸载");
    let ext_card = e.catalog().cards.iter().filter(|c| c.name.starts_with("lf_")).count();
    assert_eq!(ext_card, 1, "活局目录仍含扩展牌（Arc 目录不受注册表影响）");
    let actor = e.turn().current;
    let out = e.submit(engine_native::GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: 2,
        action: engine_native::GameAction::NextPhase,
    });
    assert!(
        matches!(out, engine_native::CommandOutcome::Accepted { .. } | engine_native::CommandOutcome::Pending { .. }),
        "卸载后活局继续可提交: {out:?}"
    );
    assert_eq!(e.save().core.ruleset_version, snapshot.core.ruleset_version, "活局规则版本不变");
    assert_eq!(content_lock_id(&[&pack_v1]), lock_pinned, "锁指纹不变");

    // ③ 锁绑定：卸载后（空集合=标准目录）恢复挂起对局被拒；
    //    带原包集合恢复成功（存档自包含，不依赖注册表）。
    assert!(
        matches!(
            Engine::restore(&snapshot),
            Err(SnapshotError::RuleVersionMismatch { .. })
        ),
        "卸载后的标准目录不得恢复扩展对局"
    );
    let mut e2 = Engine::restore_with_extensions(&snapshot, &[&pack_v1]).expect("原包集合恢复成功");
    let actor = e2.turn().current;
    let _ = e2.submit(engine_native::GameCommand {
        actor,
        expected_revision: e2.revision(),
        dedup_id: 1,
        action: engine_native::GameAction::NextPhase,
    });

    // ④ 卸载后的新开局：扩展牌不在目录（宿主按卸载后集合建局）。
    let e3 = Engine::new(5302, 3);
    assert!(
        !e3.catalog().cards.iter().any(|c| c.name.starts_with("lf_")),
        "新开局不含已卸载扩展的牌"
    );
}
