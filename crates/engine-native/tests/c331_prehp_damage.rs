use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, DamageOutcome, DamageProgress, EventData, EventKind};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

fn package(source: &str, bindings: &[(&str, EventKind)]) -> ExtensionPack {
    let raw = content_compiler::compile_source_named("c9b", source).unwrap();
    let mut decls = content_native_loader::bridge_vm_skill_decls(&raw)
        .unwrap()
        .decls;
    for (name, kind) in bindings {
        decls
            .iter_mut()
            .find(|d| d.name == *name)
            .unwrap()
            .trigger
            .event = *kind;
    }
    ExtensionPack {
        def: ExtensionDef {
            name: "c9b",
            version: 1,
            description: "actual declared damage windows",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    let out = e.submit(GameCommand {
        actor: c.observer,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action: GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
fn restore(e: &Engine, p: &ExtensionPack) -> Engine {
    let s = e.save();
    let bytes = engine_native::archive::encode_archive(&s);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(decoded, s);
    let r = Engine::restore_with_extensions(&decoded, &[p]).unwrap();
    let mut normalized = r.save();
    normalized.auth_generation = s.auth_generation;
    assert_eq!(normalized, s);
    assert_eq!(e.state_hash(), r.state_hash());
    assert!(
        r.log().is_empty(),
        "restore starts a fresh runtime log; compare continuation suffixes"
    );
    r
}
fn same(a: &Engine, b: &Engine) {
    let mut sa = a.save();
    let mut sb = b.save();
    sa.auth_generation = 0;
    sb.auth_generation = 0;
    assert_eq!(sa, sb);
    assert_eq!(a.state_hash(), b.state_hash());
    assert!(
        a.log().ends_with(b.log()),
        "restored continuation events must match the original suffix"
    );
}
#[test]
fn c9b_every_pre_hp_window_roundtrips_and_modifies_one_canonical_root() {
    let p=package("skill c9b_a(99109111) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } } skill c9b_b(99109112) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } } skill c9b_c(99109113) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } } skill c9b_d(99109114) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } } skill c9b_e(99109115) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } } skill c9b_f(99109116) { trigger damage_before owner_is_target optional { effect modify_current_damage(self,1); } } skill c9b_t(99109117) { trigger damage owner_is_target optional { effect draw_cards(self,event_amount); } }",&[("c9b_b",EventKind::DamageBegin),("c9b_c",EventKind::DamageBegin1),("c9b_d",EventKind::DamageBegin2),("c9b_e",EventKind::DamageBegin3),("c9b_f",EventKind::DamageBegin4)]);
    let mut e = Engine::new_with_extensions(12011, 3, &[&p]).unwrap();
    e.debug_set_max_hp(1, 20);
    e.debug_set_hp(1, 20);
    for name in [
        "c9b_a", "c9b_b", "c9b_c", "c9b_d", "c9b_e", "c9b_f", "c9b_t",
    ] {
        e.debug_grant(1, name);
    }
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    for (i, kind) in [
        EventKind::DamageBefore,
        EventKind::DamageBegin,
        EventKind::DamageBegin1,
        EventKind::DamageBegin2,
        EventKind::DamageBegin3,
        EventKind::DamageBegin4,
        EventKind::Damage,
    ]
    .into_iter()
    .enumerate()
    {
        let s = e.save();
        let root = s
            .resolution
            .stack
            .iter()
            .find(|f| matches!(f.data, EventData::Damage(_)))
            .unwrap();
        let d = root.damage();
        assert_eq!(d.amount, 1 + i as i32);
        assert_eq!(
            d.settlement.committed_damage,
            if i == 6 { Some(7) } else { None }
        );
        assert_eq!(e.debug_hp(1), if i == 6 { 13 } else { 20 });
        let window = s
            .resolution
            .stack
            .iter()
            .find(|f| matches!(f.data, EventData::DamageWindow { .. }))
            .unwrap();
        assert_eq!(window.parent, Some(root.id));
        assert_eq!(window.kind, kind);
        let mut r = restore(&e, &p);
        let before = e.save();
        let mut forged = before.clone();
        let root = forged
            .resolution
            .stack
            .iter_mut()
            .find(|f| matches!(f.data, EventData::Damage(_)))
            .unwrap();
        root.damage_mut().settlement.outcome = DamageOutcome::Completed;
        assert!(Engine::restore_with_extensions(&forged, &[&p]).is_err());
        assert_eq!(e.save(), before);
        answer(&mut e, vec![ChoiceOption::Confirm]);
        answer(&mut r, vec![ChoiceOption::Confirm]);
        same(&e, &r);
    }
    assert!(e.open_choice().is_none());
    assert_eq!(e.debug_hp(1), 13);
    assert_eq!(
        e.log()
            .iter()
            .filter(|x| matches!(
                x,
                engine_native::machine::LogEvent::DamageDealt { amount: 7, .. }
            ))
            .count(),
        1
    );
}
#[test]
fn c9b_post_submit_zero_stops_source_end_after_but_keeps_receipt() {
    let p=package("skill c9b_stop(99109121) { trigger damage_before owner_is_target { effect modify_current_damage(self,-2); } } skill c9b_zero(99109122) { trigger damage owner_is_target optional { effect draw_cards(self,1); } } skill c9b_end(99109123) { trigger damage owner_is_target { effect draw_cards(self,9); } }",&[("c9b_stop",EventKind::Damage),("c9b_zero",EventKind::DamageZero),("c9b_end",EventKind::DamageEnd)]);
    let mut e = Engine::new_with_extensions(12021, 3, &[&p]).unwrap();
    for name in ["c9b_stop", "c9b_zero", "c9b_end"] {
        e.debug_grant(1, name);
    }
    let h = e.debug_hand(1).len();
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 2);
    let s = e.save();
    let d = s
        .resolution
        .stack
        .iter()
        .find(|f| matches!(f.data, EventData::Damage(_)))
        .unwrap()
        .damage();
    assert_eq!(d.amount, 0);
    assert_eq!(d.settlement.committed_damage, Some(2));
    assert_eq!(d.settlement.hp_loss, 2);
    assert_eq!(d.settlement.outcome, DamageOutcome::Zero);
    assert_eq!(d.settlement.progress, DamageProgress::Stopped);
    let mut r = restore(&e, &p);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    answer(&mut r, vec![ChoiceOption::Confirm]);
    same(&e, &r);
    assert_eq!(e.debug_hand(1).len(), h + 1);
}
#[test]
fn c9b_source_death_is_cleared_before_end_and_restore() {
    let p=package("skill c9b_kill(99109131) { trigger damage owner_is_source { effect lose_hp(self,1); } } skill c9b_end(99109132) { trigger damage owner_is_target optional { effect draw_cards(self,event_amount); } }",&[("c9b_kill",EventKind::DamageSource),("c9b_end",EventKind::DamageEnd)]);
    let mut e = Engine::new_with_extensions(12031, 3, &[&p]).unwrap();
    e.debug_grant(0, "c9b_kill");
    e.debug_grant(1, "c9b_end");
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert!(!e.debug_alive(0));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
    let s = e.save();
    let d = s
        .resolution
        .stack
        .iter()
        .find(|f| matches!(f.data, EventData::Damage(_)))
        .unwrap()
        .damage();
    assert_eq!(d.source, None);
    assert_eq!(d.amount, 2);
    let mut r = restore(&e, &p);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    answer(&mut r, vec![ChoiceOption::Confirm]);
    same(&e, &r);
    assert_eq!(e.debug_hand(1).len(), 2);
}
#[test]
fn c9b_notrigger_suppresses_windows_without_suppressing_submission() {
    let p=package("skill c9b_watch(99109141) { trigger damage owner_is_target { effect draw_cards(self,9); } }",&[]);
    let mut e = Engine::new_with_extensions(12041, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9b_watch");
    let mut core = e.save().core;
    let mut res = engine_native::resolution::Resolution::new(10000);
    let mut logs = Vec::new();
    let mut d = engine_native::machine::damage_data(Some(0), 1, 2);
    d.settlement.notrigger = true;
    let id = engine_native::resolution::push_frame(&mut res, EventData::Damage(d));
    let hand = core.player(1).hand.len();
    let mut submitted = false;
    for _ in 0..200 {
        let out =
            engine_native::resolution::run(&mut core, &mut res, e.catalog(), &mut logs, Some(1));
        if let Some(root) = res.frame(id) {
            let d = root.damage();
            if d.settlement.committed_damage == Some(2) {
                submitted = true;
                assert_eq!(d.amount, 2);
                assert_eq!(d.settlement.hp_loss, 2);
            }
        }
        assert!(!matches!(
            out,
            engine_native::resolution::StepOutcome::Fault
                | engine_native::resolution::StepOutcome::Pending
        ));
        if out == engine_native::resolution::StepOutcome::Done {
            break;
        }
    }
    assert!(submitted);
    assert!(res.stack.is_empty());
    assert_eq!(core.player(1).hp, 2);
    assert_eq!(core.player(1).hand.len(), hand);
}

#[test]
fn c9b_submission_slice_restore_does_not_commit_twice_and_rejects_completed_progress() {
    let mut p=package("skill c9b_multi(99109151) { trigger damage_before owner_is_target { effect draw_cards(self,1); } trigger damage_before owner_is_target { effect draw_cards(self,1); } }",&[]);
    for decl in &mut p.vm_skills {
        decl.trigger.event = EventKind::DamageBegin2;
        if let engine_native::rvm_bridge::VmFilterPolicy::Handlers(set) = &mut decl.filter {
            for handler in &mut std::sync::Arc::make_mut(set).triggers {
                handler.meta.event = EventKind::DamageBegin2;
            }
        }
    }
    let mut e = Engine::new_with_extensions(12051, 3, &[&p]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_grant(1, "c9b_multi");
    e.debug_clear_hand(1);
    let card = e.debug_place(0, "sha");
    e.set_time_slice(Some(1));
    e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    let mut branch = None;
    let mut saw_submission = false;
    let mut checked_matrix = false;
    let mut saw_following = false;
    for _ in 0..300 {
        let s = e.save();
        if s.resolution
            .stack
            .iter()
            .any(|f| matches!(f.data, EventData::DamageWindow { .. }))
            && s.resolution.open_choice.is_none()
        {
            let mut invalid = s.clone();
            if let Some(driver) = invalid.resolution.parks.iter_mut().find(|park| {
                matches!(
                    park.machine,
                    engine_native::snap::MachineSnapshot::DamageFlow { .. }
                )
            }) {
                driver.resume_depth += 1;
                assert!(Engine::restore_with_extensions(&invalid, &[&p]).is_err());
                assert_eq!(e.save(), s);
            }
        }

        if s.resolution.parks.iter().any(|park|matches!(&park.machine,engine_native::snap::MachineSnapshot::Rvm{following,..}if !following.is_empty())) {let _=restore(&e,&p);saw_following=true;}

        if let Some(root) = s
            .resolution
            .stack
            .iter()
            .find(|f| matches!(f.data, EventData::Damage(_)))
        {
            let d = root.damage();
            if d.settlement.progress == DamageProgress::SubmitHp {
                saw_submission = true;
                assert_eq!(d.settlement.committed_damage, None);
                assert_eq!(e.debug_hp(1), 4);
                let mut r = restore(&e, &p);
                r.set_time_slice(None);
                assert!(!matches!(r.resume(), CommandOutcome::Rejected(_)));
                branch = Some(r);
            }
            if matches!(d.settlement.progress, DamageProgress::HpCommitted { .. }) {
                assert_eq!(d.amount, 1);
                assert_eq!(d.settlement.committed_damage, Some(1));
                assert_eq!(d.settlement.hp_loss, 1);
                let mut invalid = s.clone();
                let d = invalid
                    .resolution
                    .stack
                    .iter_mut()
                    .find(|f| matches!(f.data, EventData::Damage(_)))
                    .unwrap()
                    .damage_mut();
                d.settlement.outcome = DamageOutcome::Completed;
                assert!(Engine::restore_with_extensions(&invalid, &[&p]).is_err());
                assert_eq!(e.save(), s);
                checked_matrix = true;
            }
        }
        if s.resolution.stack.is_empty() && !s.segment_open {
            break;
        }
        assert!(!matches!(e.resume(), CommandOutcome::Rejected(_)));
    }
    assert!(
        saw_submission && checked_matrix && saw_following,
        "submission={saw_submission} matrix={checked_matrix} following={saw_following}"
    );
    let r = branch.unwrap();
    same(&e, &r);
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(e.state_core().player(1).hujia, 0);
    assert_eq!(
        e.log()
            .iter()
            .filter(|x| matches!(x, engine_native::machine::LogEvent::DamageDealt { .. }))
            .count(),
        1
    );
}
#[test]
fn c9b_dead_parent_keeps_chain_eligibility_and_siblings_unlink_on_their_own_submission() {
    let p=package("skill c9b_chain_pause(99109161) { trigger damage_before owner_is_target optional { effect draw_cards(self,1); } }",&[]);
    let mut e = Engine::new_with_extensions(12061, 4, &[&p]).unwrap();
    e.debug_grant(2, "c9b_chain_pause");
    for player in 0..4 {
        e.debug_clear_hand(player);
    }
    e.debug_set_hp(1, 1);
    for player in [1, 2, 3] {
        e.state_core_mut().player_mut(player).linked = true;
    }
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert!(!e.debug_alive(1));
    assert!(!e.state_core().player(1).linked);
    // 上游顺序（2026-11-02 裁決：先解链再传导）：p2 已进入自己的传导伤害提交，
    // 因此在暂停点就已解除横置；还没轮到的 p3 仍横置。
    assert!(!e.state_core().player(2).linked);
    assert!(e.state_core().player(3).linked);
    assert_eq!(e.debug_hp(2), 4);
    assert_eq!(e.debug_hp(3), 4);
    let mut r = restore(&e, &p);
    answer(&mut e, vec![]);
    answer(&mut r, vec![]);
    same(&e, &r);
    assert_eq!(e.debug_hp(2), 3);
    assert_eq!(e.debug_hp(3), 3);
    assert!(!e.state_core().player(2).linked);
    assert!(!e.state_core().player(3).linked);
}
#[test]
fn c9b_post_submit_cancel_has_a_real_cancelled_child_without_zero_or_end() {
    let p=package("skill c9b_cancel(99109171) { trigger damage_before owner_is_target { effect prevent_current_damage(); } } skill c9b_cancelled(99109172) { trigger damage owner_is_target optional { effect draw_cards(self,1); } } skill c9b_end(99109173) { trigger damage owner_is_target { effect draw_cards(self,9); } }",&[("c9b_cancel",EventKind::Damage),("c9b_cancelled",EventKind::DamageCancelled),("c9b_end",EventKind::DamageEnd)]);
    let mut e = Engine::new_with_extensions(12071, 3, &[&p]).unwrap();
    for name in ["c9b_cancel", "c9b_cancelled", "c9b_end"] {
        e.debug_grant(1, name);
    }
    let h = e.debug_hand(1).len();
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    let s = e.save();
    let root = s
        .resolution
        .stack
        .iter()
        .find(|f| matches!(f.data, EventData::Damage(_)))
        .unwrap();
    assert_eq!(root.damage().settlement.committed_damage, Some(2));
    assert_eq!(root.damage().settlement.outcome, DamageOutcome::Cancelled);
    assert!(s
        .resolution
        .stack
        .iter()
        .any(|f| matches!(f.data, EventData::DamageWindow { .. })
            && f.kind == EventKind::DamageCancelled));
    let mut r = restore(&e, &p);
    answer(&mut e, vec![ChoiceOption::Confirm]);
    answer(&mut r, vec![ChoiceOption::Confirm]);
    same(&e, &r);
    assert_eq!(e.debug_hp(1), 2);
    assert_eq!(e.debug_hand(1).len(), h + 1);
}
#[test]
fn c9b_replacement_waits_for_real_lose_hp_and_dying_children_across_restore() {
    let p=package("skill c9b_unused(99109181) { trigger damage owner_is_target { effect draw_cards(self,9); } }",&[]);
    let mut e = Engine::new_with_extensions(12081, 3, &[&p]).unwrap();
    e.debug_grant(0, "绝情");
    e.debug_grant(1, "c9b_unused");
    e.debug_set_hp(1, 1);
    for player in 0..3 {
        e.debug_clear_hand(player);
    }
    e.debug_move_named_card_to_hand(0, "tao");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dying_save");
    let s = e.save();
    let root = s
        .resolution
        .stack
        .iter()
        .find(|f| matches!(f.data, EventData::Damage(_)))
        .unwrap();
    assert_eq!(
        root.damage().settlement.outcome,
        DamageOutcome::ReplacedByLoseHp
    );
    assert_eq!(root.damage().settlement.committed_damage, None);
    assert!(s
        .resolution
        .stack
        .iter()
        .any(|f| matches!(f.data, EventData::LoseHp { .. })));
    let mut r = restore(&e, &p);
    for game in [&mut e, &mut r] {
        while game.open_choice().is_some() {
            answer(game, vec![ChoiceOption::Pass]);
        }
    }
    same(&e, &r);
    assert!(!e.debug_alive(1));
    assert!(!e
        .log()
        .iter()
        .any(|x| matches!(x, engine_native::machine::LogEvent::DamageDealt { .. })));
}
#[test]
fn c9b_damage_constructor_sharing_preserves_existing_equipped_emit_entry() {
    use engine_native::{
        event::{DamageWindowKind, Stage},
        machine::Ctx,
        resolution::{push_frame, Resolution},
    };
    let e = Engine::new(12091, 3);
    let mut core = e.save().core;
    let mut res = Resolution::new(1000);
    let mut logs = Vec::new();
    let equipped = EventData::Equipped {
        player: 0,
        card: 0,
        replaced: None,
    };
    {
        let mut ctx = Ctx {
            core: &mut core,
            res: &mut res,
            catalog: e.catalog(),
            out: &mut logs,
        };
        ctx.emit(equipped.clone());
    }
    let f = &res.stack[0];
    assert_eq!(
        f.stage,
        Stage::Commit,
        "Ctx.emit non-damage entry keeps its original semantics"
    );
    assert_eq!(f.parent, None);
    assert_eq!(f.id, 1);
    assert_eq!(f.data, equipped);
    assert!(!f.planned && !f.intrinsic_done && f.queue.is_empty());
    let mut entry = Resolution::new(1000);
    push_frame(&mut entry, equipped);
    assert_eq!(
        entry.stack[0].stage,
        Stage::BeforeTriggers,
        "engine push entry keeps its existing equipped semantics"
    );
    {
        let mut ctx = Ctx {
            core: &mut core,
            res: &mut res,
            catalog: e.catalog(),
            out: &mut logs,
        };
        ctx.emit(EventData::Damage(engine_native::machine::damage_data(
            Some(0),
            1,
            1,
        )));
        ctx.emit(EventData::DamageWindow {
            root: 2,
            window: DamageWindowKind::Before,
        });
    }
    assert_eq!(res.stack[1].stage, Stage::Intrinsic);
    assert_eq!(res.stack[1].parent, Some(1));
    assert_eq!(res.stack[2].stage, Stage::Commit);
    assert_eq!(res.stack[2].parent, Some(2));
    assert_eq!(res.next_event_id, 4);
}
#[test]
fn c9c_first_prevention_competes_before_native_jueqing_replacement() {
    let mut p=package("skill c9c_first(99109201) { trigger damage_before owner_is_target { effect prevent_current_damage(); } }",&[]);
    p.vm_skills[0].trigger.timing = engine_native::event::Timing::First;
    let mut e = Engine::new_with_extensions(12101, 3, &[&p]).unwrap();
    e.debug_grant(0, "绝情");
    e.debug_grant(1, "c9c_first");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(
        e.debug_hp(1),
        4,
        "First prevention must run before ordinary native Jueqing"
    );
    assert!(!e
        .log()
        .iter()
        .any(|event| matches!(event, engine_native::machine::LogEvent::HpLost { .. })));
}
#[test]
fn c9c_window_intrinsic_and_before_stages_are_rejected_on_byte_restore() {
    let p=package("skill c9c_pause(99109202) { trigger damage_before owner_is_target optional { effect draw_cards(self,1); } }",&[]);
    let mut e = Engine::new_with_extensions(12102, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9c_pause");
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    let original = e.save();
    for stage in [
        engine_native::event::Stage::Intrinsic,
        engine_native::event::Stage::BeforeTriggers,
    ] {
        let mut bad = original.clone();
        let id = bad
            .resolution
            .stack
            .iter_mut()
            .find_map(|f| {
                if matches!(f.data, EventData::DamageWindow { .. }) {
                    f.stage = stage;
                    f.queue.clear();
                    f.planned = false;
                    f.intrinsic_done = false;
                    Some(f.id)
                } else {
                    None
                }
            })
            .unwrap();
        bad.resolution.parks.retain(
            |p| !matches!(p.slot,engine_native::resolution::Slot::Trigger{frame}if frame==id),
        );
        bad.resolution.open_choice = None;
        let decoded =
            engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&bad))
                .unwrap();
        assert!(
            Engine::restore_with_extensions(&decoded, &[&p]).is_err(),
            "impossible window stage {stage:?}"
        );
        assert_eq!(e.save(), original);
    }
    let mut missing = original.clone();
    missing.resolution.parks.retain(|park| {
        !matches!(
            park.machine,
            engine_native::snap::MachineSnapshot::TriggerOrder { .. }
        )
    });
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&missing))
            .unwrap();
    assert!(Engine::restore_with_extensions(&decoded, &[&p]).is_err());
    assert_eq!(e.save(), original);
}
#[test]
fn c9c_first_quantity_change_is_seen_by_native_zuijiu() {
    let mut p=package("skill c9c_num(99109203) { trigger damage_before owner_is_target { effect modify_current_damage(self,-1); } }",&[("c9c_num",EventKind::DamageBegin1)]);
    p.vm_skills[0].trigger.timing = engine_native::event::Timing::First;
    let mut e = Engine::new_with_extensions(12103, 3, &[&p]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_grant(0, "醉酒");
    e.debug_grant(1, "c9c_num");
    e.debug_clear_hand(1);
    let card = e.debug_place(0, "sha");
    let h = e.debug_hp(1);
    answer_play(&mut e, card);
    assert_eq!(
        e.debug_hp(1),
        h,
        "First num1→0 stops ordinary Zuijiu; it cannot pre-increment num before First"
    );
}
fn answer_play(e: &mut Engine, card: u32) {
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
}
#[test]
fn c9c_native_admitted_body_roundtrips_and_cannot_forge_another_definition() {
    let p=package("skill c9c_unowned(99109204) { trigger damage owner_is_target { effect draw_cards(self,1); } }",&[]);
    let mut e = Engine::new_with_extensions(12104, 3, &[&p]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_grant(0, "绝情");
    e.debug_clear_hand(1);
    let card = e.debug_place(0, "sha");
    e.set_time_slice(Some(1));
    answer_play(&mut e, card);
    let mut branch = None;
    for _ in 0..200 {
        let s = e.save();
        if s.resolution.parks.iter().any(|p| {
            matches!(
                p.machine,
                engine_native::snap::MachineSnapshot::NativeDamageSkill { .. }
            )
        }) {
            let mut r = restore(&e, &p);
            r.set_time_slice(None);
            r.resume();
            branch = Some(r);
            let mut nested = s.clone();
            let event = nested.resolution.stack.last().unwrap().data.clone();
            for park in &mut nested.resolution.parks {
                if let engine_native::snap::MachineSnapshot::NativeDamageSkill {
                    owner, def, ..
                } = &park.machine
                {
                    park.machine = engine_native::snap::MachineSnapshot::OptionalGate {
                        owner: *owner,
                        def: *def,
                        event: event.clone(),
                        asked: true,
                        inner: Some(Box::new(park.machine.clone())),
                    };
                }
            }
            assert!(Engine::restore_with_extensions(&nested, &[&p]).is_err());
            let mut invalid = s.clone();
            for park in &mut invalid.resolution.parks {
                if let engine_native::snap::MachineSnapshot::NativeDamageSkill { def, .. } =
                    &mut park.machine
                {
                    *def = 0;
                }
            }
            let bytes = engine_native::archive::encode_archive(&invalid);
            let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
            assert!(Engine::restore_with_extensions(&decoded, &[&p]).is_err());
            assert_eq!(e.save(), s);
        }
        if s.resolution.stack.is_empty() && !s.segment_open {
            break;
        }
        e.resume();
    }
    assert!(branch.is_some());
    same(&e, &branch.unwrap());
    assert_eq!(e.debug_hp(1), 3);
    assert_eq!(
        e.log()
            .iter()
            .filter(|x| matches!(x, engine_native::machine::LogEvent::HpLost { .. }))
            .count(),
        1
    );
}
#[test]
fn c9c_damage_record_precedes_each_recipients_change_hp_unlink() {
    let mut e = Engine::new(12105, 3);
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    // 首目标：伤害记录先于其 changeHp 解链；传导目标：上游顺序先解链、再进入传导
    // 伤害提交（2026-11-02 裁決，与 chain_fire_propagation golden 一致）。
    let dealt = |player: u8| {
        e.log()
            .iter()
            .position(|event| matches!(event, engine_native::machine::LogEvent::DamageDealt { target, .. } if *target == player))
            .unwrap()
    };
    let unlink = |player: u8| {
        e.log()
            .iter()
            .position(|event| matches!(event, engine_native::machine::LogEvent::Linked { player: p, linked: false } if *p == player))
            .unwrap()
    };
    assert!(dealt(1) < unlink(1), "first recipient keeps damage-then-unlink");
    assert!(unlink(2) < dealt(2), "chain recipient unlinks before its damage record");
}

#[test]
fn c9d_first_zero_precedes_huoshen_cancel_and_recover() {
    let mut p = package("skill c9d_first(99109301) { trigger damage_before owner_is_target { effect modify_current_damage(self,-1); } }", &[("c9d_first", EventKind::DamageBegin1)]);
    p.vm_skills[0].trigger.timing = engine_native::event::Timing::First;
    let mut e = Engine::new_with_extensions(12901, 3, &[&p]).unwrap();
    e.debug_grant(1, "火神");
    e.debug_grant(1, "c9d_first");
    e.debug_set_hp(1, 2);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert_eq!(
        e.debug_hp(1),
        2,
        "First zero stops Huoshen before cancel/recover"
    );
}

#[test]
fn c9d_applied_helper_uses_recipient_lifetime_without_parent_ownership() {
    let mut e = Engine::new(12902, 3);
    e.debug_set_max_hp(1, 20);
    e.debug_set_hp(1, 20);
    let parent = e.catalog().skill_by_name("host:xianjian_chengxin");
    assert!(parent.is_none_or(|def| !e.state_core().has_skill_any_state(1, def)));
    for active in [true, false, true] {
        e.state_core_mut().player_mut(1).chengxin_active = active;
        let hp = e.debug_hp(1);
        e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
        assert_eq!(e.debug_hp(1), hp - if active { 0 } else { 1 });
    }
}

#[test]
fn c9d_native_helper_order_choice_and_qualified_restore_execute_once() {
    let p=package("skill c9d_peer(99109302) { trigger damage_before owner_is_target { effect draw_cards(self,1); } }",&[]);
    let mut e = Engine::new_with_extensions(12903, 3, &[&p]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_grant(1, "c9d_peer");
    e.state_core_mut().player_mut(1).chengxin_active = true;
    e.debug_clear_hand(1);
    let card = e.debug_place(0, "sha");
    answer_play(&mut e, card);
    let choice = e
        .open_choice()
        .expect("same-owner actual helper and peer compete");
    assert_eq!(choice.spec.prompt, "trigger_order");
    let def = e
        .catalog()
        .skill_by_name("native:liuli/xianjian/chengxin2")
        .unwrap();
    assert!(choice.spec.options.contains(&ChoiceOption::Skill(def)));
    let mut choice_branch = restore(&e, &p);
    answer(&mut choice_branch, vec![ChoiceOption::Skill(def)]);
    e.set_time_slice(Some(1));
    answer(&mut e, vec![ChoiceOption::Skill(def)]);
    let mut admitted = false;
    for _ in 0..200 {
        let s = e.save();
        if s.resolution.parks.iter().any(|park|matches!(park.machine,engine_native::snap::MachineSnapshot::NativeDamageSkill{def:d,..}if d==def)) {
            admitted=true;
            let mut branch=restore(&e,&p);
            branch.set_time_slice(None);branch.resume();
            for mutation in 0..7 {
                let mut bad=s.clone();
                for park in &mut bad.resolution.parks {
                    if let engine_native::snap::MachineSnapshot::NativeDamageSkill{owner,def:machine_def,window}=&mut park.machine {
                        match mutation {
                            0=>*owner=2,
                            1=>*machine_def=e.catalog().skill_by_name("native:liuli/ow/jidong2").unwrap(),
                            2=>*window=engine_native::event::DamageWindowKind::Begin1,
                            3=>park.resume_depth+=1,
                            4=>park.slot=engine_native::resolution::Slot::Intrinsic{frame:match park.slot{engine_native::resolution::Slot::Trigger{frame}=>frame,_=>unreachable!()}},
                            _=>{},
                        }
                    }
                }
                if mutation==5 {bad.core.players[1].chengxin_active=false;}
                if mutation==6 {
                    for frame in &mut bad.resolution.stack {if matches!(frame.data,EventData::DamageWindow{..}){frame.stage=engine_native::event::Stage::Commit;}}
                }
                let decoded=engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&bad)).unwrap();
                assert!(Engine::restore_with_extensions(&decoded,&[&p]).is_err(),"helper forgery {mutation}");
                assert_eq!(e.save(),s,"negative restore is atomic");
            }
            e.set_time_slice(None);e.resume();same(&e,&branch);same(&e,&choice_branch);
            break;
        }
        e.resume();
    }
    assert!(admitted);
    assert_eq!(e.debug_hp(1), 4);
    assert_eq!(
        e.log()
            .iter()
            .filter(|event| matches!(
                event,
                engine_native::machine::LogEvent::DamagePrevented { target: 1 }
            ))
            .count(),
        1
    );
}

#[test]
fn c9d_huoshen_real_boss_general_uses_the_same_queue() {
    let mut e = Engine::new_boss_stage(84001, 4, 1, 3, 6, 4, 0);
    let general = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 41) as u16;
    e.debug_set_general_def(1, general);
    e.debug_set_hp(1, 4);
    let def = e.catalog().skill_by_name("火神").unwrap();
    assert!(e.state_core().has_skill(1, def));
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Fire);
    assert_eq!(e.debug_hp(1), 5);
}

#[test]
fn c9d_first_zero_precedes_applied_mailun_quantity_bonus() {
    let mut p=package("skill c9d_first_begin(99109303) { trigger damage_before owner_is_target { effect modify_current_damage(self,-1); } }",&[("c9d_first_begin",EventKind::DamageBegin)]);
    p.vm_skills[0].trigger.timing = engine_native::event::Timing::First;
    let mut e = Engine::new_with_extensions(12904, 3, &[&p]).unwrap();
    e.debug_grant(1, "c9d_first_begin");
    e.state_core_mut().player_mut(1).mailun_mode = 4;
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    assert_eq!(e.debug_hp(1), 4);
    assert!(
        !e.state_core().player(1).mailun_in_used,
        "stopped downstream native bonus is not consumed"
    );
}

#[test]
fn c9d_huoshen_body_finishes_before_recover_child_restore() {
    let p=package("skill c9d_unowned(99109304) { trigger damage_before owner_is_target { effect draw_cards(self,1); } }",&[]);
    let mut e = Engine::new_with_extensions(12905, 3, &[&p]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_set_hp(1, 2);
    e.debug_grant(0, "招祸");
    e.debug_grant(1, "火神");
    e.debug_clear_hand(1);
    let card = e.debug_place(0, "sha");
    e.set_time_slice(Some(1));
    answer_play(&mut e, card);
    let mut found = false;
    for _ in 0..300 {
        let snap = e.save();
        if snap
            .resolution
            .stack
            .iter()
            .any(|frame| matches!(frame.data, EventData::Recover { player: 1, .. }))
        {
            found = true;
            assert_eq!(e.debug_hp(1), 3);
            assert!(
                !snap.resolution.parks.iter().any(|park| matches!(
                    park.machine,
                    engine_native::snap::MachineSnapshot::NativeDamageSkill { .. }
                )),
                "atomic native body finishes before its child"
            );
            let mut restored = restore(&e, &p);
            restored.set_time_slice(None);
            restored.resume();
            e.set_time_slice(None);
            e.resume();
            same(&e, &restored);
            break;
        }
        e.resume();
    }
    assert!(found, "actual Recover child was observed");
    assert_eq!(
        e.debug_hp(1),
        3,
        "restore never repeats the native recovery"
    );
    assert_eq!(
        e.log()
            .iter()
            .filter(|event| matches!(
                event,
                engine_native::machine::LogEvent::SkillTriggered {
                    skill: "火神",
                    owner: 1
                }
            ))
            .count(),
        1
    );
}
