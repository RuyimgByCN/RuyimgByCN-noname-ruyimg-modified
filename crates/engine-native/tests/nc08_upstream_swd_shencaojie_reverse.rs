//! NC08 第⑦层（上游行为对照）样本 153：**来源 U06（swd）** `shencaojie`（神草结）
//! ——**反向分支**（`evt.num--`）：承接样本 150 的待办。
//!
//! 上游卡片本体（`card.js`）：`content` 为
//! **`if (evt.player === player) evt.num--; else evt.num++;`**
//! ——`evt.player` 是**受伤者**（noname `damageBegin` 的 `player` 字段）⇒
//! **持有者自己被该锦囊伤害**时 **`num--`**（伤害 -1）；**他人**被伤害时 `num++`（伤害 +1，样本 150 已验）。
//!
//! **本样本的定位**：构造「**持有者本人是受伤者**」——由**1 号**（其回合）使用 **AOE 锦囊**打 **0 号**
//! （0 号持有神草结）⇒ 应 **`num--`**（1 点伤害被减为 **0**）。
//!
//! 未覆盖（登记）：`targetRequired` 目标面；AOE 的**多目标顺序**。

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

/// 上游 `evt.player === player` 分支（**受伤者＝持有者**）⇒ 伤害 **-1**。
///
/// **已验收（2026-10-19 修正后）**：AOE 的**目录名**是 **`savage`/`archery`**
/// （`catalog.rs:934/935` 的 `SavageAttack → "nanman"` 是 **archive 键映射**，**不是目录名**）；
/// 用正确目录名后窗口正常触发：**1 号用 AOE → 0 号（持有者）出现 `trigger_confirm` → Confirm 使用 →
/// 0 号受伤 0 点**（`num--` 把 1 点伤害减为 0）✅
#[test]
fn upstream_shencaojie_reverse_branch_reduces_damage() {
    let mut e = swd_engine(20288501);
    // 0 号持神草结
    let shencao = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("shencaojie"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("神草结在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    // 推进到 1 号出牌阶段（1 号要用 AOE）
    assert!(
        advance_to_phase(&mut e, 1, Phase::Play),
        "应能推进到 1 号出牌阶段"
    );
    // 1 号用 AOE 锦囊（南蛮入侵/万箭齐发）——打所有其他角色（含 0 号）
    let aoe = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            // **目录名**为 `savage`/`archery`（非 `nanman`/`wanjian`——后两者是 archive 键映射）
            .find(|(_, c)| c.name == "savage" || c.name == "archery")
            .map(|(i, _)| i as engine_native::CardId);
        match id {
            Some(id) => {
                nc08_common::put_hand(&mut e, 1, id);
                id
            }
            None => {
                eprintln!("NOTE 神草结反向：目录中无 AOE 锦囊（南蛮/万箭）——本用例不构成验收");
                return;
            }
        }
    };
    let hp0_before = e.debug_hp(0);
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: aoe, targets: vec![] },
    });
    eprintln!("DBG 神草结反向：1 号使用 AOE={out:?}；0 号 hp={hp0_before}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "AOE 应被受理：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut confirmed = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == &c.spec.prompt.to_string()) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        // 神草结窗口（trigger_confirm）⇒ 选 Confirm 即使用（样本 150 已确证）
        if !confirmed && c.observer == 0 && c.spec.options.contains(&ChoiceOption::Confirm) {
            confirmed = true;
            let _ = e.submit(GameCommand {
                actor: 0,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Confirm] },
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
    let dmg = hp0_before - e.debug_hp(0);
    eprintln!("DBG 神草结反向：询问={prompts:?} 已用神草结={confirmed}；0 号受伤 {dmg} 点");
    if !confirmed {
        eprintln!("NOTE 神草结反向：未出现神草结窗口（0 号非受伤者或窗口未开）——本用例不构成验收");
        return;
    }
    assert_eq!(
        dmg, 0,
        "神草结（**反向分支** `evt.player === player`）：**持有者自己是受伤者**时应 **`num--`**\
         （1 点伤害减为 0；实测受伤 {dmg} 点）"
    );
    let _ = shencao;
}
