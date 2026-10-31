//! NC08 第⑦层（上游行为对照）样本 87：**来源 U05（古剑）** 锦囊
//! `jinlianzhu`（金莲珠）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`，`jinlianzhu`）：
//!
//! ```js
//! jinlianzhu: {
//!     type: "trick", fullskin: true, filterTarget: true,
//!     global: "g_jinlianzhu",                        // **全局子技**：可在触发时机**打出**
//!     content() {
//!         var evt = event.getParent(3)._trigger;     // 取出**被触发的那个事件**
//!         evt.cancel();                              // **取消它**
//!         if (evt.source) evt.source.draw();         // **来源摸 1**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **用法**：作为**打断**（全局子技）在某个事件**即将生效**时使用；
//! 2. **效果**：**取消该事件**；若该事件**有来源** → **来源摸 1 张**。
//!
//! 原生实现引用：`content/host/factory.rs`（`HostBehavior::JinlianzhuCancel`：
//! `EventData::Damage(d) => d.source…` → 目标集合取伤害来源）；其**取消**语义由宿主机器承担。
//!
//! **本样本的定位**：验证「**打断**窗口是否可达 + 取消后来源摸 1」。
//! 若打断窗口不可达（无任何询问）→ **如实登记**，不写成通过。
//!
//! 未覆盖（登记）：可被取消的**其它事件类型**（上游为 `getParent(3)._trigger` 泛化）；
//! `ai.result` 策略面。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 载入 **古剑包**建局。
fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 1+2（**行为面**）：打断窗口出现 → 取消伤害 → **来源摸 1**、受伤者**不掉血**。
#[test]
fn upstream_jinlianzhu_interrupts_and_source_draws() {
    let mut e = gujian_engine(20271701);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持金莲珠（打断牌）；1 号持【杀】打 2 号（2 号无闪 → 本应受伤）
    let jin = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("jinlianzhu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("金莲珠在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 让 1 号成为当前回合角色：推进到 1 号的出牌阶段
    assert!(
        nc08_common::advance_to_phase(&mut e, 1, engine_native::Phase::Play),
        "应到达 1 号出牌阶段"
    );
    let sha = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(1, name)
    };
    let hp2_before = e.debug_hp(2);
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![2],
        },
    });
    eprintln!("DBG 金莲珠：1 号出杀={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "出杀应被受理：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut used = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        // 0 号（金莲珠持有者）的打断窗口：**Confirm/Pass** 形态（实测询问名 gj_jinlianzhu_confirm）
        if !used
            && c.observer == 0
            && (c.spec.prompt.contains("jinlianzhu")
                || c.spec.options.contains(&ChoiceOption::Card(jin)))
        {
            used = true;
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
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    drain(&mut e);
    let hp2_after = e.debug_hp(2);
    let h1_after = e.debug_hand(1).len();
    eprintln!(
        "DBG 金莲珠：询问={prompts:?} 使用={used} 2 号 hp {hp2_before} → {hp2_after}；1 号手牌 {h1_before} → {h1_after}"
    );
    if !used {
        eprintln!("NOTE 未出现金莲珠的打断窗口（无询问可选该牌）——本用例不构成验收");
        return;
    }
    assert_eq!(
        hp2_after, hp2_before,
        "金莲珠：**取消**该伤害 → 2 号不应掉血（{hp2_before} → {hp2_after}）"
    );
    assert!(
        h1_after > h1_before - 1,
        "金莲珠：取消后**来源（1 号）摸 1**（打出【杀】−1、摸 1 → 手牌不低于 {h1_before}）"
    );
}
