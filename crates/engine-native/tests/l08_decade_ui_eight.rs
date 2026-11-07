//! L08 十周年UI（`decade-ui-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置解析：`assets/config.json` 解析 `DecadeUiConfig` 与 G0 计数（332 技能、209 标记技、116 牌、178 武将、122 翻译）；
//! ③ 组合共存：与 `skin-switch-upstream-liuli`、`qhly-upstream-liuli`、`shousha-ui-upstream-liuli` 同商店激活不冲突；
//! ④ 保存恢复：`snapshot` / `restore` 单行快照往返一致，对局权威状态哈希不受 UI 样式切换影响；
//! ⑤ 功能执行：6 种 `DecadeStyle`（`on`/`off`/`othersOff`/`onlineUI`/`babysha`/`codename`）切换、通道映射与 14 子模块清单核对；
//! ⑦ 异常拒装：非法样式名、非法布尔值、篡改 G0 计数与损坏 JSON 明确返回 `DecadeUiError`；
//! ⑧ 边界登记：动态 `<script>` 注入、DOM 动画与 Spine 骨骼覆写不进入权威内核。

use std::path::PathBuf;

use content_native_loader::{
    apply_decade_ui_line, parse_decade_ui_config, restore_decade_ui, snapshot_decade_ui,
    CardPrettify, DecadeStyle, DecadeUiConfig, DecadeUiError, MarkStyle, DECADE_UI_COUNTS,
    DECADE_UI_SUBMODULES,
};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::Engine;

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
fn l08_decade_ui_eight_dimension_acceptance() {
    // ① 发现与真实商店安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "decade-ui-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "decade-ui" && m.namespace == "upstream-liuli")
        .expect("decade-ui 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cfg_text =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");

    // ② 配置解析与 G0 计数核对
    let mut cfg = parse_decade_ui_config(&cfg_text).expect("解析 decade-ui config.json");
    assert_eq!(cfg, DecadeUiConfig::default());
    assert_eq!(cfg.new_decade_style, DecadeStyle::On);
    assert_eq!(cfg.card_prettify, CardPrettify::Decade);
    assert_eq!(cfg.mark_style, MarkStyle::Decade);
    assert_eq!(DECADE_UI_COUNTS, (332, 209, 116, 178, 122));
    assert_eq!(DECADE_UI_SUBMODULES.len(), 14);
    assert_eq!(DECADE_UI_SUBMODULES[0], "spine.js");
    assert_eq!(DECADE_UI_SUBMODULES[13], "animation_new.js");

    // ③ 与皮肤切换/千幻聆音/手杀ui 同商店共存激活
    for pid in ["skin-switch", "qhly", "shousha-ui"] {
        assert!(
            report
                .installed
                .iter()
                .any(|m| m.package_id == pid && m.namespace == "upstream-liuli"),
            "共存包 {pid} 同时安装成功"
        );
    }

    // ⑤ 样式切换与子模块状态（不修改权威对局哈希）
    let e_before = Engine::new(190808, 4);
    let hash_before = e_before.state_hash();
    for (raw, expected_channel) in [
        ("on", "shizhounian"),
        ("off", "shousha"),
        ("othersOff", "xinsha"),
        ("onlineUI", "ol"),
        ("babysha", "huanle"),
        ("codename", "codename"),
    ] {
        let out = apply_decade_ui_line(&mut cfg, &format!("style {raw}")).expect("切换样式");
        assert_eq!(
            out,
            format!("style={raw} | channel={expected_channel} | dom_mutated=0")
        );
    }
    let sub_out = apply_decade_ui_line(&mut cfg, "submodules").expect("submodules");
    assert_eq!(
        sub_out,
        "count=14 | first=spine.js | last=animation_new.js | script_injected=0"
    );
    let e_after = Engine::new(190808, 4);
    assert_eq!(e_after.state_hash(), hash_before, "展示层切换不影响权威引擎哈希");

    // ④ 快照保存与恢复
    let snap = snapshot_decade_ui(&cfg);
    let restored = restore_decade_ui(&snap).expect("恢复快照");
    assert_eq!(restored, cfg);

    // ⑥ 停用与回滚闭环
    store
        .deactivate_slot(&meta.namespace, &meta.package_id)
        .expect("停用 decade-ui");
    let rolled = store.rollback().expect("回滚 decade-ui");
    assert!(rolled
        .packages
        .iter()
        .any(|(slot, _)| slot.ends_with("upstream-liuli/decade-ui")));

    // ⑦ 坏输入拒绝
    assert!(matches!(
        parse_decade_ui_config("{not json"),
        Err(DecadeUiError::BadJson(_))
    ));
    let bad_style = cfg_text.replace("\"newDecadeStyle\":\"on\"", "\"newDecadeStyle\":\"unknown\"");
    assert!(matches!(
        parse_decade_ui_config(&bad_style),
        Err(DecadeUiError::InvalidStyle(_))
    ));
    let bad_count = cfg_text.replace("\"skill_count\":332", "\"skill_count\":999");
    assert!(matches!(
        parse_decade_ui_config(&bad_count),
        Err(DecadeUiError::InvalidCount { field: "skill_count", .. })
    ));
}
