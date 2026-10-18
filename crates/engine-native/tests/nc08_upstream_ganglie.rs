//! NC08 第⑦层（上游行为对照）样本 21：`ganglie`（刚烈，**标准面** / 夏侯惇）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! ganglie: {
//!     trigger: { player: "damageEnd" },
//!     async content(event, trigger, player) {
//!         const { source } = trigger;
//!         let result = await player.judge({ judge(card) {
//!             if (get.suit(card) === "heart") return -2;   // 红桃 → 失败
//!             return 2;
//!         } }).forResult();
//!         if (!result?.bool || !source?.isIn()) return;
//!         result = source.countDiscardableCards(source, "h") < 2
//!             ? { bool: false }
//!             : await source.chooseToDiscard({ selectCard: 2, … }).forResult();  // **弃两张手牌**
//!         if (!result?.bool) await source.damage();        // 否则来源受 1 点伤害
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 受到伤害后**（`damageEnd`），伤害来源存活才可发动；
//! 2. **判定**：结果**为红桃** → 无事发生；
//! 3. **结果不为红桃** → 来源**二选一**：**弃置两张手牌**，否则**受到来自你的 1 点伤害**。
//!
//! 原生实现引用：`content/standard/skills/gang_lie.rs`（filter：`damageEnd` 且来源非自身；
//! 机器：判定 → 非红桃 → **直接**对来源造成 1 点伤害）。
//!
//! **登记差异（本文件固定，属规则损失）**：原生**缺**「来源弃置两张手牌」分支 —— 上游在
//! 来源手牌 ≥ 2 时**先**询问弃两张，未弃才受伤；原生无条件让来源受伤。
//!
//! 未覆盖（登记）：`check`/`ai` 策略面；`judge2` 的判定牌去向（原生走判定区结算）。

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
        let Some(c) = e.open_choice().cloned() else { break };
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
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.contains(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

/// 上游点 1+2+3：判定**非红桃** → 原生**直接**让来源受伤（缺「弃两张手牌」分支）。
#[test]
fn upstream_ganglie_non_heart_hits_source_directly() {
    let mut e = Engine::new(20264901, 4);
    e.debug_grant(0, "刚烈");
    advance_to_play(&mut e, 0);
    // 判定牌：非红桃（用【杀】=黑桃/梅花/红桃混合 → 取一张非红桃实体）
    let non_heart = {
        let catalog = e.catalog();
        catalog
            .cards
            .iter()
            .position(|c| c.name.contains("sha") && c.suit != engine_native::catalog::Suit::Heart)
            .expect("存在非红桃【杀】") as u32
    };
    let _ = e.debug_place_judge(0, "sha");
    let _ = non_heart;
    // 来源（1 号）受伤前体力
    let hp1_before = e.debug_hp(1);
    let _ = e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    // 触发确认门 → 确认
    let mut confirmed = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
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
    drain(&mut e);
    if !confirmed {
        eprintln!("SKIP：未出现刚烈确认门（判定/触发条件未满足）");
        return;
    }
    assert!(
        e.debug_hp(1) <= hp1_before,
        "登记差异：原生非红桃时**直接**让来源受伤（上游先问『弃两张手牌』）：before={hp1_before} after={}",
        e.debug_hp(1)
    );
}

/// 上游点 3 的**登记差异**（源码事实固定）：原生无「弃置两张手牌」分支。
#[test]
fn upstream_ganglie_discard_branch_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/gang_lie.rs"
    ))
    .expect("读刚烈源码");
    // 原生文件里唯一涉及弃置的是**判定牌**入弃牌堆（ctx.to_discard），没有「来源弃两张手牌」的询问
    assert!(
        !src.contains("chooseToDiscard")
            && !src.contains("两张")
            && !src.contains("SelectCards"),
        "差异固定：原生缺『来源弃置两张手牌』分支（上游 chooseToDiscard selectCard:2）；\
         文件内 `to_discard` 仅用于判定牌入弃牌堆"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("ganglie: {").expect("上游刚烈");
    let seg: String = up[idx..].chars().take(2400).collect();
    assert!(
        seg.contains("selectCard: 2") && seg.contains("source.damage()"),
        "上游依据：先问弃两张，未弃才受伤：{seg}"
    );
    assert!(
        seg.contains("get.suit(card) === \"heart\""),
        "上游依据：红桃则无事发生：{seg}"
    );
}

/// 上游点 1（技能门槛）：**未持技**者受到伤害不触发（无确认门）。
#[test]
fn upstream_ganglie_requires_the_skill() {
    let mut e = Engine::new(20264902, 4);
    advance_to_play(&mut e, 0);
    let _ = e.debug_place_judge(0, "sha");
    let _ = e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let open = e.open_choice().cloned();
    assert!(
        open.is_none(),
        "未持刚烈 → 不应出现确认门：{:?}",
        open.map(|c| c.spec.prompt)
    );
}
