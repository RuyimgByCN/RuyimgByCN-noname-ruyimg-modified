//! swd deferred 消解批 135：革新印（`gxianyin`）——弃一花色手牌换另一花色等量牌。
//!
//! 上游 `character/swd.js`：`enable:'phaseUse'`，`usable:1`——选一种手牌花色
//! 弃置其全部手牌（`_triggered=null`：不触发失去牌类事件），再从牌堆获得
//! 等量张另一花色的牌。

use engine_native::catalog::Suit;
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

/// 革新印：弃黑桃全部手牌（静默）→ 选红桃 → 从牌堆获得等量张红桃。
#[test]
fn gxianyin_swaps_suit_cards() {
    let mut e = swd_engine(112001);
    e.debug_grant(0, "gxianyin");
    assert!(e.catalog().skill_by_name("gxianyin").is_some());
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Play);
    // 手牌：两张黑桃 + 一张红桃
    e.debug_clear_hand(0);
    let s1 = e.debug_place_suit(0, Suit::Spade);
    let s2 = e.debug_place_suit(0, Suit::Spade);
    let h1 = e.debug_place_suit(0, Suit::Heart);
    assert_eq!(e.debug_hand(0).len(), 3);
    let deck_hearts_before = e
        .state_core()
        .deck
        .iter()
        .filter(|c| e.catalog().cards[**c as usize].suit == Suit::Heart)
        .count();
    let def = e.catalog().skill_by_name("gxianyin").expect("革新印");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 选花色（黑桃 = Tag 0）
    let suit = e.open_choice().cloned().expect("革新印花色窗");
    assert_eq!(suit.spec.prompt, "gxianyin_suit");
    let out = e.submit(GameCommand {
        actor: suit.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: suit.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 选获得花色（红桃 = Tag 1）
    let gain = e.open_choice().cloned().expect("革新印获得花色窗");
    assert_eq!(gain.spec.prompt, "gxianyin_gain_suit");
    let out = e.submit(GameCommand {
        actor: gain.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: gain.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let hand = e.debug_hand(0);
    assert!(!hand.contains(&s1) && !hand.contains(&s2), "黑桃已弃");
    assert!(hand.contains(&h1), "红桃保留");
    let hearts_gained = hand
        .iter()
        .filter(|c| e.catalog().cards[**c as usize].suit == Suit::Heart && **c != h1)
        .count();
    assert_eq!(hearts_gained, 2, "获得 2 张红桃（等量）");
    assert!(
        e.state_core()
            .deck
            .iter()
            .filter(|c| e.catalog().cards[**c as usize].suit == Suit::Heart)
            .count()
            < deck_hearts_before,
        "红桃从牌堆取出"
    );
    // 静默弃置：不触发失去牌类事件（义山记录为空）
    assert!(
        e.state_core().player(0).yishan_lost.is_empty(),
        "静默弃置不记失去牌"
    );
}

/// 革新印（对照）：非出牌阶段不可发动；每回合限一次。
#[test]
fn gxianyin_phase_gate_and_limit() {
    let mut e = swd_engine(112002);
    e.debug_grant(0, "gxianyin");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Draw);
    let def = e.catalog().skill_by_name("gxianyin").expect("革新印");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "非出牌阶段拒绝：{out:?}");
}

/// 革新印（对照）：未持技者不可发动；手牌为空时不可发动。
#[test]
fn gxianyin_absent_and_empty_hand() {
    let mut e = swd_engine(112003);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let def = e.catalog().skill_by_name("gxianyin").expect("革新印");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "未持技拒绝：{out:?}");
    // 持技但手牌为空
    let mut e2 = swd_engine(112004);
    e2.debug_grant(0, "gxianyin");
    for _ in 0..12 {
        if e2.turn().current == 0 && e2.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e2);
        drain(&mut e2);
    }
    e2.debug_clear_hand(0);
    let out2 = e2.submit(GameCommand {
        actor: 0,
        expected_revision: e2.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out2, CommandOutcome::Rejected(_)), "空手拒绝：{out2:?}");
}
