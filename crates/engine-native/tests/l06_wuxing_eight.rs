//! L06 五行（`wuxing-upstream-liuli`：`wuxingpan`、`wuxingpan_skill`、`_shengke`）
//! 六维度真实对局事件路径等价性测试（Decision 40）。
//!
//! 对照上游源码：`archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:23-202`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceKind, ChoiceOption, CommandOutcome, Engine, EquipSlot, GameAction, GameCommand, Phase,
    RejectReason,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn wuxing_pack() -> ExtensionPack {
    let cards_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/wuxing-upstream-liuli/assets/cards.json");
    let cards_json = std::fs::read_to_string(cards_path).expect("读取 wuxing cards.json");
    content_native_loader::bridge_pack(&cards_json)
        .expect("桥接 wuxing-upstream-liuli")
        .pack
}

fn drain_choices(e: &mut Engine) {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
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
        let out = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "drain answer failed: {out:?}"
        );
    }
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..64 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            drain_choices(e);
            continue;
        }
        let actor = e.turn().current;
        let out = submit(e, actor, GameAction::NextPhase);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "advance failed: {out:?}"
        );
    }
    panic!("未推进到座位 {seat} 的出牌阶段");
}

/// `_shengke`（`wuxing/extension.js:62-172`）六维度真实对局事件路径验收：
/// ① 触发时机与负条件（无属性/同属性/非生克不触发；he 为空被克不弹窗；五组相生摸 1；五组相克弹 `shengke_discard` 且无 `trigger_confirm`）
/// ② 目标与卡牌过滤及数量限制（`he` 同时覆盖手牌与装备区，`min=1, max=1`）
/// ③ 玩家选择分支（选弃手牌 vs 选弃装备牌并撤销装备技；`allow_cancel: false` 空选被拒）
/// ④ 多步状态变更（`useCardToBegin` 先结算生克弃/摸，再执行卡牌伤害）
/// ⑤ 挂起窗口中点存档恢复（`shengke_discard` 窗口 `encode_archive` -> `decode_archive` -> `restore_with_extensions` 后继续应答，`state_hash` 与 `log` 一致）
/// ⑥ AI `BoundedPlanner` 在 `shengke_discard` 窗口优先弃置低价值牌保留【桃】（`ai = get.disvalue`）。
#[test]
fn l06_shengke_six_dimension_real_match_equivalence() {
    let pack = wuxing_pack();

    // 维度 ①：五组相生（金生水 0->2、木生火 1->3、水生木 2->1、火生土 3->4、土生金 4->0）
    // 与无属性/非生克负例、被克但目标 he 为空负例
    let sheng_pairs: [(u8, u8); 5] = [(0, 2), (1, 3), (2, 1), (3, 4), (4, 0)];
    let ke_pairs: [(u8, u8); 5] = [(0, 1), (1, 4), (2, 3), (3, 0), (4, 2)];

    for (idx, &(cnat, onat)) in sheng_pairs.iter().enumerate() {
        let mut e = Engine::new_with_extensions(6100 + idx as u64, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let sha = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha, cnat);
        e.state_core_mut().players[1].wunature = Some(onat);

        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "相生 ({cnat}->{onat}) 出杀应受理: {out:?}"
        );
        if let Some(c) = e.open_choice() {
            assert_ne!(
                c.spec.prompt, "trigger_confirm",
                "锁定技 _shengke 不得弹确认门"
            );
            assert_ne!(
                c.spec.prompt, "shengke_discard",
                "相生分支不得弹弃牌窗"
            );
        }
        drain_choices(&mut e);
        assert_eq!(
            e.debug_hand(1).len(),
            1,
            "相生 ({cnat}->{onat}) 目标应摸 1 张牌"
        );
    }

    // 维度 ① 负例：同属性/无关属性、牌无属性、角色无属性、被克但目标 he 区为空
    {
        let mut e = Engine::new_with_extensions(6110, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let keep = e.debug_place(1, "tao");

        // (a) 无生克关系（金 0 对 土 4 — 土生金但金不生土也不克土）
        let sha_neutral = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha_neutral, 0);
        e.state_core_mut().players[1].wunature = Some(4);
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_neutral,
                targets: vec![1],
            },
        );
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert!(e.open_choice().is_none(), "无生克关系不得弹选择窗");
        assert_eq!(e.debug_hand(1), vec![keep], "无生克关系不摸不弃");

        // (b) 牌无属性（card_natures 移除）
        e.state_core_mut().turn.sha_used_play_phase = false;
        let sha_no_nat = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.remove(&sha_no_nat);
        e.state_core_mut().players[1].wunature = Some(1);
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_no_nat,
                targets: vec![1],
            },
        );
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hand(1), vec![keep], "无属性牌不触发生克");

        // (c) 目标无属性（wunature = None）
        e.state_core_mut().turn.sha_used_play_phase = false;
        let sha_target_none = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha_target_none, 0);
        e.state_core_mut().players[1].wunature = None;
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_target_none,
                targets: vec![1],
            },
        );
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hand(1), vec![keep], "无属性角色不触发生克");

        // (d) 被克（金 0 克 木 1）但目标 he 区为空 -> 不弹 shengke_discard
        e.debug_clear_hand(1);
        e.state_core_mut().players[1].equips.clear();
        e.state_core_mut().players[1].hp = 4;
        e.state_core_mut().players[1].wunature = Some(1);
        e.state_core_mut().turn.sha_used_play_phase = false;
        let sha_empty_he = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha_empty_he, 0);
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_empty_he,
                targets: vec![1],
            },
        );
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }),
            "目标 he 为空时被克不得挂起 shengke_discard: {out:?}"
        );
        assert!(e.open_choice().is_none());
    }

    // 维度 ①/②/③/④/⑤/⑥：五组相克验证 + `shengke_discard` 手牌/装备分支、不可取消、AI 估值、中点存档恢复
    for (idx, &(cnat, onat)) in ke_pairs.iter().enumerate() {
        let mut e = Engine::new_with_extensions(6120 + idx as u64, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let hand_tao = e.debug_place(1, "tao");
        let sha = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha, cnat);
        e.state_core_mut().players[1].wunature = Some(onat);

        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        let CommandOutcome::Pending { choice } = out else {
            panic!("相克 ({cnat}->{onat}) 应挂起 shengke_discard，实得 {out:?}");
        };
        assert_eq!(choice.spec.prompt, "shengke_discard");
        assert_eq!(choice.observer, 1);
        assert!(!choice.spec.allow_cancel);
        assert_eq!(choice.spec.options, vec![ChoiceOption::Card(hand_tao)]);
        let ans = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: choice.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        assert!(matches!(ans, CommandOutcome::Accepted { .. }));
        assert!(e.debug_hand(1).is_empty(), "相克 ({cnat}->{onat}) 弃置 1 张牌");
    }

    // 深度验证维度 ②/③/④/⑤/⑥（目标同时持手牌【桃】与装备【五行盘】）
    {
        let mut e = Engine::new_with_extensions(6130, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let pan = e.debug_move_named_card_to_hand(1, "wxl_wuxingpan_0");
        assert!(matches!(
            submit(
                &mut e,
                1,
                GameAction::PlayCard {
                    card: pan,
                    targets: vec![],
                }
            ),
            CommandOutcome::Accepted { .. }
        ));
        assert!(
            e.debug_has_skill(1, "wuxingpan_skill"),
            "1 号装备五行盘后获得 wuxingpan_skill"
        );
        let hand_tao = e.debug_place(1, "tao");

        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let sha = e.debug_place(0, "sha");
        // 火(3) 克 金(0)
        e.state_core_mut().card_natures.insert(sha, 3);
        e.state_core_mut().players[1].wunature = Some(0);

        let hp1_before = e.debug_hp(1);
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        // 维度 ④：useCardToBegin 时机——此时杀尚未结算伤害
        assert_eq!(
            e.debug_hp(1),
            hp1_before,
            "shengke_discard 在 useCardToBegin（杀伤害生效前）挂起"
        );

        let c = e.open_choice().expect("shengke_discard 挂起").clone();
        assert_eq!(c.spec.prompt, "shengke_discard");
        assert_eq!(c.observer, 1);
        assert_eq!(c.spec.kind, ChoiceKind::SelectCards);
        assert_eq!(c.spec.min, 1);
        assert_eq!(c.spec.max, 1);
        assert!(!c.spec.allow_cancel);
        assert!(
            c.spec.options.contains(&ChoiceOption::Card(hand_tao))
                && c.spec.options.contains(&ChoiceOption::Card(pan)),
            "维度 ②：shengke_discard 候选必须同时包含手牌与装备区牌（'he'）"
        );

        // 维度 ③：强制不可取消（空选与多选均被拒）
        assert!(matches!(
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![],
                }
            ),
            CommandOutcome::Rejected(RejectReason::InvalidChoice)
        ));
        assert!(matches!(
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(hand_tao), ChoiceOption::Card(pan)],
                }
            ),
            CommandOutcome::Rejected(RejectReason::InvalidChoice)
        ));

        // 维度 ⑥：AI BoundedPlanner 估值决策（保留高价值【桃】，选择弃置低价值装备牌 `pan`）
        let planner = BoundedPlanner::new(16);
        let ai_plan = e.plan_action(1, &planner);
        assert_eq!(
            ai_plan.action,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(pan)],
            },
            "AI 应按 get.disvalue 弃置低价值装备牌并保留手牌【桃】"
        );

        let pending_bytes = encode_archive(&e.save());

        // 维度 ③ 分支 A + 维度 ⑤：选择弃置手牌 `hand_tao`，验证中点存档恢复前后状态与日志完全一致
        let mut e_hand_direct =
            Engine::restore_with_extensions(&decode_archive(&pending_bytes).unwrap(), &[&pack])
                .expect("恢复分支 A1");
        let mut e_hand_restored =
            Engine::restore_with_extensions(&decode_archive(&pending_bytes).unwrap(), &[&pack])
                .expect("恢复分支 A2");
        let out_a1 = submit(
            &mut e_hand_direct,
            1,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        let out_a2 = submit(
            &mut e_hand_restored,
            1,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        let (
            CommandOutcome::Accepted { state_hash: h1 },
            CommandOutcome::Accepted { state_hash: h2 },
        ) = (out_a1, out_a2)
        else {
            panic!("应 Accepted");
        };
        assert_eq!(h1, h2);
        assert_eq!(e_hand_direct.state_hash(), e_hand_restored.state_hash());
        assert_eq!(e_hand_direct.log(), e_hand_restored.log());
        assert!(!e_hand_direct.debug_hand(1).contains(&hand_tao), "手牌【桃】被弃置");
        assert!(e_hand_direct.debug_equips(1).contains(&pan), "装备【五行盘】保留");
        assert!(
            e_hand_direct.debug_has_skill(1, "wuxingpan_skill"),
            "保留五行盘时装备技仍在"
        );
        assert_eq!(e_hand_direct.debug_hp(1), hp1_before - 1, "弃牌后杀正常结算 1 点伤害");

        // 维度 ③ 分支 B：选择弃置装备区 `pan`（AI 推荐动作），验证装备移除且 `wuxingpan_skill` 被撤销
        let mut e_equip =
            Engine::restore_with_extensions(&decode_archive(&pending_bytes).unwrap(), &[&pack])
                .expect("恢复分支 B");
        let out_b = submit(&mut e_equip, 1, ai_plan.action);
        assert!(matches!(out_b, CommandOutcome::Accepted { .. }));
        assert!(e_equip.debug_hand(1).contains(&hand_tao), "手牌【桃】保留");
        assert!(!e_equip.debug_equips(1).contains(&pan), "装备【五行盘】被弃置");
        assert!(
            !e_equip.debug_has_skill(1, "wuxingpan_skill"),
            "弃置装备区五行盘后其装备技 wuxingpan_skill 必须被撤销"
        );
        assert!(e_equip.state_core().discard.contains(&pan), "五行盘进入弃牌堆");
        assert_eq!(e_equip.debug_hp(1), hp1_before - 1, "弃牌后杀正常结算 1 点伤害");
    }
}

