//! NC08 第⑦层（上游行为对照）样本 79：**来源 U05（杀海拾遗·运筹）** ——
//! **三项未实现技能登记**（`diyjianxiong` 奸雄变体 / `xinfu_jingxie` 精械 / `xinfu_qiaosi` 巧思）。
//!
//! 运筹包共 **10 名武将 / 21 个技能**（`character.json`）。经**全库检索**
//! （`grep -rli <key> crates/engine-native/src/`）：
//!
//! | 上游键 | 译名 | 原生 |
//! |---|---|---|
//! | `diyjianxiong` | 奸雄·变体 | **0 命中**（未实现） |
//! | `xinfu_jingxie` | 精械 | **0 命中**（未实现；上游定义在 `character/mobile/skill.js`） |
//! | `xinfu_qiaosi` | 巧思 | **0 命中**（未实现；上游定义在运筹 `skill.js:489`，**骰子机制**） |
//!
//! 上游依据（逐字引用要点）：
//! - `diyjianxiong`：`mode: ["identity","guozhan"]` + `trigger: { global: "dieBefore" }`
//!   + filter（当前回合角色为持技者；身份局要求死者**非主公**）+ content 广播改身份并亮明；
//! - `xinfu_qiaosi`：`translate.js` 描述「出牌阶段限一次，你可以**投掷一枚六面骰子**，亮出牌堆顶的
//!   X 张牌并获得之。然后选择一项：1.交给一名其他角色 X 张牌。2.弃置 X 张牌。（X 为骰子点数）」；
//! - `xinfu_jingxie`：运筹 `character.js:71` 把马钧技能列为 `["xinfu_jingxie", "xinfu_qiaosi"]`。
//!
//! **未覆盖（登记）**：三项技能的**行为面**（未实现 → 无从验收）；
//! `xinfu_qiaosi` 依赖的**骰子机制**是否存在于原生（**建议单独核对**）。

use std::path::{Path, PathBuf};

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(p: &Path) -> String {
    std::fs::read_to_string(p).unwrap_or_else(|e| panic!("读 {} 失败：{e}", p.display()))
}

/// 上游侧：三项技能的定义/引用**确实存在**（避免「登记了不存在的东西」）。
#[test]
fn upstream_defines_the_three_skills() {
    let up = manifest()
        .join("../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou");
    let skill = read(&up.join("skill.js"));
    assert!(
        skill.contains("\tdiyjianxiong: {"),
        "上游应有 diyjianxiong 定义"
    );
    assert!(
        skill.contains("\txinfu_qiaosi: {"),
        "上游应有 xinfu_qiaosi 定义（skill.js:489）"
    );
    assert!(
        skill.contains("dieBefore") && skill.contains("mode: [\"identity\", \"guozhan\"]"),
        "上游依据：diyjianxiong 为身份/国战模式的 dieBefore 触发"
    );
    let chars = read(&up.join("character.js"));
    assert!(
        chars.contains("\"xinfu_jingxie\"") && chars.contains("\"xinfu_qiaosi\""),
        "上游依据：马钧技能列为 [xinfu_jingxie, xinfu_qiaosi]"
    );
    let tr = read(&up.join("translate.js"));
    assert!(
        tr.contains("投掷一枚六面骰子"),
        "上游依据：巧思含**骰子**机制（translate.js）"
    );
}

/// 原生侧：三项技能**均无实现**（固定「未实现」事实，便于后续修复后翻转该断言）。
#[test]
fn native_lacks_the_three_skills() {
    let src = manifest().join("../engine-native-kernel/src");
    let behavior = read(&src.join("vocabulary/host/behavior.rs"));
    let keys = read(&src.join("vocabulary/host/keys.rs"));
    for (key, needle) in [
        ("diyjianxiong", "Diyjianxiong"),
        ("xinfu_jingxie", "Jingxie"),
        ("xinfu_qiaosi", "Qiaosi"),
    ] {
        assert!(
            !behavior.contains(needle),
            "登记：原生 `behavior.rs` **未见** {key}（{needle}）——实现后请翻转本断言"
        );
        assert!(
            !keys.contains(needle),
            "登记：原生 `keys.rs` **未见** {key}（{needle}）"
        );
    }
}
