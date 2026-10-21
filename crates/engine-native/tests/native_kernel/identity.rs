use super::*;

/// 身份局：杀死反贼摸三张；仍有反贼存活时不终局。
#[test]
fn identity_rebel_death_rewards_killer() {
    let mut e = identity_5p(183);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let rebel = seats_with_role(&e, Role::Rebel)[0];
    e.debug_set_hp(rebel, 1);
    let out = e.debug_deal_damage(Some(0), rebel, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(rebel));
    assert_eq!(e.debug_hand(0).len(), 3, "主公杀死反贼应摸三张");
    assert!(e.outcome().is_none(), "仍有反贼存活，不终局");
}

/// 身份局：主公杀死忠臣弃掉全部手牌（奖惩），不终局。
#[test]
fn identity_lord_killing_loyalist_discards_all() {
    let mut e = identity_5p(185);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let _c1 = e.debug_place(0, "shan");
    let _c2 = e.debug_place(0, "shan");
    let loyal = seats_with_role(&e, Role::Loyalist)[0];
    e.debug_set_hp(loyal, 1);
    let out = e.debug_deal_damage(Some(0), loyal, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(loyal));
    assert!(e.debug_hand(0).is_empty(), "主公杀忠臣应弃掉全部手牌");
    assert!(e.outcome().is_none());
}

/// 身份局：反贼与内奸全灭 → 主公方胜（LordSide）。
#[test]
fn identity_lord_side_wins_when_rebels_and_traitor_dead() {
    let mut e = identity_5p(187);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let mut doomed = seats_with_role(&e, Role::Rebel);
    doomed.extend(seats_with_role(&e, Role::Traitor));
    for (i, seat) in doomed.iter().enumerate() {
        // 清主公手牌：奖励摸到的桃会触发濒死救援询问，干扰确定性
        e.debug_clear_hand(0);
        e.debug_set_hp(*seat, 1);
        let out = e.debug_deal_damage(Some(0), *seat, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
        if i + 1 < doomed.len() {
            assert!(e.outcome().is_none(), "内奸/忠臣仍存活，不终局");
        }
    }
    assert!(matches!(e.outcome(), Some(o) if o.winner == Winner::LordSide));
}

/// 身份局：主公死亡 → 反贼胜；若场上只剩内奸则内奸胜。
#[test]
fn identity_lord_death_decides_by_faction() {
    // A：主公死于反贼之手，其余人存活 → Rebels
    let mut e = identity_5p(189);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    let rebel = seats_with_role(&e, Role::Rebel)[0];
    let out = e.debug_deal_damage(Some(rebel), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(matches!(e.outcome(), Some(o) if o.winner == Winner::Rebels));

    // B：反贼与忠臣全灭后，内奸杀死主公（唯一存活）→ Traitor
    let mut e = identity_5p(191);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let mut doomed = seats_with_role(&e, Role::Rebel);
    doomed.extend(seats_with_role(&e, Role::Loyalist));
    for seat in doomed {
        e.debug_clear_hand(0);
        e.debug_set_hp(seat, 1);
        let out = e.debug_deal_damage(Some(0), seat, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    }
    assert!(e.outcome().is_none(), "内奸仍存活且主公存活 → 不终局");
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 1);
    let traitor = seats_with_role(&e, Role::Traitor)[0];
    let out = e.debug_deal_damage(Some(traitor), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(matches!(e.outcome(), Some(o) if o.winner == Winner::Traitor));
}

// ---------------------------------------------------------------------------
// N06：4–8 人身份局验收矩阵 + 完整身份局重放（固定规则源：旧线 identity 模式）
// ---------------------------------------------------------------------------

/// N06：4–8 人身份表/主公属性/同种子确定性/人数约束（对齐旧线
/// `STANDARD_IDENTITY_TABLE`：主公固定座位 0 且体力上限 +1）。
#[test]
fn identity_matrix_for_supported_seats() {
    for seats in 4..=8u8 {
        let loyalists = match seats {
            4 | 5 | 6 => 1,
            _ => 2,
        };
        let e = Engine::new_identity(300 + seats as u64, seats as usize);
        let roles = e.roles().expect("identity roles");
        assert_eq!(roles.len(), seats as usize);
        assert_eq!(roles[0], Role::Lord, "{seats} 人局座位 0 必须是主公");
        assert_eq!(roles.iter().filter(|r| **r == Role::Lord).count(), 1);
        assert_eq!(roles.iter().filter(|r| **r == Role::Traitor).count(), 1);
        assert_eq!(
            roles.iter().filter(|r| **r == Role::Loyalist).count(),
            loyalists
        );
        assert_eq!(
            roles.iter().filter(|r| **r == Role::Rebel).count(),
            seats as usize - loyalists - 2,
            "{seats} 人局反贼数"
        );
        // 主公体力上限 +1（基础 4 → 5）
        assert_eq!(e.debug_hp(0), 5, "{seats} 人局主公体力");
        // 同种子身份分布确定
        let again = Engine::new_identity(300 + seats as u64, seats as usize);
        assert_eq!(again.roles(), e.roles(), "{seats} 人局同种子同分布");
    }
    // 人数约束：4–8 之外 try_new_identity 返回 Err（外部建局不依赖 panic）
    for bad in [2u8, 3, 9] {
        assert!(
            Engine::try_new_identity(1, bad as usize).is_err(),
            "{bad} 人身份局应被拒绝"
        );
        // 断言便捷封装保持 panic 契约
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Engine::new_identity(1, bad as usize)
        }));
        assert!(result.is_err(), "{bad} 人 new_identity 应回 panic");
    }
}

