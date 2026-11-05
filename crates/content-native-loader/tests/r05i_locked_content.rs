//! R05-I (`v4 §1-§5` + `v5 §1-§7`) 锁定内容身份链与受控工厂验收测试。
//!
//! 覆盖：
//! 1. 同 `ContentPackage` 两份原始 `package.json` 仅空白/CRLF 不同：`logic_hash` 相等、`content_hash` 不同；
//!    各自构建锁定内容建局后存档，交叉恢复必拒（`SnapshotError::RuleVersionMismatch`）。
//! 2. `v4 §4` 信封政策表全部行：`""`、`"null"`、`"{}"`、null 字段、空对象锁、合法空锁 `<L0>`、
//!    空 `lock_hash`、错误类型、非空 `packages` 空 `frozen_packages`、UTF-8 BOM。
//! 3. 篡改 `namespace`/`package_id`/`version`/`logic_hash`/`content_hash`/`lock_hash`、重复槽位、
//!    缺/多包、非规范排序（`lock.packages` 或 `frozen_packages`）、缺依赖或依赖版本/`logic_hash` 不匹配必拒。
//! 4. 声明资产字节完整性（含 `wuxing-upstream-liuli` 的非卡牌/武将/牌堆资产 `assets/wuxing.json`）：
//!    单字节翻转、缺失声明资产、额外未声明资产、路径越界（`../x`、`/abs`、`a\\b`）、零 `source_hash` 必拒。
//! 5. `v5 §7` 反例：合法包 `huanle-upstream-noname` 武器 `hnl_kuwu_0` `range = 2`，
//!    受控工厂 `start_locked_engine` / `restore_locked_engine` 仅接受 `&VerifiedLockedContent`，
//!    不接受外部裸 `ExtensionPack` 参数；篡改 `FrozenPackage` 内 `assets/cards.json` 射程为 `3` 时
//!    保留原锁必在 `verify_locked_content_with_bridge` 被拒；重新生成的 `r=3` 已验证上下文与原 `r=2` 存档交叉恢复必拒。

use content_native_loader::{
    bridge_pack, prepare_locked_content_with_bridge, restore_locked_engine, start_locked_engine,
    validate_and_freeze_raw_package, verify_locked_content_with_bridge, ContentLock,
    LockedStartMode, VerifiedLockedContent,
};
use content_schema::{ContentPackage, PackageDependency};
use engine_native::SnapshotError;
use std::collections::BTreeMap;

const XIANDING_RAW: &str =
    include_str!("../../../packages/extensions/xianding-upstream-noname/package.npack.json");
const XIANDING_CHARACTERS_RAW: &[u8] =
    include_bytes!("../../../packages/extensions/xianding-upstream-noname/assets/characters.json");
const WUXING_RAW: &str =
    include_str!("../../../packages/extensions/wuxing-upstream-liuli/package.npack.json");
const WUXING_CARDS_RAW: &[u8] =
    include_bytes!("../../../packages/extensions/wuxing-upstream-liuli/assets/cards.json");
const WUXING_CONFIG_RAW: &[u8] =
    include_bytes!("../../../packages/extensions/wuxing-upstream-liuli/assets/wuxing.json");
const HUANLE_RAW: &str =
    include_str!("../../../packages/extensions/huanle-upstream-noname/package.npack.json");
const HUANLE_CARDS_RAW: &[u8] =
    include_bytes!("../../../packages/extensions/huanle-upstream-noname/assets/cards.json");

fn xianding_raw_without_assets() -> String {
    let mut pkg = ContentPackage::from_json_bytes(XIANDING_RAW.as_bytes()).expect("parse xianding");
    let _ = XIANDING_CHARACTERS_RAW;
    pkg.asset_manifest = None;
    pkg.asset_paths.clear();
    pkg.manifest.asset_hash = [0u8; 32];
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    String::from_utf8(pkg.to_json_bytes().expect("to json")).unwrap()
}

fn wuxing_assets() -> BTreeMap<String, Vec<u8>> {
    let mut assets = BTreeMap::new();
    assets.insert("assets/cards.json".to_string(), WUXING_CARDS_RAW.to_vec());
    assets.insert("assets/wuxing.json".to_string(), WUXING_CONFIG_RAW.to_vec());
    assets
}

