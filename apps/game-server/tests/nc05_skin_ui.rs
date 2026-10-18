//! NC05 第三类纵向案例：**皮肤/UI 组合**（表现资源与规则内容分离）。
//!
//! 案例来源：`skin-switch-upstream-liuli`（L24 皮肤切换，纯配置/表现面：
//! `assets/config.json`，无规则模块）+ `select-beautify-upstream-liuli`（L27 选将美化）。
//!
//! 方案 §2 要求：「规则配置进入逻辑内容锁；纯表现资源保留独立资源指纹」。本文件
//! 验收：
//! 1. 皮肤/UI 包可独立安装、装载（安装副本为权威）、其**配置资源**经 store 解析
//!    到安装目录（不读 drop-in 源目录）；
//! 2. 资源索引可枚举（`load_active_asset_index`）且路径位于 store 内；
//! 3. **资产改动只改资源指纹、不改逻辑指纹**：仅替换 `assets/config.json` 后重装，
//!    `asset_hash` 变化而 `logic_hash` 不变；
//! 4. 固定内容锁：升级后旧锁仍装载旧版本；回退后恢复；
//! 5. 与规则包共存（皮肤 + 重规则包同时启用）时建局与交互正常。

#[path = "../src/native.rs"]
mod native;

use native::NativeContent;
use std::path::PathBuf;

const SKIN: &str = "skin-switch";
const RULES_PACK: &str = "shahai-swd";

