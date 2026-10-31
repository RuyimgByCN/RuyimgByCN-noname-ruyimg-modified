//! NC08 第⑦层（上游行为对照）样本 29：`qingnang`（青囊，**标准面** / 华佗）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! qingnang: {
//!     enable: "phaseUse",
//!     filterCard: true,                 // 弃**任意**一张手牌
//!     usable: 1,                        // 出牌阶段限一次
//!     filterTarget(card, player, target) {
//!         if (target.hp >= target.maxHp) return false;   // 仅**已受伤**者
//!         return true;                                    // ← **未排除自己**
//!     },
//!     async content(event, trigger, player) { await event.target.recover(); },  // 回复 1
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/限次**：出牌阶段，**每回合限一次**（`enable: "phaseUse"` + `usable: 1`）；
//! 2. **费用**：弃**一张手牌**（`filterCard: true`，不限花色/牌种）；
//! 3. **目标**：**已受伤**者（`hp < maxHp`）——上游 `filterTarget` **未排除自己**；
//! 4. **效果**：目标**回复 1 点体力**（`recover()`）。
//!
//! 原生实现引用：`content/standard/skills/qing_nang.rs`（`activate` 要求手牌非空且存在
//! **`i != owner`** 的已受伤角色；`TargetAsk` → `Discard` 两阶段；`recover` 1 点）。
//!
//! **登记差异（本文件固定，属规则损失）**：原生 `activate` **要求受伤者不是自己**
//! （`i as PlayerId != owner`）→ **华佗不能青囊自己**；上游 `filterTarget` **无**此排除
//! → **规则损失**（源码注释亦写作「其他角色」，与上游不符）。
//!
//! 未覆盖（登记）：`check`/`ai`/`audio` 表现与策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

fn skill_id(e: &Engine, name: &str) -> u16 {
    e.catalog().skill_by_name(name).expect("技能在目录")
}

/// 上游点 2+3+4：弃一张手牌 → **已受伤的其他角色**回复 1 点。
#[test]
fn upstream_qingnang_discard_heals_wounded_other() {
    let mut e = Engine::new(20265901, 4);
    e.debug_grant(0, "青囊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let cost = nc08_common::give_colored(&mut e, 0, "shan", None);
    e.debug_set_hp(1, 2); // 已受伤
    let hp1 = e.debug_hp(1);
    let skill = skill_id(&e, "青囊");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "青囊应可发动：{out:?}"
    );
    // 目标选择询问 → 选 1 号
    let c = e.open_choice().cloned().expect("应出现目标询问");
    let pick = c
        .spec
        .options
        .iter()
        .find(|o| matches!(o, ChoiceOption::Player(1)))
        .copied()
        .unwrap_or_else(|| panic!("目标选项应含 1 号：{:?}", c.spec.options));
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![pick],
        },
    });
    // 弃牌询问 → 弃费用牌
    if let Some(c2) = e.open_choice().cloned() {
        if c2.spec.options.contains(&ChoiceOption::Card(cost)) {
            let _ = e.submit(GameCommand {
                actor: c2.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c2.id,
                    selection: vec![ChoiceOption::Card(cost)],
                },
            });
        }
    }
    drain(&mut e);
    assert_eq!(
        e.debug_hp(1),
        hp1 + 1,
        "青囊：目标回复 1 点体力（{hp1} → {}）",
        e.debug_hp(1)
    );
}

/// 上游点 3 的**登记差异（规则损失）**：原生**排除自己** → 华佗受伤时**不能**青囊自己。
#[test]
fn upstream_qingnang_self_target_difference_is_registered() {
    let mut e = Engine::new(20265902, 4);
    e.debug_grant(0, "青囊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _cost = nc08_common::give_colored(&mut e, 0, "shan", None);
    // 只有**自己**受伤（其他人都满血）
    e.debug_set_hp(0, 2);
    let skill = skill_id(&e, "青囊");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    eprintln!("DBG 仅自己受伤时发动青囊：{out:?}");
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "登记差异：原生要求受伤者不是自己（`i != owner`）→ 仅自己受伤时**不可发动**（上游可）：{out:?}"
    );
}

/// 上游点 1（限次）：**每回合限一次** → 同回合第二次发动被拒。
#[test]
fn upstream_qingnang_once_per_turn() {
    let mut e = Engine::new(20265903, 4);
    e.debug_grant(0, "青囊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..3 {
        nc08_common::give_colored(&mut e, 0, "shan", None);
    }
    e.debug_set_hp(1, 2);
    let skill = skill_id(&e, "青囊");
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
    // 放弃（若可取消）
    if let Some(c) = e.open_choice().cloned() {
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(1).copied().collect()
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
    let second = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(second, CommandOutcome::Rejected(_)),
        "每回合限一次 → 第二次应被拒：{second:?}"
    );
}

/// 上游点 3（门槛 · 反例）：**满血**目标不可选（上游 `filterTarget` 排除）。
#[test]
fn upstream_qingnang_full_hp_target_excluded() {
    let mut e = Engine::new(20265904, 4);
    e.debug_grant(0, "青囊");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    nc08_common::give_colored(&mut e, 0, "shan", None);
    // 1 号受伤（保证可发动），2 号满血
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 4);
    let skill = skill_id(&e, "青囊");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let c = e.open_choice().cloned().expect("应出现目标询问");
    assert!(
        !c.spec.options.contains(&ChoiceOption::Player(2)),
        "满血目标不应出现在候选中：{:?}",
        c.spec.options
    );
    let _ = Phase::Play;
}
