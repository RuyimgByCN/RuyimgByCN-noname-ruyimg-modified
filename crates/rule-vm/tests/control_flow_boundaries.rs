//! Instruction targets must not reinterpret immediate bytes as opcodes.
use rule_api::EffectRequest;
use rule_vm::{
    bytecode::effect_tag, module_uses_effect_tag, op, verify_module, BytecodeModule,
    CollectingHost, ExecResult, ReferenceVm, RuleVm, VerifiedModule,
};

fn module(code: Vec<u8>, entry_pc: u32) -> BytecodeModule {
    let name = "control-flow-probe".to_string();
    let logic_hash = BytecodeModule::compute_logic_hash(&name, &code, 0, 0, entry_pc);
    BytecodeModule {
        name,
        code,
        local_count: 0,
        max_stack: 16,
        entry_pc,
        logic_hash,
        caps_mask: 0,
    }
}

fn hidden_effect() -> [u8; 4] {
    let tag = effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW;
    let [lo, hi] = tag.to_le_bytes();
    [op::HOST_EFFECT_EXT, lo, hi, op::HALT]
}

fn jump_into_immediate() -> BytecodeModule {
    let mut code = vec![op::HOST_SELF, op::HOST_SELF, op::JUMP, 1, 0, op::CONST_I32];
    code.extend_from_slice(&hidden_effect());
    code.push(op::HALT);
    module(code, 0)
}

#[test]
fn rejects_interior_jump_even_when_it_hides_a_gated_effect() {
    let module = jump_into_immediate();
    let tag = effect_tag::CHOOSE_OTHER_TARGETS_BY_SKIPPED_PHASE_COUNT_DAMAGE_DRAW;
    if let Ok(verified) = VerifiedModule::verify(module.clone()) {
        assert!(!module_uses_effect_tag(&module, tag).unwrap());
        let mut host = CollectingHost::default();
        RuleVm::new().execute(&verified, &mut host, 100).unwrap();
        assert!(matches!(
            host.effects.as_slice(),
            [EffectRequest::ChooseOtherTargetsBySkippedPhaseCountDamageDraw { .. }]
        ));
        let mut reference = CollectingHost::default();
        ReferenceVm::new()
            .execute(&module, &mut reference, 100)
            .unwrap();
        assert_eq!(reference.effects, host.effects);
        eprintln!("pre-fix: tag {tag} absent from scan but emitted by both interpreters");
    }
    assert!(
        verify_module(&module).is_err(),
        "jump into an immediate was accepted"
    );
    assert!(
        module_uses_effect_tag(&module, tag).is_err(),
        "ABI scanner must reject malformed control flow"
    );
}

#[test]
fn rejects_entry_and_both_jump_forms_inside_immediates_or_past_end() {
    let entry = module(vec![op::CONST_I32, op::HALT, 0, 0, 0, op::HALT], 1);
    let conditional = module(
        vec![
            op::CONST_BOOL,
            0,
            0,
            0,
            0,
            op::JUMP_IF_FALSE,
            1,
            0,
            op::CONST_I32,
            0,
            0,
            0,
            0,
            op::HALT,
        ],
        0,
    );
    let end = module(vec![op::JUMP, 0, 0], 0);
    let backward = module(
        vec![op::CONST_I32, 0, 0, 0, 0, op::JUMP, 0xfa, 0xff, op::HALT],
        0,
    );
    let accepted: Vec<_> = [
        ("entry", entry),
        ("conditional", conditional),
        ("end", end),
        ("backward", backward),
    ]
    .into_iter()
    .filter_map(|(name, m)| verify_module(&m).is_ok().then_some(name))
    .collect();
    assert!(
        accepted.is_empty(),
        "malformed targets accepted: {accepted:?}"
    );
}

