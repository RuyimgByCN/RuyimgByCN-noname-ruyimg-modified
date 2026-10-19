//! NC08 第⑦层（上游行为对照）样本 51：`mashu`（马术，**标准面** / 马超）与 `feiying`（飞影）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! mashu: {                                   // 马超：计算与他人的距离 -1
//!     mod: { globalFrom(from, to, distance) { return distance - 1; } },
//! },
//! feiying: {                                 // 曹洪：他人计算与你的距离 +1
//!     mod: { globalTo(from, to, distance) { return distance + 1; } },
//! },
//! ```
//!
//! 由 JS 推导的规则点：**马术** = 自己到他人的距离 **−1**（`globalFrom`）；
//! **飞影** = 他人到自己的距离 **+1**（`globalTo`）——两者**方向相反**。
//!
//! 原生实现引用：`state.rs:1248`（马术：`minus += u32::from(mashu)`，与 −1 马同口径，
//! 注释「计算与他人的距离时始终 -1（下限 1）」）。
//!
//! **登记差异（本文件固定，属规则损失）**：**飞影未实现**——`crates/engine-native/src/`
//! 全库检索 `飞影`/`FEI_YING`/`feiying` **均无命中**，而 `plus` 累加面（`state.rs:1234` 起）
//! 已有义从/谛听/云身等同类实现 → 飞影的 `globalTo +1` **缺失**（**标准面技能未实现**）。
//!
//! 未覆盖（登记）：`mashu` 的**下限 1** 行为面（需构造极近距离场景）。

use engine_native::Engine;

/// 上游点（**马术**）：原生以 `minus` 累加实现，且注释标明「下限 1」。
#[test]
fn upstream_mashu_implemented_as_minus() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/state.rs"
    ))
    .expect("读 state");
    let idx = src.find("MA_SHU").expect("马术");
    let seg: String = src[..idx].chars().rev().take(160).collect::<String>().chars().rev().collect();
    let after: String = src[idx..].chars().take(400).collect();
    assert!(
        after.contains("minus") && (seg.contains("马术") || after.contains("马术")),
        "原生依据：马术以 minus 累加（距离 -1）：{after}"
    );
    assert!(
        after.contains("下限 1") || seg.contains("下限 1"),
        "原生注释应标明距离下限 1"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tmashu: {").expect("上游马术");
    let seg: String = up[idx..].chars().take(200).collect();
    assert!(
        seg.contains("globalFrom") && seg.contains("distance - 1"),
        "上游依据：globalFrom 返回 distance - 1：{seg}"
    );
}

/// 上游点（**飞影**）→ **登记差异（规则损失）**：原生全库未见实现。
#[test]
fn upstream_feiying_difference_is_registered() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("\tfeiying: {").expect("上游飞影");
    let seg: String = up[idx..].chars().take(200).collect();
    assert!(
        seg.contains("globalTo") && seg.contains("distance + 1"),
        "上游依据：globalTo 返回 distance + 1：{seg}"
    );
    // 原生：标准技能名表中**没有**飞影常量（对照 names.rs 的实现索引）
    let names = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/names.rs"
    ))
    .expect("读 names");
    assert!(
        !names.contains("飞影"),
        "差异固定：names.rs 的实现索引中**没有**飞影（标准面技能未实现）"
    );
    let _ = Engine::new(20268301, 4);
}
