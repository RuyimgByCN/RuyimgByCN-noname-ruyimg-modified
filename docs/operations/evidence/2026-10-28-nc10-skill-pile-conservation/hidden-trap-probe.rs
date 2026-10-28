//! Diagnostic: same visible state, different opponent trap identity.
use engine_native::{
    ai::{redact_opponent_hands, BoundedPlanner},
    catalog::CardKind,
    event::Phase,
    Engine,
};

#[test]
fn same_view_hidden_trap_probe() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(
        root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    let mut checked = 0;
    let mut differences = 0;
    let mut redacted_pool_differences = 0;
    for seed in 3900..4100 {
        let mut a = Engine::new_with_extensions(seed, 4, &[&pack]).unwrap();
        a.debug_set_turn(0, Phase::Play, 1);
        a.debug_grant(1, "sxianjing_gain");
        let Some(sha) = a
            .state_core()
            .player(0)
            .hand
            .iter()
            .copied()
            .find(|id| a.catalog().card_kind(*id) == CardKind::Sha)
        else {
            continue;
        };
        let suit = a.catalog().cards[sha as usize].suit;
        let Some(first) = a
            .state_core()
            .deck
            .iter()
            .position(|id| a.catalog().cards[*id as usize].suit == suit)
        else {
            continue;
        };
        let card = a.state_core_mut().deck.remove(first);
        a.state_core_mut().players[1].sxianjing_traps.push(card);
        let Some(second) = a
            .state_core()
            .deck
            .iter()
            .position(|id| a.catalog().cards[*id as usize].suit != suit)
        else {
            continue;
        };
        let mut b = a.clone();
        let core = b.state_core_mut();
        std::mem::swap(
            &mut core.players[1].sxianjing_traps[0],
            &mut core.deck[second],
        );
        assert_eq!(a.ai_session(0).view(), b.ai_session(0).view());
        assert!(Engine::restore_with_extensions(&a.save(), &[&pack]).is_ok());
        assert!(Engine::restore_with_extensions(&b.save(), &[&pack]).is_ok());
        let mut x = a.clone();
        let mut y = b.clone();
        redact_opponent_hands(&mut x, 0);
        redact_opponent_hands(&mut y, 0);
        if x.state_core().deck != y.state_core().deck
            || x.state_core().players[1].hand != y.state_core().players[1].hand
        {
            redacted_pool_differences += 1;
        }
        let planner = BoundedPlanner::new(48)
            .with_policy_variant(1)
            .with_lookahead(2);
        let pa = a.plan_action(0, &planner);
        let pb = b.plan_action(0, &planner);
        eprintln!("CASE seed={seed} a={:?} b={:?}", pa.action, pb.action);
        if pa.action != pb.action {
            differences += 1;
            if differences <= 3 {
                eprintln!(
                    "seed={seed} sha={sha} traps={card}/{} action_a={:?} action_b={:?}",
                    b.state_core().players[1].sxianjing_traps[0],
                    pa.action,
                    pb.action
                );
            }
        }
        checked += 1;
    }
    eprintln!("checked={checked} different_actions={differences} different_synthetic_pools={redacted_pool_differences}");
    assert!(checked > 0);
}
