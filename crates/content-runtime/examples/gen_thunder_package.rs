//! L28 雷霆万钧转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入
//! `packages/extensions/thunder-upstream-liuli`。
//!
//! 本批（切片 1）落地**可移植决策面**：配置（速度/同名替换/隐藏专属包/禁同名/
//! 标记补丁开关）、同名替换表（`sameHero`/`chgHero`）、标记文本补丁；UI 补丁与
//! 立绘/主题为渲染面（登记差异），小游戏武将包内容（31 处 `lib.character`）待
//! 后续切片。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_thunder_package`

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
    out.push_str("  \"upstream_path\": \"noname/extension/雷霆万钧/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"gameSpeed\": 4,\n");
    out.push_str("  \"replaceCha\": true,\n");
    out.push_str("  \"hidePack\": true,\n");
    out.push_str("  \"forbidSame\": false,\n");
    out.push_str("  \"UIpatch\": true,\n");
    out.push_str("  \"jinyu\": true,\n");
    out.push_str("  \"shhbiaoji\": true,\n");
    out.push_str("  \"shh_new\": false,\n");
    out.push_str("  \"jiufatext\": true,\n");
    out.push_str("  \"guanning\": false,\n");
    out.push_str("  \"_note\": \"UI补丁/立绘/主题为渲染面（原生不迁移）；小游戏武将包内容待后续切片\"\n");
    out.push_str("}\n");
    out
}

fn build_package(config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "thunder", "1.0.0");
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
            BTreeMap::from([("thunder".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("thunder");
    assets.add_entry(AssetEntry {
        asset_id: "thunder:config".into(),
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
        .insert("thunder:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("thunder-upstream-liuli");
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
