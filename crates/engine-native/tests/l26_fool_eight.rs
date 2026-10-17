//! L26 笨蛋插件八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/fool-plugin`。点数筛选、
//! 杀目标提示、设备判断、配音配对、衍生描述和选将语音走本包配置。
//! 选择卡牌优化的入口是 `if(false)`，不执行。不画界面，不播放音频，
//! 不采样 `Math.random`，不请求网络，不写盘，不 `eval`，不睡眠。

use std::path::PathBuf;

use content_native_loader::{
    apply_fool_line, choose_card_runs, choose_voice, custom_choose_file_runs, derivation_sample,
    fangtong_allows, fangtong_select_len, fix_peiyin, is_phone, landing_offset, minsi_allows,
    parse_fool_config, peiyin_sample, rewrite_derivation, save_liegong, sha_filter_wrapped,
    sha_hint, sha_target_allowed, throw_emotion, ziyuan_allows, AudioSpec, DerivCharacter,
    DerivSkill, FangCard, FoolError, HintQuery, SkillAudio, VoiceClip,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::{ContentPackage, PackageDependency};

const DEFAULT_LINE: &str = "zy:1 | zy_over:0 | ms:1 | ft_hand:1 | ft_fang:0 | ft_next:1 | ft_over:0 | ft_sel:2 | ft_need:3 | hint:强中加伤 | wrap:1 | allowed:1 | phone:1 | peiyin:- | deriv:- | throw:1 | move:0.7 | rot:1 | land:1 | sampled:0 | video:1 | voice:rende | file:skill/rende1.mp3 | play:0 | cardopt:0 | custom:0 | reload:0 | eval:0 | fetch:0 | write:0 | sleep:0 | dom:0 | audio:0";

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_extension() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/笨蛋插件/extension.js");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("读上游 {}", path.display()))
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

fn hint(skills: &[&str]) -> HintQuery {
    HintQuery {
        viewer_is_me: true,
        event_name: "chooseToUse".into(),
        skills: skills.iter().map(|skill| (*skill).to_string()).collect(),
        selected_count: 0,
        player_hand: 3,
        player_hp: 3,
        attack_range: 1,
        target_is_self: false,
        target_hand: 1,
        target_hp: 4,
        in_range: true,
        original_filter: true,
        distance_target_to_player: 2,
        distance_player_to_target: 2,
        color: "red".into(),
        suit: "spade".into(),
        spshidi_mark: 1,
        phase_has_sha: false,
        phase_sha_count: 3,
        use_count: 0,
        damaged_hp: 0,
        empty_slots_2_to_6: false,
        mark_basic: false,
        mark_damage: false,
    }
}

fn skill(
    id: &str,
    translate: Option<&str>,
    info: Option<&str>,
    audio: AudioSpec,
    group: Option<Vec<String>>,
) -> SkillAudio {
    SkillAudio {
        id: id.into(),
        translate: translate.map(str::to_string),
        info: info.map(str::to_string),
        audio,
        group,
        no_choosing_audio: false,
        audioname: Vec::new(),
    }
}

fn peiyin_table() -> Vec<SkillAudio> {
    vec![
        skill("a", Some("杀"), None, AudioSpec::Missing, None),
        skill("b", Some("杀"), Some("描述"), AudioSpec::Number(2), None),
        skill("c", Some("杀"), Some("描述"), AudioSpec::Number(1), None),
        skill("_pub", Some("杀"), Some("描述"), AudioSpec::Number(3), None),
        skill(
            "d",
            Some("其他"),
            Some("描述"),
            AudioSpec::Number(1),
            Some(vec!["d2".into(), "nope".into()]),
        ),
        skill("d2", Some("二"), None, AudioSpec::Missing, None),
        skill("nope", Some("三"), None, AudioSpec::Missing, None),
        skill(
            "zero",
            Some("零"),
            Some("描述"),
            AudioSpec::Number(0),
            Some(vec!["zero2".into()]),
        ),
        skill("zero2", Some("零二"), None, AudioSpec::Missing, None),
    ]
}

/// L26 八项。目标平台的字符串辅助在 `engine-native-wasm` 里覆盖。
#[test]
fn l26_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    let upstream = upstream_extension();
    assert!(upstream.contains("if(false) {"));
    assert!(upstream.contains("return get.number(card)+num<=13;"));
    assert!(upstream.contains("return get.number(button.link)+num<=36;"));
    assert!(upstream.contains("烈弓强中"));
    assert!(upstream.contains("强中加伤"));
    assert!(upstream.contains("if(this==target) return;"));
    assert!(upstream.contains("version:\"2.0.2\""));
    assert!(upstream.contains("if(config.Fool_liegong!='close')"));
    assert!(upstream.contains("_choose.mp3"));
    assert!(!choose_card_runs());
    assert!(!custom_choose_file_runs());

    assert!(ziyuan_allows(&[10], 3));
    assert!(!ziyuan_allows(&[10], 4));
    assert!(minsi_allows(&[], 13));
    assert!(!minsi_allows(&[13], 1));
    let hand = FangCard {
        zone: "h".into(),
        number: 20,
    };
    let equip = FangCard {
        zone: "e".into(),
        number: 10,
    };
    let fang = FangCard {
        zone: "x".into(),
        number: 16,
    };
    assert!(fangtong_allows(&[], &hand));
    assert!(fangtong_allows(&[], &equip));
    assert!(!fangtong_allows(&[], &fang));
    assert!(fangtong_allows(&[hand.clone()], &fang));
    assert!(!fangtong_allows(
        &[hand.clone()],
        &FangCard {
            zone: "x".into(),
            number: 17,
        }
    ));
    assert!(!fangtong_allows(&[hand.clone()], &equip));
    assert_eq!(fangtong_select_len(&[hand.clone(), fang]), 2);
    assert_eq!(fangtong_select_len(&[hand]), 3);

    let mut strong = hint(&["xinliegong"]);
    strong.selected_count = 1;
    assert_eq!(sha_hint("shousha", &strong).as_deref(), Some("强中加伤"));
    assert_eq!(
        sha_hint("olten", &strong).as_deref(),
        Some("可烈弓 不可响应 伤害+1")
    );
    assert_eq!(sha_hint("ol", &strong).as_deref(), Some("可烈弓"));
    assert!(sha_hint("close", &strong).is_none());
    assert!(!sha_filter_wrapped("close"));
    assert!(sha_filter_wrapped("shousha"));
    strong.original_filter = false;
    assert!(!sha_target_allowed(&strong));
    assert_eq!(sha_hint("shousha", &strong).as_deref(), Some("强中加伤"));
    let mut away = hint(&["xinliegong"]);
    away.viewer_is_me = false;
    assert!(sha_hint("shousha", &away).is_none());
    let mut myself = hint(&["xinliegong"]);
    myself.target_is_self = true;
    assert!(sha_hint("shousha", &myself).is_none());

    let mut diy = hint(&["DIY_liegong"]);
    diy.target_hand = 2;
    assert_eq!(sha_hint("shousha", &diy).as_deref(), Some("可烈弓"));
    diy.target_hand = 4;
    assert_eq!(sha_hint("shousha", &diy).as_deref(), Some(""));
    diy.in_range = false;
    diy.target_hand = 1;
    assert_eq!(sha_hint("shousha", &diy).as_deref(), Some(""));

    let mut fuqi = hint(&["fuqi"]);
    fuqi.distance_target_to_player = 1;
    assert_eq!(sha_hint("shousha", &fuqi).as_deref(), Some("不可响应"));
    fuqi.distance_target_to_player = 2;
    assert_eq!(sha_hint("shousha", &fuqi).as_deref(), Some(""));
    let mut refuqi = hint(&["refuqi"]);
    refuqi.distance_player_to_target = 1;
    refuqi.distance_target_to_player = 5;
    assert_eq!(sha_hint("olten", &refuqi).as_deref(), Some("不可响应"));

    let mut shidi = hint(&["spshidi"]);
    shidi.color = "black".into();
    shidi.spshidi_mark = 0;
    assert_eq!(sha_hint("shousha", &shidi).as_deref(), Some("不可响应"));
    shidi.spshidi_mark = 1;
    assert_eq!(sha_hint("shousha", &shidi).as_deref(), Some(""));
    let mut wushen = hint(&["wushen"]);
    wushen.suit = "heart".into();
    wushen.color = "black".into();
    assert_eq!(sha_hint("shousha", &wushen).as_deref(), Some("不可响应"));

    let mut jintao = hint(&["jintao"]);
    jintao.phase_sha_count = 0;
    jintao.in_range = false;
    assert_eq!(sha_hint("shousha", &jintao).as_deref(), Some("加伤"));
    assert_eq!(sha_hint("olten", &jintao).as_deref(), Some(" 伤害+1"));
    jintao.phase_sha_count = 1;
    assert_eq!(sha_hint("shousha", &jintao).as_deref(), Some("不可响应"));
    jintao.phase_sha_count = 2;
    assert_eq!(sha_hint("shousha", &jintao).as_deref(), Some(""));

    assert!(is_phone("Mozilla/5.0 (iPhone; CPU iPhone OS)"));
    assert!(is_phone("Something AndroidTV"));
    assert!(is_phone("OperaMini/1.0"));
    assert!(!is_phone("Mozilla/5.0 (Windows NT 10.0)"));
    assert!(!is_phone("Opera Mini"));

    let (saved, flags) = save_liegong("olten");
    assert_eq!(saved, "olten");
    assert!(!flags.reload_called);

    let mut table = peiyin_table();
    let changed = fix_peiyin(&mut table);
    assert_eq!(changed, vec!["a".to_string(), "d2".into()]);
    assert_eq!(peiyin_sample(), changed);
    assert_eq!(table[0].audio, AudioSpec::Text("c".into()));
    assert_eq!(table[5].audio, AudioSpec::Text("d".into()));
    assert_eq!(table[8].audio, AudioSpec::Missing);
    assert!(matches!(table[3].audio, AudioSpec::Number(3)));

    assert_eq!(derivation_sample(), "获得※资援<b>资援</b>结束");
    let mut protected = vec![
        DerivSkill {
            id: "parent".into(),
            translate: Some("父".into()),
            info: Some("甲>资援<乙".into()),
            content: Some("Skill('ziyuan')".into()),
            inherit: None,
        },
        DerivSkill {
            id: "ziyuan".into(),
            translate: Some("资援".into()),
            info: Some("资援描述".into()),
            content: None,
            inherit: None,
        },
    ];
    assert!(rewrite_derivation(
        &[DerivCharacter {
            skills: vec!["parent".into()],
        }],
        &mut protected
    )
    .is_empty());
    assert_eq!(protected[0].info.as_deref(), Some("甲>资援<乙"));
    let mut same_name = vec![
        DerivSkill {
            id: "parent".into(),
            translate: Some("资援".into()),
            info: Some("资援描述".into()),
            content: Some("Skill('ziyuan')".into()),
            inherit: None,
        },
        DerivSkill {
            id: "ziyuan".into(),
            translate: Some("资援".into()),
            info: Some("资援描述".into()),
            content: None,
            inherit: None,
        },
    ];
    assert!(rewrite_derivation(
        &[DerivCharacter {
            skills: vec!["parent".into()],
        }],
        &mut same_name
    )
    .is_empty());
    let mut inherited = vec![
        DerivSkill {
            id: "parent".into(),
            translate: Some("父".into()),
            info: Some("获得资援".into()),
            content: None,
            inherit: Some("base".into()),
        },
        DerivSkill {
            id: "base".into(),
            translate: Some("基".into()),
            info: None,
            content: Some("Skill('ziyuan')".into()),
            inherit: None,
        },
        DerivSkill {
            id: "ziyuan".into(),
            translate: Some("资援".into()),
            info: None,
            content: None,
            inherit: None,
        },
    ];
    assert!(rewrite_derivation(
        &[DerivCharacter {
            skills: vec!["parent".into()],
        }],
        &mut inherited
    )
    .is_empty());
    assert_eq!(inherited[0].info.as_deref(), Some("获得资援"));
    inherited[2].info = Some("资援描述".into());
    assert_eq!(
        rewrite_derivation(
            &[DerivCharacter {
                skills: vec!["parent".into()],
            }],
            &mut inherited
        ),
        vec!["parent".to_string()]
    );
    assert_eq!(inherited[0].info.as_deref(), Some("获得※资援"));

    let idle = throw_emotion(true, true, false, false, "flower", true, 1);
    assert!(!idle.thrown);
    assert!(!idle.video);
    assert_eq!(idle.dom, 0);
    assert!(!idle.sleep_called);
    let busy = throw_emotion(true, false, true, false, "flower", false, 1);
    assert!(!busy.thrown);
    let flower = throw_emotion(true, false, false, true, "flower", true, 2);
    assert!(flower.thrown);
    assert_eq!(flower.move_time, Some("0.7"));
    assert_eq!(flower.ease, "linear");
    assert!(flower.rotate);
    assert_eq!(flower.audio_name.as_deref(), Some("throw_flower2"));
    assert!(flower.video);
    assert_eq!(flower.delay_ms, 20);
    assert!(!flower.sleep_called);
    assert_eq!(flower.dom, 0);
    let wine = throw_emotion(true, false, false, true, "wine", false, 1);
    assert_eq!(wine.move_time, Some("0.9"));
    assert_eq!(wine.ease, "ease");
    assert!(wine.audio_name.is_none());
    let egg = throw_emotion(true, false, false, false, "egg", false, 1);
    assert!(egg.drop && egg.spin && egg.random_landing && !egg.rotate);
    let shoe = throw_emotion(true, false, false, false, "shoe", false, 1);
    assert!(shoe.shoe_shadow && shoe.dom == 0);
    assert!(landing_offset("sha", 1.0, 1.0).is_none());
    let (left, top) = landing_offset("flower", 1.0, 1.0).unwrap();
    assert!((left - 50.0).abs() < 1e-9);
    assert!((top - 84.0).abs() < 1e-6);
    let off = throw_emotion(false, false, false, false, "flower", true, 1);
    assert!(!off.thrown);

    let voice_skills = vec![skill(
        "rende",
        Some("仁德"),
        Some("描述"),
        AudioSpec::Number(1),
        None,
    )];
    let spoken = choose_voice(
        true,
        true,
        true,
        true,
        &voice_skills,
        &["rende".into()],
        "liubei",
        0,
        0,
    );
    assert_eq!(spoken.skill.as_deref(), Some("rende"));
    assert_eq!(
        spoken.clip,
        Some(VoiceClip::File("skill/rende1.mp3".into()))
    );
    assert!(!spoken.play_called && !spoken.interval && !spoken.fade_called);
    assert!(!spoken.custom_choose_file);
    let skipped = choose_voice(
        true,
        false,
        true,
        true,
        &voice_skills,
        &["rende".into()],
        "liubei",
        0,
        0,
    );
    assert!(skipped.skill.is_none());
    let silent = choose_voice(
        true,
        true,
        false,
        true,
        &voice_skills,
        &["rende".into()],
        "liubei",
        0,
        0,
    );
    assert!(silent.skill.is_none());

    let mut named = voice_skills[0].clone();
    named.audioname = vec!["liubei".into()];
    named.audio = AudioSpec::Number(2);
    let suffixed = choose_voice(
        true,
        true,
        true,
        true,
        &[named],
        &["rende".into()],
        "liubei",
        0,
        1,
    );
    assert_eq!(
        suffixed.clip,
        Some(VoiceClip::File("skill/rende_liubei2.mp3".into()))
    );

    let mut dead_a = skill(
        "a",
        Some("甲"),
        Some("描述"),
        AudioSpec::Text("b".into()),
        None,
    );
    let dead_b = skill(
        "b",
        Some("乙"),
        Some("描述"),
        AudioSpec::Text("a".into()),
        None,
    );
    dead_a.no_choosing_audio = false;
    let dead = choose_voice(
        true,
        true,
        true,
        true,
        &[dead_a, dead_b],
        &["a".into()],
        "liubei",
        0,
        0,
    );
    assert_eq!(dead.clip, Some(VoiceClip::Skill("a".into())));
    assert!(!dead.play_called);

    let mut muted = skill("mute", Some("静"), Some("描述"), AudioSpec::Number(1), None);
    muted.no_choosing_audio = true;
    let live = skill(
        "live",
        Some("活"),
        Some("描述"),
        AudioSpec::Bool(true),
        None,
    );
    let picked = choose_voice(
        true,
        true,
        true,
        true,
        &[muted, live],
        &["mute".into(), "live".into()],
        "liubei",
        0,
        0,
    );
    assert_eq!(picked.skill.as_deref(), Some("live"));
    assert_eq!(picked.clip, Some(VoiceClip::File("skill/live.mp3".into())));

    let source_root = repo_extensions_root().join("fool-plugin-upstream-liuli");
    let npack_bytes = std::fs::read(source_root.join("package.npack.json")).expect("读 npack");
    let config_text =
        std::fs::read_to_string(source_root.join("assets/config.json")).expect("读配置");
    let npack_text = String::from_utf8(npack_bytes.clone()).expect("npack utf-8");
    assert!(npack_text.contains("笨蛋插件"));
    assert!(
        npack_text.contains("\"package_id\":\"fool-plugin\"")
            || npack_text.contains("\"package_id\": \"fool-plugin\"")
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "fool-plugin" && item.namespace == "upstream-liuli")
        .expect("笨蛋插件已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    for (package_id, namespace, folder_name) in [
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
        (
            "douzhuan-xingyi",
            "upstream-liuli",
            "douzhuan-xingyi-upstream-liuli",
        ),
        (
            "update-guard",
            "upstream-liuli",
            "update-guard-upstream-liuli",
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
                .join(folder_name)
                .join("package.npack.json"),
        )
        .unwrap();
        assert_eq!(kept.content_hash, content_runtime::content_hash_hex(&bytes));
    }
    assert!(!report
        .installed
        .iter()
        .any(|item| item.package_id == "如真似幻"));

    let (installed, installed_dir) =
        read_installed(&store, "upstream-liuli", "fool-plugin", "1.0.0");
    assert_eq!(installed.manifest.namespace, "upstream-liuli");
    assert_eq!(installed.manifest.package_id, "fool-plugin");
    assert_eq!(installed.manifest.version, "1.0.0");
    assert!(installed.manifest.dependencies.is_empty());
    assert!(installed.character_index.is_empty());
    assert!(installed.card_index.is_empty());
    assert!(installed.skill_index.is_empty());
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let stored = std::fs::read_to_string(installed_dir.join("assets/config.json")).unwrap();
    assert_eq!(stored.as_bytes(), config_text.as_bytes());
    assert!(stored.contains("\"source\": \"noname-liuli\""));
    assert!(stored.contains("noname/extension/笨蛋插件/extension.js"));
    assert!(stored.contains("8485dbb29ff16f9b258a766b1a17ef75f883aade"));
    assert!(stored.contains("\"upstream_version\": \"2.0.2\""));
    assert!(!stored.contains("eval("));
    assert!(!stored.contains("game.reload"));
    assert!(!stored.contains("fetch("));
    let config = parse_fool_config(&stored).expect("配置");
    assert_eq!(config.source, "noname-liuli");
    assert_eq!(
        config.upstream_commit,
        "8485dbb29ff16f9b258a766b1a17ef75f883aade"
    );
    assert_eq!(config.upstream_version, "2.0.2");
    assert!(config.throw_emotion && config.skill_calc && config.choose_audio);
    assert!(!config.peiyin_fix && !config.derivation);
    assert_eq!(config.liegong, "shousha");
    assert_eq!(
        apply_fool_line(&stored, "{}").expect("默认行"),
        DEFAULT_LINE
    );
    let quiet = apply_fool_line(
        &stored,
        r#"{"selfTarget":true,"ua":"Mozilla/5.0 (Windows NT 10.0)"}"#,
    )
    .unwrap();
    assert!(quiet.contains("throw:0 | move:- |"));
    assert!(quiet.contains("phone:0 |"));
    assert!(quiet.contains("audio:0"));
    let voiced = apply_fool_line(
        &stored.replace("\"Fool_peiyinFix\": false", "\"Fool_peiyinFix\": true"),
        "{}",
    )
    .unwrap();
    assert!(voiced.contains("peiyin:a,d2 |"));
    let derived = apply_fool_line(
        &stored.replace(
            "\"Fool_derivationSkill\": false",
            "\"Fool_derivationSkill\": true",
        ),
        "{}",
    )
    .unwrap();
    assert!(derived.contains("deriv:获得※资援<b>资援</b>结束 |"));
    assert!(!derived.contains("javascript:"));
    assert!(matches!(
        apply_fool_line(
            &stored.replace("\"source\": \"noname-liuli\"", "\"source\": \"\""),
            "{}"
        ),
        Err(FoolError::BadConfig(_))
    ));

    let asset_names: Vec<_> = std::fs::read_dir(installed_dir.join("assets"))
        .unwrap()
        .map(|item| item.unwrap().file_name())
        .collect();
    assert_eq!(asset_names, vec![std::ffi::OsString::from("config.json")]);
    assert_eq!(store.current_lock().expect("lock").lock_hash, lock_before);

    match store.activate_version("upstream-liuli", "fool-plugin", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    let mut escaped = installed.clone();
    escaped.manifest.package_id = "../fool-plugin".into();
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
        active_version(&store, "upstream-liuli", "fool-plugin"),
        "1.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "update-guard"),
        "1.0.0"
    );

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("fool-plugin-disabled");
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
            .any(|item| item.package_id == "fool-plugin"));
        assert!(disabled
            .skipped
            .iter()
            .any(|name| name == "fool-plugin-disabled"));
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
        active_version(&store, "upstream-liuli", "fool-plugin"),
        "2.0.0"
    );
    assert_eq!(
        active_version(&store, "upstream-liuli", "update-guard"),
        "1.0.0"
    );
    assert_eq!(active_version(&store, "upstream-liuli", "coin"), "1.0.0");
    let rolled = store.rollback().expect("回滚");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, version)| slot.ends_with("upstream-liuli/fool-plugin") && version == "1.0.0"));
    drop(store);
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        active_version(&reopened, "upstream-liuli", "fool-plugin"),
        "1.0.0"
    );
    let (_, kept) = read_installed(&reopened, "upstream-liuli", "fool-plugin", "1.0.0");
    let kept_text = std::fs::read_to_string(kept.join("assets/config.json")).unwrap();
    assert_eq!(
        parse_fool_config(&kept_text).unwrap().source,
        "noname-liuli"
    );
    assert_eq!(apply_fool_line(&kept_text, "{}").unwrap(), DEFAULT_LINE);
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );
}
