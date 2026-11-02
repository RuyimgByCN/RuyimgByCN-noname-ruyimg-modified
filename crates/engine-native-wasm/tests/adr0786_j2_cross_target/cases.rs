//! Scenario families share the formal bridge and command/checkpoint recorder in the parent test.
use super::*;
const DAN: &str = include_str!("../../../../content/original/ported/onlyOL/ol_jie_danshou.skill");
const FENG: &str = include_str!("../../../../content/original/ported/shiji/pinghe.skill");
const PING: &str = include_str!("../../../../content/original/ported/onlyOL/ol_pingkou.skill");
const FEN: &str = include_str!("../../../../content/original/ported/onlyOL/ol_fenli.skill");
const MARKERS: &str = r#"
skill j2_yingba_identity(2187983153) { trigger phase_play owner_is_target { effect storage_set(self_skill, 9, 1); } }
skill j2_pingding_identity(3219705664) locked { trigger phase_end owner_is_target { effect storage_set(self_skill, 9, 1); } }
"#;

fn fill(t: &mut Trace, owner: u8, count: usize) {
    t.engine.debug_clear_hand(owner);
    for _ in 0..count {
        t.engine.debug_place(owner, "shan");
    }
}
fn discard_choice(t: &mut Trace) {
    let c = t.engine.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "rvm_danshou_discard");
    let chosen = c
        .spec
        .options
        .iter()
        .rev()
        .take(c.spec.min)
        .copied()
        .collect();
    t.answer(chosen);
}
fn danshou_branch(t: &mut Trace, option: u8, target: u8) {
    t.activate("ol_jie_danshou");
    t.answer(vec![O::Tag(option)]);
    t.answer(vec![O::Player(target)]);
    if option <= 2 {
        let card = t.engine.open_choice().unwrap().spec.options[0];
        t.answer(vec![card]);
    }
}
fn danshou(output: &mut Vec<Value>) {
    for option in 1..=4 {
        let name = format!("danshou-option-{option}");
        let mut t = Trace::start(DAN, &name);
        t.advance_to_play(0);
        fill(&mut t, 0, 8);
        fill(&mut t, 1, 2);
        t.begin();
        let before = t.engine.state_core().player(0).hand.len();
        danshou_branch(&mut t, option, 1);
        assert_eq!(t.engine.open_choice().unwrap().spec.min, option as usize);
        if option == 3 {
            assert_eq!(t.engine.debug_hp(1), 3);
        }
        discard_choice(&mut t);
        let gain = if option == 2 {
            1
        } else if option == 4 {
            2
        } else {
            0
        };
        assert_eq!(
            t.engine.state_core().player(0).hand.len(),
            before + gain - option as usize
        );
        t.activate("ol_jie_danshou");
        assert!(!t
            .engine
            .open_choice()
            .unwrap()
            .spec
            .options
            .contains(&O::Tag(option)));
        t.answer(vec![]);
        output.push(t.finish(&name));
    }
    let mut t = Trace::start(DAN, "danshou-cancel");
    t.advance_to_play(0);
    t.begin();
    let hand = t.engine.state_core().player(0).hand.clone();
    t.activate("ol_jie_danshou");
    t.answer(vec![]);
    assert_eq!(t.engine.state_core().player(0).hand, hand);
    output.push(t.finish("danshou-cancel"));
    let mut t = Trace::start(DAN, "danshou-insufficient");
    t.advance_to_play(0);
    fill(&mut t, 0, 1);
    t.begin();
    danshou_branch(&mut t, 3, 1);
    assert_eq!(t.engine.open_choice().unwrap().spec.min, 1);
    discard_choice(&mut t);
    assert!(t.engine.state_core().player(0).hand.is_empty());
    output.push(t.finish("danshou-insufficient"));
    let mut t = Trace::start(DAN, "danshou-self-draw");
    t.advance_to_play(0);
    fill(&mut t, 0, 8);
    t.begin();
    danshou_branch(&mut t, 4, 0);
    discard_choice(&mut t);
    assert_eq!(t.engine.state_core().player(0).hand.len(), 8);
    output.push(t.finish("danshou-self-draw"));
    for save in [false, true] {
        let name = if save {
            "danshou-dying-save"
        } else {
            "danshou-dying-death"
        };
        let mut t = Trace::start(DAN, name);
        t.advance_to_play(0);
        for seat in 0..4 {
            fill(&mut t, seat, 0);
        }
        fill(&mut t, 0, 8);
        let tao = save.then(|| t.engine.debug_place(0, "tao"));
        t.engine.debug_set_hp(1, 1);
        t.begin();
        danshou_branch(&mut t, 3, 1);
        while t
            .engine
            .open_choice()
            .is_some_and(|c| c.spec.prompt == "dying_save")
        {
            let c = t.engine.open_choice().unwrap();
            if c.observer == 0 && save {
                t.answer(vec![O::Card(tao.unwrap())]);
            } else {
                t.answer(vec![O::Pass]);
            }
        }
        assert_eq!(t.engine.state_core().player(1).alive, save);
        discard_choice(&mut t);
        output.push(t.finish(name));
    }
    let mut t = Trace::start(DAN, "danshou-four-options-and-reset");
    t.advance_to_play(0);
    fill(&mut t, 0, 16);
    for seat in 1..4 {
        fill(&mut t, seat, 2);
    }
    t.begin();
    for option in 1..=4 {
        danshou_branch(&mut t, option, 1);
        discard_choice(&mut t);
    }
    let skill = t.engine.catalog().skill_by_name("ol_jie_danshou").unwrap();
    assert!(!t
        .engine
        .legal_actions(0)
        .iter()
        .any(|action| matches!(action,GameAction::ActivateSkill {skill:s} if *s==skill)));
    t.step(json!({"type":"nextPhase","actor":0}));
    t.drain();
    t.advance_to_play(1);
    t.advance_to_play(0);
    fill_assert(&t);
    t.activate("ol_jie_danshou");
    assert_eq!(t.engine.open_choice().unwrap().spec.options.len(), 4);
    t.answer(vec![]);
    output.push(t.finish("danshou-four-options-and-reset"));
}
fn fill_assert(t: &Trace) {
    assert!(!t.engine.state_core().player(0).hand.is_empty());
}

