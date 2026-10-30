//! Independent-review counterexamples for ADR0786 J2. All skills enter via the real DSL bridge.
mod nc08_common;
use engine_native::content::extension::{ExtensionDef, ExtensionPack};
use engine_native::{ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand};
fn pack(source: &str) -> ExtensionPack {
    let p = content_compiler::compile_source_named("j2-review", source).unwrap();
    let decls = content_native_loader::bridge_vm_skill_decls(&p)
        .unwrap()
        .decls;
    let known = decls.iter().map(|d| d.name.clone()).collect();
    let roles = [("author", false), ("watcher", true)];
    let generals=roles.iter().map(|(name,watcher)|serde_json::json!({"name":name,"hp":4,"faction":"qun","gender":"male","skills":decls.iter().filter(|d|d.name.starts_with("j2_watcher")==*watcher).map(|d|serde_json::json!({"name":d.name})).collect::<Vec<_>>()})).collect::<Vec<_>>();
    let generals = content_native_loader::bridge_generals_with_known(
        &serde_json::json!({"pack":{"name":"j2-review","version":1},"generals":generals})
            .to_string(),
        &known,
    )
    .unwrap()
    .generals;
    ExtensionPack {
        def: ExtensionDef {
            name: "j2-review",
            version: 1,
            description: "review",
        },
        generals,
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: Default::default(),
        enabled: true,
    }
}
fn start(pack: &ExtensionPack) -> Engine {
    let mut e = Engine::new_with_extensions(99107868, 4, &[pack]).unwrap();
    e.assign_general_named(0, "author").unwrap();
    e
}
fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: nc08_common::next_dedup(),
        action,
    })
}
fn answer(e: &mut Engine, pack: &ExtensionPack, selected: Vec<O>) {
    let c = e.open_choice().cloned().unwrap();
    let snap = e.save();
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snap))
            .unwrap();
    let mut other = Engine::restore_with_extensions(&decoded, &[pack]).unwrap();
    let dedup = nc08_common::next_dedup();
    let cursor = e.log().len();
    for g in [&mut *e, &mut other] {
        let before = g.save();
        let log = g.log().to_vec();
        let bad = g.submit(GameCommand {
            actor: c.observer,
            expected_revision: g.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![O::Tag(255)],
            },
        });
        assert!(matches!(bad, CommandOutcome::Rejected(_)));
        assert_eq!(
            g.save(),
            before,
            "failed retry preserves complete archive and dedup key"
        );
        assert_eq!(g.log(), log);
        let out = g.submit(GameCommand {
            actor: c.observer,
            expected_revision: g.revision(),
            dedup_id: dedup,
            action: GameAction::Answer {
                choice_id: c.id,
                selection: selected.clone(),
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    }
    assert_eq!(e.save().core, other.save().core);
    assert_eq!(e.save().resolution, other.save().resolution);
    assert_eq!(&e.log()[cursor..], other.log());
}
fn activate(e: &mut Engine, name: &str) -> CommandOutcome {
    let skill = e.catalog().skill_by_name(name).unwrap();
    submit(e, 0, GameAction::ActivateSkill { skill })
}

#[test]
fn fenghe_forged_deck_reference_rejects_byte_restore_without_double_ownership() {
    let p = pack(include_str!(
        "../../../content/original/ported/shiji/pinghe.skill"
    ));
    let mut e = start(&p);
    nc08_common::advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let hand = e.debug_place(0, "shan");
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    if e.open_choice().unwrap().spec.prompt == "respond_shan" {
        answer(&mut e, &p, vec![O::Pass]);
    }
    answer(&mut e, &p, vec![O::Card(hand)]);
    assert_eq!(
        e.open_choice().unwrap().spec.prompt,
        "rvm_fenghe_destination"
    );
    let valid = e.save();
    assert!(
        Engine::restore_with_extensions(&valid, &[&p]).is_ok(),
        "selected hand card is a reference, not a second physical owner"
    );
    let deck = e
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|c| e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Chain)
        .unwrap();
    let mut forged = valid.clone();
    let mut changed = false;
    for park in &mut forged.resolution.parks {
        if let engine_native::MachineSnapshot::Rvm { commit_state, .. } = &mut park.machine {
            if commit_state.first() == Some(&2) {
                commit_state[1] = deck;
                changed = true;
            }
        }
    }
    assert!(changed);
    let bytes = engine_native::archive::encode_archive(&forged);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert!(
        Engine::restore_with_extensions(&decoded, &[&p]).is_err(),
        "an in-range card outside the owner's hand must not restore"
    );
    answer(&mut e, &p, vec![O::Player(2)]);
    assert!(e.state_core().player(2).hand.contains(&hand));
}

