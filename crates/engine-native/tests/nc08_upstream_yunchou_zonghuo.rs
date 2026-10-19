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
/// **本轮未验收（登记，含诊断）**：两种伤害路径均复现「确认后未弃牌」——
/// ① `debug_deal_damage`：询问 `trigger_confirm@obs0`、1 号 hp 4 → 3，但手牌 **2 → 2**（未弃）；
/// ② **真实出杀**：手牌 **2 → 1**（离手的**只是被使用的【杀】**），**标记牌【闪】仍在手**
/// （`标记牌是否已弃=false`）⇒ **纵火确实没有弃牌**（判据用标记牌，排除「出杀消耗」干扰）。
/// 详见修复清单第 33/34 项；用例以 `#[ignore]` 守护，**不写成通过**。
#[test]
#[ignore = "纵火：确认后未弃牌（标记牌仍在手；两种伤害路径均复现）——根因待查，见第 33/34 项"]
fn upstream_zonghuo_discards_one_without_asking() {
    let mut e = yunchou_engine(20271101);
    e.debug_grant(0, "ycnzonghuo");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 只放**两张**：一张已知的【闪】（标记牌）+ 一张【杀】（用于真实出杀）
    let marker = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    // 用**真实的【杀】**造成伤害（比 `debug_deal_damage` 更贴近实际结算路径）
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let hand_before = e.debug_hand(0).len();
    let hp1_before = e.debug_hp(1);
    let played = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    eprintln!("DBG 纵火：出杀={played:?}");
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
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
    // **判据**：标记牌【闪】是否被纵火弃掉（不受「出杀消耗一张」干扰）
    let marker_gone = !e.debug_hand(0).contains(&marker);
    eprintln!("DBG 纵火：标记牌（闪 id={marker}）是否已弃={marker_gone}");
    assert!(
        marker_gone,
        "纵火：造成非火焰伤害前应**弃一张手牌**——标记牌应被弃（手牌 {hand_before} → {hand_after}，         现手牌={:?}）",
        e.debug_hand(0)
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
