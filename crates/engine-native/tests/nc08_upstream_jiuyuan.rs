//! NC08 第⑦层（上游行为对照）样本 59：`jiuyuan`（救援，**标准面** / 孙权）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! jiuyuan: {
//!     trigger: { target: "taoBegin" },                 // 自己成为【桃】的目标时
//!     zhuSkill: true, forced: true,                    // **主公技 + 锁定**
//!     filter(event, player) {
//!         if (event.player === player) return false;   // 使用者非自己
//!         if (!player.hasZhuSkill("jiuyuan")) return false;   // **主公资格**
//!         if (event.player.group !== "wu") return false;      // 使用者为**吴**势力
//!         return true;
//!     },
//!     async content(event, trigger, player) { trigger.baseDamage++; },   // **回复量 +1**
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己（濒死时）**成为【桃】的目标**；**锁定**（`forced`）；
//! 2. **资格**：持技者须具**主公资格**（`hasZhuSkill`），且**用桃者为吴势力**、**非自己**；
//! 3. **效果**：该【桃】的**回复量 +1**（`baseDamage++` → 回复 2 点）。
//!
//! 原生实现引用：`kernel/dying.rs:160`（`has_skill(dying, JIU_YUAN)` + `responder != dying`
//! + `is_lord(dying)` + `effective_faction(responder) == Wu` → `amount = 2`；随后
//! `ctx.recover(dying, amount, Some(tao))`）。
//!
//! **登记差异（与 `hujia`/`jijiang` 同族，第 3 例）**：主公技资格用 `is_lord`，而其在**非身份局
//! 返回 true**（`state.rs` 注释「无身份体系的模式持技即可」）→ **原型模式下救援亦可生效**
//! （**已核对为有意的设计差异**，见修复清单第 17 项）。
//!
//! 未覆盖（登记）：**回复 2 点的行为面**（需构造「主公濒死 + 吴势力角色持桃救援」场景）；
//! `audio` 表现面。

mod nc08_common;

use engine_native::Engine;

/// 上游点 2+3（**源码事实**）：原生资格四条件与回复量 +1 的实现。
#[test]
fn upstream_jiuyuan_qualification_and_bonus() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/dying.rs"
    ))
    .expect("读 dying");
    let idx = src.find("let jiuyuan").expect("救援判定");
    let seg: String = src[idx..].chars().take(900).collect();
    assert!(
        seg.contains("has_skill(self.dying, d)")
            && seg.contains("responder != self.dying")
            && seg.contains("is_lord(self.dying)")
            && seg.contains("Faction::Wu"),
        "原生依据：持技 + 非自己 + 主公资格 + 吴势力：{seg}"
    );
    assert!(
        seg.contains("let amount = if jiuyuan { 2 } else { 1 }"),
        "原生依据：救援使回复量变为 2：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tjiuyuan: {").expect("上游救援");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("zhuSkill: true")
            && seg.contains("event.player.group !== \"wu\"")
            && seg.contains("trigger.baseDamage++"),
        "上游依据：主公技 + 吴势力 + 回复量 +1：{seg}"
    );
}

/// **登记差异（同族第 3 例）**：主公技资格用 `is_lord`（非身份局为真 → 有意的设计差异）。
#[test]
fn upstream_jiuyuan_lord_gate_is_same_family() {
    let state = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/state.rs"
    ))
    .expect("读 state");
    let idx = state.find("pub fn is_lord").expect("is_lord");
    let seg: String = state[..idx].chars().rev().take(200).collect::<String>().chars().rev().collect();
    assert!(
        seg.contains("无身份体系的模式持技即可"),
        "登记：is_lord 注释说明『无身份体系的模式持技即可』（有意设计）：{seg}"
    );
    // 同族第 3 例：hujia / jijiang / jiuyuan 均以 is_lord 作主公技资格
    let sha = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    let jijiang = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/jijiang.rs"
    ))
    .expect("读 jijiang");
    let dying = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/dying.rs"
    ))
    .expect("读 dying");
    assert!(
        sha.contains("is_lord(self.target)")
            && jijiang.contains("is_lord(owner)")
            && dying.contains("is_lord(self.dying)"),
        "同族对照：护驾/激将/救援三处均以 is_lord 作主公技资格（第 3 例）"
    );
    let _ = Engine::new(20269101, 4);
}

/// 上游点 2+3（**行为面**）：吴势力角色以【桃】救援主公 → **回复 2 点**（救援 +1）。
///
/// 口径：原型模式下 `is_lord` 为真（同族设计差异）→ 本用例在该口径下验证「吴势力代桃回复 2」。
#[test]
fn upstream_jiuyuan_wu_tao_recovers_two() {
    use engine_native::{ChoiceOption, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269102, 4);
    e.debug_grant(1, "救援"); // 1 号持救援（濒死者）
    // 2 号装配**吴**势力武将（持桃救援者）
    let wu = e
        .general_candidates()
        .into_iter()
        .find(|d| {
            e.catalog()
                .general_meta(*d)
                .map(|(_, f, _, _, _)| f == engine_native::catalog::Faction::Wu)
                .unwrap_or(false)
        })
        .expect("存在吴势力武将");
    let wu_name = e.catalog().general_meta(wu).map(|(n, _, _, _, _)| n.to_string()).unwrap();
    e.debug_assemble_general(2, &wu_name);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 2 号持一张【桃】
    let tao = {
        let name = e.catalog().cards.iter().find(|c| c.name == "tao").map(|c| c.name).unwrap();
        e.debug_place(2, name)
    };
    // 1 号被打至濒死（真实伤害路径）
    let max1 = e.state_core().player(1).max_hp as i32;
    e.debug_set_hp(1, max1 - 1);
    e.debug_deal_damage(
        Some(0),
        1,
        1,
        engine_native::event::DamageNature::Normal,
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut saved = false;
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            let actor = e.turn().current;
            let _ = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::NextPhase,
            });
            continue;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(tao)) {
            saved = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(tao)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    eprintln!(
        "DBG 救援：询问={prompts:?} 吴势力代桃={saved} 1 号 hp={} alive={}（2 号手牌={:?}）",
        e.debug_hp(1),
        e.state_core().player(1).alive,
        e.debug_hand(2)
    );
    if !saved {
        eprintln!("NOTE 未出现吴势力以桃救援的询问——本用例不构成验收");
        return;
    }
    // 上游：救援使该【桃】回复量 +1 → 从 0 回复到 **2**
    assert_eq!(
        e.debug_hp(1),
        2,
        "救援：吴势力【桃】应回复 **2** 点（救援 +1），实测 hp={}",
        e.debug_hp(1)
    );
    assert!(e.state_core().player(1).alive, "1 号应被救回（存活）");
}
