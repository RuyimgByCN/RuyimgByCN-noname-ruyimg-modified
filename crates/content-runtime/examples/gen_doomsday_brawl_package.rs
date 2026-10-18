//! L19 末日乱斗转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入 `packages/extensions/doomsday-brawl-upstream-liuli`。
//!
//! 本批落地**可移植策略面**：动皮参数重应用退避、低血/受伤/击杀/换组/阶段
//! 触发策略与配置；520 KB 数据表与渲染/互调为登记差异。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_doomsday-brawl_package`

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
    out.push_str("  \"upstream_path\": \"noname/extension/末日乱斗/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"pack\": {\n");
    out.push_str("    \"name\": \"doomsday-brawl-upstream-liuli\",\n");
    out.push_str("    \"version\": 1,\n");
    out.push_str("    \"description\": \"G1 转换：上游末日乱斗批 1-2（誓师/循梦/灾变）\",\n");
    out.push_str("    \"card_prefix\": \"db_\"\n");
    out.push_str("  },\n");
    out.push_str("  \"cards\": [],\n");
    out.push_str("  \"skills\": [\n");
    out.push_str("    { \"name\": \"shishi\", \"mapping\": \"host:shishi\" },\n");
    out.push_str("    { \"name\": \"xunmeng\", \"mapping\": \"host:xunmeng\" },\n");
    out.push_str("    { \"name\": \"zaibian\", \"mapping\": \"host:zaibian\" }\n");
    out.push_str("  ],\n");
    out.push_str("  \"generals\": [],\n");
    out.push_str("  \"deferred\": [\n");
    out.push_str("    { \"name\": \"wujinmoshi\", \"reason\": \"无尽模式配置面（_wujinmoshi* 系列，含 player==game.me 人类位判定）\" },\n");
    out.push_str("    { \"name\": \"brawl_mode\", \"reason\": \"乱斗模式框架（24 处武将引用、chanceIdentity 系列多为注释）\" },\n");
    out.push_str("    { \"name\": \"skill_batch_rest\", \"reason\": \"其余技能待批次实施（G0：66 技，中位 0.5 KB）\" }\n");
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn config_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/末日乱斗/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"skills\": [\"shishi\", \"xunmeng\", \"zaibian\"],\n");
    out.push_str("  \"_note\": \"乱斗模式框架与无尽模式配置面未迁移（登记差异）\"\n");
    out.push_str("}\n");
    out
}

fn build_package(cards: &str, config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "doomsday-brawl", "1.0.0");
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
                name: "shishi".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 1,
                name: "xunmeng".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 2,
                name: "zaibian".into(),
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
            BTreeMap::from([("doomsday-brawl".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("doomsday-brawl");
    assets.add_entry(AssetEntry {
        asset_id: "doomsday-brawl:cards".into(),
        source_hash: *hash(cards.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    assets.add_entry(AssetEntry {
        asset_id: "doomsday-brawl:config".into(),
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
        .insert("doomsday-brawl:cards".into(), "assets/cards.json".into());
    package
        .asset_paths
        .insert("doomsday-brawl:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("doomsday-brawl-upstream-liuli");
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
