//! swd deferred 消解批 149：昆仑镜（`kunlunjing`）——阶段收束快照 / 体力下降回滚。
//!
//! 上游 `character/swd.js`：组技两面——`kunlunjing2`（`phaseAfter` 静默）
//! 记录全场手牌/装备/判定区快照与当时体力；`kunlunjing1`（`phaseBegin`
//! 可选）在自己体力低于快照值时回滚（把快照中的牌拉回各自原位）。

use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..12 {
        if e.turn().current == seat && e.turn().phase == Phase::Play {
            return;
        }
        next_phase(e);
        drain(e);
    }
    panic!("未到 {seat} 号出牌阶段");
}

/// 昆仑镜：阶段收束记录快照 → 体力下降 → 下一阶段开始回滚（牌回原位）。
#[test]
fn kunlunjing_snapshot_and_rewind() {
    let mut e = swd_engine(125001);
    e.debug_grant(0, "kunlunjing");
    assert!(e.catalog().skill_by_name("kunlunjing").is_some());
    advance_to_play(&mut e, 0);
    // 布阵：0 号手牌两张、2 号装备一张（快照将记录）
    e.debug_clear_hand(0);
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_give_hand_card(0, 0, c2);
    let armor = e.debug_place(2, "armor");
    {
        let core = e.state_core_mut();
        core.players[2].hand.retain(|c| *c != armor);
        core.players[2].equips.push(armor);
    }
    // 出牌阶段收束 → 记录快照
    next_phase(&mut e);
    drain(&mut e);
    assert!(
        !e.state_core().turn.kunlunjing_snapshot.is_empty(),
        "已记录快照"
    );
    let snap_hp = e.state_core().turn.kunlunjing_hp;
    // 破坏现场：0 号手牌弃掉一张、2 号防具被取走入弃牌堆
    e.debug_discard_card(0, c1);
    {
        let core = e.state_core_mut();
        core.players[2].equips.retain(|c| *c != armor);
        core.discard.push(armor);
    }
    // 体力降到快照值以下
    e.debug_set_hp(0, (snap_hp - 1).max(1));
    // 推进到下一次阶段开始（0 号下回合判定阶段）→ 回滚窗
    let mut rolled = false;
    for _ in 0..24 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "trigger_confirm" {
                assert!(
                    matches!(
                        e.choice_view_for(Some(c.observer)),
                        Some(engine_native::ChoiceView::Own {
                            source_skill: None,
                            ..
                        })
                    ),
                    "Host same-prompt ask must not acquire OptionalGate provenance"
                );
                let out = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Confirm],
                    },
                });
                assert!(
                    matches!(
                        out,
                        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                    ),
                    "{out:?}"
                );
                rolled = true;
                break;
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
            continue;
        }
        next_phase(&mut e);
    }
    assert!(rolled, "出现昆仑镜回滚确认窗");
    drain(&mut e);
    assert!(
        e.debug_hand(0).contains(&c1),
        "快照中的手牌被拉回（手牌 {:?}）",
        e.debug_hand(0)
    );
    assert!(
        e.state_core().player(2).equips.contains(&armor),
        "快照中的装备被拉回原位"
    );
    assert!(!e.state_core().discard.contains(&armor), "装备已离开弃牌堆");
    // 回滚后同一回合继续推进，阶段收束会重新记录快照（上游 phaseAfter 同口径）
}

/// 昆仑镜（对照）：体力未低于快照值 → 不回滚（快照保留）。
#[test]
fn kunlunjing_no_rewind_when_hp_not_lower() {
    let mut e = swd_engine(125002);
    e.debug_grant(0, "kunlunjing");
    advance_to_play(&mut e, 0);
    next_phase(&mut e);
    drain(&mut e);
    assert!(
        !e.state_core().turn.kunlunjing_snapshot.is_empty(),
        "已记录快照"
    );
    // 体力保持 → 下一次阶段开始不回滚
    let mut guard = 0;
    let mut rolled = false;
    while guard < 12 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "trigger_confirm" {
                rolled = true;
                break;
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
            continue;
        }
        next_phase(&mut e);
        guard += 1;
    }
    assert!(!rolled, "体力未降：不出现回滚窗");
}

/// 昆仑镜：两字段存档往返。
#[test]
fn kunlunjing_state_roundtrip() {
    let mut e = swd_engine(125003);
    e.state_core_mut().turn.kunlunjing_snapshot = vec![(0, vec![7, 8], vec![9], vec![10])];
    e.state_core_mut().turn.kunlunjing_hp = 3;
    let snap = e.save();
    assert_eq!(snap.core.turn.kunlunjing_snapshot.len(), 1);
    assert_eq!(snap.core.turn.kunlunjing_hp, 3);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().turn.kunlunjing_snapshot.len(), 1);
    assert_eq!(
        restored.state_core().turn.kunlunjing_snapshot[0].1,
        vec![7, 8]
    );
    assert_eq!(restored.state_core().turn.kunlunjing_hp, 3);
}
