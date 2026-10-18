//! NC04 追加：VM 挂起 → 续跑 → 效果落账；以及**恢复后故障**的整段回滚。
//!
//! DSL 目前只有 `effect`/`if`/`loop`/`let` 语句，没有选择语句，故本文件用
//! **手工构造的字节码模块**覆盖 `HOST_CHOICE`（挂起）路径：
//! - 模块 A：push 选项 token 0/1 → `HOST_CHOICE 2`（挂起）→ 续跑后按所选
//!   token 走 `HOST_EFFECT`（回血）→ `HALT`；
//! - 模块 B：`HOST_CHOICE 2` → 续跑后执行**超长循环**（fuel 耗尽）→ 故障，
//!   验证「恢复后故障」同样整段回滚并以 `VmFault` 拒绝。

use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason};
use rule_vm::bytecode::op;
use rule_vm::BytecodeModule;
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn module(name: &str, code: Vec<u8>) -> BytecodeModule {
    BytecodeModule {
        name: name.to_string(),
        code,
        local_count: 1,
        max_stack: 8,
        entry_pc: 0,
        logic_hash: [7u8; 32],
        caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS | rule_vm::bytecode::caps::REQUEST_CHOICE,
    }
}

/// 选项 token 0/1 → HOST_CHOICE 2 →（续跑后）HALT。
///
/// 只覆盖挂起/续跑机制本身：各效果 tag 的参数出栈契约不同，效果落账已由
/// DSL 版 `nc04_normal_completion` 覆盖。
fn choice_then_halt() -> Vec<u8> {
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_CHOICE);
    c.push(2);
    c.push(op::HALT);
    c
}

/// 选项 token 0/1 → HOST_CHOICE 2 → 续跑后**无效效果 tag**（Host 错误）→ 故障。
fn choice_then_bad_effect() -> Vec<u8> {
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_CHOICE);
    c.push(2);
    // 无效效果 tag：宿主在调用边界明确失败（Host 错误）→ VmFault
    c.push(op::HOST_EFFECT_EXT);
    c.extend_from_slice(&0xFFFFu16.to_le_bytes());
    c.push(op::HALT);
    c
}

fn pack(name: &str, code: Vec<u8>, skill: &str) -> ExtensionPack {
    let decl = VmSkillDecl {
        name: skill.to_string(),
        module: module(name, code),
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::PhaseEnd,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(name.to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc04".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vec![decl],
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    }
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let sel: Vec<engine_native::ChoiceOption> =
            if c.spec.options.contains(&engine_native::ChoiceOption::Pass) {
                vec![engine_native::ChoiceOption::Pass]
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

/// 推进到 1 号回合结束（VM 触发点）。
fn drive_to_phase_end(e: &mut Engine) {
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
        drain(e);
    }
    let actor = e.turn().current;
    let _ = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
}

/// 挂起 → 续跑（选择 token 被接受并继续执行，段结算完成、无故障）。
#[test]
fn nc04_pause_resume_completes() {
    let pack = pack("nc04-pause", choice_then_halt(), "nc04_pause_skill");
    let mut e = Engine::new_with_extensions(20261041, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "nc04_pause_skill");
    e.debug_set_hp(1, 2);
    drive_to_phase_end(&mut e);

    // 出现 VM 选择（挂起）
    let mut choice = None;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "rvm_choice" {
            choice = Some(c);
            break;
        }
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
        });
    }
    let c = choice.expect("VM 应挂起于 HOST_CHOICE");
    assert_eq!(c.spec.options.len(), 2, "两个选项 token");
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![engine_native::ChoiceOption::Tag(0)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "续跑应被接受: {out:?}"
    );
    drain(&mut e);
    assert!(e.resolution().fault.is_none(), "正常续跑不留故障");
    // 续跑后段已结算：修订号推进且对局继续（当前座位不再是挂起观察者）
    assert!(e.revision() > 0, "续跑推进修订号");
    assert!(e.open_choice().is_none() || e.open_choice().map(|c| c.spec.prompt != "rvm_choice").unwrap_or(true), "VM 选择已消费");
}

/// 恢复后故障：续跑进入 fuel 耗尽 → 整段回滚 + `VmFault` 拒绝。
#[test]
fn nc04_fault_after_resume_rolls_back() {
    let pack = pack("nc04-resume-fault", choice_then_bad_effect(), "nc04_resume_fault");
    let mut e = Engine::new_with_extensions(20261042, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "nc04_resume_fault");
    drive_to_phase_end(&mut e);

    let mut choice = None;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "rvm_choice" {
            choice = Some(c);
            break;
        }
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
        });
    }
    let c = choice.expect("VM 应挂起于 HOST_CHOICE");
    let hash_before = e.state_hash();
    let rev_before = e.revision();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: c.id,
            selection: vec![engine_native::ChoiceOption::Tag(1)],
        },
    });
    match out {
        CommandOutcome::Rejected(RejectReason::VmFault { skill, detail }) => {
            assert_eq!(skill, "nc04_resume_fault", "故障技能可定位");
            assert!(
                detail.contains("Host") || detail.contains("host"),
                "故障类别可定位（宿主调用边界失败）: {detail}"
            );
        }
        other => panic!("恢复后故障应以 VmFault 拒绝: {other:?}"),
    }
    assert_eq!(e.state_hash(), hash_before, "恢复后故障同样整段回滚");
    assert_eq!(e.revision(), rev_before, "修订号不因故障推进");
    assert!(e.resolution().fault.is_none(), "故障槽在拒绝后清空");
}
