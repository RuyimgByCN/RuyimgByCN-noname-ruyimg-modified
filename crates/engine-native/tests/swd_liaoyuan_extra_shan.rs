//! swd deferred 消解批 145：燎原（`liaoyuan`）——弃同花色牌换额外闪需求与加伤。
//!
//! 上游 `character/swd.js`：`trigger {player:'shaBegin'}`，`direct:true`——
//! 弃任意张与该【杀】同花色的牌（手/装备），令目标额外需打出等量张【闪】；
//! 未能全部打出则该杀**直接命中**且伤害 +张数（组技 liaoyuan2/3）。

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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
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
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}


/// 从牌堆取一张指定花色的**基本牌**入手（避免特殊卡自带效果干扰）。
fn place_same_suit_basic(e: &mut Engine, player: u8, suit: Suit) -> Option<u32> {
    let found = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| {
            let def = &e.catalog().cards[*c as usize];
            def.suit == suit && def.kind.is_basic_kind()
        })?;
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != found);
    core.players[player as usize].hand.push(found);
    Some(found)
}

/// 燎原：弃 2 张同花色牌 → 目标未出额外闪 → 直接命中且加伤 2。
#[test]
fn liaoyuan_direct_hit_with_bonus() {
    let mut e = swd_engine(122001);
    e.debug_grant(0, "liaoyuan");
    assert!(e.catalog().skill_by_name("liaoyuan").is_some());
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 杀 + 两张与其同花色的手牌
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let suit = e.catalog().cards[sha as usize].suit;
    let s1 = place_same_suit_basic(&mut e, 0, suit).expect("同花色基本牌 1");
    let s2 = place_same_suit_basic(&mut e, 0, suit).expect("同花色基本牌 2");
    let _s3 = place_same_suit_basic(&mut e, 0, suit).expect("同花色基本牌 3");
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "燎原确认门"
    );
    assert!(
        answer(&mut e, "liaoyuan_discard", vec![ChoiceOption::Card(s1)]),
        "燎原弃牌窗 1"
    );
    assert!(
        answer(&mut e, "liaoyuan_discard", vec![ChoiceOption::Card(s2)]),
        "燎原弃牌窗 2"
    );
    // 余 1 张 → 循环自然结束（上游 countCards>1 才续问）
    // 目标（1 号）无闪 → 应直接命中并加伤 2
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 3, "直接命中且加伤 2（1+2）");
}

/// 燎原（对照）：放弃发动 → 正常结算（无闪则 1 伤）。
#[test]
fn liaoyuan_decline_normal_damage() {
    let mut e = swd_engine(122002);
    e.debug_grant(0, "liaoyuan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let suit = e.catalog().cards[sha as usize].suit;
    let s1 = place_same_suit_basic(&mut e, 0, suit).expect("同花色基本牌");
    let hp1 = e.debug_hp(1);
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "trigger_confirm", Vec::new()),
        "燎原确认门（放弃）"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "放弃：正常 1 伤");
    let discarded = !e.debug_hand(0).contains(&s1) && e.state_core().discard.contains(&s1);
    assert!(!discarded, "放弃：同花色牌未被弃置（手牌 {:?}）", e.debug_hand(0));
}

/// 燎原（对照）：无同花色牌 → 不开窗。
#[test]
fn liaoyuan_no_window_without_same_suit() {
    let mut e = swd_engine(122003);
    e.debug_grant(0, "liaoyuan");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let suit = e.catalog().cards[sha as usize].suit;
    let other = match suit {
        Suit::Spade => Suit::Heart,
        _ => Suit::Spade,
    };
    let _h1 = place_same_suit_basic(&mut e, 0, other).expect("异花色基本牌");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::PlayCard { card: sha, targets: vec![1] },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let c = e.open_choice().cloned();
    assert!(
        c.as_ref().map(|c| c.spec.prompt != "liaoyuan_discard").unwrap_or(true),
        "无同花色牌不开窗：{:?}",
        c.map(|c| c.spec.prompt)
    );
    drain(&mut e);
}

/// 燎原：`liaoyuan_count` 字段存档往返。
#[test]
fn liaoyuan_state_roundtrip() {
    let mut e = swd_engine(122004);
    e.state_core_mut().turn.liaoyuan_count = 3;
    let snap = e.save();
    assert_eq!(snap.core.turn.liaoyuan_count, 3);
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().turn.liaoyuan_count, 3);
}
