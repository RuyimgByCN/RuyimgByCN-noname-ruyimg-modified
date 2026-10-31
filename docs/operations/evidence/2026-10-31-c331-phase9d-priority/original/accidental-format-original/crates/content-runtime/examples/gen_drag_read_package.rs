//! L16 拖拽读取转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「拖拽读取」扩展转换为 `discover_extensions` 可发现的
//! 真实包（manifest.json + package.npack.json + assets/config.json），写入
//! 仓库 `packages/extensions/drag-read-upstream-liuli`。
//!
//! 数据来源（只读自上游工作区）：
//! - noname-liuli `noname/extension/拖拽读取/extension.js` @ 8485dbb2
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_drag_read_package`
//!
//! 与 cardpile 生成器不同，本包无卡牌/武将，只有一份配置资产（`config.json`），
//! 供 `content-native-loader::drag_read` 解析；`logic_hash`/`asset_hash` 由
//! `ContentPackage::compute_*` 计算，保证与装载校验一致。

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use rule_api::CapabilitySet;
use std::collections::BTreeMap;
use std::path::PathBuf;

const UPSTREAM_COMMIT: &str = "8485dbb29ff16f9b258a766b1a17ef75f883aade";

fn config_json() -> String {
    // 字段顺序固定，便于逐字节复核。
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/拖拽读取/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"upstream_version\": \"1.81\",\n");
    out.push_str(
        "  \"intro\": \"Windows电脑版专属扩展，把扩展包（zip或7z文件）拖入到游戏内即可导入\",\n",
    );
    out.push_str("  \"author\": \"诗笺\",\n");
    out.push_str("  \"native_plays\": [\"coin\", \"boss\", \"wuxing\", \"cardpile\"],\n");
    out.push_str("  \"complete_folders\": [\"card\", \"character\", \"font\", \"game\", \"layout\", \"mode\", \"theme\"],\n");
    out.push_str(
        "  \"complete_files\": [\"app.html\", \"index.html\", \"LICENSE\", \"README.md\"],\n",
    );
    out.push_str("  \"audio_subfolders\": [\"background\", \"card\", \"die\", \"effect\", \"skill\", \"voice\"],\n");
    out.push_str("  \"image_subfolders\": [\"background\", \"card\", \"character\", \"emotion\", \"mode\", \"splash\"],\n");
    out.push_str("  \"extension_subfolders\": [\"boss\", \"cardpile\", \"coin\", \"wuxing\"],\n");
    out.push_str("  \"node_modules_subfolders\": [\"options\", \"ultron\", \"ws\"]\n");
    out.push_str("}\n");
    out
}

fn build_package(config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "drag-read", "1.0.0");
    manifest.capabilities = CapabilitySet {
        emit_effects: false,
        run_queries: false,
        request_choice: false,
        read_skill_state: false,
        write_skill_state: false,
        read_marks_self: false,
        read_public_seats: false,
        _reserved: (),
    };
    let mut package = ContentPackage {
        manifest,
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([("drag-read".to_string(), "拖拽读取".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("drag-read");
    assets.add_entry(AssetEntry {
        asset_id: "drag-read:config".into(),
        source_hash: *hash(config.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("drag-read:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("drag-read-upstream-liuli");
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("manifest.json"),
        "{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}\n",
    )
    .unwrap();
    let config = config_json();
    std::fs::write(dir.join("assets/config.json"), &config).unwrap();
    let package = build_package(&config);
    std::fs::write(
        dir.join("package.npack.json"),
        package.to_json_bytes().unwrap(),
    )
    .unwrap();
    println!("written {}", dir.display());
}
