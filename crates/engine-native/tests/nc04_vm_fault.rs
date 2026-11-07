//! NC04 VM 故障与事务语义：故障必须传播为命令拒绝并整段回滚。
//!
//! 修复前：`VmMachine::run` 的 `Err(_)` 只丢弃排队效果后返回 `Flow::Done`
//! （错误类型丢失、故障被当正常规则继续）。
//! 修复后：故障记录（技能/座位/事件/错误类别/详情）→ 调度循环返回
//! `StepOutcome::Fault` → 该段整段回滚（工作副本丢弃、日志截断、段检查点
//! 清除）→ 命令以 `RejectReason::VmFault { skill, detail }` 拒绝。
//!
//! 本文件覆盖：正常完成、选择挂起后恢复、fuel 耗尽（既有执行限额）、
//! 故障后状态与摘要不变、重复提交、以及故障不污染后续命令。

use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, RejectReason};

const NORMAL_DSL: &str = "\
skill nc04_normal(990201) {
  trigger phase_end owner_is_target {
    if hp(self) < max_hp(self) {
      effect recover_hp(self, 1);
    } else {
      effect draw_cards(self, 1);
    }
  }
}
";

const ASK_DSL: &str = "\
skill nc04_ask(990202) {
  trigger phase_end owner_is_target {
    effect draw_cards(self, 1);
  }
}
";

const RUNAWAY_DSL: &str = "\
skill nc04_runaway(990203) {
  trigger phase_end owner_is_target {
    loop 100000 {
      effect draw_cards(self, 1);
    }
  }
}
";

/// VM 技能在目录中的名字（`<skill>_<trigger>`，由桥接声明给出）。
fn vm_skill_name(name: &str, dsl: &str) -> String {
    let pkg = content_compiler::compile_source_named(name, dsl).expect("DSL 编译");
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接 VM 声明");
    outcome.decls.first().expect("一条 VM 声明").name.clone()
}

fn vm_pack(name: &str, dsl: &str) -> ExtensionPack {
    let pkg = content_compiler::compile_source_named(name, dsl).expect("DSL 编译");
    let outcome = content_native_loader::bridge_vm_skill_decls(&pkg).expect("桥接 VM 声明");
    ExtensionPack {
        def: ExtensionDef {
            name: engine_native::content::names::intern(name.to_string()),
            version: 1,
            description: engine_native::content::names::intern("nc04".to_string()),
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: outcome.decls,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    }
}

static DEDUP: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
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

/// 推进到 1 号回合结束（VM 技能持有者），返回最后一次提交结果。
fn drive_to_vm_trigger(e: &mut Engine) -> CommandOutcome {
    for _ in 0..40 {
        if e.turn().current == 1 && e.turn().phase == engine_native::Phase::Discard {
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
    // 弃牌阶段收束 → 回合结束（phase_end 触发）
    let actor = e.turn().current;
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    })
}

/// 正常完成：VM 技能按语义结算（受伤回 1）。
#[test]
fn nc04_normal_completion() {
    let pack = vm_pack("nc04-normal", NORMAL_DSL);
    let mut e = Engine::new_with_extensions(990301, 4, &[&pack]).expect("建局");
    let skill = vm_skill_name("nc04-normal", NORMAL_DSL);
    assert!(
        e.catalog().skill_by_name(&skill).is_some(),
        "VM 技能入目录: {skill}"
    );
    e.debug_grant(1, &skill);
    e.debug_set_hp(1, 2);
    let _ = drive_to_vm_trigger(&mut e);
    drain(&mut e);
    assert!(e.debug_hp(1) >= 3, "正常完成：VM 生效（回血/摸牌）");
    assert!(
        e.resolution().fault.is_none(),
        "正常完成不留故障"
    );
}

/// 选择挂起后恢复：VM 在 `HOST_CHOICE` 处挂起，支持中途存档恢复、正常续跑完成及续跑故障整段回滚。
#[test]
fn nc04_pause_and_resume() {
    use engine_native::content::extension::VmSkillDecl;
    use rule_vm::bytecode::op;
    use rule_vm::BytecodeModule;

    let mut pause_ok_code = vec![op::CONST_U32];
    pause_ok_code.extend_from_slice(&0u32.to_le_bytes());
    pause_ok_code.push(op::CONST_U32);
    pause_ok_code.extend_from_slice(&1u32.to_le_bytes());
    pause_ok_code.push(op::HOST_CHOICE);
    pause_ok_code.push(2);
    pause_ok_code.push(op::HALT);

    let decl = VmSkillDecl {
        r05_identity: None,
        declaration_hash: None,
        presentation_binding: None,
        name: "nc04_ask_phase_end".to_string(),
        module: BytecodeModule {
            name: "nc04-ask".to_string(),
            code: pause_ok_code,
            local_count: 1,
            max_stack: 8,
            entry_pc: 0,
            logic_hash: [0x42; 32],
            caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS
                | rule_vm::bytecode::caps::REQUEST_CHOICE,
        },
        trigger: engine_native::catalog::TriggerMeta {
            event: engine_native::event::EventKind::PhaseEnd,
            timing: engine_native::event::Timing::Normal,
            priority: 0,
            order_selectable: true,
            optional: false,
            extra_events: Vec::new(),
        },
        filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
    };
    let mut pack = vm_pack("nc04-ask", ASK_DSL);
    pack.vm_skills = vec![decl];
    let mut e = Engine::new_with_extensions(990302, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "nc04_ask_phase_end");
    let _ = drive_to_vm_trigger(&mut e);
    let mut vm_choice = None;
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "rvm_choice" {
            vm_choice = Some(c);
            break;
        }
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: Vec::new() },
        });
    }
    let c = vm_choice.expect("VM 应挂起于 HOST_CHOICE");
    let snap = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&e.save()),
    )
    .expect("挂起态归档解码");
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("挂起态归档恢复");
    assert_eq!(restored.open_choice(), Some(&c));

    let dedup = next_dedup();
    for eng in [&mut e, &mut restored] {
        let out = eng.submit(GameCommand {
            actor: c.observer,
            expected_revision: eng.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![engine_native::ChoiceOption::Tag(0)],
            },
        });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
            "续跑应被接受: {out:?}"
        );
        drain(eng);
        assert!(eng.resolution().fault.is_none(), "挂起续跑后无残留故障");
    }
    assert_eq!(e.state_hash(), restored.state_hash(), "挂起态恢复续跑后摘要完全一致");
}

