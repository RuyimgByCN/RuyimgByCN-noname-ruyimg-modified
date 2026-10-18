//! L01 AI优化转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入 `packages/extensions/ai-tuning-upstream-liuli`。
//!
//! 本批落地**AI 调参面**：威胁度表 `cf`、权重表 `qz`、补充模式（`applyCf`/`ckQz`）、
//! 效益结算 `qzCf` 与展示开关；身份/模式专项策略与设置菜单交互为登记差异。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_ai-tuning_package`

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use rule_api::CapabilitySet;
use std::collections::BTreeMap;
use std::path::PathBuf;

const UPSTREAM_COMMIT: &str = "8485dbb29ff16f9b258a766b1a17ef75f883aade";

fn config_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/AI优化/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"cf\": { \"gjcxs_zhugeliang3\": 1.9, \"gjcxs_caojie1\": 0.7 },\n");
    out.push_str("  \"qz\": { \"shen_zhugeliang\": 1.75 },\n");
    out.push_str("  \"applyCf\": \"off\",\n");
    out.push_str("  \"applyQz\": false,\n");
    out.push_str("  \"ckQz\": \"off\",\n");
    out.push_str("  \"qzCf\": false,\n");
    out.push_str("  \"viewAtt\": false,\n");
    out.push_str("  \"nhFriends\": false,\n");
    out.push_str("  \"ntAoe\": false,\n");
    out.push_str("  \"banQh\": false,\n");
    out.push_str("  \"_note\": \"身份/模式专项策略（dizhu/nongmin/nei/fan/findZhong/mjAi/sfjAi/btAi）与设置菜单交互未迁移（登记差异）\"\n");
    out.push_str("}\n");
    out
}

fn build_package(config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "ai-tuning", "1.0.0");
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
            BTreeMap::from([("ai-tuning".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("ai-tuning");
    assets.add_entry(AssetEntry {
        asset_id: "ai-tuning:config".into(),
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
        .insert("ai-tuning:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("ai-tuning-upstream-liuli");
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
