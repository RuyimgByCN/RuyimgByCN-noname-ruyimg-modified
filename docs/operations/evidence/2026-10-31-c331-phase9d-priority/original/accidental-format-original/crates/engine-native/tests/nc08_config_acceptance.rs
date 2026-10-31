//! NC08 附加：**「配置」验收**（八项验收第 3 项，纯 Rust 可做）。
//!
//! 依据：八项验收清单（[证据](../../docs/operations/evidence/2026-10-19-eight-item-acceptance/README.md)）——
//! 「配置」= 配置项可读写、默认值正确、**非法值被拒**。
//!
//! 本用例覆盖四个可机械核验的配置面：
//! 1. **元数据完整性**：每个已转换包 `def.name` 非空、`version >= 1`、`description` 非空；
//! 2. **`enabled` 开关语义**：把 `enabled` 置 `false` ⇒ 该包的卡**不进入目录**（且建局不报错）；
//! 3. **`init_hook` 解析**：包数据里的 `init_hook` 字符串**必须能解析**为已知枚举（未知值应被拒）；
//! 4. **非法数据拒绝**：畸形 `cards.json`（缺 `cards`/类型错误）⇒ `bridge_pack` 返回 **Err**。
//!
//! 边界：本用例**不覆盖**「运行时改配置」与「配置持久化」——二者登记为待办。

use std::path::{Path, PathBuf};

fn assets_dirs() -> Vec<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let mut out = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path().join("assets/cards.json");
            if p.is_file() {
                out.push(p);
            }
        }
    }
    out.sort();
    out
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).expect("读 cards.json")
}

/// 配置面 1（**元数据完整性**）：`name` 非空、`version >= 1`、`description` 非空。
#[test]
fn config_metadata_complete() {
    let mut checked = 0;
    for p in assets_dirs() {
        let pack = content_native_loader::bridge_pack(&read(&p))
            .expect("桥接应成功")
            .pack;
        assert!(
            !pack.def.name.is_empty(),
            "{}: def.name 不应为空",
            p.display()
        );
        assert!(
            pack.def.version >= 1,
            "{}: version 应 >= 1（实测 {}）",
            p.display(),
            pack.def.version
        );
        assert!(
            !pack.def.description.is_empty(),
            "{}: description 不应为空",
            p.display()
        );
        checked += 1;
    }
    eprintln!("DBG 配置-元数据：检查 {checked} 个包，全部通过");
    assert!(checked >= 10, "应检查至少 10 个已转换包（实测 {checked}）");
}

/// 配置面 2（**`enabled` 开关语义**）：置 `false` ⇒ 卡不进入目录，且建局不报错。
#[test]
fn config_enabled_flag_gates_cards() {
    let on_path = assets_dirs()
        .into_iter()
        .find(|p| p.display().to_string().contains("shahai-swd"))
        .expect("swd 包");
    let json = read(&on_path);
    // 开：目录含该包的卡
    let on = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    let on_cards = on.cards.len();
    // 关：置 enabled=false 后建局，目录不应含该包卡
    let mut off = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    off.enabled = false;
    let e_on = engine_native::Engine::new_with_extensions(11, 4, &[&on]).expect("开-建局");
    let e_off = engine_native::Engine::new_with_extensions(11, 4, &[&off]).expect("关-建局");
    let names_on = e_on.catalog().cards.len();
    let names_off = e_off.catalog().cards.len();
    eprintln!(
        "DBG 配置-enabled：包卡 {on_cards}；开-目录 {names_on}；关-目录 {names_off}（应更少）"
    );
    assert!(
        names_off < names_on,
        "enabled=false 时目录应更少（开 {names_on} → 关 {names_off}）"
    );
}

/// 配置面 3（**`init_hook` 解析**）：包数据的 `init_hook` 必须能解析为已知枚举。
#[test]
fn config_init_hook_parses() {
    let mut hooks = std::collections::BTreeSet::new();
    for p in assets_dirs() {
        let json = read(&p);
        if let Ok(v) = serde_json::from_str::<serde_json::Value>(&json) {
            if let Some(h) = v.pointer("/pack/init_hook").and_then(|x| x.as_str()) {
                hooks.insert(h.to_string());
                // 已知枚举值应被接受（未知值由 bridge_pack 拒绝——见配置面 4）
                assert!(
                    ["wuxing_natures", "swd_jiqi_held"].contains(&h),
                    "{}: 未登记的 init_hook `{h}`（若确为新增，请补 InitHook 枚举与解析）",
                    p.display()
                );
            }
        }
    }
    eprintln!("DBG 配置-init_hook：出现的取值 = {hooks:?}");
}

/// 配置面 4（**非法数据拒绝**）：畸形 `cards.json` ⇒ `bridge_pack` 返回 `Err`。
#[test]
fn config_invalid_data_rejected() {
    // ① 非 JSON
    assert!(
        content_native_loader::bridge_pack("not json at all").is_err(),
        "非 JSON 应被拒"
    );
    // ② 缺 cards 字段
    assert!(
        content_native_loader::bridge_pack("{\"pack\":{\"name\":\"x\",\"version\":1}}").is_err(),
        "缺 cards 字段应被拒（或按约定视为空包——若如此请调整本断言并登记）"
    );
    // ③ 未知 init_hook
    let bad_hook = "{\"pack\":{\"name\":\"x\",\"version\":1,\"init_hook\":\"nope\"},\"cards\":[]}";
    assert!(
        content_native_loader::bridge_pack(bad_hook).is_err(),
        "未知 init_hook 应被拒"
    );
    eprintln!("DBG 配置-非法数据：三类畸形输入均被拒");
}
