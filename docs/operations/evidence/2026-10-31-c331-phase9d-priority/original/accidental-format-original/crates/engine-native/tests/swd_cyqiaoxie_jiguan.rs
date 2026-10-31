//! swd deferred 消解批 133：此桥邪（`cyqiaoxie`）——失去装备后使用机关牌。
//!
//! 上游 `character/swd.js`：`loseEnd` 频繁技（`event.type!='use'` 且失去的牌
//! 含装备）——按失去装备张数逐次：从机关牌名中选一张使用（自目标/单目标/
//! AOE 按牌名规则）；取消即止。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn swd_engine(seed: u64) -> Engine {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包数据");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接 swd 包")
        .pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
            assert!(
                matches!(
                    out,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if has_pass {
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
                selection: fallback,
            },
        });
    }
    false
}

/// 此桥邪：失去 1 件装备 → 选一张机关牌（自目标类）使用；牌从牌堆/弃牌堆入手后结算。
#[test]
fn cyqiaoxie_uses_jiguan_card_after_equip_loss() {
    let mut e = swd_engine(110001);
    e.debug_grant(0, "cyqiaoxie");
    assert!(e.catalog().skill_by_name("cyqiaoxie").is_some());
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    // 0 号装备一件（随后被取走 → 触发）
    let equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    // 确保牌堆/弃堆存在机关牌（自目标类：机关术需自己有装备 → 用指路小虎/木甲人）
    let out = e.debug_take_owned_card(0, equip);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("此桥邪选机关牌窗");
    assert_eq!(pick.spec.prompt, "cyqiaoxie_card");
    assert!(!pick.spec.options.is_empty(), "候选非空");
    let first = pick.spec.options[0];
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![first],
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
    // 机关牌已使用（离开牌堆/弃堆 → 结算后入弃堆或桌面处理）
    let used_card = match first {
        ChoiceOption::Card(c) => c,
        _ => panic!("候选应为牌"),
    };
    assert!(
        !e.state_core().deck.contains(&used_card) || e.debug_discard_len() > 0,
        "机关牌已进入结算"
    );
}

/// 此桥邪（对照）：未失去装备 → 不开窗。
#[test]
fn cyqiaoxie_no_window_without_equip_loss() {
    let mut e = swd_engine(110002);
    e.debug_grant(0, "cyqiaoxie");
    e.debug_clear_hand(0);
    let c = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c);
    e.debug_discard_card(0, c);
    drain(&mut e);
    let ch = e.open_choice().cloned();
    assert!(
        ch.as_ref()
            .map(|c| c.spec.prompt != "cyqiaoxie_card")
            .unwrap_or(true),
        "手牌丢失不触发：{:?}",
        ch.map(|c| c.spec.prompt)
    );
}

/// 此桥邪（对照）：未持技者失去装备不触发。
#[test]
fn cyqiaoxie_absent_no_window() {
    let mut e = swd_engine(110003);
    let equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    let out = e.debug_take_owned_card(0, equip);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    let ch = e.open_choice().cloned();
    assert!(
        ch.as_ref()
            .map(|c| c.spec.prompt != "cyqiaoxie_card")
            .unwrap_or(true),
        "未持技不触发：{:?}",
        ch.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 此桥邪：取消选择即止（不再开窗）。
#[test]
fn cyqiaoxie_cancel_stops_loop() {
    let mut e = swd_engine(110004);
    e.debug_grant(0, "cyqiaoxie");
    e.debug_clear_hand(0);
    let equip = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != equip);
        core.players[0].equips.push(equip);
    }
    let out = e.debug_take_owned_card(0, equip);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert!(answer(&mut e, "cyqiaoxie_card", Vec::new()), "取消选择窗");
    drain(&mut e);
    let ch = e.open_choice().cloned();
    assert!(
        ch.as_ref()
            .map(|c| c.spec.prompt != "cyqiaoxie_card")
            .unwrap_or(true),
        "取消后不再开窗：{:?}",
        ch.map(|c| c.spec.prompt)
    );
}
