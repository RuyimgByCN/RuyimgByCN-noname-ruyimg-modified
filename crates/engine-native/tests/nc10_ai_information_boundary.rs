//! NC10 AI 合法信息边界：规划决策**不得依赖隐藏信息**。
//!
//! 判据（可复跑、与实现无关）：把对手手牌与牌库内容做确定性置换（隐藏信息），
//! 保持座位视图（自己的手牌/装备/判定、公开计数、阶段、轮次）不变，则
//! `plan_action(seat)` 的结果必须**逐字段相同**。
//!
//! 反向对照（证明探针能发现差异）：置换**自己的手牌**后，至少存在一个采样状态
//! 使规划结果不同——否则说明探针无分辨力，本用例不成立。
//!
//! 背景：`ai.rs::redact_opponent_hands` 原先在任意状态重分配对手手牌（会破坏挂起
//! 机器引用 → NC12 §5.2 panic），修复后只在静止态重分配；非静止态的隐藏信息边界
//! 由本用例验收。

use engine_native::ai::BoundedPlanner;
use engine_native::{Engine, GameAction, GameCommand};

fn swd_pack() -> engine_native::content::extension::ExtensionPack {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let json = std::fs::read_to_string(root.join("shahai-swd-upstream-noname/assets/cards.json"))
        .expect("读 swd 包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

fn planner() -> BoundedPlanner {
    BoundedPlanner::new(64)
        .with_deadline_ms(50)
        .with_policy_variant(1)
}

/// 确定性置换：把对手手牌 + 牌库整体旋转 k 张（隐藏信息变化，公开视图不变）。
fn permute_hidden(e: &mut Engine, seat: u8, k: usize) {
    let n = e.state_core().players.len();
    let others: Vec<usize> = (0..n).filter(|i| *i as u8 != seat).collect();
    // 先取原手牌数（公开信息），再搬移隐藏牌
    let lens: Vec<usize> = others
        .iter()
        .map(|i| e.state_core().players[*i].hand.len())
        .collect();
    let mut pool: Vec<u32> = Vec::new();
    {
        let core = e.state_core_mut();
        for i in &others {
            pool.append(&mut core.players[*i].hand);
        }
        pool.append(&mut core.deck);
    }
    if pool.len() < 2 {
        return;
    }
    let shift = k % pool.len();
    pool.rotate_left(shift);
    // 按原手牌数放回（只改变「谁拿哪张」，不改变任何公开计数）
    let core = e.state_core_mut();
    let mut cursor = 0usize;
    for (i, len) in others.iter().zip(lens) {
        core.players[*i].hand = pool[cursor..cursor + len].to_vec();
        cursor += len;
    }
    core.deck = pool[cursor..].to_vec();
}

/// 替换自己的一张手牌（反向对照用）：把首张换成牌库首张（内容变化，非顺序变化）。
fn permute_own(e: &mut Engine, seat: u8) {
    let mut hand = e.state_core().players[seat as usize].hand.clone();
    let deck_first = e.state_core().deck.first().copied();
    if let (Some(first), Some(newc)) = (hand.first().copied(), deck_first) {
        hand[0] = newc;
        // 牌区唯一性：把被替换的牌放回牌库（替换而非复制）
        let core = e.state_core_mut();
        if let Some(pos) = core.deck.iter().position(|c| *c == newc) {
            core.deck.remove(pos);
        }
        core.deck.push(first);
        core.players[seat as usize].hand = hand;
    }
}

/// 驱动到指定步数（真实提交），返回推进步数。
fn drive(e: &mut Engine, steps: usize) {
    let p = planner();
    for step in 0..steps {
        if e.outcome().is_some() {
            break;
        }
        let seat = e.turn().current;
        let plan = e.plan_action(seat, &p);
        let _ = e.submit(GameCommand {
            actor: seat,
            expected_revision: e.revision(),
            dedup_id: step as u64 + 1,
            action: plan.action,
        });
    }
}

/// 隐藏信息置换不改变规划结果（AI 合法信息边界）。
///
/// 修复记录（2026-10-18）：旧实现把「对手手牌 ∪ 牌库」确定性洗牌后按原张数放回，
/// 保留隐藏牌**多重集** → AI 仍可读到「对手有闪」（seed 42022 warm 9 seat 1 复现）。
/// 现改为**内容无关填充**（从目录抽取互不相同的牌填满隐藏区，只保留张数）。
#[test]
fn planner_is_invariant_under_hidden_information() {
    let pack = swd_pack();
    let mut checked = 0usize;
    for seed in [42021u64, 42022, 42023] {
        for warm in [3usize, 9, 17] {
            let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
            drive(&mut base, warm);
            if base.outcome().is_some() {
                continue;
            }
            let seat = base.turn().current;
            let a = base.plan_action(seat, &planner()).action;

            let mut permuted = base.clone();
            permute_hidden(&mut permuted, seat, 7);
            let b = permuted.plan_action(seat, &planner()).action;

            // 视图等价性前提：公开面（阶段/轮次/手牌数/体力）必须一致
            assert_eq!(
                format!("{:?}", base.turn()),
                format!("{:?}", permuted.turn()),
                "置换不得改变公开视图（turn）"
            );
            for i in 0..4u8 {
                assert_eq!(
                    base.state_core().player(i).hand.len(),
                    permuted.state_core().player(i).hand.len(),
                    "置换不得改变手牌数（座位 {i}）"
                );
                assert_eq!(
                    base.debug_hp(i),
                    permuted.debug_hp(i),
                    "置换不得改变体力（座位 {i}）"
                );
            }
            let res = base.resolution();
            assert_eq!(
                format!("{a:?}"),
                format!("{b:?}"),
                "seed={seed} warm={warm} seat={seat}：规划结果依赖了隐藏信息（parks={} stack={} open_choice={}）",
                res.parks.len(),
                res.stack.len(),
                res.open_choice.is_some()
            );
            checked += 1;
        }
    }
    assert!(checked >= 6, "实际检查 {checked} 个状态（样本量足够）");
}

/// 反向对照：置换自己的手牌可改变规划结果（证明探针有分辨力）。
#[test]
fn planner_detects_own_hand_change() {
    let pack = swd_pack();
    let mut differences = 0usize;
    let mut samples = 0usize;
    for seed in [42021u64, 42022, 42023] {
        for warm in [3usize, 9, 17] {
            let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
            drive(&mut base, warm);
            if base.outcome().is_some() {
                continue;
            }
            let seat = base.turn().current;
            let a = base.plan_action(seat, &planner()).action;
            // 只在静止态改自己的手牌（非静止态挂起机器/在飞帧引用实体牌 id，
            // 改区会让它们指向别的区域——与 redact 同一安全规则）
            {
                let res = base.resolution();
                if !res.stack.is_empty() || !res.parks.is_empty() || res.open_choice.is_some() {
                    continue;
                }
            }
            let mut own = base.clone();
            permute_own(&mut own, seat);
            if own.state_core().player(seat).hand == base.state_core().player(seat).hand {
                continue;
            }
            let b = own.plan_action(seat, &planner()).action;
            samples += 1;
            if format!("{a:?}") != format!("{b:?}") {
                differences += 1;
            }
        }
    }
    assert!(samples >= 3, "实际对照样本 {samples}");
    assert!(
        differences > 0,
        "置换自己的手牌后规划结果完全不变（{samples} 个样本）——探针无分辨力，隐藏信息用例不成立"
    );
}

/// 候选集合（权威动作查询）在隐藏信息置换下不变：AI 的合法动作面只由公开信息决定。
#[test]
fn legal_action_set_is_invariant_under_hidden_information() {
    let pack = swd_pack();
    let mut checked = 0usize;
    for seed in [42024u64, 42025] {
        let mut base = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
        drive(&mut base, 11);
        if base.outcome().is_some() {
            continue;
        }
        let seat = base.turn().current;
        let a: Vec<String> = base
            .legal_actions(seat)
            .iter()
            .map(|x: &GameAction| format!("{x:?}"))
            .collect();
        let mut permuted = base.clone();
        permute_hidden(&mut permuted, seat, 5);
        let b: Vec<String> = permuted
            .legal_actions(seat)
            .iter()
            .map(|x: &GameAction| format!("{x:?}"))
            .collect();
        assert_eq!(a, b, "seed={seed}：合法动作集合依赖了隐藏信息");
        checked += 1;
    }
    assert!(checked >= 1, "实际检查 {checked} 个状态");
}

fn vm_pack(pkg: &str) -> engine_native::content::extension::ExtensionPack {
    let dir = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(pkg);
    let chars = std::fs::read_to_string(dir.join("assets/characters.json"))
        .or_else(|_| std::fs::read_to_string(dir.join("assets/character.json")))
        .expect("读 characters.json");
    let mut out = content_native_loader::bridge_pack(&chars).expect("桥接");
    if let Ok(b) = content_native_loader::bridge_generals(&chars) {
        out.pack.generals = b.generals;
    }
    let npack_raw = std::fs::read_to_string(dir.join("package.npack.json")).expect("读 package.npack.json");
    let npack: content_schema::ContentPackage = serde_json::from_str(&npack_raw).expect("解析 npack");
    out.pack.vm_skills = content_native_loader::bridge_vm_skill_decls(&npack)
        .expect("桥接 VM 技能")
        .decls;
    out.pack
}

/// NC10 20 种子 AI 质量基线门禁（Host + VM 扩展包同场）：
/// 20 局（seed 1..=20，每局上限 400 步）使用 `BoundedPlanner::new(32).with_policy_variant(1)`，
/// 断言 `sim_faults == 0`、`probe_faults == 0`、`sim_fault_log().is_empty()` 且终局率 >= 50%。
#[test]
fn nc10_ai_quality_baseline_finishes_majority_with_zero_faults() {
    let host = swd_pack();
    let vm = vm_pack("sp2-upstream-noname");
    let p = BoundedPlanner::new(32).with_policy_variant(1);
    engine_native::ai::clear_sim_fault_log();
    engine_native::engine::legal_actions::clear_probe_fault_log();
    let probe_before = engine_native::engine::legal_actions::probe_faults();
    let mut sim_faults_total = 0u64;
    let mut finished = 0usize;
    let mut dedup = 1u64;

    for seed in 1u64..=20 {
        let mut e = Engine::new_with_extensions(seed, 4, &[&host, &vm]).expect("建局");
        let _ = e.assign_general_named(0, "swd_shuwaner");
        if let Some(g) = vm.generals.first() {
            let _ = e.assign_general_named(1, &g.name);
        }
        for _step in 0..400 {
            if e.outcome().is_some() {
                break;
            }
            let actor = e
                .open_choice()
                .map(|c| c.observer)
                .unwrap_or_else(|| e.turn().current);
            let plan = e.plan_action(actor, &p);
            sim_faults_total += u64::from(plan.sim_faults);
            let outcome = e.submit(GameCommand {
                actor,
                expected_revision: e.revision(),
                dedup_id: dedup,
                action: plan.action,
            });
            dedup += 1;
            assert!(
                !matches!(outcome, engine_native::CommandOutcome::Rejected(_)),
                "seed={seed} actor={actor}: 规划产出了被拒绝动作 {outcome:?}"
            );
        }
        if e.outcome().is_some() {
            finished += 1;
        }
    }

    let probe_delta = engine_native::engine::legal_actions::probe_faults() - probe_before;
    let fault_log = engine_native::ai::sim_fault_log();
    assert_eq!(sim_faults_total, 0, "模拟故障须为 0: {fault_log:?}");
    assert_eq!(
        probe_delta,
        0,
        "探测故障须为 0: {:?}",
        engine_native::engine::legal_actions::probe_fault_log()
    );
    assert!(fault_log.is_empty(), "sim_fault_log 须为空: {fault_log:?}");
    assert!(
        finished >= 10,
        "20 局中终局数须 >= 50% (10/20)，实际 finished={finished}/20"
    );
}

#[test]
fn nc10_quiescent_storage_scrubs_opponent_entries_and_preserves_observer_counters() {
    let mut a = Engine::new(42099, 4);
    a.debug_set_turn(0, engine_native::Phase::Play, 1);
    // Observer (seat 0) has a counter = 1 in vm_aux.storage + flags.
    a.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "obs_skill#1".into()), 1);
    a.state_core_mut()
        .vm_aux
        .storage
        .insert("obs_skill#1".into(), 1);
    a.state_core_mut()
        .vm_aux
        .storage
        .insert("0:998102001:1".into(), 1);

    let mut b = a.clone();
    permute_hidden(&mut b, 0, 7);
    // Opponent (seat 1) has private storage + flags + drawn in world B only.
    b.state_core_mut()
        .vm_aux
        .flags
        .insert((1, "opp_skill#9".into()), 77);
    b.state_core_mut()
        .vm_aux
        .storage
        .insert("opp_skill#9".into(), 77);
    b.state_core_mut()
        .vm_aux
        .storage
        .insert("1:998102001:9".into(), 88);
    b.state_core_mut().vm_aux.drawn.insert(1, 3);

    engine_native::ai::redact_opponent_hands(&mut a, 0);
    engine_native::ai::redact_opponent_hands(&mut b, 0);
    assert_eq!(a.state_core().vm_aux.storage.get("obs_skill#1"), Some(&1));
    assert_eq!(a.state_core().vm_aux.storage.get("0:998102001:1"), Some(&1));
    assert!(!b.state_core().vm_aux.storage.contains_key("opp_skill#9"));
    assert!(!b.state_core().vm_aux.storage.contains_key("1:998102001:9"));
    assert_eq!(a.state_core(), b.state_core());
}

