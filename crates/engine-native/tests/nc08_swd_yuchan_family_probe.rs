//! NC08 附加：**swd 渔产族 + 天仙酒** 主动使用面探测（样本 148）。
//!
//! 背景：
//! - `yuchanxun`（渔产·巽）在上游带 **`savable: true`**，救援集合普查（台账第 144 行）已证
//!   **原生未把它纳入救援检测**（与虎符同因，修复清单第 52 项）；
//! - 该族在原生 `play_card.rs:576-600` 的**大组**中，其注释为
//!   「**器族五件：手持牌（非装备路径；效果经手持触发技）**」⇒ 预期**不能主动使用**。
//!
//! **本探测的目的**：记录这些卡的**主动使用受理结果**（作为清单/台账的可复核事实），
//! **不预设结论**（受理与被拒都如实记录）。
//!
//! 未覆盖（登记）：**手持触发技**的触发面；救援检测修复后的复测（第 52 项）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 返回该卡「三种目标形态」的受理结果摘要。
fn probe(name: &str, seed: u64) -> Option<String> {
    let mut e = swd_engine(seed);
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
        .find(|(_, c)| c.name.contains(name))
        .map(|(i, _)| i as engine_native::CardId)?;
    nc08_common::put_hand(&mut e, 0, id);
    let mut out = String::new();
    for targets in [vec![], vec![0], vec![1]] {
        let r = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card: id, targets: targets.clone() },
        });
        let tag = match &r {
            CommandOutcome::Accepted { .. } => "Accepted",
            CommandOutcome::Pending { .. } => "Pending",
            CommandOutcome::Rejected(_) => "Rejected",
            _ => "Other",
        };
        out.push_str(&format!("{targets:?}→{tag} "));
        if tag == "Accepted" || tag == "Pending" {
            for _ in 0..12 {
                let Some(c) = e.open_choice().cloned() else { break };
                let sel: Vec<engine_native::ChoiceOption> =
                    if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                        vec![engine_native::ChoiceOption::Pass]
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
            break;
        }
    }
    drain(&mut e);
    Some(out)
}

/// **探测**：渔产族与天仙酒的主动使用受理结果（如实记录）。
#[test]
fn swd_yuchan_family_active_use_probe() {
    let names = ["yuchanxun", "yuchankan", "tianxianjiu"];
    for (i, n) in names.iter().enumerate() {
        match probe(n, 20288100 + i as u64) {
            Some(r) => eprintln!("DBG 渔产族探测：{n} → {r}"),
            None => eprintln!("DBG 渔产族探测：{n} → 不在目录"),
        }
    }
    eprintln!(
        "NOTE 渔产族探测：原生 `play_card.rs:576-600` 大组注释为「器族五件：手持牌（非装备路径；\
         效果经手持触发技）」⇒ 主动使用**预期被拒**；若某项受理，须核对是否与该注释口径不符"
    );
}
