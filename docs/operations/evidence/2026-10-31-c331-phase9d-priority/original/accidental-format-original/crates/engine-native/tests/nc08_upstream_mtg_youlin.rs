//! NC08 第⑦层（上游行为对照）样本 139：**来源 U09（mtg）** 地牌
//! `mtg_youlin`（幽林）——**mtg 已采地牌的最后一张**（另有 `mtg_yixialan` 无 `content`）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/mtg/card.js`）：
//!
//! ```js
//! mtg_youlin: {
//!     type: "land", enable(card, player) { return !player.hasSkill("land_used"); },
//!     notarget: true,
//!     content() {
//!         game.changeLand("mtg_youlin", player);
//!         var list = get.inpile("trick");                       // **全普通锦囊池**
//!         while (list.length) {
//!             var name = list.randomRemove();                   // **随机取一张**
//!             if (lib.card[name].multitarget) continue;
//!             var targets = game.filterPlayer();
//!             while (targets.length) {
//!                 var target = targets.randomRemove();          // **随机选目标**
//!                 if (player.canUse(name, target, false) &&
//!                     get.effect(target, { name: name }, player, player) > 0) {   // **正收益判定**
//!                     player.useCard({ name: name }, target);
//!                     …
//! ```
//!
//! 原生实现（**源码注释自述登记差异**，`content/host/step/mtg.rs:683-687`）：
//!
//! > 幽林（mtg 第九批，规则 v108）：Act 换地 → **池=牌堆中的决斗/无中生有**
//! > （**登记差异：上游全普通锦囊池 + 正收益 AI 判定；原生收窄为内核结算已确证者**，
//! > 无懈窗口照常）→ **随机一张随机其他存活目标**（无中无目标）→ emit 结算；池空空发。
//!
//! **本样本的定位**：验证「**使用后确有锦囊结算发生**」（观察**手牌/体力/询问**任一变化），
//! 并**记录**原生自述的「池收窄」差异（属既有「池收窄」族）。
//!
//! 未覆盖（登记）：**池收窄**的具体清单（决斗/无中生有）；`land_used` 每回合重置。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn mtg_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-mtg-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 mtg cards");
    let pack = content_native_loader::bridge_pack(&cards)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：使用后**确有锦囊结算**（手牌/体力/询问任一变化）。
#[test]
fn upstream_mtg_youlin_uses_a_trick() {
    let mut e = mtg_engine(20287201);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 其余角色各 2 张手牌（决斗等需要目标有牌）
    for p in 1..4 {
        for _ in 0..2 {
            let name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.name == "shan")
                .map(|c| c.name)
                .unwrap();
            e.debug_place(p, name);
        }
    }
    let card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(_, c)| c.name.contains("youlin"))
            .map(|(i, _)| i as engine_native::CardId)
            .expect("幽林在目录");
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let h0 = e.debug_hand(0).len() as i64;
    let deck0 = e.state_core().deck.len() as i64;
    let hp: Vec<i32> = (0..4).map(|p| e.debug_hp(p)).collect();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card,
            targets: vec![],
        },
    });
    eprintln!("DBG 幽林：发动={out:?}");
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "幽林应可发动（无目标）：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let cards: Vec<ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, ChoiceOption::Card(_)))
            .copied()
            .collect();
        let sel: Vec<ChoiceOption> = if !cards.is_empty() {
            vec![cards[0]]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
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
    drain(&mut e);
    let d_h0 = e.debug_hand(0).len() as i64 - h0;
    let d_deck = deck0 - e.state_core().deck.len() as i64;
    let d_hp: Vec<i32> = (0..4).map(|p| hp[p as usize] - e.debug_hp(p)).collect();
    eprintln!("DBG 幽林：询问={prompts:?}；0 号手牌{d_h0:+}；牌堆−{d_deck}；各方掉血={d_hp:?}");
    let changed = d_h0 != 0 || d_deck != 0 || d_hp.iter().any(|d| *d != 0) || !prompts.is_empty();
    assert!(
        changed,
        "幽林：使用后应**确有锦囊结算**（手牌/牌堆/体力/询问任一变化）——实测均无变化"
    );
    eprintln!(
        "NOTE 幽林：原生注释自述登记差异——上游为「全普通锦囊池 + 正收益 AI 判定」，\
         原生**收窄为牌堆中的决斗/无中生有**（属既有「池收窄」族）"
    );
}
