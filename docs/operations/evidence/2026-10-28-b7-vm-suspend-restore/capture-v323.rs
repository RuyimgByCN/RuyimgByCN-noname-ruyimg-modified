//! B7 前置捕获：在 v323 源码上产出两类基线存档。
//! 1) native-v323.ngsa：普通新局（无 VM 挂起）——升版后应按规则版本拒绝。
//! 2) vm-suspended-v323.ngsa：HOST_CHOICE 挂起中的 VM 机器存档——旧格式
//!    不含技能身份字段，修复后应无法解码（不能可靠恢复）。
//! 同时记录原局续答后的 vm_aux 存储/限次/命中，供证据对照。

use engine_native::content::extension::{ExtensionDef, ExtensionPack, VmSkillDecl};
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand, Phase};
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
        caps_mask: rule_vm::bytecode::caps::EMIT_EFFECTS
            | rule_vm::bytecode::caps::REQUEST_CHOICE,
    }
}

/// 选项 0/1 → HOST_CHOICE 2 → 续跑后 STORAGE_SET(649)：key=7 value=11。
fn choice_then_storage() -> Vec<u8> {
    let mut c = vec![op::CONST_U32];
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&1u32.to_le_bytes());
    c.push(op::HOST_CHOICE);
    c.push(2);
    c.push(op::CONST_U32);
    c.extend_from_slice(&0u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&7u32.to_le_bytes());
    c.push(op::CONST_U32);
    c.extend_from_slice(&11u32.to_le_bytes());
    c.push(op::HOST_EFFECT_EXT);
    c.extend_from_slice(&649u16.to_le_bytes());
    c.push(op::HALT);
    c
}

fn storage_pack() -> ExtensionPack {
    let decl = VmSkillDecl {
        name: "b7_storage_skill".to_string(),
        module: module("b7-storage-module", choice_then_storage()),
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
            name: engine_native::content::names::intern("b7-storage-module".to_string()),
            version: 1,
            description: engine_native::content::names::intern("b7".to_string()),
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

fn wait_rvm_choice(e: &mut Engine) -> engine_native::OpenChoice {
    for _ in 0..8 {
        let c = e.open_choice().cloned().expect("等待选择");
        if c.spec.prompt == "rvm_choice" {
            return c;
        }
        let selection = if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection },
        });
    }
    panic!("未出现 VM 选择");
}

#[test]
fn capture_b7_v323_baselines() {
    let out_dir = std::env::var("B7_OUT").expect("B7_OUT");
    // 1) 普通新局
    let plain = Engine::new(65101, 4);
    let plain_bytes = engine_native::archive::encode_archive(&plain.save());
    std::fs::write(format!("{out_dir}/native-v323.ngsa"), &plain_bytes).unwrap();
    println!("plain archive version={} bytes={}", plain.save().ruleset_version, plain_bytes.len());

    // 2) HOST_CHOICE 挂起中的 VM 局
    let pack = storage_pack();
    let mut e = Engine::new_with_extensions(20261046, 4, &[&pack]).expect("建局");
    e.debug_grant(1, "b7_storage_skill");
    drive_to_phase_end(&mut e);
    let choice = wait_rvm_choice(&mut e);
    let suspended = engine_native::archive::encode_archive(&e.save());
    std::fs::write(format!("{out_dir}/vm-suspended-v323.ngsa"), &suspended).unwrap();
    println!(
        "suspended archive version={} bytes={} choice_id={}",
        e.save().ruleset_version,
        suspended.len(),
        choice.id
    );

    // 3) 原局续答 → 记录基线 vm_aux（供修复后对照）
    let out = e.submit(GameCommand {
        actor: choice.observer,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::Answer {
            choice_id: choice.id,
            selection: vec![engine_native::ChoiceOption::Tag(1)],
        },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    let snap = e.save();
    println!("original storage={:?}", snap.core.vm_aux.storage);
    println!("original once={:?}", snap.core.vm_aux.once);
    println!("original hits={:?}", snap.core.vm_aux.hits);
    std::fs::write(
        format!("{out_dir}/original-continuation.txt"),
        format!(
            "storage={:?}\nonce={:?}\nhits={:?}\nfinal_hash={:?}\n",
            snap.core.vm_aux.storage,
            snap.core.vm_aux.once,
            snap.core.vm_aux.hits,
            e.state_hash()
        ),
    )
    .unwrap();
}