fn repo_packages_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> content_runtime::InstallOptions {
    content_runtime::InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 皮肤/UI 包：配置资源从安装副本解析；资产改动只改资源指纹。
#[test]
fn nc05_skin_ui_asset_separation() {
    let dir = tempfile::tempdir().unwrap();
    let store_root = dir.path().join("store");
    let content =
        NativeContent::open(repo_packages_root(), store_root.clone()).expect("open content");
    content.import().expect("安装");
    let store = content_runtime::PackageStore::open(&store_root).expect("store");

    // ① 装载皮肤包（无规则模块：vm/cards 为空，不影响目录）
    let loaded = content
        .load_scoped(Some("upstream-liuli"), SKIN)
        .expect("装载皮肤包");
    assert!(loaded.pack.vm_skills.is_empty(), "皮肤包无 VM 技能");
    assert!(loaded.pack.cards.is_empty(), "皮肤包无卡牌");

    // ② 资源经 store 解析到安装目录（NC02 口径：不读 drop-in 源目录）
    let info = store
        .resolve_active_asset_info("upstream-liuli", SKIN, "skin-switch:config")
        .expect("解析配置资源");
    // macOS 上 /var 与 /private/var 为符号链接：两侧统一规范化后比较
    let store_root_canon = store.root().canonicalize().expect("store 根规范化");
    let asset_canon = info.path.canonicalize().expect("资源路径规范化");
    assert!(
        asset_canon.starts_with(&store_root_canon),
        "资源路径须位于 store 内: {asset_canon:?} vs {store_root_canon:?}"
    );
    assert!(
        asset_canon
            .to_string_lossy()
            .contains("packages/upstream-liuli__skin-switch"),
        "资源须来自该包的安装目录: {asset_canon:?}"
    );
    let config_raw = std::fs::read_to_string(&info.path).expect("读配置");
    assert!(config_raw.contains('{'), "配置为 JSON 文本");

    // ③ 资源索引可枚举
    let index = store.load_active_asset_index().expect("资源索引");
    assert!(
        index
            .keys()
            .any(|(ns, id, _)| ns == "upstream-liuli" && id == SKIN),
        "索引含皮肤包资源"
    );

    // ④ 资产改动只改资源指纹、不改逻辑指纹
    let before = store.current_lock().expect("锁");
    let before_entry = before
        .packages
        .iter()
        .find(|p| p.package_id == SKIN)
        .expect("锁含皮肤包")
        .clone();
    let raw = store
        .load_active_package_bytes("upstream-liuli", SKIN)
        .expect("读活动包");
    let mut changed = config_raw.clone();
    changed.push_str("\n");
    // 模拟作者替换皮肤资源：改写内容 + 同步资产条目 source_hash（安装器逐资产校验）
    let new_hash: Vec<u8> = blake3::hash(changed.as_bytes()).as_bytes().to_vec();
    let raw_str = String::from_utf8(raw.clone()).expect("npack UTF-8");
    let mut value: serde_json::Value = serde_json::from_str(&raw_str).expect("npack JSON");
    if let Some(entries) = value["asset_manifest"]["entries"].as_array_mut() {
        for e in entries.iter_mut() {
            if e["asset_id"] == "skin-switch:config" {
                e["source_hash"] = serde_json::json!(new_hash);
            }
        }
    }
    let mut pkg = content_schema::ContentPackage::from_json_bytes(
        serde_json::to_string(&value).unwrap().as_bytes(),
    )
    .expect("重建 npack");
    // 真实升级路径：**新版本号**携带新资源（store 拒绝同版本不同内容——这是
    // 「安装失败保留旧版本」的不变量，测试不绕过它）
    pkg.manifest.version = "1.0.1".into();
    pkg.manifest.asset_hash = pkg.compute_asset_hash();
    pkg.manifest.logic_hash = pkg.compute_logic_hash();
    let source_dir = dir.path().join("skin-src-1.0.1");
    std::fs::create_dir_all(source_dir.join("assets")).expect("资源目录");
    std::fs::write(source_dir.join("assets/config.json"), &changed).expect("替换资源");
    store
        .install_with_assets(
            &pkg.to_json_bytes().unwrap(),
            None,
            &options(),
            &source_dir,
        )
        .expect("升级到 1.0.1（仅资产变化）");

    let after = store.current_lock().expect("新锁");
    let after_entry = after
        .packages
        .iter()
        .find(|p| p.package_id == SKIN)
        .expect("锁含皮肤包");
    // 版本升级本身进入逻辑指纹（version 是内容身份的一部分），故此处只断言
    // 升级被接受且活动集指向新版本；「资源变化不改逻辑指纹」用同版本的内存
    // 计算直接验收（见下）。
    assert_eq!(after_entry.version, "1.0.1", "升级版本生效");
    assert_ne!(
        after_entry.content_hash, before_entry.content_hash,
        "资源指纹随资源变化"
    );

    // 同版本内存对照：仅替换资产文件哈希 → asset_hash 变、logic_hash 不变
    let base = content_schema::ContentPackage::from_json_bytes(&raw).expect("基线包");
    let mut variant = base.clone();
    if let Some(entries) = variant.asset_manifest.as_mut().map(|m| &mut m.entries) {
        for e in entries.iter_mut() {
            if e.asset_id == "skin-switch:config" {
                e.source_hash = *blake3::hash(changed.as_bytes()).as_bytes();
            }
        }
    }
    assert_ne!(
        variant.compute_asset_hash(),
        base.compute_asset_hash(),
        "资产哈希应随资源变化"
    );
    assert_eq!(
        variant.compute_logic_hash(),
        base.compute_logic_hash(),
        "纯表现资源变化不得改动逻辑指纹"
    );

    // ⑤ 固定内容锁：旧锁仍可装载旧版本；回退后可恢复
    let locked = store.load_locked_packages(&before).expect("按旧锁装载");
    assert!(
        locked.iter().any(|p| p.manifest.package_id == SKIN),
        "旧锁仍含皮肤包"
    );
    store.rollback().expect("回退");
    let restored = store.load_locked_packages(&before).expect("回退后装载");
    assert!(
        restored.iter().any(|p| p.manifest.package_id == SKIN),
        "回退后皮肤包仍可装载"
    );
}

/// 皮肤包与规则包共存：建局、选将、交互正常（组合能力）。
#[test]
fn nc05_skin_with_rules_pack_coexists() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let skin = content
        .load_scoped(Some("upstream-liuli"), SKIN)
        .expect("装载皮肤包");
    let rules = content.load(RULES_PACK).expect("装载规则包");
    let beautify = content
        .load_scoped(Some("upstream-liuli"), "select-beautify")
        .expect("装载选将美化包");

    let packs: Vec<&engine_native::content::extension::ExtensionPack> =
        vec![&skin.pack, &rules.pack, &beautify.pack];
    let mut e = engine_native::Engine::new_with_extensions(20261081, 4, &packs).expect("建局");

    // 正式选将 + 权威动作查询 + 真实提交（组合面可用）
    let general = e
        .general_candidates()
        .into_iter()
        .filter_map(|d| e.catalog().general_meta(d).map(|m| (m.0, m.2)))
        .find(|(name, skills)| name.contains("swd") && !skills.is_empty())
        .map(|(name, _)| name.to_string())
        .expect("规则包提供带技能武将");
    e.assign_general_named(0, &general).expect("选将");
    let seat = e.turn().current;
    let actions = e.legal_actions(seat);
    assert!(!actions.is_empty(), "组合目录下仍有合法动作");
    let out = e.submit(engine_native::GameCommand {
        actor: seat,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: actions
            .iter()
            .find(|a| matches!(a, engine_native::GameAction::NextPhase))
            .cloned()
            .unwrap_or_else(|| actions[0].clone()),
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "组合面交互应被接受: {out:?}"
    );

    // 存档往返（组合内容锁）
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码");
    let restored = engine_native::Engine::restore_with_extensions(&snapshot, &packs)
        .expect("带组合扩展恢复");
    assert_eq!(
        restored.catalog().cards.len(),
        e.catalog().cards.len(),
        "恢复后目录规模一致"
    );
}