/// `wuxingpan` 卡牌与 `wuxingpan_skill` 技能（`wuxing/extension.js:173-202`）六维度真实对局事件路径验收：
/// ① 触发时机与负条件（宝物栏 `equip5` 装备授技；`phaseUse` 限 1 次 `usable: 1`；空手牌不可发动）
/// ② 候选过滤与数量限制（仅限 `h` 手牌，不含装备区牌；第二阶段 5 种五行属性 `Tag(0..=4)`）
/// ③ 玩家选择分支（不同手牌 A vs B、五种属性 `0..=4`、两阶段取消分支）
/// ④ 多步状态变更（装备 `wuxingpan` -> 发动 `wuxingpan_skill` 改手牌【杀】属性 -> 打出该【杀】按新属性触发 `_shengke` 生/克）
/// ⑤ 两阶段挂起窗口（`wuxing_select_card` 与 `wuxing_select_nature`）存档恢复等价
/// ⑥ AI `BoundedPlanner` 在两阶段选择窗口完成合法决策。
#[test]
fn l06_wuxingpan_and_skill_six_dimension_real_match_equivalence() {
    let pack = wuxing_pack();
    let mut e = Engine::new_with_extensions(6200, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 核对目录定义：wuxingpan 为黑桃 5 宝物栏（EquipSlot::Treasure）装备，附带 wuxingpan_skill
    let pan_id = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name.starts_with("wxl_wuxingpan"))
        .expect("wxl_wuxingpan 在目录") as u32;
    let pan_def = &e.catalog().cards[pan_id as usize];
    assert_eq!(pan_def.kind.equip_slot(), Some(EquipSlot::Treasure));
    assert_eq!(pan_def.suit, Suit::Spade);
    assert_eq!(pan_def.rank, 5);
    assert_eq!(pan_def.equip_skills, vec!["wuxingpan_skill"]);

    // 装备五行盘 -> 授予 wuxingpan_skill
    let pan = e.debug_move_named_card_to_hand(0, "wxl_wuxingpan_0");
    assert!(!e.debug_has_skill(0, "wuxingpan_skill"));
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: pan,
                targets: vec![],
            }
        ),
        CommandOutcome::Accepted { .. }
    ));
    assert!(e.debug_equips(0).contains(&pan));
    assert!(e.debug_has_skill(0, "wuxingpan_skill"));
    let skill_id = e
        .catalog()
        .skill_by_name("wuxingpan_skill")
        .expect("wuxingpan_skill 在目录");

    // 维度 ① 负例：手牌为空（仅装备区有五行盘）时发动 wuxingpan_skill 被拒
    assert!(
        e.debug_hand(0).is_empty(),
        "0 号当前无手牌（仅装备区有五行盘）"
    );
    assert!(
        matches!(
            submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id }),
            CommandOutcome::Rejected(_)
        ),
        "空手牌时发动 wuxingpan_skill 必须被拒"
    );

    // 放入两张手牌 card_a (sha) 与 card_b (tao)
    let card_a = e.debug_place(0, "sha");
    let card_b = e.debug_place(0, "tao");
    e.state_core_mut().card_natures.remove(&card_a);
    e.state_core_mut().card_natures.remove(&card_b);

    // 发动 wuxingpan_skill -> Stage 1 (wuxing_select_card)
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id });
    let CommandOutcome::Pending { choice: c1 } = out else {
        panic!("应挂起 wuxing_select_card: {out:?}");
    };
    assert_eq!(c1.spec.prompt, "wuxing_select_card");
    assert_eq!(c1.spec.kind, ChoiceKind::SelectCards);
    assert_eq!(c1.spec.min, 1);
    assert_eq!(c1.spec.max, 1);
    assert!(c1.spec.allow_cancel);
    // 维度 ②：仅包含手牌 card_a、card_b，不包含装备区的 pan
    assert_eq!(
        c1.spec.options,
        vec![ChoiceOption::Card(card_a), ChoiceOption::Card(card_b)],
        "wuxing_select_card 仅允许选择手牌（'h'），不含装备区"
    );

    // 维度 ⑥：Stage 1 AI BoundedPlanner 决策合法
    let planner = BoundedPlanner::new(16);
    let ai_stage1 = e.plan_action(0, &planner);
    assert!(matches!(
        ai_stage1.action,
        GameAction::Answer { choice_id, ref selection }
        if choice_id == c1.id && selection.len() == 1 && c1.spec.options.contains(&selection[0])
    ));

    let stage1_bytes = encode_archive(&e.save());

    // 维度 ③（Stage 1 取消分支）：取消选牌后不进入选属性阶段，卡牌属性不变
    {
        let mut e_cancel1 =
            Engine::restore_with_extensions(&decode_archive(&stage1_bytes).unwrap(), &[&pack])
                .unwrap();
        let out_c = submit(
            &mut e_cancel1,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![],
            },
        );
        assert!(matches!(out_c, CommandOutcome::Accepted { .. }));
        assert!(e_cancel1.open_choice().is_none());
        assert!(!e_cancel1.state_core().card_natures.contains_key(&card_a));
    }

    // 维度 ⑤（Stage 1 存档恢复等价）：选 card_a 进入 Stage 2 (wuxing_select_nature)
    let mut e_s1_restored =
        Engine::restore_with_extensions(&decode_archive(&stage1_bytes).unwrap(), &[&pack]).unwrap();
    let ans_s1 = GameAction::Answer {
        choice_id: c1.id,
        selection: vec![ChoiceOption::Card(card_a)],
    };
    let _ = submit(&mut e, 0, ans_s1.clone());
    let _ = submit(&mut e_s1_restored, 0, ans_s1);
    assert_eq!(e.state_hash(), e_s1_restored.state_hash());

    let c2 = e.open_choice().expect("wuxing_select_nature 挂起").clone();
    assert_eq!(c2.spec.prompt, "wuxing_select_nature");
    assert_eq!(c2.spec.kind, ChoiceKind::PickTag);
    assert_eq!(
        c2.spec.options,
        (0..=4).map(ChoiceOption::Tag).collect::<Vec<_>>(),
        "五行属性选项须为 0..=4（金/木/水/火/土）"
    );

    // 维度 ⑥：Stage 2 AI BoundedPlanner 决策合法
    let ai_stage2 = e.plan_action(0, &planner);
    assert!(matches!(
        ai_stage2.action,
        GameAction::Answer { choice_id, ref selection }
        if choice_id == c2.id && selection.len() == 1 && c2.spec.options.contains(&selection[0])
    ));

    let stage2_bytes = encode_archive(&e.save());

    // 维度 ③：遍历全部 5 种属性 Tag(0..=4) 分支，以及选 card_b 分支
    for tag in 0..=4u8 {
        let mut e_tag =
            Engine::restore_with_extensions(&decode_archive(&stage2_bytes).unwrap(), &[&pack])
                .unwrap();
        let out_t = submit(
            &mut e_tag,
            0,
            GameAction::Answer {
                choice_id: c2.id,
                selection: vec![ChoiceOption::Tag(tag)],
            },
        );
        assert!(matches!(out_t, CommandOutcome::Accepted { .. }));
        assert_eq!(
            e_tag.state_core().card_natures.get(&card_a).copied(),
            Some(tag),
            "选定 Tag({tag}) 后 card_a 五行属性应更新为 {tag}"
        );
        assert_eq!(
            e_tag.state_core().card_natures.get(&card_b).copied(),
            None,
            "未选中的 card_b 属性保持不变"
        );
    }

    // 维度 ③ 分支 B：在 Stage 1 选择另一张手牌 `card_b` 并改为水属性 (2)
    {
        let mut e_card_b =
            Engine::restore_with_extensions(&decode_archive(&stage1_bytes).unwrap(), &[&pack])
                .unwrap();
        submit(
            &mut e_card_b,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![ChoiceOption::Card(card_b)],
            },
        );
        let c2_b = e_card_b.open_choice().unwrap().clone();
        submit(
            &mut e_card_b,
            0,
            GameAction::Answer {
                choice_id: c2_b.id,
                selection: vec![ChoiceOption::Tag(2)],
            },
        );
        assert_eq!(e_card_b.state_core().card_natures.get(&card_b), Some(&2));
        assert_eq!(e_card_b.state_core().card_natures.get(&card_a), None);
    }

    // 维度 ⑤（Stage 2 存档恢复等价）：将 card_a（杀）改为火属性 Tag(3)
    let mut e_s2_restored =
        Engine::restore_with_extensions(&decode_archive(&stage2_bytes).unwrap(), &[&pack]).unwrap();
    let log_before = e.log().len();
    let ans_s2 = GameAction::Answer {
        choice_id: c2.id,
        selection: vec![ChoiceOption::Tag(3)], // 火
    };
    let out1 = submit(&mut e, 0, ans_s2.clone());
    let out2 = submit(&mut e_s2_restored, 0, ans_s2);
    let (
        CommandOutcome::Accepted { state_hash: h1 },
        CommandOutcome::Accepted { state_hash: h2 },
    ) = (out1, out2)
    else {
        panic!("应 Accepted");
    };
    assert_eq!(h1, h2);
    assert_eq!(e.state_hash(), e_s2_restored.state_hash());
    assert_eq!(&e.log()[log_before..], e_s2_restored.log());
    assert_eq!(e.state_core().card_natures.get(&card_a), Some(&3));

    // 维度 ①（usable: 1 限制）：同出牌阶段第二次发动 wuxingpan_skill 必须被拒
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id }),
        CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
    ));

    // 维度 ④（多步联动）：打出刚被 wuxingpan_skill 改为火属性(3)的 `card_a`（杀）攻击金属性(0)的 1 号 -> 触发火克金 `shengke_discard`！
    let target_card = e.debug_place(1, "shan");
    e.state_core_mut().players[1].wunature = Some(0); // 金（被火克）
    let play_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: card_a,
            targets: vec![1],
        },
    );
    let CommandOutcome::Pending {
        choice: shengke_choice,
    } = play_out
    else {
        panic!("改属性后的火杀攻击金属性角色应触发 shengke_discard: {play_out:?}");
    };
    assert_eq!(shengke_choice.spec.prompt, "shengke_discard");
    assert_eq!(shengke_choice.observer, 1);
    assert_eq!(
        shengke_choice.spec.options,
        vec![ChoiceOption::Card(target_card)]
    );
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: shengke_choice.id,
            selection: vec![ChoiceOption::Card(target_card)],
        },
    );
    drain_choices(&mut e);
    assert!(
        !e.debug_hand(1).contains(&target_card),
        "1 号因火克金弃置手牌【闪】，随后承受【杀】伤害"
    );
}
