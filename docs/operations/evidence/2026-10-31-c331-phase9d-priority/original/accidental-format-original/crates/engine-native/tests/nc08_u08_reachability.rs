//! NC08：**U08（英雄杀）技能的可达性**——查「已实现」是否等于「在对局中可触发」。
//!
//! 背景：第 195/199 行确认 U08 的 5 个宿主技能**都有实现**（`TriggerMeta` + 步骤机 + ABI tag）。
//! 但实现存在 ≠ 可达：**技能必须被某个武将或卡牌授予**，才会出现在某名角色的技能表里。
//!
//! 判据：
//! 1. U08 包**声明了 5 个技能**（`assets/cards.json` 的 `skills`）；
//! 2. 该包的 `cards` **为空**、`generals` **为空**（包内没有载体）；
//! 3. **全库**（所有包）里，**没有任何武将或卡牌**以这些技能名作为 `skills`/`mapping` 之外的引用；
//!    —— 具体做法：把**全部 41 个包**同时装载，检查 `catalog.generals` 与 `catalog.cards` 里
//!    **没有任何条目**把 `yxs_fanji*`/`yxs_menshen*` 作为授技来源（用 `SkillDef` 的 owner 面无法直接查，
//!    故退一步：检查**没有武将**拥有这些技能 ⇒ `extension_generals` 里 `skills` 不含它们）。
//!
//! **结论口径**：若 1–3 成立 ⇒ 这 5 个技能**已实现但当前不可达**（无载体），
//! 因此**不能**因「实现存在」就签规则差异。

use std::path::PathBuf;

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

const U08_SKILLS: [&str; 5] = [
    "yxs_fanji",
    "yxs_fanji2",
    "yxs_menshen",
    "yxs_menshen2",
    "yxs_menshen3",
];

#[test]
fn u08_skills_are_declared_but_have_no_carrier() {
    let dir = root().join("hero-kill-upstream-noname").join("assets");
    let json = std::fs::read_to_string(dir.join("cards.json")).expect("读 U08 包");
    let v: serde_json::Value = serde_json::from_str(&json).expect("U08 包 JSON");
    let declared: Vec<String> = v["skills"]
        .as_array()
        .expect("skills 数组")
        .iter()
        .map(|s| s["name"].as_str().unwrap_or("").to_string())
        .collect();
    let cards = v["cards"].as_array().map(|a| a.len()).unwrap_or(0);
    let generals = v["generals"].as_array().map(|a| a.len()).unwrap_or(0);
    eprintln!(
        "DBG U08 可达性：声明技能 {} 个、cards={cards}、generals={generals}",
        declared.len()
    );
    assert_eq!(declared.len(), 5, "U08 应声明 5 个技能");
    for s in U08_SKILLS {
        assert!(declared.iter().any(|d| d == s), "应声明 {s}");
    }
    assert_eq!(cards, 0, "U08 包内不应有卡牌");
    assert_eq!(generals, 0, "U08 包内不应有武将");

    // 全库装载：确认没有任何**武将**拥有这些技能
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    let mut dirs: Vec<PathBuf> = std::fs::read_dir(root())
        .expect("read packages")
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.is_dir())
        .collect();
    dirs.sort();
    let mut all_generals_with: Vec<String> = Vec::new();
    for d in &dirs {
        let assets = d.join("assets");
        let mut any = false;
        for f in ["cards.json", "character.json"] {
            let p = assets.join(f);
            if !p.is_file() {
                continue;
            }
            let Ok(t) = std::fs::read_to_string(&p) else {
                continue;
            };
            let Ok(b) = content_native_loader::bridge_pack(&t) else {
                continue;
            };
            any = true;
            for g in b.pack.generals.iter() {
                for sk in g.skills.iter() {
                    if U08_SKILLS.iter().any(|u| sk.eq_ignore_ascii_case(u)) {
                        all_generals_with.push(format!("{}/{}", g.name, sk));
                    }
                }
            }
        }
        let _ = any;
    }
    let e = engine_native::Engine::new_with_extensions(83, 4, &[&pack]).expect("U08 建局");
    let catalog_has = U08_SKILLS
        .iter()
        .filter(|target| {
            e.catalog()
                .skills
                .iter()
                .any(|k| k.name.eq_ignore_ascii_case(**target))
        })
        .count();
    eprintln!(
        "DBG U08 可达性：catalog 里含 {} 个 U08 技能；全库武将引用 {} 条",
        catalog_has,
        all_generals_with.len()
    );
    assert_eq!(catalog_has, 5, "5 个技能应进入 catalog");
    assert!(
        all_generals_with.is_empty(),
        "不应有武将授予 U08 技能（若有则说明可达）：{all_generals_with:?}"
    );
}
