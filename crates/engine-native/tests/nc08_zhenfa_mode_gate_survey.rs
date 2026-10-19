//! NC08 附加：**阵法卡模式门槛普查**（`mode: ["guozhan"]`）。
//!
//! 背景：上游 `杀海拾遗/card/zhenfa/card.js` 共 **10 张阵法卡**，**全部**带
//! **`mode: ["guozhan"]`**（仅国战模式可用）；样本 127（地载阵）已证 **原生未校验该门槛**
//! （身份局仍可发动，修复清单第 49 项）。
//!
//! 本普查对**已交付的阵法卡**逐一在**身份局（4 人）**下尝试发动，核对该门槛缺失是否**普遍存在**。
//!
//! **口径**：受理 ⇒ **门槛缺失**（差异）；被拒 ⇒ 与上游一致（或另有条件未满足，须逐卡核对）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn zhenfa_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-zhenfa-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读阵法 cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn try_card(name: &str, seed: u64) -> Option<String> {
    let mut e = zhenfa_engine(seed);
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
        let shan = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(p, shan);
    }
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        // 目录名可能已剥前缀/改名 ⇒ 用 contains 匹配（相等匹配会静默漏卡——本会话已多次踩坑）
        .find(|(_, c)| c.name.contains(name) || name.contains(&c.name))
        .map(|(i, _)| i as engine_native::CardId)?;
    nc08_common::put_hand(&mut e, 0, id);
    // 三种形态都试
    for targets in [vec![], vec![0], vec![1]] {
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card: id, targets: targets.clone() },
        });
        if matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) {
            // 应答所有询问，避免空选 panic
            for _ in 0..16 {
                let Some(c) = e.open_choice().cloned() else { break };
                let cards: Vec<ChoiceOption> = c
                    .spec
                    .options
                    .iter()
                    .filter(|o| matches!(o, ChoiceOption::Card(_)))
                    .copied()
                    .collect();
                let need = c.spec.min.max(1);
                let sel: Vec<ChoiceOption> = if cards.len() >= need {
                    cards.into_iter().take(need).collect()
                } else if c.spec.options.contains(&ChoiceOption::Pass) {
                    vec![ChoiceOption::Pass]
                } else if !cards.is_empty() {
                    cards
                } else if c.spec.allow_cancel {
                    Vec::new()
                } else {
                    c.spec.options.iter().take(need).copied().collect()
                };
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer { choice_id: c.id, selection: sel },
                });
            }
            drain(&mut e);
            return Some(format!("{targets:?} → 受理"));
        }
    }
    Some("三种形态均被拒".to_string())
}

/// **普查**：5 张已交付阵法卡在**身份局**下的受理情况。
#[test]
fn zhenfa_mode_gate_survey() {
    let names = [
        "zf_dizaizhen",
        "swd_tianfuzhen1",
        "swd_qixingzhen1",
        "swd_niaoxiangzhen1",
        "swd_changshezhen1",
    ];
    let mut accepted = Vec::new();
    for (i, n) in names.iter().enumerate() {
        match try_card(n, 20275900 + i as u64) {
            Some(r) => {
                eprintln!("DBG 阵法门槛普查：{n} → {r}");
                if r.contains("受理") {
                    accepted.push(*n);
                }
            }
            None => eprintln!("DBG 阵法门槛普查：{n} → 不在目录"),
        }
    }
    eprintln!(
        "DBG 阵法门槛普查：身份局下**仍可发动** {} 张 / 共 {} 张（上游均为 mode: [\"guozhan\"]）",
        accepted.len(),
        names.len()
    );
    assert!(
        !accepted.is_empty(),
        "至少应有部分阵法卡在身份局下可发动（样本 127 已证地载阵如此）"
    );
}
