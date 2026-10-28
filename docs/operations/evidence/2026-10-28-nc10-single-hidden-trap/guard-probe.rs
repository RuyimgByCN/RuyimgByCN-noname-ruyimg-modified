use engine_native::{ai::BoundedPlanner, catalog::CardKind, event::Phase, Engine};

#[test]
fn synthetic_guard_noninterference() {
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let json = std::fs::read_to_string(root.join("../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json")).unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    for seed in 3900..4100 {
        let mut a = Engine::new_with_extensions(seed, 4, &[&pack]).unwrap();
        a.debug_set_turn(0, Phase::Play, 1);
        a.debug_grant(1, "sxianjing_gain");
        let Some(sha) = a.state_core().player(0).hand.iter().copied().find(|id| a.catalog().card_kind(*id) == CardKind::Sha) else { continue; };
        let suit = a.catalog().cards[sha as usize].suit;
        let Some(first) = a.state_core().deck.iter().position(|id| a.catalog().cards[*id as usize].suit == suit) else { continue; };
        let card = a.state_core_mut().deck.remove(first);
        a.state_core_mut().players[1].sxianjing_traps.push(card);
        let planner = BoundedPlanner::new(48).with_policy_variant(1).with_lookahead(2);
        let base = a.plan_action(0, &planner).action;
        for variant in ["flag", "storage", "lost", "history", "declared", "nature"] {
            let mut b = a.clone();
            match variant {
                "flag" => { b.state_core_mut().vm_aux.flags.insert((1, "nc10_probe_inert".into()), 1); },
                "storage" => { b.state_core_mut().vm_aux.storage.insert("nc10_probe_inert".into(), 1); },
                "lost" => { b.state_core_mut().vm_aux.lost.insert(1, vec![card]); },
                "history" => { b.state_core_mut().turn.huanxia_track.push((card, false)); },
                "declared" => { b.state_core_mut().declared_kinds.insert(card, CardKind::Sha); },
                "nature" => { b.state_core_mut().card_natures.insert(card, 1); },
                _ => unreachable!(),
            }
            assert_eq!(a.ai_session(0).view(), b.ai_session(0).view());
            assert!(Engine::restore_with_extensions(&b.save(), &[&pack]).is_ok());
            let other = b.plan_action(0, &planner).action;
            eprintln!("CASE {seed} {variant} equal={} base={base:?} other={other:?}", base == other);
        }
    }
}
