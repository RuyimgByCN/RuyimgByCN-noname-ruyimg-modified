//! NC05 第二类纵向案例：**战役/跨局档案 + 升级恢复**（Boss 关卡进度）。
//!
//! 案例来源：`boss-play-upstream-liuli`（Boss 模式扩展包）+ 原生 Boss 关卡状态
//! （`GameMode::Boss.stage`：关卡种类/等级/孟婆旗标/禁将表/选将挂起）。
//!
//! 验收（产品入口，非调试授技）：
//! 1. 独立安装扩展包 → 装载（安装副本为权威）→ **Boss 关卡建局**（产品构造入口）；
//! 2. 关卡进度写入 → **归档往返**（`encode_archive`/`decode_archive` + 带扩展恢复）
//!    后进度与目录规模一致；
//! 3. 固定内容锁：扩展包升级到 9.9.9 后，**旧锁仍装载旧版本**（战役档案不被改写）；
//! 4. 回退/重新激活原版本后，档案仍可恢复且进度不变。
//!
//! 未覆盖（登记待办）：跨局档案的**产品持久化层**（game-server `persist.rs` 的
//! 房间存档）与真实客户端；本文件覆盖内核档案 + 内容锁的升级/恢复语义。

#[path = "../src/native.rs"]
mod native;

use native::NativeContent;
use std::path::PathBuf;

const PACKAGE_ID: &str = "boss-play";

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

/// 战役/跨局档案：Boss 关卡进度 + 扩展包的安装/装载/升级/恢复闭环。
#[test]
fn nc05_campaign_profile_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let store_root = dir.path().join("store");
    let content =
        NativeContent::open(repo_packages_root(), store_root.clone()).expect("open content");

    // ① 独立安装 + 装载（安装副本为权威）
    let installed = content.import().expect("安装");
    assert!(
        installed.iter().any(|p| p.package_id == PACKAGE_ID),
        "boss-play 包应安装: {:?}",
        installed.iter().map(|p| &p.package_id).collect::<Vec<_>>()
    );
    // NC02：boss-play 在两个命名空间下都存在 → 显式指定命名空间（这正是
    // NC02「同 id 不同命名空间需显式选择」的行为）
    let loaded = content
        .load_scoped(Some("upstream-liuli"), PACKAGE_ID)
        .expect("装载 boss-play（upstream-liuli）");
    let pack = loaded.pack;

    // ② Boss 关卡建局（产品构造入口）+ 进度写入
    let mut e = engine_native::Engine::new_boss_stage_with_extensions(
        20261071,
        5,
        0,
        3,
        4,
        4,
        0,
        &[&pack],
    )
    .expect("Boss 关卡建局");
    e.debug_set_boss_progress(3, true, false);
    let cards_before = e.catalog().cards.len();
    let generals_before = e.catalog().general_count();

    // ③ 归档往返（跨局档案）：带扩展恢复后进度与目录一致
    let bytes = engine_native::archive::encode_archive(&e.save());
    assert!(!bytes.is_empty(), "档案字节非空");
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码档案");
    let restored = engine_native::Engine::restore_with_extensions(&snapshot, &[&pack])
        .expect("带扩展恢复");
    assert_eq!(
        restored.catalog().cards.len(),
        cards_before,
        "恢复后卡牌目录规模一致"
    );
    assert_eq!(
        restored.catalog().general_count(),
        generals_before,
        "恢复后武将目录规模一致"
    );
    let progress = restored
        .boss_stage_progress()
        .expect("Boss 关卡进度在档案内");
    assert_eq!(progress.0, 3, "关卡等级保留");
    assert!(progress.1, "孟婆已出现旗标保留");
    assert!(!progress.2, "孟婆未死亡旗标保留");

    // ④ 固定内容锁：升级后旧锁仍装载旧版本（战役档案不被改写）
    let store = content_runtime::PackageStore::open(&store_root).expect("store");
    let pinned = store.current_lock().expect("内容锁");
    let entry = pinned
        .packages
        .iter()
        .find(|p| p.package_id == PACKAGE_ID)
        .expect("锁含 boss-play")
        .clone();
    let raw = store
        .load_active_package_bytes("upstream-liuli", PACKAGE_ID)
        .expect("读活动包");
    let mut upgraded = content_schema::ContentPackage::from_json_bytes(&raw).expect("解析活动包");
    upgraded.manifest.version = "9.9.9".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    let version_dir = store
        .root()
        .join("packages")
        .join(format!("upstream-liuli__{PACKAGE_ID}"))
        .join(entry.version.clone());
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &version_dir,
        )
        .expect("升级安装");
    let locked = store.load_locked_packages(&pinned).expect("按旧锁装载");
    let locked_version = locked
        .iter()
        .find(|p| p.manifest.package_id == PACKAGE_ID)
        .expect("旧版本在锁内")
        .manifest
        .version
        .clone();
    assert_eq!(locked_version, entry.version, "固定内容锁版本不变");

    // ⑤ 回退 + 重新激活原版本后，档案仍可恢复且进度不变
    store.rollback().expect("回退");
    store
        .activate_version("upstream-liuli", PACKAGE_ID, &entry.version)
        .expect("激活原版本");
    let restored_again = engine_native::Engine::restore_with_extensions(&snapshot, &[&pack])
        .expect("回退后恢复");
    let progress_again = restored_again
        .boss_stage_progress()
        .expect("进度仍在");
    assert_eq!(progress_again, progress, "回退后进度一致");
}

/// 对照：跨局档案拒绝错误版本（内容锁/规则版本不符时明确失败）。
#[test]
fn nc05_campaign_profile_rejects_wrong_content() {
    let dir = tempfile::tempdir().unwrap();
    let content =
        NativeContent::open(repo_packages_root(), dir.path().join("store")).expect("open");
    content.import().expect("安装");
    let loaded = content
        .load_scoped(Some("upstream-liuli"), PACKAGE_ID)
        .expect("装载（显式命名空间）");

    let mut e = engine_native::Engine::new_boss_stage_with_extensions(
        20261072,
        5,
        0,
        3,
        4,
        4,
        0,
        &[&loaded.pack],
    )
    .expect("建局");
    e.debug_set_boss_progress(1, false, false);
    let bytes = engine_native::archive::encode_archive(&e.save());
    let snapshot = engine_native::archive::decode_archive(&bytes).expect("解码");

    // 无扩展恢复（内容不符）应被拒绝
    let err = engine_native::Engine::restore(&snapshot);
    assert!(err.is_err(), "缺少扩展时恢复应明确失败");
}
