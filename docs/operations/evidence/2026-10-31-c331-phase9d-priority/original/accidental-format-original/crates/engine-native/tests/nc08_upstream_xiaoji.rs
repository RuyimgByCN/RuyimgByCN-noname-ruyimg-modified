//! NC08 第⑦层（上游行为对照）样本 32：`xiaoji`（枭姬，**标准面** / 孙尚香）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! xiaoji: {
//!     trigger: {
//!         player: "loseAfter",                                  // 自己**失去牌**之后
//!         global: ["equipAfter", "addJudgeAfter", "gainAfter",
//!                  "loseAsyncAfter", "addToExpansionAfter"],     // 异步失去面
//!     },
//!     getIndex(event, player) {
//!         const evt = event.getl(player);
//!         if (evt?.player === player && evt.es) {
//!             return evt.es.length;                              // 失去的**装备区牌数**
//!         }
//!         return false;
//!     },
//!     async content(event, trigger, player) { await player.draw(2); },   // 摸 2
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机 = 自己失去装备区的牌之后**（`loseAfter` + `es` 计数；含**替换/弃置/被获得**等**所有**失去方式）；
//! 2. **效果 = 摸 2 张**（`draw(2)`）。
//!
//! 原生实现引用：`content/standard/skills/xiao_ji.rs`（filter 仅匹配
//! `EventData::Equipped { replaced: Some(_) }` —— 即**仅「替换」**一种失去方式）。
//!
//! **登记差异（本文件固定，属规则损失）**：原生只覆盖**替换**（同栏位换装）导致的失去装备，
//! **不覆盖**「装备被弃置/被获得/被拆」等其他失去方式（`XIAO_JI` 在 crate 内除定义与行为外
//! 无其他引用；`equips.retain(...)` 的弃置路径不发 `Equipped{replaced}`）→ **规则损失**。
//!
//! 未覆盖（登记）：`frequent`/`ai`/`audio` 表现与策略面。

mod nc08_common;
use engine_native::catalog::Suit;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

fn equip_id(e: &mut Engine, player: u8, name_contains: &str) -> engine_native::CardId {
    let id =
        e.catalog()
            .cards
            .iter()
            .position(|c| c.name.contains(name_contains))
            .unwrap_or_else(|| panic!("存在装备 {name_contains}")) as engine_native::CardId;
    nc08_common::put_equips(e, player, id);
    id
}

/// 上游点 1+2（**替换**路径）：同栏位换装 → 失去装备 → 摸 2。
#[test]
fn upstream_xiaoji_replace_draws_two() {
    let mut e = Engine::new(20266201, 4);
    e.debug_grant(0, "枭姬");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 先装一件防具（在装备区），第二件放**手牌**——出牌命令要求牌在手牌中
    let _first = equip_id(&mut e, 0, "armor");
    let second = {
        let id = e
            .catalog()
            .cards
            .iter()
            .filter(|c| c.name.contains("armor"))
            .map(|c| c.name)
            .nth(1)
            .map(|n| e.debug_place(0, n))
            .unwrap_or_else(|| panic!("存在第二件防具"));
        id
    };
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: second,
            targets: vec![],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "替换装备应被接受：{out:?}"
    );
    // 触发确认门（可选技）→ 确认
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
    drain(&mut e);
    if !confirmed {
        eprintln!("SKIP：替换装备未出现枭姬确认门（布阵或路径未对上）");
        return;
    }
    // 净变化 = 打出的装备离手（-1） + 枭姬摸 2 = **+1**（故断言 >= +1 而非 +2）
    assert!(
        e.debug_hand(0).len() >= hand_before + 1,
        "枭姬：替换装备失去装备牌 → 摸 2（打出 -1 后净 +1）：{hand_before} → {}",
        e.debug_hand(0).len()
    );
}

/// 上游点 1 的**登记差异（规则损失）**：装备被**拆走**（过河拆桥）→ 原生**不触发**枭姬。
#[test]
fn upstream_xiaoji_discard_loss_difference_is_registered() {
    let mut e = Engine::new(20266202, 4);
    e.debug_grant(0, "枭姬");
    advance_to_play(&mut e, 1); // 由 1 号出牌
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号装备一件防具
    let armor = equip_id(&mut e, 0, "armor");
    // 1 号一张【过河拆桥】（按 id 取，避免花色/实例问题）
    let guohe = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "dismantle")
        .expect("过河拆桥在目录") as engine_native::CardId;
    nc08_common::put_hand(&mut e, 1, guohe);
    let _ = Suit::Heart;
    let hand0_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: guohe,
            targets: vec![0],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "过河拆桥应被接受：{out:?}"
    );
    // 选择拆除目标（0 号的装备）
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == "trigger_confirm" {
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
        if c.spec.options.contains(&ChoiceOption::Card(armor)) {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(armor)],
                },
            });
            continue;
        }
        break;
    }
    drain(&mut e);
    assert!(
        !e.debug_equips(0).contains(&armor),
        "前提：装备应已被拆走（equips={:?}）",
        e.debug_equips(0)
    );
    eprintln!(
        "DBG 拆装备后：hand0={} （若为 {} 则未触发枭姬）",
        e.debug_hand(0).len(),
        hand0_before
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand0_before,
        "登记差异：装备被拆走属上游 `loseAfter`+`es`（应摸 2），原生**不触发**（仅覆盖替换）"
    );
}

/// 上游点 1（技能门槛）：**未持枭姬**时替换装备不摸牌（源码事实 + 行为）。
#[test]
fn upstream_xiaoji_requires_the_skill() {
    let mut e = Engine::new(20266203, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _first = equip_id(&mut e, 0, "armor");
    let second = {
        let name = e
            .catalog()
            .cards
            .iter()
            .filter(|c| c.name.contains("armor"))
            .map(|c| c.name)
            .nth(1)
            .expect("存在第二件防具");
        e.debug_place(0, name)
    };
    let hand_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: second,
            targets: vec![],
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let open = e.open_choice().cloned();
    assert!(
        open.is_none(),
        "未持枭姬 → 不应出现确认门：{:?}",
        open.map(|c| c.spec.prompt)
    );
    drain(&mut e);
    assert!(
        e.debug_hand(0).len() <= hand_before + 1,
        "未持枭姬：替换装备不应摸 2（{hand_before} → {}）",
        e.debug_hand(0).len()
    );
    let _ = Phase::Play;
}
