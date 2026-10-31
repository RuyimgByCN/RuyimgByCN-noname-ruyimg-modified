use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::{DamageNature, DamageProgress, EventData, EventKind};
use engine_native::{CardKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand};

fn source_pack(name: &str) -> ExtensionPack {
    let path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions")
        .join(name)
        .join("assets/cards.json");
    content_native_loader::bridge_pack(&std::fs::read_to_string(path).unwrap())
        .unwrap()
        .pack
}
fn vm_pack(source: &str, kind: EventKind, timing: engine_native::Timing) -> ExtensionPack {
    let raw = content_compiler::compile_source_named("c9e", source).unwrap();
    let mut vm_skills = content_native_loader::bridge_vm_skill_decls(&raw)
        .unwrap()
        .decls;
    vm_skills[0].trigger.event = kind;
    vm_skills[0].trigger.timing = timing;
    ExtensionPack {
        def: ExtensionDef {
            name: "c9e",
            version: 1,
            description: "source predicate cases",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn equip(e: &mut Engine, kind: CardKind) {
    let card = (0..e.catalog().cards.len() as u32)
        .find(|&card| e.catalog().card_kind(card) == kind)
        .unwrap();
    let name = e.catalog().cards[card as usize].name;
    let card = e.debug_place(0, name);
    e.state_core_mut().player_mut(0).hand.retain(|c| *c != card);
    e.state_core_mut().player_mut(0).equips.push(card);
}
fn card(e: &Engine, kind: CardKind) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|&c| e.catalog().card_kind(c) == kind)
        .unwrap()
}
fn play(e: &mut Engine, card: u32) {
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1,
        action: GameAction::PlayCard {
            card,
            targets: vec![1],
        },
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
}
#[test]
fn c9e_ciqiu_first_begin1_zero_keeps_skill_and_target_health() {
    let source = source_pack("wudian-upstream-noname");
    let vm=vm_pack("skill c9e_first(99109401) { trigger damage_before owner_is_target { effect modify_current_damage(self,-1); } }",EventKind::DamageBegin1,engine_native::Timing::First);
    let mut e = Engine::new_with_extensions(13001, 3, &[&source, &vm]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_clear_hand(1);
    e.debug_grant(0, "ciqiu");
    e.debug_grant(1, "c9e_first");
    e.debug_set_max_hp(1, 2);
    e.debug_set_hp(1, 2);
    let def = e.catalog().skill_by_name("ciqiu").unwrap();
    let c = e.debug_place(0, "sha");
    play(&mut e, c);
    assert_eq!(
        e.debug_hp(1),
        2,
        "First Begin1 must precede Ciqiu increment/removal"
    );
    assert!(e.state_core().has_skill(0, def));
    assert!(e.state_core().turn.ciqiu_dying.is_empty());
}
#[test]
fn c9e_ciqiu_elemental_sha_executes_admitted_quantity_and_removal() {
    let source = source_pack("wudian-upstream-noname");
    for kind in [CardKind::FireSha, CardKind::LeiSha] {
        let mut e = Engine::new_with_extensions(13002, 3, &[&source]).unwrap();
        e.debug_grant(0, "ciqiu");
        e.debug_set_max_hp(1, 2);
        e.debug_set_hp(1, 2);
        let c = card(&e, kind);
        e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Fire, c);
        assert_eq!(
            e.debug_hp(1),
            0,
            "{kind:?} must execute the admitted Sha-family body"
        );
        assert!(!e
            .state_core()
            .has_skill_any_state(0, e.catalog().skill_by_name("ciqiu").unwrap()));
    }
}
#[test]
fn c9e_xixue_elemental_sha_executes_admitted_recovery() {
    let source = source_pack("shahai-swd-upstream-noname");
    for kind in [CardKind::FireSha, CardKind::LeiSha] {
        let mut e = Engine::new_with_extensions(13003, 3, &[&source]).unwrap();
        equip(&mut e, CardKind::Xixueguizhihuan);
        e.debug_set_hp(0, 2);
        let c = card(&e, kind);
        e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Fire, c);
        assert_eq!(
            e.debug_hp(0),
            3,
            "{kind:?} must execute the admitted Sha-family recovery"
        );
    }
}
#[test]
fn c9e_chilongya_direct_linked_victim_differs_from_real_chain_child() {
    let source = source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(13004, 3, &[&source]).unwrap();
    equip(&mut e, CardKind::Chilongya);
    for p in [1, 2] {
        e.state_core_mut().player_mut(p).linked = true;
    }
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Fire);
    assert_eq!(
        e.debug_hp(1),
        2,
        "linked direct victim still receives the source bonus"
    );
    assert_eq!(
        e.debug_hp(2),
        2,
        "propagated child carries2 without applying source bonus again"
    );
    let amounts: Vec<_> = e
        .log()
        .iter()
        .filter_map(|x| {
            if let engine_native::machine::LogEvent::DamageDealt { target, amount, .. } = x {
                Some((*target, *amount))
            } else {
                None
            }
        })
        .collect();
    assert_eq!(amounts, vec![(1, 2), (2, 2)]);
}
#[test]
fn c9e_zuijiu_card_without_immediate_sha_parent_does_not_qualify() {
    let mut e = Engine::new(13005, 3);
    e.debug_grant(0, "醉酒");
    let c = card(&e, CardKind::Sha);
    e.debug_deal_damage_with_card(Some(0), 1, 1, DamageNature::Normal, c);
    assert_eq!(
        e.debug_hp(1),
        3,
        "card identity alone does not prove an immediate Sha parent"
    );
}
fn hutian_deposit(e: &mut Engine) {
    let cards: Vec<_> = e.state_core().deck.iter().copied().take(3).collect();
    assert_eq!(cards.len(), 3);
    e.state_core_mut().deck.retain(|c| !cards.contains(c));
    e.state_core_mut().player_mut(2).hutian_deposit = Some(1);
    let p = e.state_core_mut().player_mut(1);
    p.hutian_owner = Some(2);
    p.hutian_hold = cards;
}
#[test]
fn c9e_hutian_negative_cap_is_persisted_before_zero_transition() {
    let mut e = Engine::new(13006, 3);
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 2);
    hutian_deposit(&mut e);
    let c = e.debug_place(0, "sha");
    e.set_time_slice(Some(1));
    play(&mut e, c);
    let mut found = false;
    for _ in 0..300 {
        if let Some(d) = e.save().resolution.stack.iter().find_map(|f| {
            if let EventData::Damage(d) = &f.data {
                Some(d.clone())
            } else {
                None
            }
        }) {
            if d.amount <= 0 && d.settlement.progress == DamageProgress::Begin {
                found = true;
                assert_eq!(
                    d.amount, -1,
                    "source hp2-deposit3 is signed -1, never clamp tozero"
                );
                break;
            }
        }
        e.resume();
    }
    assert!(found, "actual native Hutian body was observed");
}

