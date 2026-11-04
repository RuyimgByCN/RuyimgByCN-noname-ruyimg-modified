//! NC10 helper contracts: VM sentinels and another actor's filtered choices.
use engine_native::ai::{redact_opponent_hands, BoundedPlanner};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook, VmSkillDecl};
use engine_native::{
    CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};
use rule_vm::bytecode::{caps, effect_tag, op};

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    out
}

fn collect_unknown_cards(e: &mut Engine) {
    let core = e.state_core_mut();
    let mut unknown = std::mem::take(&mut core.deck);
    for player in core.players.iter_mut().skip(1) {
        unknown.append(&mut player.hand);
    }
    core.players[1].hand = unknown;
}

// The closed copy only observes the helper's deterministic pool, not the rule continuation.
fn idle_sample(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let mut snapshot = e.save();
    snapshot.resolution.stack.clear();
    snapshot.resolution.parks.clear();
    snapshot.resolution.open_choice = None;
    snapshot.resolution.answer = None;
    let mut sample = Engine::restore_with_extensions(&snapshot, packs).unwrap();
    redact_opponent_hands(&mut sample, 0);
    sample
}

fn constant(code: &mut Vec<u8>, value: u32) {
    code.push(op::CONST_U32);
    code.extend(value.to_le_bytes());
}

fn effect(code: &mut Vec<u8>, tag: u16) {
    code.push(op::HOST_EFFECT_EXT);
    code.extend(tag.to_le_bytes());
}

fn pending_pack(card: u32, move_card: bool) -> ExtensionPack {
    let mut code = Vec::new();
    constant(&mut code, 1);
    constant(&mut code, 1);
    effect(&mut code, effect_tag::DISCARD_HAND_EQUIP_COUNT);
    if move_card {
        constant(&mut code, card);
        constant(&mut code, (1 << 8) | 3); // native seat 0 hand
        constant(&mut code, (2 << 8) | 3); // native seat 1 hand
        effect(&mut code, effect_tag::MOVE_CARD);
    } else {
        constant(&mut code, 1);
        constant(&mut code, card);
        constant(&mut code, 0);
        effect(&mut code, effect_tag::STORE_EVENT_CARD_IN_SKILL_PILE);
    }
    code.push(op::HALT);
    ExtensionPack {
        def: ExtensionDef {
            name: "nc10-pending-card",
            version: 1,
            description: "VM sentinel regression",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        init_hook: InitHook::None,
        enabled: true,
        vm_skills: vec![VmSkillDecl {
            r05_identity: None,
            declaration_hash: None,
            name: "nc10_pending_card".into(),
            module: rule_vm::BytecodeModule {
                name: "nc10-pending-card".into(),
                code,
                local_count: 0,
                max_stack: 8,
                entry_pc: 0,
                logic_hash: [73; 32],
                caps_mask: caps::EMIT_EFFECTS,
            },
            trigger: engine_native::catalog::TriggerMeta {
                event: engine_native::EventKind::PhaseEnd,
                timing: engine_native::Timing::Normal,
                priority: 0,
                order_selectable: true,
                optional: false,
                extra_events: vec![],
            },
            filter: engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget,
        }],
    }
}

fn pending_engine(seed: u64, pack: &ExtensionPack, swap: Option<(usize, usize)>) -> Engine {
    let mut e = Engine::new_with_extensions(seed, 4, &[pack]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, &pack.vm_skills[0].name);
    collect_unknown_cards(&mut e);
    if let Some((a, b)) = swap {
        e.state_core_mut().players[1].hand.swap(a, b);
    }
    submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_discard_pick");
    Engine::restore_with_extensions(&e.save(), &[pack]).expect("real accepted commit pause");
    e
}

fn pending_card_survives_redaction(move_card: bool) {
    let probe_pack = pending_pack(1, move_card);
    let seed = (991200..991216)
        .find(|seed| {
            !Engine::new_with_extensions(*seed, 4, &[&probe_pack])
                .unwrap()
                .state_core()
                .players[0]
                .hand
                .contains(&0)
        })
        .unwrap();
    let probe = pending_engine(seed, &probe_pack, None);
    let sample = idle_sample(&probe, &[&probe_pack]);
    let zero_slot = sample.state_core().players[1]
        .hand
        .iter()
        .position(|c| *c == 0)
        .unwrap();
    let hand = &probe.state_core().players[1].hand;
    let swap =
        (hand[zero_slot] == 0).then(|| (zero_slot, hand.iter().position(|c| *c != 0).unwrap()));
    let card = swap.map(|(_, b)| hand[b]).unwrap_or(hand[zero_slot]);
    assert_ne!(card, 0);
    let pack = pending_pack(card, move_card);
    let original = pending_engine(seed, &pack, swap);
    assert_eq!(original.state_core().players[1].hand[zero_slot], card);
    assert_eq!(
        idle_sample(&original, &[&pack]).state_core().players[1].hand[zero_slot],
        0,
        "the same unrestricted physical pool would assign nonzero card to native zero"
    );
    let snapshot = original.save();
    let machine = &snapshot.resolution.parks.iter().find(|p| matches!(p.machine, engine_native::MachineSnapshot::Rvm { .. })).expect("VM owner machine").machine;
    let engine_native::MachineSnapshot::Rvm { commit_pending, .. } = machine else {
        panic!("Rvm commit pause")
    };
    let effects = engine_native::rvm_ops::dec_effects(commit_pending).unwrap();
    for effect in &effects {
        engine_native::rvm_ops::accept_effect(effect, 4).unwrap();
    }
    assert!(effects.iter().any(|effect| match effect {
        rule_api::EffectRequest::StoreEventCardInSkillPile { card: c, .. } => c.raw() == card,
        rule_api::EffectRequest::MoveCards { cards, .. } => cards.iter().any(|c| c.raw() == card),
        _ => false,
    }));
    let mut redacted = original.clone();
    redact_opponent_hands(&mut redacted, 0);
    let sampled_card = redacted.state_core().players[1].hand[zero_slot];
    let snapshot = redacted.save();
    let mut restored = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snapshot))
            .unwrap(),
        &[&pack],
    )
    .unwrap();
    for budget in [0, 64] {
        let planner = BoundedPlanner::new(budget)
            .with_policy_variant(1)
            .with_lookahead(2);
        assert_eq!(
            format!("{:?}", original.ai_session(0).input()),
            format!("{:?}", redacted.ai_session(0).input())
        );
        let plan = original.plan_action(0, &planner);
        assert_eq!(
            format!("{plan:?}"),
            format!("{:?}", redacted.plan_action(0, &planner))
        );
        assert_eq!((plan.nodes_used, plan.sim_faults), (0, 0));
    }
    let choice = restored.open_choice().unwrap().clone();
    submit(
        &mut restored,
        choice.observer,
        GameAction::Answer {
            choice_id: choice.id,
            selection: choice
                .spec
                .options
                .iter()
                .take(choice.spec.min)
                .copied()
                .collect(),
        },
    );
    let moved = if move_card {
        restored.state_core().players[0]
            .hand
            .contains(&sampled_card)
    } else {
        restored
            .state_core()
            .vm_aux
            .piles
            .get(&(0, pack.vm_skills[0].name.clone()))
            .is_some_and(|pile| pile.contains(&sampled_card))
    };
    eprintln!("VM_SENTINEL move={move_card} source={card} sampled={sampled_card} moved={moved}");
    assert!(
        moved,
        "accepted nonzero pending card effect must survive helper sampling"
    );
    assert_eq!(
        redacted.save().resolution,
        original.save().resolution,
        "conservative commit helper preserves the actual pending payload"
    );
    Engine::restore_with_extensions(&restored.save(), &[&pack]).unwrap();
}

