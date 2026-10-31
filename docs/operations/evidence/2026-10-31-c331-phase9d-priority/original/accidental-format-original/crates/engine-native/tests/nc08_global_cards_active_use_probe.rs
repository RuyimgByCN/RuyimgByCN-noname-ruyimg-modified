//! NC08 附加：**`global` 卡主动使用面普查**（第 383 轮交接的优先级 1）。
//!
//! 背景：上游「杀海拾遗」共 **12 张**带 `global` 的卡（可在**响应窗口**打出）：
//! `chenhuodajie`/`jinlianzhu`/`shihuifen`/`shencaojie`/`zhuquezhizhang`/`xuanwuzhihuang`/
//! `huanglinzhicong`/`cangchizhibi`/`qinglongzhigui`/`baishouzhihu`/`shenmiguo`/`zhufangshenshi`。
//!
//! 本普查记录每张**在目录中**的 `global` 卡的**主动使用受理结果**（三形态），
//! 作为「`global` 窗口用法面」的**基线**（**不预设结论**）。
//!
//! 未覆盖（登记）：各卡**具体窗口**（触发事件）与窗口内是否可达——需**逐卡**按上游 `global` 子技定位。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn pack_engine(pkg: &str, seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg)
        .join("assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 cards");
    let mut pack = content_native_loader::bridge_pack(&cards)
        .expect("桥接")
        .pack;
    if let Ok(chars) = std::fs::read_to_string(dir.join("character.json")) {
        if let Ok(b) = content_native_loader::bridge_generals(&chars) {
            pack.generals = b.generals;
        }
    }
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 返回 `(是否在目录, 三形态受理摘要)`。
fn probe(pkg: &str, name: &str, seed: u64) -> (bool, String) {
    let mut e = pack_engine(pkg, seed);
    advance_to_phase(&mut e, 0, Phase::Play);
    for p in 0..4 {
        e.debug_clear_hand(p);
        let shan = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(p, shan);
    }
    let Some(id) = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains(name))
        .map(|(i, _)| i as engine_native::CardId)
    else {
        return (false, "不在目录".to_string());
    };
    nc08_common::put_hand(&mut e, 0, id);
    let mut out = String::new();
    for targets in [vec![], vec![0], vec![1]] {
        let r = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard {
                card: id,
                targets: targets.clone(),
            },
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
                let Some(c) = e.open_choice().cloned() else {
                    break;
                };
                let sel: Vec<engine_native::ChoiceOption> =
                    if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                        vec![engine_native::ChoiceOption::Pass]
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
            break;
        }
    }
    drain(&mut e);
    (true, out)
}

/// **普查**：12 张 `global` 卡的主动使用受理结果（如实记录）。
#[test]
fn global_cards_active_use_probe() {
    let cases = [
        ("shahai-yunchou-upstream-noname", "chenhuodajie"),
        ("shahai-gujian-upstream-noname", "jinlianzhu"),
        ("shahai-gujian-upstream-noname", "shihuifen"),
        ("shahai-swd-upstream-noname", "shencaojie"),
        ("shahai-swd-upstream-noname", "zhuquezhizhang"),
        ("shahai-swd-upstream-noname", "xuanwuzhihuang"),
        ("shahai-swd-upstream-noname", "huanglinzhicong"),
        ("shahai-swd-upstream-noname", "cangchizhibi"),
        ("shahai-swd-upstream-noname", "qinglongzhigui"),
        ("shahai-swd-upstream-noname", "baishouzhihu"),
        ("shahai-swd-upstream-noname", "shenmiguo"),
        ("shahai-swd-upstream-noname", "zhufangshenshi"),
    ];
    let mut in_catalog = 0;
    for (i, (pkg, n)) in cases.iter().enumerate() {
        let (found, r) = probe(pkg, n, 20288200 + i as u64);
        if found {
            in_catalog += 1;
        }
        eprintln!("DBG global 普查：{pkg} / {n} → 在目录={found}；{r}");
    }
    eprintln!(
        "DBG global 普查结论：12 张中 **在目录 {in_catalog} 张**；其余属转换缺口（见台账第 117 行）"
    );
    eprintln!(
        "NOTE 本普查只记录**主动使用**受理结果；各卡的**响应窗口**（`global` 子技）可达性须逐卡定位（待办）"
    );
}
