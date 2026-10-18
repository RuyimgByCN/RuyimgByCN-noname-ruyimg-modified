//! L27 选将美化转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入 `packages/extensions/select-beautify-upstream-liuli`。
//!
//! 本批落地**可移植策略面**：动皮参数重应用退避、低血/受伤/击杀/换组/阶段
//! 触发策略与配置；520 KB 数据表与渲染/互调为登记差异。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_select-beautify_package`

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use rule_api::CapabilitySet;
use std::collections::BTreeMap;
use std::path::PathBuf;

const UPSTREAM_COMMIT: &str = "8485dbb29ff16f9b258a766b1a17ef75f883aade";

fn cards_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/选将美化/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"pack\": {\n");
    out.push_str("    \"name\": \"select-beautify-upstream-liuli\",\n");
    out.push_str("    \"version\": 1,\n");
    out.push_str("    \"description\": \"G1 转换：上游选将美化批 1-4（撷芳/止息/每步/诈降）\",\n");
    out.push_str("    \"card_prefix\": \"sb_\"\n");
    out.push_str("  },\n");
    out.push_str("  \"cards\": [],\n");
    out.push_str("  \"skills\": [\n");
    out.push_str("    { \"name\": \"xiefang\", \"mapping\": \"host:xiefang\" },\n");
    out.push_str("    { \"name\": \"new_zhixi\", \"mapping\": \"host:new_zhixi\" },\n");
    out.push_str("    { \"name\": \"new_zhixi_mark2\", \"mapping\": \"host:new_zhixi\" },\n");
    out.push_str("    { \"name\": \"new_meibu\", \"mapping\": \"host:new_meibu\" },\n");
    out.push_str("    { \"name\": \"new_meibu_range\", \"mapping\": \"host:new_meibu_range\" },\n");
    out.push_str("    { \"name\": \"sbzhaxiang\", \"mapping\": \"host:sbzhaxiang\" }\n");
    out.push_str("  ],\n");
    out.push_str("  \"generals\": [],\n");
    out.push_str("  \"deferred\": [\n");
    out.push_str("    { \"name\": \"select_ui\", \"reason\": \"选将 UI 主面混淆（_0x 标识符 17844 次/唯一 2178 + 字符串表）——仅登记可观测接口\" },\n");
    out.push_str("    { \"name\": \"skill_batch_rest\", \"reason\": \"其余 41 个在用技能待批次实施（G0 见 evidence/2026-10-17-l27-select-beautify-g0）\" }\n");
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn config_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/选将美化/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"skills\": [\"xiefang\", \"new_zhixi\", \"new_zhixi_mark2\", \"new_meibu\", \"new_meibu_range\", \"sbzhaxiang\"],\n");
    out.push_str("  \"_note\": \"选将 UI 主面为混淆代码（不迁移）；技能面按批次移植\"\n");
    out.push_str("}\n");
    out
}

fn build_package(cards: &str, config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "select-beautify", "1.0.0");
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
        skill_index: vec![
            content_schema::PackageSkillEntry {
                skill_def_id: 0,
                name: "xiefang".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 1,
                name: "new_zhixi".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 2,
                name: "new_zhixi_mark2".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 3,
                name: "new_meibu".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 4,
                name: "new_meibu_range".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 5,
                name: "sbzhaxiang".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
        ],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([("select-beautify".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("select-beautify");
    assets.add_entry(AssetEntry {
        asset_id: "select-beautify:cards".into(),
        source_hash: *hash(cards.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    assets.add_entry(AssetEntry {
        asset_id: "select-beautify:config".into(),
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
        .insert("select-beautify:cards".into(), "assets/cards.json".into());
    package
        .asset_paths
        .insert("select-beautify:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("select-beautify-upstream-liuli");
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("manifest.json"),
        "{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}\n",
    )
    .unwrap();
    let cards = cards_json();
    let config = config_json();
    std::fs::write(dir.join("assets/cards.json"), &cards).unwrap();
    std::fs::write(dir.join("assets/config.json"), &config).unwrap();
    let package = build_package(&cards, &config);
    std::fs::write(
        dir.join("package.npack.json"),
        package.to_json_bytes().unwrap(),
    )
    .unwrap();
    println!("written {}", dir.display());
}
