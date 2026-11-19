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

#[test]
fn l06_wuxingpan_stage1_cancel_refund_and_stage2_mandatory() {
    let pack = wuxing_pack();
    let mut e = Engine::new_with_extensions(6201, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let pan = e.debug_move_named_card_to_hand(0, "wxl_wuxingpan_0");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: pan, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));
    let card_a = e.debug_place(0, "sha");
    let _secret_p2 = e.debug_place(2, "tao");
    e.state_core_mut().card_natures.remove(&card_a);
    let skill_id = e.catalog().skill_by_name("wuxingpan_skill").expect("wuxingpan_skill");

    // 1. Stage 1 选牌可取消且不得消耗 usable:1 次数（含中途存档恢复一致）
    let out1 = submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id });
    let CommandOutcome::Pending { choice: c1 } = out1 else {
        panic!("Stage 1 应挂起 wuxing_select_card: {out1:?}");
    };
    assert!(c1.spec.allow_cancel);
    let bytes1 = encode_archive(&e.save());
    let mut e_r1 = Engine::restore_with_extensions(&decode_archive(&bytes1).unwrap(), &[&pack]).unwrap();
    let cancel_cmd = GameAction::Answer {
        choice_id: c1.id,
        selection: vec![],
    };
    let (res_a, res_b) = (submit(&mut e, 0, cancel_cmd.clone()), submit(&mut e_r1, 0, cancel_cmd));
    match (res_a, res_b) {
        (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) => {
            assert_eq!(h1, h2);
        }
        other => panic!("Stage 1 取消应 Accepted: {other:?}"),
    }
    assert!(
        !e.save().core.skills[&(0, skill_id)].once_used,
        "wuxingpan_skill 第一步选牌取消后不得消耗 once_used"
    );

    // 同一出牌阶段必须能再次发动 wuxingpan_skill
    let out2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id });
    let CommandOutcome::Pending { choice: c1_again } = out2 else {
        panic!("第一步取消后同阶段应可再次发动 wuxingpan_skill: {out2:?}");
    };
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c1_again.id,
                selection: vec![ChoiceOption::Card(card_a)],
            }
        ),
        CommandOutcome::Pending { .. }
    ));

    // 2. Stage 2 属性选择对齐上游 chooseControl(list)：allow_cancel 必须为 false（空选被拒），含存档恢复与 AI PolicyInput
    let c2 = e.open_choice().expect("wuxing_select_nature").clone();
    assert_eq!(c2.spec.prompt, "wuxing_select_nature");
    assert!(
        !c2.spec.allow_cancel,
        "wuxingpan_skill 第二步属性选择对齐上游 chooseControl(list) 必须不可取消"
    );
    let bytes2 = encode_archive(&e.save());
    let mut e_r2 = Engine::restore_with_extensions(&decode_archive(&bytes2).unwrap(), &[&pack]).unwrap();
    let mut e_alt = Engine::restore_with_extensions(&decode_archive(&bytes2).unwrap(), &[&pack]).unwrap();
    assert!(matches!(
        submit(&mut e, 0, GameAction::Answer { choice_id: c2.id, selection: vec![] }),
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ));
    assert!(matches!(
        submit(&mut e_r2, 0, GameAction::Answer { choice_id: c2.id, selection: vec![] }),
        CommandOutcome::Rejected(RejectReason::InvalidChoice)
    ));

    // AI 仅用公开信息决策（改 2 号手牌不改变 AI 选择）
    let planner = BoundedPlanner::new(16);
    let ai_plan = e.plan_action(0, &planner).action;
    let mut e_blind = e.clone();
    e_blind.debug_clear_hand(2);
    let _ = e_blind.debug_place(2, "jiu");
    assert_eq!(e_blind.plan_action(0, &planner).action, ai_plan);

    // 选 A (火=3) vs 选 B (水=2) 均成功并消耗次数
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c2.id,
                selection: vec![ChoiceOption::Tag(3)],
            }
        ),
        CommandOutcome::Accepted { .. }
    ));
    assert!(matches!(
        submit(
            &mut e_alt,
            0,
            GameAction::Answer {
                choice_id: c2.id,
                selection: vec![ChoiceOption::Tag(2)],
            }
        ),
        CommandOutcome::Accepted { .. }
    ));
    assert_eq!(e.state_core().card_natures.get(&card_a), Some(&3));
    assert_eq!(e_alt.state_core().card_natures.get(&card_a), Some(&2));
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: skill_id }),
        CommandOutcome::Rejected(RejectReason::SkillAlreadyUsed)
    ));
}