fn draw_order_source(kill: bool) -> String {
    let watcher = if kill {
        "effect kill_player(self,event_player);"
    } else {
        "effect discard_hand_equip_count(self,1);"
    };
    format!("{}\nskill j2_watcher(99107868) locked {{ trigger cards_drawn owner_alive {{ {watcher} }} }}",include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill"))
}
#[test]
fn danshou_owner_draw_subtree_finishes_before_target_draw_and_ordinal_discard() {
    let p = pack(&draw_order_source(false));
    let mut e = start(&p);
    e.assign_general_named(1, "watcher").unwrap();
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for _ in 0..8 {
        e.debug_place(0, "shan");
    }
    e.debug_clear_hand(1);
    for _ in 0..3 {
        e.debug_place(1, "shan");
    }
    activate(&mut e, "ol_jie_danshou");
    answer(&mut e, &p, vec![O::Tag(4)]);
    answer(&mut e, &p, vec![O::Player(1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_discard_pick");
    assert_eq!(e.state_core().player(0).hand.len(), 10);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        3,
        "target has not drawn while owner's nested prompt is open"
    );
    let card = e.open_choice().unwrap().spec.options[0];
    answer(&mut e, &p, vec![card]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_discard_pick");
    assert_eq!(
        e.state_core().player(1).hand.len(),
        4,
        "target draws only after owner subtree settled"
    );
    let card = e.open_choice().unwrap().spec.options[0];
    answer(&mut e, &p, vec![card]);
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "rvm_danshou_discard");
    let picked = c.spec.options.iter().take(4).copied().collect();
    answer(&mut e, &p, picked);
    assert_eq!(e.state_core().player(0).hand.len(), 6);
    assert_eq!(e.state_core().player(1).hand.len(), 3);
}
#[test]
fn danshou_owner_draw_can_kill_target_before_its_draw_is_started() {
    let p = pack(&draw_order_source(true));
    let mut e = start(&p);
    nc08_common::advance_to_play(&mut e, 0);
    e.assign_general_named(1, "watcher").unwrap();
    e.debug_clear_hand(0);
    for _ in 0..8 {
        e.debug_place(0, "shan");
    }
    let cursor = e.log().len();
    activate(&mut e, "ol_jie_danshou");
    answer(&mut e, &p, vec![O::Tag(4)]);
    answer(&mut e, &p, vec![O::Player(1)]);
    assert!(!e.state_core().player(1).alive);
    assert!(
        !e.log()[cursor..]
            .iter()
            .any(|event| matches!(event, engine_native::LogEvent::CardsDrawn { player: 1, .. })),
        "dead target must never receive the prematurely queued draw"
    );
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "rvm_danshou_discard");
    let picked = c.spec.options.iter().take(4).copied().collect();
    answer(&mut e, &p, picked);
}

#[test]
fn grouped_following_handler_rechecks_owner_and_instance_after_first_handler() {
    for first in [
        "effect kill_player(self,self);",
        "effect remove_skill(self_skill);",
    ] {
        let source=format!("skill j2_grouped(99107869) {{ trigger phase_play owner_is_target {{ {first} }} trigger phase_play owner_is_target {{ effect draw_cards(self,1); }} active play_phase targets 0 0 {{ effect draw_cards(self,1); }} }}");
        let p = pack(&source);
        let mut e = start(&p);
        let cursor = e.log().len();
        nc08_common::advance_to_play(&mut e, 0);
        assert!(!e.log()[cursor..].iter().any(|event|matches!(event,engine_native::LogEvent::CardsDrawn {player:0,cards} if cards.len()==1)),"following handler must not bypass the ordinary owner/instance gate");
    }
}

#[test]
fn trigger_to_active_storage_reads_share_logical_identity_and_are_owner_scoped() {
    let source="skill j2_private(99107870) { trigger phase_play owner_is_target { effect storage_set(self_skill,1,7); } active play_phase targets 0 0 { if storage_has(self_skill,1) { if storage_get(self_skill,1)==7 { effect draw_cards(self,1); } } } }";
    let p = pack(source);
    let mut e = start(&p);
    e.assign_general_named(1, "author").unwrap();
    nc08_common::advance_to_play(&mut e, 0);
    let hand = e.state_core().player(0).hand.len();
    activate(&mut e, "j2_private");
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand + 1,
        "active DSL must read the trigger's exact private value"
    );
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    let mut restored = Engine::restore_with_extensions(&decoded, &[&p]).unwrap();
    let hand = restored.state_core().player(0).hand.len();
    activate(&mut restored, "j2_private");
    assert_eq!(restored.state_core().player(0).hand.len(), hand + 1);
    // Seat1 has not entered Play; changing the current seat is a fixture for an attempted private read.
    e.state_core_mut().vm_aux.marks.insert((0, 1), 7);
    e.state_core_mut().turn.current = 1;
    let hand = e.state_core().player(1).hand.len();
    let skill = e.catalog().skill_by_name("j2_private").unwrap();
    submit(&mut e, 1, GameAction::ActivateSkill { skill });
    assert_eq!(
        e.state_core().player(1).hand.len(),
        hand,
        "another owner cannot read seat0's storage or a same-key mark"
    );
}
#[test]
fn active_body_with_preceding_draw_or_unreachable_danshou_is_not_pre_rejected() {
    for body in [
        "effect draw_cards(self,1); effect resolve_ol_jie_danshou(self);",
        "if 0==1 { effect resolve_ol_jie_danshou(self); } else { effect draw_cards(self,1); }",
    ] {
        let p = pack(&format!(
            "skill j2_prefix(99107871) {{ active play_phase targets 0 0 {{ {body} }} }}"
        ));
        let mut e = start(&p);
        nc08_common::advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let out = activate(&mut e, "j2_prefix");
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "effect-tag occurrence is not an activation declaration: {out:?}"
        );
        assert_eq!(e.state_core().player(0).hand.len(), 1);
        if e.open_choice().is_some() {
            answer(&mut e, &p, vec![]);
        }
    }
}