fn fenghe(output: &mut Vec<Value>) {
    for mode in 0..4 {
        let names = [
            "fenghe-give-owned-disabled-yingba",
            "fenghe-give-no-yingba",
            "fenghe-recast",
            "fenghe-source-dies-during-give",
        ];
        let name = names[mode];
        let mut source = format!("{FENG}\n{MARKERS}");
        if mode == 3 {
            source.push_str("skill j2_after_give(99107866) locked { trigger cards_given owner_alive_gave_cards_to_other_alive { effect kill_player(event_player, self); } }");
        }
        let mut t = Trace::start(&source, name);
        t.advance_to_play(1);
        let yingba = t
            .engine
            .catalog()
            .skill_by_name("j2_yingba_identity")
            .unwrap();
        if mode == 1 {
            t.engine.state_core_mut().skills.remove(&(0, yingba));
        } else {
            t.engine
                .state_core_mut()
                .skills
                .get_mut(&(0, yingba))
                .unwrap()
                .enabled = false;
        }
        fill(&mut t, 0, 0);
        let card = t
            .engine
            .debug_place(0, if mode == 2 { "chain" } else { "shan" });
        fill(&mut t, 1, 0);
        let sha = t.engine.debug_place(1, "sha");
        t.begin();
        t.step(json!({"type":"playCard","actor":1,"card":sha,"targets":[0]}));
        if t.engine
            .open_choice()
            .is_some_and(|c| c.spec.prompt == "respond_shan")
        {
            t.answer(vec![O::Pass]);
        }
        assert_eq!(
            t.engine.open_choice().unwrap().spec.prompt,
            "rvm_fenghe_card"
        );
        assert_eq!(t.engine.state_core().player(0).max_hp, 3);
        assert_eq!(t.engine.debug_hp(0), 3);
        t.answer(vec![O::Card(card)]);
        if mode == 0 {
            let mut forged = t.engine.save();
            let wrong = t
                .engine
                .state_core()
                .deck
                .iter()
                .copied()
                .find(|c| {
                    t.engine.catalog().card_kind(*c) != engine_native::catalog::CardKind::Chain
                })
                .unwrap();
            for park in &mut forged.resolution.parks {
                if let engine_native::MachineSnapshot::Rvm { commit_state, .. } = &mut park.machine
                {
                    if commit_state.first() == Some(&2) {
                        commit_state[1] = wrong;
                    }
                }
            }
            let bytes = hex_encode(&engine_native::archive::encode_archive(&forged));
            assert!(engine_from_archive(&bytes, &t.packs).is_err());
            t.invalid_archives.push(bytes);
        }
        assert_eq!(
            t.engine
                .open_choice()
                .unwrap()
                .spec
                .options
                .contains(&O::Pass),
            mode == 2
        );
        t.answer(vec![if mode == 2 { O::Pass } else { O::Player(1) }]);
        t.drain();
        assert_eq!(
            t.engine
                .state_core()
                .vm_aux
                .marks
                .get(&(1, 3219705664))
                .copied()
                .unwrap_or(0),
            if mode == 0 { 1 } else { 0 }
        );
        if mode == 3 {
            assert!(!t.engine.state_core().player(1).alive);
        }
        output.push(t.finish(name));
    }
    for (name, source_expr, dead) in [
        ("fenghe-self-source", "self", false),
        ("fenghe-missing-source", "event_source", false),
        ("fenghe-dead-source", "2", true),
    ] {
        let source=format!("{FENG}\nskill j2_damage_probe(99107867) {{ active play_phase targets 0 0 {{ effect deal_damage({source_expr},self,1); }} }}");
        let mut t = Trace::start(&source, name);
        t.advance_to_play(0);
        fill(&mut t, 0, 4);
        if dead {
            t.engine.state_core_mut().player_mut(1).alive = false;
        }
        t.begin();
        t.activate("j2_damage_probe");
        t.drain();
        assert_eq!(t.engine.state_core().player(0).max_hp, 4);
        assert_eq!(t.engine.debug_hp(0), 3);
        output.push(t.finish(name));
    }
    for empty in [false, true] {
        let name = if empty {
            "fenghe-empty-hand"
        } else {
            "fenghe-max-hp-one"
        };
        let mut t = Trace::start(FENG, name);
        t.advance_to_play(1);
        for seat in 0..4 {
            fill(&mut t, seat, 0);
        }
        if !empty {
            t.engine.debug_set_max_hp(0, 1);
            t.engine.debug_set_hp(0, 1);
            t.engine.debug_place(0, "shan");
        }
        let sha = t.engine.debug_place(1, "sha");
        t.begin();
        t.step(json!({"type":"playCard","actor":1,"card":sha,"targets":[0]}));
        while let Some(c) = t.engine.open_choice() {
            assert!(!c.spec.prompt.starts_with("rvm_fenghe"));
            t.default_answer();
        }
        assert_eq!(
            t.engine.state_core().player(0).max_hp,
            if empty { 4 } else { 1 }
        );
        output.push(t.finish(name));
    }
}

