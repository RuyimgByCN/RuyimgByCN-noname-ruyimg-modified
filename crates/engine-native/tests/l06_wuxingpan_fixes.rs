//! L06 五行盘概率配置与真实锁定路径核验 (`wuxing_num_playpackconfig`, default `0.3`).
//!
//! 对照上游源码：
//! - `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:23-59`
//!   (`_wuxing` gameStart: 玩家随机五行；牌堆每牌 `Math.random() < lib.config.wuxing_num_playpackconfig || 0`)
//! - `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:186-205`
//!   (`wuxing_num` config entries)

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_native_loader::{
    prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, LockedStartMode,
};
use content_schema::ContentPackage;
use engine_native::catalog::CardKind;

fn wuxing_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions/wuxing-upstream-liuli")
}

fn build_verified_wuxing(config_bytes: Vec<u8>) -> content_native_loader::VerifiedLockedContent {
    let dir = wuxing_dir();
    let npack_bytes = std::fs::read(dir.join("package.npack.json")).expect("read package.npack.json");
    let cards_bytes = std::fs::read(dir.join("assets/cards.json")).expect("read assets/cards.json");

    let mut pkg = ContentPackage::from_json_bytes(&npack_bytes).expect("parse npack");
    if let Some(manifest) = pkg.asset_manifest.as_mut() {
        for entry in manifest.entries.iter_mut() {
            if pkg.asset_paths.get(&entry.asset_id).map(|p| p.as_str()) == Some("assets/wuxing.json")
            {
                entry.source_hash = *blake3::hash(&config_bytes).as_bytes();
            }
        }
    }
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    let pkg_json = String::from_utf8(pkg.to_json_bytes().expect("serialize npack")).expect("utf8");

    let mut assets = BTreeMap::new();
    assets.insert("assets/cards.json".to_string(), cards_bytes);
    assets.insert("assets/wuxing.json".to_string(), config_bytes);
    let frozen = validate_and_freeze_raw_package(pkg_json, assets).expect("freeze modified wuxing");
    prepare_locked_content_with_bridge(vec![frozen]).expect("prepare locked wuxing")
}

