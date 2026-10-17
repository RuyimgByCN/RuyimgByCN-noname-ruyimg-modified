//! U02 / L03 boss「play 包」转换包生成器（2026-10-17）。
//!
//! 把两仓的 `boss` play 扩展转换为 `discover_extensions` 可发现的真实包
//! （manifest.json + package.npack.json + assets/config.json），写入
//! `packages/extensions/boss-play-upstream-{noname,liuli}`。
//!
//! 运行：`cargo run --offline -p content-runtime --example gen_boss_packages`

use blake3::hash;
use content_schema::{ContentPackage, PackageManifest};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};
use rule_api::CapabilitySet;
use std::collections::BTreeMap;
use std::path::PathBuf;

const NONAME_COMMIT: &str = "df2682e66af9930fe02861a25cb6f2cd2ef6ae36";
const LIULI_COMMIT: &str = "8485dbb29ff16f9b258a766b1a17ef75f883aade";

const NONAME_INTRO: &[(&str, &str)] = &[
    ("boss_liedixuande", "liubei"),
    ("boss_gongshenyueying", "huangyueying"),
    ("boss_tianhoukongming", "shen_zhugeliang"),
    ("boss_yuhuoshiyuan", "pangtong"),
    ("boss_qiaokuijunyi", "zhanghe"),
    ("boss_jiarenzidan", "caozhen"),
    ("boss_duanyuzhongda", "simayi"),
    ("boss_juechenmiaocai", "xiahouyuan"),
    ("boss_kumuyuanrang", "xiahoudun"),
    ("boss_baijiwenyuan", "zhangliao"),
    ("boss_yihanyunchang", "guanyu"),
    ("boss_fuweizilong", "zhaoyun"),
    ("boss_lvbu1", "lvbu"),
    ("boss_lvbu2", "lvbu"),
    ("boss_lvbu3", "lvbu"),
];

const LIULI_INTRO: &[(&str, &str)] = &[
    ("boss_liedixuande", "liubei"),
    ("boss_gongshenyueying", "huangyueying"),
    ("boss_tianhoukongming", "shen_zhugeliang"),
    ("boss_yuhuoshiyuan", "pangtong"),
    ("boss_qiaokuijunyi", "zhanghe"),
    ("boss_jiarenzidan", "caozhen"),
    ("boss_duanyuzhongda", "simayi"),
    ("boss_juechenmiaocai", "xiahouyuan"),
];

fn aliases_json(aliases: &[(&str, &str)]) -> String {
    let parts: Vec<String> = aliases
        .iter()
        .map(|(a, b)| format!("[\"{a}\", \"{b}\"]"))
        .collect();
    parts.join(", ")
}

fn config_json(
    source: &str,
    upstream_path: &str,
    commit: &str,
    boss_key: &str,
    jiange_key: &str,
    boss_translate: &str,
    jiange_translate: &str,
    tag_boss: Option<&str>,
    tag_versus: Option<&str>,
    aliases: &[(&str, &str)],
) -> String {
    let opt = |v: Option<&str>| match v {
        Some(s) => format!("\"{s}\""),
        None => "null".to_string(),
    };
    format!(
        "{{\n  \"source\": \"{source}\",\n  \"upstream_path\": \"{upstream_path}\",\n  \"upstream_commit\": \"{commit}\",\n  \"boss_pack_key\": \"{boss_key}\",\n  \"jiange_pack_key\": \"{jiange_key}\",\n  \"boss_translate_key\": \"{boss_translate}\",\n  \"jiange_translate_key\": \"{jiange_translate}\",\n  \"boss_translate_default\": \"挑战武将\",\n  \"jiange_translate_default\": \"剑阁武将\",\n  \"tag_boss\": {},\n  \"tag_versus\": {},\n  \"intro_aliases\": [{}]\n}}\n",
        opt(tag_boss),
        opt(tag_versus),
        aliases_json(aliases),
    )
}

fn build_package(namespace: &str, package_id: &str, config: &str) -> ContentPackage {
    let mut manifest = PackageManifest::new(namespace, package_id, "1.0.0");
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
            BTreeMap::from([(package_id.to_string(), "挑战包".to_string())]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let mut assets = AssetManifest::new(package_id);
    assets.add_entry(AssetEntry {
        asset_id: format!("{package_id}:config"),
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
        .insert(format!("{package_id}:config"), "assets/config.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn write_package(dir: PathBuf, namespace: &str, package_id: &str, config: String) {
    std::fs::create_dir_all(dir.join("assets")).unwrap();
    std::fs::write(
        dir.join("manifest.json"),
        "{\n  \"schema_version\": 1,\n  \"enabled\": true,\n  \"package\": \"package.npack.json\"\n}\n",
    )
    .unwrap();
    std::fs::write(dir.join("assets/config.json"), &config).unwrap();
    let package = build_package(namespace, package_id, &config);
    std::fs::write(
        dir.join("package.npack.json"),
        package.to_json_bytes().unwrap(),
    )
    .unwrap();
    println!("written {}", dir.display());
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");

    let noname = config_json(
        "noname",
        "apps/core/extension/boss/extension.js",
        NONAME_COMMIT,
        "boss",
        "jiange",
        "boss_character_config",
        "jiange_character_config",
        None,
        None,
        NONAME_INTRO,
    );
    write_package(
        root.join("boss-play-upstream-noname"),
        "upstream-noname",
        "boss-play",
        noname,
    );

    let liuli = config_json(
        "noname-liuli",
        "noname/extension/boss/extension.js",
        LIULI_COMMIT,
        "mode_extension_boss",
        "mode_extension_jiange",
        "mode_extension_boss_character_config",
        "mode_extension_jiange_character_config",
        Some("mode:boss"),
        Some("mode:versus"),
        LIULI_INTRO,
    );
    write_package(
        root.join("boss-play-upstream-liuli"),
        "upstream-liuli",
        "boss-play",
        liuli,
    );
}
