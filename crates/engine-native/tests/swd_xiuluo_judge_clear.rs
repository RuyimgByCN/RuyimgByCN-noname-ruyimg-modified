//! swd 批 153：修罗（`swd_xiuluo`）——弃手牌清同花色判定区延时锦囊。
//!
//! 上游（`杀海拾遗/character/swd/skill.js`）：`trigger {player:'phaseZhunbeiBegin'}`，
//! `direct:true`——判定区有牌时可选：弃一张手牌并弃置判定区一张**同花色**
//! 延时锦囊（`discardPlayerCard(player, 2, 'hj')`）。

use engine_native::catalog::Suit;
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
    let pack = content_native_loader::bridge_pack(&json).expect("桥接 swd 包").pack;
    Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
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

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}

fn place_suit_basic(e: &mut Engine, player: u8, suit: Suit) -> u32 {
    let found = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| {
            let d = &e.catalog().cards[*c as usize];
            d.kind.is_basic_kind() && d.suit == suit
        })
        .expect("该花色基本牌");
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != found);
    core.players[player as usize].hand.push(found);
    found
}

/// 修罗：手牌与判定区同花色 → 双弃；花色不同不出现判定窗。
#[test]
fn xiuluo_discards_same_suit_judge_card() {
    let mut e = swd_engine(129001);
    e.debug_grant(0, "swd_xiuluo");
    assert!(e.catalog().skill_by_name("swd_xiuluo").is_some());
    // 判定区放一张延时锦囊（乐不思蜀），手牌放同花色基本牌
    let le = e.debug_place(0, "le");
    let suit = e.catalog().cards[le as usize].suit;
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != le);
        core.players[0].judge.push(le);
    }
    e.debug_clear_hand(0);
    let hand = place_suit_basic(&mut e, 0, suit);
    // 推进到 0 号**下一次**准备阶段（游戏起始判定帧已在授予前计划完毕）
    let mut answered = false;
    for _ in 0..40 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "xiuluo_hand" {
                let out = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Card(hand)],
                    },
                });
                assert!(
                    matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                    "{out:?}"
                );
                continue;
            }
            if c.spec.prompt == "xiuluo_judge" {
                assert!(
                    c.spec.options.contains(&ChoiceOption::Card(le)),
                    "同花色判定牌在候选：{:?}",
                    c.spec.options
                );
                let out = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Card(le)],
                    },
                });
                assert!(
                    matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                    "{out:?}"
                );
                answered = true;
                break;
            }
            // 其他窗（确认门/判定窗/闪窗等）：取保守回退
            let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
            let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if has_pass {
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
        next_phase(&mut e);
    }
    assert!(answered, "出现修罗判定牌窗");
    drain(&mut e);
    assert!(!e.state_core().player(0).judge.contains(&le), "判定牌已弃置");
    assert!(!e.debug_hand(0).contains(&hand), "手牌已弃置");
    assert!(e.state_core().discard.contains(&le), "判定牌入弃牌堆");
}

/// 修罗（对照）：判定区无牌 → 不开窗。
#[test]
fn xiuluo_no_window_without_judge_card() {
    let mut e = swd_engine(129002);
    e.debug_grant(0, "swd_xiuluo");
    e.debug_clear_hand(0);
    let _ = place_suit_basic(&mut e, 0, Suit::Heart);
    let mut guard = 0;
    while guard < 8 {
        if let Some(c) = e.open_choice().cloned() {
            assert_ne!(c.spec.prompt, "xiuluo_hand", "判定区无牌不开窗");
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
        next_phase(&mut e);
        guard += 1;
    }
    drain(&mut e);
}

/// 修罗（对照）：未持技者判定区有牌亦不开窗。
#[test]
fn xiuluo_absent_no_window() {
    let mut e = swd_engine(129003);
    let le = e.debug_place(0, "le");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != le);
        core.players[0].judge.push(le);
    }
    e.debug_clear_hand(0);
    let _ = place_suit_basic(&mut e, 0, Suit::Heart);
    let mut guard = 0;
    while guard < 8 {
        if let Some(c) = e.open_choice().cloned() {
            assert_ne!(c.spec.prompt, "xiuluo_hand", "未持技不开窗");
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
        next_phase(&mut e);
        guard += 1;
    }
    drain(&mut e);
    assert!(e.state_core().player(0).judge.contains(&le), "判定牌保留");
}
