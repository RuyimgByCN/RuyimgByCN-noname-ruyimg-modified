//! NC08 登记复核项的反例探针（2026-10-28；只取证不改语义）。
//!
//! **现行定案**（规则基准「触发全序」，2026-10-16）：同角色同优先级非
//! silent 技能按 `SkillDefId` 决胜的确定性全序，不向玩家开放顺序选择。
//! **上游对照**（`archive…/content.ts` arrangeTrigger，摘录见
//! docs/operations/evidence/2026-10-17-cov-permanent-diffs/sources.txt）：
//! 普通时机同优先级多个非 silent（有描述）技能时，置 `event.choice` 交
//! `chooseControl` 由玩家选择下一个触发的技能。
//!
//! 本探针锁定反例两面：
//! 1. 同事件/同时机/同优先级的两个可选 VM 技能，触发顺序严格跟随注册
//!    （def）序——翻转注册序即翻转执行序（def_id 决胜，非玩家决定）；
//! 2. 整个触发窗口**只出现**逐技能的 confirm 询问（`trigger_confirm`），
//!    不存在任何「选择发动顺序」的询问——玩家无法影响顺序，与上游
//!    `chooseControl` 的交互并列分歧即 NC08 待复核点。
//!
//! 复核改判（玩家可选顺序）须另立 ADR + 授权升规则版本；届时本测试的
//! 断言②将反转为「存在排序询问」，断言①按新裁决重录。

use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use rule_vm::bytecode::op;
use rule_vm::BytecodeModule;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);
fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn draw_module(name: &str, count: u32) -> BytecodeModule {
    // draw_cards(rule_pid=2（座位 1）, count) → HALT
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&2u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&count.to_le_bytes());
    c.push(op::HOST_EFFECT_EXT);
    c.extend_from_slice(&1u16.to_le_bytes()); // DRAW_CARDS
    c.push(op::HALT);
    BytecodeModule {
        name: name.to_string(),
        code: c,
        local_count: 1,
        max_stack: 8,
        entry_pc: 0,
        logic_hash: [9u8; 32],
        caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS,
    }
}

/// 两个可选 VM 技能，同一 TriggerMeta（PhaseEnd/Normal/priority 0）。
fn order_pack(first: (&str, u32), second: (&str, u32)) -> ExtensionPack {
    let decl = |name: &str, count: u32| VmSkillDecl {
        name: name.to_string(),
        module: draw_module(name, count),
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::PhaseEnd,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            optional: true,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(format!("nc08-order-{}-{}", first.0, second.0)),
            version: 1,
            description: engine_native::content::names::intern("nc08".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl(first.0, first.1), decl(second.0, second.1)],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    }
}

fn drain(e: &mut Engine, seen_prompts: &mut Vec<&'static str>) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        seen_prompts.push(c.spec.prompt);
        let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 推进到 1 号回合结束（触发点），收集窗口内全部询问的 prompt。
fn drive_to_phase_end(e: &mut Engine, prompts: &mut Vec<&'static str>) {
    for _ in 0..40 {
        if e.turn().current == 1 && e.turn().phase == Phase::Discard {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(e, prompts);
    }
    let actor = e.turn().current;
    let _ = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
}

/// 逐个应答 confirm 门并记录每次应答后座位 1 的手牌量（技能执行序可观测）。
fn answer_gates_observing_hand(e: &mut Engine, prompts: &mut Vec<&'static str>) -> Vec<usize> {
    let mut deltas = Vec::new();
    let mut hand = hand_len(e, 1);
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        prompts.push(c.spec.prompt);
        if c.spec.prompt != "trigger_confirm" {
            // 非技能门询问按最小项应答（不参与顺序观测）
            let sel: Vec<ChoiceOption> = if c.spec.options.contains(&ChoiceOption::Pass) {
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
            hand = hand_len(e, 1);
            continue;
        }
        let out = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Confirm] },
        });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "{out:?}"
        );
        let now = hand_len(e, 1);
        deltas.push(now - hand);
        hand = now;
    }
    deltas
}

/// 反例第①②面：执行序跟随注册（def）序；窗口内只有 confirm 询问。
#[test]
fn nc08_same_priority_pair_resolves_in_def_order_without_player_choice() {
    // 注册序 [甲(摸1), 乙(摸2)]
    let pack_ab = order_pack(("nc08_order_a", 1), ("nc08_order_b", 2));
    let mut e = Engine::new_with_extensions(20261060, 4, &[&pack_ab]).unwrap();
    e.debug_grant(1, "nc08_order_a");
    e.debug_grant(1, "nc08_order_b");
    let mut prompts = Vec::new();
    drive_to_phase_end(&mut e, &mut prompts);
    let deltas = answer_gates_observing_hand(&mut e, &mut prompts);
    drain(&mut e, &mut prompts);
    assert_eq!(deltas, vec![1, 2], "注册序 a(摸1)→b(摸2) 决定执行序");

    // 翻转注册序 [乙(摸2), 甲(摸1)]：同一触发面，执行序随之翻转——
    // 顺序由 def 序决定，与玩家无关。
    let pack_ba = order_pack(("nc08_order_b", 2), ("nc08_order_a", 1));
    let mut e2 = Engine::new_with_extensions(20261061, 4, &[&pack_ba]).unwrap();
    e2.debug_grant(1, "nc08_order_a");
    e2.debug_grant(1, "nc08_order_b");
    let mut prompts2 = Vec::new();
    drive_to_phase_end(&mut e2, &mut prompts2);
    let deltas2 = answer_gates_observing_hand(&mut e2, &mut prompts2);
    drain(&mut e2, &mut prompts2);
    assert_eq!(deltas2, vec![2, 1], "翻转注册序后执行序翻转（def_id 决胜）");

    // 反例第②面：两个对局的双技能触发窗口内，出现的询问只有逐技能
    // confirm（trigger_confirm）与常规阶段询问——不存在任何排序询问。
    let ordering_prompts: Vec<_> = prompts
        .iter()
        .chain(prompts2.iter())
        .filter(|p| !matches!(**p, "trigger_confirm" | "discard_to_limit" | "rvm_choice"))
        .collect();
    assert!(
        ordering_prompts.is_empty(),
        "现行定案下不应出现排序/其他询问，实际：{ordering_prompts:?}"
    );
}
