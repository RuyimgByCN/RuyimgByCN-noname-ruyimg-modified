//! NC08 第⑦层（上游行为对照）样本 11：`chunbing`（春饼，U05 古剑卡牌子包）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/gujian/card.js`）：
//!
//! ```js
//! chunbing: {
//!     fullskin: true,
//!     type: "food",
//!     enable: true,
//!     filterTarget(card, player, target) {
//!         return !target.hasSkill("chunbing");     // 目标未持有该食物技能
//!     },
//!     content() {
//!         target.$gain2(cards);                    // 目标**获得**本体
//!         target.storage.chunbing = card;
//!         target.storage.chunbing_markcount = 5;   // 5 个标记
//!         target.addSkill("chunbing");             // 授予监听技
//!     },
//!     ai: { … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **目标门槛 = 未持有该食物技能**（`!target.hasSkill("chunbing")`）；
//! 2. **授予监听技实例**，其**计数为 5**（`chunbing_markcount = 5`）；
//! 3. **本体归属**：上游 `target.$gain2(cards)` → 目标**获得**本体。
//!
//! 原生实现引用：`kernel/intrinsic.rs:244`（食物牌 → `GujianFoodGrant`）、
//! `content/host/step/gujian.rs:197`（授予监听技实例 + 弃置本体）、
//! `content/host/constants.rs:12`（`GJ_FOOD_ROUNDS_CHUNBING = 5`）。
//!
//! **登记差异（本文件固定）**：原生把上游的「标记数」建模为监听技的 **`rounds`（剩余回合）**，
//! 且**本体入弃牌堆**（上游为目标获得）——两处均以断言固定，不写成对齐。
//!
//! 未覆盖（登记）：`fullskin`/`ai` 属表现与策略面；监听技自身的触发逻辑另有批次覆盖。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn gujian_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-gujian-upstream-noname/assets/cards.json"))
            .expect("读古剑包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        // 目录名可能带命名空间/花色点数前后缀 → 用 contains 匹配
        .find(|n| n.contains(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
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

/// 上游点 2+3（含**登记差异**）：使用春饼 → 目标获得监听技实例（计数 5 = rounds 5）；
/// **本体入弃牌堆**（上游为目标获得本体）。
#[test]
fn upstream_chunbing_grants_listener_with_count_five() {
    let mut e = gujian_engine(20263601);
    advance_to_play(&mut e, 0);
    let chunbing = place_by_prefix(&mut e, "chunbing", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: chunbing,
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
    drain(&mut e);
    // 授予的监听技实例：rounds == 5（上游 markcount = 5）
    let def = e
        .catalog()
        .skill_by_name("gj_food_chunbing")
        .expect("春饼监听技在目录");
    let inst = e
        .state_core()
        .skills
        .get(&(1, def))
        .expect("目标应获得监听技实例");
    assert_eq!(inst.rounds, 5, "计数 5（原生以 rounds 建模，登记差异）");
    assert!(inst.enabled, "实例应启用");
    // 登记差异：本体入弃牌堆（上游为目标获得）
    assert!(
        e.state_core().discard.contains(&chunbing),
        "原生：本体入弃牌堆（上游 target.$gain2(cards) 为目标获得）——差异已登记"
    );
}

/// 上游点 1：目标门槛——**已持有该食物技能**的目标不可再次被指定。
#[test]
fn upstream_chunbing_target_filter_rejects_existing_holder() {
    let mut e = gujian_engine(20263602);
    advance_to_play(&mut e, 0);
    // 先给 1 号装一次
    let first = place_by_prefix(&mut e, "chunbing", 0);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: first,
            targets: vec![1],
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    drain(&mut e);
    // 再次指定同一目标 → 应被拒（上游 filterTarget !hasSkill）
    let second = place_by_prefix(&mut e, "chunbing", 0);
    let again = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard {
            card: second,
            targets: vec![1],
        },
    });
    assert!(
        matches!(again, CommandOutcome::Rejected(_)),
        "已持有该食物技能的目标不可再次被指定（上游 filterTarget）：{again:?}"
    );
}

/// 差异固定（源码事实）：原生以 `rounds` 建模上游「标记数」，并弃置本体。
#[test]
fn upstream_chunbing_modeling_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/step/gujian.rs"
    ))
    .expect("读古剑源码");
    let idx = src
        .find("HostBehavior::GujianFoodGrant =>")
        .expect("食物授予分支");
    let seg: String = src[idx..].chars().take(3200).collect();
    assert!(
        seg.contains("rounds") && seg.contains("ctx.to_discard(c)"),
        "差异固定：原生以 rounds 建模计数并弃置本体：{seg}"
    );
    let consts = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/vocabulary/host/constants.rs"
    ))
    .expect("读常量");
    assert!(
        consts.contains("GJ_FOOD_ROUNDS_CHUNBING: u8 = 5"),
        "春饼计数常量为 5（与上游 markcount = 5 一致）"
    );
}
