//! NC08 附加：**救援检测卡牌集合**普查（承接修复清单第 52 项）。
//!
//! 背景：第 52 项的对照实验确认——同一濒死局面下，**【桃】会开启 `dying_save` 救援窗口并救回**，
//! 而**【虎符】完全不触发救援**（目标直接死亡）⇒ 需**系统核对「哪些牌被视为救援牌」**。
//!
//! **方法**：对候选救援牌逐一构造同一局面（0 号持该牌、1 号 1 血、0 号出【杀】），
//! 观察濒死时该牌**是否出现在 `dying_save` 窗口中**。
//!
//! **口径**：本普查只记录「**是否获提供**」这一事实；**是否应当**由上游语义判定（登记/修复项）。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 返回 `(是否获提供, 是否被救回, 询问名)`。
fn probe(card_name: &str, seed: u64) -> (bool, bool, Vec<String>) {
    let mut e = swd_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let Some(rescue) = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        // 目录名可能带来源前缀（如 `swd_hufu`）⇒ 用 contains（相等匹配会静默漏卡）
        .find(|(_, c)| c.name.contains(card_name))
        .map(|(i, _)| i as engine_native::CardId)
    else {
        return (false, false, vec!["NOT_IN_CATALOG".into()]);
    };
    nc08_common::put_hand(&mut e, 0, rescue);
    e.debug_set_hp(1, 1);
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let _ = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    let mut offered = false;
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == &c.spec.prompt.to_string()) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.options.contains(&ChoiceOption::Card(rescue)) {
            offered = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(rescue)] },
            });
            continue;
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
    (offered, e.state_core().player(1).alive, prompts)
}

/// **普查**：候选救援牌在 `dying_save` 窗口中是否获提供。
#[test]
fn rescue_card_set_survey() {
    let names = ["tao", "jiu", "hufu"];
    let mut rows = Vec::new();
    for (i, n) in names.iter().enumerate() {
        let (offered, saved, prompts) = probe(n, 20287800 + i as u64);
        eprintln!("DBG 救援集合：{n} → 获提供={offered} 被救回={saved} 询问={prompts:?}");
        rows.push((*n, offered, saved));
    }
    eprintln!("DBG 救援集合结论：{rows:?}");
    let tao = rows.iter().find(|(n, _, _)| *n == "tao").expect("桃");
    assert!(tao.1, "【桃】应在 `dying_save` 窗口中获提供（对照基准）");
    eprintln!(
        "NOTE 救援集合：仅【桃】获提供（{}）⇒ 与第 52 项一致：**救援检测未覆盖【虎符】**；\
         本普查给出「救援牌集合」的可复核清单",
        rows.iter().filter(|(_, o, _)| *o).map(|(n, _, _)| *n).collect::<Vec<_>>().join("/")
    );
}
