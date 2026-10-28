// Replace the identically named test in nc04_vm_resume.rs with this probe.
#[test]
fn nc04_vm_fault_after_archive_restores_choice_and_allows_retry() {
    let pack = pack("nc04-restored-fault", choice_then_bad_effect(), "nc04_restored_fault");
    let mut original = Engine::new_with_extensions(20261044, 4, &[&pack]).unwrap();
    original.debug_grant(1, "nc04_restored_fault");
    let choice = pending_vm_choice(&mut original);
    let snapshot = engine_native::archive::decode_archive(
        &engine_native::archive::encode_archive(&original.save()),
    ).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &[&pack]).unwrap();
    let dedup = next_dedup();
    for e in [&mut original, &mut restored] {
        let before = e.save();
        let log = e.log().to_vec();
        let hash = e.state_hash();
        for _ in 0..2 {
            let out = e.submit(GameCommand {
                actor: choice.observer,
                expected_revision: e.revision(),
                dedup_id: dedup,
                action: GameAction::Answer {
                    choice_id: choice.id,
                    selection: vec![engine_native::ChoiceOption::Tag(1)],
                },
            });
            match out {
                CommandOutcome::Rejected(RejectReason::VmFault { skill, detail }) => {
                    assert_eq!(skill, "nc04_restored_fault");
                    assert!(detail.contains("Host") || detail.contains("host"), "{detail}");
                }
                other => panic!("续跑应报告 VM 故障: {other:?}"),
            }
            assert_eq!(e.save(), before, "故障不改变快照、随机流或去重水位");
            assert_eq!(e.state_hash(), hash);
            assert_eq!(e.log(), log);
            assert_eq!(e.open_choice(), Some(&choice));
            assert!(e.resolution().fault.is_none());
        }
    }
    assert_eq!(original.state_hash(), restored.state_hash());
}
