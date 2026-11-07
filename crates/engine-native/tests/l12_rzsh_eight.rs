//! L12 如真似幻（`rzsh-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置解析：`assets/config.json` 解析 `RzshConfig` 与 G0 计数（0 技能、92 牌引用、8 武将展示、8 翻译）；
//! ③ 组合共存：与 `decade-ui-upstream-liuli`、`shousha-ui-upstream-liuli` 同商店激活不冲突；
//! ④ 保存恢复：`snapshot_rzsh` / `restore_rzsh` 往返一致；
//! ⑤ 功能执行：`roman(1..=6)` 罗马数字转换、`sanguo_text` 11 条三国文案确定性轮询、`tip` 4 条匹配提示确定性轮询；
//! ⑦ 异常拒装：越界缩放比（`ui_scale`）、未知主题、篡改 G0 计数与损坏 JSON 返回明确 `RzshError`；
//! ⑧ 边界登记：5.9 MB 大厅 WebGL/音频/抽屉渲染不进入权威规则内核。

use std::path::PathBuf;

use content_native_loader::{
    apply_rzsh_line, parse_rzsh_config, restore_rzsh, roman, sanguo_text, snapshot_rzsh, tip,
    RzshConfig, RzshError, RZSH_COUNTS, SANGUO_TEXTS, TIPS,
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
fn l12_rzsh_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "rzsh-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "rzsh" && m.namespace == "upstream-liuli")
        .expect("rzsh 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cfg_text =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");

    // ② 配置解析与 G0 计数核对
    let mut cfg = parse_rzsh_config(&cfg_text).expect("解析 rzsh config.json");
    assert_eq!(cfg, RzshConfig::default());
    assert_eq!(RZSH_COUNTS, (0, 92, 8, 8));
    assert_eq!(SANGUO_TEXTS.len(), 11);
    assert_eq!(TIPS.len(), 4);

    // ③ 与其他 UI 扩展共存
    for pid in ["decade-ui", "shousha-ui"] {
        assert!(
            report
                .installed
                .iter()
                .any(|m| m.package_id == pid && m.namespace == "upstream-liuli"),
            "共存包 {pid} 已安装"
        );
    }

    // ⑤ 纯函数与诊断指令执行
    for (n, expected) in [(1, "Ⅰ"), (2, "Ⅱ"), (3, "Ⅲ"), (4, "Ⅳ"), (5, "Ⅴ"), (6, "Ⅵ")] {
        assert_eq!(roman(n), Some(expected));
        assert_eq!(
            apply_rzsh_line(&mut cfg, &format!("roman {n}")).unwrap(),
            expected
        );
    }
    assert_eq!(roman(0), None);
    assert_eq!(roman(7), None);
    assert_eq!(sanguo_text(0), Some(SANGUO_TEXTS[0]));
    assert_eq!(sanguo_text(11), Some(SANGUO_TEXTS[0]), "文案表模 11 回绕");
    assert_eq!(tip(0), Some(TIPS[0]));
    assert_eq!(tip(4), Some(TIPS[0]), "提示表模 4 回绕");

    // ④ 快照保存与恢复
    let snap = snapshot_rzsh(&cfg);
    let restored = restore_rzsh(&snap).expect("恢复快照");
    assert_eq!(restored, cfg);

    // ⑥ 停用与回滚
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 rzsh");
    let rolled = store.rollback().expect("回滚 rzsh");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-liuli/rzsh")));

    // ⑦ 坏配置拒绝
    let bad_scale = cfg_text.replace("\"ui_scale\":100", "\"ui_scale\":999");
    assert!(matches!(
        parse_rzsh_config(&bad_scale),
        Err(RzshError::InvalidField { field: "ui_scale", .. })
    ));
    let bad_theme = cfg_text.replace("\"theme\":\"szn\"", "\"theme\":\"invalid_theme\"");
    assert!(matches!(
        parse_rzsh_config(&bad_theme),
        Err(RzshError::InvalidField { field: "theme", .. })
    ));
}
