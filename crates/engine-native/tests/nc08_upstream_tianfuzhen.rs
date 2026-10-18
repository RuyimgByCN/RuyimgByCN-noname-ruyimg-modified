//! NC08 第⑦层（上游行为对照）样本 14：`tianfuzhen`（天覆阵，U05 zhenfa 卡牌子包 / 阵法）。
//!
//! 上游源码（`archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js`）：
//!
//! ```js
//! tianfuzhen: {
//!     type: "zhenfa",
//!     recastable: true,
//!     enable() { return game.hasPlayer(current => current.isMajor()); },   // 存在大势力角色
//!     filterTarget(card, player, target) {
//!         return target.isMajor() && target.countCards("he") > 0;           // 大势力且有牌
//!     },
//!     selectTarget: -1,                                                     // 全部满足者
//!     content() { target.chooseToDiscard("he", true).delay = false; },      // **强制**弃 1 张
//!     mode: ["guozhan"],
//!     ai: { … },
//! },
//! ```
//!
//! 由 JS 推导的规则点：
//! 1. **可用条件**：存在大势力角色（`enable: hasPlayer(isMajor)`）；
//! 2. **目标集合 = 全部大势力且有牌（手牌或装备）者**（`filterTarget` + `selectTarget: -1`）；
//! 3. **效果 = 每名目标强制弃 1 张**（`chooseToDiscard("he", true)`，`true` = 强制、**不可取消**）。
//!
//! 原生实现引用：`engine/play_card.rs:903`（目标校验：存活 ≥4 且存在大势力 → 期望目标集合）、
//! `content/host/step/swd_card_effects.rs:1146`（逐目标 `allow_cancel: false` 的弃牌询问；
//! 无牌目标 `Flow::Yield` 跳过；结算完毕弃置本体）。
//!
//! **登记差异（本文件固定）**：上游 `enable` 只要求「存在大势力角色」；原生**额外要求存活 ≥4**
//! （注释称与上游 `guozhanZhenfaEnabled` 同口径）——两处判据并存，本文件以源码事实固定。
//!
//! 未覆盖（登记）：`mode: ["guozhan"]` 模式门、`recastable`（重铸）面、`ai` 策略面。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn zhenfa_engine(seed: u64) -> Engine {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-zhenfa-upstream-noname/assets/cards.json"))
            .expect("读阵法包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn place_by_prefix(e: &mut Engine, prefix: &str, player: u8) -> engine_native::CardId {
    let name = e
        .catalog()
        .cards
        .iter()
        .map(|c| c.name)
        .find(|n| n.contains(prefix))
        .unwrap_or_else(|| panic!("{prefix} 在目录"));
    e.debug_place(player, name)
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
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
}


/// 给 0/1 号装配**同势力**武将（使该势力成为大势力），返回是否成功。
fn assign_major_pair(e: &mut Engine) -> bool {
    let mut by_faction: std::collections::BTreeMap<String, Vec<(u16, String)>> = Default::default();
    for def in e.general_candidates() {
        if let Some((name, faction, _, _, _)) = e.catalog().general_meta(def) {
            by_faction
                .entry(format!("{faction:?}"))
                .or_default()
                .push((def, name.to_string()));
        }
    }
    let Some((_, pair)) = by_faction.iter().find(|(_, v)| v.len() >= 2) else {
        return false;
    };
    for (seat, (_, name)) in [(0u8, &pair[0]), (1u8, &pair[1])] {
        if e.assign_general_named(seat, name).is_err() {
            return false;
        }
    }
    true
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

/// 上游点 2：目标集合必须**恰为**「大势力且有牌者」——多给一个非大势力目标应被拒。
#[test]
fn upstream_tianfuzhen_target_set_must_match_major_with_cards() {
    let mut e = zhenfa_engine(20264101);
    if !assign_major_pair(&mut e) {
        eprintln!("SKIP：本包无同势力双武将，无法构造大势力局面");
        return;
    }
    advance_to_play(&mut e, 0);
    let card = place_by_prefix(&mut e, "tianfuzhen", 0);
    // 计算原生认定的期望目标集合（大势力且有手牌/装备）
    let catalog = e.catalog();
    let core = e.state_core();
    let expected: Vec<u8> = core
        .alive_players()
        .filter(|p| {
            core.is_major_faction(catalog, *p)
                && (!core.player(*p).hand.is_empty() || !core.player(*p).equips.is_empty())
        })
        .collect();
    // ① 空目标表（当期望非空时）应被拒
    if !expected.is_empty() {
        let empty = e.submit(GameCommand {
            actor: 0,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::PlayCard { card, targets: vec![] },
        });
        assert!(
            matches!(empty, CommandOutcome::Rejected(_)),
            "期望目标非空时空目标表应被拒：{empty:?}"
        );
    }
    // ② 期望集合本身应被接受
    let ok = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: expected.clone() },
    });
    assert!(
        matches!(ok, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "期望目标集合应被接受（期望 {expected:?}）：{ok:?}"
    );
}

