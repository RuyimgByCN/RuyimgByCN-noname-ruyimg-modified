//! T9 / D5：Boss 诸葛亮与 ylyg* 不能装成“已支持的空操作”。
//!
//! 上游体力是 Infinity，四技与 ylyg* 没有宿主行为键。装载器必须拒绝
//! 非整数体力，并把 mapping 缺省的技能计为暂缓，而不是写进武将技能表。

use content_native_loader::{bridge_generals, BridgeError};

fn general(hp: &str, skills: &str) -> String {
    format!(
        r#"{{
  "generals": [{{
    "name": "boss_zhugeliang",
    "faction": "shu",
    "gender": "male",
    "hp": {hp},
    "skills": [{skills}]
  }}]
}}"#
    )
}

fn skill(name: &str, mapping: &str) -> String {
    format!(r#"{{"name":"{name}","mapping":{mapping}}}"#)
}

#[test]
fn t9_infinity_hp_is_rejected() {
    for hp in [r#""Infinity""#, "null", "0", "0.5", "256"] {
        let json = general(hp, &skill("xiangxing", "null"));
        let err = match bridge_generals(&json) {
            Ok(_) => panic!("{hp} 不应被接受"),
            Err(e) => e,
        };
        match err {
            BridgeError::InvalidField { field, .. } | BridgeError::MissingField(field) => {
                assert!(
                    field.contains("hp"),
                    "{hp} 应落在体力字段，实际 {err}"
                );
            }
            other => panic!("{hp} 不应被接受: {other}"),
        }
    }
}

#[test]
fn t9_zhuge_and_ylyg_skills_stay_unmapped() {
    let skills = [
        "xiangxing",
        "yueyin",
        "fengqi",
        "gaiming",
        "ylygxiangxing",
        "ylyglianhuan",
        "ylygniepan",
        "ylyg_jiang",
        "ylygguhuo",
        "ylygtianxiang",
    ]
    .into_iter()
    .map(|name| skill(name, "null"))
    .collect::<Vec<_>>()
    .join(",");
    let bridged = bridge_generals(&general("4", &skills)).expect("整数体力可装载");
    assert!(bridged.host_skills.is_empty(), "暂缓技能不得变成宿主行为");
    assert_eq!(bridged.unmapped_skills, 10);
    assert!(bridged.generals[0].skills.is_empty(), "暂缓技能不得写入武将技能表");
    assert_eq!(bridged.generals[0].max_hp, 4);
}

#[test]
fn t9_unknown_mapping_is_not_a_noop() {
    let json = general("4", &skill("xiangxing", r#""xiangxing""#));
    let err = match bridge_generals(&json) {
        Ok(_) => panic!("未知行为键不应被接受"),
        Err(e) => e,
    };
    assert!(
        matches!(err, BridgeError::UnknownBehaviorKey(ref k) if k == "xiangxing"),
        "{err}"
    );
}
