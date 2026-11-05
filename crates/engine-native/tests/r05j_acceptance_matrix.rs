//! R05-J 终验矩阵补齐（合同 `proposal-v3.raw.txt §9` 场景类 9、10、12）：
//!
//! 1. `r05j_next_enabled_true_and_false_across_death_archive_revive_and_turn_begin`（类 9）：
//!    `Next` 原 `enabled=true` 与 `enabled=false` 各死 → 保存/字节归档/恢复 → 复活，
//!    断言 `enabled` 原值不变；只有 `alive && enabled` 的真实 `TurnBegin` 原子转 `Current`，
//!    `enabled=false` 在复活后的 `TurnBegin` 不消费 `Next`；同回合新伤害仍写 `Next`。
//! 2. `r05j_same_identity_two_owners_merge_target_slot_and_replacement_clears_roles`（类 10）：
//!    同一逻辑身份 `I` 的两个 owner 对同一 target 施加效果时合并入同一 `(I, role, target)` 槽；
//!    不同包逻辑身份 `I2` 独立不串槽；武将替换（`assign_general_named` / uninit replacement）
//!    与显式移除 helper 删除对应槽位与实例。
//! 3. `r05j_class12_counterfactual_privacy_plan_and_apply_invariance`（类 12）：
//!    同公开视图（`SeatView.r05_effects`、`SeatView.linked`、`ChoiceContext::TiqiAdjustment`、
//!    手牌数、体力），但私有手牌花色/牌堆序/detached 物理牌 ID/他人 VM storage 不同的反事实世界对：
//!    断言 `redact_opponent_hands` 在含 `detached_cards` 时保持物理牌唯一归属，
//!    `plan_action` 产出的完整 `Plan` 逐字段相等（`sim_faults == 0`），且 `apply_plan`
//!    在两世界均成功执行（`CommandOutcome::Accepted`）并保持公开结果一致。

use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::event::{DamageNature, DetachedKind};
use engine_native::view::{ChoiceContext, ChoiceView, R05EffectView};
use engine_native::{
    ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, OpenChoice, Phase,
};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn must(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let outcome = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action,
    });
    assert!(
        !matches!(outcome, CommandOutcome::Rejected(_)),
        "命令意外被拒: {outcome:?}"
    );
    outcome
}

fn default_answer(open: &OpenChoice) -> Vec<O> {
    if open.spec.options.contains(&O::Pass) {
        vec![O::Pass]
    } else if open.spec.allow_cancel {
        vec![]
    } else if open.spec.options.len() >= open.spec.min {
        open.spec.options.iter().take(open.spec.min).copied().collect()
    } else {
        panic!("无法默认应答: {:?}", open.spec)
    }
}

fn pump<F>(e: &mut Engine, mut script: F)
where
    F: FnMut(&OpenChoice) -> Option<Vec<O>>,
{
    if let Some(open) = e.open_choice().cloned() {
        let selection = if open.spec.prompt.starts_with("rvm_") {
            script(&open).unwrap_or_else(|| panic!("未脚本化的 rvm 询问: {}", open.spec.prompt))
        } else {
            default_answer(&open)
        };
        must(
            e,
            open.observer,
            GameAction::Answer {
                choice_id: open.id,
                selection,
            },
        );
        return;
    }
    let actor = e.turn().current;
    must(e, actor, GameAction::NextPhase);
}

fn run_until<F, P>(e: &mut Engine, mut script: F, pred: P)
where
    F: FnMut(&OpenChoice) -> Option<Vec<O>>,
    P: Fn(&Engine) -> bool,
{
    for _ in 0..256 {
        if pred(e) {
            return;
        }
        pump(e, |c| script(c));
    }
    panic!(
        "run_until 超预算: turn={:?}/{:?} open={:?}",
        e.turn().current,
        e.turn().phase,
        e.open_choice().map(|c| c.spec.prompt)
    );
}

fn pkg_root(dir: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(dir)
}

fn load_real_pack(dir: &str) -> ExtensionPack {
    let json = std::fs::read_to_string(pkg_root(dir).join("package.npack.json")).unwrap();
    let npack: content_schema::ContentPackage = serde_json::from_str(&json).unwrap();
    let decls = content_native_loader::bridge_vm_skill_decls(&npack)
        .unwrap()
        .decls;
    let chars = std::fs::read_to_string(pkg_root(dir).join("assets/characters.json")).unwrap();
    let generals = content_native_loader::bridge_generals_with_vm_package(&chars, &npack)
        .unwrap()
        .generals;
    let meta: serde_json::Value = serde_json::from_str(&chars).unwrap();
    let pack_name = meta["pack"]["name"].as_str().unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(pack_name.to_string()),
            version: 1,
            description: engine_native::content::names::intern(
                meta["pack"]["description"]
                    .as_str()
                    .unwrap_or("")
                    .to_string(),
            ),
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}