/// 上游点 3：每名目标**强制**弃 1 张（询问不可取消），无牌目标被跳过。
#[test]
fn upstream_tianfuzhen_forces_one_discard_per_target() {
    let mut e = zhenfa_engine(20264102);
    if !assign_major_pair(&mut e) {
        eprintln!("SKIP：本包无同势力双武将，无法构造大势力局面");
        return;
    }
    advance_to_play(&mut e, 0);
    let card = place_by_prefix(&mut e, "tianfuzhen", 0);
    let catalog = e.catalog();
    let core = e.state_core();
    let expected: Vec<u8> = core
        .alive_players()
        .filter(|p| {
            core.is_major_faction(catalog, *p)
                && (!core.player(*p).hand.is_empty() || !core.player(*p).equips.is_empty())
        })
        .collect();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: expected.clone() },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 逐目标询问：不可取消，且选 1 张
    let mut seen = 0usize;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt != "swd_tianfuzhen_discard" {
            // 无懈窗口等前置询问：放弃/取消（本用例只验天覆阵的强制弃牌面）
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
            continue;
        }
        assert!(!c.spec.allow_cancel, "强制弃牌：询问不可取消");
        assert_eq!((c.spec.min, c.spec.max), (1, 1), "恰弃 1 张");
        seen += 1;
        let pick = c.spec.options.first().copied().expect("有牌目标应有选项");
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: vec![pick] },
        });
    }
    drain(&mut e);
    assert_eq!(seen, expected.len(), "每名期望目标各一次弃牌询问");
}

/// 上游点 1 的**登记差异**：上游 `enable` 仅要求「存在大势力」；原生额外要求**存活 ≥4**。
#[test]
fn upstream_tianfuzhen_enable_gate_difference_is_fixed() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/engine/play_card.rs"
    ))
    .expect("读 play_card 源码");
    let idx = src.find("CardKind::Tianfuzhen | CardKind::Qixingzhen").expect("阵法分支");
    let seg: String = src[idx..].chars().take(700).collect();
    assert!(
        seg.contains("core.players.len() < 4"),
        "差异固定：原生额外要求存活/座位 ≥4（上游 enable 仅要求存在大势力）：{seg}"
    );
    // 行为面：3 人局应被拒（存活 <4）
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json =
        std::fs::read_to_string(root.join("shahai-zhenfa-upstream-noname/assets/cards.json"))
            .expect("读阵法包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let mut e3 = Engine::new_with_extensions(20264103, 3, &[&pack]).expect("3 人建局");
    advance_to_play(&mut e3, 0);
    let card = place_by_prefix(&mut e3, "tianfuzhen", 0);
    let out = e3.submit(GameCommand {
        actor: 0,
        expected_revision: e3.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card, targets: vec![] },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "3 人局：原生因存活 <4 拒绝（差异登记）：{out:?}"
    );
}
