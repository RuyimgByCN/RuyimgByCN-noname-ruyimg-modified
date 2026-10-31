//! Public Own-choice advice must not execute saved private continuations.
use engine_native::ai::BoundedPlanner;
use engine_native::{ChoiceOption, ChoiceView, CommandOutcome, Engine, GameAction, Phase};

fn restore(snapshot: &engine_native::StateSnapshot) -> Engine {
    Engine::restore(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(snapshot))
            .unwrap(),
    )
    .unwrap()
}

fn yiji_pair() -> [Engine; 2] {
    let mut engine = Engine::new(2306, 3);
    engine.debug_set_turn(0, Phase::Play, 1);
    for seat in 0..3 {
        engine.debug_clear_hand(seat);
    }
    engine.debug_grant(1, "遗计");
    engine.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let snapshot = engine.save();
    let mut later = snapshot.clone();
    later.resolution.parks.push(engine_native::ParkSnapshot {
        resume_depth: snapshot.resolution.stack.len(),
        slot: engine_native::resolution::Slot::Trigger {
            frame: engine_native::resolution::NO_FRAME,
        },
        machine: engine_native::MachineSnapshot::JiJie {
            owner: 0,
            stage: engine_native::snap::JiJieStageSnap::Ask,
            card: None,
        },
    });
    [restore(&snapshot), restore(&later)]
}

#[test]
fn complete_own_plan_cannot_depend_on_an_unseen_later_park() {
    let pair = yiji_pair();
    let inputs = pair
        .each_ref()
        .map(|engine| format!("{:?}", engine.ai_session(1).input()));
    let events = pair
        .each_ref()
        .map(|engine| format!("{:?}", engine.event_view_for(1)));
    assert_eq!(inputs[0], inputs[1]);
    assert_eq!(events[0], events[1]);
    assert!(matches!(
        pair[0].ai_session(1).choice(),
        Some(ChoiceView::Own {
            source_skill: Some("遗计"),
            ..
        })
    ));
    let originals = pair.each_ref().map(|engine| engine.save());
    let mut samples = Vec::new();
    for budget in [48, 64, 0, 1, 2, 4, 8] {
        let planner = BoundedPlanner::new(budget)
            .with_lookahead(2)
            .with_policy_variant(1);
        let plans = pair
            .each_ref()
            .map(|engine| engine.plan_action(1, &planner));
        let full = plans.each_ref().map(|plan| format!("{plan:?}"));
        eprintln!(
            "OWN_PARK budget={budget} inputs={inputs:?} pre_events={events:?} plans={full:?}"
        );
        if let Ok(directory) = std::env::var("NC10_OWN_EVIDENCE") {
            let path = std::path::Path::new(&directory);
            std::fs::create_dir_all(path).unwrap();
            std::fs::write(
                path.join(format!("plans-{budget}.txt")),
                format!("{full:#?}\ninputs={inputs:#?}\nevents={events:#?}\n"),
            )
            .unwrap();
            for (side, snapshot) in originals.iter().enumerate() {
                std::fs::write(
                    path.join(format!("park-{side}.ngsa")),
                    engine_native::archive::encode_archive(snapshot),
                )
                .unwrap();
            }
        }
        samples.push((budget, full, plans));
    }
    for (budget, full, plans) in samples {
        assert_eq!(
            full[0], full[1],
            "complete Plan must depend only on public input"
        );
        for (side, (engine, plan)) in pair.iter().zip(&plans).enumerate() {
            assert_eq!((plan.nodes_used, plan.sim_faults), (0, 0));
            assert!(!plan.budget_exhausted && !plan.timed_out && !plan.cancelled);
            assert!(
                matches!(&plan.action, GameAction::Answer { selection, .. } if selection == &[ChoiceOption::Confirm])
            );
            let mut restored = restore(&engine.save());
            let before_revision = restored.revision();
            let outcome = restored.apply_plan(plan).unwrap();
            assert!(
                matches!(outcome, CommandOutcome::Pending { .. }),
                "{outcome:?}"
            );
            assert_eq!(restored.revision(), before_revision + 1);
            Engine::restore(&restored.save()).unwrap();
            eprintln!("OWN_APPLY budget={budget} side={side} outcome={outcome:?}");
            if let Ok(directory) = std::env::var("NC10_OWN_EVIDENCE") {
                let path = std::path::Path::new(&directory);
                std::fs::write(
                    path.join(format!("park-{side}-after-{budget}.ngsa")),
                    engine_native::archive::encode_archive(&restored.save()),
                )
                .unwrap();
                std::fs::write(
                    path.join(format!("park-{side}-outcome-{budget}.txt")),
                    format!("{outcome:?}\n"),
                )
                .unwrap();
            }
        }
    }
    assert_eq!(pair.each_ref().map(|engine| engine.save()), originals);
}

#[test]
fn own_advice_cancelled_before_apply_does_not_submit() {
    let [mut engine, _] = yiji_pair();
    let before = engine.save();
    let planner = BoundedPlanner::new(64).with_deadline_ms(0);
    planner.cancel();
    let plan = engine.plan_action(1, &planner);
    assert!(plan.cancelled);
    assert_eq!((plan.nodes_used, plan.sim_faults), (0, 0));
    assert!(matches!(
        engine.apply_plan(&plan),
        Err(engine_native::ai::ApplyPlanError::Cancelled)
    ));
    assert_eq!(engine.save(), before);
}