#[test]
fn grouped_handler_checkpoint_before_following_preserves_death_and_disabled_gate() {
    for disable in [false, true] {
        let first = if disable {
            "effect draw_cards(self,1);"
        } else {
            "effect kill_player(self,self);"
        };
        let source=format!("skill j2_sliced(99107872) {{ trigger phase_play owner_is_target {{ {first} }} trigger phase_play owner_is_target {{ effect draw_cards(self,2); }} active play_phase targets 0 0 {{ effect draw_cards(self,1); }} }}");
        let p = pack(&source);
        let mut e = start(&p);
        assert!(nc08_common::advance_to_phase(
            &mut e,
            0,
            engine_native::Phase::Draw
        ));
        let before = e.state_core().player(0).hand.len();
        e.set_time_slice(Some(1));
        let mut out = submit(&mut e, 0, GameAction::NextPhase);
        let mut saw = false;
        for _ in 0..200 {
            let pending=e.save().resolution.parks.iter().any(|park|matches!(&park.machine,engine_native::MachineSnapshot::Rvm {committing:true,following,commit_pending,..} if !following.is_empty() && engine_native::rvm_ops::dec_effects(commit_pending).is_some_and(|effects|effects.is_empty())));
            if pending && (disable || !e.state_core().player(0).alive) {
                if disable {
                    let def = e.catalog().skill_by_name("j2_sliced").unwrap();
                    e.state_core_mut()
                        .skills
                        .get_mut(&(0, def))
                        .unwrap()
                        .enabled = false;
                }
                let bytes = engine_native::archive::encode_archive(&e.save());
                let snapshot = engine_native::archive::decode_archive(&bytes).unwrap();
                e = Engine::restore_with_extensions(&snapshot, &[&p]).unwrap();
                e.set_time_slice(Some(1));
                saw = true;
            }
            if !matches!(out, CommandOutcome::Suspended) {
                break;
            }
            out = e.resume();
        }
        assert!(saw, "checkpoint occurs between grouped handlers");
        assert!(
            !matches!(out, CommandOutcome::Suspended | CommandOutcome::Rejected(_)),
            "{out:?}"
        );
        assert_eq!(
            e.state_core().player(0).hand.len(),
            if disable { before + 1 } else { 0 },
            "following draw cannot run after disabled/dead checkpoint"
        );
    }
}

