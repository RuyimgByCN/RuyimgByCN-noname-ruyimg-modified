//! NC08 第⑦层（上游行为对照）样本 25：`zhiheng`（制衡，**标准面** / 孙权）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! zhiheng: {
//!     enable: "phaseUse",
//!     usable: 1,                       // 每回合限一次
//!     position: "he",                  // **手牌 + 装备区**
//!     filterCard: true,
//!     selectCard: [1, Infinity],       // 任意张（≥1）
//!     allowChooseAll: true,
//!     prompt: "弃置任意张牌并摸等量的牌",
//!     async content(event, trigger, player) { await player.draw(event.cards.length); },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/限次**：出牌阶段，**每回合限一次**（`enable: "phaseUse"` + `usable: 1`）；
//! 2. **牌区**：`he`（**手牌 + 装备区**）；
//! 3. **数量**：任意张（`[1, Infinity]`，至少 1）；
//! 4. **效果**：弃置后**摸等量**牌（`draw(event.cards.length)`）。
//!
//! 原生实现引用：`content/standard/skills/zhi_heng.rs`（`activate` 要求**手牌非空**；
//! `Select` 阶段选项**仅来自 `player.hand`**；`Draw` 阶段弃置并摸等量）。
//!
//! **登记差异（本文件固定）**：
//! - **①牌区**：上游 `position: "he"` 含**装备区**，原生候选**只取手牌**（`hand.iter()`）
//!   → 装备牌**不可**用于制衡（**规则损失**）；
//! - **②最小张数**：上游 `selectCard: [1, Infinity]`（**至少 1**），原生 `min: 0` 且 `allow_cancel`
//!   → 空选择即「放弃发动」，语义等价但**询问下限口径不同**（登记，不写成对齐）。
//!
//! 未覆盖（登记）：`check`/`ai`/`audio` 表现与策略面；`allowChooseAll` 属界面便捷项。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
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

fn skill_id(e: &Engine, name: &str) -> u16 {
    e.catalog().skill_by_name(name).expect("技能在目录")
}

/// 上游点 1+3+4：出牌阶段发动 → 弃 2 张手牌 → **摸 2 张**。
#[test]
fn upstream_zhiheng_discard_two_draw_two() {
    let mut e = Engine::new(20265401, 4);
    e.debug_grant(0, "制衡");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..4 {
        let c = place_exact(&mut e, 0, "shan");
        e.debug_give_hand_card(0, 0, c);
    }
    let skill = skill_id(&e, "制衡");
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "制衡应可发动：{out:?}"
    );
    let c = e.open_choice().cloned().expect("应出现 zhiheng_select 询问");
    assert_eq!(c.spec.prompt, "zhiheng_select");
    assert!(c.spec.allow_cancel, "空应答=放弃发动（原生口径）");
    // 选 2 张
    let picks: Vec<ChoiceOption> = c.spec.options.iter().take(2).copied().collect();
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: picks },
    });
    drain(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before,
        "弃 2 摸 2 → 手牌数不变（{hand_before} → {}）",
        e.debug_hand(0).len()
    );
}

/// 上游点 2 的**登记差异①（规则损失）**：装备区牌**不可**用于制衡（候选只取手牌）。
#[test]
fn upstream_zhiheng_equip_zone_difference_is_registered() {
    let mut e = Engine::new(20265402, 4);
    e.debug_grant(0, "制衡");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 手牌 1 张 + 装备 1 件
    let hand_card = place_exact(&mut e, 0, "shan");
    let equip = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name.contains("armor") || c.name.contains("horse"))
            .map(|c| c.name)
            .expect("存在装备牌");
        e.debug_place(0, name)
    };
    // 无 `debug_equip` 助手：经 `state_core_mut()` 放入装备区（既有测试同款做法）
    {
        let core = e.state_core_mut();
        core.player_mut(0).hand.retain(|c| *c != equip);
        core.player_mut(0).equips.push(equip);
    }
    let skill = skill_id(&e, "制衡");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned().expect("应出现询问");
    let options = c.spec.options.clone();
    eprintln!("DBG 制衡候选：{options:?}（手牌={hand_card} 装备={equip}）");
    assert!(
        !options.contains(&ChoiceOption::Card(equip)),
        "登记差异①：原生候选**不含装备区牌**（上游 position: 'he' 含）——规则损失"
    );
    assert!(
        options.contains(&ChoiceOption::Card(hand_card)),
        "手牌应在候选中"
    );
}

/// 上游点 1（限次）：**每回合限一次** → 同回合第二次发动被拒。
#[test]
fn upstream_zhiheng_once_per_turn() {
    let mut e = Engine::new(20265403, 4);
    e.debug_grant(0, "制衡");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..3 {
        let c = place_exact(&mut e, 0, "shan");
        e.debug_give_hand_card(0, 0, c);
    }
    let skill = skill_id(&e, "制衡");
    let first = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(matches!(
        first,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 放弃发动（空应答）→ 消耗次数？
    if let Some(c) = e.open_choice().cloned() {
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
        });
    }
    drain(&mut e);
    let second = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(second, CommandOutcome::Rejected(_)),
        "每回合限一次 → 第二次发动应被拒：{second:?}"
    );
}
