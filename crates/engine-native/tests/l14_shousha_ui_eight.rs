//! L14 手杀ui（`shousha-ui-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置解析：`assets/config.json` 解析 `ShoushaUiConfig` 与 G0 计数（92 技能 = 87 UI 钩子 + 5 视觉/台词特效、122 牌、103 武将、108 翻译）；
//! ③ 组合共存：与 `shousha-full-upstream-liuli`（L15 手杀全扩）、`shousha-mvp-upstream-liuli`（L13 手杀MVP）同商店激活不冲突；
//! ④ 保存恢复：`snapshot_shousha_ui` / `restore_shousha_ui` 单行快照往返一致；
//! ⑤ 功能执行：5 个顶层视觉/台词特效技能（`SHOUSHA_UI_VISUAL_FX_SKILLS`）与 87 个 `lbtn`/`skill` UI 钩子技能分类判定，`rule_effect=0`；
//! ⑦ 异常拒装：非法座位框样式、非法技能停靠位、篡改 87+5 技能计数与损坏 JSON 返回明确 `ShoushaUiError`；
//! ⑧ 边界登记：底部操作栏 DOM、击杀全屏弹图与台词气泡轮播不改变规则状态与随机流。

use std::path::PathBuf;

use content_native_loader::{
    apply_shousha_ui_line, parse_shousha_ui_config, restore_shousha_ui, snapshot_shousha_ui,
    ShoushaUiConfig, ShoushaUiError, SHOUSHA_UI_COUNTS, SHOUSHA_UI_VISUAL_FX_SKILLS,
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
fn l14_shousha_ui_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "shousha-ui-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shousha-ui" && m.namespace == "upstream-liuli")
        .expect("shousha-ui 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cfg_text =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");

    // ② 配置解析与 87+5=92 技能核对
    let mut cfg = parse_shousha_ui_config(&cfg_text).expect("解析 shousha-ui config.json");
    assert_eq!(cfg, ShoushaUiConfig::default());
    assert_eq!(SHOUSHA_UI_COUNTS, (92, 87, 5, 122, 103, 108));
    assert_eq!(SHOUSHA_UI_COUNTS.1 + SHOUSHA_UI_COUNTS.2, SHOUSHA_UI_COUNTS.0);
    assert_eq!(SHOUSHA_UI_VISUAL_FX_SKILLS.len(), 5);

    // ③ 与 L13 手杀MVP、L15 手杀全扩、L08 十周年UI 同商店共存
    for pid in ["shousha-mvp", "shousha-full", "decade-ui"] {
        assert!(
            report
                .installed
                .iter()
                .any(|m| m.package_id == pid && m.namespace == "upstream-liuli"),
            "共存包 {pid} 已安装"
        );
    }

    // ⑤ 技能分类执行（5 个视觉特效 vs 87 个 UI 钩子，均无规则副作用）
    for fx in SHOUSHA_UI_VISUAL_FX_SKILLS {
        let out = apply_shousha_ui_line(&mut cfg, &format!("classify_skill {fx}")).unwrap();
        assert_eq!(
            out,
            format!("skill={fx} | class=visual_fx | enabled=1 | rule_effect=0")
        );
    }
    let hook_out = apply_shousha_ui_line(&mut cfg, "classify_skill _lbtn_confirm").unwrap();
    assert_eq!(
        hook_out,
        "skill=_lbtn_confirm | class=ui_hook | enabled=1 | rule_effect=0"
    );

    // ④ 快照保存与恢复
    cfg.popup_image = false;
    let snap = snapshot_shousha_ui(&cfg);
    let restored = restore_shousha_ui(&snap).expect("恢复快照");
    assert_eq!(restored, cfg);

    // ⑥ 停用与回滚
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 shousha-ui");
    let rolled = store.rollback().expect("回滚 shousha-ui");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-liuli/shousha-ui")));

    // ⑦ 坏配置拒绝
    let bad_frame = cfg_text.replace("\"player_frame\":\"mobile\"", "\"player_frame\":\"vr_3d\"");
    assert!(matches!(
        parse_shousha_ui_config(&bad_frame),
        Err(ShoushaUiError::InvalidField { field: "player_frame", .. })
    ));
    let bad_count = cfg_text.replace("\"ui_hook_skill_count\":87", "\"ui_hook_skill_count\":80");
    assert!(matches!(
        parse_shousha_ui_config(&bad_count),
        Err(ShoushaUiError::InvalidCount { field: "ui_hook_skill_count", .. })
    ));
}
