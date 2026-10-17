//! swd deferred 消解批 138：天伦（`tianlun`）——判定区牌替换本次判定。
//!
//! 上游 `character/swd.js`：`trigger {global:'judge'}`，`direct:true`——可选：
//! 从「本次判定牌 + 全场判定区牌」中选一张替换本次判定牌，旧判定牌弃置。

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

/// 天伦：判定时用判定区牌替换本次判定牌（旧判定牌弃置）。
///
/// 布阵：0 号判定区放【乐不思蜀】（触发判定）+ 一张红桃牌（天伦候选），
/// 推进到 0 号**下一回合**的判定阶段（届时乐不思蜀结算）。
#[test]
fn tianlun_replaces_judge_card_from_judge_zone() {
    let mut e = swd_engine(115001);
    e.debug_grant(0, "tianlun");
    assert!(e.catalog().skill_by_name("tianlun").is_some());
    // 判定区两张延时锦囊：le 先结算，bing 作为天伦候选（判定区牌）
    let le = e.debug_place(0, "le");
    e.state_core_mut().player_mut(0).judge.push(le);
    let zone_card = e.debug_place(0, "bing");
    e.state_core_mut().player_mut(0).judge.push(zone_card);
    // 推进到下一次判定窗（0 号下一回合判定阶段）
    let mut found = None;
    for _ in 0..30 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "tianlun_rewrite" {
                found = Some(c);
                break;
            }
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
    }
    let c = found.expect("天伦窗");
    assert!(
        c.spec.options.contains(&ChoiceOption::Card(zone_card)),
        "候选含判定区牌：{:?}",
        c.spec.options
    );
    // 原判定牌 = 候选中既非 Pass 也非 bing 的那张
    let old_judge = c
        .spec
        .options
        .iter()
        .find_map(|o| match o {
            ChoiceOption::Card(id) if *id != zone_card => Some(*id),
            _ => None,
        })
        .expect("原判定牌在候选中");
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Card(zone_card)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(
        e.state_core().discard.contains(&old_judge),
        "旧判定牌弃置（上游 judging[0].discard() 同口径）"
    );
}

/// 天伦（对照）：放弃改判 → 判定区不变。
#[test]
fn tianlun_decline_keeps_judge_zone() {
    let mut e = swd_engine(115002);
    e.debug_grant(0, "tianlun");
    // 判定区两张延时锦囊：le 先结算，bing 作为天伦候选（判定区牌）
    let le = e.debug_place(0, "le");
    e.state_core_mut().player_mut(0).judge.push(le);
    let zone_card = e.debug_place(0, "bing");
    e.state_core_mut().player_mut(0).judge.push(zone_card);
    let mut declined = false;
    for _ in 0..30 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "tianlun_rewrite" {
                let out = e.submit(GameCommand {
                    actor: c.observer,
                    expected_revision: e.revision(),
                    dedup_id: next_dedup(),
                    action: GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Pass],
                    },
                });
                assert!(
                    matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                    "{out:?}"
                );
                declined = true;
                break;
            }
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
    }
    assert!(declined, "出现天伦窗并放弃");
    assert!(
        !e.state_core().discard.contains(&zone_card),
        "放弃改判：判定区牌未被本次改判消费"
    );
    drain(&mut e);
}

/// 天伦（对照）：未持技者不出现改判窗。
#[test]
fn tianlun_absent_no_window() {
    let mut e = swd_engine(115003);
    // 判定区两张延时锦囊：le 先结算，bing 作为天伦候选（判定区牌）
    let le = e.debug_place(0, "le");
    e.state_core_mut().player_mut(0).judge.push(le);
    let zone_card = e.debug_place(0, "bing");
    e.state_core_mut().player_mut(0).judge.push(zone_card);
    for _ in 0..30 {
        if let Some(c) = e.open_choice().cloned() {
            assert_ne!(c.spec.prompt, "tianlun_rewrite", "未持技不出现天伦窗");
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
    }
    drain(&mut e);
}
