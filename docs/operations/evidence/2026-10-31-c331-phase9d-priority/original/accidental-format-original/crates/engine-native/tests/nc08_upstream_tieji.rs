//! NC08 第⑦层（上游行为对照）样本 52：`tieji`（铁骑，**标准面** / 马超）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! tieji: {
//!     trigger: { player: "useCardToPlayered" },
//!     filter(event, player) { return event.card.name === "sha"; },
//!     logTarget: "target",
//!     async content(event, trigger, player) {
//!         const { bool } = await player.judge({
//!             judge(card) {
//!                 if (get.zhu(get.player(), "shouyue")) {          // 守约（主公技变体）
//!                     if (get.suit(card) !== "spade") return 2;    //   非黑桃即成功
//!                 } else {
//!                     if (get.color(card) === "red") return 2;     //   红色即成功
//!                 }
//!                 return -0.5;
//!             },
//!             judge2(result) { return result.bool; },
//!         }).forResult();
//!         if (bool) {
//!             trigger.getParent()?.directHit.add(trigger.target);   // **判定成功 → 目标必中**
//!         }
//!     },
//!     ai: { directHit_ai: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己使用【杀】指定目标后；**判定**：红色（或持守约时**非黑桃**）→ 成功；
//! 2. **效果**：成功则**把该目标标记为必中**（`directHit`）——即目标**不能出【闪】响应**。
//!
//! 原生实现引用：`kernel/sha/step.rs:850`（判定牌**红色** → 目标不能出闪响应；
//! 结果存于 `self.tieji_red`）。
//!
//! **登记差异（本文件固定）**：
//! - **①守约变体未建模**：上游 `get.zhu(player, "shouyue")` 时判据变为「**非黑桃**」；
//!   原生只判**红色** → **守约持有者**的铁骑判据**未建模**（与既有 `wusheng` 守约登记同族）；
//! - **②实现方式**：上游把目标加入 **`directHit`**（通用必中标记，供**其它技能**如无双读取）；
//!   原生把结果存在**杀机器自己的字段**（`tieji_red`）→ **其它技能无法读到该必中标记**
//!   → **与样本 50（无双未检查 `directHit`）互为印证**（同一机制面缺口）。
//!
//! 未覆盖（登记）：`ai.directHit_ai` 策略面；铁骑与**无双**的**交互行为面**（需构造
//! 「铁骑判定成功 + 无双持有者」场景，登记待办）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1+2（**判定成功 → 不能出闪**）：构造判定牌为红色 → 目标无出闪询问、直接受伤。
#[test]
fn upstream_tieji_red_judge_blocks_shan() {
    let mut e = Engine::new(20268401, 4);
    e.debug_grant(0, "铁骑");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 牌堆顶置一张**红色**牌（判定牌）——必须在**摸牌阶段之后**放，否则会被摸走
    let red_top = e.debug_set_deck_top_by(|c| {
        matches!(
            c.suit,
            engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
        )
    });
    let sha = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    // 目标持一张【闪】（若铁骑生效则不会被询问）
    let shan = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(1, name)
    };
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec
                    .options
                    .iter()
                    .take(c.spec.min.max(1))
                    .copied()
                    .collect()
            };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
    drain(&mut e);
    let judged_suit = e.catalog().cards[red_top as usize].suit;
    eprintln!(
        "DBG 铁骑（红判定）：置顶牌={red_top} 花色={judged_suit:?} 询问={prompts:?} 目标 hp {hp1} → {}（持闪={shan}）",
        e.debug_hp(1)
    );
    assert!(
        !prompts.iter().any(|p| p == "respond_shan"),
        "铁骑判定为红色时**不应**出现 respond_shan 询问（目标必中）：{prompts:?}"
    );
    assert!(
        e.debug_hp(1) < hp1,
        "铁骑判定成功 → 目标应受伤（{hp1} → {}）",
        e.debug_hp(1)
    );
    let _ = Phase::Play;
}

/// **登记差异①②**（源码事实固定）：守约变体未建模；必中标记未写入通用 `directHit`。
#[test]
fn upstream_tieji_differences_are_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\ttieji: {").expect("上游铁骑");
    let seg: String = up[idx..].chars().take(1000).collect();
    assert!(
        seg.contains("shouyue") && seg.contains("spade"),
        "上游依据①：守约时判据为非黑桃：{seg}"
    );
    assert!(
        seg.contains("directHit.add(trigger.target)"),
        "上游依据②：成功则加入 directHit（通用必中标记）：{seg}"
    );
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha/step.rs"
    ))
    .expect("读 sha/step");
    let idx = src.find("铁骑（马超）").expect("原生铁骑注释");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        !seg.contains("守约") && !seg.contains("shouyue"),
        "差异①固定：原生未建模守约变体：{seg}"
    );
    assert!(
        seg.contains("tieji_red"),
        "差异②固定：原生把结果存在杀机器字段（tieji_red），未写通用 directHit：{seg}"
    );
}
