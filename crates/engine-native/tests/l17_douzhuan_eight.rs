//! L17 斗转星移八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/douzhuan-xingyi`。禁将方案、
//! 导入导出、描述改写和斗地主排序走本包配置。不打开菜单，不播放音频，不调用
//! `game.reload`，不 `eval`。不安装如真似幻。

use std::path::PathBuf;

use content_native_loader::{
    apply_douzhuan, apply_douzhuan_line, export_plan, format_douzhuan_report,
    parse_douzhuan_config, parse_plan_text, rank_score, rewrite_skill_text, DouzhuanError,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

const DEFAULT_REQUEST: &str = r#"{"mode":"identity","characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":["unseen"]},{"id":"hidden","tags":["forbidai"]},{"id":"notag"}],"skills":[{"id":"draw","info":"伤害不大于2且不小于1，阴阳"}]}"#;
const DEFAULT_LINE: &str = "mode:identity | type:forbidai | gate:0 | plan:- | banned:- | lock:guanyu,hidden | now:guanyu,hidden | tags:liubei=-,guanyu=unseen,hidden=forbidai | names:1=方案一,2=方案二,3=方案三 | plans:- | packs:- | sort:- | pick:- | info:draw=伤害不大于2且不小于1，阴阳 | dyn:- | would:0 | reload:0 | audio:0 | eval:0";

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_data(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(format!(
        "../../../archive/upstream/noname-liuli/noname/extension/斗转星移/data/{name}"
    ));
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("读上游方案 {}", path.display()))
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

fn line_of(config: &str, request: &str) -> String {
    apply_douzhuan_line(config, request).unwrap_or_else(|err| panic!("{err:?}"))
}

/// L17 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l17_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let source_root = repo_extensions_root().join("douzhuan-xingyi-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("斗转星移"));
    assert!(
        npack_text.contains("\"package_id\":\"douzhuan-xingyi\"")
            || npack_text.contains("\"package_id\": \"douzhuan-xingyi\"")
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "douzhuan-xingyi" && item.namespace == "upstream-liuli")
        .expect("斗转星移已安装");
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
        (
            "shousha-full",
            "upstream-liuli",
            "shousha-full-upstream-liuli",
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
        read_installed(&store, "upstream-liuli", "douzhuan-xingyi", "1.0.0");
    assert!(
        installed.manifest.dependencies.is_empty(),
        "依赖：不声明外部扩展"
    );
    assert!(installed.character_index.is_empty());
    assert!(installed.card_index.is_empty());
    assert!(installed.skill_index.is_empty());
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/斗转星移/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(!stored.contains("playAudio"));
    assert!(!stored.contains("game.reload"));
    assert!(!stored.contains("eval("));
    let config = parse_douzhuan_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(
        config.upstream_commit,
        "8485dbb29ff16f9b258a766b1a17ef75f883aade"
    );
    assert_eq!(config.upstream_version, "1.0");
    assert!(config.jin_jiang);
    assert_eq!(config.jin_jiang_type, "forbidai");
    assert_eq!(config.jin_jiang_ru_zhen, "online");
    assert!(config.jj_group_color);
    assert!(config.qnss_background);
    assert!(config.ai_qianghua);
    assert!(!config.qidongye);
    assert!(!config.miaoshu);
    assert!(config.forbid_reload);
    assert_eq!(config.plan_names, vec!["方案一", "方案二", "方案三"]);
    assert!(config.mode_plans.is_empty());
    assert!(config.stars.is_empty());
    assert!(config.plan_packs.is_empty());

    let fresh = line_of(&stored, DEFAULT_REQUEST);
    assert_eq!(fresh, DEFAULT_LINE);
    let parsed = content_native_loader::request_from_json(DEFAULT_REQUEST).unwrap();
    let applied = apply_douzhuan(&config, &parsed).unwrap();
    assert_eq!(format_douzhuan_report(&applied), DEFAULT_LINE);
    assert_eq!(
        applied
            .characters
            .iter()
            .map(|item| item.id.as_str())
            .collect::<Vec<_>>(),
        vec!["liubei", "guanyu", "hidden", "notag"]
    );
    assert!(
        applied.characters[3].tags.is_none(),
        "没有标签数组的武将不写入标记"
    );
    assert!(!applied.audio_played);
    assert!(!applied.eval_called);
    assert!(!applied.reload_called);
    assert!(!applied.group_colored, "势力颜色不画");
    assert!(!applied.menu_opened, "不打开禁将菜单");

    let banned = line_of(
        &stored,
        r#"{"mode":"identity","network":"online","modePlans":{"identity":1},"importText":"['liubei','liubei','caocao']","importIndex":1,"characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":["unseen"]},{"id":"caocao"}],"skills":[]}"#,
    );
    assert!(
        banned.contains("gate:1 | plan:1 | banned:liubei,liubei,caocao |"),
        "{banned}"
    );
    assert!(banned.contains("lock:guanyu | now:guanyu |"), "{banned}");
    assert!(
        banned.contains("tags:liubei=forbidai,guanyu=unseen |"),
        "{banned}"
    );
    assert!(!banned.contains("caocao="), "{banned}");
    let saved = banned
        .split("plans:")
        .nth(1)
        .unwrap()
        .split(" |")
        .next()
        .unwrap()
        .to_string();
    assert_eq!(saved, "1=liubei+liubei+caocao");
    let restored = line_of(
        &stored,
        r#"{"mode":"identity","network":"online","modePlans":{"identity":1},"plans":{"1":["liubei","liubei","caocao"]},"characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":["unseen"]},{"id":"caocao"}],"skills":[]}"#,
    );
    assert_eq!(restored, banned, "把方案交回去得到同一行");

    let offline = line_of(
        &stored,
        r#"{"mode":"identity","network":"offline","modePlans":{"identity":1},"plans":{"1":["liubei"]},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
    );
    assert!(
        offline.contains("gate:0 | plan:- | banned:- |"),
        "{offline}"
    );
    assert!(offline.contains("tags:liubei=- |"), "{offline}");
    let both = stored.replace(
        "\"jinJiangRuZhen\": \"online\"",
        "\"jinJiangRuZhen\": \"online+offline\"",
    );
    let either = line_of(
        &both,
        r#"{"mode":"identity","network":"offline","modePlans":{"identity":1},"plans":{"1":["liubei"]},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
    );
    assert!(
        either.contains("gate:1 | plan:1 | banned:liubei |"),
        "{either}"
    );
    let substring = line_of(
        &stored,
        r#"{"mode":"identity","network":"line","modePlans":{"identity":1},"plans":{"1":["liubei"]},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
    );
    assert!(
        substring.contains("gate:1 |"),
        "indexOf 命中 online 里的 line：{substring}"
    );
    let exact_miss = line_of(
        &stored,
        r#"{"mode":"doudizhu","network":"onlineX","modePlans":{"doudizhu":1},"plans":{"1":["liubei"]},"characters":[{"id":"liubei","tags":[]}],"skills":[],"pick":{"zhu":true,"double":false,"roll":0,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"junk"}]}}"#,
    );
    assert!(exact_miss.contains("gate:0 |"), "{exact_miss}");
    assert!(
        exact_miss.contains("sort:- | pick:- |"),
        "AI 要网络正好是 online：{exact_miss}"
    );

    let none_plan = line_of(
        &stored,
        r#"{"mode":"identity","network":"online","modePlans":{"identity":0},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
    );
    assert!(
        none_plan.contains("gate:1 | plan:0 | banned:- |"),
        "{none_plan}"
    );
    let missing_mode = line_of(
        &stored,
        r#"{"mode":"identity","network":"online","modePlans":{"doudizhu":1},"plans":{"1":["liubei"]},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
    );
    assert!(missing_mode.contains("gate:0 |"), "{missing_mode}");
    let missing_plan = line_of(
        &stored,
        r#"{"mode":"identity","network":"online","modePlans":{"identity":9},"plans":{},"characters":[{"id":"liubei","tags":[]}],"skills":[]}"#,
    );
    assert!(missing_plan.contains("gate:0 |"), "{missing_plan}");
    let story = line_of(
        &stored,
        r#"{"mode":"identity","brawl":true,"story":"taixuhuanjing","network":"online","modePlans":{"taixuhuanjing":1,"brawl":2},"plans":{"1":["liubei"],"2":["guanyu"]},"characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":[]}],"skills":[]}"#,
    );
    assert!(story.contains("mode:taixuhuanjing |"), "{story}");
    assert!(story.contains("banned:liubei |"), "{story}");
    assert!(
        !apply_douzhuan(
            &config,
            &content_native_loader::request_from_json(
                r#"{"mode":"brawl","characters":[],"skills":[]}"#
            )
            .unwrap(),
        )
        .unwrap()
        .menu_opened
    );

    let hidden = stored.replace(
        "\"jinJiangType\": \"forbidai\"",
        "\"jinJiangType\": \"hide\"",
    );
    let unseen = line_of(
        &hidden,
        r#"{"mode":"identity","network":"online","modePlans":{"identity":1},"plans":{"1":["liubei","guanyu"]},"characters":[{"id":"liubei","tags":[]},{"id":"guanyu","tags":["unseen"]},{"id":"kept","tags":["forbidai"]}],"skills":[]}"#,
    );
    assert!(unseen.contains("type:unseen |"), "{unseen}");
    assert!(unseen.contains("lock:guanyu | now:guanyu |"), "{unseen}");
    assert!(
        unseen.contains("tags:liubei=unseen+imforbid,guanyu=unseen+imforbid,kept=forbidai |"),
        "{unseen}"
    );
    assert!(
        !unseen.contains("now:guanyu,liubei"),
        "新禁的不进入 now：{unseen}"
    );

    let renamed = line_of(
        &stored,
        r#"{"mode":"identity","characters":[],"skills":[],"rename":"新禁将方案","renameIndex":1}"#,
    );
    assert!(
        renamed.contains("names:1=新禁将方,2=方案二,3=方案三 |"),
        "{renamed}"
    );
    let blank_name = line_of(
        &stored,
        r#"{"mode":"identity","characters":[],"skills":[],"rename":"","renameIndex":1}"#,
    );
    assert!(
        blank_name.contains("names:1=方案一,2=方案二,3=方案三 |"),
        "{blank_name}"
    );

    let edited = line_of(
        &stored,
        r#"{"mode":"identity","characters":[],"skills":[],"editIndex":1,"plans":{"1":["a","b","c"]},"banIds":["b","d"],"openIds":["a","b"],"keepIds":["a"],"packBase":["standard","extra"],"packViews":[{"id":"standard","count":0},{"id":"sp","count":3}]}"#,
    );
    assert!(edited.contains("plans:1=a+c+d |"), "{edited}");
    assert!(edited.contains("packs:extra,sp |"), "{edited}");

    let described = stored.replace("\"miaoshuXiuGai\": false", "\"miaoshuXiuGai\": true");
    let text = line_of(
        &described,
        r#"{"mode":"identity","characters":[],"skills":[{"id":"draw","info":"伤害不大于2且不小于1，阴阳"}],"dynamic":["不大于不小于","function(){return '阴'}"]}"#,
    );
    assert!(text.contains("info:draw=伤害≤2且≥1，阳阴 |"), "{text}");
    assert!(text.contains("dyn:≤≥;function(){return '阳'} |"), "{text}");
    assert!(text.contains("eval:0"), "{text}");
    assert_eq!(rewrite_skill_text("temp不大于"), "≤≤");
    assert_eq!(rewrite_skill_text("阳阴"), "阴阳");

    let picked = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":true,"double":false,"roll":0.9,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"legend"}]}}"#,
    );
    assert!(picked.contains("sort:a,b | pick:b |"), "{picked}");
    let first = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":true,"double":false,"roll":0.79,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"junk"}]}}"#,
    );
    assert!(first.contains("sort:a,b | pick:a |"), "{first}");
    let gap = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":true,"double":false,"roll":0.9,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"epic"}]}}"#,
    );
    assert!(gap.contains("pick:a |"), "分差 2 超过地主的 1：{gap}");
    let zero = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":false,"double":false,"roll":0.7,"candidates":[{"id":"a","stars":[0,0,0,0,0,0,10]},{"id":"b","stars":[0,0,0,0,0,0,0]}]}}"#,
    );
    assert!(
        zero.contains("sort:a,b | pick:a |"),
        "第二名是 0 分不换：{zero}"
    );
    let farmer = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":false,"double":false,"roll":0.7,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"epic"}]}}"#,
    );
    assert!(
        farmer.contains("sort:a,b | pick:b |"),
        "农民分差 2 可以换：{farmer}"
    );
    let farmer_first = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":false,"double":false,"roll":0.69,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"epic"}]}}"#,
    );
    assert!(farmer_first.contains("pick:a |"), "{farmer_first}");
    let doubled = line_of(
        &stored,
        r#"{"mode":"doudizhu","characters":[],"skills":[],"pick":{"zhu":true,"double":true,"roll":0,"second":0,"candidates":[{"id":"a","rarity":"legend"},{"id":"b","rarity":"rare"},{"id":"c","rarity":"junk"}]}}"#,
    );
    assert!(doubled.contains("sort:a,b,c | pick:a/b |"), "{doubled}");
    let identity_mode = line_of(
        &stored,
        r#"{"mode":"identity","characters":[],"skills":[],"pick":{"zhu":true,"double":false,"roll":0,"candidates":[{"id":"a","rarity":"legend"}]}}"#,
    );
    assert!(
        identity_mode.contains("sort:- | pick:- |"),
        "{identity_mode}"
    );
    let stars = [9_i64, 8, 7, 6, 5, 4, 3];
    assert_eq!(rank_score(Some(&stars), "全能", Some("junk")), Some(9));
    assert_eq!(rank_score(Some(&stars), "主公", None), Some(8));
    assert_eq!(rank_score(Some(&stars), "地主", Some("legend")), Some(4));
    assert_eq!(rank_score(Some(&stars), "农民", None), Some(3));
    assert_eq!(rank_score(Some(&stars), "路人", Some("legend")), None);
    assert_eq!(rank_score(None, "地主", Some("legend")), Some(10));
    assert_eq!(rank_score(None, "地主", Some("0")), None);
    assert_eq!(rank_score(Some(&[]), "全能", Some("legend")), None);

    assert_eq!(export_plan(&[]), "['']");
    assert_eq!(parse_plan_text("['']").unwrap(), vec![""]);
    assert!(parse_plan_text("[]").unwrap().is_empty());
    assert!(matches!(
        parse_plan_text("alert(1)"),
        Err(DouzhuanError::BadPlan(_))
    ));
    assert!(matches!(
        parse_plan_text("['a',1]"),
        Err(DouzhuanError::BadPlan(_))
    ));
    let blank = upstream_data("inputPlan/空白");
    let none = upstream_data("outputPlan/无");
    assert_eq!(parse_plan_text(&blank).unwrap(), vec![""]);
    assert_eq!(parse_plan_text(&none).unwrap(), vec![""]);
    assert_eq!(export_plan(&parse_plan_text(&blank).unwrap()), blank.trim());
    let standard = parse_plan_text(&upstream_data("inputPlan/标准包")).unwrap();
    let volcano = parse_plan_text(&upstream_data("inputPlan/风林火山")).unwrap();
    assert_eq!(standard.first().map(String::as_str), Some("old_caocao"));
    assert_eq!(standard.last().map(String::as_str), Some("mateng"));
    assert!(standard.iter().any(|id| id == "shen_diaochan"));
    assert_eq!(volcano.last().map(String::as_str), Some("zhulu_jiaxu"));
    assert_ne!(standard, volcano);
    assert_eq!(
        export_plan(&standard),
        upstream_data("inputPlan/标准包").trim()
    );
    let imported = line_of(
        &stored,
        &format!(
            r#"{{"mode":"identity","network":"online","modePlans":{{"identity":2}},"importText":"{}","importIndex":2,"characters":[{{"id":"old_caocao","tags":[]}},{{"id":"mateng","tags":[]}}],"skills":[]}}"#,
            upstream_data("inputPlan/空白").trim()
        ),
    );
    assert!(
        imported.contains("banned: |") || imported.contains("banned:-"),
        "{imported}"
    );
    assert!(
        imported.contains("plans:2= |") || imported.contains("plans:-"),
        "{imported}"
    );

    let reloaded = stored.replace("\"forbidReload\": true", "\"forbidReload\": false");
    let reload_line = line_of(
        &reloaded,
        r#"{"mode":"identity","previousTempmode":"doudizhu","characters":[],"skills":[]}"#,
    );
    assert!(
        reload_line.contains("would:1 | reload:0 |"),
        "{reload_line}"
    );
    assert!(matches!(
        apply_douzhuan_line(
            &stored,
            r#"{"mode":"identity","characters":[],"importText":"['a');alert(1);(']","importIndex":1}"#,
        ),
        Err(DouzhuanError::BadPlan(_))
    ));
    assert!(matches!(
        apply_douzhuan_line(
            &stored.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            DEFAULT_REQUEST,
        ),
        Err(DouzhuanError::BadConfig(_))
    ));

    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);
    assert_eq!(
        store.current_lock().expect("lock").lock_hash,
        lock_before,
        "方案不改内容锁"
    );

    match store.activate_version("upstream-liuli", "douzhuan-xingyi", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../douzhuan-xingyi".into();
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
        active_version(&store, "upstream-liuli", "douzhuan-xingyi"),
        "1.0.0"
    );
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    assert_eq!(active_version(&store, "upstream-noname", "coin"), "1.0.0");
    assert_eq!(
        active_version(&store, "upstream-liuli", "elo-sanction"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "sj-settings"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "shousha-full"),
        "1.0.0"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("douzhuan-xingyi-disabled");
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
        assert!(
            !disabled
                .installed
                .iter()
                .any(|item| item.namespace == "upstream-liuli"
                    && item.package_id == "douzhuan-xingyi")
        );
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "douzhuan-xingyi-disabled"));
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
        active_version(&store, "upstream-liuli", "douzhuan-xingyi"),
        "2.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "shousha-full"),
        "1.0.0",
        "升级斗转星移不改手杀全扩"
    );
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    let rolled = store.rollback().expect("回滚");
    assert!(rolled.packages.iter().any(|(slot, version)| slot
        .ends_with("upstream-liuli/douzhuan-xingyi")
        && version == "1.0.0"));
    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "douzhuan-xingyi"),
        "1.0.0"
    );
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "douzhuan-xingyi", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    let kept_config = parse_douzhuan_config(&kept_text).unwrap();
    assert_eq!(kept_config.source, "noname-liuli");
    assert_eq!(line_of(&kept_text, DEFAULT_REQUEST), DEFAULT_LINE);
}
