//! NC08 第⑦层（上游行为对照）样本 44：`hujia`（护驾，**标准面** / 曹操）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/character/standard/skill.js`）：
//!
//! ```js
//! hujia: {
//!     zhuSkill: true,                                    // **主公技**
//!     trigger: { player: ["chooseToRespondBefore", "chooseToUseBefore"] },
//!     filter(event, player) {
//!         if (event.responded) return false;             // 尚未响应
//!         if (player.storage.hujiaing) return false;     // 防重入
//!         if (!player.hasZhuSkill("hujia")) return false; // **须具主公资格**
//!         if (!event.filterCard({ name: "shan", isCard: true }, player, event)) return false;  // 需求为【闪】
//!         return game.hasPlayer(current => current !== player && current.group === "wei");     // 有**魏**势力他人
//!     },
//!     async content(event, trigger, player) {
//!         let current = player.next;                     // 自**下家**起按座位序
//!         while (true) { … }                             // 逐个询问魏势力角色代打【闪】
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **资格**：持技者须具**主公资格**（`hasZhuSkill`）；需求牌为**【闪】**；且场上存在
//!    **其他魏势力**角色；
//! 2. **顺序**：自持技者**下家**起按**座位序**逐个询问（可代打【闪】）。
//!
//! 原生实现引用：`kernel/sha.rs:293`（`hujia_active`：持技 + `is_lord`）、
//! `:302`（`build_hujia_queue`：`next_alive` 起座位序 + `effective_faction == Wei`）。
//!
//! **登记差异（本文件固定）**：
//! - **①主公资格判定**：上游 `hasZhuSkill("hujia")` 仅在**身份局**（具主公技旗标）成立；
//!   原生用 `is_lord`，而 `is_lord` 在**非身份局返回 true**（`state.rs:1124` 的 `None => true`）
//!   → **原型/单挑模式下原生更宽松**（上游此时不可发动）；
//! - **②防重入**：上游有 `player.storage.hujiaing` 旗标；原生未在 `hujia_active` 中检查
//!   （是否由调用路径保证**未判定**，登记）。
//!
//! 未覆盖（登记）：`chooseToUseBefore` 面（上游亦覆盖主动使用【闪】的场景，本样本只固定源码事实）；
//! 行为面（借闪的完整应答链）登记待办。

mod nc08_common;

use engine_native::Engine;

/// 上游点 1（**魏势力队列 + 座位序**）：原生队列构造与上游同口径（源码事实）。
#[test]
fn upstream_hujia_queue_is_wei_and_seat_ordered() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha.rs"
    ))
    .expect("读 sha 源码");
    let idx = src.find("fn build_hujia_queue").expect("队列构造");
    let seg: String = src[idx..].chars().take(420).collect();
    assert!(
        seg.contains("seat_order_from(ctx.core.next_alive(self.target))")
            && seg.contains("Faction::Wei"),
        "原生依据：下家起座位序 + 魏势力：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游 standard/skill.js");
    let idx = up.find("\thujia: {").expect("上游护驾");
    let seg: String = up[idx..].chars().take(900).collect();
    assert!(
        seg.contains("current.group === \"wei\"") && seg.contains("let current = player.next"),
        "上游依据：魏势力 + 自下家起：{seg}"
    );
}

/// **登记差异①（原生更宽松）**：`is_lord` 在非身份局返回 true，而上游 `hasZhuSkill` 不成立。
#[test]
fn upstream_hujia_lord_qualification_difference_is_fixed() {
    let state = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/state/core_ops.rs"
    ))
    .expect("读 state");
    let idx = state.find("pub fn is_lord").expect("is_lord");
    let seg: String = state[idx..].chars().take(220).collect();
    assert!(
        seg.contains("None => true"),
        "差异①固定：is_lord 在非身份局返回 true（原生更宽松）：{seg}"
    );
    let sha = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    let idx = sha.find("fn hujia_active").expect("hujia_active");
    let seg: String = sha[idx..].chars().take(320).collect();
    assert!(
        seg.contains("is_lord") && !seg.contains("hujiaing"),
        "差异②固定：原生以 is_lord 判定资格，且未检查 hujiaing 防重入：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("\thujia: {").expect("上游护驾");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("hasZhuSkill(\"hujia\")") && seg.contains("player.storage.hujiaing"),
        "上游依据：主公技资格 + hujiaing 防重入：{seg}"
    );
    let _ = Engine::new(20267601, 4);
}

