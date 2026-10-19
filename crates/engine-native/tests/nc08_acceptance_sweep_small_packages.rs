//! NC08 第⑦层附加：**受理面普查**（小包：hearth / huanle / yunchou / zhenfa）。
//!
//! **动机（样本 126 火山）**：「**完全不可用**」（发动即被拒）比「效果为空」更严重，
//! 但**只测效果**的样本会整类漏掉它 ⇒ 需要对**每张已交付卡**至少验证「**能否被受理**」。
//!
//! **方法**：对每个包的每张卡，构造**尽量宽松**的合法局面（出牌阶段、手牌/装备/判定区均有牌、
//! 目标存活），用**两种目标形态**（`vec![]` 与「显式自己/首个其他角色」）分别尝试发动，
//! 记录**受理结果**。**判读**：本普查只产出**候选**（被拒 ≠ 缺陷，可能是阶段/条件不合），
//! 须**逐卡**核对上游 `filterTarget`/`enable` 后才能定性。
//!
//! **边界**：不判语义等价；只判「是否至少有一种形态被受理」。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

/// 建局：`packages/extensions/<pkg>/assets/cards.json`（可选并入 `character.json`）。
fn engine_for(pkg: &str, seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 cards.json");
    let mut pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 对一张卡尝试两种目标形态，返回 `(空目标结果, 显式目标结果)`。
fn try_play(pkg: &str, name: &str, seed: u64) -> Option<(String, String)> {
    let mut e = engine_for(pkg, seed);
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 宽松局面：其余角色各 1 张手牌 + 1 件装备；0 号 1 张手牌
    for p in 0..4 {
        let shan = e.catalog().cards.iter().find(|c| c.name == "shan").map(|c| c.name).unwrap();
        e.debug_place(p, shan);
    }
    let equip = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("armor") || c.name.contains("weapon"))
        .map(|(i, _)| i as engine_native::CardId);
    if let Some(q) = equip {
        nc08_common::put_equips(&mut e, 1, q);
    }
    let id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        // 目录名可能带来源前缀（如 `ycn_`）⇒ 用 contains 匹配（否则会**静默跳过**该卡）
        .find(|(_, c)| c.name.contains(name))
        .map(|(i, _)| i as engine_native::CardId)?;
    nc08_common::put_hand(&mut e, 0, id);
    let a = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: id, targets: vec![] },
    });
    let first = match a {
        CommandOutcome::Accepted { .. } => "Accepted".to_string(),
        CommandOutcome::Pending { .. } => "Pending".to_string(),
        CommandOutcome::Rejected(r) => format!("Rejected({r:?})"),
        other => format!("{other:?}"),
    };
    if first == "Accepted" || first == "Pending" {
        drain(&mut e);
        return Some((first.clone(), first));
    }
    let b = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: id, targets: vec![0] },
    });
    let second = match b {
        CommandOutcome::Accepted { .. } => "Accepted".to_string(),
        CommandOutcome::Pending { .. } => "Pending".to_string(),
        CommandOutcome::Rejected(r) => format!("Rejected({r:?})"),
        other => format!("{other:?}"),
    };
    drain(&mut e);
    Some((first, second))
}

/// **受理面普查**：逐包逐卡记录两种形态的受理结果。
#[test]
fn acceptance_sweep_small_packages() {
    let packages = [
        "shahai-hearth-upstream-noname",
        "huanle-upstream-noname",
        "shahai-yunchou-upstream-noname",
        "shahai-zhenfa-upstream-noname",
    ];
    let mut never: Vec<(String, String, String, String)> = Vec::new();
    let mut total = 0usize;
    for (pi, pkg) in packages.iter().enumerate() {
        let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions")
            .join(pkg)
            .join("assets/cards.json");
        let d: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(&dir).expect("读包")).expect("解析包");
        let names: Vec<String> = d["cards"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|c| c["name"].as_str().map(|s| s.to_string()))
                    .collect()
            })
            .unwrap_or_default();
        for (ci, name) in names.iter().enumerate() {
            total += 1;
            let seed = 20276000 + (pi as u64) * 100 + ci as u64;
            match try_play(pkg, name, seed) {
                Some((a, b)) => {
                    if a.starts_with("Rejected") && b.starts_with("Rejected") {
                        never.push((pkg.to_string(), name.clone(), a, b));
                    }
                }
                // **不静默跳过**：目录中找不到该卡也要显式记录
                None => never.push((pkg.to_string(), name.clone(), "NOT_IN_CATALOG".into(), String::new())),
            }
        }
    }
    eprintln!("DBG 受理面普查：共试 {total} 张；两种形态**均被拒** {} 张", never.len());
    for (pkg, name, a, b) in &never {
        eprintln!("DBG   {pkg} / {name}: [] → {a}；[0] → {b}");
    }
    eprintln!(
        "NOTE 本普查只产出**候选**（被拒可能是阶段/条件不合）；须逐卡核对上游 filterTarget/enable 后定性"
    );
}
