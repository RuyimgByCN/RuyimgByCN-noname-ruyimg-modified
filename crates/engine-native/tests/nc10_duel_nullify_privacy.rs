//! NC10: a duel's nullify window must not reveal other seats' hidden cards.
use engine_native::ai::BoundedPlanner;
use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot,
};

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action,
    })
}

fn fixture() -> Engine {
    let mut e = Engine::new(46, 4);
    for _ in 0..2 {
        assert!(!matches!(
            submit(&mut e, 0, GameAction::NextPhase),
            CommandOutcome::Rejected(_)
        ));
    }
    assert!(e.state_core().players[0].hand.contains(&58));
    assert_eq!(e.catalog().card_kind(58), CardKind::Duel);
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: 58,
                targets: vec![1]
            }
        ),
        CommandOutcome::Pending { .. }
    ));
    let choice = e.open_choice().unwrap();
    assert_eq!(choice.observer, 2);
    assert_eq!(choice.spec.prompt, "nullify_window");
    assert_eq!(
        choice.spec.options,
        vec![ChoiceOption::Pass, ChoiceOption::Card(71)]
    );
    assert!(matches!(e.resolution().parks[0].machine.snapshot(),
        MachineSnapshot::NullifyWindow {
            card: 58, source: 0, nullified: false, asked: true, window_open: true,
            effect: Some(inner), ..
        } if matches!(*inner, MachineSnapshot::Duel {
            card: 58, source: 0, target: 1, on_target: true, asked: false,
            second_pending: false,
        })
    ));
    e
}

fn alternate_hidden(mut e: Engine) -> Engine {
    assert_eq!(e.state_core().players[0].hand[0], 75);
    assert_eq!(e.state_core().deck[3], 88);
    assert_eq!(e.catalog().card_kind(75), CardKind::Le);
    assert_eq!(e.catalog().card_kind(88), CardKind::LeiSha);
    e.state_core_mut().players[0].hand[0] = 88;
    e.state_core_mut().deck[3] = 75;
    e
}

#[test]
fn planning_uses_only_observers_information() {
    let e = fixture();
    let alt = alternate_hidden(e.clone());
    let a = e.ai_session(2).input();
    let b = alt.ai_session(2).input();
    assert_eq!(a.view, b.view);
    assert_eq!(a.choice, b.choice);
    assert_eq!(a.own_hand_kinds, b.own_hand_kinds);
    assert_eq!(e.resolution().stack, alt.resolution().stack);
    assert_eq!(
        e.resolution().parks[0].machine.snapshot(),
        alt.resolution().parks[0].machine.snapshot()
    );
    for variant in 0..=2 {
        for budget in [0, 1, 48] {
            let planner = BoundedPlanner::new(budget)
                .with_policy_variant(variant)
                .with_lookahead(2);
            let p = e.plan_action(2, &planner);
            let q = alt.plan_action(2, &planner);
            assert_eq!(
                p.action, q.action,
                "variant={variant} budget={budget}: {p:?} vs {q:?}"
            );
            assert_eq!(p.nodes_used, q.nodes_used);
            assert_eq!(p.sim_faults, 0);
            assert_eq!(q.sim_faults, 0);
            for original in [&e, &alt] {
                let mut copy =
                    Engine::restore(&original.save()).expect("restore original pending window");
                assert!(!matches!(
                    copy.apply_plan(&p).unwrap(),
                    CommandOutcome::Rejected(_)
                ));
            }
        }
    }
}

#[test]
fn redaction_preserves_window_and_produces_restorable_equal_hidden_zones() {
    let e = fixture();
    let mut left = e.clone();
    let mut right = alternate_hidden(e.clone());
    engine_native::ai::redact_opponent_hands(&mut left, 2);
    engine_native::ai::redact_opponent_hands(&mut right, 2);
    assert_eq!(left.resolution().stack, e.resolution().stack);
    assert_eq!(
        left.resolution().parks[0].machine.snapshot(),
        e.resolution().parks[0].machine.snapshot()
    );
    assert_eq!(left.open_choice(), e.open_choice());
    assert_eq!(left.state_core().table, e.state_core().table);
    assert_eq!(
        left.state_core().players[2].hand,
        e.state_core().players[2].hand
    );
    for seat in [0, 1, 3] {
        assert_eq!(
            left.state_core().players[seat].hand,
            right.state_core().players[seat].hand
        );
    }
    assert_eq!(left.state_core().deck, right.state_core().deck);
    let core = left.state_core();
    let mut cards = core.deck.clone();
    cards.extend(&core.discard);
    cards.extend(&core.table);
    for player in &core.players {
        cards.extend(&player.hand);
        cards.extend(&player.equips);
        cards.extend(&player.judge);
        cards.extend(&player.off_court);
    }
    cards.sort_unstable();
    assert!(
        cards.windows(2).all(|pair| pair[0] != pair[1]),
        "redaction must not duplicate physical cards"
    );
    let restored = Engine::restore(&left.save()).expect("redacted pending window must restore");
    assert_eq!(restored.open_choice(), left.open_choice());
    assert_eq!(
        restored.resolution().parks[0].machine.snapshot(),
        left.resolution().parks[0].machine.snapshot()
    );
}

#[test]
fn both_answers_can_resume_redacted_window_after_restore() {
    let mut e = fixture();
    engine_native::ai::redact_opponent_hands(&mut e, 2);
    for option in [ChoiceOption::Pass, ChoiceOption::Card(71)] {
        let mut restored = Engine::restore(&e.save()).expect("pending window must restore");
        let choice_id = restored.open_choice().unwrap().id;
        let outcome = submit(
            &mut restored,
            2,
            GameAction::Answer {
                choice_id,
                selection: vec![option],
            },
        );
        assert!(
            !matches!(outcome, CommandOutcome::Rejected(_)),
            "{option:?}: {outcome:?}"
        );
        Engine::restore(&restored.save()).expect("resulting state must restore");
    }
}

#[test]
fn special_zones_remain_conservative() {
    let mut e = fixture();
    let card = e.state_core_mut().deck.pop().unwrap();
    e.state_core_mut().players[1].zhucheng.push(card);
    let before = e.state_core().players[0].hand.clone();
    let machine = e.resolution().parks[0].machine.snapshot();
    engine_native::ai::redact_opponent_hands(&mut e, 2);
    assert_eq!(e.state_core().players[0].hand, before);
    assert_eq!(e.state_core().players[1].zhucheng, vec![card]);
    assert_eq!(e.resolution().parks[0].machine.snapshot(), machine);
}

#[test]
#[ignore = "manual same-machine latency sampling"]
fn nullify_timing() {
    let e = fixture();
    let planner = BoundedPlanner::new(48)
        .with_policy_variant(1)
        .with_lookahead(2);
    for _ in 0..32 {
        std::hint::black_box(e.plan_action(2, &planner));
    }
    let mut samples = Vec::new();
    for _ in 0..500 {
        let t = std::time::Instant::now();
        std::hint::black_box(e.plan_action(2, &planner));
        samples.push(t.elapsed().as_micros());
    }
    samples.sort_unstable();
    eprintln!(
        "samples={} p50={}us p95={}us p99={}us",
        samples.len(),
        samples[249],
        samples[474],
        samples[494]
    );
}
