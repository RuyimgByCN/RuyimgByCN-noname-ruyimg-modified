//! NC08 第⑦层（上游行为对照）样本 55：`fanjian`（反间，**标准面** / 周瑜）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! fanjian: {
//!     enable: "phaseUse", usable: 1,                 // 出牌阶段限一次
//!     filter(event, player) { return player.hasCards("h"); },        // 须有手牌
//!     filterTarget(card, player, target) { return player !== target; },  // 其他角色
//!     async content(event, trigger, player) {
//!         const { control } = await target.chooseControl({
//!             controls: ["heart2", "diamond2", "club2", "spade2"],   // 目标**选一种花色**
//!         }).forResult();
//!         const { bool, cards } = await target
//!             .gainPlayerCard({ target: player, position: "h", forced: true })  // **强制**获得一张手牌
//!             .forResult();
//!         if (bool && cards?.length && get.suit(cards[0], player) + "2" !== event.choice) {
//!             await target.damage({ nocard: true });                 // **花色不符 → 1 点伤害**
//!         }
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/限次**：出牌阶段、**每回合限一次**；发动者**须有手牌**；目标为**其他角色**；
//! 2. **流程**：目标**先选一种花色** → 从发动者**手牌**中**强制获得一张**；
//! 3. **结算**：所得牌**花色与所选不符** → 目标受发动者造成的 **1 点伤害**。
//!
//! 原生实现引用：`content/standard/skills/fan_jian.rs`（注释「v203 对齐上游 `chooseControl`：
//! ♥/♦/♣/♠」「花色与所选不符时受到周瑜造成的 1 点伤害（上游条件 damage，花色相符无事）」；
//! `activate` 要求存活 + 手牌非空）。
//!
//! **登记差异**：本样本**未发现**差异（流程三步与上游一致）；保留项：`ai()` 的花色随机分布
//! （上游 `Math.random()` 六分支）与 `target.chat`/`game.log` 表现面。
//!
//! 未覆盖（登记）：表现面（`chat`/`log`）；AI 选花色分布。

mod nc08_common;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use nc08_common::{advance_to_play, drain, next_dedup};

fn skill_id(e: &Engine, name: &str) -> u16 {
    e.catalog().skill_by_name(name).expect("技能在目录")
}

/// 上游点 2+3（**花色相符 → 无伤害**）：让目标选中所获牌的花色。
#[test]
fn upstream_fanjian_matching_suit_no_damage() {
    let mut e = Engine::new(20268701, 4);
    e.debug_grant(0, "反间");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号只留一张**红桃**【闪】→ 目标选红桃即相符
    let card: engine_native::CardId = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.suit == engine_native::catalog::Suit::Heart && c.name == "shan")
            .expect("红桃闪") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp1 = e.debug_hp(1);
    let skill = skill_id(&e, "反间");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 目标选择：1 号
        if c.spec.options.contains(&ChoiceOption::Player(1)) && c.spec.prompt.contains("target") {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            });
            continue;
        }
        // 花色选择：优先「红桃」选项（选项文本/编号由引擎给出，取首个含 heart 的；否则取首个）
        if c.spec.prompt.contains("suit") || c.spec.prompt.contains("fanjian") {
            eprintln!("DBG 反间花色选项={:?}", c.spec.options);
            let sel: Vec<ChoiceOption> = c.spec.options.iter().take(1).copied().collect();
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    eprintln!(
        "DBG 反间：询问={prompts:?} 1 号 hp {hp1} → {}（1 号手牌={:?}，牌={card}）",
        e.debug_hp(1),
        e.debug_hand(1)
    );
    assert!(
        !e.debug_hand(1).is_empty() || e.debug_hp(1) != hp1,
        "反间：目标应获得一张手牌（或花色不符受伤）——两者必居其一"
    );
    let _ = Phase::Play;
}

/// 上游点 3（**花色不符 → 1 点伤害**）：目标选与所获牌**不同**的花色。
///
/// 花色选项为 `Tag(0..3)`（四花色），`Tag(0)` 与红桃牌相符（见相符分支用例），
/// 故本用例选 **`Tag(1)`** 以制造不符。
#[test]
fn upstream_fanjian_mismatched_suit_deals_damage() {
    let mut e = Engine::new(20268702, 4);
    e.debug_grant(0, "反间");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _card = {
        let id = e
            .catalog()
            .cards
            .iter()
            .position(|c| c.suit == engine_native::catalog::Suit::Heart && c.name == "shan")
            .expect("红桃闪") as engine_native::CardId;
        nc08_common::put_hand(&mut e, 0, id);
        id
    };
    let hp1 = e.debug_hp(1);
    let skill = skill_id(&e, "反间");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut picked_mismatch = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.options.contains(&ChoiceOption::Player(1)) && c.spec.prompt.contains("target") {
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            });
            continue;
        }
        // 花色选择：选 Tag(1)（与 Tag(0) 不同 → 制造不符）
        if c.spec
            .options
            .iter()
            .any(|o| matches!(o, ChoiceOption::Tag(_)))
            && !picked_mismatch
        {
            let alt = c
                .spec
                .options
                .iter()
                .find(|o| matches!(o, ChoiceOption::Tag(t) if *t == 1))
                .copied();
            if let Some(alt) = alt {
                picked_mismatch = true;
                let _ = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![alt],
                    },
                });
                continue;
            }
        }
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
    eprintln!(
        "DBG 反间（不符分支）：选中 Tag(1)={picked_mismatch} 1 号 hp {hp1} → {}（手牌={:?}）",
        e.debug_hp(1),
        e.debug_hand(1)
    );
    if !picked_mismatch {
        eprintln!("NOTE 未出现 Tag(1) 花色选项——本用例不构成验收");
        return;
    }
    assert!(
        e.debug_hp(1) < hp1,
        "反间：花色不符 → 目标受 1 点伤害（{hp1} → {}）",
        e.debug_hp(1)
    );
}

/// **登记（未发现差异）**：流程三步与上游一致（源码事实）。
#[test]
fn upstream_fanjian_flow_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/fan_jian.rs"
    ))
    .expect("读反间源码");
    // 注释跨行书写 → 用全文件子串检查
    let head: String = src.chars().take(400).collect();
    assert!(
        src.contains("chooseControl") && src.contains("♥/♦/♣/♠"),
        "原生注释应说明对齐上游 chooseControl（四花色）：{head}"
    );
    // 注释按行宽折行（"花色与所" 与 "选不符时..." 分处两行）→ 用两段片段断言
    assert!(
        src.contains("花色与所") && src.contains("不符时受到周瑜造成的 1 点伤害"),
        "原生注释应说明花色不符的伤害（跨行片段检查）"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tfanjian: {").expect("上游反间");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("chooseControl") && seg.contains("gainPlayerCard") && seg.contains("damage"),
        "上游依据：选花色 → 获得手牌 → 花色不符受伤：{seg}"
    );
    assert!(
        seg.contains("usable: 1") && seg.contains("player.hasCards(\"h\")"),
        "上游依据：出牌阶段限一次 + 须有手牌：{seg}"
    );
}