fn huanle_assets() -> BTreeMap<String, Vec<u8>> {
    let mut assets = BTreeMap::new();
    assets.insert("assets/cards.json".to_string(), HUANLE_CARDS_RAW.to_vec());
    assets
}

#[test]
fn whitespace_crlf_variants_preserve_raw_bytes_and_reject_cross_restore() {
    let base_raw = xianding_raw_without_assets();
    let raw_a = base_raw.clone();
    let raw_b = format!("{base_raw}\r\n  \n");
    assert_ne!(raw_a.as_bytes(), raw_b.as_bytes());

    let frozen_a =
        validate_and_freeze_raw_package(raw_a.clone(), BTreeMap::new()).expect("freeze A");
    let frozen_b =
        validate_and_freeze_raw_package(raw_b.clone(), BTreeMap::new()).expect("freeze B");

    assert_eq!(frozen_a.package_json, raw_a);
    assert_eq!(frozen_b.package_json, raw_b);
    assert_eq!(
        frozen_a.binding.logic_hash, frozen_b.binding.logic_hash,
        "空白/CRLF 变体语义不变，logic_hash 必须相同"
    );
    assert_ne!(
        frozen_a.binding.content_hash, frozen_b.binding.content_hash,
        "空白/CRLF 变体原始字节不同，content_hash 必须不同"
    );

    let verified_a =
        prepare_locked_content_with_bridge(vec![frozen_a]).expect("prepare verified A");
    let verified_b =
        prepare_locked_content_with_bridge(vec![frozen_b]).expect("prepare verified B");
    assert_ne!(verified_a.lock().lock_hash, verified_b.lock().lock_hash);

    let engine_a =
        start_locked_engine(42, 2, LockedStartMode::Prototype, &verified_a).expect("start A");
    let engine_b =
        start_locked_engine(42, 2, LockedStartMode::Prototype, &verified_b).expect("start B");
    let snap_a = engine_a.save();
    let snap_b = engine_b.save();

    // 同上下文恢复通过
    let restored_a = restore_locked_engine(&snap_a, &verified_a).expect("restore A with A");
    assert_eq!(restored_a.state_hash(), engine_a.state_hash());
    let restored_b = restore_locked_engine(&snap_b, &verified_b).expect("restore B with B");
    assert_eq!(restored_b.state_hash(), engine_b.state_hash());

    // 交叉恢复必拒（pack provenance 中包含完整 LockedPackage.content_hash）
    assert!(matches!(
        restore_locked_engine(&snap_a, &verified_b),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ));
    assert!(matches!(
        restore_locked_engine(&snap_b, &verified_a),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ));
}

#[test]
fn envelope_policy_table_v4_section_4_exact_matrix() {
    // 空字符串、空白、"null"、"{}"
    assert!(VerifiedLockedContent::from_envelope_json("").is_err());
    assert!(VerifiedLockedContent::from_envelope_json("   ").is_err());
    assert!(VerifiedLockedContent::from_envelope_json("null").is_err());
    assert!(VerifiedLockedContent::from_envelope_json("{}").is_err());

    // 新字段出现但为 null 或空对象
    assert!(VerifiedLockedContent::from_envelope_json(
        r#"{"canonical_locked_content":null,"frozen_packages":null}"#
    )
    .is_err());
    assert!(VerifiedLockedContent::from_envelope_json(
        r#"{"canonical_locked_content":{},"frozen_packages":[]}"#
    )
    .is_err());

    // 合法标准空锁 <L0>
    let empty_lock = ContentLock::empty();
    assert!(!empty_lock.lock_hash.is_empty());
    let valid_empty_json = format!(
        r#"{{"canonical_locked_content":{{"packages":[],"lock_hash":"{}"}},"frozen_packages":[]}}"#,
        empty_lock.lock_hash
    );
    let verified_empty =
        VerifiedLockedContent::from_envelope_json(&valid_empty_json).expect("valid empty L0");
    assert!(verified_empty.is_empty());
    assert_eq!(verified_empty, VerifiedLockedContent::empty());

    // 空 lock_hash 被拒
    assert!(VerifiedLockedContent::from_envelope_json(
        r#"{"canonical_locked_content":{"packages":[],"lock_hash":""},"frozen_packages":[]}"#
    )
    .is_err());

    // 错误类型（字符串代替对象）
    assert!(VerifiedLockedContent::from_envelope_json(
        r#"{"canonical_locked_content":"bad","frozen_packages":["bad"]}"#
    )
    .is_err());

    // legacy 字段伪装成 envelope（unknown fields 拒绝）
    assert!(VerifiedLockedContent::from_envelope_json(
        r#"{"contentLock":["ns/pkg"],"lockSeed":"old","packsJson":"[]"}"#
    )
    .is_err());

    // packages 非空但 frozen_packages 为空
    let frozen = validate_and_freeze_raw_package(xianding_raw_without_assets(), BTreeMap::new())
        .expect("freeze");
    let verified = prepare_locked_content_with_bridge(vec![frozen]).expect("prepare");
    let missing_copies_json = serde_json::json!({
        "canonical_locked_content": verified.lock(),
        "frozen_packages": []
    })
    .to_string();
    assert!(VerifiedLockedContent::from_envelope_json(&missing_copies_json).is_err());

    // UTF-8 BOM 在原局包 JSON 入口必拒
    let bom_pkg = format!("\u{feff}{}", xianding_raw_without_assets());
    assert!(validate_and_freeze_raw_package(bom_pkg, BTreeMap::new()).is_err());
}

