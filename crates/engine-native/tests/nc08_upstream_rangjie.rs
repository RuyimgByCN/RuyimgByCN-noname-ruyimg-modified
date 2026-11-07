//! NC08 第⑦层（上游行为对照）样本 8：`rangjie`（让节，L10 史诗卡牌）。
//!
//! 上游源码（`archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js`）：
//!
//! ```js
//! lib.skill.rangjie = {
//!     trigger: { player: "damageEnd" },          // 受到伤害后
//!     direct: true,
//!     content: function () {
//!         'step 0'; event.count = trigger.num;   // 次数 = 本次伤害点数
//!         'step 1'; event.count--;
//!             var choice = ['获得牌'];
//!             if (player.canMoveCard()) choice.push('移动牌');
//!             choice.push('取消');
//!             player.chooseControl(choice)…;
//!         'step 2';
//!             if (result.control == 'cancel2') event.finish();
//!             else {
//!                 player.logSkill('rangjie');
//!                 player.draw();                 // ★ 分支**之前**无条件摸 1 张
//!                 if (result.index == 0) { /* 获得一张指定类型的牌 */ }
//!                 else { /* 移动场上的一张牌 */ }
//!             }
//!     },
//! };
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **触发时机 = 受到伤害后**（`damageEnd`），且**次数 = 本次伤害点数**（`event.count = trigger.num`）；
//! 2. 选项为「获得牌 / 移动牌（仅 `canMoveCard()` 为真时）/ 取消」；
//! 3. **取消** → 直接结束（不摸牌）；
//! 4. **非取消** → 先 `draw()` 摸 1 张，**再**进入所选分支（因此选「移动牌」同样摸 1 张）。
//!
//! 原生实现引用：`content/host/step/skills_cards.rs:201`（三段机）、`:243`（分支 0 内 `draw_cards(owner,1)`）、
//! `:270+`（分支 1 移动牌）。
//!
//! **已知缺陷（本文件用 `#[ignore]` 守护，登记未修）**：原生只在分支 0 摸牌，分支 1（移动牌）
//! **不摸牌** → 违反上游点 4。修复需改 `skills_cards.rs` 的 `HostStage::SelectNature`（`1 =>` 分支
//! 开头补 `ctx.draw_cards(self.owner, 1);`），该文件属**未跟踪的 host 拆分目录** → 待拆分落定后修复。
//!
//! 未覆盖（登记）：`audio` 属表现面；`canMoveCard()` 的完整判据（原生按可用移动目标是否存在）由
//! 选项面承载。

use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn epic_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("epic-cards-upstream-liuli/assets/cards.json"))
        .expect("读史诗卡牌包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<ChoiceOption> =
            if c.spec.options.contains(&ChoiceOption::Pass) {
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

/// 上游点 1+2+3：受伤害后触发，选项含「获得牌/移动牌/取消」，选「取消」不摸牌。
#[test]
fn upstream_rangjie_triggers_after_damage_and_cancel_draws_nothing() {
    let mut e = epic_engine(20263001);
    e.debug_grant(0, "rangjie");
    let hand_before = e.debug_hand(0).len();
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    // 询问 0：触发确认（可选技门）
    let c0 = e.open_choice().cloned().expect("让节触发确认");
    assert_eq!(c0.spec.prompt, "trigger_confirm", "先经过触发确认门");
    let _ = e.submit(GameCommand {
        actor: c0.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c0.id, selection: vec![ChoiceOption::Confirm] },
    });
    // 询问 1：选项面
    let c = e.open_choice().cloned().expect("让节选项询问");
    assert_eq!(c.spec.prompt, "rangjie_choice", "确认后应为选项");
    assert!(
        c.spec.options.contains(&ChoiceOption::Tag(0)),
        "应含「获得牌」：{:?}",
        c.spec.options
    );
    assert!(
        c.spec.options.iter().any(|o| *o == ChoiceOption::Tag(2) || *o == ChoiceOption::Pass),
        "应含「取消」：{:?}",
        c.spec.options
    );
    // 选「取消」→ 不摸牌（原生以 Tag(2) 或 Pass 表达 cancel2）
    let cancel = if c.spec.options.contains(&ChoiceOption::Tag(2)) {
        ChoiceOption::Tag(2)
    } else {
        ChoiceOption::Pass
    };
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: vec![cancel] },
    });
    drain(&mut e);
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before,
        "上游：取消（cancel2）→ event.finish()，不摸牌"
    );
}

