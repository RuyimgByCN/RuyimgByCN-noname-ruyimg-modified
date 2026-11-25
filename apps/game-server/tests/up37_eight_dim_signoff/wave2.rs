//! Wave 2 (`L06, L01, L02, L03, L04, L05, L07`) 仓内八维补全验收测试。

use content_native_loader::{
    apply_wuxing_config, bridge_generals, bridge_pack, bridge_pile, parse_ai_tuning_config,
    parse_boss_play_config, parse_coin_config, parse_config, parse_pile_factors, parse_sj_config,
    parse_wuxing_config,
};

use super::harness::{repo_extensions_root, verify_combo_match, verify_source_full_lifecycle_and_deps};

#[test]
fn up37_wave2_seven_sources_full_lifecycle_deps_config_combo_and_restore() {
    let root = repo_extensions_root();

    // 1. 逐来源全生命周期 + 依赖门禁 + 升级回滚 + 物理卸载后旧锁无损恢复（L03 含双正式包）
    verify_source_full_lifecycle_and_deps("L06", &["wuxing-upstream-liuli"], 261201);
    verify_source_full_lifecycle_and_deps("L01", &["ai-tuning-upstream-liuli"], 261202);
    verify_source_full_lifecycle_and_deps("L02", &["sj-settings-upstream-liuli"], 261203);
    verify_source_full_lifecycle_and_deps(
        "L03",
        &["boss-play-upstream-liuli", "boss-zhugeliang-upstream-liuli"],
        261204,
    );
    verify_source_full_lifecycle_and_deps("L04", &["cardpile-upstream-liuli"], 261205);
    verify_source_full_lifecycle_and_deps("L05", &["coin-upstream-liuli"], 261206);
    verify_source_full_lifecycle_and_deps("L07", &["omnisearch-upstream-liuli"], 261207);

    // 2. 配置/资产正反例校验（② 配置）
    let l06_cards =
        std::fs::read_to_string(root.join("wuxing-upstream-liuli/assets/cards.json")).unwrap();
    let mut l06_bridged = bridge_pack(&l06_cards).unwrap();
    assert_eq!(l06_bridged.pack.card_count(), 1);
    let l06_wuxing =
        std::fs::read_to_string(root.join("wuxing-upstream-liuli/assets/wuxing.json")).unwrap();
    assert!(parse_wuxing_config(&l06_wuxing).is_ok());
    apply_wuxing_config(&mut l06_bridged.pack, &l06_wuxing).unwrap();
    assert!(parse_wuxing_config(r#"{"wuxing_num_playpackconfig": 1.5}"#).is_err());

    let l01_cfg =
        std::fs::read_to_string(root.join("ai-tuning-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_ai_tuning_config(&l01_cfg).is_ok());
    assert!(parse_ai_tuning_config(r#"{"source":"liuli","upstream_path":"x"}"#).is_err());

    let l02_cfg =
        std::fs::read_to_string(root.join("sj-settings-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_sj_config(&l02_cfg).is_ok());
    assert!(parse_sj_config(r#"{"openTools": "yes"}"#).is_err());

    let l03_cfg =
        std::fs::read_to_string(root.join("boss-play-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_boss_play_config(&l03_cfg).is_ok());
    assert!(parse_boss_play_config(r#"{"source":""}"#).is_err());
    let l03_zl =
        std::fs::read_to_string(root.join("boss-zhugeliang-upstream-liuli/assets/cards.json"))
            .unwrap();
    assert!(!bridge_generals(&l03_zl).unwrap().generals.is_empty());

    let l04_pile =
        std::fs::read_to_string(root.join("cardpile-upstream-liuli/assets/pile.json")).unwrap();
    assert!(bridge_pile(&l04_pile).unwrap().total >= 1);
    assert!(parse_pile_factors(r#"{"unknown_key": 1.0}"#).is_err());

    let l05_cfg =
        std::fs::read_to_string(root.join("coin-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_coin_config(&l05_cfg).is_ok());
    assert!(parse_coin_config(r#"{"coin":"nan"}"#).is_err());

    let l07_cfg =
        std::fs::read_to_string(root.join("omnisearch-upstream-liuli/assets/config.json")).unwrap();
    assert!(parse_config(&l07_cfg).is_ok());
    assert!(parse_config(r#"{"css":"missing.css","background":"随机"}"#).is_err());

    // 3. 多包同局组合与反序装配守恒（④ 组合）
    verify_combo_match(
        "wave2:L06+L01+L02+L07",
        &[
            "wuxing-upstream-liuli",
            "ai-tuning-upstream-liuli",
            "sj-settings-upstream-liuli",
            "omnisearch-upstream-liuli",
        ],
        261210,
    );
    verify_combo_match(
        "wave2:L03(2pkg)+L04+L05",
        &[
            "boss-play-upstream-liuli",
            "boss-zhugeliang-upstream-liuli",
            "cardpile-upstream-liuli",
            "coin-upstream-liuli",
        ],
        261211,
    );
    verify_combo_match(
        "wave2+wave1:L06+L04+U06+U04",
        &[
            "wuxing-upstream-liuli",
            "cardpile-upstream-liuli",
            "huanle-upstream-noname",
            "coin-upstream-noname",
        ],
        261212,
    );
}
