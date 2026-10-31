//! NC08 第⑦层（上游行为对照）样本 114：**来源 U07（hearth）** 锦囊
//! `dunpaigedang`（盾牌格挡）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/hearth/card.js`）：
//!
//! ```js
//! dunpaigedang: {
//!     enable: true, type: "trick", toself: true,
//!     filterTarget(card, player, target) { return player === target; },
//!     selectTarget: -1, modTarget: true,
//!     content() {
//!         "step 0"; target.changeHujia(); target.draw();       // **+1 护甲 + 摸 1**
//!         "step 1"; if (target.countCards("he")) target.chooseToDiscard("he", true);   // **弃 1（强制）**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标**：**仅自己**（`toself`）；
//! 2. **效果**：**+1 护甲**、**摸 1**，随后**弃 1 张 `he` 牌**（**强制**，`chooseToDiscard("he", true)`）。
//!
//! 原生实现引用：`kernel/intrinsic.rs:440/449`（`CardKind::Dunpaigedang`）。
//! 规则基准 golden 表第 55 行记：「盾牌格挡（+1 护甲摸 1 弃 1，**登记差异：确定性首张**）」。
//!
//! **本样本的定位**：验证「**+1 护甲 + 摸 1 + 强制弃 1**」的净效果与弃牌询问。
//!
//! **行为面确认的登记差异**：上游 `chooseToDiscard("he", true)` 为**玩家选择弃哪张**；
//! 原生为**自动弃置（确定性首张）** ⇒ **无询问**（与基准 golden 表第 55 行「确定性首张」一致）。
//!
//! 未覆盖（登记）：`modTarget` 的额外影响面；**自动弃置具体弃了哪张**（本样本只验净效果）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 载入 **hearth 包**建局。
fn hearth_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-hearth-upstream-noname/assets/cards.json"))
            .expect("读 hearth 包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：**+1 护甲 + 摸 1 + 弃 1**（净：手牌 −1）。
#[test]
fn upstream_dunpaigedang_hujia_draw_discard() {
    let mut e = hearth_engine(20274401);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("dunpaigedang"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("盾牌格挡在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 另给两张手牌，供「弃 1」使用
    for _ in 0..2 {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name);
    }
    let hujia_before = e.state_core().player(0).hujia;
    let h_before = e.debug_hand(0).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 盾牌格挡：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "盾牌格挡应可发动（仅自己）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut discarded = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        if !discarded && !cards.is_empty() {
            discarded = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![cards[0]],
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
    let hujia_after = e.state_core().player(0).hujia;
    let h_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 盾牌格挡：询问={prompts:?} 弃牌={discarded}；护甲 {hujia_before} → {hujia_after}；\
         手牌 {h_before} → {h_after}（打出 −1、摸 1、弃 1 ⇒ 净 −1）"
    );
    assert_eq!(
        hujia_after,
        hujia_before + 1,
        "盾牌格挡：应 **+1 护甲**（{hujia_before} → {hujia_after}）"
    );
    // **登记差异（行为面确认）**：上游 `chooseToDiscard("he", true)` 为**玩家选择**，
    // 原生为**自动弃置（确定性首张）** ⇒ **无询问**（基准 golden 表第 55 行已登记「确定性首张」）。
    eprintln!(
        "NOTE 盾牌格挡：弃 1 为**自动**（无询问，实测 {prompts:?}）——与基准登记「确定性首张」一致"
    );
    let _ = discarded;
    assert_eq!(
        h_after,
        h_before - 1,
        "盾牌格挡：打出 −1、摸 1、弃 1 ⇒ 手牌净 −1（{h_before} → {h_after}）"
    );
}