fn pingkou(output: &mut Vec<Value>) {
    for mode in 0..5 {
        let names = [
            "pingkou-real-three-skips",
            "pingkou-prevention-and-save",
            "pingkou-prevention-and-death",
            "pingkou-cancel",
            "pingkou-next-target-dies-before-damage",
        ];
        let name = names[mode];
        let mut source = format!("{FEN}\n{PING}");
        if mode == 1 || mode == 2 {
            source.push_str("skill j2_prevent(99107863) locked { trigger damage_before owner_is_target { effect prevent_current_damage(); } }");
        }
        if mode == 4 {
            source.push_str("skill j2_kill_next(99107864) locked { trigger death owner_alive_not_target { if 0 < hp(3) { effect kill_player(3,self); } } }");
        }
        let mut t = Trace::start(&source, name);
        if mode == 1 || mode == 2 {
            t.engine.assign_general_named(1, "j2-cross-author").unwrap();
        }
        for seat in 0..4 {
            fill(&mut t, seat, 0);
        }
        fill(&mut t, 0, 8);
        let equipment_name = t
            .engine
            .catalog()
            .cards
            .iter()
            .find(|card| card.kind.equip_slot().is_some())
            .unwrap()
            .name;
        let equipment = t.engine.debug_place(0, equipment_name);
        t.engine
            .state_core_mut()
            .player_mut(0)
            .hand
            .retain(|c| *c != equipment);
        t.engine
            .state_core_mut()
            .player_mut(0)
            .equips
            .push(equipment);
        let tao = (mode == 1).then(|| t.engine.debug_place(0, "tao"));
        if mode == 1 || mode == 2 {
            t.engine.debug_set_hp(2, 1);
        }
        if mode == 4 {
            t.engine.debug_set_hp(1, 1);
        }
        t.begin();
        for _ in 0..20 {
            if t.engine
                .open_choice()
                .is_some_and(|c| c.spec.prompt == "rvm_pingkou_targets")
            {
                break;
            }
            if t.engine.open_choice().is_some() {
                t.default_answer();
            } else {
                t.step(json!({"type":"nextPhase","actor":0}));
            }
        }
        assert_eq!(
            t.engine.open_choice().unwrap().spec.max,
            3,
            "all three actual skipped phases count"
        );
        let hand = t.engine.state_core().player(0).hand.len();
        if mode == 3 {
            t.answer(vec![]);
            assert_eq!(t.engine.state_core().player(0).hand.len(), hand);
            output.push(t.finish(name));
            continue;
        }
        t.answer(vec![O::Player(3), O::Player(2), O::Player(1)]);
        while let Some(c) = t.engine.open_choice() {
            assert_eq!(c.spec.prompt, "dying_save");
            if mode == 1 && c.observer == 0 {
                t.answer(vec![O::Card(tao.unwrap())]);
            } else {
                t.answer(vec![O::Pass]);
            }
        }
        let draws = if mode == 0 { 3 } else { 2 };
        let saved_cost = usize::from(mode == 1);
        assert_eq!(
            t.engine.state_core().player(0).hand.len(),
            hand + draws - saved_cost
        );
        if mode == 1 || mode == 2 {
            assert_eq!(t.engine.debug_hp(1), 4);
            assert_eq!(t.engine.state_core().player(2).alive, mode == 1);
        }
        if mode == 4 {
            assert!(!t.engine.state_core().player(1).alive);
            assert!(!t.engine.state_core().player(2).alive);
            assert_eq!(t.engine.debug_hp(3), 3);
        }
        assert!(!t
            .engine
            .state_core()
            .vm_aux
            .flags
            .keys()
            .any(|(_, key)| key.starts_with("vm_damage_receipt:")));
        output.push(t.finish(name));
    }
}