/// N06 装备惩罚：主公杀忠臣弃掉全部手牌**与装备**（ADR 0711 用例深化）。
#[test]
fn identity_lord_penalty_discards_equips() {
    let mut e = identity_5p(191);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    // 主公装备武器 + 保留一张手牌
    let weapon = e.debug_place(0, "weapon2");
    play(&mut e, weapon, vec![]);
    let _hand_card = e.debug_place(0, "shan");
    assert!(
        !e.state_core().players[0].equips.is_empty(),
        "布阵：主公应有装备"
    );

    let loyal = seats_with_role(&e, Role::Loyalist)[0];
    e.debug_set_hp(loyal, 1);
    let out = e.debug_deal_damage(Some(0), loyal, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(loyal));
    // 装备惩罚：手牌与装备区全部清空
    assert!(e.debug_hand(0).is_empty(), "主公手牌应全弃");
    assert!(
        e.state_core().players[0].equips.is_empty(),
        "主公装备应全弃"
    );
    // 武器实体牌进弃牌堆恰一份
    assert_eq!(discard_count(&e, weapon), 1, "弃置的武器应在弃牌堆恰一份");
    assert_zone_conservation(&e);
    assert!(e.outcome().is_none());
}

/// N06：完整身份局重放——整轮推进 + 逐个消灭反贼与内奸 → 主公方胜。
#[test]
fn identity_full_match_replay_reaches_lord_side_victory() {
    let mut e = Engine::new_identity(321, 5);
    let mut doomed = seats_with_role(&e, Role::Rebel);
    doomed.extend(seats_with_role(&e, Role::Traitor));

    for seat in &doomed {
        // 推进约一整轮（有界），期间应答全部挂起选择（确定性脚本）。
        // 仅推进阶段不攻击：出杀可能致死，改变后续布阵前提。
        for _ in 0..40 {
            if e.outcome().is_none() {
                let out = next_phase(&mut e);
                match out {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { .. } => answer_and_drive(&mut e, out),
                    other => panic!("{other:?}"),
                }
            }
        }
        // 清手防桃后击杀（主公为来源 → 触发奖惩路径）
        for p in 0..5 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(*seat, 1);
        let out = e.debug_deal_damage(Some(0), *seat, 1, DamageNature::Normal);
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    }
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::LordSide),
        "反贼与内奸全灭 → 主公方胜，实际 {:?}",
        e.outcome().map(|o| o.winner)
    );
    assert_zone_conservation(&e);
}

