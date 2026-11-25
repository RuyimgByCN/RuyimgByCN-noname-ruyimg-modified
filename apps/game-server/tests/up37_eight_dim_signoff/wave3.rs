//! Wave 3 (`L13..L21`, 9 源) 仓内八维补全验收测试。

use content_native_loader::{
    load_mark_supplement_config, parse_douzhuan_config, parse_drag_config, parse_martial_config,
    parse_mvp_config, parse_shousha_config, parse_shousha_ui_config, parse_update_guard_config,
    parse_zuan_brawl,
};

use super::harness::{repo_extensions_root, verify_combo_match, verify_source_full_lifecycle_and_deps};

#[test]
fn up37_wave3_nine_sources_full_lifecycle_deps_config_combo_and_restore() {
    let root = repo_extensions_root();

    // 1. 逐来源全生命周期 + 依赖门禁 + 升级回滚 + 物理卸载后旧锁无损恢复
    verify_source_full_lifecycle_and_deps("L13", &["shousha-mvp-upstream-liuli"], 261301);
    verify_source_full_lifecycle_and_deps("L14", &["shousha-ui-upstream-liuli"], 261302);
    verify_source_full_lifecycle_and_deps("L15", &["shousha-full-upstream-liuli"], 261303);
    verify_source_full_lifecycle_and_deps("L16", &["drag-read-upstream-liuli"], 261304);
    verify_source_full_lifecycle_and_deps("L17", &["douzhuan-xingyi-upstream-liuli"], 261305);
    verify_source_full_lifecycle_and_deps("L18", &["update-guard-upstream-liuli"], 261306);
    verify_source_full_lifecycle_and_deps("L19", &["doomsday-brawl-upstream-liuli"], 261307);
    verify_source_full_lifecycle_and_deps("L20", &["mark-supplement-upstream-liuli"], 261308);
    verify_source_full_lifecycle_and_deps("L21", &["martial-upstream-liuli"], 261309);

    // 2. 配置/资产正反例校验（② 配置）
    let l13_cfg =
        std::fs::read_to_string(root.join("shousha-mvp-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_mvp_config(&l13_cfg).is_ok());
    assert!(parse_mvp_config(r#"{"source":""}"#).is_err());

    let l14_cfg =
        std::fs::read_to_string(root.join("shousha-ui-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_shousha_ui_config(&l14_cfg).is_ok());
    assert!(parse_shousha_ui_config(r#"{"default_config":{"player_frame":"bad"}}"#).is_err());

    let l15_cfg =
        std::fs::read_to_string(root.join("shousha-full-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_shousha_config(&l15_cfg).is_ok());
    assert!(parse_shousha_config(r#"{"source":""}"#).is_err());

    let l16_cfg =
        std::fs::read_to_string(root.join("drag-read-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_drag_config(&l16_cfg).is_ok());
    assert!(parse_drag_config(r#"{"source":""}"#).is_err());

    let l17_cfg =
        std::fs::read_to_string(root.join("douzhuan-xingyi-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(parse_douzhuan_config(&l17_cfg).is_ok());
    assert!(parse_douzhuan_config(r#"{"source":""}"#).is_err());

    let l18_cfg =
        std::fs::read_to_string(root.join("update-guard-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(parse_update_guard_config(&l18_cfg).is_ok());
    assert!(parse_update_guard_config(r#"{"source":""}"#).is_err());

    let l19_cfg =
        std::fs::read_to_string(root.join("doomsday-brawl-upstream-liuli/assets/config.json"))
            .unwrap();
    let l19_val: serde_json::Value = serde_json::from_str(&l19_cfg).unwrap();
    assert_eq!(l19_val["skills"].as_array().unwrap().len(), 3);
    assert!(parse_zuan_brawl(Some("all")).is_ok());
    assert!(parse_zuan_brawl(Some("invalid_mode")).is_err());

    let l20_cfg =
        std::fs::read_to_string(root.join("mark-supplement-upstream-liuli/assets/config.json"))
            .unwrap();
    assert!(load_mark_supplement_config(&l20_cfg).is_ok());
    assert!(load_mark_supplement_config(r#"{"youxishengfu":"unknown"}"#).is_err());

    let l21_cfg =
        std::fs::read_to_string(root.join("martial-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_martial_config(&l21_cfg).is_ok());
    assert!(parse_martial_config(r#"{"source":"noname"}"#).is_err());

    // 3. 多包同局组合与反序装配守恒（④ 组合）
    verify_combo_match(
        "wave3:L13+L14+L15+L16+L17",
        &[
            "shousha-mvp-upstream-liuli",
            "shousha-ui-upstream-liuli",
            "shousha-full-upstream-liuli",
            "drag-read-upstream-liuli",
            "douzhuan-xingyi-upstream-liuli",
        ],
        261310,
    );
    verify_combo_match(
        "wave3:L18+L19+L20+L21",
        &[
            "update-guard-upstream-liuli",
            "doomsday-brawl-upstream-liuli",
            "mark-supplement-upstream-liuli",
            "martial-upstream-liuli",
        ],
        261311,
    );
}
