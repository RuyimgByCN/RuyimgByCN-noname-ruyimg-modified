use engine_native_wasm::{NativeEngine, NativeSession};

#[test]
fn native_errors_are_strings_and_leave_the_session_usable() {
    let mut session = NativeSession::new(4, 42).unwrap();
    let before = session.save_hex();
    assert!(session.view_json(255).unwrap_err().contains("座位越界"));
    assert!(session
        .answer(0, 123, vec![])
        .unwrap_err()
        .contains("无开放选择"));
    assert!(session.restore_hex("中").unwrap_err().contains("ASCII"));
    assert!(session.assign_general(0, "missing-general").is_err());
    assert_eq!(session.save_hex(), before);
    assert!(session.view_json(0).is_ok());
}

#[test]
fn zero_deadline_keeps_the_node_bounded_candidate_policy() {
    let mut unbounded = NativeSession::new(4, 42).unwrap();
    let mut generous = NativeSession::new(4, 42).unwrap();
    unbounded.set_planner(64, 0);
    generous.set_planner(64, 10_000);
    for step in 0..64 {
        assert_eq!(
            unbounded.auto_step().unwrap(),
            generous.auto_step().unwrap(),
            "step {step}"
        );
        assert_eq!(
            unbounded.state_hash_hex(),
            generous.state_hash_hex(),
            "step {step}"
        );
    }
}

#[test]
fn native_session_and_existing_wasm_api_keep_identical_trajectories() {
    let mut native = NativeSession::new(4, 42).unwrap();
    let mut wasm = NativeEngine::new(4, 42).unwrap();
    native.set_planner(0, 0);
    wasm.set_planner(0, 0);
    for _ in 0..64 {
        assert_eq!(native.state_hash_hex(), wasm.state_hash_hex());
        assert_eq!(native.view_json(0).unwrap(), wasm.view_json(0).unwrap());
        assert_eq!(native.auto_step().unwrap(), wasm.auto_step().unwrap());
    }
    let saved = native.save_hex();
    wasm.restore_hex_checked(&saved).unwrap();
    assert_eq!(native.state_hash_hex(), wasm.state_hash_hex());
    native.restore_hex(&wasm.save_hex()).unwrap();
    assert_eq!(native.state_hash_hex(), wasm.state_hash_hex());
}

#[test]
fn auto_step_answers_real_empty_hujia_window_without_panicking() {
    use engine_native::{ai::BoundedPlanner, CommandOutcome, Engine};

    let mut engine = Engine::new(45000, 8);
    for seat in 0..8 {
        engine
            .assign_general_named(seat, ["曹操", "刘备", "孙权", "赵云"][seat as usize % 4])
            .unwrap();
    }
    let mut pending = None;
    for _ in 0..100 {
        if let Some(choice) = engine.open_choice() {
            if choice.spec.options.is_empty() && choice.spec.min == 0 && !choice.spec.allow_cancel {
                assert_eq!(choice.spec.prompt, "hujia_respond");
                pending = Some((engine.save(), choice.observer, choice.id));
                break;
            }
        }
        let actor = engine
            .open_choice()
            .map(|c| c.observer)
            .unwrap_or(engine.turn().current);
        let plan = engine.plan_action(actor, &BoundedPlanner::new(8));
        assert!(!matches!(
            engine.apply_plan(&plan).unwrap(),
            CommandOutcome::Rejected(_)
        ));
    }
    let (snapshot, observer, choice_id) = pending.expect("real empty hujia response window");
    let archive =
        engine_native_wasm::hex_encode(&engine_native::archive::encode_archive(&snapshot));
    let mut expected = NativeSession::new(8, 45000).unwrap();
    expected.restore_hex_checked(&archive).unwrap();
    expected.answer(observer as u32, choice_id, vec![]).unwrap();

    let mut native = NativeSession::new(8, 45000).unwrap();
    native.restore_hex_checked(&archive).unwrap();
    let reply = native.auto_step().unwrap();
    let parsed: serde_json::Value = serde_json::from_str(&reply).unwrap();
    assert_eq!(parsed["kind"], "answer");
    assert_eq!(native.save_hex(), expected.save_hex());

    let mut binding = NativeEngine::new(8, 45000).unwrap();
    binding.restore_hex_checked(&archive).unwrap();
    assert_eq!(binding.auto_step().unwrap(), reply);
    assert_eq!(binding.save_hex(), native.save_hex());
}

#[test]
fn source_actual_native_ask_survives_session_and_binding_json() {
    let mut engine = engine_native::Engine::new(2306, 3);
    engine.debug_set_turn(0, engine_native::Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let hex =
        engine_native_wasm::hex_encode(&engine_native::archive::encode_archive(&engine.save()));
    let mut session = NativeSession::new(3, 0).unwrap();
    session.restore_hex(&hex).unwrap();
    let mut binding = NativeEngine::new(3, 0).unwrap();
    binding.restore_hex_checked(&hex).unwrap();
    let before = session.save_hex();
    for seat in 0..3 {
        let raw = session.open_choice_json(seat).unwrap();
        assert_eq!(raw, binding.open_choice_json(seat).unwrap());
        let value: serde_json::Value = serde_json::from_str(&raw).unwrap();
        if seat == 1 {
            assert_eq!(value["source_skill"], "遗计");
            assert_eq!(value["prompt"], "trigger_confirm");
        } else {
            assert!(value.get("source_skill").is_none());
            assert_eq!(value["hidden"], true);
        }
    }
    assert_eq!(session.save_hex(), before);
    assert_eq!(binding.save_hex(), before);
}
