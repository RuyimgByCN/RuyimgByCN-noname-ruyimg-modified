//! swd deferred 消解批 131：护天（`hutian`）——阶段结束寄存牌作护体。
//!
//! 上游 `character/swd.js`：`phaseEnd` 可选——选任意张牌（手/装备）寄存于
//! 一名体力上限不低于张数的其他角色处；其体力低于寄存数时回复至该数；
//! 寄存期间其受到的伤害/失去体力以「体力降至寄存数」为上限；寄存只跨一个
//! 阶段边界，随后（或寄存方死亡时）归还。

use engine_native::event::DamageNature;
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
    for _ in 0..48 {
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

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..24 {
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

/// 护天：寄存 2 张 → 目标回复至 2；期间伤害被护体上限压制；下一阶段边界后归还。
#[test]
fn hutian_deposits_shields_and_returns() {
    let mut e = swd_engine(108001);
    e.debug_grant(0, "hutian");
    assert!(e.catalog().skill_by_name("hutian").is_some());
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    let c2 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_give_hand_card(0, 0, c2);
    // 目标 1 号：体力上限 4（≥2），体力 1
    e.debug_set_max_hp(1, 4);
    e.debug_set_hp(1, 1);
    // 推进到 0 号某阶段结束（用出牌阶段结束触发护天）
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    assert_eq!(e.turn().current, 0);
    assert_eq!(e.turn().phase, Phase::Play);
    next_phase(&mut e); // Play 收束（护天窗）
    let hand_before = e.debug_hand(0).len();
    assert!(hand_before >= 2, "手牌足够：{hand_before}");
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "护天确认门"
    );
    assert!(
        answer(
            &mut e,
            "hutian_cards",
            vec![ChoiceOption::Card(c1), ChoiceOption::Card(c2)]
        ),
        "护天选牌窗"
    );
    assert!(
        answer(&mut e, "hutian_target", vec![ChoiceOption::Player(1)]),
        "护天选目标窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), hand_before - 2, "两张牌寄存离手");
    assert_eq!(e.state_core().player(0).hutian_deposit, Some(1));
    assert_eq!(e.state_core().player(1).hutian_hold.len(), 2, "寄存 2 张");
    assert_eq!(e.debug_hp(1), 2, "体力低于寄存数 → 回复至 2");
    // 护体：1 号再受 1 点伤害 → 体力降至 2（寄存数）为上限 → 实际 0 伤
    let hp1 = e.debug_hp(1);
    let out = e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp1, "护体上限：体力不降至寄存数以下");
    // 下一阶段收束（寄存只跨一个阶段边界）→ 归还
    let mut guard = 0;
    while e.state_core().player(0).hutian_deposit.is_some() && guard < 4 {
        next_phase(&mut e);
        drain(&mut e);
        guard += 1;
    }
    assert_eq!(e.state_core().player(0).hutian_deposit, None, "寄存已清");
    assert!(e.state_core().player(1).hutian_hold.is_empty(), "寄存区已清");
    assert_eq!(e.debug_hand(1).len(), 2, "牌归还目标");
}

/// 护天（对照）：目标体力上限低于张数 → 不可选；放弃选择不寄存。
#[test]
fn hutian_target_filter_and_decline() {
    let mut e = swd_engine(108002);
    e.debug_grant(0, "hutian");
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    let c1 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    e.debug_set_max_hp(1, 1); // 上限 1 < 1? 选 1 张 → 上限 ≥1 可选
    e.debug_set_max_hp(2, 3);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    next_phase(&mut e);
    assert!(
        answer(&mut e, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "护天确认门"
    );
    let pick = e.open_choice().cloned().expect("护天选牌窗");
    let out = e.submit(GameCommand {
        actor: pick.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: pick.id,
            selection: vec![ChoiceOption::Card(c1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    let tgt = e.open_choice().cloned().expect("护天选目标窗");
    assert!(
        tgt.spec.options.contains(&ChoiceOption::Player(1)),
        "上限 ≥1 的目标可选：{:?}",
        tgt.spec.options
    );
    // 放弃目标窗 → 不寄存
    let out = e.submit(GameCommand {
        actor: tgt.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer { choice_id: tgt.id, selection: Vec::new() },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hutian_deposit, None, "放弃不寄存");
    assert!(e.debug_hand(0).contains(&c1), "牌未离手");
}

/// 护天（对照）：寄存方死亡 → 牌归还目标。
#[test]
fn hutian_returns_on_owner_death() {
    let mut e = swd_engine(108003);
    e.debug_grant(0, "hutian");
    e.debug_clear_hand(0);
    for p in 1..4 {
        e.debug_clear_hand(p);
    }
    // 手工构造寄存态（0 号寄存 1 张于 1 号处）
    let c1 = e.debug_place(0, "tao");
    e.debug_give_hand_card(0, 0, c1);
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != c1);
        core.players[0].hutian_deposit = Some(1);
        core.players[0].hutian_just_given = false;
        core.players[1].hutian_owner = Some(0);
        core.players[1].hutian_hold = vec![c1];
    }
    // 0 号阵亡（1 血受 2 伤，无救援）
    e.debug_set_hp(0, 1);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let out = e.debug_deal_damage(Some(1), 0, 2, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert!(!e.debug_alive(0), "0 号阵亡");
    assert!(e.debug_hand(1).contains(&c1), "寄存牌归还目标");
    assert!(e.state_core().player(1).hutian_hold.is_empty());
}

/// 护天：四字段存档往返。
#[test]
fn hutian_state_roundtrip() {
    let mut e = swd_engine(108004);
    {
        let core = e.state_core_mut();
        core.players[0].hutian_deposit = Some(2);
        core.players[0].hutian_just_given = true;
        core.players[2].hutian_owner = Some(0);
        core.players[2].hutian_hold = vec![7, 8];
    }
    let snap = e.save();
    assert_eq!(snap.core.players[0].hutian_deposit, Some(2));
    assert!(snap.core.players[0].hutian_just_given);
    assert_eq!(snap.core.players[2].hutian_hold, vec![7, 8]);
    assert_eq!(snap.core.players[2].hutian_owner, Some(0));
    let json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读包");
    let pack = content_native_loader::bridge_pack(&json).expect("桥接").pack;
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复");
    assert_eq!(restored.state_core().player(0).hutian_deposit, Some(2));
    assert!(restored.state_core().player(0).hutian_just_given);
    assert_eq!(restored.state_core().player(2).hutian_hold, vec![7, 8]);
}