#[test]
fn tamper_ordering_and_dependency_closure_rejected_before_engine_init() {
    let frozen_xianding =
        validate_and_freeze_raw_package(xianding_raw_without_assets(), BTreeMap::new())
            .expect("freeze xianding");
    let frozen_wuxing =
        validate_and_freeze_raw_package(WUXING_RAW.to_string(), wuxing_assets())
            .expect("freeze wuxing");

    let verified_two = prepare_locked_content_with_bridge(vec![
        frozen_xianding.clone(),
        frozen_wuxing.clone(),
    ])
    .expect("prepare two packages");
    assert_eq!(verified_two.copies().len(), 2);
    // 规范顺序：upstream-liuli/wuxing 在 upstream-noname/xianding 之前
    assert_eq!(verified_two.copies()[0].binding.namespace, "upstream-liuli");
    assert_eq!(verified_two.copies()[1].binding.namespace, "upstream-noname");

    // 1. 篡改 binding.namespace / package_id / version / logic_hash / content_hash / lock_hash
    for field in [
        "namespace",
        "package_id",
        "version",
        "logic_hash",
        "content_hash",
        "lock_hash",
    ] {
        let mut lock = verified_two.lock().clone();
        let mut copies = verified_two.copies().to_vec();
        match field {
            "namespace" => {
                lock.packages[0].namespace = "tampered-ns".into();
                copies[0].binding.namespace = "tampered-ns".into();
                lock = ContentLock::from_packages(lock.packages);
            }
            "package_id" => {
                lock.packages[0].package_id = "tampered-id".into();
                copies[0].binding.package_id = "tampered-id".into();
                lock = ContentLock::from_packages(lock.packages);
            }
            "version" => {
                lock.packages[0].version = "9.9.9".into();
                copies[0].binding.version = "9.9.9".into();
                lock = ContentLock::from_packages(lock.packages);
            }
            "logic_hash" => {
                lock.packages[0].logic_hash = "00".repeat(32);
                copies[0].binding.logic_hash = "00".repeat(32);
                lock = ContentLock::from_packages(lock.packages);
            }
            "content_hash" => {
                lock.packages[0].content_hash = "ff".repeat(32);
                copies[0].binding.content_hash = "ff".repeat(32);
                lock = ContentLock::from_packages(lock.packages);
            }
            "lock_hash" => {
                lock.lock_hash = "11".repeat(32);
            }
            _ => unreachable!(),
        }
        assert!(
            verify_locked_content_with_bridge(&lock, &copies).is_err(),
            "篡改 {field} 必须拒绝"
        );
    }

    // 2. 重复槽位
    let dup_lock = ContentLock {
        packages: vec![
            frozen_xianding.binding.clone(),
            frozen_xianding.binding.clone(),
        ],
        lock_hash: "deadbeef".into(),
    };
    assert!(verify_locked_content_with_bridge(
        &dup_lock,
        &[frozen_xianding.clone(), frozen_xianding.clone()]
    )
    .is_err());

    // 3. 锁与 copies 顺序不规范（反转 copies 或反转 lock.packages）
    let mut reversed_copies = verified_two.copies().to_vec();
    reversed_copies.reverse();
    assert!(
        verify_locked_content_with_bridge(verified_two.lock(), &reversed_copies).is_err(),
        "copies 非规范顺序必拒"
    );

    // 4. 依赖缺失或版本/logic_hash 不匹配
    let mut pkg_with_dep =
        ContentPackage::from_json_bytes(xianding_raw_without_assets().as_bytes())
            .expect("parse xianding");
    pkg_with_dep.manifest.dependencies.push(PackageDependency {
        namespace: "upstream-liuli".into(),
        package_id: "wuxing".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    pkg_with_dep.manifest.logic_hash = pkg_with_dep.compute_logic_hash();
    let raw_with_dep = String::from_utf8(pkg_with_dep.to_json_bytes().expect("to json")).unwrap();
    let frozen_with_dep =
        validate_and_freeze_raw_package(raw_with_dep, BTreeMap::new()).expect("freeze with dep");

    // 缺少依赖 wuxing 时 prepare 必拒
    assert!(
        prepare_locked_content_with_bridge(vec![frozen_with_dep.clone()]).is_err(),
        "缺少声明依赖 wuxing 必拒"
    );
    // 补齐依赖 wuxing 后通过
    assert!(
        prepare_locked_content_with_bridge(vec![frozen_with_dep, frozen_wuxing]).is_ok(),
        "依赖闭包齐全应通过"
    );
}

#[test]
fn asset_bytes_fidelity_including_non_card_wuxing_config_asset() {
    // 正常冻结包含 assets/cards.json 与 assets/wuxing.json
    let frozen = validate_and_freeze_raw_package(WUXING_RAW.to_string(), wuxing_assets())
        .expect("freeze wuxing");
    assert_eq!(frozen.assets.len(), 2);

    // 1. 翻转非卡牌资产 assets/wuxing.json 的 1 个字节
    let mut flipped_assets = wuxing_assets();
    flipped_assets.get_mut("assets/wuxing.json").unwrap()[0] ^= 0x01;
    assert!(
        validate_and_freeze_raw_package(WUXING_RAW.to_string(), flipped_assets.clone()).is_err(),
        "assets/wuxing.json 改 1 byte 必拒"
    );
    let mut tampered_copy = frozen.clone();
    tampered_copy.assets = flipped_assets;
    let lock = ContentLock::from_packages(vec![frozen.binding.clone()]);
    assert!(
        verify_locked_content_with_bridge(&lock, &[tampered_copy]).is_err(),
        "恢复校验必须覆盖非卡牌声明资产 assets/wuxing.json"
    );

    // 2. 缺失 assets/wuxing.json
    let mut missing_assets = wuxing_assets();
    missing_assets.remove("assets/wuxing.json");
    assert!(validate_and_freeze_raw_package(WUXING_RAW.to_string(), missing_assets).is_err());

    // 3. 多出未声明资产
    let mut extra_assets = wuxing_assets();
    extra_assets.insert("assets/extra.json".into(), b"{}".to_vec());
    assert!(validate_and_freeze_raw_package(WUXING_RAW.to_string(), extra_assets).is_err());

    // 4. 越界路径（../x, /abs, a\\b）
    for bad_path in ["../secret.json", "/etc/passwd", "assets\\cards.json"] {
        let mut bad_assets = wuxing_assets();
        bad_assets.insert(bad_path.to_string(), b"{}".to_vec());
        assert!(
            validate_and_freeze_raw_package(WUXING_RAW.to_string(), bad_assets).is_err(),
            "越界资产路径 {bad_path} 必拒"
        );
    }

    // 5. 声明资产 source_hash 为全 0 必拒
    let mut zero_hash_pkg =
        ContentPackage::from_json_bytes(WUXING_RAW.as_bytes()).expect("parse wuxing");
    zero_hash_pkg.asset_manifest.as_mut().unwrap().entries[0].source_hash = [0u8; 32];
    zero_hash_pkg.manifest.asset_hash = zero_hash_pkg.compute_asset_hash();
    let zero_raw = String::from_utf8(zero_hash_pkg.to_json_bytes().unwrap()).unwrap();
    assert!(
        validate_and_freeze_raw_package(zero_raw, wuxing_assets()).is_err(),
        "全 0 source_hash 必拒"
    );
}

#[test]
fn v5_section_7_weapon_range_mutation_has_no_bare_packs_bypass() {
    // 合法原包 huanle-upstream-noname：武器 hnl_kuwu_0 的 range = 2
    let frozen_r2 = validate_and_freeze_raw_package(HUANLE_RAW.to_string(), huanle_assets())
        .expect("freeze huanle r=2");
    let verified_r2 =
        prepare_locked_content_with_bridge(vec![frozen_r2.clone()]).expect("prepare r=2");

    // 即使调用方在外部构造了一份将 hnl_kuwu_0 range 改为 3 的裸 ExtensionPack 并保留同 provenance 名字：
    let mut mutated_external_pack =
        bridge_pack(std::str::from_utf8(HUANLE_CARDS_RAW).unwrap())
            .expect("bridge huanle")
            .pack;
    mutated_external_pack.def.name = engine_native::content::names::intern(format!(
        "installed:{}",
        serde_json::to_string(&frozen_r2.binding).unwrap()
    ));
    let kuwu = mutated_external_pack
        .cards
        .iter_mut()
        .find(|c| c.name == "hnl_kuwu_0")
        .expect("find kuwu");
    assert_eq!(kuwu.range, 2);
    kuwu.range = 3;

    // 受控工厂 `start_locked_engine` 与 `restore_locked_engine` 签名仅接收 `&VerifiedLockedContent`，
    // 没有任何参数位置可传入 `mutated_external_pack`：
    let engine_r2 =
        start_locked_engine(7, 2, LockedStartMode::Prototype, &verified_r2).expect("start r=2");
    let catalog_kuwu = engine_r2
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "hnl_kuwu_0")
        .expect("kuwu in catalog");
    assert_eq!(
        catalog_kuwu.range, 2,
        "受控工厂只从 VerifiedLockedContent 内部资产桥接，range 必须保持 2"
    );

    let snap_r2 = engine_r2.save();
    let restored_r2 = restore_locked_engine(&snap_r2, &verified_r2).expect("restore r=2");
    let restored_kuwu = restored_r2
        .catalog()
        .cards
        .iter()
        .find(|c| c.name == "hnl_kuwu_0")
        .expect("kuwu in restored catalog");
    assert_eq!(restored_kuwu.range, 2);

    // 若攻击者尝试直接修改 FrozenPackage 的 assets/cards.json（把 range: 2 改为 range: 3）并保留旧锁：
    let cards_r3_bytes = std::str::from_utf8(HUANLE_CARDS_RAW)
        .unwrap()
        .replacen("\"range\": 2", "\"range\": 3", 1)
        .into_bytes();
    let mut tampered_frozen = frozen_r2.clone();
    tampered_frozen
        .assets
        .insert("assets/cards.json".into(), cards_r3_bytes.clone());
    assert!(
        verify_locked_content_with_bridge(verified_r2.lock(), &[tampered_frozen]).is_err(),
        "修改 FrozenPackage 原资产并保留原锁必被 source_hash 校验拒绝"
    );

    // 若攻击者重新打包生成合法的 r=3 新包并取得新的 VerifiedLockedContent，则新锁与 r=2 原局存档不匹配，恢复必拒：
    let mut pkg_r3 = ContentPackage::from_json_bytes(HUANLE_RAW.as_bytes()).expect("parse huanle");
    let r3_source_hash = *blake3::hash(&cards_r3_bytes).as_bytes();
    pkg_r3.asset_manifest.as_mut().unwrap().entries[0].source_hash = r3_source_hash;
    pkg_r3.manifest.asset_hash = pkg_r3.compute_asset_hash();
    pkg_r3.manifest.logic_hash = pkg_r3.compute_logic_hash();
    let raw_r3 = String::from_utf8(pkg_r3.to_json_bytes().unwrap()).unwrap();
    let mut assets_r3 = BTreeMap::new();
    assets_r3.insert("assets/cards.json".into(), cards_r3_bytes);
    let frozen_r3 = validate_and_freeze_raw_package(raw_r3, assets_r3).expect("freeze r=3");
    let verified_r3 = prepare_locked_content_with_bridge(vec![frozen_r3]).expect("prepare r=3");

    assert!(
        matches!(
            restore_locked_engine(&snap_r2, &verified_r3),
            Err(SnapshotError::RuleVersionMismatch { .. })
        ),
        "r=3 新验证上下文恢复 r=2 存档必须因 provenance/content_lock_id 不匹配被拒"
    );
}
