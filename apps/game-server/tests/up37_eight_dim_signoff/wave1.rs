//! Wave 1 (`U06, U07, U08, U02, U03, U04`) 仓内八维补全验收测试。

use content_native_loader::{
    bridge_generals, bridge_pack, bridge_pile, parse_boss_play_config, parse_coin_config,
    parse_pile_factors,
};

use super::harness::{repo_extensions_root, verify_combo_match, verify_source_full_lifecycle_and_deps};

#[test]
fn up37_wave1_six_sources_full_lifecycle_deps_config_combo_and_restore() {
    let root = repo_extensions_root();

    // 1. 逐来源全生命周期 + 依赖门禁 + 升级回滚 + 物理卸载后旧锁无损恢复
    verify_source_full_lifecycle_and_deps("U06", &["huanle-upstream-noname"], 261101);
    verify_source_full_lifecycle_and_deps("U07", &["wudian-upstream-noname"], 261102);
    verify_source_full_lifecycle_and_deps("U08", &["hero-kill-upstream-noname"], 261103);
    verify_source_full_lifecycle_and_deps("U02", &["boss-play-upstream-noname"], 261104);
    verify_source_full_lifecycle_and_deps("U03", &["cardpile-upstream-noname"], 261105);
    verify_source_full_lifecycle_and_deps("U04", &["coin-upstream-noname"], 261106);

    // 2. 配置/资产正反例校验（② 配置）
    let u06_cards = std::fs::read_to_string(root.join("huanle-upstream-noname/assets/cards.json")).unwrap();
    assert_eq!(bridge_pack(&u06_cards).unwrap().pack.card_count(), 7);
    assert!(bridge_pack("{bad json").is_err());

    let u07_cards = std::fs::read_to_string(root.join("wudian-upstream-noname/assets/cards.json")).unwrap();
    assert_eq!(bridge_pack(&u07_cards).unwrap().pack.card_count(), 16);
    assert_eq!(bridge_generals(&u07_cards).unwrap().generals.len(), 9);

    let u08_cfg = std::fs::read_to_string(root.join("hero-kill-upstream-noname/assets/config.json")).unwrap();
    let u08_val: serde_json::Value = serde_json::from_str(&u08_cfg).unwrap();
    assert_eq!(u08_val["skills"].as_array().unwrap().len(), 5);

    let u02_cfg = std::fs::read_to_string(root.join("boss-play-upstream-noname/assets/config.json")).unwrap();
    let u02_parsed = parse_boss_play_config(&u02_cfg).unwrap();
    assert_eq!(u02_parsed.boss_pack_key, "boss");
    assert!(parse_boss_play_config("{bad}").is_err());

    let u03_pile = std::fs::read_to_string(root.join("cardpile-upstream-noname/assets/pile.json")).unwrap();
    assert_eq!(bridge_pile(&u03_pile).unwrap().total, 160);
    assert!(parse_pile_factors(r#"{"sha":-1}"#).is_err());

    let u04_cfg = std::fs::read_to_string(root.join("coin-upstream-noname/assets/config.json")).unwrap();
    assert_eq!(parse_coin_config(&u04_cfg).unwrap().balance, 0);
    assert!(parse_coin_config(r#"{"coin": 1.5}"#).is_err());

    // 3. 真实组合共存校验（④ 组合）
    verify_combo_match(
        "wave1:U06+U07+U08",
        &[
            "huanle-upstream-noname",
            "wudian-upstream-noname",
            "hero-kill-upstream-noname",
        ],
        261110,
    );
    verify_combo_match(
        "wave1:U02+U04+U06",
        &[
            "boss-play-upstream-noname",
            "coin-upstream-noname",
            "huanle-upstream-noname",
        ],
        261111,
    );
    verify_combo_match(
        "wave1:U03+U06",
        &["cardpile-upstream-noname", "huanle-upstream-noname"],
        261112,
    );
}
