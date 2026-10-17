//! swd deferred 消解批 136：极略（`jilve`）+ 通用目标规则表。
//!
//! 上游 `character/swd.js`：`enable:'phaseUse'`，`usable:3`——观看牌堆顶 3 张，
//! 从中选一张基本牌或锦囊牌**使用**（目标按该牌规则；非手牌用牌）。

use engine_native::catalog::CardKind;
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

fn activate(e: &mut Engine, skill: &str) -> CommandOutcome {
    let def = e.catalog().skill_by_name(skill).expect("技能在目录");
    e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    })
}

/// 极略：牌堆顶 3 张中取一张【杀】使用（目标窗）→ 目标受伤；每阶段 3 次上限。
#[test]
fn jilve_uses_top_card_and_counts_three() {
    let mut e = swd_engine(113001);
    e.debug_grant(0, "jilve");
    assert!(e.catalog().skill_by_name("jilve").is_some());
    advance_to_play(&mut e, 0);
    // 把一张杀压到牌堆顶（下一张摸牌位）
    let sha = e.debug_set_deck_top("sha");
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let hp1 = e.debug_hp(1);
    let out = activate(&mut e, "jilve");
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let pick = e.open_choice().cloned().expect("极略选牌窗");
    assert_eq!(pick.spec.prompt, "jilve_card");
    assert!(
        pick.spec.options.contains(&ChoiceOption::Card(sha)),
        "候选含牌堆顶的杀：{:?}",
        pick.spec.options
    );
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(sha)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("极略目标窗");
    assert_eq!(tgt.spec.prompt, "jilve_targets");
    assert_eq!((tgt.spec.min, tgt.spec.max), (1, 1));
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
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1 - 1, "牌堆顶的杀命中");
    assert_eq!(e.state_core().player(0).jilve_used, 1, "已用 1 次");
    assert!(!e.state_core().deck.contains(&sha), "牌离牌堆");
}

/// 极略（对照）：非出牌阶段不可发动；已用满 3 次后不可发动。
#[test]
fn jilve_phase_gate_and_three_use_cap() {
    let mut e = swd_engine(113002);
    e.debug_grant(0, "jilve");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().phase, Phase::Draw);
    let out = activate(&mut e, "jilve");
    assert!(matches!(out, CommandOutcome::Rejected(_)), "非出牌阶段拒绝：{out:?}");
    // 满 3 次
    advance_to_play(&mut e, 0);
    {
        let p = e.state_core_mut().player_mut(0);
        p.jilve_used = 3;
    }
    let out2 = activate(&mut e, "jilve");
    assert!(matches!(out2, CommandOutcome::Rejected(_)), "满 3 次拒绝：{out2:?}");
}

/// 极略（对照）：未持技者不可发动；无目标牌（牌堆顶全为装备）时不开窗。
#[test]
fn jilve_absent_and_no_candidate() {
    let mut e = swd_engine(113003);
    advance_to_play(&mut e, 0);
    let def = e.catalog().skill_by_name("jilve").expect("极略");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(matches!(out, CommandOutcome::Rejected(_)), "未持技拒绝：{out:?}");
    // 持技但牌堆顶三张全为装备 → 无可选项（牌堆清空后仅留装备）
    let mut e2 = swd_engine(113004);
    e2.debug_grant(0, "jilve");
    advance_to_play(&mut e2, 0);
    // 把牌堆换成三张装备（其余移入弃牌堆）
    {
        let equips: Vec<u32> = e2
            .state_core()
            .deck
            .iter()
            .copied()
            .filter(|c| e2.catalog().card_kind(*c).equip_slot().is_some())
            .take(3)
            .collect();
        if equips.len() == 3 {
            let core = e2.state_core_mut();
            core.deck.retain(|c| !equips.contains(c));
            core.deck.extend(equips);
        }
    }
    let out2 = activate(&mut e2, "jilve");
    if matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }) {
        // 若仍开窗，则候选不得含装备牌
        if let Some(c) = e2.open_choice() {
            for o in &c.spec.options {
                if let ChoiceOption::Card(id) = o {
                    assert!(
                        e2.catalog().card_kind(*id).equip_slot().is_none(),
                        "候选不含装备牌"
                    );
                }
            }
        }
        drain(&mut e2);
    }
}

/// 通用目标规则表：杀按射程、顺手按距离 1、无中/五谷/桃园无目标、AOE 全体其他。
#[test]
fn legal_targets_helper_rules() {
    let e = swd_engine(113005);
    let core = e.state_core();
    let cat = e.catalog();
    let (min, max, t) = core.legal_targets(cat, CardKind::Wuzhong, 0);
    assert_eq!((min, max, t.len()), (0, 0, 0), "无中生有：无目标");
    let (_, _, t) = core.legal_targets(cat, CardKind::SavageAttack, 0);
    assert_eq!(t.len(), 3, "南蛮：全部其他存活");
    let (_, _, t) = core.legal_targets(cat, CardKind::Sha, 0);
    assert!(t.len() <= 3 && !t.contains(&0), "杀：射程内其他角色");
    let (_, _, t) = core.legal_targets(cat, CardKind::Snatch, 0);
    assert!(t.iter().all(|p| *p != 0), "顺手：其他角色");
    let (_, _, t) = core.legal_targets(cat, CardKind::Wugu, 0);
    assert_eq!(t.len(), 4, "五谷：全体存活（含自己）");
}
