//! NC08 第⑦层（上游行为对照）样本 146：**来源 U06（swd）** 法器
//! `cangchizhibi`（苍螭之璧）——**手持 `jiqi` 被动卡（`global: "g_cangchizhibi"`）**。
//!
//! 上游源码（`extension/杀海拾遗/card/swd/{card,skill,list}.js`，**逐字**）：
//!
//! ```js
//! // list.js
//! ["spade", 7, "cangchizhibi"],
//!
//! // card.js
//! cangchizhibi: {
//!     type: "jiqi",
//!     fullskin: true,
//!     global: "g_cangchizhibi",
//!     ai: { value: 8, useful: 6.5 },
//! },
//!
//! // skill.js
//! g_cangchizhibi: {
//!     trigger: { player: "phaseBegin" },
//!     direct: true,
//!     filter(event, player) {
//!         return player.countCards("h", "cangchizhibi") > 0;
//!     },
//!     content() {
//!         "step 0";
//!         player.chooseTarget([1, 3], get.prompt("cangchizhibi")).ai = function (target) {
//!             if (target.isLinked()) {
//!                 return get.attitude(player, target);
//!             }
//!             return -get.attitude(player, target);
//!         };
//!         "step 1";
//!         if (result.bool) {
//!             player.logSkill("cangchizhibi", result.targets);
//!             for (var i = 0; i < result.targets.length; i++) {
//!                 result.targets[i].link();
//!             }
//!         }
//!     },
//! },
//! ```
//!
//! **裁决 33（2026-11-08）**：按选项 A 将 `cangchizhibi` 补入 `shahai-swd-upstream-noname`
//! 正式包（90→91 张），移除旧转换缺口断言与误用 `jiguantong` 模板生成的弃牌探针，
//! 验证准备阶段（回合开始 `TurnBegin`）手持 `cangchizhibi` 时可选 1–3 名角色横置/重置（`link()`），
//! 支持取消、存档恢复与未持牌不触发负例。

mod nc08_common;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};
use nc08_common::{next_dedup, put_hand};

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/shahai-swd-upstream-noname/assets");
    let cards = std::fs::read_to_string(dir.join("cards.json"))
        .expect("读 swd cards")
        .replace("\r\n", "\n");
    content_native_loader::bridge_pack(&cards).expect("桥接").pack
}

/// 推进直到触发 `cangchizhibi_target` 窗口或到达指定步数上限。
fn advance_until_cangchizhibi_window(e: &mut Engine, max_steps: usize) -> bool {
    for _ in 0..max_steps {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "cangchizhibi_target" {
                return true;
            }
        }
        let actor = e.turn().current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        if !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ) {
            break;
        }
        while let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "cangchizhibi_target" {
                return true;
            }
            let sel: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.max.max(1)).copied().collect()
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
    false
}

fn place_unique_in_hand(e: &mut Engine, player: u8, card_id: engine_native::CardId) {
    let core = e.state_core_mut();
    core.deck.retain(|c| *c != card_id);
    core.discard.retain(|c| *c != card_id);
    core.table.retain(|c| *c != card_id);
    for p in &mut core.players {
        p.hand.retain(|c| *c != card_id);
        p.equips.retain(|c| *c != card_id);
        p.judge.retain(|c| *c != card_id);
    }
    core.players[player as usize].hand.push(card_id);
}

