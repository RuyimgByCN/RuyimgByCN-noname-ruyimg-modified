//! cardpile 双包转换生成器（ADR 0776）：把两仓 cardpile 扩展转换为
//! `discover_extensions` 可发现的真实包（manifest.json + package.npack.json
//! + assets/pile.json），写入仓库 `packages/extensions/`。
//!
//! 数据来源（逐字节核对自上游工作区）：
//! - noname      `apps/core/extension/cardpile/extension.js` @ 5c02a745
//! - noname-liuli `noname/extension/cardpile/extension.js`  @ 8485dbb2
//! 两仓 data 表语义一致（仅 JS 语法差异），各保留独立来源身份。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_cardpile_packages`
//!
//! **范围（T20，2026-10-11 收紧）：本示例只生成 cardpile-upstream-noname /
//! cardpile-upstream-liuli 双包（受 `converted_packages_match_generator`
//! 字节守卫约束，重生成安全）。其余转换包（huanle/d3/gujian/gwent/hearth/
//! yunchou/wudian/wuxing）为手工维护，禁止由本示例生成——原全量生成段已
//! 移除：生成器数据落后于手工包数据，整跑曾覆盖丢失 huanle 手工补牌
//! （T15① 实证）。**

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use std::collections::BTreeMap;
use std::path::PathBuf;

/// 上游 cardpile 比例表（两仓一致；牌名 = 上游 data 键）。
const ENTRIES: &[(&str, &[(&str, u32)])] = &[
    (
        "sha",
        &[("diamond", 6), ("club", 14), ("heart", 3), ("spade", 7)],
    ),
    ("huosha", &[("diamond", 2), ("heart", 3)]),
    ("leisha", &[("spade", 5), ("club", 4)]),
    ("shan", &[("heart", 6), ("diamond", 18)]),
    ("jiu", &[("diamond", 1), ("spade", 2), ("club", 2)]),
    ("tao", &[("heart", 9), ("diamond", 3)]),
    ("wanjian", &[("heart", 1)]),
    ("nanman", &[("spade", 2), ("club", 1)]),
    ("guohe", &[("spade", 3), ("club", 2), ("heart", 1)]),
    ("shunshou", &[("spade", 3), ("diamond", 2)]),
    (
        "wuxie",
        &[("heart", 2), ("diamond", 1), ("spade", 2), ("club", 2)],
    ),
    ("tiesuo", &[("spade", 2), ("club", 4)]),
];
const TOTAL: u32 = 160;

fn pile_json(source: &str, upstream_path: &str, upstream_commit: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"source\": \"{source}\",\n"));
    out.push_str(&format!("  \"upstream_path\": \"{upstream_path}\",\n"));
    out.push_str(&format!("  \"upstream_commit\": \"{upstream_commit}\",\n"));
    out.push_str(&format!("  \"total\": {TOTAL},\n"));
    out.push_str("  \"entries\": [\n");
    for (idx, (card, counts)) in ENTRIES.iter().enumerate() {
        let mut line = String::from("    {\"card\": \"");
        line.push_str(card);
        line.push_str("\", \"counts\": {");
        for (j, (suit, n)) in counts.iter().enumerate() {
            if j > 0 {
                line.push_str(", ");
            }
            line.push_str(&format!("\"{suit}\": {n}"));
        }
        line.push_str("}}");
        if idx + 1 < ENTRIES.len() {
            line.push(',');
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn build_package(
    namespace: &str,
    source: &str,
    upstream_path: &str,
    upstream_commit: &str,
) -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new(namespace, "cardpile", "1.0.0"),
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
            BTreeMap::from([
                ("jiu".to_string(), "酒".to_string()),
                ("leisha".to_string(), "雷杀".to_string()),
                ("huosha".to_string(), "火杀".to_string()),
                ("cardpile".to_string(), "牌堆".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let pile = pile_json(source, upstream_path, upstream_commit);
    let mut assets = AssetManifest::new("cardpile");
    assets.add_entry(AssetEntry {
        asset_id: "cardpile:pile".into(),
        source_hash: *hash(pile.as_bytes()).as_bytes(),
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
        .insert("cardpile:pile".into(), "assets/pile.json".into());
    // Keep the published data-only package fingerprint independent of host ABI additions.
    package.manifest.rule_vm_abi_version = 753;
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn main() {
    let out_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");

    let sources: &[(&str, &str, &str, &str)] = &[
        (
            "upstream-noname",
            "noname",
            "apps/core/extension/cardpile/extension.js",
            "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
        ),
        (
            "upstream-liuli",
            "noname-liuli",
            "noname/extension/cardpile/extension.js",
            "8485dbb29ff16f9b258a766b1a17ef75f883aade",
        ),
    ];
    for (namespace, source, path, commit) in sources {
        let dir = out_root.join(format!("cardpile-{namespace}"));
        std::fs::create_dir_all(dir.join("assets")).unwrap();
        let manifest = format!(
            "{{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}}\n"
        );
        std::fs::write(dir.join("manifest.json"), manifest).unwrap();
        let pile = pile_json(source, path, commit);
        std::fs::write(dir.join("assets/pile.json"), &pile).unwrap();
        let package = build_package(namespace, source, path, commit);
        std::fs::write(
            dir.join("package.npack.json"),
            package.to_json_bytes().unwrap(),
        )
        .unwrap();
        println!("written {}", dir.display());
    }
}
