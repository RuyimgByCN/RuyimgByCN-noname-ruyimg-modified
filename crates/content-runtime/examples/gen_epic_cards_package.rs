//! L10 史诗卡牌转换包生成器（2026-10-17）。
//!
//! 把 noname-liuli 的「史诗卡牌」扩展转换为 `discover_extensions` 可发现的
//! 真实包（manifest.json + package.npack.json + assets/cards.json +
//! assets/config.json），写入仓库 `packages/extensions/epic-cards-upstream-liuli`。
//!
//! 数据来源（只读自上游工作区）：
//! - noname-liuli `noname/extension/史诗卡牌/extension.js`
//!
//! 本批（批 1）落地：技能 `reguhuo`（蛊惑）声明；`rangjie`（让节）与
//! 十常侍面、卡面/技能特效（展示面）登记为 deferred。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_epic_cards_package`

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest, PackageSkillEntry};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use rule_api::CapabilitySet;
use std::collections::BTreeMap;
use std::path::PathBuf;

const UPSTREAM_COMMIT: &str = "8485dbb29ff16f9b258a766b1a17ef75f883aade";

fn cards_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/史诗卡牌/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"pack\": {\n");
    out.push_str("    \"name\": \"epic-cards-upstream-liuli\",\n");
    out.push_str("    \"version\": 1,\n");
    out.push_str("    \"description\": \"G1 转换：上游史诗卡牌首切片（蛊惑）\",\n");
    out.push_str("    \"card_prefix\": \"epic_\"\n");
    out.push_str("  },\n");
    out.push_str("  \"cards\": [],\n");
    out.push_str("  \"skills\": [\n");
    out.push_str("    { \"name\": \"reguhuo\", \"mapping\": \"host:reguhuo\" },\n");
    out.push_str("    { \"name\": \"rangjie\", \"mapping\": \"host:rangjie\" }\n");
    out.push_str("  ],\n");
    out.push_str("  \"generals\": [],\n");
    out.push_str("  \"deferred\": [\n");
    out.push_str("    { \"name\": \"shichangshi\", \"reason\": \"十常侍面\" },\n");
    out.push_str("    { \"name\": \"card_effects\", \"reason\": \"卡面/技能特效（展示面不迁移）\" }\n");
    out.push_str("  ]\n");
    out.push_str("}\n");
    out
}

fn config_json() -> String {
    let mut out = String::from("{\n");
    out.push_str("  \"source\": \"noname-liuli\",\n");
    out.push_str("  \"upstream_path\": \"noname/extension/史诗卡牌/extension.js\",\n");
    out.push_str(&format!("  \"upstream_commit\": \"{UPSTREAM_COMMIT}\",\n"));
    out.push_str("  \"cardEffects\": true,\n");
    out.push_str("  \"skillEffects\": true,\n");
    out.push_str("  \"skillEffectsScale\": 1.0,\n");
    out.push_str("  \"cardEmoji\": false,\n");
    out.push_str("  \"cardFold\": false,\n");
    out.push_str("  \"cardPhantom\": false,\n");
    out.push_str("  \"meihuaskills\": false,\n");
    out.push_str("  \"shichangshi\": false,\n");
    out.push_str("  \"skinEffects\": false,\n");
    out.push_str("  \"_note\": \"展示面配置（原生不迁移特效渲染，仅登记配置面）\"\n");
    out.push_str("}\n");
    out
}

fn build_package(cards: &str, config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new("upstream-liuli", "epic-cards", "1.0.0");
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
        skill_index: vec![PackageSkillEntry {
            charlotte: None,
            persevere: None,
            forever: None,
            temporary: None,
            group_children: None,
            reveal: None,
            onremove: None,

            voice_assets: Default::default(),
            skill_def_id: 0,
            name: "reguhuo".into(),
            locked: false,
            lord_only: false,
            non_gainable: false,
            conversion: false,
        },
        PackageSkillEntry {
            charlotte: None,
            persevere: None,
            forever: None,
            temporary: None,
            group_children: None,
            reveal: None,
            onremove: None,

            voice_assets: Default::default(),
            skill_def_id: 1,
            name: "rangjie".into(),
            locked: false,
            lord_only: false,
            non_gainable: false,
            conversion: false,
        }],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("epic-cards".to_string(), "史诗卡牌".to_string()),
                ("reguhuo".to_string(), "蛊惑".to_string()),
                ("rangjie".to_string(), "让节".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new("epic-cards");
    assets.add_entry(AssetEntry {
        asset_id: "epic-cards:cards".into(),
        source_hash: *hash(cards.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    assets.add_entry(AssetEntry {
        asset_id: "epic-cards:config".into(),
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
        .insert("epic-cards:cards".into(), "assets/cards.json".into());
    package
        .asset_paths
        .insert("epic-cards:config".into(), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = out_root.join("epic-cards-upstream-liuli");
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
