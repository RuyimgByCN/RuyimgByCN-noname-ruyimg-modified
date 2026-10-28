// Append to nc04_vm_resume.rs to reproduce on the recorded clean HEAD.
#[test]
fn nc04_vm_restore_uses_declared_skill_for_storage() {
    let mut code = choice_then_halt();
    code.pop();
    for value in [0u32, 7, 11] {
        code.push(op::CONST_U32);
        code.extend_from_slice(&value.to_le_bytes());
    }
    code.push(op::HOST_EFFECT_EXT);
    code.extend_from_slice(&649u16.to_le_bytes());
    code.push(op::HALT);
    let pack = pack("nc04-storage-module", code, "nc04_storage_skill");
    let mut original = Engine::new_with_extensions(20261045, 4, &[&pack]).unwrap();
    original.debug_grant(1, "nc04_storage_skill");
    let choice = pending_vm_choice(&mut original);
    let snapshot = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&original.save()),
    ).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    let dedup = next_dedup();
    for e in [&mut original, &mut restored] {
        let out = e.submit(GameCommand {
            actor: choice.observer,
            expected_revision: e.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: choice.id,
                selection: vec![engine_native::ChoiceOption::Tag(1)],
            },
        });
        assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out:?}");
    }
    eprintln!("original storage: {:?}", original.save().core.vm_aux.storage);
    eprintln!("restored storage: {:?}", restored.save().core.vm_aux.storage);
    assert_eq!(original.save().core.vm_aux.storage, restored.save().core.vm_aux.storage);
}
