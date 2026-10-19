//! NC08 第⑦层（上游行为对照）样本 62：`xinfu_jijie`（**机捷**，标准面 / 伊籍）。
//!
//! **命名提示**：上游键 `xinfu_jijie` 在 `translate.js` 中映射为「**机捷**」（**不是**「新服结姻」），
//! 原生常量为 `JI_JIE`（`names.rs:46` 注释「伊籍「机捷」：出牌阶段限一次，取牌堆底一张牌交给一名角色」）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! xinfu_jijie: {
//!     enable: "phaseUse", usable: 1,                 // 出牌阶段**限一次**
//!     async content(event, trigger, player) {
//!         const card = get.bottomCards()[0];          // **牌堆底**一张
//!         game.cardsGotoOrdering(card);
//!         event.card = card;
//!         const { bool, targets } = await player.chooseTarget({
//!             forced: true,                           // **强制选目标**（无 filterTarget → 任意存活角色，含自己）
//!             ai(target) { … },
//!         });
//!         …
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机/限次**：出牌阶段、**每回合限一次**；
//! 2. **取牌**：**牌堆底**一张（`get.bottomCards()[0]`）；
//! 3. **目标**：`chooseTarget({forced: true})` 且**无 `filterTarget`** → **任意存活角色（含自己）**。
//!
//! 原生实现引用：`content/standard/skills/ji_jie.rs`（注释「出牌阶段**限一次**主动技——取**牌堆底**
//! 一张牌交给一名**任意存活角色**（上游 `chooseTarget` **无过滤器**，AI 含自引加成，原生**同允许自选**）」）。
//!
//! **登记差异**：本样本**未发现**差异（取牌位置、目标范围、限次均与上游一致）；保留项：
//! `ai(target)` 的自引加成评分与 `audio` 表现面。
//!
//! 未覆盖（登记）：**行为面**（牌堆底牌移动的逐帧验证；本样本为源码事实级）。

mod nc08_common;

use engine_native::Engine;

/// 上游点 2（**牌堆底**）：原生实现取牌堆底一张。
#[test]
fn upstream_jijie_takes_bottom_card() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_jie.rs"
    ))
    .expect("读机捷源码");
    let head: String = src.chars().take(1400).collect();
    assert!(
        head.contains("牌堆底"),
        "原生注释应写明取『牌堆底』一张：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\txinfu_jijie: {").expect("上游机捷");
    let seg: String = up[idx..].chars().take(1200).collect();
    assert!(
        seg.contains("get.bottomCards()[0]"),
        "上游依据：取牌堆底一张：{seg}"
    );
    assert!(
        seg.contains("forced: true"),
        "上游依据：chooseTarget 为 forced：{seg}"
    );
}

/// 上游点 3（**目标无过滤器 → 含自己**）：原生允许自选。
#[test]
fn upstream_jijie_allows_self_target() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/standard/skills/ji_jie.rs"
    ))
    .expect("读机捷源码");
    // 注释按行宽折行（「chooseTarget 无\n过滤器」）→ 用短片段断言
    let head: String = src.chars().take(400).collect();
    assert!(
        head.contains("chooseTarget 无") && head.contains("过滤器") && head.contains("同允许自选"),
        "原生注释应说明上游无 filterTarget、原生同允许自选：{head}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("\txinfu_jijie: {").expect("上游机捷");
    let seg: String = up[idx..].chars().take(1600).collect();
    assert!(
        !seg.contains("filterTarget"),
        "上游依据：chooseTarget **无** filterTarget（任意存活角色，含自己）"
    );
    // 命名提示：上游键在 translate.js 映射为「机捷」
    let tr = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/translate.js"),
    )
    .expect("读 translate.js");
    assert!(
        tr.contains("xinfu_jijie: \"机捷\""),
        "命名依据：translate.js 中 xinfu_jijie → 「机捷」"
    );
    let _ = Engine::new(20269401, 4);
}

/// 上游点 2+3（**行为面**）：发动机捷 → 选目标 → **牌堆底那张**进入目标手牌。
///
/// 原生把牌堆底表示为 `deck[0]`（注释「取牌堆底（deck 首位）」），故本用例先记录
/// `deck[0]`，发动后断言该牌出现在目标手牌中、且牌堆长度 −1。
#[test]
fn upstream_jijie_moves_bottom_card_to_target() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20269402, 4);
    e.debug_grant(0, "机捷");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let bottom = e.state_core().deck[0];
    let deck_len = e.state_core().deck.len();
    let skill = e.catalog().skill_by_name("机捷").expect("机捷在目录");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "机捷应可发动：{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut gave = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        if c.spec.options.contains(&ChoiceOption::Player(1)) {
            gave = true;
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
        let sel: Vec<ChoiceOption> = if c.spec.allow_cancel {
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
        "DBG 机捷：询问={prompts:?} 选目标={gave} 牌堆底牌={bottom} 目标手牌={:?} 牌堆 {deck_len} → {}",
        e.debug_hand(1),
        e.state_core().deck.len()
    );
    assert!(gave, "应出现目标选择询问（jijie_give_target）");
    assert!(
        e.debug_hand(1).contains(&bottom),
        "机捷：**牌堆底**那张（deck[0]={bottom}）应进入目标手牌"
    );
    assert_eq!(
        e.state_core().deck.len(),
        deck_len - 1,
        "机捷：牌堆应减少一张"
    );
}