#[test]
fn l06_shengke_connect_mode_and_target_changes() {
    let pack = wuxing_pack();

    // 1. 联机模式守卫（connect_mode = 1 时禁用 _shengke，对照 connect_mode = 0 正常触发）
    for connect_mode in [1u32, 0u32] {
        let mut e = Engine::new_with_extensions(6202, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        if connect_mode != 0 {
            e.state_core_mut()
                .vm_aux
                .flags
                .insert((0, "connect_mode".to_string()), connect_mode);
        }
        e.state_core_mut().players[1].wunature = Some(0); // 金（被火克）
        e.state_core_mut().players[2].wunature = Some(4); // 土（被火生）
        let t1_card = e.debug_place(1, "shan");
        let aoe_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == engine_native::CardKind::SavageAttack)
            .expect("SavageAttack 在目录")
            .name;
        let aoe = e.debug_place(0, aoe_name);
        e.state_core_mut().card_natures.insert(aoe, 3); // 火属性南蛮入侵（多目标 1, 2）
        let p2_hand_before = e.debug_hand(2).len();

        let out = submit(&mut e, 0, GameAction::PlayCard { card: aoe, targets: vec![] });
        if connect_mode == 1 {
            // 联机禁用：不会先弹 shengke_discard，而是直接进入南蛮杀询问或完成
            while let Some(c) = e.open_choice().cloned() {
                assert_ne!(
                    c.spec.prompt, "shengke_discard",
                    "connect_mode=1 时 _shengke 必须被禁用"
                );
                submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: vec![] });
            }
            assert!(e.debug_hand(1).contains(&t1_card), "联机禁用时不得克牌弃置");
            assert_eq!(e.debug_hand(2).len(), p2_hand_before, "联机禁用时不得生牌摸 1");
        } else {
            // 非联机：多目标南蛮先逐目标结算 _shengke（1 号 shengke_discard，2 号火生土摸 1）
            assert!(matches!(out, CommandOutcome::Pending { .. }));
            let sk = e.open_choice().expect("应挂起 shengke_discard").clone();
            assert_eq!(sk.spec.prompt, "shengke_discard");
            assert_eq!(sk.observer, 1);

            // 存档恢复 + AI PolicyInput 公开信息决策
            let bytes = encode_archive(&e.save());
            let mut e_restored =
                Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
            let planner = BoundedPlanner::new(16);
            let ai_plan = e.plan_action(1, &planner).action;
            let mut e_blind = e.clone();
            e_blind.debug_clear_hand(0);
            let _ = e_blind.debug_place(0, "tao");
            assert_eq!(e_blind.plan_action(1, &planner).action, ai_plan);

            let ans = GameAction::Answer {
                choice_id: sk.id,
                selection: vec![ChoiceOption::Card(t1_card)],
            };
            submit(&mut e, 1, ans.clone());
            submit(&mut e_restored, 1, ans);
            assert_eq!(e.state_hash(), e_restored.state_hash());
            drain_choices(&mut e);
            assert!(!e.debug_hand(1).contains(&t1_card), "1 号火克金弃牌");
            assert_eq!(e.debug_hand(2).len(), p2_hand_before + 1, "2 号火生土摸 1");
        }
    }

    // 2. 目标被移除（Timing::Normal 触发技 `shouxi` 取消目标并置 u.cancelled）→ _shengke (Timing::Last) 不得触发
    {
        let p9_pack = content_native_loader::decision34_pack9_for_coexistence(&[&pack]);
        let packs: Vec<&ExtensionPack> = vec![&pack, &p9_pack];
        let mut e = Engine::new_with_extensions(6203, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "shouxi");
        e.state_core_mut().players[1].wunature = Some(0); // 金（被火克）
        let t1_card = e.debug_place(1, "tao");
        let sha = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha, 3); // 火杀

        let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let first_ask = e.open_choice().expect("应先问 shouxi 而非 wuxing_shengke").clone();
        assert_ne!(
            first_ask.spec.prompt, "shengke_discard",
            "_shengke 须处于 Timing::Last，不得先于 Normal 阶 shouxi 弹窗"
        );
        // 1 号先打开 shouxi:declare_kind → 0 号手牌为空自动判定无效并移除目标 1
        assert_eq!(first_ask.spec.prompt, "shouxi:declare_kind");
        submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: first_ask.id,
                selection: vec![first_ask.spec.options[0]],
            },
        );
        drain_choices(&mut e);
        assert!(
            e.debug_hand(1).contains(&t1_card),
            "被 shouxi 移除目标/取消后，_shengke 不得触发"
        );
    }

    // 3. 目标在 Timing::Normal 被追加到更前顺位（2 号触发 Timing::Normal 追加 0 号为目标）→ _shengke (Timing::Last) 对新增目标照常触发
    {
        let add_t_src = r#"
skill test_shengke_add_target(99110001) {
    trigger card_used owner_alive optional {
        effect choose_add_or_remove_current_card_target(self, event_card);
    }
}
"#;
        let pkg = content_compiler::compile_source_named("test_shengke_add_target", add_t_src)
            .expect("compile test_shengke_add_target");
        let vm_out = content_native_loader::bridge_vm_skill_decls(&pkg).expect("bridge");
        let vm_pack = ExtensionPack {
            def: engine_native::content::extension::ExtensionDef {
                name: "test-shengke-add-target",
                version: 1,
                description: "test Timing::Last target add",
            },
            generals: Vec::new(),
            cards: Vec::new(),
            host_skills: Vec::new(),
            vm_skills: vm_out.decls,
            init_hook: engine_native::content::extension::InitHook::None,
            enabled: true,
        };
        let packs: Vec<&ExtensionPack> = vec![&pack, &vm_pack];
        let mut e = Engine::new_with_extensions(6204, 3, &packs).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(2, "test_shengke_add_target");
        e.state_core_mut().players[1].wunature = Some(4); // 土（被火生，本应在 Timing::Normal 已跳过 rel=1）
        e.state_core_mut().players[2].wunature = Some(2); // 水（火对水无生克关系）
        let sha = e.debug_place(0, "sha");
        e.state_core_mut().card_natures.insert(sha, 3); // 火属性
        let p1_hand_before = e.debug_hand(1).len();

        let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![2] });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "out={out:?}");
        let add_ask = e.open_choice().expect("2 号应询问追加目标确认").clone();
        assert_eq!(add_ask.spec.prompt, "trigger_confirm");
        assert_eq!(add_ask.observer, 2);
        submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: add_ask.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        let pick_extra = e.open_choice().expect("2 号选择追加目标 1 号").clone();
        assert_eq!(pick_extra.spec.prompt, "rvm_choice");
        submit(
            &mut e,
            2,
            GameAction::Answer {
                choice_id: pick_extra.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        drain_choices(&mut e);
        assert_eq!(
            e.debug_hand(1).len(),
            p1_hand_before + 1,
            "2 号在 Normal 阶追加前序 1 号为目标后，Timing::Last 的 _shengke 须对新增目标 1 号触发摸 1 牌"
        );
    }

    // 4. u.cancelled == true 时即使 targets 仍含 owner 也不得触发 _shengke
    {
        let mut e = Engine::new_with_extensions(6205, 3, &[&pack]).expect("建局");
        e.state_core_mut().players[1].wunature = Some(0);
        let sha = e.debug_place(0, "sha");
        let _t1_card = e.debug_place(1, "shan");
        e.state_core_mut().card_natures.insert(sha, 3);
        let mut core = e.save().core;
        let mut res = engine_native::resolution::Resolution::new(21000);
        let mut logs = Vec::new();
        let _ = engine_native::resolution::push_frame(
            &mut res,
            engine_native::EventData::CardUsed(engine_native::event::CardUseData {
                card: Some(sha),
                player: 0,
                targets: vec![1],
                nature: engine_native::event::DamageNature::Normal,
                use_as: None,
                cancelled: true,
            }),
        );
        for _ in 0..32 {
            match engine_native::resolution::run(&mut core, &mut res, e.catalog(), &mut logs, Some(0)) {
                engine_native::resolution::StepOutcome::Done => break,
                engine_native::resolution::StepOutcome::Yielded => continue,
                other => panic!("cancelled CardUsed 不得打开 shengke_discard: {other:?}"),
            }
        }
    }
}