#[test]
fn danshou_option_four_every_continuation_stage_roundtrips_and_matches_uninterrupted() {
    use std::collections::BTreeSet;
    fn drive(
        mut e: Engine,
        p: &ExtensionPack,
        sliced: bool,
    ) -> (Engine, Vec<engine_native::LogEvent>, BTreeSet<u32>) {
        let mut stages = BTreeSet::new();
        let mut events = vec![];
        let mut out = None;
        let mut commands = 0;
        e.set_time_slice(sliced.then_some(1));
        for _ in 0..1000 {
            if matches!(out, Some(CommandOutcome::Suspended)) {
                out = Some(e.resume());
            } else if commands < 4 {
                let action = match commands {
                    0 => GameAction::ActivateSkill {
                        skill: e.catalog().skill_by_name("ol_jie_danshou").unwrap(),
                    },
                    _ => {
                        let choice = e.open_choice().unwrap();
                        let selection = match commands {
                            1 => vec![O::Tag(4)],
                            2 => vec![O::Player(1)],
                            _ => choice.spec.options.iter().take(4).copied().collect(),
                        };
                        GameAction::Answer {
                            choice_id: choice.id,
                            selection,
                        }
                    }
                };
                out = Some(submit(&mut e, 0, action));
                commands += 1;
            } else {
                break;
            }
            assert!(!matches!(out, Some(CommandOutcome::Rejected(_))), "{out:?}");
            if sliced {
                events.extend_from_slice(e.log());
                let snapshot = e.save();
                for park in &snapshot.resolution.parks {
                    if let engine_native::MachineSnapshot::Rvm { commit_state, .. } = &park.machine
                    {
                        if let Some(stage) = commit_state.first() {
                            stages.insert(*stage);
                        }
                    }
                }
                let decoded = engine_native::archive::decode_archive(
                    &engine_native::archive::encode_archive(&snapshot),
                )
                .unwrap();
                e = Engine::restore_with_extensions(&decoded, &[p]).unwrap();
                e.set_time_slice(Some(1));
            }
        }
        assert_eq!(commands, 4);
        assert!(!matches!(out, Some(CommandOutcome::Suspended)));
        assert!(e.open_choice().is_none());
        if !sliced {
            events = e.log().to_vec();
        }
        (e, events, stages)
    }
    let p = pack(include_str!(
        "../../../content/original/ported/onlyOL/ol_jie_danshou.skill"
    ));
    let mut e = start(&p);
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for _ in 0..8 {
        e.debug_place(0, "shan");
    }
    let snapshot = e.save();
    let baseline = Engine::restore_with_extensions(&snapshot, &[&p]).unwrap();
    let sliced = Engine::restore_with_extensions(&snapshot, &[&p]).unwrap();
    let (baseline, events, _) = drive(baseline, &p, false);
    let (sliced, resumed, stages) = drive(sliced, &p, true);
    assert_eq!(events, resumed);
    assert_eq!(baseline.save().core, sliced.save().core);
    assert_eq!(baseline.save().core.rng, sliced.save().core.rng);
    assert_eq!(baseline.save().resolution, sliced.save().resolution);
    assert_eq!(
        stages,
        BTreeSet::from([1, 2, 4, 5, 6, 7]),
        "option4 includes the distinct owner-draw and target-draw continuations"
    );
}