/// 上游 `g_cangchizhibi`：手持苍螭之璧时在回合开始开启 `1..=3` 目标选择窗，
/// 对所选目标逐一执行 `link()`（翻转 `linked` 横置状态），支持窗口处存档恢复。
#[test]
fn upstream_cangchizhibi_links_one_to_three_targets_on_turn_begin() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20288101, 4, &[&pack]).expect("建局");
    let (card_id, suit, rank) = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("cangchizhibi"))
        .map(|(i, c)| (i as engine_native::CardId, c.suit, c.rank))
        .expect("苍螭之璧应在 swd 目录中");
    assert_eq!(suit, engine_native::catalog::Suit::Spade);
    assert_eq!(rank, 7);
    place_unique_in_hand(&mut e, 0, card_id);

    // 预置 2 号玩家已横置（验证 link() 为翻转而非单向置 true）
    e.state_core_mut().players[2].linked = true;
    assert!(!e.state_core().player(1).linked);
    assert!(e.state_core().player(2).linked);
    assert!(!e.state_core().player(3).linked);

    assert!(
        advance_until_cangchizhibi_window(&mut e, 260),
        "0 号手持苍螭之璧到达下一回合开始时应开启 cangchizhibi_target 窗口"
    );

    let choice = e.open_choice().expect("cangchizhibi_target 挂起").clone();
    assert_eq!(choice.observer, 0);
    assert_eq!(choice.spec.prompt, "cangchizhibi_target");
    assert_eq!(choice.spec.min, 1);
    assert_eq!(choice.spec.max, 3);
    assert!(choice.spec.allow_cancel);

    // 在挂起窗口处验证存档编解码与恢复一致性
    let encoded = encode_archive(&e.save());
    let decoded = decode_archive(&encoded).expect("解码存档");
    let mut restored = Engine::restore_with_extensions(&decoded, &[&pack]).expect("恢复引擎");
    assert_eq!(e.state_hash(), restored.state_hash());

    let selection = vec![
        ChoiceOption::Player(1),
        ChoiceOption::Player(2),
        ChoiceOption::Player(3),
    ];
    for engine in [&mut e, &mut restored] {
        let out = engine.submit(GameCommand {
            actor: 0,
            expected_revision: engine.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: selection.clone(),
            },
        });
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "提交 1..=3 目标横置应被受理：{out:?}"
        );
    }

    assert_eq!(e.state_hash(), restored.state_hash(), "恢复后步进哈希须一致");
    assert!(!e.state_core().player(0).linked, "未选中的 0 号保持未横置");
    assert!(e.state_core().player(1).linked, "1 号应由未横置翻转为横置");
    assert!(!e.state_core().player(2).linked, "2 号应由已横置翻转为重置");
    assert!(e.state_core().player(3).linked, "3 号应由未横置翻转为横置");
    assert!(
        e.debug_hand(0).contains(&card_id),
        "苍螭之璧作为手持法器触发后仍留在手牌中"
    );
}