#[test]
fn valid_instruction_targets_and_choice_resume_remain_usable() {
    // Jump forward to Halt; a separate valid entry may start there too.
    let code = vec![op::JUMP, 5, 0, op::CONST_I32, 1, 0, 0, 0, op::HALT];
    for entry in [0, 8] {
        let m = module(code.clone(), entry);
        let verified = VerifiedModule::verify(m.clone()).unwrap();
        RuleVm::new()
            .execute(&verified, &mut CollectingHost::default(), 100)
            .unwrap();
        ReferenceVm::new()
            .execute(&m, &mut CollectingHost::default(), 100)
            .unwrap();
    }
    // A conditional backward jump to an instruction boundary remains valid.
    let loop_code = vec![
        op::CONST_BOOL,
        1,
        0,
        0,
        0,
        op::JUMP_IF_FALSE,
        0xf8,
        0xff,
        op::HALT,
    ];
    let m = module(loop_code, 0);
    let verified = VerifiedModule::verify(m.clone()).unwrap();
    RuleVm::new()
        .execute(&verified, &mut CollectingHost::default(), 100)
        .unwrap();
    ReferenceVm::new()
        .execute(&m, &mut CollectingHost::default(), 100)
        .unwrap();
    assert_eq!(
        serde_json::to_vec(&m).unwrap(),
        serde_json::to_vec(&verified).unwrap()
    );
    let _: VerifiedModule =
        serde_json::from_slice(&serde_json::to_vec(&verified).unwrap()).unwrap();
    // A real pause produces a resume PC immediately after HOST_CHOICE.
    let m = choice_module();
    let verified = VerifiedModule::verify(m.clone()).unwrap();
    let mut host = CollectingHost::default();
    let ExecResult::Paused {
        continuation,
        options,
    } = RuleVm::new().execute(&verified, &mut host, 100).unwrap()
    else {
        panic!("choice must pause")
    };
    assert_eq!(options, [7]);
    let mut vm = RuleVm::new();
    assert!(matches!(
        vm.resume(&verified, &mut host, continuation, 7).unwrap(),
        ExecResult::Halted { .. }
    ));
    let mut reference = ReferenceVm::new();
    let ExecResult::Paused { continuation, .. } = reference.execute(&m, &mut host, 100).unwrap()
    else {
        panic!("reference choice must pause")
    };
    assert!(matches!(
        reference.resume(&m, &mut host, continuation, 7).unwrap(),
        ExecResult::Halted { .. }
    ));

    let range = module(
        vec![
            op::CONST_I32,
            1,
            0,
            0,
            0,
            op::CONST_I32,
            3,
            0,
            0,
            0,
            op::HOST_CHOICE_RANGE,
            op::POP,
            op::HALT,
        ],
        0,
    );
    let verified = VerifiedModule::verify(range.clone()).unwrap();
    let mut optimized = RuleVm::new();
    let ExecResult::Paused {
        continuation,
        options,
    } = optimized.execute(&verified, &mut host, 100).unwrap()
    else {
        panic!("range choice must pause")
    };
    assert_eq!(options, [1, 2, 3]);
    assert!(matches!(
        optimized
            .resume(&verified, &mut host, continuation, 2)
            .unwrap(),
        ExecResult::Halted { .. }
    ));
    let mut reference = ReferenceVm::new();
    let ExecResult::Paused {
        continuation,
        options,
    } = reference.execute(&range, &mut host, 100).unwrap()
    else {
        panic!("reference range choice must pause")
    };
    assert_eq!(options, [1, 2, 3]);
    assert!(matches!(
        reference
            .resume(&range, &mut host, continuation, 2)
            .unwrap(),
        ExecResult::Halted { .. }
    ));
}

fn choice_module() -> BytecodeModule {
    let mut code = vec![
        op::HOST_SELF,
        op::CONST_I32,
        7,
        0,
        0,
        0,
        op::HOST_CHOICE,
        1,
        op::POP,
        op::CONST_I32,
    ];
    code.extend_from_slice(&hidden_effect());
    code.push(op::HALT);
    module(code, 0)
}

#[test]
fn rejects_forged_resume_pc_before_it_can_execute_hidden_immediates() {
    let m = choice_module();
    let verified = VerifiedModule::verify(m.clone()).unwrap();
    let mut optimized = RuleVm::new();
    let mut host = CollectingHost::default();
    let ExecResult::Paused {
        mut continuation, ..
    } = optimized.execute(&verified, &mut host, 100).unwrap()
    else {
        panic!("choice must pause")
    };
    let original = continuation.clone();
    continuation.pc = 10;
    let result = optimized.resume(&verified, &mut host, continuation, 7);
    if result.is_ok() {
        eprintln!("pre-fix: optimized resume emitted {:?}", host.effects);
    }
    let mut reference = ReferenceVm::new();
    let mut host2 = CollectingHost::default();
    let ExecResult::Paused {
        continuation: mut c,
        ..
    } = reference.execute(&m, &mut host2, 100).unwrap()
    else {
        panic!("choice must pause")
    };
    c.pc = 10;
    let result2 = reference.resume(&m, &mut host2, c, 7);
    if result2.is_ok() {
        eprintln!("pre-fix: reference resume emitted {:?}", host2.effects);
    }
    assert!(
        result.is_err() && result2.is_err(),
        "forged continuation accepted by at least one interpreter"
    );
    assert!(
        host.effects.is_empty() && host2.effects.is_empty(),
        "bad continuation must not emit effects"
    );
    for bad_pc in [0, 9, m.code.len() as u32] {
        let mut c = original.clone();
        c.pc = bad_pc;
        assert!(
            optimized
                .resume(&verified, &mut CollectingHost::default(), c.clone(), 7)
                .is_err(),
            "optimized accepted pc {bad_pc}"
        );
        assert!(
            reference
                .resume(&m, &mut CollectingHost::default(), c, 7)
                .is_err(),
            "reference accepted pc {bad_pc}"
        );
    }
}
