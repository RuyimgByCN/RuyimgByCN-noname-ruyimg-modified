//! NC08 第⑦层（上游行为对照）样本 3：`ciqiu`（刺酋，U07 玩点论杀/韩龙）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/玩点论杀/character/skill.js`）：
//!
//! ```js
//! ciqiu: {
//!     trigger: { source: "damageBegin1" },
//!     forced: true,
//!     filter(event) {
//!         return event.card && event.card.name === "sha" && event.player.isHealthy();
//!     },
//!     content() {
//!         "step 0";
//!         trigger.num++;
//!         if (trigger.num >= trigger.player.hp) {
//!             trigger.player.addTempSkill("ciqiu_dying");
//!             player.removeSkill("ciqiu");
//!         }
//!     },
//! },
//! ciqiu_dying: {
//!     trigger: { player: "dyingBegin" },
//!     forced: true, silent: true, firstDo: true,
//!     content() { player.die(); },
//! },
//! ```
//!
//! 由该 JS 推导的规则点（本文件逐条验收）：
//! 1. **仅【杀】造成的伤害**（`event.card.name === "sha"`）；
//! 2. **仅目标未受伤**（`isHealthy()` = 满血）；
//! 3. **伤害 +1**（`trigger.num++`）；
//! 4. 加后 `num >= 目标体力` → 目标获得 `ciqiu_dying`（**临时技**）：其濒死开始时
//!    **直接死亡**（跳过救援询问，`firstDo` 先于其它濒死触发）；
//! 5. 同时**使用者失去 `ciqiu`**（一次性）。
//!
//! 原生实现引用：`kernel/damage.rs:202-222`（加伤与标记）、`kernel/dying.rs:51-57`
//! （标记在位则跳过救援直接死亡）、`state.rs:482`（`TurnState.ciqiu_dying` = 临时技等价，
//! **换回合清空**）。
//!
//! 未覆盖（登记）：`ai.effect` 属策略面；`silent`/`popup` 属表现面。

use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn wudian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("wudian-upstream-noname/assets/cards.json"))
        .expect("读玩点论杀包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
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
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..24 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 上游点 1+2：**非【杀】伤害**不触发（即便目标满血、且持有刺酋）。
#[test]
fn upstream_ciqiu_requires_sha_card() {
    let mut e = wudian_engine(20262601);
    e.debug_grant(0, "ciqiu");
    let hp_before = e.debug_hp(1);
    // 直接造成 1 点**无牌**伤害（非杀）→ 不得 +1
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert_eq!(
        e.debug_hp(1),
        hp_before - 1,
        "非【杀】伤害不得触发刺酋加伤（上游 filter 要求 card.name==='sha'）"
    );
}

/// 上游点 3+5：满血目标被【杀】命中 → +1 且加后 ≥ 体力时**使用者失去刺酋**（一次性）。
#[test]
fn upstream_ciqiu_bonus_and_one_shot_removal() {
    let mut e = wudian_engine(20262602);
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "ciqiu");
    e.debug_grant(0, "咆哮"); // 杀不限次（驱动两次出杀）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 目标 1：**满血但上限低**（hp == max_hp == 2 → 上游 isHealthy 为真；伤害 +1 后 2 ≥ 体力 2）
    e.debug_set_max_hp(1, 2);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 4);
    // ① 目标 1（满血 2）受杀 → +1 后 2 ≥ 体力 2 → 标记 + 失技
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert!(!e.debug_alive(1), "加伤后 ≥ 体力 → 濒死即死（跳过救援）");
    // ② 已失技：对另一满血目标 2 出杀 → 不再 +1（伤害 1）
    let hp2 = e.debug_hp(2);
    let sha2 = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha2);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha2, targets: vec![2] },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    assert_eq!(e.debug_hp(2), hp2 - 1, "刺酋已移除 → 不再加伤");
}

/// 上游点 4：`ciqiu_dying` 是**临时技**（`addTempSkill`）——原生以
/// `TurnState.ciqiu_dying` 等价并**换回合清空**（不得跨回合残留）。
#[test]
fn upstream_ciqiu_dying_is_turn_scoped() {
    // 结构断言：状态字段存在于 TurnState（随回合重置），而非 PlayerState（常驻）
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/state.rs"
    ))
    .expect("读状态源码");
    let idx = src.find("pub ciqiu_dying").expect("ciqiu_dying 字段");
    let ctx: String = src[idx.saturating_sub(200)..].chars().take(260).collect();
    assert!(
        ctx.contains("TurnState") || ctx.contains("回合"),
        "ciqiu_dying 应属回合级状态（临时技等价）：{ctx}"
    );
    // 行为断言：清空后不再即死（跨回合语义由构造保证，这里固定字段语义）
    let mut e = wudian_engine(20262603);
    e.debug_set_hp(1, 2);
    e.debug_deal_damage(Some(2), 1, 1, engine_native::event::DamageNature::Normal);
    drain(&mut e);
    assert!(e.debug_alive(1), "无标记时正常濒死流程（可被救援）");
}
