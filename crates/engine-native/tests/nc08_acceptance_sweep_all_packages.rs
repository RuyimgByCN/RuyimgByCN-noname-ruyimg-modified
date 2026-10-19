//! NC08 附加：**受理面普查（全部已转换包）**。
//!
//! 动机与口径同 `nc08_acceptance_sweep_small_packages`：
//! - 对每张**已交付卡**用**三种目标形态**（`[]`/`[0]`/`[1]`）尝试发动；
//! - **应答所有中途询问**（不应答会以空选恢复 ⇒ `answer_first` panic，把夹具问题误报为崩溃）；
//! - 判据：**三种形态均被拒** = **候选**（**被拒 ≠ 缺陷**，须完整读取上游 `enable`/`filterTarget`/
//!   `filterAddedTarget` 后才能定性——见台账第 127/128 行与两处证据目录）。
//!
//! 目录名匹配用 `contains`（相等匹配会静默漏卡——本会话多次踩坑）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn engine_for(pkg: &str, seed: u64) -> Option<Engine> {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).ok()?;
    let mut pack = content_native_loader::bridge_pack(&cards).ok()?.pack;
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    Engine::new_with_extensions(seed, 4, &[&pack]).ok()
}

fn answer_all(e: &mut Engine) {
    for _ in 0..24 {
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
        } else if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
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
    drain(e);
}

/// 返回 `None` 表示目录中找不到该卡；否则返回「是否受理」。
fn try_play(pkg: &str, name: &str, seed: u64) -> Option<bool> {
    let mut e = engine_for(pkg, seed)?;
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
        .find(|(_, c)| c.name.contains(name) || name.contains(&c.name))
        .map(|(i, _)| i as engine_native::CardId)?;
    nc08_common::put_hand(&mut e, 0, id);
    for targets in [vec![], vec![0], vec![1]] {
        let out = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card: id, targets },
        });
        if matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) {
            answer_all(&mut e);
            return Some(true);
        }
    }
    answer_all(&mut e);
    Some(false)
}

/// **全包受理面普查**：逐包逐卡记录「三形态是否均被拒」。
#[test]
fn acceptance_sweep_all_packages() {
    let mut dirs: Vec<String> = Vec::new();
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    if let Ok(rd) = std::fs::read_dir(&root) {
        for e in rd.flatten() {
            let p = e.path();
            if p.join("assets/cards.json").is_file() {
                if let Some(n) = p.file_name().and_then(|s| s.to_str()) {
                    dirs.push(n.to_string());
                }
            }
        }
    }
    dirs.sort();
    let mut total = 0usize;
    let mut not_found = Vec::new();
    let mut rejected: Vec<(String, String)> = Vec::new();
    for (pi, pkg) in dirs.iter().enumerate() {
        let f = root.join(pkg).join("assets/cards.json");
        let Ok(txt) = std::fs::read_to_string(&f) else { continue };
        let Ok(v) = serde_json::from_str::<serde_json::Value>(&txt) else { continue };
        let names: Vec<String> = v["cards"]
            .as_array()
            .map(|a| a.iter().filter_map(|c| c["name"].as_str().map(|s| s.to_string())).collect())
            .unwrap_or_default();
        for (ci, name) in names.iter().enumerate() {
            total += 1;
            let seed = 20280000 + (pi as u64) * 200 + ci as u64;
            match try_play(pkg, name, seed) {
                Some(true) => {}
                Some(false) => rejected.push((pkg.clone(), name.clone())),
                None => not_found.push((pkg.clone(), name.clone())),
            }
        }
    }
    rejected.sort();
    rejected.dedup();
    not_found.sort();
    not_found.dedup();
    eprintln!(
        "DBG 全包受理普查：共试 {total} 条；**三形态均被拒 {}(去重)**；目录未找到 {}(去重)",
        rejected.len(),
        not_found.len()
    );
    for (pkg, n) in &rejected {
        eprintln!("DBG   REJECTED {pkg} / {n}");
    }
    for (pkg, n) in &not_found {
        eprintln!("DBG   NOT_FOUND {pkg} / {n}");
    }
}