fn reviewed_boundaries(output: &mut Vec<Value>) {
    for kill in [false, true] {
        let name = if kill {
            "danshou-owner-draw-kills-target"
        } else {
            "danshou-owner-draw-nested-choice-order"
        };
        let body = if kill {
            "effect kill_player(self,event_player);"
        } else {
            "effect discard_hand_equip_count(self,1);"
        };
        let source=format!("{DAN}\nskill j2_watcher(99107868) locked {{ trigger cards_drawn owner_alive {{ {body} }} }}");
        let mut t = Trace::start(&source, name);
        t.advance_to_play(0);
        t.engine
            .assign_general_named(1, "j2-cross-watcher")
            .unwrap();
        fill(&mut t, 0, 8);
        fill(&mut t, 1, 3);
        t.begin();
        t.activate("ol_jie_danshou");
        t.answer(vec![O::Tag(4)]);
        t.answer(vec![O::Player(1)]);
        if kill {
            assert!(!t.engine.state_core().player(1).alive);
        } else {
            assert_eq!(
                t.engine.open_choice().unwrap().spec.prompt,
                "rvm_discard_pick"
            );
            assert_eq!(t.engine.state_core().player(1).hand.len(), 3);
            let card = t.engine.open_choice().unwrap().spec.options[0];
            t.answer(vec![card]);
            assert_eq!(t.engine.state_core().player(1).hand.len(), 4);
            let card = t.engine.open_choice().unwrap().spec.options[0];
            t.answer(vec![card]);
        }
        discard_choice(&mut t);
        assert_eq!(t.engine.state_core().player(0).hand.len(), 6);
        output.push(t.finish(name));
    }
    for remove in [false, true] {
        let name = if remove {
            "grouped-handler-removed-after-choice"
        } else {
            "grouped-owner-dead-after-choice"
        };
        let effect = if remove {
            "effect remove_skill(self_skill);"
        } else {
            "effect kill_player(self,self);"
        };
        let source=format!("skill j2_group(99107872) {{ trigger phase_play owner_is_target {{ effect discard_hand_equip_count(self,1); {effect} }} trigger phase_play owner_is_target {{ effect draw_cards(self,2); }} active play_phase targets 0 0 {{ effect draw_cards(self,1); }} }}");
        let mut t = Trace::start(&source, name);
        t.begin();
        for _ in 0..10 {
            if t.engine.open_choice().is_some() {
                break;
            }
            t.step(json!({"type":"nextPhase","actor":0}));
        }
        assert_eq!(
            t.engine.open_choice().unwrap().spec.prompt,
            "rvm_discard_pick"
        );
        let card = t.engine.open_choice().unwrap().spec.options[0];
        let hand = t.engine.state_core().player(0).hand.len();
        t.answer(vec![card]);
        assert_eq!(
            t.engine.state_core().player(0).hand.len(),
            if remove { hand - 1 } else { 0 }
        );
        output.push(t.finish(name));
    }
    let source="skill j2_disabled(99107874) { trigger phase_play owner_is_target { effect discard_hand_equip_count(self,1); effect draw_cards(self,1); } trigger phase_play owner_is_target { effect draw_cards(self,2); } active play_phase targets 0 0 { effect draw_cards(self,1); } }";
    let mut t = Trace::start(source, "grouped-instance-disabled-at-checkpoint");
    for _ in 0..10 {
        if t.engine.open_choice().is_some() {
            break;
        }
        t.step(json!({"type":"nextPhase","actor":0}));
    }
    let def = t.engine.catalog().skill_by_name("j2_disabled").unwrap();
    t.engine
        .state_core_mut()
        .skills
        .get_mut(&(0, def))
        .unwrap()
        .enabled = false;
    t.begin();
    let hand = t.engine.state_core().player(0).hand.len();
    let card = t.engine.open_choice().unwrap().spec.options[0];
    t.answer(vec![card]);
    assert_eq!(t.engine.state_core().player(0).hand.len(), hand);
    output.push(t.finish("grouped-instance-disabled-at-checkpoint"));
    let source="skill j2_private(99107873) { trigger phase_play owner_is_target { effect storage_set(self_skill,1,self); } active play_phase targets 0 0 { if storage_has(self_skill,1) { if storage_get(self_skill,1)==self { effect draw_cards(self,1); } } } }";
    for other in [false, true] {
        let name = if other {
            "storage-two-owner-no-mark-fallback"
        } else {
            "storage-trigger-active-owner-identity"
        };
        let mut t = Trace::start(source, name);
        t.engine.assign_general_named(1, "j2-cross-author").unwrap();
        t.advance_to_play(0);
        if other {
            t.engine.state_core_mut().vm_aux.marks.insert((0, 1), 2);
            t.engine.state_core_mut().turn.current = 1;
        }
        t.begin();
        let owner = u8::from(other);
        let hand = t.engine.state_core().player(owner).hand.len();
        let skill = t.engine.catalog().skill_by_name("j2_private").unwrap();
        t.step(json!({"type":"activateSkill","actor":owner,"skill":skill}));
        assert_eq!(
            t.engine.state_core().player(owner).hand.len(),
            hand + usize::from(!other)
        );
        output.push(t.finish(name));
    }
    for unreachable in [false, true] {
        let name = if unreachable {
            "active-unreachable-danshou"
        } else {
            "active-preceding-draw-danshou"
        };
        let body = if unreachable {
            "if 0==1 { effect resolve_ol_jie_danshou(self); } else { effect draw_cards(self,1); }"
        } else {
            "effect draw_cards(self,1); effect resolve_ol_jie_danshou(self);"
        };
        let source =
            format!("skill j2_prefix(99107875) {{ active play_phase targets 0 0 {{ {body} }} }}");
        let mut t = Trace::start(&source, name);
        t.advance_to_play(0);
        fill(&mut t, 0, 0);
        t.begin();
        t.activate("j2_prefix");
        assert_eq!(t.engine.state_core().player(0).hand.len(), 1);
        if t.engine.open_choice().is_some() {
            t.answer(vec![]);
        }
        output.push(t.finish(name));
    }
}

