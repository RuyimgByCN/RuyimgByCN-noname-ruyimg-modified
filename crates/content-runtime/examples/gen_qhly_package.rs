//! L09 千幻聆音转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入 `packages/extensions/qhly-upstream-liuli`。
//!
//! 本批落地**可移植策略面**：换肤决策表、随机皮肤开关与索引面配置；
//! 皮肤/语音数据与详情页插件系统为登记差异。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_qhly_package`

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
    out.push_str("  \"upstream_path\": \"noname/extension/千幻聆音/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"qhly_randskin\": true,\n");
    out.push_str("  \"qhly_randskin_me\": false,\n");
    out.push_str("  \"qhly_autoChangeSkin\": \"close\",\n");
    out.push_str("  \"qhly_banskinlist\": [\"liubei-[original]\"],\n");
    out.push_str("  \"qhly_skinShare\": { \"gz_liubei\": \"liubei\" },\n");
    out.push_str("  \"qhly_skinChange\": {\n");
    out.push_str("    \"sunce\": { \"skin_a\": { \"source\": \"hp_1\", \"phase\": false, \"target\": \"skin_b\" } }\n");
    out.push_str("  },\n");
    out.push_str("  \"_note\": \"皮肤/语音数据（BGM 库/立绘索引）与详情页插件系统不迁移（登记差异）\"\n");
    out.push_str("}\n");
    out
}

fn build_package(config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "qhly", "1.0.0");
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
            BTreeMap::from([("qhly".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("qhly");
    assets.add_entry(AssetEntry {
        asset_id: "qhly:config".into(),
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
        .insert("qhly:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("qhly-upstream-liuli");
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