fn capture(e: &Engine, name: &str) {
    let Ok(directory) = std::env::var("C331_PREDICATE_TRACE_DIR") else {
        return;
    };
    let dir = std::path::Path::new(&directory);
    std::fs::create_dir_all(dir).unwrap();
    let s = e.save();
    let views: Vec<_> = (0..s.core.players.len() as u8)
        .map(|p| e.view_for(p))
        .collect();
    let choices: Vec<_> = (0..s.core.players.len() as u8)
        .map(|p| e.choice_view_for(Some(p)))
        .collect();
    let value = serde_json::json!({"case":name,"core":format!("{:#?}",s.core),"events":format!("{:#?}",e.log()),"rng":format!("{:?}",s.core.rng),"views":format!("{views:#?}"),"choices":format!("{choices:#?}"),"referee":format!("{:#?}",e.choice_view_referee()),"resolution":format!("{:#?}",s.resolution),"state_hash":e.state_hash()});
    std::fs::write(
        dir.join(format!("{name}.json")),
        serde_json::to_vec_pretty(&value).unwrap(),
    )
    .unwrap();
    std::fs::write(dir.join(format!("{name}.snapshot.txt")), format!("{s:#?}")).unwrap();
    std::fs::write(
        dir.join(format!("{name}.archive.ngsa")),
        engine_native::archive::encode_archive(&s),
    )
    .unwrap();
}
fn restore(e: &Engine, packs: &[&ExtensionPack]) -> Engine {
    let saved = e.save();
    let bytes = engine_native::archive::encode_archive(&saved);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert_eq!(saved, decoded);
    let typed = Engine::restore_with_extensions(&saved, packs).unwrap();
    let restored = Engine::restore_with_extensions(&decoded, packs).unwrap();
    let mut normalized = restored.save();
    normalized.auth_generation = saved.auth_generation;
    assert_eq!(normalized, saved);
    assert_eq!(typed.state_hash(), restored.state_hash());
    restored
}
fn finish(e: &mut Engine) {
    e.set_time_slice(None);
    for _ in 0..500 {
        if let Some(c) = e.open_choice().cloned() {
            let selection = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else if c.spec.allow_cancel {
                vec![]
            } else {
                c.spec
                    .options
                    .iter()
                    .take(c.spec.min as usize)
                    .copied()
                    .collect()
            };
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
        } else if e.save().resolution.stack.is_empty() && !e.save().segment_open {
            return;
        } else {
            e.resume();
        }
    }
    panic!("continuation did not finish");
}
fn same_final(expected: &Engine, restored: &Engine) {
    let mut a = expected.save();
    let mut b = restored.save();
    a.auth_generation = 0;
    b.auth_generation = 0;
    assert_eq!(a, b);
    assert_eq!(expected.state_hash(), restored.state_hash());
    assert!(
        expected.log().ends_with(restored.log()),
        "full log/RNG/cost/child suffix must agree"
    );
}
fn sliced(mut e: Engine, packs: &[&ExtensionPack], card: u32, name: &str) -> Engine {
    let mut expected = e.clone();
    play(&mut expected, card);
    finish(&mut expected);
    e.set_time_slice(Some(1));
    play(&mut e, card);
    let mut count = 0;
    for _ in 0..500 {
        if e.save().resolution.stack.is_empty() && !e.save().segment_open {
            break;
        }
        assert!(
            e.open_choice().is_none(),
            "fixture must have no discretionary response"
        );
        if e.save()
            .resolution
            .stack
            .iter()
            .any(|f| matches!(&f.data,EventData::Damage(d)if d.amount<0))
        {
            capture(&e, &format!("{name}-negative-{count}"));
        }
        let mut restored = restore(&e, packs);
        finish(&mut restored);
        same_final(&expected, &restored);
        count += 1;
        if e.save().resolution.parks.iter().any(|p| {
            matches!(
                p.machine,
                engine_native::snap::MachineSnapshot::NativeDamageSkill { .. }
            )
        }) {
            capture(&e, &format!("{name}-native-{count}"));
        }
        e.resume();
    }
    assert!(count > 0);
    same_final(&expected, &e);
    capture(&e, &format!("{name}-final"));
    e
}
#[test]
fn c9e_real_sha_chain_and_alias_parent_roundtrip_at_every_slice() {
    let mut aliases = source_pack("shahai-swd-upstream-noname");
    aliases.cards.push(engine_native::catalog::CardDef::basic(
        "c9e_sha_alias",
        CardKind::FireSha,
    ));
    for name in ["sha", "fire_sha", "leisha", "c9e_sha_alias"] {
        let mut e = Engine::new_with_extensions(13007, 3, &[&aliases]).unwrap();
        e.debug_set_turn(0, engine_native::Phase::Play, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(0, "醉酒");
        for p in [1, 2] {
            e.state_core_mut().player_mut(p).linked = true;
        }
        let c = e.debug_place(0, name);
        let e = sliced(e, &[&aliases], c, name);
        assert_eq!(
            e.debug_hp(1),
            2,
            "direct actual Sha parent qualifies {name}"
        );
        assert_eq!(
            e.debug_hp(2),
            if name == "sha" { 4 } else { 2 },
            "independent same-card chain never requalifies through its Sha ancestor"
        );
    }
}
#[test]
fn c9e_elemental_ciqiu_and_xixue_roundtrip_before_body_and_children() {
    let swd = source_pack("shahai-swd-upstream-noname");
    let diy = source_pack("wudian-upstream-noname");
    for name in ["fire_sha", "leisha"] {
        for ciqiu in [true, false] {
            let mut e = Engine::new_with_extensions(13008, 3, &[&swd, &diy]).unwrap();
            e.debug_set_turn(0, engine_native::Phase::Play, 1);
            for p in 0..3 {
                e.debug_clear_hand(p);
            }
            e.debug_set_hp(0, 2);
            e.debug_set_max_hp(1, 8);
            e.debug_set_hp(1, 8);
            if ciqiu {
                e.debug_grant(0, "ciqiu");
            } else {
                equip(&mut e, CardKind::Xixueguizhihuan);
            }
            let c = e.debug_place(0, name);
            let e = sliced(e, &[&swd, &diy], c, &format!("{name}-{ciqiu}"));
            assert_eq!(e.debug_hp(1), if ciqiu { 6 } else { 7 });
            assert_eq!(e.debug_hp(0), if ciqiu { 2 } else { 3 });
        }
    }
}
#[test]
fn c9e_chilongya_equipment_qualified_chain_roundtrips() {
    let swd = source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(13009, 3, &[&swd]).unwrap();
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    equip(&mut e, CardKind::Chilongya);
    for p in [1, 2] {
        e.state_core_mut().player_mut(p).linked = true;
    }
    let c = e.debug_place(0, "fire_sha");
    let e = sliced(e, &[&swd], c, "chilongya-chain");
    assert_eq!(e.debug_hp(1), 2);
    assert_eq!(e.debug_hp(2), 2);
}

fn reject_restore_atomic(
    e: &Engine,
    packs: &[&ExtensionPack],
    bad: &engine_native::snap::StateSnapshot,
) {
    let original = e.save();
    let bytes = engine_native::archive::encode_archive(bad);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    assert!(Engine::restore_with_extensions(bad, packs).is_err());
    assert!(Engine::restore_with_extensions(&decoded, packs).is_err());
    assert_eq!(e.save(), original);
}
#[test]
fn c9e_zuijiu_admitted_parent_and_source_forgeries_reject_atomically() {
    let mut e = Engine::new(13010, 3);
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "醉酒");
    let c = e.debug_place(0, "sha");
    let def = e.catalog().skill_by_name("醉酒").unwrap();
    e.set_time_slice(Some(1));
    play(&mut e, c);
    let mut found = false;
    for _ in 0..300 {
        let s = e.save();
        if s.resolution.parks.iter().any(|p|matches!(p.machine,engine_native::snap::MachineSnapshot::NativeDamageSkill{def:d,..}if d==def)) {
            found=true;capture(&e,"zuijiu-admitted-parent");
            let root=s.resolution.stack.iter().position(|f|matches!(f.data,EventData::Damage(_))).unwrap();let parent=root-1;
            for case in 0..6 {
                let mut bad=s.clone();
                match case {
                    0=>bad.resolution.stack[root].parent=None,
                    1=>if let EventData::Damage(d)=&mut bad.resolution.stack[root].data{d.from_chain=true;},
                    2=>if let EventData::CardUsed(u)=&mut bad.resolution.stack[parent].data{u.use_as=Some(CardKind::Duel);},
                    3=>bad.resolution.stack[parent].stage=engine_native::Stage::AfterTriggers,
                    4=>if let EventData::Damage(d)=&mut bad.resolution.stack[root].data{d.card=None;},
                    5=>if let EventData::Damage(d)=&mut bad.resolution.stack[root].data{d.source=Some(2);},
                    _=>unreachable!(),
                }
                reject_restore_atomic(&e,&[],&bad);
            }
            let mut restored=restore(&e,&[]);finish(&mut restored);finish(&mut e);same_final(&e,&restored);break;
        }
        e.resume();
    }
    assert!(found);
    assert_eq!(e.debug_hp(1), 2);
}
#[test]
fn c9e_chain_cause_cannot_be_forged_into_ancestor_sha_admission() {
    let mut e = Engine::new(13011, 3);
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "醉酒");
    for p in [1, 2] {
        e.state_core_mut().player_mut(p).linked = true;
    }
    let c = e.debug_place(0, "fire_sha");
    e.set_time_slice(Some(1));
    play(&mut e, c);
    let mut found = false;
    for _ in 0..400 {
        let s = e.save();
        if let Some(index) = s
            .resolution
            .stack
            .iter()
            .position(|f| matches!(&f.data,EventData::Damage(d)if d.from_chain&&d.target==2))
        {
            found = true;
            capture(&e, "same-card-chain-immediate-parent");
            let ancestor = s
                .resolution
                .stack
                .iter()
                .find(|f| matches!(f.data, EventData::CardUsed(_)))
                .unwrap()
                .id;
            for skip_parent in [false, true] {
                let mut bad = s.clone();
                if let EventData::Damage(d) = &mut bad.resolution.stack[index].data {
                    d.from_chain = false;
                }
                if skip_parent {
                    bad.resolution.stack[index].parent = Some(ancestor);
                }
                reject_restore_atomic(&e, &[], &bad);
            }
            let mut restored = restore(&e, &[]);
            finish(&mut restored);
            finish(&mut e);
            same_final(&e, &restored);
            break;
        }
        e.resume();
    }
    assert!(found);
    assert_eq!(e.debug_hp(2), 2);
}
#[test]
fn c9e_hutian_signed_zero_host_amount_and_later_filters_roundtrip() {
    let mut vm=vm_pack("skill c9e_zero(99109402) { trigger damage_before owner_is_target { if event_amount < 0 { effect draw_cards(self,0-event_amount); } } } skill c9e_late(99109403) { trigger damage_before owner_is_target { effect draw_cards(self,1); } }",EventKind::DamageZero,engine_native::Timing::Normal);
    vm.vm_skills[1].trigger.event = EventKind::DamageBegin;
    vm.vm_skills[1].trigger.timing = engine_native::Timing::Last;
    for hp in [2, 3, 4] {
        let mut e = Engine::new_with_extensions(13012, 3, &[&vm]).unwrap();
        e.debug_set_turn(0, engine_native::Phase::Play, 1);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        e.debug_grant(1, "c9e_zero");
        e.debug_grant(1, "c9e_late");
        e.debug_set_hp(1, hp);
        hutian_deposit(&mut e);
        let c = e.debug_place(0, "sha");
        let e = sliced(e, &[&vm], c, &format!("hutian-hp{hp}"));
        assert_eq!(e.debug_hp(1), if hp == 4 { 3 } else { hp });
        assert_eq!(e.debug_hand(1).len(),if hp==3{0}else{1},"negative signed host amount draws1; zero draws0; positive allows later ordinary handler");
        let dealt = e
            .log()
            .iter()
            .filter(|x| matches!(x, engine_native::machine::LogEvent::DamageDealt { .. }))
            .count();
        assert_eq!(dealt, if hp == 4 { 1 } else { 0 });
    }
}