fn theft_and_gift_events(output: &mut Vec<Value>) {
    let watcher="skill j2_watcher_gift(99107880) locked { trigger cards_given owner_alive_gave_cards_to_other_alive { effect draw_cards(self,1); } } skill j2_watcher_move(99107881) locked { trigger cards_moved owner_alive { effect storage_set(self_skill,1,1); } } skill j2_gear(99107882) locked { trigger phase_end owner_alive { effect storage_set(self_skill,1,1); } }";
    for equipped in [false, true] {
        let name = if equipped {
            "danshou-equipment-theft-not-gift"
        } else {
            "danshou-hand-theft-not-gift"
        };
        let mut t = Trace::start(&format!("{DAN}\n{watcher}"), name);
        t.engine
            .assign_general_named(1, "j2-cross-watcher")
            .unwrap();
        let gear = if equipped {
            t.advance_to_play(1);
            let card = t.engine.debug_move_named_card_to_hand(1, "j2-theft-gear_0");
            t.step(json!({"type":"playCard","actor":1,"card":card,"targets":[]}));
            let def = t.engine.catalog().skill_by_name("j2_theft_gear").unwrap();
            assert!(t.engine.state_core().has_skill(1, def));
            Some((card, def))
        } else {
            None
        };
        t.advance_to_play(0);
        fill(&mut t, 0, 6);
        fill(&mut t, 1, 2);
        let card = gear
            .map(|(card, _)| card)
            .unwrap_or(t.engine.state_core().player(1).hand[0]);
        t.begin();
        t.activate("ol_jie_danshou");
        t.answer(vec![O::Tag(2)]);
        t.answer(vec![O::Player(1)]);
        assert_eq!(
            t.engine.open_choice().unwrap().spec.prompt,
            "rvm_danshou_card"
        );
        let cursor = t.engine.log().len();
        t.answer(vec![O::Card(card)]);
        assert_eq!(
            t.engine.state_core().player(1).hand.len(),
            if equipped { 2 } else { 1 },
            "theft must not trigger giver benefits"
        );
        assert!(!t.engine.log()[cursor..]
            .iter()
            .any(|event| matches!(event, engine_native::LogEvent::CardsGiven { .. })));
        assert!(t
            .engine
            .state_core()
            .vm_aux
            .flags
            .iter()
            .any(|((owner, key), value)| *owner == 1
                && key.starts_with("storage:j2_watcher_move")
                && *value == 1));
        if let Some((_, def)) = gear {
            assert!(!t.engine.state_core().player(1).equips.contains(&card));
            assert!(!t.engine.state_core().has_skill(1, def));
        }
        let selection = t
            .engine
            .open_choice()
            .unwrap()
            .spec
            .options
            .iter()
            .filter(|o| **o != O::Card(card))
            .take(2)
            .copied()
            .collect();
        t.answer(selection);
        assert!(t.engine.state_core().player(0).hand.contains(&card));
        output.push(t.finish(name));
    }
    let name = "fenghe-genuine-gift-benefit";
    let source=format!("{FENG}\nskill j2_giver_gift(99107883) locked {{ trigger cards_given owner_alive_gave_cards_to_other_alive {{ effect draw_cards(self,1); }} }}");
    let mut t = Trace::start(&source, name);
    t.advance_to_play(1);
    fill(&mut t, 0, 0);
    let card = t.engine.debug_place(0, "shan");
    fill(&mut t, 1, 0);
    let sha = t.engine.debug_place(1, "sha");
    t.begin();
    let cursor = t.engine.log().len();
    t.step(json!({"type":"playCard","actor":1,"card":sha,"targets":[0]}));
    if t.engine
        .open_choice()
        .is_some_and(|c| c.spec.prompt == "respond_shan")
    {
        t.answer(vec![O::Pass]);
    }
    t.answer(vec![O::Card(card)]);
    t.answer(vec![O::Player(2)]);
    assert_eq!(t.engine.state_core().player(0).hand.len(), 1);
    assert!(t.engine.state_core().player(2).hand.contains(&card));
    assert!(t.engine.log()[cursor..].iter().any(|event|matches!(event,engine_native::LogEvent::CardsGiven {from:0,to:2,cards} if cards==&vec![card])));
    output.push(t.finish(name));
}

pub(super) fn scenarios() -> Vec<Value> {
    let mut output = vec![];
    danshou(&mut output);
    fenghe(&mut output);
    pingkou(&mut output);
    reviewed_boundaries(&mut output);
    theft_and_gift_events(&mut output);
    output
}
