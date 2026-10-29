//! NC08 第⑦层（上游行为对照）样本 61：`qingguo`（倾国，**标准面** / 甄姬）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! qingguo: {
//!     viewAs: { name: "shan" },                        // 当【闪】使用/打出
//!     viewAsFilter(player) {
//!         if (!player.hasCards("hs", { color: "black" })) return false;   // 黑色牌（hs）
//!     },
//!     position: "hs",                                  // **手牌 + 装备区**
//!     prompt: "将一张黑色手牌当闪使用或打出",
//!     ai: { order: 3, respondShan: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **转化**：**黑色**牌当【闪】**使用或打出**（`viewAs`）；
//! 2. **牌区**：**`hs`（手牌 + 装备区）**；
//! 3. 与龙胆（闪当杀/杀当闪）**并列**出现在【闪】响应候选中。
//!
//! 原生实现引用：`kernel/sha/step.rs:374`/`:789`/`:1195`（倾国并入【闪】响应选项；
//! `:1192` 注释「倾国（甄姬，**v202 对齐上游 `position: "hs"`**）：黑色手牌可当闪打出」）。
//!
//! **登记差异（牌区族第 6 例）**：原生候选**只遍历 `.hand`**（`ctx.core.player(target).hand.iter()`）
//! → **装备区的黑色牌不能当【闪】**；上游 `position: "hs"` **含装备区** → **规则损失**
//! （与 `zhiheng`/`qixi`/`guose`/`wusheng`/`lijian` **同族**）。
//!
//! 未覆盖（登记）：**行为面**（黑色手牌当闪的完整响应链；本样本为源码事实级）；
//! `ai.order`/`respondShan` 策略面。

mod nc08_common;

use engine_native::Engine;

/// 上游点 1+3（**源码事实**）：倾国并入【闪】响应候选，且与龙胆并列。
#[test]
fn upstream_qingguo_in_shan_options() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha/resume.rs"
    ))
    .expect("读 sha/step");
    // 全文件检查（同文件有多个候选构造点，固定窗口会命中其它处）
    let seg: &str = &src;
    assert!(
        seg.contains("qingguo && black") && seg.contains("longdan && kind.is_sha_like()"),
        "原生依据：倾国（黑色）与龙胆并列进入【闪】候选：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tqingguo: {").expect("上游倾国");
    let seg: String = up[idx..].chars().take(1500).collect();
    assert!(
        seg.contains("viewAs: { name: \"shan\" }") && seg.contains("get.color(card)"),
        "上游依据：黑色牌当闪：{seg}"
    );
    assert!(
        seg.contains("position: \"hs\""),
        "上游依据：position 为 hs（含装备区）：{seg}"
    );
}

/// **登记差异（牌区族第 6 例）**：原生只遍历手牌，未含装备区。
#[test]
fn upstream_qingguo_zone_difference_is_registered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/kernel/sha/resume.rs"
    ))
    .expect("读 sha/step");
    // 定位到含 `qingguo && black` 的候选构造点（其上方即为 `.hand` 遍历）
    let anchor = src.find("qingguo && black").expect("倾国候选");
    let start = src[..anchor].rfind(".hand").expect("手牌遍历");
    let seg: String = src[start.saturating_sub(40)..anchor].chars().take(400).collect();
    assert!(
        seg.contains(".hand") && !seg.contains(".equips"),
        "差异固定：原生候选只取手牌（.hand），未含装备区（上游 position 'hs'）：{seg}"
    );
    // 与既有牌区族成员对照：guose/qixi 等在 play_card.rs 的命令面同样限定手牌
    let play_card = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_rules.rs"
    ))
    .expect("读 play_card");
    assert!(
        play_card.contains("hand.contains(&card)"),
        "同族对照：出牌命令面亦限定手牌（play_card.rs）"
    );
    let _ = Engine::new(20269301, 4);
}

/// 上游点 1（**行为面**）：黑色手牌可当【闪】打出 → 响应后不受伤害。
#[test]
fn upstream_qingguo_black_hand_card_responds_as_shan() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269302, 4);
    e.debug_grant(1, "倾国");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持【杀】；1 号持一张**黑色非【闪】**牌（如黑色【过河拆桥】——出牌阶段不可当闪用）
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let black_card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                ) && c.name == "dismantle"
            })
            .expect("黑色过河拆桥") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut used_black = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.prompt == "respond_shan" && c.spec.options.contains(&ChoiceOption::Card(black_card)) {
            used_black = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(black_card)],
                },
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
    eprintln!(
        "DBG 倾国：询问={prompts:?} 黑色手牌入候选={used_black} 1 号 hp {hp1} → {}",
        e.debug_hp(1)
    );
    assert!(used_black, "倾国：黑色手牌应出现在 respond_shan 候选中");
    assert_eq!(
        e.debug_hp(1),
        hp1,
        "倾国：黑色手牌当【闪】打出后，【杀】被抵消（hp 不变）"
    );
}

/// **登记差异的行为面确认**：**装备区**的黑色牌**不**出现在【闪】候选中（上游 `position: "hs"`）。
#[test]
fn upstream_qingguo_equip_zone_absent_from_options() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269303, 4);
    e.debug_grant(1, "倾国");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    // 1 号**装备区**放一张黑色牌（无手牌 → 只有装备区的黑色牌可当闪）
    let equip_black = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| {
                matches!(
                    c.suit,
                    engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
                ) && c.name.contains("armor")
            })
            .expect("黑色防具") as engine_native::CardId;
        nc08_common::put_equips(&mut e, 1, id);
        id
    };
    // 另给 1 号一张**红桃**手牌：确保出现 respond_shan 询问（否则无牌可响应时引擎不询问）
    let _red_hand = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.suit == engine_native::catalog::Suit::Heart && c.name == "shan")
            .expect("红桃闪") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 1, id);
        id
    };
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let mut offered_equip = false;
    let mut saw_shan_prompt = false;
    let hp1 = e.debug_hp(1);
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "respond_shan" {
            saw_shan_prompt = true;
            offered_equip = c.spec.options.contains(&ChoiceOption::Card(equip_black));
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
    eprintln!(
        "DBG 倾国（装备区）：出现 respond_shan={saw_shan_prompt} 装备区黑色牌入候选={offered_equip} 1 号 hp {hp1} → {}",
        e.debug_hp(1)
    );
    assert!(
        !offered_equip,
        "登记差异的行为面确认：**装备区**黑色牌**不**在【闪】候选中（上游 position 'hs' 应含）"
    );
    let _ = hp1;
}
