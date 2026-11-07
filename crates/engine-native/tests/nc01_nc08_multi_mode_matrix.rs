//! NC01 & NC08 多模式内核确定性、同优先级触发排序、中断挂起存档恢复与跨端等价矩阵（7 模式全覆盖）。
//!
//! 覆盖模式：
//! 1. `identity`（身份局 4–8 人）
//! 2. `kingdoms`（国战双将暗置与势力结算）
//! 3. `versus`（2v2 队伍对抗与替补）
//! 4. `boss`（挑战 Boss 与重整/助战）
//! 5. `doudizhu`（斗地主 1v2）
//! 6. `brawl`（乱斗生存）
//! 7. `single`（1v1 单挑 KOF 会话）

use engine_native::archive::{decode_archive, encode_archive};
use engine_native::event::DamageNature;
use engine_native::{
    ChoiceKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, GameMode, Phase,
};

fn build_all_seven_modes(seed: u64) -> Vec<(&'static str, Engine)> {
    vec![
        ("identity", Engine::new_identity(seed, 5)),
        ("kingdoms", {
            let mut e = Engine::new_kingdoms(seed, 4);
            let mut snap = e.save();
            snap.core.players[0].general_slots = [Some(0), Some(1)];
            snap.core.players[1].general_slots = [Some(2), Some(3)];
            snap.core.players[2].general_slots = [Some(4), Some(5)];
            snap.core.players[3].general_slots = [Some(6), Some(7)];
            e = Engine::restore(&snap).expect("restore kingdoms setup");
            e
        }),
        ("versus", Engine::new_versus(seed, [vec![0, 2], vec![1, 3]])),
        ("boss", Engine::new_boss(seed, 4, 0, 2)),
        ("doudizhu", Engine::new_doudizhu(seed, 3, 0)),
        ("brawl", Engine::new_brawl(seed, 4)),
        ("single", Engine::new_single(seed)),
    ]
}

fn step_default(e: &mut Engine, dedup: u64) {
    let (actor, action) = if let Some(open) = e.open_choice().cloned() {
        let sel = if open.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if open.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if open.spec.allow_cancel {
            vec![]
        } else {
            open.spec.options.iter().take(open.spec.min).copied().collect()
        };
        (
            open.observer,
            GameAction::Answer {
                choice_id: open.id,
                selection: sel,
            },
        )
    } else {
        (e.turn().current, GameAction::NextPhase)
    };
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: dedup,
        action,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "step rejected: {out:?}"
    );
}

#[test]
fn nc01_nc08_seven_modes_opening_invariants_and_card_conservation() {
    for (name, e) in build_all_seven_modes(20261106) {
        let core = e.state_core();
        assert!(!core.players.is_empty(), "{name}: players non-empty");
        for (seat, p) in core.players.iter().enumerate() {
            assert!(p.alive, "{name} seat {seat} alive at opening");
            assert_eq!(p.hand.len(), 4, "{name} seat {seat} starts with 4 hand cards");
        }
        let hand_total: usize = core.players.iter().map(|p| p.hand.len()).sum();
        assert_eq!(
            hand_total + core.deck.len() + core.discard.len() + core.table.len(),
            e.catalog().cards.len(),
            "{name}: physical card conservation at opening"
        );
        match name {
            "identity" => {
                assert!(matches!(core.mode, GameMode::Identity { .. }));
                assert_eq!(core.players[0].max_hp, 5, "identity lord +1 max_hp");
            }
            "kingdoms" => assert!(matches!(core.mode, GameMode::Kingdoms)),
            "versus" => assert!(matches!(core.mode, GameMode::Versus { .. })),
            "boss" => assert!(matches!(core.mode, GameMode::Boss { boss: 0, .. })),
            "doudizhu" => assert!(matches!(core.mode, GameMode::Doudizhu { lord: 0 })),
            "brawl" => assert!(matches!(core.mode, GameMode::Prototype)),
            "single" => assert!(matches!(core.mode, GameMode::Kof { .. })),
            _ => unreachable!(),
        }
    }
}

#[test]
fn nc01_nc08_seven_modes_same_priority_trigger_order_and_mid_choice_archive_roundtrip() {
    for (idx, (name, mut direct)) in build_all_seven_modes(20261107).into_iter().enumerate() {
        let mut via_restore = build_all_seven_modes(20261107).remove(idx).1;

        for e in [&mut direct, &mut via_restore] {
            for _ in 0..16 {
                if e.turn().current == 0 && e.turn().phase == Phase::Play {
                    break;
                }
                let rev = e.revision();
                step_default(e, rev + 1);
            }
            assert_eq!(
                (e.turn().current, e.turn().phase),
                (0, Phase::Play),
                "{name}: reached seat 0 Play phase"
            );
            // 授予同窗口（Damage Normal priority=0）双技能：遗计 + 刚烈
            e.debug_grant(0, "遗计");
            e.debug_grant(0, "刚烈");
            e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
        }

        let open = direct.open_choice().cloned().expect("trigger_order open choice");
        assert_eq!(
            open.spec.prompt, "trigger_order",
            "{name}: same-priority triggers must open trigger_order choice"
        );
        assert_eq!(open.spec.kind, ChoiceKind::PickTag);
        assert_eq!(open.spec.options.len(), 2);

        // 在 trigger_order 挂起态做 encode_archive -> decode_archive -> Engine::restore
        let bytes = encode_archive(&via_restore.save());
        let decoded = decode_archive(&bytes).expect("decode mid-trigger_order archive");
        let mut restored = Engine::restore(&decoded).expect("restore mid-trigger_order archive");
        assert_eq!(
            direct.state_hash(),
            restored.state_hash(),
            "{name}: state_hash identical right after mid-choice restore"
        );

        // 选择第一个触发技能并在后续步骤中保持完全一致的哈希与日志
        let pick = open.spec.options[0];
        for e in [&mut direct, &mut restored] {
            let c = e.open_choice().cloned().unwrap();
            let rev = e.revision();
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: rev,
                dedup_id: rev + 100,
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![pick],
                },
            });
            assert!(!matches!(out, CommandOutcome::Rejected(_)), "{name}: {out:?}");
        }
        assert_eq!(
            direct.state_hash(),
            restored.state_hash(),
            "{name}: state_hash matches after answering trigger_order"
        );

        // 再推进 20 步（含第二次挂起态存档往返），验证 RNG、事件流与状态摘要完全确定一致
        for step in 0..20u64 {
            if step == 7 {
                let b = encode_archive(&restored.save());
                let d = decode_archive(&b).expect("decode mid-trajectory");
                restored = Engine::restore(&d).expect("restore mid-trajectory");
            }
            step_default(&mut direct, 1000 + step);
            step_default(&mut restored, 1000 + step);
            assert_eq!(
                direct.state_hash(),
                restored.state_hash(),
                "{name} step {step}: state_hash mismatch after restore continuation"
            );
            assert_eq!(
                direct.revision(),
                restored.revision(),
                "{name} step {step}: revision mismatch"
            );
        }
    }
}
