//! L20 标记补充转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「雷霆万钧」扩展转换为可发现包（manifest.json +
//! package.npack.json + assets/config.json），写入 `packages/extensions/mark-supplement-upstream-liuli`。
//!
//! 本批落地**可移植策略面**：动皮参数重应用退避、低血/受伤/击杀/换组/阶段
//! 触发策略与配置；520 KB 数据表与渲染/互调为登记差异。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_mark-supplement_package`

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
    out.push_str("  \"upstream_path\": \"noname/extension/标记补充/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"pack\": {\n");
    out.push_str("    \"name\": \"mark-supplement-upstream-liuli\",\n");
    out.push_str("    \"version\": 1,\n");
    out.push_str("    \"description\": \"G1 转换：上游标记补充批 1-2（配置门控 + 苦肉/主亡忠继）\",\n");
    out.push_str("    \"card_prefix\": \"ms_\"\n");
    out.push_str("  },\n");
    out.push_str("  \"cards\": [],\n");
    out.push_str("  \"skills\": [\n");
    out.push_str("    { \"name\": \"xigua_mopaifanbei\", \"mapping\": \"host:xigua_mopaifanbei\" },\n");
    out.push_str("    { \"name\": \"xigua_xueliangfanbei\", \"mapping\": \"host:xigua_xueliangfanbei\" },\n");
    out.push_str("    { \"name\": \"kurou\", \"mapping\": \"host:kurou\" },\n");
    out.push_str("    { \"name\": \"zhuwanzhongji\", \"mapping\": \"host:zhuwanzhongji\" }\n");
    out.push_str("  ],\n");
    out.push_str("  \"generals\": [],\n");
    out.push_str("  \"deferred\": [\n");
    out.push_str("    { \"name\": \"mark_display\", \"reason\": \"纯标记展示（yinyang_mark/zhulianbihe_mark/yexinjia_mark 空定义 + wengua 组技）\" },\n");
    out.push_str("    { \"name\": \"spine_effects\", \"reason\": \"Spine 动画/音效技能（_losehp_/_o_o_* 等，展示面不迁移）\" },\n");
    out.push_str("    { \"name\": \"skill_batch_rest\", \"reason\": \"其余技能待批次（G0：75 技，中位 2.5 KB）\" }\n");
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn config_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/标记补充/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"mpfbmoshi\": false,\n");
    out.push_str("  \"xueliangfanbei\": false,\n");
    out.push_str("  \"daojianfu\": false,\n");
    out.push_str("  \"_note\": \"上游以 lib.config 门控模式修正；原生以技能实例启用承载（宿主按配置注入）\"\n");
    out.push_str("}\n");
    out
}

fn build_package(cards: &str, config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "mark-supplement", "1.0.0");
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
                name: "xigua_mopaifanbei".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 1,
                name: "xigua_xueliangfanbei".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 2,
                name: "kurou".into(),
                locked: false,
                lord_only: false,
                non_gainable: false,
                conversion: false,
            },
            content_schema::PackageSkillEntry {
                skill_def_id: 3,
                name: "zhuwanzhongji".into(),
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
            BTreeMap::from([("mark-supplement".to_string(), "雷霆万钧".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("mark-supplement");
    assets.add_entry(AssetEntry {
        asset_id: "mark-supplement:cards".into(),
        source_hash: *hash(cards.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    assets.add_entry(AssetEntry {
        asset_id: "mark-supplement:config".into(),
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
        .insert("mark-supplement:cards".into(), "assets/cards.json".into());
    package
        .asset_paths
        .insert("mark-supplement:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("mark-supplement-upstream-liuli");
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
