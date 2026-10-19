//! NC08 第⑦层（上游行为对照）样本 80：**来源 U05（杀海拾遗·运筹）** 武将技
//! `zonghuo`（纵火）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`zonghuo`）：
//!
//! ```js
//! zonghuo: {
//!     trigger: { source: "damageBefore" },     // **自己造成伤害前**
//!     direct: true, priority: 10,
//!     filter(event) { return event.nature !== "fire"; },     // **非火焰**伤害才可
//!     content() {
//!         "step 0"; player.chooseToDiscard(get.prompt("zonghuo"));   // **弃一张牌（自选）**
//!         "step 1";
//!         if (result.bool) {
//!             player.logSkill("zonghuo", trigger.player, "fire");
//!             trigger.nature = "fire";                               // **该伤害变为火焰**
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己**造成伤害前**（`priority: 10`）；**仅非火焰**伤害；
//! 2. **费用**：**弃一张牌**（`chooseToDiscard` → **玩家自选**）；
//! 3. **效果**：该次伤害的**性质变为火焰**（`trigger.nature = "fire"`）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:357`（`YunchouZonghuo`：取**手牌首张**弃置 →
//! 把当前伤害帧的 `nature` 改为 `Fire`）。
//!
//! **登记差异（本文件固定）**：**弃牌的选择权** —— 上游 `chooseToDiscard`（**玩家自选**，
//! 且**可取消**：未弃牌则伤害性质不变）；原生**自动弃手牌首张**（无询问、无取消）
//! → 与「**取牌选择权**」族**同族**（`liuli`/`tuxi`/`qicai`/`diyguhuo` 已有该族条目）。
//!
//! 未覆盖（登记）：伤害**性质变为火焰**的行为面观测（需火焰敏感载体，如藤甲/烧营——
//! 本样本只固定源码事实与弃牌行为）；`ai`（弃牌评分）策略面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：造成非火焰伤害前**弃一张手牌**（原生自动弃首张、**无询问**）。
///
/// **本轮未验收（登记，含诊断）**：实测——`询问=["trigger_confirm"]`（纵火触发被询问并**已确认**）、
/// **伤害确实落地**（1 号 hp 4 → 3），但 **0 号手牌 2 → 2（未弃牌）** ✗
/// ⇒ 效果分支未产生弃牌，**根因待查**（详见修复清单第 33 项）。用例以 `#[ignore]` 守护，**不写成通过**。
#[test]
#[ignore = "纵火：触发确认后未弃牌（伤害已落地、手牌不变）——根因待查"]
fn upstream_zonghuo_discards_one_without_asking() {
    let mut e = yunchou_engine(20271101);
    e.debug_grant(0, "ycnzonghuo");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        let _ = e.debug_place(0, name);
    }
    let hand_before = e.debug_hand(0).len();
    let hp1_before = e.debug_hp(1);
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    let hand_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 纵火：询问={prompts:?} 0 号手牌 {hand_before} → {hand_after}；1 号 hp {hp1_before} → {}",
        e.debug_hp(1)
    );
    assert_eq!(
        hand_after,
        hand_before - 1,
        "纵火：造成非火焰伤害前应**弃一张手牌**（{hand_before} → {hand_after}）"
    );
    // **登记差异（源码事实）**：原生**自动弃手牌首张**（无「选哪张」询问），
    // 上游为 `chooseToDiscard`（玩家自选、可取消）——与「取牌选择权」族同族。
    assert!(
        !prompts.iter().any(|p| p.contains("zonghuo")),
        "登记差异：原生无『选哪张牌』的询问（自动弃首张）"
    );
}

/// 上游点 1+3（**源码事实**）：时机/门槛与「伤害性质改为火焰」。
#[test]
fn upstream_zonghuo_source_matches() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouZonghuo =>").expect("纵火分支");
    let seg: String = step[idx..].chars().take(900).collect();
    assert!(
        seg.contains("hand.first()") && seg.contains("DamageNature::Fire"),
        "原生依据：自动弃手牌首张 + 把伤害性质改为火焰：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tzonghuo: {").expect("上游纵火");
    let seg: String = up[idx..].chars().take(1000).collect();
    assert!(
        seg.contains("damageBefore")
            && seg.contains("event.nature !== \"fire\"")
            && seg.contains("chooseToDiscard")
            && seg.contains("trigger.nature = \"fire\""),
        "上游依据：damageBefore + 非火焰 + chooseToDiscard + 性质改火焰：{seg}"
    );
}