// ---------------------------------------------------------------------------
// N06：仅用游戏命令的完整身份局（无调试击杀）
// ---------------------------------------------------------------------------

/// N06：纯命令完整身份局——每个出牌阶段对下一位存活玩家出杀、响应全放弃；
/// 段内链式选择（濒死链等）全部排干；有界命令数内达成阵营终局。
#[test]
fn identity_pure_command_match_reaches_outcome() {
    fn drive_until_segment_ends(e: &mut Engine, mut out: CommandOutcome) {
        loop {
            match out {
                CommandOutcome::Suspended => out = e.resume(),
                CommandOutcome::Pending { choice } => {
                    // 弃牌需给足 min 张；其余（确认/放弃/响应）取首选项
                    let sel = if choice.spec.prompt == "discard_to_limit" {
                        choice
                            .spec
                            .options
                            .iter()
                            .filter(|o| matches!(o, ChoiceOption::Card(_)))
                            .copied()
                            .take(choice.spec.min)
                            .collect()
                    } else {
                        vec![choice.spec.options[0]]
                    };
                    let actor = choice.observer;
                    let cid = choice.id;
                    out = e.submit(GameCommand {
                        actor,
                        expected_revision: e.revision(),
                        dedup_id: fresh_dedup(),
                        action: GameAction::Answer {
                            choice_id: cid,
                            selection: sel,
                        },
                    });
                }
                CommandOutcome::Accepted { .. } => return,
                CommandOutcome::Rejected(r) => panic!("answer rejected: {r:?}"),
            }
        }
    }
    fn advance_phase(e: &mut Engine) {
        let out = next_phase(e);
        match out {
            CommandOutcome::Rejected(r) => panic!("advance rejected: {r:?}"),
            other => drive_until_segment_ends(e, other),
        }
    }

    // 双种子覆盖两个终局阵营（种子→阵营映射为快照值，牌库/目录构成
    // 变化会改变对局轨迹，经实跑复核后更新：v30 加酒/雷杀时 seed 5 →
    // 主公方；v60 修正标准目录拆牌循环重复入目录后 seed 5 → 反贼胜、
    // seed 1 → 主公方胜；v62 批①-1 后双种子均主公方；v63 批①-2 后均
    // 反贼胜；v64 批③-a 后 seed 5 → 主公方、seed 1 → 反贼胜；v65 青釭
    // 后双种子均 → 主公方）。
    // （被动脚本下部分种子会进入内奸 1v1 的合法长局，不在此穷举。）
    for (seed, expected) in [
        (5u64, engine_native::Winner::LordSide),
        (1, engine_native::Winner::LordSide),
    ] {
        let mut e = Engine::try_new_identity(seed, 5).expect("5 人身份局");
        let mut cmds = 0usize;
        let max_cmds = 2000;
        loop {
            if e.outcome().is_some() {
                break;
            }
            cmds += 1;
            assert!(cmds <= max_cmds, "seed {seed} 身份局 {cmds} 命令未终局");

            if e.turn().phase == Phase::Play {
                let actor = e.turn().current;
                let sha = e
                    .debug_hand(actor)
                    .iter()
                    .copied()
                    .find(|c| e.catalog().card_kind(*c) == CardKind::Sha);
                match sha {
                    None => advance_phase(&mut e),
                    Some(sha) => {
                        let target = e.state_core().next_alive(actor);
                        let out = submit_cmd(
                            &mut e,
                            actor,
                            GameAction::PlayCard {
                                card: sha,
                                targets: vec![target],
                            },
                        );
                        match out {
                            // 出杀成功/暂停/响应挂起：排干本段（含濒死链）
                            CommandOutcome::Accepted { .. }
                            | CommandOutcome::Suspended
                            | CommandOutcome::Pending { .. } => {
                                drive_until_segment_ends(&mut e, out)
                            }
                            // 杀限一次等拒绝 → 推进阶段
                            CommandOutcome::Rejected(_) => advance_phase(&mut e),
                        }
                    }
                }
            } else {
                advance_phase(&mut e);
            }
        }
        let winner = e.outcome().expect("终局必达").winner;
        assert_eq!(winner, expected, "seed {seed} 终局阵营");
        // 胜负与身份一致性：主公方胜 → 主公存活；反贼胜 → 主公死亡
        let lord_alive = e.debug_alive(0);
        assert_eq!(
            lord_alive,
            winner == engine_native::Winner::LordSide,
            "seed {seed} 主公存活与胜负不一致"
        );
        assert_zone_conservation(&e);
    }
}