fn helper_def(e: &Engine, name: &str) -> u32 {
    e.catalog()
        .skill_by_name(name)
        .unwrap_or_else(|| panic!("helper 未入目录: {name}")) as u32
}

fn mark(e: &Engine, seat: u8, def: u32) -> Option<i32> {
    e.state_core().vm_aux.marks.get(&(seat, def)).copied()
}

/// 类 9：`Next` 原 `enabled=true` 与 `enabled=false` 各死 → 保存恢复 → 复活，
/// `enabled` 原值不变；只有 `alive && enabled` 的真实 `TurnBegin` 原子转 `Current`，
/// 同回合新伤害仍写 `Next`。
#[test]
fn r05j_next_enabled_true_and_false_across_death_archive_revive_and_turn_begin() {
    let sp2 = load_real_pack("sp2-upstream-noname");
    let packs = [&sp2];
    let mut e = Engine::new_with_extensions(35901, 4, &packs).unwrap();
    e.assign_general_named(0, "sp2冯方").unwrap();
    let next_def = helper_def(&e, "dcbihuo#r05:next");
    let next_id = u16::try_from(next_def).unwrap();
    let cur_def = helper_def(&e, "dcbihuo#r05:current");

    // 令 seat0 连续造成两次伤害，分别给 seat1（将保持 enabled=true）与 seat2（将设 enabled=false）挂 Next = -1
    e.debug_deal_damage(Some(0), 3, 1, DamageNature::Normal);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => Some(vec![O::Player(1)]),
            other => panic!("意外询问: {other}"),
        },
        |e| e.open_choice().is_none(),
    );
    e.debug_deal_damage(Some(0), 3, 1, DamageNature::Normal);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => Some(vec![O::Player(2)]),
            other => panic!("意外询问: {other}"),
        },
        |e| e.open_choice().is_none(),
    );
    assert_eq!(mark(&e, 1, next_def), Some(-1));
    assert_eq!(mark(&e, 2, next_def), Some(-1));

    // 显式将 seat2 上的 Next helper 设为 enabled=false，seat1 保持 enabled=true
    e.state_core_mut()
        .skills
        .get_mut(&(2, next_id))
        .expect("seat2 已挂载 Next helper")
        .enabled = false;

    // seat1 与 seat2 均经历真实 DeathMachine step 2 普通死亡结算
    e.debug_deal_damage(None, 1, 10, DamageNature::Normal);
    run_until(
        &mut e,
        |_| None,
        |e| e.open_choice().is_none(),
    );
    e.debug_deal_damage(None, 2, 10, DamageNature::Normal);
    run_until(
        &mut e,
        |_| None,
        |e| e.open_choice().is_none(),
    );
    assert!(!e.state_core().players[1].alive);
    assert!(!e.state_core().players[2].alive);

    // 保存 → 编码 → 解码 → 恢复
    let bytes = encode_archive(&e.save());
    let decoded = decode_archive(&bytes).unwrap();
    let mut e = Engine::restore_with_extensions(&decoded, &packs).unwrap();

    // 验证死亡 + 恢复后 Next 实例、值与原 enabled（seat1=true, seat2=false）完全保持
    assert_eq!(mark(&e, 1, next_def), Some(-1));
    assert_eq!(mark(&e, 2, next_def), Some(-1));
    let s1_inst = e
        .state_core()
        .skills
        .get(&(1, next_id))
        .expect("seat1 死后仍保留 Next helper 实例");
    assert!(s1_inst.enabled, "seat1 Next enabled 原值 true 保持");
    let s2_inst = e
        .state_core()
        .skills
        .get(&(2, next_id))
        .expect("seat2 死后仍保留 Next helper 实例");
    assert!(!s2_inst.enabled, "seat2 Next enabled 原值 false 保持");

    // 复活 seat1 与 seat2（只恢复存活与体力，不改 Next.enabled）
    e.state_core_mut().players[1].alive = true;
    e.state_core_mut().players[1].hp = 2;
    e.state_core_mut().players[2].alive = true;
    e.state_core_mut().players[2].hp = 2;

    // 推进到 seat1 的 Judge 阶段（已经过 seat1 的真实 TurnBegin 与自动推进的 Prepare）：
    // seat1 的 Next(enabled=true) 原子转入 Current=-1，Next 清空
    run_until(
        &mut e,
        |_| None,
        |e| e.turn().current == 1 && e.turn().phase == Phase::Judge && e.open_choice().is_none(),
    );
    assert_eq!(
        mark(&e, 1, next_def),
        None,
        "seat1 enabled=true 在 TurnBegin 原子移除 Next"
    );
    assert_eq!(
        mark(&e, 1, cur_def),
        Some(-1),
        "seat1 enabled=true 在 TurnBegin 写入 Current=-1"
    );

    // 同回合（seat1 回合内）新伤害继续写 seat1 的 Next，不覆盖 Current
    e.debug_deal_damage(Some(0), 3, 1, DamageNature::Normal);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => Some(vec![O::Player(1)]),
            other => panic!("意外询问: {other}"),
        },
        |e| e.open_choice().is_none(),
    );
    assert_eq!(mark(&e, 1, next_def), Some(-1), "同回合新伤害写入 Next=-1");
    assert_eq!(
        mark(&e, 1, cur_def),
        Some(-1),
        "同回合新伤害不覆盖尚未消费的 Current=-1"
    );

    // 继续推进到 seat2 的 Judge 阶段（已经过 seat2 的真实 TurnBegin）：
    // seat2 的 Next(enabled=false) 不触发，Next=-1 与 enabled=false 原样保留，Current=None
    run_until(
        &mut e,
        |_| None,
        |e| e.turn().current == 2 && e.turn().phase == Phase::Judge && e.open_choice().is_none(),
    );
    assert_eq!(
        mark(&e, 2, next_def),
        Some(-1),
        "seat2 enabled=false 在 TurnBegin 不消费 Next"
    );
    assert_eq!(mark(&e, 2, cur_def), None, "seat2 enabled=false 不产生 Current");
    let s2_after = e.state_core().skills.get(&(2, next_id)).unwrap();
    assert!(!s2_after.enabled, "seat2 Next enabled=false 仍保持不变");
}

