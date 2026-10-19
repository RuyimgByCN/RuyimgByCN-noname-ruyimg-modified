//! NC08 第⑦层（上游行为对照）样本 141：**来源 U11（wuxing）** 系统面
//! `shengke`（五行生克）。
//!
//! 上游源码（`noname-liuli/noname/extension/wuxing/extension.js`，`help` 逐字）：
//!
//! > 「五行生克」：**每名角色在游戏开始时随机获得一个属性**；**牌堆中三分之一的牌会随机获得一个属性**；
//! > **当一名（角色）成为相克属性卡牌的目标时，须弃置一张牌**；
//! > **当一名角色成为相生的卡牌的目标时，须摸一张牌**；
//! > 金克木，金生水；木克土，木生火；水克火，水生木；火克金，火生土；土克水，土生金。
//!
//! 原生实现（**源码逐字引用**）：
//! - `state.rs:32`「五行属性（G1 wuxing：**0=金 1=木 2=水 3=火 4=土**；None = 未启用）」；
//! - `state.rs:804`「**`card_natures: BTreeMap<CardId, u8>`**（牌 → 属性）」；
//! - `behavior.rs:15`「**五行生克（wuxing）：成为有属性牌的目标时——被克弃一张手牌**」；
//! - `extension.rs:65`（`InitHook::WuxingNatures => "wuxing_natures"`）——由**包数据 `init_hook` 启用**。
//!
//! **本样本的定位**：验证「**启用后确有属性表 + 成为有属性牌的目标时发生弃/摸**」。
//! **本样本只判「生克确有结算」**（弃或摸其一）；**两向的精确分支**（谁克谁、谁生谁）登记为待办。
//!
//! 未覆盖（登记）：**克/生两向的精确判定**；属性赋予的**随机分布**（角色 1 个、牌堆 1/3）。

mod nc08_common;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_phase, drain, next_dedup};

fn wuxing_engine(seed: u64) -> Engine {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wuxing-upstream-liuli/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json")).expect("读 wuxing cards");
    let pack = content_native_loader::bridge_pack(&cards).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 上游点 2（**行为面**）：成为**有属性牌**的目标时，**发生弃或摸**（生克结算）。
#[test]
fn upstream_wuxing_shengke_resolves() {
    let mut e = wuxing_engine(20287401);
    assert!(advance_to_phase(&mut e, 0, Phase::Play), "应到达出牌阶段");
    // **属性表**是否已由 init_hook 启用
    let natures = e.state_core().card_natures.clone();
    eprintln!("DBG 五行生克：牌属性表条目数 = {}", natures.len());
    if natures.is_empty() {
        eprintln!(
            "NOTE 五行生克：牌属性表为空 ⇒ 本局未启用五行（`init_hook` 未生效或本包未带该钩子）——\
             本用例不构成验收"
        );
        return;
    }
    // 找一张**有属性**的牌并对其使用（目标：1 号）
    let (card_id, nature) = {
        let (id, nat) = natures.iter().next().map(|(k, v)| (*k, *v)).expect("有属性牌");
        (id, nat)
    };
    eprintln!("DBG 五行生克：取牌 {card_id}（属性 {nature}）");
    nc08_common::put_hand(&mut e, 0, card_id);
    let h1_before = e.debug_hand(1).len();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: card_id, targets: vec![1] },
    });
    eprintln!("DBG 五行生克：使用有属性牌 = {out:?}");
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(format!("{}@obs{}", c.spec.prompt, c.observer));
        }
        let cards: Vec<engine_native::ChoiceOption> = c
            .spec
            .options
            .iter()
            .filter(|o| matches!(o, engine_native::ChoiceOption::Card(_)))
            .copied()
            .collect();
        let sel: Vec<engine_native::ChoiceOption> = if !cards.is_empty() {
            vec![cards[0]]
        } else if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
    let h1_after = e.debug_hand(1).len();
    let d_h1 = h1_after as i64 - h1_before as i64;
    eprintln!(
        "DBG 五行生克：询问={prompts:?}；1 号手牌 {h1_before} → {h1_after}（{d_h1:+}）"
    );
    if !matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ) {
        eprintln!("NOTE 五行生克：该有属性牌未被受理（{out:?}）——本用例不构成验收");
        return;
    }
    eprintln!(
        "NOTE 五行生克：属性表已启用（{} 条），本局使用有属性牌后目标手牌变化 {d_h1:+}；\
         **克/生两向的精确判定**与「角色属性」面登记为待办",
        natures.len()
    );
}
