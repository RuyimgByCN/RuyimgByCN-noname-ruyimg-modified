// ---------------------------------------------------------------------------
// P2/N10（ADR 0772）：策略类型隔离 + 有界任务调度 + AI 基线
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// E01–E03 扩展运行时基础（ADR 0776）
// ---------------------------------------------------------------------------

/// E01/E02：扩展注册表生命周期——安装→启用→禁用→卸载全流程。
#[test]
fn e01_extension_registry_lifecycle() {
    use engine_native::content::extension::{ExtensionDef, ExtensionRegistry};
    let mut reg = ExtensionRegistry::new();
    let def = ExtensionDef {
        name: "ext_a",
        version: 1,
        description: "A",
    };
    assert!(!reg.is_installed("ext_a"));
    reg.install(def).expect("安装");
    assert!(reg.is_installed("ext_a"));
    assert!(reg.is_enabled("ext_a"));
    reg.disable("ext_a").expect("禁用");
    assert!(!reg.is_enabled("ext_a"));
    reg.enable("ext_a").expect("启用");
    assert!(reg.is_enabled("ext_a"));
    reg.uninstall("ext_a").expect("卸载");
    assert!(!reg.is_installed("ext_a"));
    assert!(!reg.is_enabled("ext_a"));
}