/// 多帧非静止态续体（`!quiescent`，含挂起机器栈与开放选择）的信息边界：
/// 对手手牌、牌库、对手陷阱（`sxianjing_traps`）、对手私有技能堆（`vm_aux.piles`）及
/// 对手 `vm_aux.storage` / `flags` / `drawn` 置换或注入后，遮蔽快照与规划结果完全一致。
#[test]
fn planner_is_invariant_under_multi_frame_non_quiescent_continuations() {
    for seed in [9800u64, 9801, 9802] {
        let mut a = Engine::new(seed, 4);
        a.debug_set_turn(0, engine_native::Phase::Play, 1);
        a.debug_grant(1, "刚烈");
        let sha = a.debug_move_named_card_to_hand(0, "sha");
        let _shan = a.debug_move_named_card_to_hand(1, "shan");
        // 为对手 2 号与 3 号布置私有陷阱与私有 VM 牌堆
        let trap_card = a.state_core_mut().deck.pop().unwrap();
        let pile_card = a.state_core_mut().deck.pop().unwrap();
        a.state_core_mut().players[2].sxianjing_traps.push(trap_card);
        a.state_core_mut()
            .vm_aux
            .piles
            .insert((3, "private_pile".into()), vec![pile_card]);

        // 0 号对 1 号出【杀】 → 1 号放弃出【闪】 → 压入 Damage 帧与【刚烈】OptionalGate（Sha + OptionalGate 双帧非静止态）
        let out = a.submit(GameCommand {
            actor: 0,
            expected_revision: a.revision(),
            dedup_id: 1,
            action: GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        });
        assert!(!matches!(out, engine_native::CommandOutcome::Rejected(_)));
        let dodge_c = a.open_choice().cloned().expect("应有闪响应窗口");
        let pass_out = a.submit(GameCommand {
            actor: dodge_c.observer,
            expected_revision: a.revision(),
            dedup_id: 2,
            action: GameAction::Answer {
                choice_id: dodge_c.id,
                selection: vec![engine_native::ChoiceOption::Pass],
            },
        });
        assert!(!matches!(pass_out, engine_native::CommandOutcome::Rejected(_)));
        let res = a.resolution();
        assert!(
            res.parks.len() >= 2 && res.stack.len() >= 2 && res.open_choice.is_some(),
            "必须处于多帧非静止态续体: parks={} stack={}",
            res.parks.len(),
            res.stack.len()
        );
        let observer = res.open_choice.as_ref().unwrap().observer;
        a.state_core_mut()
            .vm_aux
            .flags
            .insert((observer, "obs_flag#1".into()), 5);
        a.state_core_mut()
            .vm_aux
            .storage
            .insert(format!("{observer}:998102001:1"), 5);

        let mut b = a.clone();
        permute_hidden(&mut b, observer, 5);
        {
            let core = b.state_core_mut();
            std::mem::swap(&mut core.players[2].sxianjing_traps[0], &mut core.deck[0]);
            std::mem::swap(
                &mut core.vm_aux.piles.get_mut(&(3, "private_pile".into())).unwrap()[0],
                &mut core.deck[1],
            );
            let opp = (observer + 1) % 4;
            core.vm_aux.flags.insert((opp, "opp_secret#2".into()), 42);
            core.vm_aux.storage.insert("opp_secret#2".into(), 42);
            core.vm_aux.storage.insert(format!("{opp}:998102001:2"), 99);
            core.vm_aux.drawn.insert(opp, 4);
        }

        let plan_a = a.plan_action(observer, &planner());
        let plan_b = b.plan_action(observer, &planner());
        assert_eq!(
            format!("{plan_a:?}"),
            format!("{plan_b:?}"),
            "seed={seed}: 多帧非静止态续体规划结果不得依赖隐藏信息"
        );
        assert_eq!(plan_a.sim_faults, 0);

        let mut red_a = a.clone();
        let mut red_b = b.clone();
        engine_native::ai::redact_opponent_hands(&mut red_a, observer);
        engine_native::ai::redact_opponent_hands(&mut red_b, observer);
        assert_eq!(red_a.state_core(), red_b.state_core());
        assert_eq!(red_a.save().resolution, red_b.save().resolution);
    }

    // Reviewer Attack Surface #1：挂起栈同时包含非空 `cards` 的 `Host` 机器（【让节】2 点伤害）
    // 与内层 `Draw` 事件触发的 `Rvm` 选择窗口（`HOST_CHOICE 2`），验证多帧 `Host + Rvm` 嵌套续体下：
    // 1. `Host(state.cards)` 被固定（不发生悬垂重映射）；
    // 2. 对手手牌、牌库、陷阱、私有 VM 牌堆与 `vm_aux.{storage, flags, drawn}` 全部遮蔽；
    // 3. `plan_action` 完全不依赖隐藏信息且 `sim_faults == 0`。
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
    use rule_vm::bytecode::op;
    use rule_vm::BytecodeModule;
    let mut choice_code = vec![op::CONST_U32];
    choice_code.extend_from_slice(&0u32.to_le_bytes());
    choice_code.push(op::CONST_U32);
    choice_code.extend_from_slice(&1u32.to_le_bytes());
    choice_code.push(op::HOST_CHOICE);
    choice_code.push(2);
    choice_code.push(op::HALT);
    let draw_vm_decl = VmSkillDecl {
        r05_identity: None,
        declaration_hash: None,
        presentation_binding: None,
        name: "nc10_nested_draw_choice".to_string(),
        module: BytecodeModule {
            name: "nc10-nested-draw".to_string(),
            code: choice_code,
            local_count: 1,
            max_stack: 8,
            entry_pc: 0,
            logic_hash: [0x66; 32],
            caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS
                | rule_vm::bytecode::caps::REQUEST_CHOICE,
        },
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::CardsDrawn,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern("nc10-nested-draw".to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc10".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![draw_vm_decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let epic_root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let epic_json =
        std::fs::read_to_string(epic_root.join("epic-cards-upstream-liuli/assets/cards.json"))
            .expect("读史诗卡牌包");
    let epic_pack = content_native_loader::bridge_pack(&epic_json)
        .expect("桥接史诗卡牌包")
        .pack;
    let mut host_vm_a =
        Engine::new_with_extensions(9805, 4, &[&epic_pack, &pack]).expect("建局");
    host_vm_a.debug_set_turn(0, engine_native::Phase::Play, 1);
    host_vm_a.debug_grant(1, "rangjie");
    host_vm_a.debug_grant(1, "nc10_nested_draw_choice");
    // 1 号受到 2 点伤害 → 确认【让节】→ 选择分支 0（摸 1 张牌，此时 HostState.cards 仍剩 1 项非空）
    // → 内层 Draw 帧触发 `nc10_nested_draw_choice` 挂起于 `rvm_choice`
    host_vm_a.debug_deal_damage(Some(0), 1, 2, engine_native::event::DamageNature::Normal);
    let c_confirm = host_vm_a.open_choice().cloned().expect("让节确认门");
    let _ = host_vm_a.submit(GameCommand {
        actor: c_confirm.observer,
        expected_revision: host_vm_a.revision(),
        dedup_id: 10,
        action: GameAction::Answer {
            choice_id: c_confirm.id,
            selection: vec![engine_native::ChoiceOption::Confirm],
        },
    });
    let c_branch = host_vm_a.open_choice().cloned().expect("让节分支选择");
    assert_eq!(c_branch.spec.prompt, "rangjie_choice");
    let _ = host_vm_a.submit(GameCommand {
        actor: c_branch.observer,
        expected_revision: host_vm_a.revision(),
        dedup_id: 11,
        action: GameAction::Answer {
            choice_id: c_branch.id,
            selection: vec![engine_native::ChoiceOption::Tag(0)],
        },
    });
    let c_rvm = host_vm_a.open_choice().cloned().expect("内层 Rvm 选择窗口");
    assert_eq!(c_rvm.spec.prompt, "rvm_choice");
    assert!(
        host_vm_a.resolution().parks.len() >= 2 && host_vm_a.resolution().stack.len() >= 2,
        "parks 应同时包含外层 Host(让节) 与内层 Rvm(nc10_nested_draw_choice)"
    );
    let mut host_vm_b = host_vm_a.clone();
    permute_hidden(&mut host_vm_b, 1, 9);
    host_vm_b
        .state_core_mut()
        .vm_aux
        .flags
        .insert((2, "opp_nested_secret".into()), 123);
    host_vm_b
        .state_core_mut()
        .vm_aux
        .storage
        .insert("2:998102001:7".into(), 456);
    host_vm_b.state_core_mut().vm_aux.drawn.insert(2, 5);

    let plan_hv_a = host_vm_a.plan_action(1, &planner());
    let plan_hv_b = host_vm_b.plan_action(1, &planner());
    assert_eq!(
        format!("{plan_hv_a:?}"),
        format!("{plan_hv_b:?}"),
        "Host(非空 cards) + 内层 Rvm 挂起续体的规划不得依赖隐藏信息"
    );
    assert_eq!(plan_hv_a.sim_faults, 0);
}