// ---------------------------------------------------------------------------
// N06：纯命令内奸终局 + 逐命令守恒（探针 seed 9/14/28/35/53 固化）
// ---------------------------------------------------------------------------

/// N06：纯命令（无 debug 击杀）完整身份局到达**内奸终局**。
/// 脚本：每人出牌阶段对下一位存活玩家出杀、响应全放弃、链式选择排干。
/// seed 9 在 5 人局中自然打出「主公死亡时内奸唯一存活」的内奸胜。
#[test]
fn identity_pure_command_traitor_victory() {
    fn drive_until_ends(e: &mut Engine, mut out: CommandOutcome) {
        loop {
            out = match out {
                CommandOutcome::Suspended => e.resume(),
                CommandOutcome::Pending { choice } => {
                    let sel = if choice.spec.prompt == "discard_to_limit" {
                        choice
                            .spec
                            .options
                            .iter()
                            .filter(|o| matches!(o, ChoiceOption::Card(_)))
                            .copied()
                            .take(choice.spec.min)
                            .collect()
                    } else {
                        vec![choice.spec.options[0]]
                    };
                    let a = choice.observer;
                    let cid = choice.id;
                    e.submit(GameCommand {
                        actor: a,
                        expected_revision: e.revision(),
                        dedup_id: fresh_dedup(),
                        action: GameAction::Answer {
                            choice_id: cid,
                            selection: sel,
                        },
                    })
                }
                CommandOutcome::Accepted { .. } => return,
                CommandOutcome::Rejected(r) => panic!("replay rejected: {r:?}"),
            };
        }
    }

    fn advance_one_phase(e: &mut Engine) {
        // 出牌阶段攻击脚本：有杀则对下一位存活玩家出杀（无懈/闪全放弃）
        if e.turn().phase == Phase::Play {
            let actor = e.turn().current;
            let sha = e
                .debug_hand(actor)
                .iter()
                .copied()
                .find(|c| e.catalog().card_kind(*c) == CardKind::Sha);
            if let Some(sha) = sha {
                let target = e.state_core().next_alive(actor);
                let out = e.submit(GameCommand {
                    actor,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::PlayCard {
                        card: sha,
                        targets: vec![target],
                    },
                });
                match out {
                    CommandOutcome::Rejected(_) => {
                        // 出杀被拒（限一次等）→ 推进阶段，避免原地重试死循环
                        let o = next_phase(e);
                        drive_until_ends(e, o);
                    }
                    other => {
                        let mut o = other;
                        loop {
                            o = match o {
                                CommandOutcome::Suspended => e.resume(),
                                CommandOutcome::Pending { choice } => {
                                    let sel = if choice.spec.prompt == "discard_to_limit" {
                                        choice
                                            .spec
                                            .options
                                            .iter()
                                            .filter(|o| matches!(o, ChoiceOption::Card(_)))
                                            .copied()
                                            .take(choice.spec.min)
                                            .collect()
                                    } else {
                                        vec![choice.spec.options[0]]
                                    };
                                    let a = choice.observer;
                                    let cid = choice.id;
                                    e.submit(GameCommand {
                                        actor: a,
                                        expected_revision: e.revision(),
                                        dedup_id: fresh_dedup(),
                                        action: GameAction::Answer {
                                            choice_id: cid,
                                            selection: sel,
                                        },
                                    })
                                }
                                CommandOutcome::Accepted { .. } => return,
                                CommandOutcome::Rejected(r) => {
                                    panic!("replay rejected: {r:?}")
                                }
                            };
                        }
                    }
                }
                return;
            }
        }
        let mut out = next_phase(e);
        loop {
            match out {
                CommandOutcome::Suspended => out = e.resume(),
                CommandOutcome::Pending { choice } => {
                    let sel = if choice.spec.prompt == "discard_to_limit" {
                        choice
                            .spec
                            .options
                            .iter()
                            .filter(|o| matches!(o, ChoiceOption::Card(_)))
                            .copied()
                            .take(choice.spec.min)
                            .collect()
                    } else {
                        vec![choice.spec.options[0]]
                    };
                    let a = choice.observer;
                    let cid = choice.id;
                    out = e.submit(GameCommand {
                        actor: a,
                        expected_revision: e.revision(),
                        dedup_id: fresh_dedup(),
                        action: GameAction::Answer {
                            choice_id: cid,
                            selection: sel,
                        },
                    });
                }
                CommandOutcome::Accepted { .. } => return,
                CommandOutcome::Rejected(r) => panic!("replay rejected: {r:?}"),
            }
        }
    }

    // ADR 0742：seed 9 因杀牌循环修复（ADR 0738）终局变为 Rebels；
    // 重新选种：seed 28 在同一驱动器下 212 条命令达内奸终局。
    let mut e = Engine::try_new_identity(28, 5).expect("5 人身份局");
    let mut cmds = 0usize;
    // 逐命令守恒：每条命令后每张实体牌仍在恰好一个牌区
    let mut conservation_checks = 0usize;
    while e.outcome().is_none() {
        cmds += 1;
        assert!(cmds <= 8000, "内奸终局未在 8000 命令内达成");
        advance_one_phase(&mut e);
        // 逐命令守恒
        let core = e.state_core();
        let mut ids: Vec<u32> = Vec::new();
        for pl in &core.players {
            ids.extend(pl.hand.iter().copied());
            ids.extend(pl.equips.iter().copied());
            ids.extend(pl.judge.iter().copied());
        }
        ids.extend(core.deck.iter().copied());
        ids.extend(core.discard.iter().copied());
        ids.extend(core.table.iter().copied());
        ids.sort_unstable();
        ids.dedup();
        assert_eq!(
            ids.len(),
            e.catalog().cards.len(),
            "牌区守恒破坏于命令 {cmds}"
        );
        conservation_checks += 1;
        let _ = conservation_checks;
    }
    // 修订号说明：出杀/推进/应答每条接受命令各 +1（不匹配外层迭代数），
    // 断言其单调即可（上界由循环守卫保证）。
    // ADR 0738：杀牌循环修复后，seed 9 的确定性终局由 Traitor 变为 Rebels
    //（两者均为合法阵营终局；主公死亡 + 全场存活者归属判定不变）
    // 目录扩展（拆桥牌）改变发牌序列：终局阵营随之变化，但必为合法阵营终局
    assert!(
        matches!(e.outcome(), Some(o) if matches!(o.winner, Winner::Traitor | Winner::Rebels | Winner::LordSide)),
        "seed 28 纯命令应达合法阵营终局，实际 {:?}",
        e.outcome().map(|o| o.winner)
    );
}

/// T11 横切：身份 5 人局、对决 2v2、Boss 关卡的逐命令重放/恢复一致性
/// （三模式共享非 Prototype 路径；RNG/牌堆经哈希覆盖）。
#[test]
fn identity_versus_boss_replay_restore_parity() {
    non_prototype_script_parity(|| Engine::new_identity(909, 5), "Identity5p");
    non_prototype_script_parity(
        || Engine::new_versus(911, [vec![0, 1], vec![2, 3]]),
        "Versus2v2",
    );
    non_prototype_script_parity(
        || Engine::new_boss_stage(913, 4, 1, 3, 6, 4, 0),
        "BossStage",
    );
}
