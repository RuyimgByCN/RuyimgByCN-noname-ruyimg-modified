//! Wave 5 (`L08, L09, L10, L11, L12, L24` 6 个含 Spine 4.0 来源) 与 Wave 6 (`U01, L28, U05` 3 个冻结边界来源)
//! 仓内八维补全验收测试 + `map.tsv` 全量 37 来源 / 46 正式包完备性守卫。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::{
    bridge_generals, bridge_pack, parse_decade_ui_config, parse_epic_config, parse_qhly_config,
    parse_rzsh_config, parse_shahai_main_config, parse_skin_switch_config, parse_taixu_config,
    parse_thunder_config,
};

use super::harness::{repo_extensions_root, verify_combo_match, verify_source_full_lifecycle_and_deps};

const U05_PKG_DIRS: [&str; 9] = [
    "shaai-ow-upstream-noname",
    "shaai-xianjian-upstream-noname",
    "shahai-gujian-upstream-noname",
    "shahai-gwent-upstream-noname",
    "shahai-hearth-upstream-noname",
    "shahai-mtg-upstream-noname",
    "shahai-swd-upstream-noname",
    "shahai-yunchou-upstream-noname",
    "shahai-zhenfa-upstream-noname",
];

#[test]
fn up37_wave5_six_spine_sources_full_lifecycle_deps_config_combo_and_restore() {
    let root = repo_extensions_root();

    // 1. 逐来源全生命周期 + 依赖门禁 + 升级回滚 + 物理卸载后旧锁无损恢复
    verify_source_full_lifecycle_and_deps("L08", &["decade-ui-upstream-liuli"], 261501);
    verify_source_full_lifecycle_and_deps("L09", &["qhly-upstream-liuli"], 261502);
    verify_source_full_lifecycle_and_deps("L10", &["epic-cards-upstream-liuli"], 261503);
    verify_source_full_lifecycle_and_deps("L11", &["taixu-upstream-liuli"], 261504);
    verify_source_full_lifecycle_and_deps("L12", &["rzsh-upstream-liuli"], 261505);
    verify_source_full_lifecycle_and_deps("L24", &["skin-switch-upstream-liuli"], 261506);

    // 2. 配置正反例校验（② 配置）
    let l08_cfg =
        std::fs::read_to_string(root.join("decade-ui-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_decade_ui_config(&l08_cfg).is_ok());
    assert!(parse_decade_ui_config(r#"{"default_config":{"newDecadeStyle":"invalid"}}"#).is_err());

    let l09_cfg =
        std::fs::read_to_string(root.join("qhly-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_qhly_config(&l09_cfg).is_ok());
    assert!(parse_qhly_config(r#"{"source":"x"}"#).is_err());

    let l10_cfg =
        std::fs::read_to_string(root.join("epic-cards-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_epic_config(&l10_cfg).is_ok());
    assert!(
        parse_epic_config(
            r#"{"source":"x","upstream_path":"y","upstream_commit":"z","skillEffectsScale":99.0}"#
        )
        .is_err()
    );

    let l11_cfg =
        std::fs::read_to_string(root.join("taixu-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_taixu_config(&l11_cfg).is_ok());
    assert!(parse_taixu_config(r#"{"default_config":{"buff_limit":0}}"#).is_err());

    let l12_cfg =
        std::fs::read_to_string(root.join("rzsh-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_rzsh_config(&l12_cfg).is_ok());
    assert!(parse_rzsh_config(r#"{"default_config":{"ui_scale":10}}"#).is_err());

    let l24_cfg =
        std::fs::read_to_string(root.join("skin-switch-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(parse_skin_switch_config(&l24_cfg).is_ok());
    assert!(
        parse_skin_switch_config(
            r#"{"source":"x","upstream_path":"y","upstream_commit":"z","savedPositions":"bad"}"#
        )
        .is_err()
    );

    // 3. 多包同局组合与反序装配守恒（④ 组合）
    verify_combo_match(
        "wave5:L08+L09+L10",
        &[
            "decade-ui-upstream-liuli",
            "qhly-upstream-liuli",
            "epic-cards-upstream-liuli",
        ],
        261510,
    );
    verify_combo_match(
        "wave5:L11+L12+L24",
        &[
            "taixu-upstream-liuli",
            "rzsh-upstream-liuli",
            "skin-switch-upstream-liuli",
        ],
        261511,
    );
}

#[test]
fn up37_wave6_three_frozen_sources_and_37_source_46_pkg_completeness() {
    let root = repo_extensions_root();

    // 1. 逐来源全生命周期 + 依赖门禁 + 升级回滚 + 物理卸载后旧锁无损恢复
    verify_source_full_lifecycle_and_deps("U01", &["d3-upstream-noname"], 261601);
    verify_source_full_lifecycle_and_deps("L28", &["thunder-upstream-liuli"], 261602);
    verify_source_full_lifecycle_and_deps("U05", &U05_PKG_DIRS, 261603);

    // 2. 配置/资产正反例校验（② 配置）
    let u01_chars =
        std::fs::read_to_string(root.join("d3-upstream-noname/assets/character.json")).unwrap();
    assert!(!bridge_generals(&u01_chars).unwrap().generals.is_empty());
    assert!(bridge_generals("{bad").is_err());

    let l28_cfg =
        std::fs::read_to_string(root.join("thunder-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_thunder_config(&l28_cfg).is_ok());
    assert!(
        parse_thunder_config(
            r#"{"source":"x","upstream_path":"y","upstream_commit":"z","gameSpeed":99}"#
        )
        .is_err()
    );

    assert!(parse_shahai_main_config(r#"{"wuxing":false}"#).is_ok());
    assert!(parse_shahai_main_config(r#"{"rand":"0.9"}"#).is_err());
    for dir in U05_PKG_DIRS {
        let cards =
            std::fs::read_to_string(root.join(dir).join("assets/cards.json")).unwrap();
        assert!(bridge_pack(&cards).is_ok(), "U05 bridge_pack failed: {dir}");
    }
    let yunchou_chars =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .unwrap();
    assert!(!bridge_generals(&yunchou_chars).unwrap().generals.is_empty());

    // 3. 多包同局组合与反序装配守恒（④ 组合）
    verify_combo_match(
        "wave6:U01+L28",
        &["d3-upstream-noname", "thunder-upstream-liuli"],
        261610,
    );
    verify_combo_match(
        "wave6:L08+U01",
        &["decade-ui-upstream-liuli", "d3-upstream-noname"],
        261611,
    );
    verify_combo_match("wave6:U05-9pkgs", &U05_PKG_DIRS, 261612);

    // 4. 严格校验 wave1..wave6 已完全覆盖 map.tsv 的全部 37 来源 / 46 正式包
    let map_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../docs/operations/evidence/2026-10-19-package-source-map/map.tsv");
    let raw_map = std::fs::read_to_string(&map_path).expect("read map.tsv");
    let mut expected = BTreeMap::<String, Vec<String>>::new();
    for (idx, line) in raw_map.lines().enumerate() {
        let line = line.trim();
        if idx == 0 || line.is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 4, "map.tsv line {} cols={}", idx + 1, cols.len());
        let mut pkgs: Vec<String> = cols[3].split(',').map(|s| s.to_string()).collect();
        pkgs.sort();
        let declared_count: usize = cols[2].parse().unwrap();
        assert_eq!(pkgs.len(), declared_count, "map.tsv pkg count mismatch for {}", cols[0]);
        assert!(expected.insert(cols[0].to_string(), pkgs).is_none());
    }
    assert_eq!(expected.len(), 37, "map.tsv must declare exactly 37 sources");
    assert_eq!(expected.values().map(Vec::len).sum::<usize>(), 46);

    let covered_list: [(&str, &[&str]); 37] = [
        // Wave 1 (6)
        ("U06", &["huanle-upstream-noname"]),
        ("U07", &["wudian-upstream-noname"]),
        ("U08", &["hero-kill-upstream-noname"]),
        ("U02", &["boss-play-upstream-noname"]),
        ("U03", &["cardpile-upstream-noname"]),
        ("U04", &["coin-upstream-noname"]),
        // Wave 2 (7, L03=2 pkgs)
        ("L06", &["wuxing-upstream-liuli"]),
        ("L01", &["ai-tuning-upstream-liuli"]),
        ("L02", &["sj-settings-upstream-liuli"]),
        ("L03", &["boss-play-upstream-liuli", "boss-zhugeliang-upstream-liuli"]),
        ("L04", &["cardpile-upstream-liuli"]),
        ("L05", &["coin-upstream-liuli"]),
        ("L07", &["omnisearch-upstream-liuli"]),
        // Wave 3 (9)
        ("L13", &["shousha-mvp-upstream-liuli"]),
        ("L14", &["shousha-ui-upstream-liuli"]),
        ("L15", &["shousha-full-upstream-liuli"]),
        ("L16", &["drag-read-upstream-liuli"]),
        ("L17", &["douzhuan-xingyi-upstream-liuli"]),
        ("L18", &["update-guard-upstream-liuli"]),
        ("L19", &["doomsday-brawl-upstream-liuli"]),
        ("L20", &["mark-supplement-upstream-liuli"]),
        ("L21", &["martial-upstream-liuli"]),
        // Wave 4 (6)
        ("L22", &["elo-sanction-upstream-liuli"]),
        ("L23", &["desktop-adapt-upstream-liuli"]),
        ("L25", &["zuan-upstream-liuli"]),
        ("L26", &["fool-plugin-upstream-liuli"]),
        ("L27", &["select-beautify-upstream-liuli"]),
        ("L29", &["longxue-upstream-liuli"]),
        // Wave 5 (6)
        ("L08", &["decade-ui-upstream-liuli"]),
        ("L09", &["qhly-upstream-liuli"]),
        ("L10", &["epic-cards-upstream-liuli"]),
        ("L11", &["taixu-upstream-liuli"]),
        ("L12", &["rzsh-upstream-liuli"]),
        ("L24", &["skin-switch-upstream-liuli"]),
        // Wave 6 (3, U05=9 pkgs)
        ("U01", &["d3-upstream-noname"]),
        ("L28", &["thunder-upstream-liuli"]),
        ("U05", &U05_PKG_DIRS),
    ];
    let mut covered = BTreeMap::<String, Vec<String>>::new();
    for (src, dirs) in covered_list {
        let mut sorted: Vec<String> = dirs.iter().map(|s| (*s).to_string()).collect();
        sorted.sort();
        assert!(covered.insert(src.to_string(), sorted).is_none(), "duplicate {src}");
    }
    assert_eq!(covered, expected, "wave1..wave6 coverage must match map.tsv 37/46");
}
