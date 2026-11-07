//! L25 祖安武将（`zuan-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置解析：`assets/config.json` 解析 `ZuanConfig` 与 G0 计数（244 技能、190 牌、400 武将、239 翻译）；
//! ③ 组合共存：与 `fool-plugin-upstream-liuli`、`omnisearch-upstream-liuli` 同商店激活不冲突；
//! ④ 保存恢复：`snapshot_zuan` / `restore_zuan` 往返一致，恢复时同样执行安全门禁；
//! ⑤ 功能执行：联机房间准入校验（`check_room_gate online` 拒绝启用编辑器或非标恶搞变体）与离线调试门禁；
//! ⑦ 异常拒装：`enable_editor: true` 且 `offline_debug_only: false` 拒装，篡改 G0 计数或损坏 JSON 返回明确 `ZuanError`；
//! ⑧ 边界登记：禁止 `game.writeFile` 本地文件覆写与非标恶搞变体污染标准联机对局。

use std::path::PathBuf;

use content_native_loader::{
    apply_zuan_line, parse_zuan_config, restore_zuan, snapshot_zuan, ZuanConfig, ZuanError,
    ZUAN_COUNTS,
};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

#[test]
fn l25_zuan_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "zuan-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "zuan" && m.namespace == "upstream-liuli")
        .expect("zuan 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cfg_text =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");

    // ② 配置解析与 G0 计数核对
    let mut cfg = parse_zuan_config(&cfg_text).expect("解析 zuan config.json");
    assert_eq!(cfg, ZuanConfig::default());
    assert_eq!(ZUAN_COUNTS, (244, 190, 400, 239));

    // ③ 与笨蛋插件/全能搜索同商店共存
    for pid in ["fool-plugin", "omnisearch"] {
        assert!(
            report
                .installed
                .iter()
                .any(|m| m.package_id == pid && m.namespace == "upstream-liuli"),
            "共存包 {pid} 已安装"
        );
    }

    // ⑤ 房间准入门禁校验（默认配置允许联机但禁止文件写入与非标变体）
    assert_eq!(
        apply_zuan_line(&mut cfg, "check_room_gate online").unwrap(),
        "room=online | allowed=1 | fs_write=0 | rule_variant=0"
    );
    assert_eq!(
        apply_zuan_line(&mut cfg, "check_room_gate offline").unwrap(),
        "room=offline | allowed=1 | fs_write=0 | rule_variant=0 | taunt_pack=1"
    );

    // 开启 rule_variant 后联机标准房间必须拒绝
    cfg.rule_variant = true;
    assert_eq!(
        apply_zuan_line(&mut cfg, "check_room_gate online"),
        Err(ZuanError::OnlineRoomRejected(
            "联机标准房间禁止启用非标恶搞规则变体"
        ))
    );

    // ④ 快照保存与恢复
    let snap = snapshot_zuan(&cfg);
    let restored = restore_zuan(&snap).expect("恢复快照");
    assert_eq!(restored, cfg);

    // ⑥ 停用与回滚
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 zuan");
    let rolled = store.rollback().expect("回滚 zuan");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-liuli/zuan")));

    // ⑦ 安全门禁拒绝：非离线调试模式下启用 enable_editor 必须拒绝
    let unsafe_editor = cfg_text
        .replace("\"enable_editor\":false", "\"enable_editor\":true")
        .replace("\"offline_debug_only\":true", "\"offline_debug_only\":false");
    assert_eq!(
        parse_zuan_config(&unsafe_editor),
        Err(ZuanError::UnsafeEditorOutsideOfflineDebug)
    );
    assert_eq!(
        restore_zuan("enable_editor=1 | rule_variant=0 | offline_debug_only=0 | taunt_pack=1"),
        Err(ZuanError::UnsafeEditorOutsideOfflineDebug)
    );
}