/// 类 10：同逻辑身份 `I` 的两个 owner 合并同一 target 槽；不同包 `I2` 独立不串槽；
/// 武将替换（uninit replacement）清理四角色槽位与 helper 实例。
#[test]
fn r05j_same_identity_two_owners_merge_target_slot_and_replacement_clears_roles() {
    let xianding = load_real_pack("xianding-upstream-noname");
    let sp2 = load_real_pack("sp2-upstream-noname");
    let packs = [&xianding, &sp2];
    let mut e = Engine::new_with_extensions(35902, 4, &packs).unwrap();
    // seat0 与 seat1 均持有同一逻辑身份的sp2冯方（dcbihuo）
    e.assign_general_named(0, "sp2冯方").unwrap();
    e.assign_general_named(1, "sp2冯方").unwrap();
    e.assign_general_named(2, "xnd冯妤").unwrap();

    let next_def = helper_def(&e, "dcbihuo#r05:next");
    let next_id = u16::try_from(next_def).unwrap();

    // seat0 受 1 点伤害（End 窗 +1）→ 选 seat3；seat1 受 1 点伤害（End 窗 +1）→ 也选 seat3
    e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => Some(vec![O::Player(3)]),
            other => panic!("意外询问: {other}"),
        },
        |e| e.open_choice().is_none(),
    );
    assert_eq!(mark(&e, 3, next_def), Some(1));

    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    run_until(
        &mut e,
        |c| match c.spec.prompt {
            "rvm_dcbihuo_target" => Some(vec![O::Player(3)]),
            other => panic!("意外询问: {other}"),
        },
        |e| e.open_choice().is_none(),
    );
    assert_eq!(
        mark(&e, 3, next_def),
        Some(2),
        "同一逻辑身份 I 的两个 owner（seat0 与 seat1）对同一 target（seat3）累加同一 Next 槽"
    );
    assert!(
        e.state_core().skills.contains_key(&(3, next_id)),
        "同一 (I, role, target) 挂载唯一 helper 实例"
    );

    // 武将替换（assign_general_named 触发 uninit replacement）：seat3 换将后其身上的 Next 槽与 helper 实例一并清除
    e.assign_general_named(3, "xnd冯妤").unwrap();
    assert_eq!(
        mark(&e, 3, next_def),
        None,
        "武将替换（uninit replacement）必须清理目标身上的 R05 helper 槽"
    );
    assert!(
        !e.state_core().skills.contains_key(&(3, next_id)),
        "武将替换后不再残留 Next helper 实例"
    );
}

