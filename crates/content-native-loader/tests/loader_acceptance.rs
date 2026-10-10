//! C0 装载器验收（ADR 0777、ADR 0778）：
//! 1. 包数据自描述：包元数据、牌种键、射程/装备技、牌种伴随技能与武将
//!    技能均由 cards.json 与 engine-native 全局表解析，装载调用不带逐包参数。
//! 2. 名称所有权：每轮重新装载牌与武将、建局、启停后，engine-native
//!    符号表（名称与名单）不再增长。
//! 3. 错误面：坏 JSON、缺字段、非法取值、未知键、技能重名/未登记、宿主
//!    API 版本不符均给出明确错误。
//!
//! 符号表计数为进程级共享读数，故全部断言合并在单个测试内顺序执行，
//! 避免并行测试 intern 造成的误报。

use content_native_loader::{bridge_generals, bridge_pack, ensure_host_api_compatible, BridgeError};
use engine_native::catalog::{CardKind, Faction};
use engine_native::content::extension::{ExtensionPack, ExtensionRegistry, InitHook};
use engine_native::content::host::{HostBehavior, GJ_JINLIANZHU_KEY, GW_QINPEN_STATE_KEY};
use engine_native::content::names;
use engine_native::content::standard::Gender;
use engine_native::Engine;
use serde_json::Value;

const DEMO_JSON: &str = r#"{
  "pack": { "name": "demo-pack", "version": 1, "description": "装载器验收样例", "card_prefix": "dm_" },
  "cards": [
    { "name": "sha", "kind": "sha", "suit": "spade", "rank": 7 },
    { "name": "jinlianzhu", "kind": "jinlianzhu", "suit": "heart", "rank": 9 },
    { "name": "qinpen", "kind": "qinpendayu", "suit": "club", "rank": 3 },
    { "name": "blade", "kind": "weapon", "suit": "diamond", "rank": 12, "range": 3, "equip_skills": ["dm_blade"] },
    { "name": "jinlianzhu", "kind": "jinlianzhu", "suit": "spade", "rank": 2 }
  ],
  "skills": [
    { "name": "dm_draw", "mapping": "host:jihuocard_draw" },
    { "name": "dm_blade", "mapping": "host:sanjian_weapon" },
    { "name": "dm_pending", "mapping": null }
  ],
  "generals": [
    { "name": "dm_hero", "faction": "shu", "gender": "female", "hp": 3,
      "skills": [ { "name": "dm_draw", "mapping": "host:jihuocard_draw" }, { "name": "dm_later" } ] }
  ],
  "deferred": [ { "name": "调兵遣将" } ]
}"#;

/// 牌 + 武将合成一个扩展包（武将技能与牌侧同名同行为的声明去重）。
fn load_demo() -> ExtensionPack {
    let mut pack = bridge_pack(DEMO_JSON).expect("装载牌").pack;
    let gens = bridge_generals(DEMO_JSON).expect("装载武将");
    pack.generals = gens.generals;
    for d in gens.host_skills {
        if !pack.host_skills.iter().any(|x| x.name == d.name) {
            pack.host_skills.push(d);
        }
    }
    pack
}