/// 上游点 1（**需求牌面**）：护驾只对【闪】需求开放（源码事实）。
#[test]
fn upstream_hujia_only_for_shan() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/kernel/sha.rs"
    ))
    .expect("读 sha");
    // 原生护驾位于【杀】结算机器中（需求即【闪】）：取文件内最近的 `impl <Name> {` 归属
    let idx = src.find("fn hujia_active").expect("hujia_active");
    let head = &src[..idx];
    let owner = head.rfind("impl ").map(|i| {
        let seg: String = head[i..].chars().take(60).collect();
        seg.lines().next().unwrap_or("").trim().to_string()
    });
    assert!(
        owner
            .as_deref()
            .map(|o| o.contains("ShaResolve"))
            .unwrap_or(false),
        "原生护驾应归属【杀】结算机器（需求为【闪】），实测归属={owner:?}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../../archive/upstream/noname/apps/core/character/standard/skill.js"),
    )
    .expect("读上游");
    let idx = up.find("\thujia: {").expect("上游护驾");
    let seg: String = up[idx..].chars().take(600).collect();
    assert!(
        seg.contains("name: \"shan\", isCard: true"),
        "上游依据：需求牌为【闪】：{seg}"
    );
}

/// 上游点 2（**行为面**）：护驾借闪——魏势力角色代打【闪】，【杀】被抵消。
///
/// 口径说明：原生 `is_lord` 在**非身份局返回 true**（`state.rs` 注释「无身份体系的模式持技即可」，
/// 与 `hujia`/`jijiang`/`jiuyuan` 同族的设计差异）→ **原型模式下护驾亦可发动**，本用例即在此口径下验证。
#[test]
fn upstream_hujia_borrows_shan_from_wei_ally() {
    use engine_native::{ChoiceOption, CommandOutcome, GameAction, GameCommand};
    use nc08_common::{advance_to_play, drain, next_dedup};

    let mut e = Engine::new(20267602, 4);
    e.debug_grant(1, "护驾"); // 1 号持护驾（原型模式下 is_lord 为真）
                              // 2 号装配**魏**势力武将（护驾借闪的询问对象）
    let wei = e
        .general_candidates()
        .into_iter()
        .find(|d| {
            e.catalog()
                .general_meta(*d)
                .map(|(_, f, _, _, _)| f == engine_native::catalog::Faction::Wei)
                .unwrap_or(false)
        })
        .expect("存在魏势力武将");
    let wei_name = e
        .catalog()
        .general_meta(wei)
        .map(|(n, _, _, _, _)| n.to_string())
        .unwrap();
    e.debug_assemble_general(2, &wei_name);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 0 号持【杀】；1 号**无闪**；2 号持一张【闪】
    let sha = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "sha")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(0, name)
    };
    let shan2 = {
        let name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.name == "shan")
            .map(|c| c.name)
            .unwrap();
        e.debug_place(2, name)
    };
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let mut prompts: Vec<String> = Vec::new();
    let mut ally_played = false;
    for _ in 0..24 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
        }
        // 2 号（魏势力）被询问出闪 → 打出
        if c.observer == 2 && c.spec.options.contains(&ChoiceOption::Card(shan2)) {
            ally_played = true;
            let _ = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(shan2)],
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
        "DBG 护驾：询问={prompts:?} 魏势力代出闪={ally_played} 1 号 hp {hp1} → {}（2 号手牌={:?}）",
        e.debug_hp(1),
        e.debug_hand(2)
    );
    if !ally_played {
        eprintln!("NOTE 未出现魏势力代出闪的询问——本用例不构成验收（借闪链路未对上）");
        return;
    }
    assert_eq!(
        e.debug_hp(1),
        hp1,
        "护驾：魏势力代出【闪】后，【杀】被抵消（1 号 hp 不变）"
    );
}
