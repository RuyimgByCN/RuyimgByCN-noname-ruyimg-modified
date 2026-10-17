//! L15 手杀全扩八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/shousha-full`。改名、将包筛选和
//! 禁将走本包配置里的表。不改稳定 id。不播放奶龙音频，不画界面，不调用
//! `clsPrefixName`。不安装如真似幻。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::{
    apply_shousha, apply_shousha_line, format_report, parse_shousha_config, CharacterIn,
    ShoushaConfig, ShoushaError, ShoushaReport, ShoushaRequest, SkillIn,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

const DEFAULT_LINE: &str = "packs:standard,huicui | liubei:刘备:-,re_liubei:界刘备:-,outsider:路人:forbidai,sunwukong:孙悟空:forbidai,txhj_wolf:狼:forbidai+unseen,xin:赵云:forbidai,ol:OL刘备:forbidai | forbid:- | sunben:界孙策 | skills:draw=摸一张牌。;ended=已经有。;paren=括号）;foo_map_bar=不加;blank=    | audio:0 | loop:0";

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn store_dir(store: &PackageStore, namespace: &str, package_id: &str, version: &str) -> PathBuf {
    store
        .root()
        .join("packages")
        .join(format!("{namespace}__{package_id}"))
        .join(version)
}

fn read_installed(
    store: &PackageStore,
    namespace: &str,
    package_id: &str,
    version: &str,
) -> (ContentPackage, PathBuf) {
    let dir = store_dir(store, namespace, package_id, version);
    let bytes = std::fs::read(dir.join("package.json")).expect("商店副本 package.json");
    let package = ContentPackage::from_json_bytes(&bytes).expect("解析商店副本");
    (package, dir)
}

fn active_version(store: &PackageStore, namespace: &str, package_id: &str) -> String {
    let needle = format!("{namespace}/{package_id}");
    store
        .read_active()
        .expect("active")
        .packages
        .iter()
        .find(|(slot, _)| slot.ends_with(&needle))
        .map(|(_, version)| version.clone())
        .unwrap_or_else(|| panic!("激活集含 {needle}"))
}

fn ch(id: &str, gender: &str, pack: &str, name: &str) -> CharacterIn {
    CharacterIn {
        id: id.into(),
        gender: gender.into(),
        pack: pack.into(),
        name: name.into(),
    }
}

fn sk(id: &str, info: &str) -> SkillIn {
    SkillIn {
        id: id.into(),
        info: info.into(),
    }
}

fn packs_of(names: &[&str]) -> Vec<String> {
    names.iter().map(|name| (*name).to_string()).collect()
}

fn members(rows: &[(&str, &[&str])]) -> BTreeMap<String, Vec<String>> {
    rows.iter()
        .map(|(pack, ids)| {
            (
                (*pack).to_string(),
                ids.iter().map(|id| (*id).to_string()).collect(),
            )
        })
        .collect()
}

fn request(
    mode: &str,
    ban_bt: bool,
    characters: Vec<CharacterIn>,
    packs: &[&str],
    pack_members: BTreeMap<String, Vec<String>>,
    skills: Vec<SkillIn>,
) -> ShoushaRequest {
    ShoushaRequest {
        mode: mode.into(),
        ban_bt,
        characters,
        packs: packs_of(packs),
        pack_members,
        skills,
    }
}

fn apply(config: &ShoushaConfig, request: &ShoushaRequest) -> ShoushaReport {
    let report = apply_shousha(config, request).expect("apply");
    assert_eq!(
        report
            .characters
            .iter()
            .map(|character| character.id.as_str())
            .collect::<Vec<_>>(),
        request
            .characters
            .iter()
            .map(|character| character.id.as_str())
            .collect::<Vec<_>>(),
        "不改稳定 id"
    );
    assert!(!report.audio_played);
    assert!(!report.audio_looped);
    report
}

fn ban_members() -> BTreeMap<String, Vec<String>> {
    members(&[
        ("mobile", &["m1"]),
        ("extra", &["shen_diaochan", "kept_extra", "boss_zhaoyun"]),
        ("refresh", &["ol_xiaoqiao", "kept_refresh"]),
        ("sb", &["sb1"]),
        ("yijiang", &["yj1"]),
        ("shenhua", &["sh1"]),
        ("standard", &["liubei"]),
        ("xinghuoliaoyuan", &["xh1"]),
        ("sp", &["old_maliang", "kept_sp"]),
        ("jsrg", &["js1"]),
        ("shiji", &["sj1"]),
    ])
}

