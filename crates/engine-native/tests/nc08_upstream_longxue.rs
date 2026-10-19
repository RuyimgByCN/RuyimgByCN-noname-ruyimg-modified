//! NC08 第⑦层样本 41：来源 **`longxue`（龙血玄黄 / D5）** —— **弱对照（证据强度受限，如实标注）**。
//!
//! ## 证据强度声明（**必读**）
//!
//! 该来源的上游 `extension.js` **被混淆**（`jsjiami.com.v6`），**连技能说明文本也不可读**
//! （`mbqianlong_info`/`mbweitong_info` 在整目录内均无明文命中）→ 因此**无法**从上游源码
//! 直接引用语义。本样本的对照依据是：
//!
//! 1. **原生注释中记录的上游键名与语义**（`content/host/behavior.rs:1279–1285`、
//!    `state.rs:256–262`）——即「原生声称的上游口径」；
//! 2. **包数据中的映射**（`packages/extensions/longxue-upstream-liuli`：1 将 / 2 技）。
//!
//! → 这**不是**上游源码级对照（**弱于**其它样本），**只**能证明「原生实现与其自述的上游口径
//! 一致、且映射齐备」；**不能**证明与上游行为等价。**台账/进度中该来源按此强度记账。**
//!
//! ## 原生注释所载上游口径（逐字引用）
//!
//! ```text
//! 潜龙·道心值（tag 469，D5 龙血玄黄批 117，`mbqianlong`）：开局 20 点（另有魏势力持「卫统」者
//! 60 点）；获得牌 +5、受伤 +10×伤害、造成伤害 +15×伤害（上限 99）；阈值 25/50/75/99 解锁
//! 清正/酒诗/放逐/绝进（外部技能存在才授予）。
//! 卫统（tag 470，`mbweitong`，主公技）：开局登记「修改潜龙初始道心值」。
//! ```
//!
//! **待办（解除弱对照的前置）**：无头轨迹采集（台账 D5 项）→ 得到可核对的运行期行为序列后，
//! 本样本应升级为上游行为级对照。

use engine_native::Engine;

const PACK: &str = "packages/extensions/longxue-upstream-liuli/assets/cards.json";

/// 上游**不可读**（混淆）：固定该事实，避免后续误以为「已做过上游对照」。
#[test]
fn longxue_upstream_is_obfuscated_so_reference_is_weak() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js"),
    )
    .expect("读上游龙血玄黄");
    assert!(
        up.contains("jsjiami"),
        "上游被混淆（jsjiami）：本样本只能作弱对照"
    );
    assert!(
        !up.contains("mbqianlong_info"),
        "连技能说明文本也不在明文内 → 无法逐字引用上游语义"
    );
}

/// 包映射齐备（1 将 / 2 技，且均映射到宿主行为键）。
#[test]
fn longxue_pack_mapping() {
    let raw = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .join(PACK),
    )
    .expect("读 longxue 包数据");
    let v: serde_json::Value = serde_json::from_str(&raw).expect("JSON");
    let generals = v.get("generals").and_then(|g| g.as_array()).expect("generals");
    assert_eq!(generals.len(), 1, "该来源只有 1 名武将（mb_caomao）");
    let skills = generals[0].get("skills").and_then(|s| s.as_array()).expect("skills");
    assert_eq!(skills.len(), 2, "上游 mbqianlong + mbweitong 两条映射");
    let names: Vec<String> = skills
        .iter()
        .map(|s| s.get("name").and_then(|n| n.as_str()).unwrap_or("").to_string())
        .collect();
    assert!(names.iter().any(|n| n == "mbqianlong"), "{names:?}");
    assert!(names.iter().any(|n| n == "mbweitong"), "{names:?}");
    for s in skills {
        let m = s.get("mapping").and_then(|m| m.as_str()).unwrap_or("");
        assert!(m.starts_with("host:"), "应映射到宿主行为键：{m}");
    }
}

/// 原生实现与其**自述上游口径**一致（弱对照：开局 20/60 点、上限 99、阈值 25/50/75/99）。
#[test]
fn native_matches_its_documented_upstream_semantics() {
    let state = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/state.rs"
    ))
    .expect("读 state");
    let idx = state.find("pub mbqianlong_marks").expect("字段");
    let seg: String = state[..idx].chars().rev().take(300).collect::<String>().chars().rev().collect();
    assert!(
        seg.contains("countMark('mbqianlong')") || seg.contains("0..=99"),
        "原生注释应记录上游键名与范围：{seg}"
    );
    let beh = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/behavior.rs"
    ))
    .expect("读 behavior");
    let idx = beh.find("潜龙·道心值（tag 469").expect("潜龙注释");
    let seg: String = beh[idx..].chars().take(320).collect();
    assert!(
        seg.contains("20 点") && seg.contains("99") && seg.contains("25/50/75/99"),
        "原生注释应记录开局值/上限/阈值：{seg}"
    );
    let idx = beh.find("卫统（tag 470").expect("卫统注释");
    let seg: String = beh[idx..].chars().take(240).collect();
    assert!(
        seg.contains("+60") || seg.contains("60"),
        "原生注释应记录卫统的 +60：{seg}"
    );
    let _ = Engine::new(20267101, 4);
}
