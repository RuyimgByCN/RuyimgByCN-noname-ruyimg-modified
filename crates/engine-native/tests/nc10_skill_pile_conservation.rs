//! NC10: redaction preserves public piles and the observer's private piles.
use engine_native::{ai::redact_opponent_hands, Engine};

fn pile<'a>(engine: &'a mut Engine, owner: usize, name: &str) -> &'a mut Vec<u32> {
    if name == "vm_aux" {
        return engine
            .state_core_mut()
            .vm_aux
            .piles
            .entry((owner as u8, "probe".into()))
            .or_default();
    }
    let player = &mut engine.state_core_mut().players[owner];
    match name {
        "zhucheng" => &mut player.zhucheng,
        "lianyaohu" => &mut player.lianyaohu_store,
        "sxianjing" => &mut player.sxianjing_traps,
        "hutian" => &mut player.hutian_hold,
        _ => panic!("unknown pile: {name}"),
    }
}

#[test]
fn redaction_preserves_owned_skill_cards_for_self_and_opponents() {
    let mut failures = Vec::new();
    for name in ["zhucheng", "lianyaohu", "sxianjing", "hutian", "vm_aux"] {
        for owner in [0, 1, 2] {
            if owner != 0 && matches!(name, "sxianjing" | "vm_aux") {
                continue;
            }
            let mut engine = Engine::new(3510 + owner as u64, 3);
            let card = engine.state_core_mut().deck.pop().unwrap();
            pile(&mut engine, owner, name).push(card);
            assert!(
                Engine::restore(&engine.save()).is_ok(),
                "invalid initial {name}/{owner}"
            );
            let original = engine.save();
            let mut sim = engine.clone();
            redact_opponent_hands(&mut sim, 0);
            assert_eq!(pile(&mut sim, owner, name), &[card]);
            if let Err(error) = Engine::restore(&sim.save()) {
                failures.push(format!("{name}/{owner}: {error:?}"));
            }
            assert_eq!(
                engine.save(),
                original,
                "simulation changed authoritative state"
            );
        }
    }
    assert!(failures.is_empty(), "invalid redacted states: {failures:?}");
}

#[test]
fn fixed_piles_keep_hidden_hand_permutations_equivalent() {
    use engine_native::ai::BoundedPlanner;

    for name in ["zhucheng", "lianyaohu", "sxianjing", "hutian", "vm_aux"] {
        let mut left = Engine::new(3520, 3);
        let card = left.state_core_mut().deck.pop().unwrap();
        let owner = if matches!(name, "sxianjing" | "vm_aux") {
            0
        } else {
            1
        };
        pile(&mut left, owner, name).push(card);
        let mut right = left.clone();
        let core = right.state_core_mut();
        std::mem::swap(&mut core.players[1].hand[0], &mut core.deck[0]);
        assert_eq!(
            format!("{:?}", left.ai_session(0).view()),
            format!("{:?}", right.ai_session(0).view())
        );
        let planner = BoundedPlanner::new(32).with_policy_variant(1);
        let left_plan = left.plan_action(0, &planner);
        let right_plan = right.plan_action(0, &planner);
        assert_eq!(left_plan.action, right_plan.action, "{name}");
        let mut left_sim = left.clone();
        let mut right_sim = right.clone();
        redact_opponent_hands(&mut left_sim, 0);
        redact_opponent_hands(&mut right_sim, 0);
        assert_eq!(left_sim.state_core(), right_sim.state_core(), "{name}");
        assert!(Engine::restore(&left_sim.save()).is_ok(), "{name}");
        assert!(Engine::restore(&right_sim.save()).is_ok(), "{name}");
    }
}

#[test]
fn opponent_private_pile_identity_does_not_change_synthetic_hands() {
    for name in ["sxianjing", "vm_aux"] {
        let mut a = Engine::new(3530, 3);
        let card = a.state_core_mut().deck.pop().unwrap();
        pile(&mut a, 1, name).push(card);
        let mut b = a.clone();
        let replacement = b.state_core().deck[0];
        let old = std::mem::replace(&mut pile(&mut b, 1, name)[0], replacement);
        b.state_core_mut().deck[0] = old;
        assert_eq!(a.ai_session(0).view(), b.ai_session(0).view());
        redact_opponent_hands(&mut a, 0);
        redact_opponent_hands(&mut b, 0);
        assert_eq!(a.state_core().deck, b.state_core().deck, "{name}");
        for owner in [1, 2] {
            assert_eq!(
                a.state_core().players[owner].hand,
                b.state_core().players[owner].hand,
                "{name}/{owner}"
            );
        }
    }
}