/// 取消与未持牌负例：窗口取消不改变横置状态；未持苍螭之璧不开窗口。
#[test]
fn upstream_cangchizhibi_cancel_and_not_held_negative_cases() {
    let pack = swd_pack();
    let mut e = Engine::new_with_extensions(20288102, 4, &[&pack]).expect("建局");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let card_id = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("cangchizhibi"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("苍螭之璧应在 swd 目录中");
    put_hand(&mut e, 0, card_id);

    assert!(
        advance_until_cangchizhibi_window(&mut e, 260),
        "手持苍螭之璧应开启窗口"
    );
    let choice = e.open_choice().expect("窗口").clone();
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: Vec::new(),
        },
    });
    assert!(matches!(
        out,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    for p in 0..4 {
        assert!(!e.state_core().player(p).linked, "取消后 {p} 号不应被横置");
    }

    // 移除手牌中的苍螭之璧后，再推进一轮不应开启窗口
    e.debug_clear_hand(0);
    assert!(
        !advance_until_cangchizhibi_window(&mut e, 40),
        "未持有苍螭之璧时不应开启 cangchizhibi_target 窗口"
    );
}

/// 边界与 AI 规划验证：
/// 1) 同一名角色手牌持有两张苍螭之璧时，准备阶段 `g_cangchizhibi`（`countCards('h','cangchizhibi') > 0`）仅触发一次；
/// 2) 窗口拒绝超上限（4 名目标）与重复目标（`[Player(1), Player(1)]`）；
/// 3) AI `BoundedPlanner` 在 `cangchizhibi_target` 窗口零故障（`sim_faults == 0`）给出合法决策；
/// 4) 同属 `swd_equipment.rs` 的手持法器 `qinglongzhigui`（青龙之圭）在 `qinglongzhigui_discard` 窗口支持存档恢复。
#[test]
fn upstream_cangchizhibi_multiple_copies_invalid_choices_and_ai_plan() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::RejectReason;

    let mut pack = swd_pack();
    // 额外复制一张苍螭之璧，验证手持多张时全局触发技每回合仅触发一次
    let mut extra = pack
        .cards
        .iter()
        .find(|c| c.name.contains("cangchizhibi"))
        .expect("cangchizhibi 存在")
        .clone();
    extra.name = "swd_cangchizhibi_91";
    pack.cards.push(extra);

    let mut e = Engine::new_with_extensions(20288103, 4, &[&pack]).expect("建局");
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let cangchi_ids: Vec<engine_native::CardId> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter(|(_, c)| c.name.contains("cangchizhibi"))
        .map(|(i, _)| i as engine_native::CardId)
        .collect();
    assert_eq!(cangchi_ids.len(), 2, "测试牌堆应含 2 张苍螭之璧");
    for id in &cangchi_ids {
        place_unique_in_hand(&mut e, 0, *id);
    }

    assert!(
        advance_until_cangchizhibi_window(&mut e, 260),
        "手持两张苍螭之璧应开启窗口"
    );
    let choice = e.open_choice().expect("窗口").clone();
    assert_eq!(choice.spec.prompt, "cangchizhibi_target");

    // 非法选择 1：超过 max=3（选 4 名角色）
    let out_over = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![
                ChoiceOption::Player(0),
                ChoiceOption::Player(1),
                ChoiceOption::Player(2),
                ChoiceOption::Player(3),
            ],
        },
    });
    assert!(matches!(
        out_over,
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ));

    // 非法选择 2：重复目标
    let out_dup = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(1)],
        },
    });
    assert!(matches!(
        out_dup,
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ));

    // AI 规划器在 cangchizhibi_target 窗口下应零故障生成合法动作
    let planner = BoundedPlanner::new(16).with_lookahead(2);
    let plan = e.plan_action(0, &planner);
    assert_eq!(plan.sim_faults, 0);
    let out_plan = e.apply_plan(&plan).expect("应用 AI 计划");
    assert!(matches!(
        out_plan,
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));

    // 本回合内已结算过一次 g_cangchizhibi，不应因手握第 2 张而连续二次弹出 cangchizhibi_target
    if let Some(after) = e.open_choice() {
        assert_ne!(
            after.spec.prompt, "cangchizhibi_target",
            "手握多张苍螭之璧在同一准备阶段只应触发一次"
        );
    }

    // 验证同属 swd_equipment.rs 的手持法器 qinglongzhigui 在 qinglongzhigui_discard 窗口存档恢复
    let std_pack = swd_pack();
    let mut eq = Engine::new_with_extensions(20288104, 4, &[&std_pack]).expect("建局");
    for p in 0..4 {
        eq.debug_clear_hand(p);
    }
    let qinglong_id = eq
        .catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.name.contains("qinglongzhigui"))
        .map(|(i, _)| i as engine_native::CardId)
        .expect("青龙之圭应在 swd 目录中");
    place_unique_in_hand(&mut eq, 0, qinglong_id);
    for _ in 0..260 {
        if eq
            .open_choice()
            .is_some_and(|c| c.spec.prompt == "qinglongzhigui_discard")
        {
            break;
        }
        let actor = eq.turn().current;
        let _ = eq.submit(GameCommand {
            actor,
            expected_revision: eq.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        while let Some(c) = eq.open_choice().cloned() {
            if c.spec.prompt == "qinglongzhigui_discard" {
                break;
            }
            let sel = if c.spec.prompt == "trigger_confirm" {
                vec![ChoiceOption::Confirm]
            } else if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            let _ = eq.submit(GameCommand {
                actor: c.observer,
                expected_revision: eq.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            });
        }
    }
    let q_choice = eq
        .open_choice()
        .expect("应停在 qinglongzhigui_discard 窗口")
        .clone();
    assert_eq!(q_choice.spec.prompt, "qinglongzhigui_discard");
    let q_bytes = encode_archive(&eq.save());
    let q_decoded = decode_archive(&q_bytes).expect("解码青龙之圭窗口存档");
    let q_restored =
        Engine::restore_with_extensions(&q_decoded, &[&std_pack]).expect("恢复青龙之圭窗口");
    assert_eq!(eq.state_hash(), q_restored.state_hash());
}