/// 上游点 4：**非取消**路径在分支前 `player.draw()` → 选「获得牌」摸 1 张后再取一张。
#[test]
fn upstream_rangjie_gain_branch_draws_then_takes_card() {
    let mut e = epic_engine(20263002);
    e.debug_grant(0, "rangjie");
    let hand_before = e.debug_hand(0).len();
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let c0 = e.open_choice().cloned().expect("让节触发确认");
    let _ = e.submit(GameCommand {
        actor: c0.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c0.id, selection: vec![ChoiceOption::Confirm] },
    });
    let c = e.open_choice().cloned().expect("让节选项询问");
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Tag(0)] },
    });
    // 第二段：类型选择
    let c2 = e.open_choice().cloned().expect("让节类型询问");
    assert_eq!(c2.spec.prompt, "rangjie_type", "第二段应为类型选择");
    assert_eq!(e.debug_hand(0).len(), hand_before + 1, "分支前 draw()：先摸 1 张");
    let _ = e.submit(GameCommand {
        actor: c2.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: c2.id, selection: vec![ChoiceOption::Tag(0)] },
    });
    drain(&mut e);
    assert!(
        e.debug_hand(0).len() >= hand_before + 1,
        "获得牌分支：draw() 后再取一张（≥ +1）"
    );
}

/// 上游点 4 的**分支 1（移动牌）**：同样必须先摸 1 张，再移动场上可见区的一张牌。
#[test]
fn upstream_rangjie_move_branch_must_draw_one() {
    let mut e = epic_engine(20263003);
    e.debug_grant(0, "rangjie");
    e.debug_clear_hand(0);
    // 在 1 号位装备区放置一把武器以激活移动牌分支
    let weapon = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind.is_equip())
        .expect("catalog has equip card") as u32;
    e.state_core_mut().deck.retain(|c| *c != weapon);
    for p in 0..4 {
        e.state_core_mut().player_mut(p).hand.retain(|c| *c != weapon);
    }
    e.state_core_mut().player_mut(1).equips.push(weapon);

    let hand_before = e.debug_hand(0).len();
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    let c0 = e.open_choice().cloned().expect("让节触发确认");
    assert_eq!(c0.spec.prompt, "trigger_confirm");
    let _ = e.submit(GameCommand {
        actor: c0.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c0.id,
            selection: vec![ChoiceOption::Confirm],
        },
    });
    let c = e.open_choice().cloned().expect("场上有牌应提供移动牌分支");
    assert_eq!(c.spec.prompt, "rangjie_choice");
    assert!(
        c.spec.options.contains(&ChoiceOption::Tag(1)),
        "场上有装备时应包含分支 1（移动牌）：{:?}",
        c.spec.options
    );
    let _ = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    });
    let c2 = e.open_choice().cloned().expect("摸 1 张后应打开移动牌选择");
    assert_eq!(c2.spec.prompt, "rangjie_move_card");
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "选择分支 1（移动牌）后必须先摸 1 张牌"
    );
    let _ = e.submit(GameCommand {
        actor: c2.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c2.id,
            selection: vec![ChoiceOption::Card(weapon)],
        },
    });
    let c3 = e.open_choice().cloned().expect("选牌后应打开目标选择");
    assert_eq!(c3.spec.prompt, "rangjie_move_target");
    let _ = e.submit(GameCommand {
        actor: c3.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c3.id,
            selection: vec![ChoiceOption::Player(0)],
        },
    });
    drain(&mut e);
    assert!(
        !e.state_core().player(1).equips.contains(&weapon),
        "1 号位装备应被移走"
    );
    assert!(
        e.state_core().player(0).equips.contains(&weapon),
        "0 号位应获得被移动的装备"
    );
    assert_eq!(
        e.debug_hand(0).len(),
        hand_before + 1,
        "分支 1 结算完成后 0 号位手牌数应净 +1"
    );
}