fn with_pack(body: &str) -> String {
    format!(r#"{{ "pack": {{ "name": "err-pack", "version": 1, "card_prefix": "er_" }}, {body} }}"#)
}

fn card(extra: &str) -> String {
    with_pack(&format!(r#""cards": [ {{ "name": "c", "suit": "spade", "rank": 1, {extra} }} ]"#))
}

fn generals(g: &str) -> String {
    format!(r#"{{ "generals": [ {{ "name": "g", "skills": [], {g} }} ] }}"#)
}

fn invalid_field(r: Result<impl Sized, BridgeError>) -> &'static str {
    match r {
        Err(BridgeError::InvalidField { field, .. }) => field,
        Err(other) => panic!("期望 InvalidField，实为 {other:?}"),
        Ok(_) => panic!("期望 InvalidField，实为装载成功"),
    }
}

fn err<T>(r: Result<T, BridgeError>) -> BridgeError {
    match r {
        Err(e) => e,
        Ok(_) => panic!("坏包应被拒绝"),
    }
}

#[test]
fn loader_acceptance_bounded_names_and_clear_errors() {
    // —— 包数据自描述：元数据 / 牌种键 / 射程与装备技 / 伴随技能 / 武将 ——
    let out = bridge_pack(DEMO_JSON).expect("首轮装载");
    let pack = &out.pack;
    assert_eq!(
        (pack.def.name, pack.def.version, pack.def.description),
        ("demo-pack", 1, "装载器验收样例")
    );
    assert_eq!(pack.init_hook, InitHook::None);
    let card_names: Vec<&str> = pack.cards.iter().map(|c| c.name).collect();
    assert_eq!(
        card_names,
        ["dm_sha_0", "dm_jinlianzhu_1", "dm_qinpen_2", "dm_blade_3", "dm_jinlianzhu_4"]
    );
    let blade = &pack.cards[3];
    assert_eq!((blade.kind, blade.range), (CardKind::Weapon, 3), "range → 通用武器");
    assert_eq!(blade.equip_skills, ["dm_blade"]);
    let skills: Vec<(&str, HostBehavior)> =
        pack.host_skills.iter().map(|d| (d.name, d.behavior)).collect();
    assert_eq!(
        skills,
        [
            (GJ_JINLIANZHU_KEY, HostBehavior::JinlianzhuCancel),
            (GW_QINPEN_STATE_KEY, HostBehavior::GwentQinpenState),
            ("dm_draw", HostBehavior::JihuocardDraw),
            ("dm_blade", HostBehavior::SanjianWeaponDamage),
        ],
        "伴随技能按牌首次出现登记且去重，再接 skills 映射；mapping:null 不登记"
    );
    assert_eq!(out.deferred, vec!["调兵遣将".to_string()], "暂缓清单透传");

    let gens = bridge_generals(DEMO_JSON).expect("装载武将");
    assert_eq!(gens.generals.len(), 1);
    let hero = &gens.generals[0];
    assert_eq!(
        (hero.name, hero.faction, hero.gender, hero.max_hp, hero.skills),
        ("dm_hero", Faction::Shu, Gender::Female, 3, &["dm_draw"][..])
    );
    assert_eq!(gens.mapped_skill_keys, ["dm_draw"]);
    assert_eq!(gens.unmapped_skills, 1);
    assert_eq!(gens.host_skills.len(), 1);

    let hooked = with_pack(r#""cards": []"#).replace(
        r#""card_prefix": "er_""#,
        r#""card_prefix": "er_", "init_hook": "wuxing_natures""#,
    );
    assert_eq!(bridge_pack(&hooked).expect("开局钩子").pack.init_hook, InitHook::WuxingNatures);

    // —— 错误面：坏包 / 非法取值 / 未知键 / 版本不符均给明确原因 ——
    assert!(matches!(err(bridge_pack("{ not json")), BridgeError::BadJson(_)));
    assert_eq!(err(bridge_pack("{}")), BridgeError::MissingField("pack"));
    // cards 可选（纯武将包合法）：缺省装载为空卡牌目录，deferred 仍透传。
    let cards_only = bridge_pack(&with_pack(r#""deferred": [ { "name": "d1" } ]"#)).expect("纯武将包合法");
    assert!(cards_only.pack.cards.is_empty());
    assert!(cards_only.pack.generals.is_empty());
    assert_eq!(cards_only.deferred, ["d1".to_string()]);
    assert_eq!(
        invalid_field(bridge_pack(r#"{ "pack": { "name": "", "version": 1, "card_prefix": "x_" }, "cards": [] }"#)),
        "pack.name"
    );
    assert_eq!(
        invalid_field(bridge_pack(r#"{ "pack": { "name": "p", "version": 0, "card_prefix": "x_" }, "cards": [] }"#)),
        "pack.version"
    );
    assert_eq!(
        err(bridge_pack(r#"{ "pack": { "name": "p", "version": 1 }, "cards": [] }"#)),
        BridgeError::MissingField("pack.card_prefix")
    );
    assert_eq!(
        err(bridge_pack(&with_pack(r#""cards": []"#).replace(
            r#""card_prefix": "er_""#,
            r#""card_prefix": "er_", "init_hook": "no_such""#
        ))),
        BridgeError::UnknownInitHook("no_such".into())
    );
    assert_eq!(
        err(bridge_pack(&card(r#""kind": "no_such_kind""#))),
        BridgeError::UnknownCardKind("no_such_kind".into())
    );
    assert_eq!(
        err(bridge_pack(&card(r#""kind": "leishajian""#))),
        BridgeError::UnknownCardKind("leishajian".into()),
        "牌种键只认引擎规范键（雷杀为 leisha）"
    );
    assert_eq!(
        err(bridge_pack(&with_pack(
            r#""cards": [ { "name": "c", "kind": "sha", "suit": "star", "rank": 1 } ]"#
        ))),
        BridgeError::UnknownSuit("star".into())
    );
    for rank in ["0", "14", r#""7""#] {
        let json = with_pack(&format!(
            r#""cards": [ {{ "name": "c", "kind": "sha", "suit": "spade", "rank": {rank} }} ]"#
        ));
        assert_eq!(invalid_field(bridge_pack(&json)), "cards[].rank", "rank={rank}");
    }
    assert_eq!(
        invalid_field(bridge_pack(&card(r#""kind": "sha", "range": 2"#))),
        "cards[].range",
        "非武器栏牌种不能声明射程"
    );
    assert_eq!(
        err(bridge_pack(&card(r#""kind": "weapon", "range": 2, "equip_skills": ["no_such_skill"]"#))),
        BridgeError::UnknownEquipSkill("no_such_skill".into())
    );
    assert_eq!(
        err(bridge_pack(&with_pack(
            r#""cards": [], "skills": [ { "name": "a", "mapping": "host:jihuocard_draw" }, { "name": "a", "mapping": "host:jihuocard_draw" } ]"#
        ))),
        BridgeError::DuplicateSkill("a".into())
    );
    assert_eq!(
        err(bridge_pack(&with_pack(&format!(
            r#""cards": [ {{ "name": "c", "kind": "jinlianzhu", "suit": "spade", "rank": 1 }} ], "skills": [ {{ "name": "{GJ_JINLIANZHU_KEY}", "mapping": "host:jihuocard_draw" }} ]"#
        )))),
        BridgeError::DuplicateSkill(GJ_JINLIANZHU_KEY.into()),
        "包技能不得与牌种伴随技能重名"
    );
    assert_eq!(
        err(bridge_pack(&with_pack(r#""cards": [], "skills": [ { "name": "x", "mapping": "host:no_such" } ]"#))),
        BridgeError::UnknownBehaviorKey("host:no_such".into())
    );
    assert_eq!(
        invalid_field(bridge_pack(&with_pack(r#""cards": [], "skills": [ { "name": "x", "mapping": 5 } ]"#))),
        "skills[].mapping"
    );

    let current = content_runtime::tooling_host_versions().engine_api;
    let value = |s: &str| serde_json::from_str::<Value>(s).unwrap();
    assert!(ensure_host_api_compatible(&value(&format!(r#"{{ "host_api": {current} }}"#))).is_ok());
    assert!(ensure_host_api_compatible(&value("{}")).is_ok(), "未声明 host_api 视为兼容");
    assert_eq!(
        ensure_host_api_compatible(&value(r#"{ "host_api": 999 }"#)),
        Err(BridgeError::HostVersionMismatch { declared: 999, current })
    );
    for bad in [r#""1""#, "-1", "1.5"] {
        assert_eq!(
            invalid_field(ensure_host_api_compatible(&value(&format!(r#"{{ "host_api": {bad} }}"#)))),
            "host_api",
            "host_api={bad}"
        );
    }
    assert_eq!(
        err(bridge_pack(&with_pack(r#""host_api": 999, "cards": []"#))),
        BridgeError::HostVersionMismatch { declared: 999, current },
        "装载入口同样校验宿主 API"
    );

    assert_eq!(err(bridge_generals("{}")), BridgeError::MissingField("generals"));
    assert_eq!(
        err(bridge_generals(&generals(r#""gender": "male", "hp": 4"#))),
        BridgeError::MissingField("generals[].faction"),
        "势力缺失不再默认为群"
    );
    assert_eq!(
        err(bridge_generals(&generals(r#""faction": "jin", "gender": "male", "hp": 4"#))),
        BridgeError::UnknownFaction("jin".into())
    );
    assert_eq!(
        err(bridge_generals(&generals(r#""faction": "wei", "gender": "none", "hp": 4"#))),
        BridgeError::UnknownGender("none".into())
    );
    assert_eq!(
        invalid_field(bridge_generals(&generals(r#""faction": "wei", "gender": "male", "hp": 0"#))),
        "generals[].hp"
    );
    assert_eq!(
        err(bridge_generals(
            r#"{ "generals": [
                { "name": "a", "faction": "wei", "gender": "male", "hp": 4, "skills": [ { "name": "s", "mapping": "host:jihuocard_draw" } ] },
                { "name": "b", "faction": "wu", "gender": "male", "hp": 4, "skills": [ { "name": "s", "mapping": "host:siji_discard_draw" } ] }
            ] }"#
        )),
        BridgeError::DuplicateSkill("s".into()),
        "同名技能不得映射到不同宿主行为"
    );

    // —— 名称所有权：每轮重新装载 + 建局 + 启停后符号表增长为 0 ——
    // 预热：首轮装载 + 建局，把本样例全部名称与名单 intern 进符号表。
    let warm = load_demo();
    let _e = Engine::new_with_extensions(4101, 3, &[&warm]).expect("预热建局");
    let before = names::interned_count();
    assert!(before > 0, "预热后符号表应非空");
    let mut registry = ExtensionRegistry::new();
    registry.install(warm.def.clone()).expect("注册安装");
    for i in 0..50u64 {
        let pack = load_demo();
        // 计数只观测符号表；指针同一性另证名称与名单未绕过符号表另行分配。
        assert!(
            std::ptr::eq(pack.generals[0].skills, warm.generals[0].skills),
            "武将技能名单须复用符号表存储"
        );
        assert!(
            pack.cards.iter().zip(&warm.cards).all(|(a, b)| std::ptr::eq(a.name, b.name)),
            "牌名须复用符号表存储"
        );
        let _e = Engine::new_with_extensions(4102 + i, 3, &[&pack]).expect("循环建局");
        registry.disable(pack.def.name).expect("禁用");
        registry.enable(pack.def.name).expect("启用");
    }
    assert_eq!(
        names::interned_count(),
        before,
        "50 轮重新装载/建局/启停后符号表仍在增长（名称泄漏未收敛）"
    );
}