#[test]
fn l06_wuxing_asset_defaults_to_0_3_and_respects_custom_rates() {
    let raw_cfg = std::fs::read_to_string(wuxing_dir().join("assets/wuxing.json")).expect("read wuxing.json");
    let cfg_val: serde_json::Value = serde_json::from_str(&raw_cfg).expect("parse wuxing.json");
    let default_rate = cfg_val
        .get("wuxing_num_playpackconfig")
        .and_then(|v| v.as_f64())
        .expect("wuxing.json must declare numeric wuxing_num_playpackconfig defaulting to 0.3");
    assert!(
        (default_rate - 0.3).abs() < 1e-9,
        "wuxing_num_playpackconfig must default to 0.3 per native-rules-baseline.md, got {default_rate}"
    );

    // 1. Default 0.3 on locked factory path: mean ratio across seeds stays near 0.30 and wuxingpan has no nature
    let default_verified = build_verified_wuxing(raw_cfg.as_bytes().to_vec());
    let mut natured_total = 0usize;
    let mut non_pan_total = 0usize;
    for seed in 106_001..106_021 {
        let e = start_locked_engine(seed, 4, LockedStartMode::Prototype, &default_verified)
            .expect("start default wuxing");
        let core = e.state_core();
        for p in &core.players {
            assert!(p.wunature.is_some_and(|n| n <= 4), "player must get random wunature");
        }
        let pan = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.kind == CardKind::WuxingPan)
            .map(|(i, _)| i as u32)
            .expect("wuxingpan in catalog");
        assert!(
            !core.card_natures.contains_key(&pan),
            "wuxingpan itself must never receive a nature (`wuxing/extension.js:45`)"
        );
        non_pan_total += e.catalog().cards.len() - 1;
        natured_total += core.card_natures.len();
    }
    let ratio = natured_total as f64 / non_pan_total as f64;
    assert!(
        (0.26..=0.34).contains(&ratio),
        "default wuxing_num_playpackconfig=0.3 must produce ~30% card natures, got {ratio:.4} ({natured_total}/{non_pan_total})"
    );

    // 2. Rate 0.0 => 0 card natures across deck/hands while players still get wunature (`wuxing/extension.js:42-59`)
    let cfg_zero = serde_json::json!({
        "source": "noname-liuli",
        "upstream_path": "noname/extension/wuxing/extension.js",
        "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
        "wuxing_num_playpackconfig": 0.0,
        "natures": ["metal", "wood", "water", "fire", "soil"],
        "wuxingpan": { "suit": "spade", "rank": 5 }
    })
    .to_string();
    let verified_zero = build_verified_wuxing(cfg_zero.as_bytes().to_vec());
    let e_zero = start_locked_engine(106_100, 4, LockedStartMode::Prototype, &verified_zero)
        .expect("start wuxing 0.0");
    assert!(
        e_zero.state_core().card_natures.is_empty(),
        "wuxing_num_playpackconfig=0.0 must assign 0 card natures, got {}",
        e_zero.state_core().card_natures.len()
    );
    assert!(e_zero.state_core().players.iter().all(|p| p.wunature.is_some()));

    // 3. Rate 1.0 => every non-wuxingpan card in deck/hands receives a nature
    let cfg_one = serde_json::json!({
        "source": "noname-liuli",
        "upstream_path": "noname/extension/wuxing/extension.js",
        "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
        "wuxing_num_playpackconfig": 1.0,
        "natures": ["metal", "wood", "water", "fire", "soil"],
        "wuxingpan": { "suit": "spade", "rank": 5 }
    })
    .to_string();
    let verified_one = build_verified_wuxing(cfg_one.as_bytes().to_vec());
    let e_one = start_locked_engine(106_101, 4, LockedStartMode::Prototype, &verified_one)
        .expect("start wuxing 1.0");
    assert_eq!(
        e_one.state_core().card_natures.len(),
        e_one.catalog().cards.len() - 1,
        "wuxing_num_playpackconfig=1.0 must nature all non-wuxingpan cards"
    );

    // 4. Mid rate 0.1 vs 0.5 diverges and enforces content lock isolation across rates + clean save/restore
    let cfg_mid = serde_json::json!({
        "source": "noname-liuli",
        "upstream_path": "noname/extension/wuxing/extension.js",
        "upstream_commit": "8485dbb29ff16f9b258a766b1a17ef75f883aade",
        "wuxing_num_playpackconfig": "0.1",
        "natures": ["metal", "wood", "water", "fire", "soil"],
        "wuxingpan": { "suit": "spade", "rank": 5 }
    })
    .to_string();
    let verified_mid = build_verified_wuxing(cfg_mid.as_bytes().to_vec());
    let e_mid = start_locked_engine(106_102, 4, LockedStartMode::Prototype, &verified_mid)
        .expect("start wuxing 0.1");
    assert!(
        e_mid.state_core().card_natures.len() < e_one.state_core().card_natures.len() / 2,
        "0.1 config must give noticeably fewer natures than 1.0"
    );

    let snap_mid = e_mid.save();
    let bytes = engine_native::archive::encode_archive(&snap_mid);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("decode wuxing snap");
    let restored = restore_locked_engine(&decoded, &verified_mid).expect("restore same config");
    assert_eq!(restored.state_hash(), e_mid.state_hash());
    assert!(
        restore_locked_engine(&decoded, &default_verified).is_err(),
        "different wuxing_num_playpackconfig rates must produce distinct ruleset locks"
    );

    let npack_bytes =
        std::fs::read(wuxing_dir().join("package.npack.json")).expect("read wuxing package.npack.json");
    let expected_hash = content_runtime::content_hash_hex(&npack_bytes);
    let bundled_ts = std::fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../apps/web-client/src/engine/bundledNativePackages.ts"),
    )
    .expect("read bundledNativePackages.ts");
    assert!(
        bundled_ts.contains(&expected_hash),
        "bundledNativePackages.ts wuxing-upstream-liuli contentHash must equal {expected_hash}"
    );
}
