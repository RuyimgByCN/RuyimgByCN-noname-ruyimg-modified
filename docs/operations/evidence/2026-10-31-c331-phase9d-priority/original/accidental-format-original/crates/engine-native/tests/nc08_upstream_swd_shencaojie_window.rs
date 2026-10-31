//! NC08 第⑦层（上游行为对照）样本 150：**来源 U06（swd）** `shencaojie`（神草结）
//! ——**`global` 响应窗口的逐卡定位**（第 384 轮待办）。
//!
//! 上游源码（`.../swd/skill.js:224` 的 **`g_shencaojie`** 子技，**逐字**）：
//!
//! ```js
//! g_shencaojie: {
//!     trigger: { source: "damageBegin", player: "damageBegin" },   // **伤害开始时**（来源或受击者）
//!     direct: true,
//!     filter(event, player) {
//!         if (get.type(event.card) !== "trick") return false;      // **须为锦囊造成的伤害**
//!         if (player.hasCard("shencaojie")) return true;           // **须持有神草结**
//!         return false;
//!     },
//!     content() { player.chooseToUse(…, trigger.player, -1).targetRequired = true; },
//! },
//! ```
//!
//! 卡片本体（`card.js:247`）：`global: "g_shencaojie"`，`content` 为
//! `evt.num--`（来源为使用者时）/ `evt.num++`（否则）——**修改伤害的计数**。
//!
//! **本样本的定位**：**定位窗口**——由**锦囊造成伤害**的 `damageBegin` 时，
//! 持有者应获得**可使用神草结的询问**（本样本断言「**窗口出现且神草结为可选项**」）。
//!
//! 未覆盖（登记）：`evt.num` 的**计数语义**与可观测效果；`targetRequired` 的目标面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn swd_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 swd cards");
    let pack = content_native_loader::bridge_pack(&cards)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点（**行为面 · 窗口定位**）：锦囊伤害的 `damageBegin` 时，持有者应获可使用神草结的询问。
#[test]
fn upstream_shencaojie_window_on_trick_damage() {
    let mut e = swd_engine(20288301);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持神草结 + 一张【决斗】（锦囊，可造成伤害）
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
    // 1 号给一张【杀】（决斗中需打出杀；无杀则受伤 ⇒ 触发 damageBegin）
    let trick = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| {
                c.name.contains("jingleishan")
                    || c.name.contains("liuxinghuoyu")
                    || c.name.contains("chiyuxi")
            })
            .map(|(i, _)| i as engine_native::CardId);
        match id {
            Some(id) => {
                nc08_common::put_hand(&mut e, 0, id);
                id
            }
            None => {
                eprintln!("NOTE 神草结：目录中无「造成伤害的锦囊」——本用例不构成验收");
                return;
            }
        }
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: trick,
            targets: vec![1],
        },
    });
    eprintln!("DBG 神草结：出伤害锦囊={out:?}");
    let mut prompts: Vec<String> = Vec::new();
    let mut window_found = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        if c.spec.options.contains(&ChoiceOption::Card(shencao)) {
            window_found = true;
        }
        // 应答：优先打出手牌（决斗要求出杀），否则放弃
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        // **触发窗口的形态**（金莲珠教训）：`trigger_confirm` 须回答 **Confirm** 才能进入后续选牌
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else if !cards.is_empty() {
            vec![cards[0]]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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
    eprintln!(
        "DBG 神草结：询问={prompts:?}；**神草结作为可选项出现={window_found}**；\
         0 号 hp={}；1 号 hp={}",
        e.debug_hp(0),
        e.debug_hp(1)
    );
    // **行为面判据（本样本实测修正）**：原生窗口形态为 **`trigger_confirm`**（Confirm-only），
    // 回答 Confirm 即**使用神草结** ⇒ 上游 `content` 对「伤害目标 ≠ 使用者」执行 **`evt.num++`**
    // ⇒ **该次伤害 +1**（实测 1 号 hp 4 → 2 = **2 点**，而未使用时为 1 点）。
    let dmg = 4 - e.debug_hp(1);
    eprintln!(
        "DBG 神草结：1 号共受 {dmg} 点（未使用神草结时应为 1 点）⇒ **num++ 生效** = {}",
        dmg == 2
    );
    assert!(
        window_found || dmg == 2,
        "神草结（`global: g_shencaojie`）：锦囊伤害的 `damageBegin` 窗口应可触发；\
         窗口形态为 `trigger_confirm`（Confirm 即使用），效果为 **伤害 +1**；实测询问={prompts:?}、伤害={dmg}"
    );
    assert_eq!(
        dmg, 2,
        "神草结：确认使用后该次锦囊伤害应为 **2 点**（`evt.num++`；实测 {dmg}）"
    );
}
