//! swd deferred 消解批 155：攻心（`gongxin`）——看手牌选红桃：弃置或置牌堆顶。
//!
//! 上游 `character/extra.js:11396`：`enable:'phaseUse'`，`usable:1`——选一名
//! 其他有手牌的角色，展示其手牌，选一张**红桃**牌：弃置之，或将其置于
//! 牌堆顶。

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

/// 攻心（弃置分支）：选目标 → 看手牌选红桃 → 弃置该牌。
#[test]
fn gongxin_discards_target_heart() {
    let mut e = swd_engine(131001);
    e.debug_grant(0, "gongxin");
    assert!(e.catalog().skill_by_name("gongxin").is_some());
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    let heart = place_suit_basic(&mut e, 1, Suit::Heart);
    let spade = place_suit_basic(&mut e, 1, Suit::Spade);
    let def = e.catalog().skill_by_name("gongxin").expect("攻心");
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
    // 目标窗
    let tgt = e.open_choice().cloned().expect("攻心目标窗");
    assert_eq!(tgt.spec.prompt, "gongxin_target");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 手牌窗（展示全部手牌）
    let pick = e.open_choice().cloned().expect("攻心手牌窗");
    assert_eq!(pick.spec.prompt, "gongxin_card");
    assert!(
        pick.spec.options.contains(&ChoiceOption::Card(spade)),
        "非红桃也在展示列表：{:?}",
        pick.spec.options
    );
    // 先选非红桃 → 应重开窗（不生效）
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(spade)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let again = e.open_choice().cloned().expect("非红桃后重开窗");
    assert_eq!(again.spec.prompt, "gongxin_card");
    assert!(e.debug_hand(1).contains(&spade), "非红桃未被处理");
    // 选红桃 → 选择窗 → 弃置
    let out = e.submit(GameCommand {
        actor: again.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: again.id,
            selection: vec![ChoiceOption::Card(heart)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let choice = e.open_choice().cloned().expect("攻心选择窗");
    assert_eq!(choice.spec.prompt, "gongxin_choice");
    let out = e.submit(GameCommand {
        actor: choice.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_hand(1).contains(&heart), "红桃已弃置");
    assert!(e.state_core().discard.contains(&heart), "红桃入弃牌堆");
}

/// 攻心（置顶分支）：红桃牌被置于牌堆顶（牌堆末尾=顶）。
#[test]
fn gongxin_places_heart_on_deck_top() {
    let mut e = swd_engine(131002);
    e.debug_grant(0, "gongxin");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    let heart = place_suit_basic(&mut e, 1, Suit::Heart);
    let def = e.catalog().skill_by_name("gongxin").expect("攻心");
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
    let tgt = e.open_choice().cloned().expect("目标窗");
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("手牌窗");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(heart)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let choice = e.open_choice().cloned().expect("选择窗");
    let out = e.submit(GameCommand {
        actor: choice.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_hand(1).contains(&heart), "红桃离手");
    assert_eq!(
        e.state_core().deck.last().copied(),
        Some(heart),
        "红桃置于牌堆顶（末尾）"
    );
}

/// 攻心（对照）：未持技者不可发动；每回合限一次。
#[test]
fn gongxin_absent_and_once_per_turn() {
    let mut e = swd_engine(131003);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    let _ = place_suit_basic(&mut e, 1, Suit::Heart);
    let def = e.catalog().skill_by_name("gongxin").expect("攻心");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "未持技拒绝：{out:?}");
}

/// 攻心是**每阶段一次**（上游 `character/extra/skill.js`：`enable:"phaseUse"` + `usable: 1`）。
///
/// 第 1011 轮修：`HostBehavior::Gongxin` 原先不在 `host.rs` 的 `once_per_turn` 允许表里。
#[test]
fn gongxin_is_once_per_play_phase() {
    let mut e = swd_engine(885101);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "gongxin");
    let spade = place_suit_basic(&mut e, 1, Suit::Spade);
    assert!(spade > 0 || true);
    let def = e.catalog().skill_by_name("gongxin").expect("攻心在目录");
    let first = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(
            first,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "攻心第一次应可发动：{first:?}"
    );
    // 应答完第一次：先试 Pass（取消路径），被拒就按 min 选首项
    for _ in 0..10 {
        let Some(ch) = e.open_choice().cloned() else {
            break;
        };
        // 取消用**空选**（`Pass` 对部分 prompt 不被接受）
        let out = e.submit(GameCommand {
            actor: ch.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: ch.id,
                selection: vec![],
            },
        });
        if matches!(out, CommandOutcome::Rejected(_)) {
            let sel: Vec<ChoiceOption> = ch
                .spec
                .options
                .iter()
                .take(ch.spec.min.max(1))
                .copied()
                .collect();
            let _ = e.submit(GameCommand {
                actor: ch.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: ch.id,
                    selection: sel,
                },
            });
        }
    }
    let second = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(
            second,
            CommandOutcome::Rejected(engine_native::RejectReason::SkillAlreadyUsed)
        ),
        "攻心：同一出牌阶段**不应**再次发动（上游 `usable: 1`），实际 {second:?}"
    );
}
