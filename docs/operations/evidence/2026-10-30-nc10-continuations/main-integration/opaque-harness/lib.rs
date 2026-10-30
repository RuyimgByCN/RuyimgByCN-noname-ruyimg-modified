mod fixture;

use engine_native::ai::BoundedPlanner;
use engine_native::{CommandOutcome, Engine, MachineSnapshot};
use serde_json::{json, Value};

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|byte| format!("{byte:02x}")).collect()
}
fn archive(engine: &Engine) -> Vec<u8> {
    engine_native::archive::encode_archive(&engine.save())
}
fn projected(engine: &Engine) -> Value {
    json!({
        "input": format!("{:?}", engine.ai_session(0).input()),
        "choice": format!("{:?}", engine.ai_session(0).choice()),
        "events": format!("{:?}", engine.ai_session(0).events()),
    })
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen)]
pub fn run_opaque_planner_matrix() -> String {
    let pack = fixture::pack();
    let pair = [fixture::fixture(&pack, false), fixture::fixture(&pack, true)];
    let originals = [archive(&pair[0]), archive(&pair[1])];
    let public = [projected(&pair[0]), projected(&pair[1])];
    assert_eq!(public[0], public[1], "complete public policy input/choice/events");
    assert_ne!(originals[0], originals[1], "counterfactual private states differ");
    let mut private_locals = Vec::new();
    for (engine, expected) in pair.iter().zip([0i32, 4i32]) {
        let snapshot = engine.save();
        let MachineSnapshot::Rvm { paused: Some(paused), owner, .. } =
            &snapshot.resolution.parks[0].machine else { panic!("reachable paused VM") };
        assert_eq!(*owner, 0);
        let local = i32::from_le_bytes(paused.frames[2..6].try_into().unwrap());
        assert_eq!(local, expected, "actual query -> STORE private local");
        private_locals.push(json!({"owner": owner, "local0": local, "pc": paused.pc, "frames": paused.frames}));
    }
    let mut budgets = Vec::new();
    for budget in [0, 48] {
        let planner = BoundedPlanner::new(budget).with_policy_variant(1).with_lookahead(2);
        let plans = [pair[0].plan_action(0, &planner), pair[1].plan_action(0, &planner)];
        assert_eq!(format!("{:?}", plans[0]), format!("{:?}", plans[1]), "complete counterfactual Plans");
        let mut variants = Vec::new();
        for (index, (source, plan)) in pair.iter().zip(plans.iter()).enumerate() {
            assert_eq!((plan.nodes_used, plan.sim_faults), (0, 0));
            assert!(!plan.cancelled && !plan.timed_out);
            assert_eq!(archive(source), originals[index], "source archive immutable after planning");
            assert_eq!(projected(source), public[index]);
            let decoded = engine_native::archive::decode_archive(&originals[index]).unwrap();
            let mut restored = Engine::restore_with_extensions(&decoded, &[&pack]).unwrap();
            let log_start = restored.log().len();
            let outcome = restored.apply_plan(plan).unwrap();
            assert!(!matches!(outcome, CommandOutcome::Rejected(_)), "restored plan admitted");
            let after = archive(&restored);
            let decoded_after = engine_native::archive::decode_archive(&after).unwrap();
            Engine::restore_with_extensions(&decoded_after, &[&pack]).unwrap();
            variants.push(json!({
                "privateLocal0": private_locals[index]["local0"],
                "fullPlan": format!("{plan:?}"),
                "nodesUsed": plan.nodes_used,
                "simFaults": plan.sim_faults,
                "sourceArchiveImmutable": true,
                "restoredApplyValid": true,
                "appliedOutcome": format!("{outcome:?}"),
                "afterHash": hex(&restored.state_hash()),
                "afterArchiveHex": hex(&after),
                "afterEvents": restored.public_event_codes_from(log_start),
            }));
        }
        budgets.push(json!({"budget": budget, "policyVariant": 1, "lookaheadDepth": 2, "variants": variants}));
    }
    for (index, source) in pair.iter().enumerate() {
        assert_eq!(archive(source), originals[index], "all calls leave source archive immutable");
    }
    json!({
        "passed": true,
        "entrypoint": "Engine::plan_action",
        "cases": [{
            "case": "reachable_query_STORE_HOST_CHOICE",
            "prompt": "rvm_choice",
            "privateLocals": private_locals,
            "projectedPair": public,
            "sourceArchiveHex": originals.iter().map(|bytes| hex(bytes)).collect::<Vec<_>>(),
            "budgets": budgets,
        }],
        "caseCount": 1,
        "privateVariants": 2,
        "plannerCalls": 4,
        "restoredApplyChecks": 4,
        "claimBoundary": "public opaque rvm_choice leaf only; no generic Host or no-choice storage privacy claim",
    }).to_string()
}
