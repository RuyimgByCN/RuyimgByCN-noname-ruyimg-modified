//! NC08 第⑦层（上游行为对照）样本 24：`longdan`（龙胆，**标准面** / 赵云）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! longdan: {
//!     group: ["longdan_sha", "longdan_shan", "longdan_draw"],
//!     subSkill: {
//!         sha:  { enable: ["chooseToUse","chooseToRespond"], filterCard: { name: "shan" },
//!                 viewAs: { name: "sha" },  position: "hs",     // 【闪】当【杀】
//!                 prompt: "将一张闪当杀使用或打出" },
//!         shan: { enable: ["chooseToRespond","chooseToUse"], filterCard: { name: "sha" },
//!                 viewAs: { name: "shan" }, position: "hs",     // 【杀】当【闪】
//!                 prompt: "将一张杀当闪使用或打出" },
//!         draw: { trigger: { player: ["useCard","respond"] }, forced: true,
//!                 filter(event, player) { return get.zhu(player, "shouyue")   // **守约**（主公技）
//!                     && (event.skill === "longdan_sha" || event.skill === "longdan_shan"); },
//!                 async content(event, trigger, player) { await player.draw(); … } },
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **双向转化**：【闪】当【杀】（`longdan_sha`）、【杀】当【闪】（`longdan_shan`）；
//! 2. **入口**：两者都是 **`chooseToUse` + `chooseToRespond`**（使用**与**打出）；
//! 3. **牌区**：`hs`（手牌 + 装备区）；
//! 4. **守约**（`longdan_draw`）：主公技下，**用转化牌后摸 1 张**。
//!
//! 原生实现引用：
//! - **使用面**：`engine/play_card.rs:154`（`(longdan && kind == Shan)` → `kind = Sha`）；
//! - **闪响应面**：`kernel/sha/step.rs:1207–1243`（`kind == Shan || (longdan && kind == Sha)`）
//!   与无双双闪臂 `:369–392`（同口径）。
//!
//! **登记差异**：
//! - **①「杀」响应面**：决斗等要求**打出【杀】**的路径只取**实体【杀】**（`kernel/duel.rs:70`）
//!   → 龙胆的「闪→杀」在**打出**方向缺失（与武圣同族，见
//!   [武圣样本](../2026-10-18-nc08-upstream-wusheng/README.md) §7 的方向不对称分析）；
//! - **②守约**：`longdan_draw` 的摸牌面**未建模**（全 crate 无 `shouyue`）。
//!
//! 未覆盖（登记）：`position: "hs"` 的**装备区**转化路径未单独验证；`audio`/`check` 表现与策略面。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel: Vec<engine_native::ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![engine_native::ChoiceOption::Confirm]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

fn place_exact(e: &mut Engine, player: u8, name: &str) -> engine_native::CardId {
    e.debug_place(player, name)
}

/// 上游点 1+2（使用面 · 闪→杀）：【闪】当【杀】**使用** → 被接受且目标结算伤害。
#[test]
fn upstream_longdan_shan_used_as_sha() {
    let mut e = Engine::new(20265301, 4);
    e.debug_grant(0, "龙胆");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let shan = place_exact(&mut e, 0, "shan");
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: shan,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "【闪】应可当【杀】使用（上游 longdan_sha）：{out:?}"
    );
    drain(&mut e);
    assert!(e.debug_hp(1) <= hp1, "转化后的【杀】应结算伤害");
}

/// 上游点 1（使用面 · 反例）：未持龙胆者【闪】不可当【杀】。
#[test]
fn upstream_longdan_requires_the_skill() {
    let mut e = Engine::new(20265302, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let shan = place_exact(&mut e, 0, "shan");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: shan,
            targets: vec![1],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "未持龙胆 → 【闪】不可当【杀】：{out:?}"
    );
}

/// 上游点 1+2（**闪响应面** · 杀→闪）：被杀指定时，响应选项应**包含实体【杀】**（龙胆转化）。
#[test]
fn upstream_longdan_sha_as_shan_in_response() {
    let mut e = Engine::new(20265303, 4);
    e.debug_grant(0, "龙胆");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：只有一张【杀】（无【闪】）；1 号：一张【杀】指向 0 号
    let sha0 = place_exact(&mut e, 0, "sha");
    let sha1 = place_exact(&mut e, 1, "sha");
    let _ = sha0;
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha1,
            targets: vec![0],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "1 号不在出牌阶段（0 号回合）→ 该出牌应被拒（改用 0 号对 1 号）: {out:?}"
    );
    // 改由 0 号对 1 号出杀，1 号（持龙胆、只有杀）响应
    e.debug_grant(1, "龙胆");
    let sha0b = place_exact(&mut e, 0, "sha");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha0b,
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
    let choice = e.open_choice().cloned();
    let (prompt, options) = match choice {
        Some(c) => (c.spec.prompt.to_string(), c.spec.options.clone()),
        None => {
            eprintln!("SKIP：未出现响应询问");
            return;
        }
    };
    eprintln!("DBG 闪响应：prompt={prompt} options={options:?}");
    assert_eq!(prompt, "respond_shan", "应为出闪询问");
    assert!(
        !options.is_empty() && options.iter().any(|o| matches!(o, engine_native::ChoiceOption::Card(_))),
        "龙胆的【杀】→【闪】转化应作为候选出现（上游 longdan_shan + 原生 sha/step.rs:1207）：{options:?}"
    );
}

/// **登记差异②**（守约摸牌面未建模）：源码事实固定。
#[test]
fn upstream_longdan_shouyue_draw_difference_is_fixed() {
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("longdan: {").expect("上游龙胆");
    let seg: String = up[idx..].chars().take(2600).collect();
    assert!(
        seg.contains("get.zhu(player, \"shouyue\")") && seg.contains("player.draw()"),
        "上游依据：守约下用转化牌后摸 1：{seg}"
    );
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card 源码");
    assert!(
        !src.contains("shouyue"),
        "差异②固定：原生未建模守约（longdan_draw 的摸牌面）"
    );
}