#[test]
fn grouped_following_waits_for_last_discard_subtree_before_rechecking_owner() {
    let source="skill j2_last_window(99107876) { trigger phase_play owner_is_target { effect discard_hand_equip_count(self,1); } trigger phase_play owner_is_target { effect draw_cards(self,2); } active play_phase targets 0 0 { effect draw_cards(self,1); } } skill j2_watcher(99107877) locked { trigger cards_drawn owner_alive { effect kill_player(1,0); } }";
    let mut p = pack(source);
    let mut skills = p.generals[0].skills.to_vec();
    skills.push("连营");
    p.generals[0].skills = Box::leak(skills.into_boxed_slice());
    let mut e = start(&p);
    assert!(nc08_common::advance_to_phase(
        &mut e,
        0,
        engine_native::Phase::Draw
    ));
    e.assign_general_named(1, "watcher").unwrap();
    e.debug_clear_hand(0);
    let card = e.debug_place(0, "shan");
    let cursor = e.log().len();
    submit(&mut e, 0, GameAction::NextPhase);
    if e.open_choice().unwrap().spec.prompt == "trigger_confirm" {
        answer(&mut e, &p, vec![O::Confirm]);
    }
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_discard_pick");
    answer(&mut e, &p, vec![O::Card(card)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    answer(&mut e, &p, vec![O::Confirm]);
    assert!(!e.state_core().player(0).alive);
    assert!(!e.log()[cursor..].iter().any(|event|matches!(event,engine_native::LogEvent::CardsDrawn {player:0,cards} if cards.len()==2)), "following draw must not run after the first handler's discard subtree kills its owner");
    assert!(e.state_core().player(0).hand.is_empty(),"following handler cannot be selected before the first handler's final child subtree settles");
}

fn theft_with_gift_watcher(equipped: bool) {
    let source=format!("{}\nskill j2_watcher_gift(99107880) locked {{ trigger cards_given owner_alive_gave_cards_to_other_alive {{ effect draw_cards(self,1); }} }}\nskill j2_watcher_move(99107881) locked {{ trigger cards_moved owner_alive {{ effect storage_set(self_skill,1,1); }} }}\nskill j2_gear(99107882) locked {{ trigger phase_end owner_alive {{ effect storage_set(self_skill,1,1); }} }}",include_str!("../../../content/original/ported/onlyOL/ol_jie_danshou.skill"));
    let mut p = pack(&source);
    if equipped {
        p.cards.push(
            engine_native::catalog::CardDef::weapon("j2-theft-gear", 2)
                .with_equip_skills(&["j2_gear_phase_end"]),
        );
    }
    let mut e = start(&p);
    e.assign_general_named(1, "watcher").unwrap();
    let gear = if equipped {
        nc08_common::advance_to_play(&mut e, 1);
        let card = e.debug_move_named_card_to_hand(1, "j2-theft-gear");
        assert!(matches!(
            submit(
                &mut e,
                1,
                GameAction::PlayCard {
                    card,
                    targets: vec![]
                }
            ),
            CommandOutcome::Accepted { .. }
        ));
        let def = e.catalog().skill_by_name("j2_gear_phase_end").unwrap();
        assert!(e.state_core().has_skill(1, def));
        Some((card, def))
    } else {
        None
    };
    nc08_common::advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    for _ in 0..6 {
        e.debug_place(0, "shan");
    }
    e.debug_clear_hand(1);
    let hand = e.debug_place(1, "shan");
    e.debug_place(1, "shan");
    let card = gear.map(|(card, _)| card).unwrap_or(hand);
    activate(&mut e, "ol_jie_danshou");
    answer(&mut e, &p, vec![O::Tag(2)]);
    answer(&mut e, &p, vec![O::Player(1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_danshou_card");
    let before = e.save();
    let cursor = e.log().len();
    answer(&mut e, &p, vec![O::Card(card)]);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        if equipped { 2 } else { 1 },
        "the robbed player must not receive a giver-benefit draw"
    );
    assert!(
        !e.log()[cursor..]
            .iter()
            .any(|event| matches!(event, engine_native::LogEvent::CardsGiven { .. })),
        "theft is movement, not a gift"
    );
    assert!(e.state_core().player(0).hand.contains(&card));
    assert!(
        e.state_core()
            .vm_aux
            .flags
            .iter()
            .any(|((owner, key), value)| *owner == 1
                && key.starts_with("storage:j2_watcher_move")
                && *value == 1),
        "the ordinary movement notification still fires"
    );
    if let Some((_, def)) = gear {
        assert!(!e.state_core().player(1).equips.contains(&card));
        assert!(
            !e.state_core().has_skill(1, def),
            "equipment-only skill is revoked"
        );
    }
    let mut sliced = Engine::restore_with_extensions(&before, &[&p]).unwrap();
    sliced.set_time_slice(Some(1));
    let choice = sliced.open_choice().unwrap().id;
    assert!(matches!(
        submit(
            &mut sliced,
            0,
            GameAction::Answer {
                choice_id: choice,
                selection: vec![O::Card(card)]
            }
        ),
        CommandOutcome::Suspended
    ));
    let snapshot = sliced.save();
    let origin = if equipped {
        engine_native::event::LostOrigin::Equip
    } else {
        engine_native::event::LostOrigin::Hand
    };
    assert!(snapshot.resolution.stack.iter().any(|frame|matches!(&frame.data,engine_native::event::EventData::CardsLost {player:1,cards,origins} if cards==&vec![card] && origins==&vec![origin])),"loss notification preserves the source zone");
    assert!(snapshot.resolution.stack.iter().any(|frame|matches!(&frame.data,engine_native::event::EventData::CardsMoved {from:1,to:0,cards} if cards==&vec![card])));
    let mut events = sliced.log().to_vec();
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&snapshot))
            .unwrap();
    sliced = Engine::restore_with_extensions(&decoded, &[&p]).unwrap();
    assert!(matches!(sliced.resume(), CommandOutcome::Pending { .. }));
    events.extend_from_slice(sliced.log());
    assert_eq!(sliced.save().core, e.save().core);
    assert_eq!(sliced.save().resolution, e.save().resolution);
    assert_eq!(events, &e.log()[cursor..]);
    let selection = e
        .open_choice()
        .unwrap()
        .spec
        .options
        .iter()
        .filter(|option| **option != O::Card(card))
        .take(2)
        .copied()
        .collect();
    answer(&mut e, &p, selection);
    assert!(e.state_core().player(0).hand.contains(&card));
}

#[test]
fn danshou_hand_theft_is_movement_without_gift_benefits_after_byte_restore() {
    theft_with_gift_watcher(false);
}

#[test]
fn danshou_equipment_theft_preserves_loss_and_revokes_equipment_skills() {
    theft_with_gift_watcher(true);
}

#[test]
fn fenghe_genuine_gift_keeps_cards_given_and_giver_benefit() {
    let source=format!("{}\nskill j2_giver_gift(99107883) locked {{ trigger cards_given owner_alive_gave_cards_to_other_alive {{ effect draw_cards(self,1); }} }}",include_str!("../../../content/original/ported/shiji/pinghe.skill"));
    let p = pack(&source);
    let mut e = start(&p);
    nc08_common::advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let hand = e.debug_place(0, "shan");
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    let cursor = e.log().len();
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    if e.open_choice().unwrap().spec.prompt == "respond_shan" {
        answer(&mut e, &p, vec![O::Pass]);
    }
    answer(&mut e, &p, vec![O::Card(hand)]);
    answer(&mut e, &p, vec![O::Player(2)]);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        1,
        "a genuine gift still grants the giver-benefit draw"
    );
    assert!(e.state_core().player(2).hand.contains(&hand));
    assert!(e.log()[cursor..].iter().any(|event|matches!(event,engine_native::LogEvent::CardsGiven {from:0,to:2,cards} if cards==&vec![hand])));
}