fn named(id: &str, pack: &str) -> CharacterIn {
    ch(id, "male", pack, "甲")
}

/// L15 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l15_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("shousha-full-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("手杀全扩"));
    assert!(
        npack_text.contains("\"package_id\":\"shousha-full\"")
            || npack_text.contains("\"package_id\": \"shousha-full\"")
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "shousha-full" && item.namespace == "upstream-liuli")
        .expect("手杀全扩已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店 content_hash 来自本包 npack"
    );
    for (package_id, namespace, folder) in [
        ("coin", "upstream-liuli", "coin-upstream-liuli"),
        ("coin", "upstream-noname", "coin-upstream-noname"),
        (
            "elo-sanction",
            "upstream-liuli",
            "elo-sanction-upstream-liuli",
        ),
        (
            "sj-settings",
            "upstream-liuli",
            "sj-settings-upstream-liuli",
        ),
    ] {
        let kept = report
            .installed
            .iter()
            .find(|item| item.package_id == package_id && item.namespace == namespace)
            .unwrap_or_else(|| panic!("组合：{namespace}/{package_id} 仍在"));
        assert_eq!(kept.version, "1.0.0");
        let bytes = std::fs::read(
            repo_extensions_root()
                .join(folder)
                .join("package.npack.json"),
        )
        .unwrap();
        assert_eq!(
            kept.content_hash,
            content_runtime::content_hash_hex(&bytes),
            "不改写 {folder}"
        );
    }
    assert!(
        !report
            .installed
            .iter()
            .any(|item| item.package_id == "如真似幻"),
        "不把如真似幻计入"
    );

    let (installed, installed_dir) =
        read_installed(&store, "upstream-liuli", "shousha-full", "1.0.0");
    assert!(
        installed.manifest.dependencies.is_empty(),
        "依赖：不声明外部扩展"
    );
    assert!(
        installed.character_index.is_empty(),
        "武将索引为空，功能在配置表"
    );
    assert!(installed.card_index.is_empty());
    assert!(installed.skill_index.is_empty());
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/手杀全扩/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(!stored.contains("<font"), "HTML 配置名不进包");
    assert!(!stored.contains("helaAudio"));
    assert!(!stored.contains("clsPrefixName"));
    assert!(!stored.contains("changeGroup"), "出厂配置没有 changeGroup");
    let config = parse_shousha_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(
        config.upstream_commit,
        "8485dbb29ff16f9b258a766b1a17ef75f883aade"
    );
    assert!(config.jinjiang);
    assert!(!config.jinjiang2);
    assert!(!config.huanyuan);
    assert!(!config.yinjian);
    assert_eq!(config.unseen, "forbidai");
    assert!(config.packs);
    assert!(config.txhj);
    assert_eq!(config.sex_forbid, "none");
    assert_eq!(config.shousha_name, "shousha");
    assert!(!config.nailong);
    assert!(config.infofix);
    assert!(!config.change_group);
    assert_eq!(config.base_len(), 511);
    assert_eq!(config.doudizhu_len(), 11);
    assert_eq!(config.huanyuan_len(), 167);
    assert_eq!(config.yinjian_len(), 373);
    assert_eq!(config.jinjiang_packs().len(), 12);
    assert_eq!(
        config.jinjiang_packs().first().map(String::as_str),
        Some("standard")
    );
    assert!(config.jinjiang_packs().iter().any(|pack| pack == "huicui"));
    assert!(!config.jinjiang_packs().iter().any(|pack| pack == "tw"));
    assert_eq!(config.mapped_name("base", "liubei"), Some("刘备"));
    assert_eq!(config.mapped_name("base", "guanyu"), Some("关羽"));
    assert_eq!(config.mapped_name("doudizhu", "sunwukong"), Some("孙悟空"));
    assert_eq!(config.mapped_name("huanyuan", "guanyu"), None);
    assert_eq!(config.mapped_name("huanyuan", "liubei"), Some("刘备"));
    assert_eq!(config.mapped_name("yinjian", "re_liubei"), Some("界刘备"));
    assert_eq!(config.mapped_name("yinjian", "guanyu"), None);
    assert!(matches!(
        parse_shousha_config(&stored.replace("\"source\": \"noname-liuli\"", "\"source\": \"\"")),
        Err(ShoushaError::BadConfig(msg)) if msg.contains("source")
    ));
    assert!(matches!(
        parse_shousha_config(&stored.replacen("\"liubei\": \"刘备\"", "\"liubei\": \"刘玄德\"", 1)),
        Err(ShoushaError::BadConfig(msg)) if msg.contains("base")
    ));
    let lock_before = store.current_lock().expect("lock").lock_hash;

    let fixture = request(
        "identity",
        false,
        vec![
            ch("liubei", "male", "standard", "刘备"),
            ch("re_liubei", "male", "standard", "手杀刘备"),
            ch("outsider", "male", "diy", "路人"),
            ch("sunwukong", "male", "diy", "孙悟空"),
            ch("txhj_wolf", "male", "diy", "狼"),
            ch("xin", "male", "standard", "新杀赵云"),
            ch("ol", "male", "standard", "OL刘备"),
        ],
        &["diy", "standard", "standard", "huicui", "tw"],
        BTreeMap::new(),
        vec![
            sk("draw", "摸一张牌"),
            sk("ended", "已经有。"),
            sk("paren", "括号）"),
            sk("foo_map_bar", "不加"),
            sk("blank", "   "),
        ],
    );
    let played = apply(&config, &fixture);
    assert_eq!(format_report(&played), DEFAULT_LINE);
    assert!(played.forbidai.is_empty());
    assert_eq!(played.re_sunben.as_deref(), Some("界孙策"));
    assert!(played
        .characters
        .iter()
        .all(|character| character.group.is_none()));
    let json = r#"{"mode":"identity","banBt":false,"characters":[{"id":"liubei","gender":"male","pack":"standard","name":"刘备"},{"id":"re_liubei","gender":"male","pack":"standard","name":"手杀刘备"},{"id":"outsider","gender":"male","pack":"diy","name":"路人"},{"id":"sunwukong","gender":"male","pack":"diy","name":"孙悟空"},{"id":"txhj_wolf","gender":"male","pack":"diy","name":"狼"},{"id":"xin","gender":"male","pack":"standard","name":"新杀赵云"},{"id":"ol","gender":"male","pack":"standard","name":"OL刘备"}],"packs":["diy","standard","standard","huicui","tw"],"skills":[{"id":"draw","info":"摸一张牌"},{"id":"ended","info":"已经有。"},{"id":"paren","info":"括号）"},{"id":"foo_map_bar","info":"不加"},{"id":"blank","info":"   "}]}"#;
    assert_eq!(apply_shousha_line(&stored, json).unwrap(), DEFAULT_LINE);
    let mut padded = fixture.clone();
    padded.skills.push(sk("pad", " 摸一张牌 "));
    assert_eq!(
        apply(&config, &padded).skills.last().unwrap().info,
        " 摸一张牌 。"
    );
    assert_eq!(
        std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap(),
        stored,
        "公式不改包文件"
    );

    let mut doudizhu = fixture.clone();
    doudizhu.mode = "doudizhu".into();
    let exempt = apply(&config, &doudizhu);
    assert_eq!(
        exempt.characters[3].tags,
        Vec::<String>::new(),
        "斗地主专属不打禁将标记"
    );
    assert_eq!(
        exempt.characters[4].tags,
        vec!["forbidai".to_string(), "unseen".to_string()]
    );

    let mut taixu = fixture.clone();
    taixu.mode = "taixuhuanjing".into();
    assert_eq!(
        apply(&config, &taixu).characters[4].tags,
        vec!["forbidai".to_string()]
    );

    let unseen =
        parse_shousha_config(&stored.replace("\"unseen\": \"forbidai\"", "\"unseen\": \"\""))
            .unwrap();
    let blank_tag = apply(&unseen, &fixture);
    assert_eq!(blank_tag.characters[2].tags, vec!["unseen".to_string()]);
    assert_eq!(
        blank_tag.characters[4].tags,
        vec!["unseen".to_string()],
        "空标记与太虚标记去重"
    );
    let zero =
        parse_shousha_config(&stored.replace("\"unseen\": \"forbidai\"", "\"unseen\": \"0\""))
            .unwrap();
    let zero_tag = apply(&zero, &fixture);
    assert_eq!(zero_tag.characters[2].tags, vec!["0".to_string()]);
    assert_eq!(
        zero_tag.characters[4].tags,
        vec!["0".to_string(), "unseen".to_string()]
    );

    let all_names = parse_shousha_config(
        &stored.replace("\"shoushanameX\": \"shousha\"", "\"shoushanameX\": \"all\""),
    )
    .unwrap();
    let all = apply(&all_names, &fixture);
    assert_eq!(all.characters[6].name, "刘备");
    assert_eq!(all.characters[5].name, "赵云");
    assert_eq!(all.characters[1].name, "界刘备");

    let no_rename = parse_shousha_config(&stored.replace(
        "\"shoushanameX\": \"shousha\"",
        "\"shoushanameX\": \"none\"",
    ))
    .unwrap();
    let mut empty_ok = fixture.clone();
    empty_ok.characters[2].name.clear();
    assert_eq!(apply(&no_rename, &empty_ok).characters[1].name, "手杀刘备");
    assert_eq!(
        apply(&no_rename, &empty_ok).re_sunben.as_deref(),
        Some("界孙策")
    );
    let mut blank_name = fixture.clone();
    blank_name.characters[2].name.clear();
    assert!(
        matches!(apply_shousha(&config, &blank_name), Err(ShoushaError::BadName(id)) if id == "outsider")
    );

    let mut sunben = fixture.clone();
    sunben
        .characters
        .push(ch("re_sunben", "male", "standard", "手杀孙策"));
    let overwritten = apply(&config, &sunben);
    assert_eq!(overwritten.characters.last().unwrap().name, "孙策");
    assert_eq!(
        overwritten.re_sunben.as_deref(),
        Some("孙策"),
        "改名覆盖直接写入的界孙策"
    );

    let huanyuan =
        parse_shousha_config(&stored.replace("\"huanyuan\": false", "\"huanyuan\": true")).unwrap();
    let pooled = apply(
        &huanyuan,
        &request(
            "identity",
            false,
            vec![
                ch("liubei", "male", "standard", "刘备"),
                ch("guanyu", "male", "standard", "关羽"),
            ],
            &["diy", "standard", "huicui", "tw", "sp", "refresh", "shiji"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert_eq!(
        pooled.packs,
        packs_of(&["standard", "sp", "refresh", "shiji"])
    );
    assert_eq!(pooled.characters[0].tags, vec!["forbidai".to_string()]);
    assert!(pooled.characters[1].tags.is_empty(), "关羽不在还原表");
    let only_shiji = apply(
        &huanyuan,
        &request(
            "identity",
            false,
            Vec::new(),
            &["shiji"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert!(only_shiji.packs.is_empty(), "始计篇只在界限突破在列时保留");
    let refresh_only = apply(
        &huanyuan,
        &request(
            "identity",
            false,
            Vec::new(),
            &["refresh"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert_eq!(
        refresh_only.packs,
        packs_of(&["refresh"]),
        "允许表多出来的包不追加"
    );
    let packs_off = parse_shousha_config(
        &stored
            .replace("\"packs\": true", "\"packs\": false")
            .replace("\"huanyuan\": false", "\"huanyuan\": true"),
    )
    .unwrap();
    let tw = apply(
        &packs_off,
        &request(
            "identity",
            false,
            Vec::new(),
            &["sp", "tw"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert_eq!(tw.packs, packs_of(&["sp", "tw"]));
    assert!(apply(
        &packs_off,
        &request(
            "identity",
            false,
            Vec::new(),
            &["tw"],
            BTreeMap::new(),
            Vec::new()
        )
    )
    .packs
    .is_empty());

    let both = parse_shousha_config(
        &stored
            .replace("\"huanyuan\": false", "\"huanyuan\": true")
            .replace("\"yinjian\": false", "\"yinjian\": true"),
    )
    .unwrap();
    let deduped = apply(
        &both,
        &request(
            "identity",
            false,
            vec![
                ch("liubei", "male", "standard", "刘备"),
                ch("re_liubei", "male", "standard", "手杀刘备"),
                ch("guanyu", "male", "standard", "关羽"),
            ],
            &["standard"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert_eq!(deduped.characters[0].tags, vec!["forbidai".to_string()]);
    assert_eq!(deduped.characters[1].tags, vec!["forbidai".to_string()]);
    assert!(deduped.characters[2].tags.is_empty());

    let grouped = parse_shousha_config(
        &stored
            .replace(
                "\"shoushanameX\": \"shousha\"",
                "\"shoushanameX\": \"none\"",
            )
            .replace("\"huanyuan\": false", "\"huanyuan\": true")
            .replace(
                "\"infofix\": true",
                "\"infofix\": true, \"changeGroup\": true",
            ),
    )
    .unwrap();
    assert!(grouped.change_group);
    let groups = apply(
        &grouped,
        &request(
            "identity",
            false,
            vec![
                ch("liubei", "male", "standard", "手杀刘备"),
                ch("guanyu", "male", "standard", "关羽"),
            ],
            &["standard"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert_eq!(groups.characters[0].name, "手杀刘备", "关闭改名时保留原名");
    assert_eq!(groups.characters[0].group.as_deref(), Some("刘备"));
    assert_eq!(groups.characters[1].group, None);

    let sex =
        parse_shousha_config(&stored.replace("\"sexforbid\": \"none\"", "\"sexforbid\": \"male\""))
            .unwrap();
    let gendered = apply(
        &sex,
        &request(
            "identity",
            false,
            vec![
                ch("liubei", "male", "standard", "刘备"),
                ch("daqiao", "female", "standard", "大乔"),
            ],
            &["standard"],
            BTreeMap::new(),
            Vec::new(),
        ),
    );
    assert!(gendered.characters[0].tags.is_empty());
    assert_eq!(gendered.characters[1].tags, vec!["forbidai".to_string()]);
    let ignored =
        parse_shousha_config(&stored.replace("\"sexforbid\": \"none\"", "\"sexforbid\": \"0\""))
            .unwrap();
    assert!(apply(&ignored, &fixture).characters[0].tags.is_empty());

    let banned_cfg =
        parse_shousha_config(&stored.replace("\"jinjiang2\": false", "\"jinjiang2\": true"))
            .unwrap();
    let banned = apply(
        &banned_cfg,
        &request(
            "identity",
            false,
            vec![
                named("shen_diaochan", "extra"),
                named("kept_extra", "extra"),
                named("boss_zhaoyun", "extra"),
                named("ol_xiaoqiao", "refresh"),
                named("kept_refresh", "refresh"),
                named("old_maliang", "sp"),
                named("kept_sp", "sp"),
                named("m1", "mobile"),
                named("sb1", "sb"),
                named("yj1", "yijiang"),
                named("sh1", "shenhua"),
                named("liubei", "standard"),
                named("xh1", "xinghuoliaoyuan"),
                named("js1", "jsrg"),
                named("sj1", "shiji"),
                named("outsider", "diy"),
                named("fanchou", "extra"),
                named("boss_custom", "diy"),
            ],
            &[],
            ban_members(),
            Vec::new(),
        ),
    );
    assert_eq!(
        banned.forbidai,
        packs_of(&["shen_diaochan", "ol_xiaoqiao", "old_maliang", "outsider"])
    );
    let mut empty_mobile = ban_members();
    empty_mobile.insert("mobile".into(), Vec::new());
    let empty_pack = apply(
        &banned_cfg,
        &request(
            "identity",
            false,
            vec![named("m1", "mobile")],
            &[],
            empty_mobile,
            Vec::new(),
        ),
    );
    assert_eq!(
        empty_pack.forbidai,
        vec!["m1".to_string()],
        "空数组算包存在"
    );
    let mut missing_mobile = ban_members();
    missing_mobile.remove("mobile");
    assert!(matches!(
        apply_shousha(&banned_cfg, &request("identity", false, Vec::new(), &[], missing_mobile, Vec::new())),
        Err(ShoushaError::MissingPack(pack)) if pack == "mobile"
    ));
    let bt = apply(
        &banned_cfg,
        &request(
            "identity",
            true,
            vec![
                named("shen_diaochan", "extra"),
                named("kept_extra", "extra"),
                named("kept_refresh", "refresh"),
                named("shen_zhugeliang", "extra"),
                named("fanchou", "extra"),
                named("boss_zhaoyun", "extra"),
                named("boss_custom", "diy"),
            ],
            &[],
            members(&[("extra", &["shen_diaochan", "kept_extra", "boss_zhaoyun"])]),
            Vec::new(),
        ),
    );
    assert_eq!(
        bt.forbidai,
        packs_of(&["shen_diaochan", "kept_refresh", "fanchou"])
    );
    assert!(matches!(
        apply_shousha(&banned_cfg, &request("identity", true, Vec::new(), &[], BTreeMap::new(), Vec::new())),
        Err(ShoushaError::MissingPack(pack)) if pack == "extra"
    ));

    let nailong =
        parse_shousha_config(&stored.replace("\"nailong\": false", "\"nailong\": true")).unwrap();
    assert!(nailong.nailong);
    let quiet = apply(&nailong, &fixture);
    assert!(!quiet.audio_played);
    assert!(!quiet.audio_looped);
    assert_eq!(
        format_report(&quiet),
        DEFAULT_LINE,
        "打开奶龙也不播放、不循环"
    );
    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);
    assert_eq!(
        store.current_lock().expect("lock").lock_hash,
        lock_before,
        "公式不改内容锁"
    );

    match store.activate_version("upstream-liuli", "shousha-full", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../shousha-full".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    match store.install(&escaped.to_json_bytes().unwrap(), None, &options()) {
        Err(RuntimeError::Package(msg)) => assert!(msg.contains("invalid package_id"), "{msg}"),
        other => panic!("包 id 逃逸应拒绝: {other:?}"),
    }
    let mut missing = installed.clone();
    missing.manifest.version = "1.0.1".into();
    missing.manifest.dependencies.push(PackageDependency {
        namespace: "upstream-liuli".into(),
        package_id: "missing-base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    missing.manifest.logic_hash = missing.compute_logic_hash();
    match store.install_with_assets(
        &missing.to_json_bytes().unwrap(),
        None,
        &options(),
        &source_root,
    ) {
        Err(RuntimeError::Invalid(msg)) => assert!(msg.contains("missing dependency"), "{msg}"),
        other => panic!("缺依赖应拒绝: {other:?}"),
    }
    assert_eq!(
        active_version(&store, "upstream-liuli", "shousha-full"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "失败安装不撤掉 liuli 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-noname", "coin"),
        "1.0.0",
        "失败安装不撤掉 noname 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "elo-sanction"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "sj-settings"),
        "1.0.0"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("shousha-full-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(
        source_root.join("manifest.json"),
        disabled_root.join("manifest.json"),
    )
    .unwrap();
    std::fs::copy(
        source_root.join("package.npack.json"),
        disabled_root.join("package.npack.json"),
    )
    .unwrap();
    std::fs::copy(
        source_root.join("assets/config.json"),
        disabled_root.join("assets/config.json"),
    )
    .unwrap();
    let mut manifest: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(disabled_root.join("manifest.json")).unwrap(),
    )
    .unwrap();
    manifest["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(
        disabled_root.join("manifest.json"),
        serde_json::to_string_pretty(&manifest).unwrap(),
    )
    .unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled = import_extensions(&disabled_store, disabled_dir.path(), &options())
            .expect("disabled import");
        assert!(!disabled
            .installed
            .iter()
            .any(|item| item.namespace == "upstream-liuli" && item.package_id == "shousha-full"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "shousha-full-disabled"));
    }

    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &source_root,
        )
        .expect("升级");
    assert_eq!(
        active_version(&store, "upstream-liuli", "shousha-full"),
        "2.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "升级手杀全扩不改 liuli 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-noname", "coin"),
        "1.0.0",
        "升级手杀全扩不改 noname 金币"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "elo-sanction"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "sj-settings"),
        "1.0.0"
    );
    let rolled = store.rollback().expect("回滚");
    assert!(rolled.packages.iter().any(|(slot, version)| slot
        .ends_with("upstream-liuli/shousha-full")
        && version == "1.0.0"));
    assert_eq!(
        active_version(&store, "upstream-liuli", "coin"),
        "1.0.0",
        "回滚后 liuli 金币仍是 1.0.0"
    );

    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "shousha-full"),
        "1.0.0"
    );
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "shousha-full", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    let kept_config = parse_shousha_config(&kept_text).unwrap();
    assert_eq!(kept_config.source, "noname-liuli");
    assert_eq!(kept_config.base_len(), 511);
    assert_eq!(kept_config.mapped_name("base", "liubei"), Some("刘备"));
    assert!(kept_text.contains("\"source\": \"noname-liuli\""));
}
