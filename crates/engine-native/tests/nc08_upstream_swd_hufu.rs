//! NC08 第⑦层（上游行为对照）样本 143：**来源 U06（swd）** 基本牌
//! `hufu`（虎符）——**濒死救援专用**。
//!
//! 上游依据（规则基准 golden 表第 45 行「**虎符**（现 spade1+heart1）」+ 原生实现）：
//!
//! ```rust
//! // engine/play_card.rs:1162  与 CardKind::Tao 同组（救援牌）
//! // engine/play_card.rs:1178  if kind == CardKind::Hufu && !p.dying { … 拒绝 … }
//! ```
//!
//! 原生实现（**逐字**）：`CardKind::Tao | CardKind::Hufu` 同组受理，且 **`Hufu` 要求目标处于濒死**
//! （`!p.dying` 即拒绝）⇒ **虎符是「濒死救援」牌**（与【桃】同组但**仅限濒死**）。
//!
//! **本样本的定位**：**两支**——①目标**濒死** ⇒ **受理且回血**；②目标**未濒死** ⇒ **拒绝**。
//!
//! 夹具要点：**濒死须由真实伤害造成**（`debug_deal_damage`），直接写 `hp = 0` **不会**进入濒死。
//!
//! 未覆盖（登记）：虎符的**花色/点数**面（基准记 spade1+heart1）；与【桃】的**差异语义**（是否等同回 1）。

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

/// 跑一次：`dying` 决定 1 号是否濒死。返回 `(发动结果, hp 变化)`。
fn run(seed: u64, dying: bool) -> (String, i32) {
    let mut e = swd_engine(seed);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    if dying {
        // **真实伤害**造成濒死（写 hp=0 不会设置 dying）
        let hp1 = e.debug_hp(1).max(0) as u32;
        let dmg = e.debug_deal_damage(None, 1, hp1, engine_native::event::DamageNature::Normal);
        eprintln!("DBG 虎符：造成伤害 → {dmg:?}；1 号 hp={}", e.debug_hp(1));
        // 濒死救援窗口：若其中**提供虎符**（或救援类牌）则打出之
        let card_id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("hufu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("虎符在目录");
        nc08_common::put_hand(&mut e, 0, card_id);
        let mut prompts: Vec<String> = Vec::new();
        let mut used = false;
        for _ in 0..16 {
            let Some(c) = e.open_choice().cloned() else { break };
            if !prompts.iter().any(|p| p == c.spec.prompt) {
                prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
            }
            if !used && c.spec.options.contains(&engine_native::ChoiceOption::Card(card_id)) {
                used = true;
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![engine_native::ChoiceOption::Card(card_id)],
                    },
                });
                continue;
            }
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
        drain(&mut e);
        eprintln!(
            "DBG 虎符（救援窗口）：询问={prompts:?} 用虎符={used} ⇒ 1 号 hp={}",
            e.debug_hp(1)
        );
        return (if used { "RescueAccepted".to_string() } else { "RescueNoOffer".to_string() }, e.debug_hp(1));
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("hufu"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("虎符在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp_before = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![1] },
    });
    let tag = match &out {
        CommandOutcome::Accepted { .. } => "Accepted".to_string(),
        CommandOutcome::Pending { .. } => "Pending".to_string(),
        CommandOutcome::Rejected(r) => format!("Rejected({r:?})"),
        other => format!("{other:?}"),
    };
    for _ in 0..16 {
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
    drain(&mut e);
    let d = e.debug_hp(1) - hp_before;
    eprintln!("DBG 虎符：dying={dying} → {tag}；1 号 hp {hp_before} → {}（{d:+}）", e.debug_hp(1));
    (tag, d)
}

/// 上游/原生口径（**两支**）：**濒死 ⇒ 受理并回血**；**未濒死 ⇒ 拒绝**。
///
/// **本轮未验收（登记，含诊断）**：①**未濒死支**已验：**`Rejected(IllegalTargets)`** ✅
/// （与 `play_card.rs:1178` 的 `kind == Hufu && !p.dying` 一致）；
/// ②**濒死支未复现**：以 `debug_deal_damage` 造成**真实伤害**后 1 号 `hp=0`，但
/// **`debug_deal_damage` 返回 `Accepted`（非 `Pending`）且 `询问=[]`** ⇒ **救援窗口未打开**，
/// 虎符**未获提供** ⇒ 需按引擎**濒死流程**构造（可能需先把手牌备好、或经 `NextPhase`/专门救援命令推进）。
/// 用例以 `#[ignore]` 作为修复/夹具补齐后的验收条件（第 52 项）。
#[test]
#[ignore = "虎符濒死支未复现（救援窗口未打开）——未濒死支已验为 Rejected，见第 52 项"]
fn upstream_swd_hufu_dying_only() {
    let (tag_dying, hp_after) = run(20287601, true);
    let (tag_alive, _) = run(20287602, false);
    eprintln!("DBG 虎符：濒死支 → {tag_dying}（hp 至 {hp_after}）；未濒死支 → {tag_alive}");
    assert!(
        tag_dying == "RescueAccepted",
        "虎符（**濒死**）：应在**救援窗口**被提供并可打出（实测 {tag_dying}）"
    );
    assert!(hp_after >= 1, "虎符（濒死）：目标应**回复体力**（实测 hp={hp_after}）");
    assert!(
        tag_alive.starts_with("Rejected"),
        "虎符（**未濒死**）：应被拒绝（`play_card.rs:1178`：`kind == Hufu && !p.dying`）（实测 {tag_alive}）"
    );
}
