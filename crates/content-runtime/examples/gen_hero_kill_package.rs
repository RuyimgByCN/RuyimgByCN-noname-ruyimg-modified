//! U08 英雄杀转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入 `packages/extensions/hero-kill-upstream-noname`。
//!
//! 本批落地**可移植策略面**：动皮参数重应用退避、低血/受伤/击杀/换组/阶段
//! 触发策略与配置；520 KB 数据表与渲染/互调为登记差异。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_hero-kill_package`

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use rule_api::CapabilitySet;
use std::collections::BTreeMap;
use std::path::PathBuf;

const UPSTREAM_COMMIT: &str = "8485dbb29ff16f9b258a766b1a17ef75f883aade";

fn cards_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname\",\n");
    out.push_str("  \"upstream_path\": \"apps/core/extension/英雄杀/character/skill.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"pack\": {\n");
    out.push_str("    \"name\": \"hero-kill-upstream-noname\",\n");
    out.push_str("    \"version\": 1,\n");
    out.push_str("    \"description\": \"G1 转换：上游英雄杀首批（反击族）\",\n");
    out.push_str("    \"card_prefix\": \"yxs_\"\n");
    out.push_str("  },\n");
    out.push_str("  \"cards\": [],\n");
    out.push_str("  \"skills\": [\n");
    out.push_str("    { \"name\": \"yxs_fanji\", \"mapping\": \"host:yxs_fanji\" },\n");
    out.push_str("    { \"name\": \"yxs_fanji2\", \"mapping\": \"host:yxs_fanji2\" }\n");
    out.push_str("  ],\n");
    out.push_str("  \"generals\": [],\n");
    out.push_str("  \"deferred\": [\n");
    out.push_str("    { \"name\": \"yxs_menshen2\", \"reason\": \"门神主体（70 KB 单技）单列评估\" },\n");
    out.push_str("    { \"name\": \"character_table\", \"reason\": \"43 条武将（character.js）引用大量外部技能（nvquan/qiandu/yxsweiyi 等，本仓他处定义）——依赖未齐\" },\n");
    out.push_str("    { \"name\": \"skill_batch_rest\", \"reason\": \"yxs_menshen/yxs_menshen3 与外部技能待批次\" }\n");
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn config_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname\",\n");
    out.push_str("  \"upstream_path\": \"apps/core/extension/英雄杀/character/skill.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"skills\": [\"yxs_fanji\", \"yxs_fanji2\"],\n");
    out.push_str("  \"_note\": \"武将表依赖外部技能，未随包发布（登记差异）\"\n");
    out.push_str("}\n");
    out
}

fn build_package(cards: &str, config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-noname", "hero-kill", "1.0.0");
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
                name: "yxs_fanji".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 1,
                name: "yxs_fanji2".into(),
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
            BTreeMap::from([("hero-kill".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("hero-kill");
    assets.add_entry(AssetEntry {
        asset_id: "hero-kill:cards".into(),
        source_hash: *hash(cards.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    assets.add_entry(AssetEntry {
        asset_id: "hero-kill:config".into(),
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
        .insert("hero-kill:cards".into(), "assets/cards.json".into());
    package
        .asset_paths
        .insert("hero-kill:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("hero-kill-upstream-noname");
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
