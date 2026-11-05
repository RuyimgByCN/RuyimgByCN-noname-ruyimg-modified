//! UP37-R10 与 UP37-N01 的元数据/显示/AI 估值配置子集：
//! 上游 2026-10-24 范围（c2c9ae0b、673bc921、90b58bb9、a44934c5、2492902c、
//! 19d86519、c8d29532）里的姓名/氏族、技能引用、提示文案、子卡译名、manualConfirm、
//! poptip、动态伤害层数显示与有限 AI 估值常数均已随 I01 同步入库。

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

fn skill<'a>(catalog: &'a serde_json::Value, name: &str) -> &'a serde_json::Value {
    catalog["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["name"] == name)
        .unwrap_or_else(|| panic!("missing skill {name}"))
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
    let row = skill(&catalog, "stardangchen");
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
    let rouke = skill(&catalog, "twrouke")["description"].as_str().unwrap();
    assert!(rouke.contains("直到下轮游戏开始"), "{rouke}");
    let yishi = skill(&catalog, "spyishi")["description"].as_str().unwrap();
    assert!(yishi.contains("获得其装备区内的一张牌"), "{yishi}");
}

#[test]
fn offline_tundog_skill_name_and_reset_source_survive() {
    let catalog = value();
    let row = skill(&catalog, "hm_tuntian");
    assert_eq!(row["display_name"], "屯犬");
    assert_eq!(
        row["source"],
        "legacy-source/character/character/offline/skill/offline_hanmo.js"
    );
}

#[test]
fn r10_characters_wordings_and_n01_roster_entries_survive_sync() {
    let catalog = value();

    // 2492902c: hm_yudu & xia_shie display names, portraits, and skills
    let yudu = character(&catalog, "hm_yudu");
    assert_eq!(yudu["display_name"], "汉末于毒");
    assert_eq!(yudu["portrait"], "assets/portraits/yudu.jpg");
    assert_eq!(yudu["skills"], serde_json::json!(["hm_dafu", "hm_jipin"]));

    let shie = character(&catalog, "xia_shie");
    assert_eq!(shie["display_name"], "侠史阿");
    assert_eq!(shie["portrait"], "assets/portraits/xia_shie.jpg");
    assert_eq!(
        shie["skills"],
        serde_json::json!(["twdengjian", "twxinshou"])
    );

    // 2492902c: peculv wording ("回复1点体力并摸两张牌")
    let peculv = skill(&catalog, "peculv")["description"].as_str().unwrap();
    assert!(peculv.contains("回复1点体力并摸两张牌"), "{peculv}");

    // 673bc921: jsrgzhimeng half-width X ("准备阶段，你可以亮出牌堆顶的X张牌（X为存活角色数）")
    let zhimeng = skill(&catalog, "jsrgzhimeng")["description"]
        .as_str()
        .unwrap();
    assert!(
        zhimeng.contains("亮出牌堆顶的X张牌（X为存活角色数）"),
        "{zhimeng}"
    );
    assert!(!zhimeng.contains('Ｘ'), "must not contain full-width X");

    // c8d29532 (UP37-N01): ol_jie_zhuran & ol_zhuhuan catalog entries & skills
    let zhuran = character(&catalog, "ol_jie_zhuran");
    assert_eq!(zhuran["display_name"], "OL界朱然");
    assert_eq!(zhuran["group"], "wu");
    assert_eq!(zhuran["hp"], 4);
    assert_eq!(zhuran["skills"], serde_json::json!(["ol_jie_danshou"]));

    let zhuhuan = character(&catalog, "ol_zhuhuan");
    assert_eq!(zhuhuan["display_name"], "OL界朱桓");
    assert_eq!(zhuhuan["group"], "wu");
    assert_eq!(zhuhuan["hp"], 4);
    assert_eq!(
        zhuhuan["skills"],
        serde_json::json!(["ol_fenli", "ol_pingkou"])
    );
}

#[test]
fn r10_and_n01_legacy_source_display_card_and_ai_finite_hunks_survive() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../content/original/legacy-source/character");

    // c2c9ae0b & a44934c5: twfushu dynamic damage layer `+#` and twyingjin prompt2
    let tw_skill = std::fs::read_to_string(root.join("tw/skill.js")).unwrap();
    assert!(tw_skill.contains(r#"intro: { content: "下次受到的伤害+#" }"#));
    assert!(tw_skill.contains("对自己造成1点伤害并摸已损失体力值+1张牌"));

    // 90b58bb9: dchuashang sub-card translations and original card.type restoration after equip AI init
    let xianding_card = std::fs::read_to_string(root.join("xianding/card.js")).unwrap();
    for sub in ["dchuashang_equip", "dchuashang_trick", "dchuashang_basic"] {
        assert!(xianding_card.contains(sub), "missing {sub} in card.js");
    }
    let xianding_translate = std::fs::read_to_string(root.join("xianding/translate.js")).unwrap();
    assert!(xianding_translate.contains(r#"dchuashang_equip: "华裳·装备""#));
    assert!(xianding_translate.contains(r#"dchuashang_trick: "华裳·锦囊""#));
    assert!(xianding_translate.contains(r#"dchuashang_basic: "华裳·基本""#));
    assert!(xianding_translate.contains(r#"get.poptip("refenyin")"#));

    let xianding_skill = std::fs::read_to_string(root.join("xianding/skill.js")).unwrap();
    assert!(xianding_skill.contains(r#"lib.card[cardId].type = "equip";"#));
    assert!(xianding_skill.contains(r#"game.finishCard(cardId);"#));
    assert!(xianding_skill.contains(r#"lib.card[cardId].type = type;"#));

    // 2492902c & a44934c5: manualConfirm on mbshanjia / olshanjia, hm_tuntian poptip
    let mobile_skill = std::fs::read_to_string(root.join("mobile/skill.js")).unwrap();
    let mb_pos = mobile_skill.find("mbshanjia:").expect("mbshanjia");
    assert!(mobile_skill[mb_pos..mb_pos + 320].contains("manualConfirm: true"));

    let sp_skill = std::fs::read_to_string(root.join("sp/skill.js")).unwrap();
    let ol_pos = sp_skill.find("olshanjia:").expect("olshanjia");
    assert!(sp_skill[ol_pos..ol_pos + 320].contains("manualConfirm: true"));

    let offline_translate = std::fs::read_to_string(root.join("offline/translate.js")).unwrap();
    assert!(offline_translate.contains(r#"get.poptip("hm_qianjun")"#));

    // 19d86519: offline_yongjian.js replaces unbounded Infinity AI values with finite numbers
    let yongjian =
        std::fs::read_to_string(root.join("offline/skill/offline_yongjian.js")).unwrap();
    assert!(
        !yongjian.contains("Infinity"),
        "offline_yongjian.js must not use unbounded Infinity AI values"
    );

    // c8d29532: onlyOL titles, translations, sort order, and replace groups
    let onlyol_titles = std::fs::read_to_string(root.join("onlyOL/characterTitle.js")).unwrap();
    assert!(onlyol_titles.contains(r#"ol_jie_zhuran: "不动之督""#));
    assert!(onlyol_titles.contains(r#"ol_zhuhuan: "中洲拒天人""#));
    let replace_js = std::fs::read_to_string(root.join("replace.js")).unwrap();
    assert!(replace_js.contains(r#""ol_zhuhuan""#));
    assert!(replace_js.contains(r#""ol_jie_zhuran""#));
}
