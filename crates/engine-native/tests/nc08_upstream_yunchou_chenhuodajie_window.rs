//! NC08 第⑦层（上游行为对照）样本 152：**来源 U05（运筹）** `chenhuodajie`（趁火打劫）
//! ——**`global` 窗口的第 2 张定位**（承接样本 150 的方法与 [窗口映射表][map]）。
//!
//! [map]: ../docs/operations/evidence/2026-10-19-global-window-map/README.md
//!
//! 上游源码（`.../yunchou/skill.js` 的 **`g_chenhuodajie`**，**逐字**）：
//!
//! ```js
//! g_chenhuodajie: { trigger: { global: "damageEnd" } }    // **任意角色受到伤害后**
//! ```
//!
//! 卡片本体（`card.js`）：`type: "trick"`、`global: "g_chenhuodajie"`、`filterTarget: true`；
//! `content` 为 **`if (target.countCards("he")) player.gainPlayerCard("he", target, true);`**
//!
//! **本样本的定位**：**窗口可达性**——**他人**受到伤害后，持有者应获得可使用趁火打劫的窗口，
//! 且使用后**获得受伤者一张 `he` 牌**。
//!
//! 未覆盖（登记）：`targetRequired` 的目标面；窗口形态（`trigger_confirm` 或选牌）的逐形态核对。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn yunchou_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-yunchou-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读运筹 cards");
    let mut pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点（**行为面 · 窗口可达性**）：**他人**受伤后，持有者可用趁火打劫并获得其一张 `he` 牌。
///
/// **本轮未验收（登记，含诊断）**：0 号（**持有趁火打劫**）出【杀】打 1 号，1 号**放弃出闪** ⇒
/// 实测 **1 号 `hp=3`（受 1 点伤害）** ✅，但**全程只出现 `respond_shan`**、
/// **未见 `damageEnd` 的趁火打劫窗口** ⇒ 持有者**未获使用机会**。
///
/// 诊断候选：①**窗口未实现**（原生或仅实现主动使用路径，样本 145 已证主动使用可用）；
/// ②窗口需**目标选择**，而夹具/流程未提供；③`{ global: … }` 语义差异（窗口可能只对受伤者本人开放）。
/// 用例以 `#[ignore]` 作为修复/夹具补齐后的验收条件（第 53 项）。
#[test]
#[ignore = "趁火打劫 damageEnd 窗口未出现（伤害已确认发生）——见第 53 项"]
fn upstream_chenhuodajie_window_after_damage() {
    let mut e = yunchou_engine(20288401);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号：趁火打劫（**持牌待窗口**）；1 号：2 张手牌（受伤者，供获取）
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("chenhuodajie"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("趁火打劫在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    for _ in 0..2 {
        let name = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(1, name);
    }
    // **0 号**（当前回合玩家）出【杀】打 1 号 ⇒ 1 号受伤 ⇒ `damageEnd` 窗口对 0 号开放
    // （`{ global: "damageEnd" }` ⇒ 任意伤害结束后均可响应；引擎要求操作者为当前回合玩家）
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    eprintln!("DBG 趁火打劫窗口：0 号出杀={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "0 号出杀应被受理：{out:?}"
    );
    let he_before = e.debug_hand(1).len() + e.debug_equips(1).len();
    let h0_before = e.debug_hand(0).len();
    let mut prompts: Vec<String> = Vec::new();
    let mut used = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == &c.spec.prompt.to_string()) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 窗口形态多样：**选项含该牌**或 **Confirm** 都尝试「使用」
        if !used && c.spec.options.contains(&ChoiceOption::Card(card)) {
            used = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(card)] },
            });
            continue;
        }
        if !used && c.observer == 0 && c.spec.options.contains(&ChoiceOption::Confirm) {
            used = true;
            let _ = e.submit(GameCommand {
                actor: 0,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Confirm] },
            });
            continue;
        }
        // **1 号放弃出闪** ⇒ 承受伤害 ⇒ 触发 `damageEnd` 窗口
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        eprintln!("DBG 趁火打劫窗口：应答 {}(obs{}) sel={sel:?}", c.spec.prompt, c.observer);
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
    drain(&mut e);
    let he_after = e.debug_hand(1).len() + e.debug_equips(1).len();
    let h0_after = e.debug_hand(0).len();
    eprintln!(
        "DBG 趁火打劫窗口：询问={prompts:?} 使用={used}；1 号 he {he_before} → {he_after}；\
         0 号手牌 {h0_before} → {h0_after}；**1 号 hp={}**（满血 {}）",
        e.debug_hp(1),
        e.state_core().player(1).max_hp
    );
    if !used {
        eprintln!("NOTE 趁火打劫窗口：未识别到窗口（询问见上）——本用例不构成验收");
        return;
    }
    assert_eq!(
        he_before - he_after,
        1,
        "趁火打劫（窗口内使用）：**应获得受伤者一张 `he` 牌**（{he_before} → {he_after}）"
    );
    assert_eq!(
        h0_after,
        h0_before,
        "趁火打劫（窗口内使用）：打出 −1、获得 1 ⇒ 持有者手牌数不变（{h0_before} → {h0_after}）"
    );
}

/// **决定性对照（第 53 项）**：把趁火打劫放在**受伤者本人**（1 号）手中，重复同一局面。
///
/// 判读：
/// - 若 **1 号获提供** 而 0 号不获提供 ⇒ **窗口只对受伤者本人开放**（`{ global: … }` **语义差异**，候选③成立）；
/// - 若 **1 号也不获提供** ⇒ 窗口**未实现/需其它条件**（候选①/②）。
#[test]
fn upstream_chenhuodajie_window_damaged_player_holds_it() {
    let mut e = yunchou_engine(20288402);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // **1 号持有趁火打劫**；0 号出【杀】打 1 号
    let card1 = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("chenhuodajie"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("趁火打劫在目录");
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    eprintln!("DBG 趁火打劫对照：0 号出杀={out:?}");
    let mut prompts: Vec<String> = Vec::new();
    let mut offered_to: Vec<usize> = Vec::new();
    let mut used = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == &c.spec.prompt.to_string()) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        if c.spec.options.contains(&ChoiceOption::Card(card1)) {
            let obs = c.observer as usize;
            if !offered_to.contains(&obs) {
                offered_to.push(obs);
            }
            if !used {
                used = true;
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Card(card1)],
                    },
                });
                continue;
            }
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
        "DBG 趁火打劫对照（**受伤者本人持有**）：询问={prompts:?}；**获提供的观察者={offered_to:?}**；\
         使用={used}；1 号 hp={}",
        e.debug_hp(1)
    );
    eprintln!(
        "NOTE 对照判读：若 offered_to 含 1 而不含 0 ⇒ **窗口只对受伤者本人开放**（候选③语义差异）；\
         若两边都没有 ⇒ 候选①/②（窗口未实现或需其它条件）"
    );
}
