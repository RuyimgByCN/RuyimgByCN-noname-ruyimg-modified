//! NC08 附加：**`cards[].skills`（牌的宿主技能声明）装载支持**（第 53 项的**声明侧**）。
//!
//! 背景：第 53 项（趁火打劫）的根因是「非装备牌的 `global` 子技能**没有任何声明通道**」——
//! `bridge_pack` 此前只支持 `cards[].equip_skills`，不支持 `cards[].skills`。
//!
//! 本轮在**已跟踪**的 `content-native-loader` 中补上该字段（并入 `pack.host_skills`，由目录注册），
//! 并在此验证：
//! 1. **正向**：声明 `host:chenhuodajie_card` ⇒ 该键出现在 `pack.host_skills`，且行为解析正确；
//! 2. **去重**：同一键在两张牌上重复声明 ⇒ 只登记一次；
//! 3. **负向**：未知宿主键 ⇒ 桥接**报错**（不静默忽略）。
//!
//! **边界（务必保留）**：本用例只验证**声明**。技能进入目录后**仍需授予**才有实例，
//! 授予侧（`kernel/equipment.rs`、`construction.rs`）**未跟踪** ⇒ 第 53 项**未解除阻塞**。

use engine_native::content::host::HostBehavior;

fn json_with(cards: &str) -> String {
    format!(
        r#"{{
  "pack": {{ "name": "loader-card-skills", "version": 1, "card_prefix": "lcs_" }},
  "cards": [ {cards} ]
}}"#
    )
}

const ONE_CARD: &str = r#"{ "name": "chenhuodajie", "kind": "chenhuodajie",
    "suit": "diamond", "rank": 6, "skills": ["host:chenhuodajie_card"] }"#;

/// **正向**：`cards[].skills` 声明的宿主键进入 `pack.host_skills`。
#[test]
fn card_skills_declare_host_skill() {
    let out = content_native_loader::bridge_pack(&json_with(ONE_CARD)).expect("桥接");
    let names: Vec<String> = out.pack.host_skills.iter().map(|d| d.name.to_string()).collect();
    eprintln!("DBG cards[].skills：host_skills = {names:?}");
    assert!(
        names.iter().any(|n| n == "host:chenhuodajie_card"),
        "声明的宿主键应进入 pack.host_skills（实得 {names:?}）"
    );
    let decl = out
        .pack
        .host_skills
        .iter()
        .find(|d| d.name == "host:chenhuodajie_card")
        .expect("有该声明");
    assert_eq!(
        decl.behavior,
        HostBehavior::ChenhuodajieCard,
        "行为应按键解析正确"
    );
}

/// **去重**：同一键在两张牌上重复声明 ⇒ 只登记一次。
#[test]
fn card_skills_deduplicated() {
    let two = format!("{ONE_CARD}, {}", ONE_CARD.replace("\"rank\": 6", "\"rank\": 9"));
    let out = content_native_loader::bridge_pack(&json_with(&two)).expect("桥接");
    let n = out
        .pack
        .host_skills
        .iter()
        .filter(|d| d.name == "host:chenhuodajie_card")
        .count();
    eprintln!("DBG cards[].skills 去重：同名声明出现 {n} 次（应为 1）");
    assert_eq!(n, 1, "同一宿主键应只登记一次");
}

/// **负向**：未知宿主键 ⇒ 桥接报错（不静默忽略）。
#[test]
fn unknown_card_skill_key_rejected() {
    let bad = r#"{ "name": "x", "kind": "sha", "suit": "spade", "rank": 1,
        "skills": ["host:no_such_behavior"] }"#;
    let err = content_native_loader::bridge_pack(&json_with(bad));
    eprintln!(
        "DBG cards[].skills 负例：is_err={}；错误={:?}",
        err.is_err(),
        err.as_ref().err()
    );
    assert!(err.is_err(), "未知宿主键应被拒绝");
}

/// **兼容性**：真实包（swd）仍正常桥接，且未因新字段产生重复声明。
#[test]
fn real_pack_still_bridges() {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let out = content_native_loader::bridge_pack(&cards).expect("swd 桥接");
    let mut names: Vec<String> = out.pack.host_skills.iter().map(|d| d.name.to_string()).collect();
    let before = names.len();
    names.sort();
    names.dedup();
    eprintln!("DBG swd：host_skills {before} 项、去重后 {}", names.len());
    assert_eq!(before, names.len(), "swd 的 host_skills 不应有重名");
}