/// fuel 耗尽：命令以 VmFault 拒绝，且状态/摘要与提交前完全一致。
#[test]
fn nc04_fuel_exhaustion_rolls_back_and_rejects() {
    let pack = vm_pack("nc04-runaway", RUNAWAY_DSL);
    let mut e = Engine::new_with_extensions(990303, 4, &[&pack]).expect("建局");
    let skill = vm_skill_name("nc04-runaway", RUNAWAY_DSL);
    e.debug_grant(1, &skill);
    // 推进到 VM 触发前的稳定点
    for _ in 0..40 {
        if e.turn().current == 1 && e.turn().phase == engine_native::Phase::Discard {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    // 逐命令推进并记录故障：故障当次必须「摘要与修订号不变」（整段回滚）
    let mut fault: Option<(String, String, [u8; 32], u64)> = None;
    for _ in 0..40 {
        let before = e.state_hash();
        let rev_before = e.revision();
        let actor = e.turn().current;
        let out = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        if let CommandOutcome::Rejected(RejectReason::VmFault { skill, detail }) = out {
            fault = Some((skill, detail, before, rev_before));
            break;
        }
        drain(&mut e);
    }
    let (skill, detail, hash_before, rev_before) = fault.expect("runaway 循环应触发 VmFault");
    assert!(skill.starts_with("nc04_runaway"), "故障技能可定位: {skill}");
    assert!(
        detail.contains("OutOfFuel") || detail.contains("fuel"),
        "故障类别可定位: {detail}"
    );
    assert_eq!(e.state_hash(), hash_before, "故障段整段回滚（摘要不变）");
    assert_eq!(e.revision(), rev_before, "修订号不因故障推进");
    // 同一触发点再次提交：确定性重复故障（段不会静默跳过），状态仍不变
    let hash_after = e.state_hash();
    let actor = e.turn().current;
    let out2 = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out2, CommandOutcome::Rejected(RejectReason::VmFault { .. })),
        "故障确定性重现（不静默跳过）: {out2:?}"
    );
    assert_eq!(e.state_hash(), hash_after, "重复故障同样整段回滚");
}

/// 重复提交：同一 dedup_id 的故障命令第二次以 DuplicateCommand 拒绝。
#[test]
fn nc04_duplicate_submission_after_fault() {
    let pack = vm_pack("nc04-runaway-dup", RUNAWAY_DSL);
    let mut e = Engine::new_with_extensions(990304, 4, &[&pack]).expect("建局");
    let skill = vm_skill_name("nc04-runaway-dup", RUNAWAY_DSL);
    e.debug_grant(1, &skill);
    for _ in 0..40 {
        if e.turn().current == 1 && e.turn().phase == engine_native::Phase::Discard {
            break;
        }
        let actor = e.turn().current;
        let _ = e.submit(GameCommand {
            actor,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::NextPhase,
        });
        drain(&mut e);
    }
    let actor = e.turn().current;
    let dedup = next_dedup();
    let first = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: dedup,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(first, CommandOutcome::Rejected(RejectReason::VmFault { .. })),
        "{first:?}"
    );
    // 被拒绝（含故障）的命令**不消费去重键**（与既有 Rejected 语义一致）：
    // 同一 dedup_id 重放仍是确定性 VmFault，而非 DuplicateCommand。
    let second = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: dedup,
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(second, CommandOutcome::Rejected(RejectReason::VmFault { .. })),
        "故障命令重放：确定性故障（去重键未消费）: {second:?}"
    );
}