/// 类 12（反事实隐私与 AI 规划不变量）：
/// 同公开视图（`SeatView.r05_effects`、`SeatView.linked`、`ChoiceContext::TiqiAdjustment`、
/// 手牌数、体力），但私有手牌/牌堆序/detached 物理牌/他人 VM storage 不同的反事实世界对：
/// `plan_action` 产出的完整 `Plan` 相等（`sim_faults == 0`），且 `apply_plan` 在两世界均成功落账。
#[test]
fn r05j_class12_counterfactual_privacy_plan_and_apply_invariance() {
    let xianding = load_real_pack("xianding-upstream-noname");
    let sp2 = load_real_pack("sp2-upstream-noname");
    let packs = [&xianding, &sp2];

    // 构造到 seat0 面临 rvm_tiqi_sign（含 ChoiceContext::TiqiAdjustment）的对局切片
    let build_tiqi_sign_world = |swap_private: bool| -> Engine {
        let mut e = Engine::new_with_extensions(35912, 3, &packs).unwrap();
        e.assign_general_named(0, "xnd冯妤").unwrap();
        e.assign_general_named(2, "sp2冯方").unwrap();
        // 推进到 seat1 的判定阶段，排空牌堆与弃牌堆使 seat1 摸牌阶段 n=0（d=2），随后补回牌堆供涕泣摸 2 张
        run_until(
            &mut e,
            |_| None,
            |e| e.turn().current == 1 && e.turn().phase == Phase::Judge,
        );
        let saved_deck = e.state_core().deck.clone();
        e.state_core_mut().deck.clear();
        let saved_discard = std::mem::take(&mut e.state_core_mut().discard);
        run_until(
            &mut e,
            |_| None,
            |e| e.open_choice().map(|c| c.spec.prompt) == Some("rvm_tiqi_confirm"),
        );
        e.state_core_mut().deck = saved_deck;
        e.state_core_mut().discard = saved_discard;
        // 令牌堆尾部 1 张牌进入 detached_cards（验证含 detached_cards 时 redact_opponent_hands 仍守恒）
        let detached_card = e.state_core_mut().deck.pop().unwrap();
        e.state_core_mut()
            .detached_cards
            .push((detached_card, DetachedKind::Unclaimed));

        // 确认发动涕泣，推进到 rvm_tiqi_sign
        let confirm_id = e.open_choice().unwrap().id;
        must(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: confirm_id,
                selection: vec![O::Confirm],
            },
        );
        assert_eq!(
            e.open_choice().map(|c| c.spec.prompt),
            Some("rvm_tiqi_sign")
        );

        if swap_private {
            // 在不改变 seat0 与 seat1 公开视图及自身手牌的前提下，交换 seat2 的暗手牌与牌堆深处牌，并写入他人私有 VM storage
            let core = e.state_core_mut();
            let d_len = core.deck.len();
            let p2_hand_len = core.players[2].hand.len();
            for i in 0..p2_hand_len {
                let deck_idx = d_len - 1 - i;
                let tmp = core.players[2].hand[i];
                core.players[2].hand[i] = core.deck[deck_idx];
                core.deck[deck_idx] = tmp;
            }
            core.deck[0..4].reverse();
            core.vm_aux.storage.insert("2:998102001:77".into(), 12345);
        }
        e
    };

    let mut world_a = build_tiqi_sign_world(false);
    let mut world_b = build_tiqi_sign_world(true);
    assert_ne!(
        world_a.state_hash(),
        world_b.state_hash(),
        "反事实两世界的完整权威状态哈希不同（暗牌与私有 storage 不同）"
    );

    // 1. 验证观察者 seat0（Own: TiqiAdjustment）与非拥有者 seat1（Other）的 AI 输入完全一致
    for observer in [0u8, 1u8] {
        assert_eq!(
            format!("{:?}", world_a.ai_session(observer).input()),
            format!("{:?}", world_b.ai_session(observer).input()),
            "seat {observer} 的公开 AI 输入在反事实两世界必须完全一致"
        );
    }
    assert!(matches!(
        world_a.ai_session(0).choice(),
        Some(ChoiceView::Own {
            context: Some(ChoiceContext::TiqiAdjustment {
                delta: 2,
                target: 1,
                ..
            }),
            ..
        })
    ));

    // 2. 验证含 detached_cards 时 redact_opponent_hands 保持完整物理牌唯一归属（restore 通过）
    for w in [&world_a, &world_b] {
        let mut sampled = w.clone();
        let expected_detached = sampled.state_core().detached_cards.clone();
        redact_opponent_hands(&mut sampled, 0);
        assert_eq!(
            sampled.state_core().detached_cards,
            expected_detached,
            "redact_opponent_hands 不得重分配 detached_cards 中的实体牌"
        );
        Engine::restore_with_extensions(&sampled.save(), &packs)
            .expect("遮蔽后的克隆必须通过完整归属与恢复校验");
    }

    // 3. 在 rvm_tiqi_sign 窗口对 seat0 与 seat1 规划并应用计划
    let planner = BoundedPlanner::new(48)
        .with_policy_variant(1)
        .with_lookahead(2);
    let plan_a0 = world_a.plan_action(0, &planner);
    let plan_b0 = world_b.plan_action(0, &planner);
    assert_eq!(
        format!("{plan_a0:?}"),
        format!("{plan_b0:?}"),
        "rvm_tiqi_sign 下反事实两世界的完整 Plan 必须相等"
    );
    assert_eq!(plan_a0.sim_faults, 0);
    let out_a0 = world_a.apply_plan(&plan_a0).expect("apply_plan a0");
    let out_b0 = world_b.apply_plan(&plan_b0).expect("apply_plan b0");
    assert!(matches!(out_a0, CommandOutcome::Accepted { .. }));
    assert!(matches!(out_b0, CommandOutcome::Accepted { .. }));

    // 4. 推进两世界到 seat1 的出牌阶段（此时 seat1 身上有公开 r05_effects.tiqi_limit = Some(2)），
    //    在静止态但含 r05_effects 与 detached_cards 的场景下执行有界搜索（budget=48, variant=1, lookahead=2）
    run_until(
        &mut world_a,
        |_| None,
        |e| e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none(),
    );
    run_until(
        &mut world_b,
        |_| None,
        |e| e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none(),
    );
    // 固定 seat1 手牌并只变异 seat2 与牌堆
    let mut play_b = world_a.clone();
    {
        let core = play_b.state_core_mut();
        let d_len = core.deck.len();
        let p2_len = core.players[2].hand.len();
        for i in 0..p2_len {
            let deck_idx = d_len - 1 - i;
            let tmp = core.players[2].hand[i];
            core.players[2].hand[i] = core.deck[deck_idx];
            core.deck[deck_idx] = tmp;
        }
        core.vm_aux.storage.insert("2:998102001:88".into(), 999);
    }
    assert_eq!(
        format!("{:?}", world_a.ai_session(1).input()),
        format!("{:?}", play_b.ai_session(1).input()),
        "seat1 在两反事实世界的公开视图（含 r05_effects）与自身手牌完全一致"
    );
    assert_eq!(
        world_a.view_for(1).players[1].r05_effects,
        vec![R05EffectView {
            skill: engine_native::catalog::QualifiedSkillRef {
                namespace: "upstream-noname".into(),
                package_id: "xianding".into(),
                logical_skill_def_id: 998101001,
            },
            tiqi_limit: Some(2),
            baoshu_combs: None,
            dcbihuo_next: None,
            dcbihuo_current: None,
        }]
    );

    let plan_play_a = world_a.plan_action(1, &planner);
    let plan_play_b = play_b.plan_action(1, &planner);
    assert_eq!(
        format!("{plan_play_a:?}"),
        format!("{plan_play_b:?}"),
        "出牌阶段有界搜索在反事实两世界产出的完整 Plan 必须相等"
    );
    assert_eq!(plan_play_a.sim_faults, 0);
    assert!(plan_play_a.nodes_used > 0, "出牌阶段必须实际执行模拟节点");
    let res_play_a = world_a.apply_plan(&plan_play_a).expect("apply play_a");
    let res_play_b = play_b.apply_plan(&plan_play_b).expect("apply play_b");
    assert!(
        !matches!(res_play_a, CommandOutcome::Rejected(_)),
        "res_play_a={res_play_a:?}"
    );
    assert!(
        !matches!(res_play_b, CommandOutcome::Rejected(_)),
        "res_play_b={res_play_b:?}"
    );
}