#[test]
fn pending_store_preserves_vm_nonzero_card_contract() {
    pending_card_survives_redaction(false);
}

#[test]
fn pending_move_preserves_vm_nonzero_card_contract() {
    pending_card_survives_redaction(true);
}

fn sha_setup() -> (Engine, u32) {
    let mut e = Engine::new(991300, 4);
    e.debug_set_turn(0, Phase::Play, 1);
    let sha = e.debug_move_named_card_to_hand(0, "sha");
    collect_unknown_cards(&mut e);
    (e, sha)
}

#[test]
fn nonobserver_shan_window_retains_restorable_filtered_options() {
    let (setup, sha) = sha_setup();
    let mut probe = setup.clone();
    submit(
        &mut probe,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert_eq!(probe.open_choice().unwrap().observer, 1);
    let sample = idle_sample(&probe, &[]);
    let slot = sample.state_core().players[1]
        .hand
        .iter()
        .position(|card| sample.catalog().card_kind(*card) != CardKind::Shan)
        .unwrap();
    let shan = setup.state_core().players[1]
        .hand
        .iter()
        .position(|card| setup.catalog().card_kind(*card) == CardKind::Shan)
        .unwrap();
    let mut e = setup;
    e.state_core_mut().players[1].hand.swap(slot, shan);
    let original_shan = e.state_core().players[1].hand[slot];
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    Engine::restore(&e.save()).expect("real accepted Sha response before redaction");
    let before = e.save();
    let input = format!("{:?}", e.ai_session(0).input());
    let events = e.ai_session(0).events();
    redact_opponent_hands(&mut e, 0);
    eprintln!(
        "NONOBSERVER_SHAN original={original_shan} sampled={} kind={:?}",
        e.state_core().players[1].hand[slot],
        e.catalog().card_kind(e.state_core().players[1].hand[slot])
    );
    assert_eq!(format!("{:?}", e.ai_session(0).input()), input);
    assert_eq!(e.ai_session(0).events(), events);
    let mut restored = Engine::restore(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap(),
    )
    .expect("non-observer helper preserves sampled choice restoration");
    assert_eq!(e.save().resolution, before.resolution);
    assert_eq!(e.state_core().players[1].hand, before.core.players[1].hand);
    let choice = restored.open_choice().unwrap().clone();
    assert!(choice.spec.options.contains(&ChoiceOption::Pass));
    submit(
        &mut restored,
        1,
        GameAction::Answer {
            choice_id: choice.id,
            selection: vec![ChoiceOption::Pass],
        },
    );
}
