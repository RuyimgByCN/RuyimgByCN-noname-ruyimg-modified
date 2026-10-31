//! UP37-R10 的元数据子集：上游 2026-10-24 范围里的姓名/氏族、技能引用与文案
//! 已随 I01 同步进入 catalog。本测试只守护**数据**，不把元数据通过当作 AI 或
//! 规则行为等价。

const CATALOG: &[u8] = include_bytes!("../../../content/original/catalog.json");

fn value() -> serde_json::Value {
    serde_json::from_slice(CATALOG).expect("catalog JSON")
}

fn character<'a>(catalog: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    catalog["characters"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap_or_else(|| panic!("missing character {name}"))
}

#[test]
fn zhugejun_names_and_clan_survive_sync() {
    let catalog = value();
    for name in ["tw_zhugejun", "dc_zhugejun"] {
        let row = character(&catalog, name);
        assert_eq!(row["names"], "诸葛|均", "{name} names");
        assert_eq!(
            row["clans"],
            serde_json::json!(["琅琊诸葛氏"]),
            "{name} clans"
        );
    }
}

#[test]
fn wufan_skill_references_resolve() {
    let catalog = value();
    let row = character(&catalog, "wufan");
    assert_eq!(row["skills"], serde_json::json!(["tianyun", "wfyuyan"]));
    let resolved: Vec<&str> = row["resolved_skills"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|value| value.as_str())
        .collect();
    assert!(resolved.iter().any(|id| id.ends_with(":tianyun")));
    assert!(resolved.iter().any(|id| id.ends_with(":wfyuyan")));
}

#[test]
fn dangchen_description_uses_final_upstream_wording() {
    let catalog = value();
    let row = catalog["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "stardangchen")
        .expect("stardangchen");
    let description = row["description"].as_str().unwrap();
    assert!(
        description.contains("令一名其他角色交给你任意张牌"),
        "{description}"
    );
    assert!(description.contains("此牌额外结算一次"), "{description}");
}

#[test]
fn rouke_duration_and_yishi_reward_wording_survive_sync() {
    let catalog = value();
    let skill = |name: &str| {
        catalog["skills"]
            .as_array()
            .unwrap()
            .iter()
            .find(|row| row["name"] == name)
            .unwrap_or_else(|| panic!("missing skill {name}"))
    };
    let rouke = skill("twrouke")["description"].as_str().unwrap();
    assert!(rouke.contains("直到下轮游戏开始"), "{rouke}");
    let yishi = skill("spyishi")["description"].as_str().unwrap();
    assert!(yishi.contains("获得其装备区内的一张牌"), "{yishi}");
}

#[test]
fn offline_tundog_skill_name_and_reset_source_survive() {
    let catalog = value();
    let row = catalog["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == "hm_tuntian")
        .expect("hm_tuntian");
    assert_eq!(row["display_name"], "屯犬");
    assert_eq!(
        row["source"],
        "legacy-source/character/character/offline/skill/offline_hanmo.js"
    );
}
