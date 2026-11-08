//! NC08 第⑦层（上游行为对照）样本 58：`jijiang`（激将，**标准面** / 刘备）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! jijiang: {
//!     group: ["jijiang1"], zhuSkill: true,                    // **主公技**
//!     filter(event, player) {
//!         if (!player.hasZhuSkill("jijiang")                      // **须具主公资格**
//!             || !game.hasPlayer(c => c !== player && c.group === "shu")) return false;  // 有蜀势力他人
//!         return !event.jijiang && (event.type !== "phase" || !player.hasSkill("jijiang3"));
//!     },
//!     enable: ["chooseToUse", "chooseToRespond"],
//!     viewAs: { name: "sha" },
//!     filterCard() { return false; },                         // **不用自己的牌**
//!     selectCard: -1,
//!     ai: { respondSha: true, … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **资格**：持技者须具**主公资格**（`hasZhuSkill`），且场上存在**其他蜀势力**角色；
//! 2. **用法**：**不使用自己的牌**（`filterCard → false`、`selectCard: -1`），由**蜀势力角色代打**
//!    【杀】；可用于**主动使用**与**响应**（`chooseToUse` / `chooseToRespond`）。
//!
//! 原生实现引用：`content/standard/skills/jijiang.rs`（`activate` 要求 `is_lord(owner)`
//! —— 注释「主公资格（**身份局须主公身份**）+ 存在蜀势力角色；**不以暗手牌预筛**——有无杀由
//! 被询问者逐人应答时才知道」；机器 `HelperAsk` → …）。
//!
//! **登记差异（与 `hujia` 同族）**：**主公技资格判定**——上游 `hasZhuSkill` **仅身份局**成立；
//! 原生用 `is_lord`，而其在**非身份局返回 true**（`state.rs` 的 `None => true`，注释「无身份体系的
//! 模式持技即可」）→ **原型模式下激将亦可发动**（**已核对为有意的设计差异**，见修复清单第 17 项）。
//!
//! 未覆盖（登记）：`jijiang1` 子技（代打询问队列）的**行为面**（需构造「需要【杀】」的场景）；
//! `jijiang3` 相关分支；`audio` 表现面。

mod nc08_common;

use engine_native::Engine;

/// 上游点 1（**资格**）：原生 `activate` 的主公资格 + 蜀势力判定与上游 filter 对应（源码事实）。
#[test]
fn upstream_jijiang_qualification_matches() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/jijiang.rs"
    ))
    .expect("读激将源码");
    let head: String = src.chars().take(1600).collect();
    assert!(
        head.contains("is_lord(owner)") && head.contains("Faction::Shu"),
        "原生依据：主公资格 + 蜀势力判定：{head}"
    );
    assert!(
        head.contains("不以暗手牌预筛"),
        "原生注释：不以暗手牌预筛（与上游一致——有无杀由应答时才知道）"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\tjijiang: {").expect("上游激将");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("hasZhuSkill(\"jijiang\")") && seg.contains("current.group === \"shu\""),
        "上游依据：主公技资格 + 其他蜀势力：{seg}"
    );
    assert!(
        seg.contains("filterCard() {") && seg.contains("selectCard: -1"),
        "上游依据：不用自己的牌（filterCard false / selectCard -1）：{seg}"
    );
}

/// 上游点 2（**不用自己的牌**）：原生以「代打」形态实现（无手牌也可请求）。
#[test]
fn upstream_jijiang_uses_helpers_not_own_cards() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/standard/skills/jijiang.rs"
    ))
    .expect("读激将源码");
    assert!(
        src.contains("HelperAsk"),
        "原生依据：以『代打（HelperAsk）』形态实现，不使用自己的牌"
    );
    // 与「需要自己的牌」的主动技对比：激将的 activate 不检查手牌
    let idx = src.find("fn activate").expect("activate");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        !seg.contains("hand.is_empty"),
        "差异对照：激将 activate **不**要求手牌非空（上游 filterCard 恒 false）：{seg}"
    );
}

/// **登记差异（与 `hujia` 同族）**：主公技资格判定在非身份局为真（有意的设计差异）。
#[test]
fn upstream_jijiang_lord_gate_is_design_difference() {
    let state = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/state/core_ops.rs"
    ))
    .expect("读 state");
    let idx = state.find("pub fn is_lord").expect("is_lord");
    let seg: String = state[..idx].chars().rev().take(200).collect::<String>().chars().rev().collect();
    assert!(
        seg.contains("无身份体系的模式持技即可"),
        "登记：is_lord 的文档注释说明『无身份体系的模式持技即可』（有意设计）：{seg}"
    );
    // 与护驾同族：两处均用 is_lord 作主公技资格
    let hujia = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    assert!(
        hujia.contains("is_lord(self.target)"),
        "同族对照：护驾亦用 is_lord 作主公技资格"
    );
    let _ = Engine::new(20269001, 4);
}

