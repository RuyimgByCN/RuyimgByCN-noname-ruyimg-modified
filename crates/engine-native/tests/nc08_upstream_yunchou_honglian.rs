//! NC08 第⑦层（上游行为对照）样本 75：**来源 U05（杀海拾遗·运筹）** 武将技
//! `honglian`（红莲）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`，`honglian`）：
//!
//! ```js
//! honglian: {
//!     trigger: { player: "damageEnd" },                       // **自己受到伤害后**
//!     check(event, player) { return get.attitude(player, event.player) < 0; },
//!     filter(event, player) {
//!         return event.source && event.source !== player
//!             && event.source.countCards("he", { color: "red" }) > 0;   // **来源（非自己）有红牌**
//!     },
//!     content() {
//!         trigger.source.discard(trigger.source.getCards("he", { color: "red" }));   // **来源弃全部红牌**
//!     },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **时机**：自己**受到伤害后**（非锁定 → 可选择发动）；
//! 2. **门槛**：来源**非自己**且其**手牌+装备区**至少有一张**红牌**；
//! 3. **效果**：**来源弃置其手牌与装备区的全部红牌**（黑牌不受影响）。
//!
//! 原生实现引用：`content/host/step/yunchou.rs:321`（来源弃其手牌与装备区全部红牌，
//! 装备走 `revoke`）；过滤 `content/host/filtering/common.rs`（`Damage` 且目标为自己、
//! 来源非自己且有红牌）。
//!
//! **登记差异**：本样本**未发现**差异（红牌全弃、黑牌保留、无红牌不触发均与上游一致）；
//! 保留项：`ai.expose`/`ai.result` 策略面；**装备区红牌**分支（本样本只覆盖手牌）。
//!
//! 未覆盖（登记）：装备区红牌的弃置与装备技撤销；`check`（AI 是否发动）策略面。

mod nc08_common;
use engine_native::{ChoiceOption, Engine, GameAction, GameCommand};
use nc08_common::{advance_to_play, drain, next_dedup};

/// 运筹帷幄：**双产物桥接**（武将面在 `character.json`）。
fn yunchou_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-yunchou-upstream-noname/assets/character.json"))
            .expect("读运筹 character.json");
    let mut pack = content_native_loader::bridge_pack(&json).expect("桥接 pack 段").pack;
    let bridged = content_native_loader::bridge_generals(&json).expect("桥接 generals 段");
    pack.generals = bridged.generals;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

/// 跑一次「0 号（来源）打 1 号（持红莲）」，返回 `(来源手牌, 询问序列, 布阵红牌, 布阵黑牌)`。
fn run(seed: u64, reds: usize, blacks: usize) -> (Vec<engine_native::CardId>, Vec<String>, Vec<engine_native::CardId>, Vec<engine_native::CardId>) {
    let mut e = yunchou_engine(seed);
    e.debug_grant(1, "ycnhonglian");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // **先收集 id 再布阵**（避免同时可变/不可变借用）
    let red_ids: Vec<engine_native::CardId> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            matches!(
                c.suit,
                engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
            )
        })
        .map(|(i, _)| i as engine_native::CardId)
        .take(reds)
        .collect();
    let black_ids: Vec<engine_native::CardId> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| {
            matches!(
                c.suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            )
        })
        .map(|(i, _)| i as engine_native::CardId)
        .take(blacks)
        .collect();
    for id in red_ids.iter().chain(black_ids.iter()) {
        nc08_common::put_hand(&mut e, 0, *id);
    }
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let mut prompts: Vec<String> = Vec::new();
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if !prompts.iter().any(|p| p == c.spec.prompt) {
            prompts.push(c.spec.prompt.to_string());
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
    (e.debug_hand(0), prompts, red_ids, black_ids)
}

/// 上游点 2+3（**行为面**）：来源红牌全弃、黑牌保留。
#[test]
fn upstream_honglian_source_discards_all_red() {
    let (hand, prompts, reds, blacks) = run(20270701, 2, 1);
    eprintln!("DBG 红莲：询问={prompts:?} 来源手牌={hand:?}（红={reds:?} 黑={blacks:?}）");
    assert!(
        !reds.iter().any(|r| hand.contains(r)),
        "红莲：来源的**全部红牌**应被弃置（手牌={hand:?}）"
    );
    assert!(
        blacks.iter().all(|b| hand.contains(b)),
        "红莲：来源的**黑牌应保留**（只弃红牌）"
    );
}

/// 上游点 2（**反例**）：来源**无红牌** → **不触发**。
#[test]
fn upstream_honglian_no_red_no_trigger() {
    let (hand, prompts, reds, blacks) = run(20270702, 0, 1);
    eprintln!(
        "DBG 红莲（无红牌）：询问={prompts:?} 来源手牌={hand:?}（红={} 黑={}）",
        reds.len(),
        blacks.len()
    );
    assert!(
        !prompts.iter().any(|p| p.contains("honglian")),
        "红莲：来源无红牌时**不应**出现红莲询问，实测 {prompts:?}"
    );
    assert!(
        blacks.iter().all(|b| hand.contains(b)),
        "红莲：不触发时来源手牌应不变"
    );
}

/// 上游点 1+2+3（**源码事实**）。
#[test]
fn upstream_honglian_source_matches() {
    let step = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/yunchou.rs"
    ))
    .expect("读 yunchou");
    let idx = step.find("YunchouHonglian =>").expect("红莲分支");
    let seg: String = step[idx..].chars().take(900).collect();
    assert!(
        seg.contains("revoke") || seg.contains("equips"),
        "原生依据：装备区红牌同弃（revoke）：{seg}"
    );
    let up = std::fs::read_to_string(
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
            "../../../archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js",
        ),
    )
    .expect("读上游 yunchou/skill.js");
    let idx = up.find("\thonglian: {").expect("上游红莲");
    let seg: String = up[idx..].chars().take(700).collect();
    assert!(
        seg.contains("countCards(\"he\", { color: \"red\" }) > 0")
            && seg.contains("trigger.source.discard(trigger.source.getCards(\"he\", { color: \"red\" }))"),
        "上游依据：来源有红牌 + 来源弃全部红牌：{seg}"
    );
}
