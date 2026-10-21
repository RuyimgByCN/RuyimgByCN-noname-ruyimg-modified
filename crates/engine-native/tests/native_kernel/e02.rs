// ---------------------------------------------------------------------------
// E02/E03 扩展基础接线（ADR 0776 修订）：
// 注册表拒绝未安装包操作；扩展卡牌经内容锁合并入真实目录；
// 存档按扩展集合（名称+版本指纹）绑定恢复。
// ---------------------------------------------------------------------------

/// E02：启用/禁用/卸载未安装扩展必须返回错误（此前 enable 可启用
/// 未安装名——反例关闭）；重复安装返回错误。
#[test]
fn e02_registry_rejects_uninstalled_operations() {
    use engine_native::content::extension::{ExtensionDef, ExtensionError, ExtensionRegistry};
    let mut reg = ExtensionRegistry::new();
    // 旧反例：enable("not_installed") 曾被接受
    assert_eq!(
        reg.enable("not_installed"),
        Err(ExtensionError::NotInstalled {
            name: "not_installed"
        })
    );
    assert!(!reg.is_enabled("not_installed"));
    assert_eq!(
        reg.disable("not_installed"),
        Err(ExtensionError::NotInstalled {
            name: "not_installed"
        })
    );
    assert_eq!(
        reg.uninstall("not_installed"),
        Err(ExtensionError::NotInstalled {
            name: "not_installed"
        })
    );
    let def = ExtensionDef {
        name: "ext_a",
        version: 1,
        description: "",
    };
    reg.install(def).expect("install");
    // 重复安装 → 错误（升级须显式卸载）
    assert_eq!(
        reg.install(ExtensionDef {
            name: "ext_a",
            version: 2,
            description: ""
        }),
        Err(ExtensionError::AlreadyInstalled { name: "ext_a" })
    );
    assert!(reg.is_installed("ext_a"));
}