/// 上游点 2（**行为面 · 代打**）：决斗中刘备（持激将）无【杀】→ 蜀势力角色**代打【杀】**。
///
/// 口径：原型模式下 `is_lord` 为真（同族设计差异）→ 本用例在该口径下验证「蜀势力代打【杀】」。
///
/// **本轮未验收（登记，含诊断）**：以「决斗中 1 号（持激将、无杀）需出杀」构造场景，实测
/// `询问=[]`、`蜀势力代打=false`、`0 号 hp 4 → 4`（决斗未产生任何询问与伤害）→ **代打链路未对上**，
/// 疑与「决斗的杀响应是否走激将入口」有关 → **根因待查**；用例以 `#[ignore]` 守护，**不写成通过**。
#[test]
fn upstream_jijiang_shu_ally_plays_sha_for_lord() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269002, 4);
    e.debug_grant(1, "激将"); // 1 号持激将（需要【杀】的一方）
    // 2 号装配**蜀**势力武将并持一张【杀】
    let shu = e
        .general_candidates()
        .into_iter()
        .find(|d| {
            e.catalog()
                .general_meta(*d)
                .map(|(_, f, _, _, _)| f == engine_native::catalog::Faction::Shu)
                .unwrap_or(false)
        })
        .expect("存在蜀势力武将");
    let shu_name = e.catalog().general_meta(shu).map(|(n, _, _, _, _)| n.to_string()).unwrap();
    e.debug_assemble_general(2, &shu_name);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持【决斗】；1 号**无杀**；2 号持【杀】
    let duel = {
        let name = e.catalog().cards.iter().find(|c| c.name == "duel").map(|c| c.name).unwrap();
        e.debug_place(0, name)
    };
    let sha2 = {
        let name = e.catalog().cards.iter().find(|c| c.name == "sha").map(|c| c.name).unwrap();
        e.debug_place(2, name)
    };
    let hp0 = e.debug_hp(0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: duel, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut ally_played = false;
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 2 号（蜀势力）被询问代打【杀】→ 打出
        if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(sha2)) {
            ally_played = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(sha2)],
                },
            });
            continue;
        }
        let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if c.spec.options.contains(&ChoiceOption::Pass) {
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
        "DBG 激将：询问={prompts:?} 蜀势力代打={ally_played} 0 号 hp {hp0} → {}（2 号手牌={:?}）",
        e.debug_hp(0),
        e.debug_hand(2)
    );
    assert!(
        ally_played,
        "激将：应出现蜀势力 2 号代打【杀】询问并打出（实际询问={prompts:?}）"
    );
    // 2 号代打【杀】后，1 号方已出杀 → 轮到 0 号出杀；0 号无杀 → **0 号受伤**
    assert!(
        e.debug_hp(0) < hp0,
        "激将：蜀势力代打【杀】后，0 号无杀应对 → 0 号应受伤（{hp0} → {}）",
        e.debug_hp(0)
    );
}

/// 激将（决斗多蜀将逐人询问 + 存档恢复）：首位蜀将（2 号）选 Pass 后，
/// 不应直接结算主公受伤，而应继续询问下一位蜀将（3 号），且挂起态可跨存档恢复。
#[test]
fn upstream_jijiang_duel_multi_ally_pass_then_second_ally_plays_sha_and_restores() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269003, 4);
    e.debug_grant(1, "激将");
    let shu_names: Vec<String> = e
        .general_candidates()
        .into_iter()
        .filter_map(|d| {
            e.catalog().general_meta(d).and_then(|(n, f, _, _, _)| {
                (f == engine_native::catalog::Faction::Shu).then(|| n.to_string())
            })
        })
        .take(2)
        .collect();
    assert_eq!(shu_names.len(), 2, "应至少有 2 名蜀势力候选武将");
    e.debug_assemble_general(2, &shu_names[0]);
    e.debug_assemble_general(3, &shu_names[1]);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let duel = e.debug_place(0, "duel");
    let _sha2 = e.debug_place(2, "sha");
    let sha3 = e.debug_place(3, "sha");
    let hp0 = e.debug_hp(0);
    let hp1 = e.debug_hp(1);

    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: duel,
            targets: vec![1],
        },
    });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");

    // 首位蜀将（2 号）被询问，先验证存档恢复等价，再选 Pass
    let c2 = e.open_choice().cloned().expect("应先询问 2 号蜀将");
    assert_eq!(c2.observer, 2, "首位被询问者应为 2 号");
    let snap2 = e.save();
    let mut e = Engine::restore(&snap2).expect("恢复 2 号挂起态");
    let _ = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c2.id,
            selection: vec![ChoiceOption::Pass],
        },
    });

    // 2 号 Pass 后，1 号不应受伤，应继续询问下一位蜀将（3 号）
    assert_eq!(e.debug_hp(1), hp1, "2 号放弃后主公尚未受伤，应继续问 3 号");
    let c3 = e.open_choice().cloned().expect("应继续询问 3 号蜀将");
    assert_eq!(c3.observer, 3, "第二位被询问者应为 3 号");
    assert!(c3.spec.options.contains(&ChoiceOption::Card(sha3)));
    let snap3 = e.save();
    let mut e = Engine::restore(&snap3).expect("恢复 3 号挂起态");
    let _ = e.submit(GameCommand {
        actor: 3,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c3.id,
            selection: vec![ChoiceOption::Card(sha3)],
        },
    });
    drain(&mut e);

    assert_eq!(e.debug_hp(1), hp1, "3 号代打【杀】后主公不应受伤");
    assert_eq!(e.debug_hp(0), hp0 - 1, "0 号无杀应对，应受 1 点伤害");
}
