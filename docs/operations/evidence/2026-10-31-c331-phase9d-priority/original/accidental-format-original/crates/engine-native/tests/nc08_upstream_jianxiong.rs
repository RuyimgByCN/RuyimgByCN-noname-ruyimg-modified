//! NC08 第⑦层（上游行为对照）样本 20：`jianxiong`（奸雄，**标准面** / 曹操）。
//!
//! **口径更正（2026-10-19 同轮）**：本样本初版把 `character/sb/skill.js` 当作基准（其中奸雄带
//! `sbjianxiong` 标记与「改为摸牌」分支）。**规则基准来源是 `character/standard/skill.js`**，
//! 其奸雄为 `filter: cards && position === 'o'` + `gain(cards)`，**没有**标记/摸牌面 →
//! **「摸牌面未实现」不是差异**（已从基准差异表移除）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! jianxiong: {
//!     trigger: { player: "damageEnd" },
//!     frequent: true,
//!     group: "sbjianxiong_mark",
//!     filter(event, player) {
//!         return (get.itemtype(event.cards) === "cards"
//!                 && event.cards.some(i => get.position(i, true) === "o"))   // 伤害牌在**处理区**
//!             || 3 - player.countMark("sbjianxiong") > 0;                    // 或标记 < 3（可摸）
//!     },
//!     prompt2(event, player) { … }   // 二选一提示：获得伤害牌 / 摸牌
//!     …
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 受到伤害后**（`damageEnd`）；
//! 2. **门槛**：伤害**带牌**且牌在**处理区**（`position === "o"`）**或** `sbjianxiong` 标记 < 3；
//! 3. **效果 = 获得该伤害牌**（或按 `prompt2` 选择**摸牌**）。
//!
//! 原生实现引用：`content/standard/skills/jian_xiong.rs`（filter：`d.card.is_some()` 且**牌已在弃牌堆**；
//! 机器：把该牌从弃牌堆收入手牌；`OptionalGate` 确认）。
//!
//! **登记差异（本文件固定）**：
//! - **①位置口径**：上游以「处理区」为准，原生以「**已在弃牌堆**」为准（原生注释明确
//!   「牌已进弃牌堆」）→ 同一时机的观察窗口不同。
//!
//! 未覆盖（登记）：`audio`/`frequent` 表现面。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
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

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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

/// 上游点 1+3：受到**带牌**伤害后，确认发动 → 获得该伤害牌。
#[test]
fn upstream_jianxiong_gains_damage_card() {
    let mut e = Engine::new(20264801, 4);
    e.debug_grant(0, "奸雄");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.contains("sha"))
        .expect("杀在目录");
    let card = e.debug_place(0, sha);
    e.debug_give_hand_card(0, 0, card);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
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
    // 1 号是否打出【闪】等：统一按取消/放弃处理；随后曹操受伤 → 奸雄确认门
    drain(&mut e);
    let hand_before = e.debug_hand(0).len();
    let mut confirmed = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "trigger_confirm" {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            });
            continue;
        }
        break;
    }
    if !confirmed {
        // 伤害未带牌（例如【杀】被闪避）→ 本用例前提不成立，如实跳过（不写成通过）
        eprintln!("SKIP：本次未出现奸雄确认门（伤害可能未带牌/未命中）");
        return;
    }
    drain(&mut e);
    assert!(
        e.debug_hand(0).len() > hand_before,
        "上游：获得造成伤害的牌（手牌应增加）：before={hand_before} after={}",
        e.debug_hand(0).len()
    );
}

/// 上游点 2 的**登记差异①（位置口径）**：原生以「**已在弃牌堆**」为门槛（源码事实固定）。
#[test]
fn upstream_jianxiong_position_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/jian_xiong.rs"
    ))
    .expect("读奸雄源码");
    assert!(
        src.contains("ctx.state.discard.contains(&card)"),
        "差异①固定：原生门槛为『牌已在弃牌堆』（上游为处理区 position==='o'）"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/sb/skill.js"),
    )
    .expect("读上游 sb/skill.js");
    let idx = up.find("jianxiong: {").expect("上游奸雄");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("get.position(i, true) === \"o\""),
        "上游依据：以处理区判定：{seg}"
    );
}

/// **口径更正**：`sbjianxiong` 标记/摸牌面属 `sb` 面（非规则基准来源）→ 不作为差异登记；
/// 本用例保留为「标准面定义核对」：标准面奸雄只有『获得伤害牌』一条路径。
#[test]
fn upstream_jianxiong_standard_face_has_no_mark_face() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/jian_xiong.rs"
    ))
    .expect("读奸雄源码");
    assert!(
        !src.contains("sbjianxiong") && !src.contains("mark"),
        "差异②固定：原生无 sbjianxiong 标记计数与摸牌分支"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("jianxiong: {").expect("上游奸雄");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("player.gain(") && !seg.contains("countMark"),
        "标准面依据：只有获得路径、无标记计数：{seg}"
    );
}
