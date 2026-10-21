use super::*;

// ---------------------------------------------------------------------------
// AOE：南蛮入侵/万箭齐发的多目标响应队列（阶段 2 扩面第三项）
// ---------------------------------------------------------------------------

/// 南蛮：持杀者被询问（出杀免伤）、无杀者跳过询问直接受伤、Pass 受伤。
#[test]
fn savage_attack_response_skip_and_pass() {
    let mut e = Engine::new(131, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    let savage = e.debug_place(0, "savage");
    // 1 有杀（询问→出杀免伤）；2 无杀（跳过→受伤）；3 有杀（询问→Pass→受伤）
    let sha1 = e.debug_place(1, "sha");
    let sha3 = e.debug_place(3, "sha");
    let asks_before = count_log(&e, |l| matches!(l, LE::ChoiceOpened { .. }));
    assert!(matches!(
        play(&mut e, savage, vec![]),
        CommandOutcome::Pending { .. }
    ));
    // 1 出杀免伤 → 2 无杀被跳过（不询问）→ 3 被询问
    let out = answer(&mut e, ChoiceOption::Card(sha1));
    assert!(
        matches!(out, CommandOutcome::Pending { .. }),
        "应链式询问下一持牌者: {out:?}"
    );
    // 2 被跳过未询问；3 被询问
    assert_eq!(prompt_of(&e), "aoe_response");
    assert_eq!(e.open_choice().unwrap().observer, 3);
    assert!(matches!(
        answer(&mut e, ChoiceOption::Pass),
        CommandOutcome::Accepted { .. }
    ));
    // 结算：1 免伤；2、3 各 -1；3 的杀留在手上（Pass），1 的杀进弃牌堆
    assert_eq!(hp(&e, 1), 4);
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
    assert!(!e.debug_hand(1).contains(&sha1));
    assert!(e.debug_hand(3).contains(&sha3));
    // 询问次数口径（2026-10-16 按上游核对冻结）：上游 `autochoose = lib.filter.
    // autoRespondSha`（`!player.hasSha('respond')`，library/index.js:12002）在无
    // 响应牌时直接 `result = {bool:false}`（content.ts:5489），即**不发起询问**；
    // 原生同为「无牌跳过」。因此本场景恰好 2 次 aoe_response 询问（目标 1、3），
    // 无杀的目标 2 不产生选择。
    assert_eq!(
        count_log(&e, |l| matches!(l, LE::ChoiceOpened { .. })) - asks_before,
        2,
        "无杀目标不发起询问（对齐上游 autochoose）"
    );
}

/// 万箭：全部目标持闪但全部 Pass → 全员 -1。
#[test]
fn archery_attack_all_pass_takes_damage() {
    let mut e = Engine::new(133, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for p in [1u8, 2, 3] {
        e.debug_clear_hand(p);
        e.debug_place(p, "shan");
    }
    let archery = e.debug_place(0, "archery");
    assert!(matches!(
        play(&mut e, archery, vec![]),
        CommandOutcome::Pending { .. }
    ));
    // 三个目标依次被询问，全部 Pass
    for p in [1u8, 2, 3] {
        assert_eq!(prompt_of(&e), "aoe_response");
        assert_eq!(e.open_choice().unwrap().observer, p, "按座位顺序询问");
        let out = answer(&mut e, ChoiceOption::Pass);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    assert_eq!(hp(&e, 1), 3);
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
}

/// 中途濒死死亡后队列继续结算后续目标。
#[test]
fn aoe_continues_after_midqueue_death() {
    let mut e = Engine::new(135, 4);
    e.debug_set_hp(1, 1);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    assert!(matches!(
        play(&mut e, savage, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    // 1 无杀直接受伤 → 濒死 → 无人救 → 死亡；队列继续：2、3 无杀受伤
    assert!(!e.debug_alive(1));
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
    assert!(log_has(&e, |l| matches!(
        l,
        LE::PlayerDied { player: 1, .. }
    )));
    assert!(log_has(&e, |l| matches!(
        l,
        LE::DamageDealt { target: 2, .. }
    )));
}

// ---------------------------------------------------------------------------
// 语义差异收敛（ADR 0705）：逐目标无懈 / 铁索无懈 / 判定无懈窗口 /
// 同名判定牌限制 / 死亡清判定区 / 身份局正式胜负与奖惩
// ---------------------------------------------------------------------------

/// AOE 逐目标无懈：抵消只作用于单目标，其余目标照常结算。
#[test]
fn savage_per_target_nullify() {
    let mut e = Engine::new(171, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    play(&mut e, savage, vec![]);
    // 目标 1 的窗口：自来源下家起询问持无懈者 → 1
    assert_eq!(prompt_of(&e), "nullify_window");
    assert_eq!(e.open_choice().unwrap().observer, 1);
    let out = answer(&mut e, ChoiceOption::Card(nul1));
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "抵消后重问无人、目标 2/3 无杀直接受伤，本段应完结: {out:?}"
    );
    // 逐目标抵消：1 免伤且不被询问响应；2、3 无杀直接受伤
    assert_eq!(hp(&e, 1), 4, "目标 1 被抵消，不受伤害");
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
    assert!(log_has(&e, |l| matches!(
        l,
        LE::CardNullified { player: 0, .. }
    )));
    // 仅目标 1 的窗口产生过一次询问（其余窗口无人持无懈，不询问）
    assert_eq!(count_log(&e, |l| matches!(l, LE::ChoiceOpened { .. })), 1);
}

/// N01：AOE 目标被无懈抵消后，锦囊由外层收尾统一弃置——弃牌堆中恰一份。
#[test]
fn savage_nullified_target_discards_once() {
    let mut e = Engine::new(171, 4);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    play(&mut e, savage, vec![]);
    let out = answer(&mut e, ChoiceOption::Card(nul1));
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    // 逐目标抵消语义：1 免伤，2/3 受伤
    assert_eq!(hp(&e, 1), 4);
    assert_eq!(hp(&e, 2), 3);
    assert_eq!(hp(&e, 3), 3);
    // 南蛮实体牌在弃牌堆恰一份（抵消弃牌 + 收尾弃牌会变两份）
    assert_eq!(discard_count(&e, savage), 1, "南蛮必须恰好弃置一次");
    assert_zone_conservation(&e);
}

// ---------------------------------------------------------------------------
// N05 补充：多目标无懈×洗牌组合守恒（无重复无丢失）
// ---------------------------------------------------------------------------

/// N05：牌库抽干后南蛮+双无懈链——无重复实体牌、无丢失。
/// 被动脚本下可能不收敛（有界跳出），但守恒断言在每步后均有效。
#[test]
fn savage_nullify_after_deck_drain_conserves() {
    let mut e = Engine::new(303, 4);
    advance_to_play(&mut e, 0);
    // 抽干牌库：纯阶段推进让各玩家摸牌，直至牌库见底
    for _ in 0..400 {
        if e.state_core().deck.is_empty() {
            break;
        }
        let out = next_phase(&mut e);
        match out {
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
                let r = e.submit(GameCommand {
                    actor: a,
                    expected_revision: e.revision(),
                    dedup_id: fresh_dedup(),
                    action: GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                });
                match r {
                    CommandOutcome::Accepted { .. } => {}
                    CommandOutcome::Pending { choice: c2 } => {
                        // 链式：继续答
                        let sel2 = vec![c2.spec.options[0]];
                        let a2 = c2.observer;
                        let cid2 = c2.id;
                        let r2 = e.submit(GameCommand {
                            actor: a2,
                            expected_revision: e.revision(),
                            dedup_id: fresh_dedup(),
                            action: GameAction::Answer {
                                choice_id: cid2,
                                selection: sel2,
                            },
                        });
                        assert!(!matches!(r2, CommandOutcome::Rejected(_)), "{r2:?}");
                    }
                    other => panic!("drain unexpected {other:?}"),
                }
            }
            CommandOutcome::Accepted { .. } => {}
            other => panic!("drain unexpected {other:?}"),
        }
    }
    // 南蛮 + 双无懈链（1 无懈 → 2 反无懈 → 南蛮生效）
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    let nul1 = e.debug_place(1, "nullify");
    let nul2 = e.debug_place(2, "nullify");
    let out = play(&mut e, savage, vec![]);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = e.submit(GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Card(nul1)],
        },
    });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let _out = e.submit(GameCommand {
        actor: 2,
        expected_revision: e.revision(),
        dedup_id: fresh_dedup(),
        action: GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Card(nul2)],
        },
    });
    // 南蛮生效后全员守恒
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
    let _total = ids.len();
    let dups: Vec<u32> = ids
        .windows(2)
        .filter(|w| w[0] == w[1])
        .map(|w| w[0])
        .collect();
    ids.dedup();
    assert!(dups.is_empty(), "重复实体牌 {dups:?}");
    assert_eq!(ids.len(), e.catalog().cards.len(), "牌区不守恒");
}

/// AOE 冻结目标：出牌提交时冻结目标列表，后续死亡由存活过滤跳过。
#[test]
fn aoe_target_freeze_at_commit() {
    let mut e = Engine::new(305, 5);
    e.debug_set_hp(2, 1);
    advance_to_play(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    let savage = e.debug_place(0, "savage");
    // 放置后无懈窗口开启，此时目标 [1,2,3,4] 已冻结
    play(&mut e, savage, vec![]);
    // 2 号（1血）无杀 → 1 点 → 濒死 → 死亡
    // 队列继续：3、4 各无杀受伤
    let _out = next_phase(&mut e);
    // 南蛮结算完毕
    assert!(!e.debug_alive(2), "2 号应死亡");
    assert!(e.debug_alive(3) && e.debug_alive(4));
    assert_eq!(hp(&e, 3), 3);
    assert_eq!(hp(&e, 4), 3);
    assert!(log_has(&e, |l| matches!(
        l,
        LE::PlayerDied { player: 2, .. }
    )));
    assert_zone_conservation(&e);
}
