//! NC05/NC09：正式选将的 WASM 导出面（客户端选将 UI 的数据源）。
//!
//! 验收：候选导出含扩展武将；按名选将成功后**技能集在册**（来自武将表而非调试授技）；
//! 越界座位/未知武将**明确报错**（不静默）。

use engine_native_wasm::test_support::{
    assign_general_for, engine_from_packs, general_candidates_json_for,
};

const PACKS: &str = r#"[{"pack":{"name":"nc05-sel","version":1,"description":"nc05","card_prefix":"nc05s_"},"cards":[{"name":"extra","kind":"sha","suit":"spade","rank":7}],"generals":[{"name":"nc05_hero","faction":"shu","hp":4,"gender":"male","skills":[{"name":"kurou","mapping":"host:kurou"}]}]}]"#;

/// 候选导出：含扩展武将且技能列表非空。
#[test]
fn nc05_candidates_export_lists_generals() {
    let engine = engine_from_packs(4, 20261901, PACKS).expect("建局");
    let json = general_candidates_json_for(&engine);
    let value: serde_json::Value = serde_json::from_str(&json).expect("JSON");
    let count = value["count"].as_u64().expect("count");
    assert!(count > 0, "候选非空");
    let has_named = value["candidates"]
        .as_array()
        .expect("candidates")
        .iter()
        .any(|c| c["name"].as_str() == Some("nc05_hero"));
    assert!(has_named, "候选应含扩展武将 nc05_hero：{json}");
}

/// 选将导出：成功后技能集在册；未知武将与越界座位明确报错。
#[test]
fn nc05_assign_general_export_is_authoritative() {
    let mut engine = engine_from_packs(4, 20261902, PACKS).expect("建局");
    // 先确认 kurou 在目录（扩展武将技能集引用它）
    let has_kurou = engine.catalog().skill_by_name("kurou").is_some();
    let out = assign_general_for(&mut engine, 0, "nc05_hero");
    if has_kurou {
        let json = out.expect("选将应成功");
        let value: serde_json::Value = serde_json::from_str(&json).expect("JSON");
        assert_eq!(value["seat"].as_u64(), Some(0));
        // 技能集在册（来自武将表）
        let def = engine.catalog().skill_by_name("kurou").expect("kurou");
        assert!(
            engine.state_core().skills.contains_key(&(0, def)),
            "选将后技能集应在册"
        );
    } else {
        // 目录无该技能时选将应**明确失败**（不静默接受）
        assert!(out.is_err(), "技能缺失时应报错");
    }
    // 未知武将
    assert!(
        assign_general_for(&mut engine, 0, "no-such-general").is_err(),
        "未知武将必须报错"
    );
    // 越界座位
    assert!(
        assign_general_for(&mut engine, 99, "nc05_hero").is_err(),
        "越界座位必须报错"
    );
}