/// `_shengke` 对无显式 `targets` 的自目标/隐式群体牌（【桃】【酒】【无中】【桃园】【五谷】）
/// 及装备牌使用（`EventData::Equipped`）的真实对局触发等价性验收：
/// - `archive/upstream/noname-liuli/noname/extension/wuxing/extension.js:62-172`
/// - `archive/upstream/noname-liuli/noname/card/standard.js:845-847`（桃园 `ignoreTarget: target.isHealthy()`）
/// - `archive/upstream/noname-liuli/noname/game/game.js:19153,19547,41754`（`useCardToBegin`）
#[test]
fn l06_shengke_implicit_targets_and_equip_cards() {
    let pack = wuxing_pack();

    // 1. 自用【桃】：相生摸 1；相克在结算回血前强制 shengke_discard（Choice A 弃手牌 / Choice B 弃装备 + 中点存档恢复一致）
    {
        let mut e = Engine::new_with_extensions(6301, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(0, 2);
        e.state_core_mut().players[0].wunature = Some(4); // 土
        let tao = e.debug_place(0, "tao");
        e.state_core_mut().card_natures.insert(tao, 3); // 火生土
        let out = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Accepted { .. }), "tao sheng: {out:?}");
        assert_eq!(e.debug_hp(0), 3, "桃回复 1 点体力");
        assert_eq!(e.debug_hand(0).len(), 1, "火属性桃对土属性自己应触发火生土摸 1");
    }
    {
        let mut e = Engine::new_with_extensions(6302, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(0, 2);
        e.state_core_mut().players[0].wunature = Some(1); // 木
        let tao = e.debug_place(0, "tao");
        let hand_keep = e.debug_place(0, "sha");
        let eq_keep = e.debug_place(0, "zhuge");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: eq_keep, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        e.state_core_mut().card_natures.insert(tao, 0); // 金克木
        let out = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "金克木用桃应打开 shengke_discard: {out:?}");
        let ask = e.open_choice().expect("shengke_discard").clone();
        assert_eq!(ask.spec.prompt, "shengke_discard");
        assert_eq!(ask.observer, 0);
        assert!(!ask.spec.allow_cancel);

        // 存档恢复 + Choice A(弃手牌) vs Choice B(弃装备)
        let bytes = encode_archive(&e.save());
        let mut e_restored =
            Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let mut e_equip = e.clone();

        let log_len = e.log().len();
        let out_a = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(hand_keep)],
            },
        );
        let out_r = submit(
            &mut e_restored,
            0,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(hand_keep)],
            },
        );
        match (&out_a, &out_r) {
            (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) => {
                assert_eq!(h1, h2, "相克弃手牌存档恢复状态哈希一致");
            }
            _ => panic!("out_a={out_a:?}, out_r={out_r:?}"),
        }
        assert_eq!(&e.log()[log_len..], e_restored.log());
        assert_eq!(e.debug_hp(0), 3);
        assert!(!e.debug_hand(0).contains(&hand_keep));
        assert!(e.debug_equips(0).contains(&eq_keep));

        let out_b = submit(
            &mut e_equip,
            0,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(eq_keep)],
            },
        );
        assert!(matches!(out_b, CommandOutcome::Accepted { .. }));
        assert_eq!(e_equip.debug_hp(0), 3);
        assert!(e_equip.debug_hand(0).contains(&hand_keep));
        assert!(!e_equip.debug_equips(0).contains(&eq_keep));
    }

    // 2. 自用【酒】与【无中生有】：相生摸 1；相克在无中生有摸 2 前触发 shengke_discard
    {
        let mut e = Engine::new_with_extensions(6303, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.state_core_mut().players[0].wunature = Some(3); // 火
        let jiu = e.debug_place(0, "jiu");
        e.state_core_mut().card_natures.insert(jiu, 1); // 木生火
        let out = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hand(0).len(), 1, "木生火自用【酒】应摸 1 张牌");

        let wuzhong = e.debug_place(0, "wuzhong");
        e.state_core_mut().card_natures.insert(wuzhong, 0); // 金（金克木）
        e.state_core_mut().players[0].wunature = Some(1); // 木
        let pre_card = e.debug_hand(0)[0];
        let out_wz = submit(&mut e, 0, GameAction::PlayCard { card: wuzhong, targets: vec![] });
        assert!(matches!(out_wz, CommandOutcome::Pending { .. }), "金克木用无中应触发 shengke_discard: {out_wz:?}");
        let ask = e.open_choice().expect("shengke_discard").clone();
        assert_eq!(ask.spec.prompt, "shengke_discard");
        assert_eq!(ask.spec.options, vec![ChoiceOption::Card(pre_card)], "此时无中生有尚未摸 2，仅原手牌可选");
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(pre_card)],
            },
        );
        assert_eq!(e.debug_hand(0).len(), 2, "弃 1 后无中生有摸 2");
    }

    // 3. 【桃园结义】：受伤者触发 _shengke（火生土摸 1 / 火克金弃 1），满血者依 ignoreTarget: target.isHealthy() 不触发
    {
        let mut e = Engine::new_with_extensions(6304, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_set_hp(0, 2);
        e.state_core_mut().players[0].wunature = Some(4); // 土：火生土 -> 摸 1
        e.debug_set_hp(1, 2);
        e.state_core_mut().players[1].wunature = Some(0); // 金（受伤）：火克金 -> shengke_discard
        let p1_card = e.debug_place(1, "sha");
        e.debug_set_hp(2, 4); // 满血（4/4）！虽为金属性且有手牌，但桃园 ignoreTarget 跳过满血角色
        e.state_core_mut().players[2].wunature = Some(0);
        let p2_card = e.debug_place(2, "shan");
        let taoyuan = e.debug_place(0, "taoyuan");
        e.state_core_mut().card_natures.insert(taoyuan, 3); // 火

        let out = submit(&mut e, 0, GameAction::PlayCard { card: taoyuan, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }), "1 号受伤且被火克金应询问 shengke_discard");
        let ask = e.open_choice().expect("p1 shengke_discard").clone();
        assert_eq!(ask.observer, 1);
        assert_eq!(ask.spec.prompt, "shengke_discard");
        let out2 = submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(p1_card)],
            },
        );
        assert!(
            matches!(out2, CommandOutcome::Accepted { .. }) && e.open_choice().is_none(),
            "满血的 2 号不得触发桃园 _shengke: open={:?}",
            e.open_choice()
        );
        assert_eq!(e.debug_hand(0).len(), 1, "0 号火生土摸 1");
        assert!(e.debug_hand(1).is_empty(), "1 号弃置手牌");
        assert_eq!(e.debug_hand(2), vec![p2_card], "满血 2 号手牌不受 _shengke 影响");
    }

    // 4. 【五谷丰登】：全体存活角色均成为目标触发 _shengke
    {
        let mut e = Engine::new_with_extensions(6305, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.state_core_mut().players[0].wunature = Some(1); // 水生木 -> 摸 1
        e.state_core_mut().players[1].wunature = Some(3); // 水克火 -> 弃 1
        e.state_core_mut().players[2].wunature = Some(4); // 无关（土生木无关）
        let p1_card = e.debug_place(1, "sha");
        let wugu = e.debug_place(0, "wugu");
        e.state_core_mut().card_natures.insert(wugu, 2); // 水
        let out = submit(&mut e, 0, GameAction::PlayCard { card: wugu, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let ask = e.open_choice().expect("p1 shengke_discard").clone();
        assert_eq!(ask.observer, 1);
        assert_eq!(ask.spec.prompt, "shengke_discard");
        submit(
            &mut e,
            1,
            GameAction::Answer {
                choice_id: ask.id,
                selection: vec![ChoiceOption::Card(p1_card)],
            },
        );
        assert_eq!(e.debug_hand(0).len(), 1, "0 号在五谷结算选牌前先触发水生木摸 1");
        drain_choices(&mut e); // 完成五谷各自选牌（各自再得 1 张）
        assert_eq!(e.debug_hand(0).len(), 2);
        assert_eq!(e.debug_hand(1).len(), 1);
        assert_eq!(e.debug_hand(2).len(), 1);
    }

    // 5. 装备牌（PlayCard -> EventData::Equipped）：带属性装备使用后触发 _shengke（且不得将刚装备的牌本身作为被克可弃项）
    {
        let mut e = Engine::new_with_extensions(6306, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.state_core_mut().players[0].wunature = Some(4); // 土
        let weapon = e.debug_place(0, "zhuge");
        e.state_core_mut().card_natures.insert(weapon, 3); // 火生土
        let out = submit(&mut e, 0, GameAction::PlayCard { card: weapon, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Accepted { .. }));
        assert_eq!(e.debug_hand(0).len(), 1, "使用火属性装备牌对土属性自己应触发火生土摸 1");

        // 相克：无其他 he 牌时（仅装备该武器本身），依上游 ordering 时点 he 为空不弹窗、武器留在装备区
        let mut e2 = Engine::new_with_extensions(6307, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e2, 0);
        for p in 0..3 {
            e2.debug_clear_hand(p);
        }
        e2.state_core_mut().players[0].wunature = Some(1); // 木
        let weapon2 = e2.debug_place(0, "horse_plus");
        e2.state_core_mut().card_natures.insert(weapon2, 0); // 金克木
        let out2 = submit(&mut e2, 0, GameAction::PlayCard { card: weapon2, targets: vec![] });
        assert!(
            matches!(out2, CommandOutcome::Accepted { .. }) && e2.open_choice().is_none(),
            "使用带金属性装备时上游 useCardToBegin 时点该牌尚在处理区而非 he，仅凭其自身不得弹 shengke_discard: out={out2:?}"
        );

        // 相克且有其他手牌时弹出 shengke_discard，且选项仅含原手牌/原装备不含刚装备的 weapon3，含存档恢复一致
        let hand_other = e2.debug_place(0, "sha");
        let weapon3 = e2.debug_place(0, "horse_minus");
        e2.state_core_mut().card_natures.insert(weapon3, 0); // 金克木
        let out3 = submit(&mut e2, 0, GameAction::PlayCard { card: weapon3, targets: vec![] });
        assert!(matches!(out3, CommandOutcome::Pending { .. }));
        let ask3 = e2.open_choice().expect("shengke_discard").clone();
        assert_eq!(ask3.spec.prompt, "shengke_discard");
        assert!(ask3.spec.options.contains(&ChoiceOption::Card(hand_other)));
        assert!(ask3.spec.options.contains(&ChoiceOption::Card(weapon2)));
        assert!(!ask3.spec.options.contains(&ChoiceOption::Card(weapon3)));
        let bytes3 = encode_archive(&e2.save());
        let mut e3_restored =
            Engine::restore_with_extensions(&decode_archive(&bytes3).unwrap(), &[&pack]).unwrap();
        let out3_a = submit(
            &mut e2,
            0,
            GameAction::Answer {
                choice_id: ask3.id,
                selection: vec![ChoiceOption::Card(hand_other)],
            },
        );
        let out3_r = submit(
            &mut e3_restored,
            0,
            GameAction::Answer {
                choice_id: ask3.id,
                selection: vec![ChoiceOption::Card(hand_other)],
            },
        );
        match (&out3_a, &out3_r) {
            (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) => {
                assert_eq!(h1, h2);
            }
            _ => panic!("out3_a={out3_a:?}, out3_r={out3_r:?}"),
        }
        assert!(e2.debug_equips(0).contains(&weapon3));
        assert!(!e2.debug_hand(0).contains(&hand_other));
    }
}

/// 独立复核补遗验收：
/// 1. `wuxingpan` 自身永远不得随机获得 `card_natures`（`wuxing/extension.js:45`），且开局手牌须同样参与初始化（`:43-58`）；
/// 2. `_wuxie`（`priority: 5`）先于 `_shengke`（`priority: 0`）开窗，被无懈抵消的锦囊/目标不得触发 `_shengke`；
/// 3. 直接 `EventData::Equipped`（无 `PlayCard` 使用帧，对应 `player.equip()`）不得触发 `_shengke`。
#[test]
fn l06_wuxingpan_init_shengke_wuxie_order_and_direct_equip_guard() {
    let pack = wuxing_pack();

    // 1. 多种子校验：wuxingpan 永不获 wunature；开局手牌亦有机会带 wunature
    let mut saw_opening_hand_nature = false;
    for seed in 6400..6416 {
        let e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
        let snap = e.save();
        for (cid, _) in &snap.core.card_natures {
            assert_ne!(
                e.catalog().card_kind(*cid),
                engine_native::CardKind::WuxingPan,
                "seed={seed}: wuxingpan 自身不得分配五行属性（extension.js:45）"
            );
        }
        if snap
            .core
            .players
            .iter()
            .flat_map(|p| p.hand.iter())
            .any(|c| snap.core.card_natures.contains_key(c))
        {
            saw_opening_hand_nature = true;
        }
    }
    assert!(saw_opening_hand_nature, "开局初始手牌亦须经 card.init 随机赋予 wunature");

    // 2. _wuxie 先于 _shengke，被无懈则跳过 _shengke，放行后再触发 _shengke（含中点恢复 parity）
    for use_wuxie in [true, false] {
        let mut e = Engine::new_with_extensions(6420 + u64::from(use_wuxie), 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.state_core_mut().players[0].wunature = Some(1); // 木
        let p0_keep = e.debug_place(0, "shan");
        let wuzhong = e.debug_place(0, "wuzhong");
        e.state_core_mut().card_natures.insert(wuzhong, 0); // 金克木
        let wuxie = e.debug_place(1, "nullify");

        let out = submit(&mut e, 0, GameAction::PlayCard { card: wuzhong, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let first_ask = e.open_choice().expect("nullify_window 必须先于 shengke_discard").clone();
        assert_eq!(first_ask.spec.prompt, "nullify_window", "useCardToBegin 上 _wuxie (priority 5) 须先于 _shengke");
        assert_eq!(first_ask.observer, 1);

        if use_wuxie {
            let out_wx = submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: first_ask.id,
                    selection: vec![ChoiceOption::Card(wuxie)],
                },
            );
            assert!(
                matches!(out_wx, CommandOutcome::Accepted { .. }) && e.open_choice().is_none(),
                "被无懈抵消的锦囊不得再触发 _shengke 弃牌: out={out_wx:?}"
            );
            assert_eq!(e.debug_hand(0), vec![p0_keep], "被无懈后原手牌保持未弃置且未摸无中生有 2 牌");
        } else {
            let out_pass = submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: first_ask.id,
                    selection: vec![ChoiceOption::Pass],
                },
            );
            assert!(matches!(out_pass, CommandOutcome::Pending { .. }));
            let sk_ask = e.open_choice().expect("无懈未抵消后才弹 shengke_discard").clone();
            assert_eq!(sk_ask.spec.prompt, "shengke_discard");
            let bytes = encode_archive(&e.save());
            let mut e_restored =
                Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
            let ans = GameAction::Answer {
                choice_id: sk_ask.id,
                selection: vec![ChoiceOption::Card(p0_keep)],
            };
            let log_len = e.log().len();
            let out_a = submit(&mut e, 0, ans.clone());
            let out_r = submit(&mut e_restored, 0, ans);
            assert!(matches!(out_a, CommandOutcome::Accepted { .. }));
            assert!(matches!(out_r, CommandOutcome::Accepted { .. }));
            assert_eq!(e.state_hash(), e_restored.state_hash());
            assert_eq!(&e.log()[log_len..], e_restored.log());
            assert_eq!(e.debug_hand(0).len(), 2, "放行无懈 + _shengke 弃 1 后无中摸 2");
        }
    }

    // 2b. AOE（南蛮入侵）逐目标无懈：1 号无懈抵消不触发 _shengke，2 号未抵消须触发 _shengke
    {
        let mut e = Engine::new_with_extensions(6425, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.state_core_mut().players[1].wunature = Some(2); // 水（火对水无关）-> 或用被火生土，便于观测摸 1
        e.state_core_mut().players[1].wunature = Some(4); // 土：火生土
        e.state_core_mut().players[2].wunature = Some(4); // 土：火生土
        let wuxie = e.debug_place(0, "nullify");
        let aoe_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == engine_native::CardKind::SavageAttack)
            .expect("SavageAttack")
            .name;
        let aoe = e.debug_place(0, aoe_name);
        e.state_core_mut().card_natures.insert(aoe, 3); // 火

        submit(&mut e, 0, GameAction::PlayCard { card: aoe, targets: vec![] });
        // 先问 1 号南蛮无懈窗口（0 号有 wuxie）→ 出无懈抵消 1 号，不得给 1 号火生土摸 1
        let nw1 = e.open_choice().expect("p1 nullify_window").clone();
        assert_eq!(nw1.spec.prompt, "nullify_window");
        submit(&mut e, 0, GameAction::Answer { choice_id: nw1.id, selection: vec![ChoiceOption::Card(wuxie)] });
        assert_eq!(e.debug_hand(1).len(), 0, "1 号被无懈抵消，不得触发火生土摸 1");
        assert_eq!(e.debug_hand(2).len(), 1, "2 号无人无懈（已无牌），须在结算受伤前触发火生土摸 1");
    }

    // 3. 直接 EventData::Equipped（无 PlayCard useCard 标志，即非 useCard 的 direct equip）不得触发 _shengke
    {
        let mut e = Engine::new_with_extensions(6430, 3, &[&pack]).expect("建局");
        e.debug_clear_hand(0);
        e.state_core_mut().players[0].wunature = Some(4); // 土
        let weapon = e.debug_place(0, "zhuge");
        e.state_core_mut().card_natures.insert(weapon, 3); // 火生土
        let mut core = e.save().core;
        let mut res = engine_native::resolution::Resolution::new(20000);
        let mut logs = Vec::new();
        let _ = engine_native::resolution::push_frame(
            &mut res,
            engine_native::EventData::Equipped {
                player: 0,
                card: weapon,
                replaced: None,
            },
        );
        loop {
            match engine_native::resolution::run(&mut core, &mut res, e.catalog(), &mut logs, Some(1)) {
                engine_native::resolution::StepOutcome::Done => break,
                engine_native::resolution::StepOutcome::Yielded => continue,
                other => panic!("unexpected outcome: {other:?}"),
            }
        }
        assert!(core.player(0).hand.is_empty(), "非 PlayCard 的直接 Equipped 不得触发 _shengke 摸牌");
        assert!(core.player(0).equips.contains(&weapon));
    }

    // 4. 扩展无懈包裹锦囊（wudian wangmeizhike）与 _shengke 交互：先开无懈窗，无懈抵消跳过 _shengke，放行仅触发 1 次（不得二次触发）
    {
        let wudian_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../packages/extensions/wudian-upstream-noname/assets/cards.json");
        let wudian_json = std::fs::read_to_string(wudian_path).expect("读取 wudian cards.json");
        let wudian_pack = content_native_loader::bridge_pack(&wudian_json)
            .expect("桥接 wudian-upstream-noname")
            .pack;
        let packs: Vec<&ExtensionPack> = vec![&pack, &wudian_pack];

        for use_wuxie in [true, false] {
            let mut e = Engine::new_with_extensions(6435 + u64::from(use_wuxie), 3, &packs).expect("建局");
            advance_to_play(&mut e, 0);
            for p in 0..3 {
                e.debug_clear_hand(p);
            }
            e.debug_set_hp(1, 1);
            e.state_core_mut().players[1].wunature = Some(4); // 土
            let wm_name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.kind == engine_native::CardKind::WangmeiZhike)
                .expect("WangmeiZhike card")
                .name;
            let wm = e.debug_place(0, wm_name);
            e.state_core_mut().card_natures.insert(wm, 3); // 火生土
            let wuxie = e.debug_place(1, "nullify");

            let out = submit(&mut e, 0, GameAction::PlayCard { card: wm, targets: vec![1] });
            assert!(matches!(out, CommandOutcome::Pending { .. }));
            let nw = e.open_choice().expect("nullify_window 必须先于扩展锦囊 _shengke").clone();
            assert_eq!(nw.spec.prompt, "nullify_window");
            assert_eq!(nw.observer, 1);
            assert_eq!(e.debug_hand(1), vec![wuxie], "无懈未响应前不得先触发 _shengke 摸牌");

            if use_wuxie {
                submit(&mut e, 1, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Card(wuxie)] });
                assert_eq!(e.debug_hand(1).len(), 0, "扩展锦囊被无懈抵消不得触发 _shengke");
                assert_eq!(e.debug_hp(1), 1);
            } else {
                submit(&mut e, 1, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
                assert_eq!(e.debug_hand(1).len(), 2, "放行无懈后 1 号含原无懈手牌+仅 1 张火生土摸牌（不得双触）");
                assert_eq!(e.debug_hp(1), 2);
            }
        }
    }
}

