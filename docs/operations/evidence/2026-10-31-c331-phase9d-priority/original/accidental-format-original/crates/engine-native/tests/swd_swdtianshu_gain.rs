//! swd deferred 消解批 142：天书（`swdtianshu`）——获取目标武将的一个技能。
//!
//! 上游 `character/swd.js`：`enable:'phaseUse'`，`unique:true`——弃一张锦囊
//! 手牌，选一名角色，获取其武将的一个自己未持有的技能（永久）；来源角色
//! 阵亡时移除（组技 `swdtianshu_remove`）。

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

/// 天书：弃锦囊牌 → 选 1 号 → 选其一个技能 → 永久获得；来源阵亡后移除。
#[test]
fn swdtianshu_gains_skill_and_removes_on_source_death() {
    let mut e = swd_engine(119001);
    e.debug_grant(0, "swdtianshu");
    assert!(e.catalog().skill_by_name("swdtianshu").is_some());
    // 给 1 号一个已知武将（标准将），并确保 0 号未持有其技能
    e.debug_assemble_general(1, "关羽");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let trick = e.debug_place(0, "wuzhong");
    e.debug_give_hand_card(0, 0, trick);
    let def = e.catalog().skill_by_name("swdtianshu").expect("天书");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 选锦囊牌
    let pick = e.open_choice().cloned().expect("天书选牌窗");
    assert_eq!(pick.spec.prompt, "swdtianshu_card");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(trick)],
        },
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 选目标（1 号）
    let tgt = e.open_choice().cloned().expect("天书选目标窗");
    assert_eq!(tgt.spec.prompt, "swdtianshu_target");
    assert!(tgt.spec.options.contains(&ChoiceOption::Player(1)));
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
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    // 选技能（关羽的技能之一）
    let skill = e.open_choice().cloned().expect("天书选技能窗");
    assert_eq!(skill.spec.prompt, "swdtianshu_skill");
    assert!(!skill.spec.options.is_empty(), "候选技能非空");
    let first = skill.spec.options[0];
    let out = e.submit(GameCommand {
        actor: skill.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: skill.id,
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
    let idx = match first {
        ChoiceOption::Tag(t) => t as usize,
        _ => panic!("技能选项应为 Tag"),
    };
    // Tag=候选下标 → 重枚举映射到技能定义（与宿主实现同口径）
    let names: Vec<String> = {
        let mut out = Vec::new();
        for slot in e.state_core().player(1).general_slots.iter().flatten() {
            if let Some((_, _, skills, _, _)) = e.catalog().general_meta(*slot) {
                for n in skills {
                    if !out.contains(&n.to_string()) {
                        out.push(n.to_string());
                    }
                }
            }
        }
        out
    };
    let def_id = e.catalog().skill_by_name(&names[idx]).expect("技能定义");
    assert!(
        e.state_core().skills.contains_key(&(0, def_id)),
        "已获得技能 {}",
        names[idx]
    );
    assert!(!e.debug_hand(0).contains(&trick), "锦囊牌已弃");
    assert_eq!(e.state_core().player(0).swdtianshu_from, Some(1));
    // 1 号阵亡 → 移除所获技能
    e.debug_set_hp(1, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(2), 1, 2, engine_native::event::DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_alive(1), "1 号阵亡");
    assert!(
        !e.state_core().skills.contains_key(&(0, def_id)),
        "来源阵亡后移除所获技能"
    );
    assert_eq!(e.state_core().player(0).swdtianshu_from, None);
}

/// 天书（对照）：手牌无锦囊 → 不可发动。
#[test]
fn swdtianshu_requires_trick_card() {
    let mut e = swd_engine(119002);
    e.debug_grant(0, "swdtianshu");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let tao = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, tao);
    let def = e.catalog().skill_by_name("swdtianshu").expect("天书");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "无锦囊拒绝：{out:?}"
    );
}

/// 天书（对照）：未持技者不可发动；非出牌阶段不可发动。
#[test]
fn swdtianshu_absent_and_phase_gate() {
    let mut e = swd_engine(119003);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    let def = e.catalog().skill_by_name("swdtianshu").expect("天书");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Rejected(_)),
        "未持技拒绝：{out:?}"
    );
}

/// 天书：两字段存档往返。
#[test]
fn swdtianshu_state_roundtrip() {
    let mut e = swd_engine(119004);
    {
        let p = e.state_core_mut().player_mut(0);
        p.swdtianshu_from = Some(2);
        p.swdtianshu_granted = Some(7);
    }
    let snap = e.save();
    assert_eq!(snap.core.players[0].swdtianshu_from, Some(2));
    assert_eq!(snap.core.players[0].swdtianshu_granted, Some(7));
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json)
        .expect("桥接")
        .pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().player(0).swdtianshu_from, Some(2));
    assert_eq!(restored.state_core().player(0).swdtianshu_granted, Some(7));
}
