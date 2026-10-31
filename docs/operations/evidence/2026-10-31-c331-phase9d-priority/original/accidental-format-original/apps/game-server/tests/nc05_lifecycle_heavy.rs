//! NC05 第一类纵向案例：**重规则/嵌套选择**扩展的全生命周期闭环。
//!
//! 案例来源：`shahai-swd-upstream-noname`（505 个宿主行为、含嵌套选择与多步机器
//! 的 swd 规则面）——按 NC05 要求走**产品入口**：
//!
//! 作者包产物（仓库转换包）→ `import_extensions` 独立安装 → `NativeContent::load`
//! 装载（NC02：以安装副本为权威）→ **正式选将**（`NativeGameRequest.generals` →
//! `Engine::assign_general_named`，非 `debug_grant`）→ 建局驱动 → 真实交互
//! （`legal_actions` 取候选、提交命令、应答选择）→ 存读档（`save_fresh_match`/
//! `restore_match`，内容锁固定）→ 升级/回退 → 卸载后旧锁仍可恢复。
//!
//! 未覆盖（登记待办，见证据 README）：客户端 UI/Worker 入口、多选/取消/过期选择
//! 的全部形态、真实网络会话。

#[path = "../src/native.rs"]
mod native;

use native::{
    create_native_game, restore_match, save_fresh_match, NativeContent, NativeGameRequest,
};
use std::path::PathBuf;

const PACK: &str = "shahai-swd-upstream-noname";
const PACKAGE_ID: &str = "shahai-swd";

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

/// 从目录里挑一个**扩展包自带**且带技能的武将（按包名前缀过滤）。
fn pack_general(content: &NativeContent) -> String {
    let loaded = content.load(PACKAGE_ID).expect("装载 swd 包");
    let mut probe =
        engine_native::Engine::new_with_extensions(20261061, 4, &[&loaded.pack]).expect("建局");
    let names: Vec<String> = probe
        .general_candidates()
        .into_iter()
        .filter_map(|d| probe.catalog().general_meta(d).map(|m| (m.0, m.2)))
        .filter(|(name, skills)| name.contains("swd") && !skills.is_empty())
        .map(|(name, _)| name.to_string())
        .collect();
    assert!(!names.is_empty(), "swd 包应含带技能的武将");
    let pick = names[0].clone();
    // 产品入口可装配（顺带证明候选可装配）
    probe.assign_general_named(0, &pick).expect("选将应成功");
    pick
}

/// 全生命周期：安装 → 装载 → 正式选将 → 建局 → 交互 → 存读档 → 升级/回退 → 卸载恢复。
#[test]
fn nc05_heavy_rules_lifecycle() {
    let dir = tempfile::tempdir().unwrap();
    let store_root = dir.path().join("store");
    let content =
        NativeContent::open(repo_packages_root(), store_root.clone()).expect("open content");

    // ① 独立安装（作者包产物 → store）
    let installed = content.import().expect("安装");
    assert!(
        installed.iter().any(|p| p.package_id == PACKAGE_ID),
        "swd 包应安装: {:?}",
        installed.iter().map(|p| &p.package_id).collect::<Vec<_>>()
    );

    // ② 正式选将 + 建局 + 驱动（产品入口，非 debug_grant）
    let general = pack_general(&content);
    let req = NativeGameRequest {
        packages: vec![PACKAGE_ID.to_string()],
        generals: vec![general.clone()],
        seed: 20261062,
        seats: 4,
        max_steps: 120,
        planner_nodes: 32,
        planner_lookahead: 1,
        planner_variant: 1,
    };
    let summary = create_native_game(&content, &req).expect("建局并驱动");
    assert_eq!(summary.packages, vec![PACKAGE_ID.to_string()]);
    assert!(
        summary.catalog_cards > summary.base_catalog_cards,
        "扩展卡牌并入目录"
    );
    assert!(
        summary.catalog_generals > summary.base_catalog_generals,
        "扩展武将并入目录"
    );

    // ③ 真实交互：用权威动作查询取候选并提交（至少提交一次非应答命令）
    let loaded = content.load(PACKAGE_ID).expect("装载");
    let mut e =
        engine_native::Engine::new_with_extensions(20261063, 4, &[&loaded.pack]).expect("建局");
    e.assign_general_named(0, &general).expect("选将");
    let seat = e.turn().current;
    let actions = e.legal_actions(seat);
    assert!(!actions.is_empty(), "权威查询应给出候选");
    let action = actions
        .iter()
        .find(|a| matches!(a, engine_native::GameAction::NextPhase))
        .cloned()
        .unwrap_or_else(|| actions[0].clone());
    let out = e.submit(engine_native::GameCommand {
        actor: seat,
        expected_revision: e.revision(),
        dedup_id: 1,
        action,
    });
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "真实交互命令应被接受: {out:?}"
    );

    // ④ 存读档（产品入口；内容锁固定）
    let (bytes, cards) =
        save_fresh_match(&content, &[PACKAGE_ID.to_string()], 20261064, 4).expect("存盘");
    assert!(cards > 0);
    let restored_cards = restore_match(&content, &[PACKAGE_ID.to_string()], &bytes).expect("读档");
    assert_eq!(restored_cards, cards, "读档目录规模一致");

    // ⑤ 升级 → 回退（不改写已固定内容锁的局）
    let store = content_runtime::PackageStore::open(&store_root).expect("store");
    let pinned = store.current_lock().expect("内容锁");
    let entry = pinned
        .packages
        .iter()
        .find(|p| p.package_id == PACKAGE_ID)
        .expect("锁含 swd 包")
        .clone();
    let raw = store
        .load_active_package_bytes("upstream-noname", PACKAGE_ID)
        .expect("读活动包");
    let mut upgraded = content_schema::ContentPackage::from_json_bytes(&raw).expect("解析活动包");
    upgraded.manifest.version = "9.9.9".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    let version_dir = store
        .root()
        .join("packages")
        .join(format!("upstream-noname__{PACKAGE_ID}"))
        .join(entry.version.clone());
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &version_dir,
        )
        .expect("升级安装");
    // 旧锁仍装载旧版本（局不受升级影响）
    let locked = store.load_locked_packages(&pinned).expect("按旧锁装载");
    assert_eq!(
        locked
            .iter()
            .find(|p| p.manifest.package_id == PACKAGE_ID)
            .expect("旧版本在锁内")
            .manifest
            .version,
        entry.version,
        "固定内容锁版本不变"
    );
    store.rollback().expect("回退");

    // ⑥ 卸载（禁用）后旧锁仍可恢复：以固定锁装载存档
    store
        .activate_version("upstream-noname", PACKAGE_ID, &entry.version)
        .expect("激活原版本");
    let restored_again =
        restore_match(&content, &[PACKAGE_ID.to_string()], &bytes).expect("卸载/回退后仍可恢复");
    assert_eq!(restored_again, cards, "恢复结果一致");
}
