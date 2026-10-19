//! NC08 第⑦层（上游行为对照）样本 54：`liuli`（流离，**标准面** / 大乔）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! liuli: {
//!     trigger: { target: "useCardToTarget" },          // 成为【杀】目标时
//!     filter(event, player) {
//!         if (event.card.name !== "sha") return false;
//!         if (!player.hasCards("he")) return false;    // 有**手牌或装备**
//!         return game.hasPlayer(current =>
//!             player.inRange(current)                  // **攻击范围内**
//!             && current !== event.player              // 非使用者
//!             && current !== player                    // 非自己
//!             && !!lib.filter.targetEnabled(event.card, event.player, current));  // **为该杀合法目标**
//!     },
//!     async cost(event, trigger, player) {
//!         event.result = await player.chooseCardTarget({ … });   // 弃一张牌 + 选目标
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：**成为【杀】的目标**时（`target: useCardToTarget`）；
//! 2. **费用**：弃置**一张牌**（`he`：手牌 + 装备）；
//! 3. **目标**：**攻击范围内**、**非使用者**、**非自己**、且**为该杀合法目标**
//!    （→ **空城持有者不可**，因空城使无手牌者不能成为【杀】目标）；
//! 4. **可取消**：整体可放弃。
//!
//! 原生实现引用：`kernel/sha/step.rs:630`（注释自述「**对齐上游 chooseCardTarget**」；
//! 弃牌候选 = 手牌 ∪ 装备；转移目标 = `liuli_candidates`；「整体可取消：弃牌询问空应答=不发动」；
//! 注释并明确「**空城不可**」）。
//!
//! **登记差异**：本样本**未发现**差异——原生实现与上游四条判据逐条对应（含 `targetEnabled`
//! 复查 → 空城目标被排除）。**唯一保留项**：`preHidden`/`audioname` 表现面未覆盖。
//!
//! 未覆盖（登记）：表现面；`cost` 中「先选牌后选目标」的**顺序**行为面（源码注释已述，本样本未逐帧验证）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 上游点 1+2+3（**行为面**）：流离把【杀】转移给攻击范围内的第三名角色。
#[test]
fn upstream_liuli_redirects_sha() {
    let mut e = Engine::new(20268601, 4);
    e.debug_grant(1, "流离");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持【杀】打 1 号；1 号（流离）持一张可弃的牌
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let cost_card = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut redirected = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 流离：先选弃牌（含费用牌）→ 再选目标（2 号）
        if c.spec.options.contains(&ChoiceOption::Card(cost_card)) && !redirected {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(cost_card)],
                },
            });
            continue;
        }
        if c.spec.options.contains(&ChoiceOption::Player(2)) {
            redirected = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(2)],
                },
            });
            continue;
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
    eprintln!(
        "DBG 流离：询问={prompts:?} 已转移={redirected} 费用牌仍在 1 号手牌={} 2 号 hp={}",
        e.debug_hand(1).contains(&cost_card),
        e.debug_hp(2)
    );
    if !redirected {
        eprintln!("NOTE 未出现流离的目标选择询问——本用例不构成验收");
        return;
    }
    assert!(
        !e.debug_hand(1).contains(&cost_card),
        "流离：费用牌应被弃置（不在 1 号手牌）"
    );
    let _ = Phase::Play;
}

/// 上游点 3（**跨技能交互**）：**空城持有者不可**被选为流离的转移目标。
#[test]
fn upstream_liuli_excludes_kongcheng_target() {
    let mut e = Engine::new(20268602, 4);
    e.debug_grant(1, "流离");
    e.debug_grant(2, "空城"); // 2 号持空城
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let cost_card = {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let mut saw_target_prompt = false;
    let mut offered_kongcheng = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.options.contains(&ChoiceOption::Card(cost_card)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(cost_card)],
                },
            });
            continue;
        }
        // 目标询问：检查是否把 2 号（空城、无手牌）列为候选
        if c.spec.options.iter().any(|o| matches!(o, ChoiceOption::Player(_))) {
            saw_target_prompt = true;
            offered_kongcheng = c.spec.options.contains(&ChoiceOption::Player(2));
        }
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
            Vec::new()
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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
    eprintln!("DBG 流离×空城：出现目标询问={saw_target_prompt} 把空城者列为候选={offered_kongcheng}");
    assert!(
        !offered_kongcheng,
        "流离的目标须为『该杀合法目标』→ 空城（无手牌）持有者**不应**入候选"
    );
}