/// `_shengke`（`wuxing/extension.js:62-172`）补闭环：
/// 1. `CardKind::Tianfuzhen` / `CardKind::Niaoxiangzhen`（均由 `wrap_nullify_window` 包裹，且带显式 `u.targets`）
///    必须先问 `_wuxie` 再按未抵消结果仅触发 1 次 `_shengke`；
/// 2. `use_as = Some(CardKind::Wuzhong)` 从非无中物理牌转化使用时，仍走无懈窗口并遵循先无懈后生克；
/// 3. 不包无懈的自定义 `Wuzhong`/`Jiguan`（如 `txhj_zhijizhibi`、`lingjiandai`）空 `targets` 时对使用者触发 1 次 `_shengke`。
#[test]
fn l06_shengke_zhenfa_and_view_as_wuzhong_nullify_order() {
    let pack = wuxing_pack();
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let zf_json = std::fs::read_to_string(root.join("shahai-zhenfa-upstream-noname/assets/cards.json"))
        .expect("read zhenfa cards.json");
    let zf_pack = content_native_loader::bridge_pack(&zf_json).expect("bridge zhenfa").pack;
    let swd_json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("read swd cards.json");
    let swd_pack = content_native_loader::bridge_pack(&swd_json).expect("bridge swd").pack;

    let wei = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .find(|g| g.faction == engine_native::catalog::Faction::Wei)
        .expect("wei")
        .name
        .to_string();
    let shu = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .find(|g| g.faction == engine_native::catalog::Faction::Shu)
        .expect("shu")
        .name
        .to_string();

    // 1a. 天覆阵：0/1 魏、2/3 蜀，1 号土属性 + 持无懈，0 号出火属性天覆阵（目标 [0, 1]）
    {
        let packs: Vec<&ExtensionPack> = vec![&pack, &zf_pack];
        for use_wuxie in [true, false] {
            let mut e = Engine::new_with_extensions(6450 + u64::from(use_wuxie), 4, &packs).expect("建局");
            advance_to_play(&mut e, 0);
            e.debug_assemble_general(0, &wei);
            e.debug_assemble_general(1, &wei);
            e.debug_assemble_general(2, &shu);
            e.debug_assemble_general(3, &shu);
            for p in 0..4 {
                e.debug_clear_hand(p);
                e.state_core_mut().players[usize::from(p)].wunature = None;
            }
            e.state_core_mut().card_natures.clear();
            e.state_core_mut().players[1].wunature = Some(4); // 土
            let tf_name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.kind == engine_native::CardKind::Tianfuzhen)
                .expect("Tianfuzhen")
                .name;
            let tf = e.debug_place(0, tf_name);
            e.state_core_mut().card_natures.insert(tf, 3); // 火生土
            let wuxie = e.debug_place(1, "nullify");

            let out = submit(&mut e, 0, GameAction::PlayCard { card: tf, targets: vec![0, 1] });
            assert!(matches!(out, CommandOutcome::Pending { .. }), "tf out={out:?}");
            let nw = e.open_choice().expect("tianfuzhen nullify_window must precede _shengke").clone();
            assert_eq!(nw.spec.prompt, "nullify_window");
            assert_eq!(e.debug_hand(1), vec![wuxie], "无懈未响应前天覆阵不得先触发 _shengke");

            if use_wuxie {
                submit(&mut e, 1, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Card(wuxie)] });
                assert!(e.debug_hand(1).is_empty(), "天覆阵被无懈后不得触发 _shengke");
            } else {
                submit(&mut e, 1, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
                let disc = e.open_choice().expect("swd_tianfuzhen_discard").clone();
                assert_eq!(disc.spec.prompt, "swd_tianfuzhen_discard");
                assert_eq!(disc.observer, 1);
                assert_eq!(e.debug_hand(1).len(), 2, "放行天覆阵后 1 号仅多摸 1 张（原无懈+生克1，不得双触）");
                drain_choices(&mut e);
            }
        }
    }

    // 1b. 鸟翔阵：0/1 魏、2/3 蜀，2 号土属性 + 持无懈，0 号出火属性鸟翔阵（目标 [2, 3]）
    {
        let packs: Vec<&ExtensionPack> = vec![&pack, &zf_pack];
        for use_wuxie in [true, false] {
            let mut e = Engine::new_with_extensions(6454 + u64::from(use_wuxie), 4, &packs).expect("建局");
            advance_to_play(&mut e, 0);
            e.debug_assemble_general(0, &wei);
            e.debug_assemble_general(1, &wei);
            e.debug_assemble_general(2, &shu);
            e.debug_assemble_general(3, &shu);
            for p in 0..4 {
                e.debug_clear_hand(p);
                e.state_core_mut().players[usize::from(p)].wunature = None;
            }
            e.state_core_mut().card_natures.clear();
            e.state_core_mut().players[2].wunature = Some(4); // 土
            let nx_name = e
                .catalog()
                .cards
                .iter()
                .find(|c| c.kind == engine_native::CardKind::Niaoxiangzhen)
                .expect("Niaoxiangzhen")
                .name;
            let nx = e.debug_place(0, nx_name);
            e.state_core_mut().card_natures.insert(nx, 3); // 火生土
            let wuxie = e.debug_place(2, "nullify");
            let _shan3 = e.debug_place(3, "shan");

            let out = submit(&mut e, 0, GameAction::PlayCard { card: nx, targets: vec![2, 3] });
            assert!(matches!(out, CommandOutcome::Pending { .. }), "nx out={out:?}");
            let nw = e.open_choice().expect("niaoxiangzhen nullify_window must precede _shengke").clone();
            assert_eq!(nw.spec.prompt, "nullify_window");
            assert_eq!(e.debug_hand(2), vec![wuxie], "无懈未响应前鸟翔阵不得先触发 _shengke");

            if use_wuxie {
                submit(&mut e, 2, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Card(wuxie)] });
                assert!(e.debug_hand(2).is_empty(), "鸟翔阵被无懈后不得触发 _shengke");
            } else {
                submit(&mut e, 2, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
                let shan_ask = e.open_choice().expect("swd_niaoxiangzhen_shan").clone();
                assert_eq!(shan_ask.spec.prompt, "swd_niaoxiangzhen_shan");
                assert_eq!(e.debug_hand(2).len(), 2, "放行鸟翔阵后 2 号仅多摸 1 张（原无懈+生克1，不得双触）");
                drain_choices(&mut e);
            }
        }
    }

    // 2. 非无中实体牌经 view_as 转无中（use_as = Some(Wuzhong)）：先无懈后生克、仅 1 次
    {
        let vm_src = r#"
skill test_va_wuzhong(99060099) {
    view_as wuzhong play_phase any from hand;
}
"#;
        let vm_pkg = content_compiler::compile_source_named("l06_va_wuzhong", vm_src).expect("compile vm");
        let vm_out = content_native_loader::bridge_vm_skill_decls(&vm_pkg).expect("bridge vm");
        let custom_pack = ExtensionPack {
            def: engine_native::content::extension::ExtensionDef {
                name: "l06-wuzhong-jiguan-probe",
                version: 1,
                description: "view_as wuzhong and unwrapped custom wuzhong probe",
            },
            generals: Vec::new(),
            cards: vec![
                engine_native::catalog::CardDef::basic("txhj_zhijizhibi", engine_native::CardKind::Wuzhong),
                engine_native::catalog::CardDef::basic("lingjiandai", engine_native::CardKind::Jiguan),
            ],
            host_skills: Vec::new(),
            vm_skills: vm_out.decls,
            init_hook: engine_native::content::extension::InitHook::None,
            enabled: true,
        };
        let packs: Vec<&ExtensionPack> = vec![&pack, &custom_pack, &swd_pack];
        for use_wuxie in [true, false] {
            let mut e = Engine::new_with_extensions(6458 + u64::from(use_wuxie), 3, &packs).expect("建局");
            for p in 0..3 {
                e.debug_remove_general(p);
            }
            advance_to_play(&mut e, 0);
            for p in 0..3 {
                e.debug_clear_hand(p);
                e.state_core_mut().players[usize::from(p)].wunature = None;
            }
            e.state_core_mut().card_natures.clear();
            e.state_core_mut().players[0].wunature = Some(4); // 土
            e.debug_grant(0, "test_va_wuzhong");
            let sha = e.debug_place(0, "sha");
            e.state_core_mut().card_natures.insert(sha, 3); // 火生土
            let wuxie = e.debug_place(1, "nullify");
            let va_act = e
                .legal_actions(0)
                .into_iter()
                .find(|a| matches!(a, GameAction::PlayCard { card, targets } if *card == sha && targets.is_empty()))
                .expect("view_as wuzhong action");
            let out = submit(&mut e, 0, va_act);
            assert!(matches!(out, CommandOutcome::Pending { .. }), "va wuzhong out={out:?}");
            let nw = e.open_choice().expect("view_as wuzhong nullify_window").clone();
            assert_eq!(nw.spec.prompt, "nullify_window");
            assert!(e.debug_hand(0).is_empty(), "无懈未响应前转化无中不得先触发 _shengke");

            if use_wuxie {
                submit(&mut e, 1, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Card(wuxie)] });
                assert!(e.debug_hand(0).is_empty(), "转化无中被无懈后不得触发 _shengke");
            } else {
                submit(&mut e, 1, GameAction::Answer { choice_id: nw.id, selection: vec![ChoiceOption::Pass] });
                assert_eq!(e.debug_hand(0).len(), 3, "放行转化无中应摸 2+生克 1（不得双触）");
            }
        }

        // 3. 不包无懈的自定义 Wuzhong (txhj_zhijizhibi) / Jiguan (lingjiandai) 触发 1 次 _shengke
        {
            let mut e = Engine::new_with_extensions(6460, 3, &packs).expect("建局");
            for p in 0..3 {
                e.debug_remove_general(p);
            }
            advance_to_play(&mut e, 0);
            for p in 0..3 {
                e.debug_clear_hand(p);
                e.state_core_mut().players[usize::from(p)].wunature = None;
            }
            e.state_core_mut().card_natures.clear();
            e.state_core_mut().players[0].wunature = Some(4); // 土
            let zz = e.debug_move_named_card_to_hand(0, "txhj_zhijizhibi");
            e.state_core_mut().card_natures.insert(zz, 3); // 火生土
            submit(&mut e, 0, GameAction::PlayCard { card: zz, targets: vec![] });
            drain_choices(&mut e);
            assert_eq!(e.debug_hand(0).len(), 2, "不包无懈自定义 Wuzhong 牌效摸 1 + _shengke 摸 1");

            e.debug_clear_hand(0);
            let lj = e.debug_move_named_card_to_hand(0, "lingjiandai");
            e.state_core_mut().card_natures.insert(lj, 3); // 火生土
            submit(&mut e, 0, GameAction::PlayCard { card: lj, targets: vec![] });
            drain_choices(&mut e);
            assert_eq!(e.debug_hand(0).len(), 2, "不包无懈自定义 Jiguan 牌效摸/得 1 + _shengke 摸 1");
        }
    }
}
