//! NC08 第⑦层（上游行为对照）样本 81：**来源 U05（杀海拾遗·运筹）** 武将技
//! `shaoying`（烧营）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`shaoying`）：
//!
//! ```js
//! shaoying: {
//!     trigger: { source: "damageAfter" },      // **自己造成伤害后**
//!     direct: true,
//!     filter(event) { return event.nature === "fire"; },     // **仅火焰伤害**
//!     content() {
//!         "step 0";
//!         player.chooseTarget(get.prompt("shaoying"), function (card, player, target) {
//!             return get.distance(trigger.player, target) <= 1 && trigger.player !== target;   // **玩家选目标**
//!         }).ai = …;
//!         "step 1";
//!         if (result.bool) {
//!             var card = get.cards()[0];      // **亮出牌堆顶一张**
//!             card.discard();
//!             player.showCard…
//!             // 红色 → 该目标受 1 点火伤（原生同口径）
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/门槛**：自己造成**火焰**伤害后（`direct: true` → 不询问是否发动）；
//! 2. **目标**：**受伤者攻击距离 ≤ 1** 且**非受伤者本人**（上游由**玩家选**）；
//! 3. **效果**：**亮出牌堆顶一张**并弃置 → **红色**则目标受 **1 点火焰伤害**。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:372`（`YunchouShaoying`：座位序取**首**合法目标
//! （≠受伤者、≠自己、存活、`seat_distance(hurt, i) <= 1`）→ `deck.pop()` 亮出并弃 →
//! **红色**则对该目标 `Damage(Fire, 1)`）。
//!
//! **登记差异（本文件固定）**：**目标的选择权** —— 上游 `chooseTarget`（**玩家自选**）↔
//! 原生**座位序首合法目标**（无询问）→ 与「**链杀**（`luweiyan2` 自动首目标）」同族
//! （**「目标选择权」族，2 例**）。
//!
//! 未覆盖（登记）：`ai`（目标评分）策略面；牌堆为空时的分支（原生 `deck.pop()` 为 None → 直接结束）。

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

/// 上游点 2+3（**行为面**）：火焰伤害后亮出**牌堆顶**——**红色**则首合法目标受 1 点火伤。
///
/// **本轮未验收（登记，含诊断）**：实测 `询问=["trigger_confirm"]`（触发被询问）、
/// 1 号因 0 号的火伤 **hp 4 → 3**，但 **2 号未受伤**（`[4,4,4,4] → [4,3,4,4]`）
/// ⇒ 确认后**效果未执行**——与样本 80（纵火）**同一现象**（详见修复清单第 34 项）。
/// 用例以 `#[ignore]` 守护，**不写成通过**。
#[test]
fn upstream_shaoying_reveals_deck_top_and_burns_red() {
    let mut e = yunchou_engine(20271201);
    e.debug_grant(0, "ycnshaoying");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 把一张**确实为红色**的牌置于牌堆顶。
    // 注意：`debug_set_deck_top(name)` 按**牌种**取牌，同种牌可能有多张不同花色 →
    // 置顶后须**回读 `deck.last()`** 确认花色，必要时换名重试。
    let red_names: Vec<&'static str> = e
        .catalog()
        .cards
        .iter()
        .filter(|c| {
            matches!(
                c.suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            )
        })
        .map(|c| c.name)
        .collect();
    let mut red_top = None;
    for name in red_names.iter().take(40) {
        let id = e.debug_set_deck_top(name);
        let is_red = matches!(
            e.state_core().deck.last().map(|c| e.catalog().cards[*c as usize].suit),
            Some(engine_native::catalog::Suit::Heart) | Some(engine_native::catalog::Suit::Diamond)
        );
        if is_red {
            red_top = Some(id);
            break;
        }
    }
    let red_top = red_top.expect("应能置顶一张确实为红色的牌");
    let hp_before: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    // 0 号对 1 号造成**火焰**伤害 → 触发烧营
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Fire);
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
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
    let hp_after: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    eprintln!(
        "DBG 烧营：询问={prompts:?} 牌堆顶红牌={red_top} hp {hp_before:?} → {hp_after:?}"
    );
    // 0 号火伤 1 号 → 1 号 −1；烧营目标 = 座位序首个「距 1 号 ≤1 且 ≠1 号 ≠0 号」= **2 号**
    assert_eq!(hp_after[1], hp_before[1] - 1, "0 号的火伤应命中 1 号");
    assert_eq!(
        hp_after[2],
        hp_before[2] - 1,
        "烧营：亮出**红色**牌 → 首合法目标（2 号）应受 1 点火伤（{hp_before:?} → {hp_after:?}）"
    );
    assert_eq!(hp_after[0], hp_before[0], "烧营：**自己不是目标**（0 号 hp 不变）");
    assert_eq!(hp_after[3], hp_before[3], "烧营：3 号距 1 号 > 1 → 不受影响");
}

/// 上游点 3（**反例**）：牌堆顶为**黑色** → 目标**不受伤**。
///
/// **注意（空过风险）**：该用例当前**必然通过**——因为红牌分支的效果同样未执行（见第 34 项）。
/// 修复后须**重跑本用例**以确认它仍在「黑色不烧」上成立（而非空过）。
#[test]
fn upstream_shaoying_black_reveal_no_burn() {
    let mut e = yunchou_engine(20271202);
    e.debug_grant(0, "ycnshaoying");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    {
        let black_names: Vec<&'static str> = e
            .catalog()
            .cards
            .iter()
            .filter(|c| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                )
            })
            .map(|c| c.name)
            .collect();
        for name in black_names.iter().take(40) {
            let _ = e.debug_set_deck_top(name);
            let is_black = matches!(
                e.state_core().deck.last().map(|c| e.catalog().cards[*c as usize].suit),
                Some(engine_native::catalog::Suit::Spade) | Some(engine_native::catalog::Suit::Club)
            );
            if is_black {
                break;
            }
        }
    }
    let hp_before: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Fire);
    drain(&mut e);
    let hp_after: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    eprintln!("DBG 烧营（黑牌）：hp {hp_before:?} → {hp_after:?}");
    assert_eq!(hp_after[1], hp_before[1] - 1, "0 号的火伤应命中 1 号");
    assert_eq!(
        hp_after[2], hp_before[2],
        "烧营：亮出**黑色**牌 → 2 号**不受**火伤（{hp_before:?} → {hp_after:?}）"
    );
}

/// 上游点 1+2+3（**源码事实**）：时机/门槛与「首合法目标 + 红牌才烧」。
#[test]
fn upstream_shaoying_source_matches() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    // 分支较长 → 用**全文件**检查（固定窗口会截断）
    assert!(
        step.contains("seat_distance(hurt, i) <= 1")
            && step.contains("deck.pop()")
            && step.contains("if red"),
        "原生依据：距离≤1 首目标 + 取牌堆顶 + 红色才烧"
    );
    assert!(
        step.contains("上游玩家选目标"),
        "原生注释已自述该差异（上游玩家选目标）"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\tshaoying: {").expect("上游烧营");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("event.nature === \"fire\"")
            && seg.contains("get.distance(trigger.player, target) <= 1")
            && seg.contains("chooseTarget"),
        "上游依据：仅火伤 + 距离≤1 + 玩家选目标：{seg}"
    );
}
