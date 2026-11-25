//! Wave 4 (`L22, L23, L25, L26, L27, L29`, 6 源) 仓内八维补全验收测试。

use content_native_loader::{
    bridge_pack, parse_desktop_config, parse_elo_config, parse_fool_config, parse_longxue_config,
    parse_select_beautify_config, parse_zuan_config,
};

use super::harness::{repo_extensions_root, verify_combo_match, verify_source_full_lifecycle_and_deps};

#[test]
fn up37_wave4_six_sources_full_lifecycle_deps_config_combo_and_restore() {
    let root = repo_extensions_root();

    // 1. 逐来源全生命周期 + 依赖门禁 + 升级回滚 + 物理卸载后旧锁无损恢复
    verify_source_full_lifecycle_and_deps("L22", &["elo-sanction-upstream-liuli"], 261401);
    verify_source_full_lifecycle_and_deps("L23", &["desktop-adapt-upstream-liuli"], 261402);
    verify_source_full_lifecycle_and_deps("L25", &["zuan-upstream-liuli"], 261403);
    verify_source_full_lifecycle_and_deps("L26", &["fool-plugin-upstream-liuli"], 261404);
    verify_source_full_lifecycle_and_deps("L27", &["select-beautify-upstream-liuli"], 261405);
    verify_source_full_lifecycle_and_deps("L29", &["longxue-upstream-liuli"], 261406);

    // 2. 配置/资产正反例校验（② 配置）
    let l22_cfg =
        std::fs::read_to_string(root.join("elo-sanction-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(parse_elo_config(&l22_cfg).is_ok());
    assert!(parse_elo_config(r#"{"source":""}"#).is_err());

    let l23_cfg =
        std::fs::read_to_string(root.join("desktop-adapt-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(parse_desktop_config(&l23_cfg).is_ok());
    assert!(parse_desktop_config(r#"{"device_profile":"ios"}"#).is_err());

    let l25_cfg =
        std::fs::read_to_string(root.join("zuan-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_zuan_config(&l25_cfg).is_ok());
    assert!(
        parse_zuan_config(
            r#"{"enable_editor":true,"rule_variant":false,"offline_debug_only":false,"taunt_pack":true}"#
        )
        .is_err()
    );

    let l26_cfg =
        std::fs::read_to_string(root.join("fool-plugin-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(parse_fool_config(&l26_cfg).is_ok());
    assert!(parse_fool_config(r#"{"source":""}"#).is_err());

    let l27_cfg =
        std::fs::read_to_string(root.join("select-beautify-upstream-liuli/assets/config.json"))
            .unwrap();
    let l27_cards =
        std::fs::read_to_string(root.join("select-beautify-upstream-liuli/assets/cards.json"))
            .unwrap();
    assert!(parse_select_beautify_config(&l27_cfg).is_ok());
    assert!(bridge_pack(&l27_cards).is_ok());
    assert!(
        parse_select_beautify_config(
            r#"{"source":"x","upstream_path":"y","upstream_commit":"z","jinxuanbiaoyu":"9"}"#
        )
        .is_err()
    );

    let l29_cfg =
        std::fs::read_to_string(root.join("longxue-upstream-liuli/assets/config.json")).unwrap();
    let l29_cards =
        std::fs::read_to_string(root.join("longxue-upstream-liuli/assets/cards.json")).unwrap();
    assert!(parse_longxue_config(&l29_cfg).is_ok());
    assert!(bridge_pack(&l29_cards).is_ok());
    assert!(parse_longxue_config(r#"{"source":""}"#).is_err());

    // 3. 多包同局组合与反序装配守恒（④ 组合）
    verify_combo_match(
        "wave4:L22+L23+L25",
        &[
            "elo-sanction-upstream-liuli",
            "desktop-adapt-upstream-liuli",
            "zuan-upstream-liuli",
        ],
        261410,
    );
    verify_combo_match(
        "wave4:L26+L27+L29",
        &[
            "fool-plugin-upstream-liuli",
            "select-beautify-upstream-liuli",
            "longxue-upstream-liuli",
        ],
        261411,
    );
}
