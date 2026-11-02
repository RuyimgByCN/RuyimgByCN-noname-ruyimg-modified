use super::c331_genuine_invocations::{answer, emitter, fire, fixture};
use super::c331_invocation_boundaries::qinglong_ready;
use super::c331_source_predicates::{
    capture, finish, play, reject_restore_atomic, restore, same_final,
};
use engine_native::event::{EventData, NativeInvocationProgress as P};
use engine_native::{
    ChoiceOption as O, CommandOutcome, Engine, GameAction, GameCommand, MachineSnapshot, Phase,
};
fn body(name: &str) -> (engine_native::content::extension::ExtensionPack, Engine) {
    let p = emitter();
    let mut e = fixture(&p, name);
    if name == "极点" {
        e.debug_grant(2, "极点");
    }
    e.set_time_slice(Some(1));
    fire(&mut e);
    for _ in 0..800 {
        if e.save().resolution.parks.iter().any(|p| {
            matches!(
                p.machine,
                MachineSnapshot::NativeInvocation {
                    progress: P::Body,
                    ..
                }
            )
        }) {
            return (p, e);
        }
        e.resume();
    }
    panic!("real body {name}");
}
#[test]
fn c9i_same_effect_terminal_forgery_rejected() {
    for name in ["急叮", "远策", "极点"] {
        let (p, e) = body(name);
        let _ = restore(&e, &[&p]);
        let mut bad = e.save();
        for park in &mut bad.resolution.parks {
            if let MachineSnapshot::NativeInvocation { progress, .. } = &mut park.machine {
                if *progress == P::Body {
                    *progress = if name == "急叮" {
                        P::SourceLoss
                    } else {
                        P::BodyDone
                    };
                }
            }
        }
        reject_restore_atomic(&e, &[&p], &bad);
    }
}
#[test]
fn c9i_jiding_child_payload_is_admitted_operation() {
    let (p, mut e) = body("急叮");
    for _ in 0..800 {
        let s = e.save();
        if let Some(f)=s.resolution.stack.iter().find(|f|matches!(&f.data,EventData::CardUsed(u)if u.use_as==Some(engine_native::CardKind::LeiSha))){let _=restore(&e,&[&p]);for variant in 0..4{let mut bad=s.clone();if let EventData::CardUsed(u)=&mut bad.resolution.stack.iter_mut().find(|x|x.id==f.id).unwrap().data{match variant{0=>u.targets=vec![3],1=>u.card=Some(0),2=>u.nature=engine_native::event::DamageNature::Normal,_=>u.player=3}}reject_restore_atomic(&e,&[&p],&bad);}return}
        e.resume();
    }
    panic!("actual emitted card child");
}
#[test]
fn c9i_qinglong_cancel_no_log_success_exactly_one() {
    for cancel in [true, false] {
        let (_, mut e, c) = qinglong_ready();
        play(&mut e, c);
        let logcount = |e: &Engine| {
            e.log()
                .iter()
                .filter(|x| {
                    matches!(
                        x,
                        engine_native::LogEvent::SkillTriggered {
                            skill: "native:liuli/card-swd/qinglonglingzhu",
                            ..
                        }
                    )
                })
                .count()
        };
        assert_eq!(logcount(&e), 0, "preselection has no source activation");
        answer(
            &mut e,
            if cancel {
                vec![]
            } else {
                vec![O::HiddenCard(1)]
            },
        );
        finish(&mut e);
        assert_eq!(logcount(&e), usize::from(!cancel));
    }
}
#[test]
fn c9i_qinglong_private_custody_at_loss_pause() {
    let (p, mut e, c) = qinglong_ready();
    play(&mut e, c);
    let hidden = e.debug_hand(1)[1];
    e.set_time_slice(Some(1));
    answer(&mut e, vec![O::HiddenCard(1)]);
    for _ in 0..800 {
        if e.save().resolution.parks.iter().any(|p| {
            matches!(
                p.machine,
                MachineSnapshot::NativeInvocation {
                    progress: P::QinglongLoss { .. },
                    ..
                }
            )
        }) {
            for seat in [2, 3] {
                assert!(
                    !e.view_for(seat).visible_card_ids().contains(&hidden),
                    "foreign hidden card stays private during loss subtree"
                );
                assert!(!e.view_for(seat).table.contains(&hidden));
            }
            assert!(!e.view_spectator().visible_card_ids().contains(&hidden));
            assert!(e.view_referee().visible_card_ids().contains(&hidden));
            let mut copy = restore(&e, &[&p]);
            capture(&e, "c9i-qinglong-custody");
            finish(&mut e);
            finish(&mut copy);
            same_final(&e, &copy);
            return;
        }
        e.resume();
    }
    panic!("actual loss pause");
}
#[test]
fn c9i_qinglong_theft_does_not_trigger_genuine_gift() {
    let (pack, _, c) = qinglong_ready();
    let watcher=super::c331_source_predicates::vm_pack("skill c9i_gift(99109701) locked { trigger cards_given owner_alive_gave_cards_to_other_alive { effect draw_cards(self,1); } }",engine_native::event::EventKind::CardsGiven,engine_native::Timing::Normal);
    let mut e = Engine::new_with_extensions(16009, 4, &[&pack, &watcher]).unwrap();
    for seat in 0..4 {
        e.debug_clear_hand(seat);
    }
    e.debug_set_turn(0, Phase::Play, 1);
    super::c331_source_predicates::equip(&mut e, engine_native::CardKind::Qinglonglingzhu);
    e.debug_grant(1, "c9i_gift");
    e.debug_place(1, "tao");
    e.debug_place(1, "sha");
    let attack = e.debug_place(0, "fire_sha");
    play(&mut e, attack);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "c331_qinglong_card");
    let before = e.debug_hand(1).len();
    answer(&mut e, vec![O::HiddenCard(1)]);
    finish(&mut e);
    assert_eq!(
        e.debug_hand(1).len(),
        before - 1,
        "robbed actor must not receive gift-trigger draw"
    );
    let _ = c;
}
#[test]
fn c9i_nested_source_death_preserves_captured_identity() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    e.debug_set_hp(2, 1);
    e.set_time_slice(Some(1));
    fire(&mut e);
    let mut checked = false;
    for _ in 0..2000 {
        if !e.state_core().player(2).alive
            && e.save()
                .resolution
                .stack
                .iter()
                .any(|f| matches!(&f.data,EventData::Damage(d)if d.source.is_none()))
            && e.save()
                .resolution
                .stack
                .iter()
                .any(|f| matches!(f.data, EventData::SkillInvocation(_)))
        {
            let mut copy = restore(&e, &[&p]);
            finish(&mut e);
            finish(&mut copy);
            same_final(&e, &copy);
            checked = true;
            break;
        }
        if e.open_choice().is_some() {
            answer(&mut e, vec![])
        } else {
            e.resume();
        }
    }
    assert!(
        checked,
        "actual nested Sha kills original source before outer invocation finishes"
    );
}
fn submit(e: &mut Engine, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor: e.state_core().turn.current,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    })
}
#[test]
fn c9i_yudun_transformed_material_does_not_consume_regular_sha() {
    let pack = super::c331_source_predicates::source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(16021, 4, &[&pack]).unwrap();
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, "yudun");
    let a = e.debug_place(0, "wuzhong");
    let b = e.debug_place(0, "snatch");
    let def = e.catalog().skill_by_name("yudun").unwrap();
    let out = submit(&mut e, GameAction::ActivateSkill { skill: def });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    answer(&mut e, vec![O::Card(a), O::Card(b)]);
    answer(&mut e, vec![O::Player(1)]);
    finish(&mut e);
    assert_eq!(e.debug_hp(1), 3);
    assert!(
        !e.state_core().turn.sha_used_play_phase,
        "physical trick transformed to Sha is exempt"
    );
    let c = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        GameAction::PlayCard {
            card: c,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "second actual Sha: {out:?}"
    );
    finish(&mut e);
    assert_eq!(e.debug_hp(1), 2);
    assert!(e.state_core().turn.sha_used_play_phase);
}
#[test]
fn c9i_shenmiguo_excludes_transformed_material_but_replays_physical_sha() {
    for transformed in [true, false] {
        let pack = super::c331_source_predicates::source_pack("shahai-swd-upstream-noname");
        let mut e = Engine::new_with_extensions(16022, 4, &[&pack]).unwrap();
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        e.debug_set_turn(0, Phase::Play, 1);
        let fruit_name = e
            .catalog()
            .cards
            .iter()
            .find(|c| c.kind == engine_native::CardKind::Shenmiguo)
            .unwrap()
            .name;
        let fruit = e.debug_place(0, fruit_name);
        let card = if transformed {
            e.debug_grant(0, "武圣");
            e.debug_place(0, "tao")
        } else {
            e.debug_place(0, "sha")
        };
        let out = submit(
            &mut e,
            GameAction::PlayCard {
                card,
                targets: vec![1],
            },
        );
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
        if transformed {
            assert!(
                e.open_choice().is_none(),
                "material replay cannot reconstruct transformed card semantics"
            );
            assert!(e.debug_hand(0).contains(&fruit));
            assert_eq!(e.debug_hp(1), 3);
        } else {
            assert_eq!(e.open_choice().unwrap().spec.prompt, "trigger_confirm");
            answer(&mut e, vec![O::Confirm]);
            finish(&mut e);
            assert_eq!(e.debug_hp(1), 2);
            assert!(!e.debug_hand(0).contains(&fruit));
        }
    }
}
#[test]
fn c9i_mixed_judge_red_replacement_pays_guicai_route() {
    let p = emitter();
    let mut e = fixture(&p, "远策");
    e.debug_grant(3, "鬼才");
    e.debug_grant(3, "鬼道");
    let red = e.debug_place(3, "tao");
    assert!(matches!(
        e.catalog().cards[red as usize].suit,
        engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
    ));
    fire(&mut e);
    answer(
        &mut e,
        vec![O::Control(engine_native::event::YuanceOption::LoseHp)],
    );
    if e.open_choice().unwrap().spec.prompt == "judge_rewrite_provider" {
        let def = e.catalog().skill_by_name("鬼才").unwrap();
        answer(&mut e, vec![O::Skill(def)]);
    }
    assert!(e
        .open_choice()
        .unwrap()
        .spec
        .options
        .contains(&O::Card(red)));
    let old = e
        .log()
        .iter()
        .rev()
        .find_map(|x| {
            if let engine_native::LogEvent::Judged { card, .. } = x {
                Some(*card)
            } else {
                None
            }
        })
        .unwrap();
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![O::Card(red)]);
        finish(branch);
        assert!(
            branch.state_core().discard.contains(&old),
            "only GuiCai admits red hand card, so its old-card discard route must be used"
        );
        assert!(!branch.debug_hand(3).contains(&old));
    }
    same_final(&e, &copy);
}
#[test]
fn c9i_qinglong_successful_same_index_worlds_are_private_at_every_pause() {
    let (_, mut a, c) = qinglong_ready();
    let (_, mut b, d) = qinglong_ready();
    b.state_core_mut().player_mut(1).hand.swap(0, 1);
    let pa = a.debug_hand(1)[1];
    let pb = b.debug_hand(1)[1];
    assert_ne!(pa, pb);
    a.set_time_slice(Some(1));
    b.set_time_slice(Some(1));
    play(&mut a, c);
    play(&mut b, d);
    let mut selected = false;
    let mut pauses = 0;
    for _ in 0..1600 {
        for seat in [2, 3] {
            assert_eq!(
                a.view_for(seat),
                b.view_for(seat),
                "public seat projection at pause {pauses}"
            );
            assert_eq!(a.choice_view_for(Some(seat)), b.choice_view_for(Some(seat)));
            assert_eq!(a.event_view_for(seat), b.event_view_for(seat));
        }
        assert_eq!(a.view_spectator(), b.view_spectator());
        assert_eq!(a.presentation_for(None), b.presentation_for(None));
        assert_eq!(
            format!("{:?}", a.state_core().rng),
            format!("{:?}", b.state_core().rng)
        );
        pauses += 1;
        if !selected && a.open_choice().is_some() {
            if let Ok(dir) = std::env::var("C9I_HOST_OUT") {
                let dir = std::path::Path::new(&dir);
                std::fs::create_dir_all(dir).unwrap();
                for (label, engine) in [("P", &a), ("Q", &b)] {
                    std::fs::write(
                        dir.join(format!("qinglong-{label}.ngsa.hex")),
                        engine_native::archive::encode_archive(&engine.save())
                            .iter()
                            .map(|b| format!("{b:02x}"))
                            .collect::<String>(),
                    )
                    .unwrap();
                }
                let raw = std::fs::read_to_string(
                    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(
                        "../../packages/extensions/shahai-swd-upstream-noname/assets/cards.json",
                    ),
                )
                .unwrap();
                std::fs::write(dir.join("packs.json"), format!("[{raw}]")).unwrap();
            }
            assert_eq!(a.open_choice().unwrap().spec, b.open_choice().unwrap().spec);
            answer(&mut a, vec![O::HiddenCard(1)]);
            answer(&mut b, vec![O::HiddenCard(1)]);
            selected = true;
        } else if a.save().resolution.stack.is_empty() && b.save().resolution.stack.is_empty() {
            break;
        } else {
            a.resume();
            b.resume();
        }
    }
    assert!(selected && pauses > 10);
    assert!(a.debug_hand(0).contains(&pa));
    assert!(b.debug_hand(0).contains(&pb));
    assert!(!a.debug_hand(0).contains(&pb));
    assert!(!b.debug_hand(0).contains(&pa));
}
#[test]
fn c9i_qinglong_public_equipment_occurrence_remains_public_after_gain() {
    let (_, mut e, c) = qinglong_ready();
    e.debug_set_turn(1, Phase::Play, 1);
    let eq = e.debug_place(1, "zhangba");
    let out = submit(
        &mut e,
        GameAction::PlayCard {
            card: eq,
            targets: vec![],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    e.debug_set_turn(0, Phase::Play, 1);
    play(&mut e, c);
    answer(&mut e, vec![O::Card(eq)]);
    finish(&mut e);
    for seat in [2, 3] {
        assert!(e.event_view_for(seat).iter().any(|v|matches!(v,engine_native::view::EventView::Public(engine_native::LogEvent::CardsTaken{cards,public:true,..})if cards==&vec![eq])));
    }
    assert!(e.presentation_for(None).iter().any(|v|matches!(v,engine_native::presentation::PresentationEvent::Given{cards,..}if cards==&vec!["zhangba".to_string()])));
}
#[test]
fn c9i_overlapping_judge_provider_is_selected_and_paid_route_survives_loss() {
    for name in ["鬼才", "鬼道"] {
        let p = emitter();
        let mut e = fixture(&p, "远策");
        e.debug_grant(3, "鬼才");
        e.debug_grant(3, "鬼道");
        let black = loop {
            let card = e.debug_place(3, "sha");
            if matches!(
                e.catalog().cards[card as usize].suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            ) {
                break card;
            }
        };
        fire(&mut e);
        answer(
            &mut e,
            vec![O::Control(engine_native::event::YuanceOption::LoseHp)],
        );
        assert_eq!(
            e.open_choice().unwrap().spec.prompt,
            "judge_rewrite_provider"
        );
        let provider = e.catalog().skill_by_name(name).unwrap();
        answer(&mut e, vec![O::Skill(provider)]);
        let mut copy = restore(&e, &[&p]);
        let old = e
            .log()
            .iter()
            .rev()
            .find_map(|x| {
                if let engine_native::LogEvent::Judged { card, .. } = x {
                    Some(*card)
                } else {
                    None
                }
            })
            .unwrap();
        for branch in [&mut e, &mut copy] {
            branch.set_time_slice(Some(1));
            answer(branch, vec![O::Card(black)]);
            for _ in 0..100 {
                if branch.save().resolution.parks.iter().any(|p| {
                    matches!(
                        p.machine,
                        MachineSnapshot::JudgeContinuation {
                            payment: Some(_),
                            ..
                        }
                    )
                }) {
                    branch.state_core_mut().skills.remove(&(3, provider));
                    let mut after = restore(branch, &[&p]);
                    finish(branch);
                    finish(&mut after);
                    same_final(branch, &after);
                    break;
                }
                branch.resume();
            }
            if name == "鬼才" {
                assert!(branch.state_core().discard.contains(&old));
            } else {
                assert!(branch.debug_hand(3).contains(&old));
            }
        }
        same_final(&e, &copy);
    }
}
#[test]
fn c9i_kingdoms_real_reveal_declares_distinct_enemy_facts() {
    let mut e = Engine::new(16031, 4);
    let general = |e: &Engine, faction| {
        (0..e.catalog().general_count())
            .find(|id| {
                e.catalog()
                    .general_meta(*id as u16)
                    .is_some_and(|g| g.1 == faction)
            })
            .unwrap() as u16
    };
    let wei = general(&e, engine_native::catalog::Faction::Wei);
    let shu = general(&e, engine_native::catalog::Faction::Shu);
    e.state_core_mut().mode = engine_native::state::GameMode::Kingdoms;
    e.debug_set_general_def(0, wei);
    e.debug_set_general_def(1, shu);
    for actor in [0, 1] {
        e.debug_set_turn(actor, Phase::Play, 1);
        let out = submit(&mut e, GameAction::RevealGeneral { slot: 0 });
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "real reveal {out:?}"
        );
    }
    assert_eq!(
        e.state_core().source_is_enemy_of(e.catalog(), 0, 1),
        Some(true),
        "source isEnemyOf is !isFriendOf for two distinct declared identities"
    );
    let _ = restore(&e, &[]);
}
#[test]
fn c9i_kingdom_identity_and_mutual_friend_facts_are_authoritative() {
    use engine_native::state::KingdomIdentity as K;
    let mut e = Engine::new(16032, 4);
    e.state_core_mut().mode = engine_native::state::GameMode::Kingdoms;
    for (p, id) in [
        (0, K::Declared("wei".into())),
        (1, K::Declared("shu".into())),
        (2, K::Unknown),
        (3, K::Ye),
    ] {
        e.state_core_mut().set_kingdom_identity(p, id).unwrap();
    }
    for p in 1..4 {
        assert_eq!(
            e.state_core().source_is_enemy_of(e.catalog(), 0, p),
            Some(true)
        );
    }
    e.state_core_mut().set_yexinjia_friends(3, vec![0]).unwrap();
    assert_eq!(
        e.state_core().source_is_enemy_of(e.catalog(), 0, 3),
        Some(false)
    );
    assert_eq!(
        e.state_core().source_is_enemy_of(e.catalog(), 3, 0),
        Some(false)
    );
    let copy = restore(&e, &[]);
    assert_eq!(copy.state_core().players, e.state_core().players);
    let before = e.save();
    assert!(e.state_core_mut().set_yexinjia_friends(0, vec![0]).is_err());
    assert_eq!(e.save(), before);
    e.state_core_mut().set_yexinjia_friends(3, vec![]).unwrap();
    assert_eq!(
        e.state_core().source_is_enemy_of(e.catalog(), 3, 0),
        Some(true)
    );
}
#[test]
fn c9i_supported_unique_extension_cards_prove_wide_f09_eligibility() {
    let mut p = emitter();
    p.cards = (0..300)
        .map(|i| {
            engine_native::catalog::CardDef::basic(
                Box::leak(format!("c9i-actual-sha-{i}").into_boxed_str()),
                engine_native::CardKind::Sha,
            )
        })
        .collect();
    let mut e = fixture(&p, "章武");
    assert!(e.catalog().cards.len() > 300);
    let ids: Vec<_> = e
        .catalog()
        .cards
        .iter()
        .enumerate()
        .filter_map(|(i, c)| c.name.starts_with("c9i-actual-sha-").then_some(i as u32))
        .take(256)
        .collect();
    assert_eq!(ids.len(), 256);
    for id in &ids {
        let name = e.catalog().cards[*id as usize].name;
        e.debug_move_named_card_to_hand(1, name);
    }
    let _ = restore(&e, &[&p]);
    fire(&mut e);
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "c331_zhangwu_holder");
    assert_eq!(
        c.spec.max, 256,
        "actual supported unique catalog/ownership/eligibility has no255 cap"
    );
    assert!(ids
        .iter()
        .all(|c| e.open_choice().unwrap().spec.options.contains(&O::Card(*c))));
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![]);
        finish(branch);
        assert_eq!(branch.debug_hand(1).len(), 256);
    }
    same_final(&e, &copy);
    capture(&e, "c9i-supported-wide-catalog");
}
#[test]
fn c9i_source_jubao_gainability_filters_actual_treasure_before_choice() {
    let (p, mut e, c) = qinglong_ready();
    let def = e
        .catalog()
        .skill_by_name("native:liuli/guozhan/jubao")
        .expect("actual source Guozhan jubao gain modifier is represented");
    e.debug_grant(1, "native:liuli/guozhan/jubao");
    e.debug_set_turn(1, Phase::Play, 1);
    let name = e
        .catalog()
        .cards
        .iter()
        .find(|x| {
            x.kind != engine_native::CardKind::Qinglonglingzhu
                && x.kind.equip_slot() == Some(engine_native::catalog::EquipSlot::Treasure)
        })
        .unwrap()
        .name;
    let eq = e.debug_place(1, name);
    let out = submit(
        &mut e,
        GameAction::PlayCard {
            card: eq,
            targets: vec![],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    e.debug_set_turn(0, Phase::Play, 1);
    play(&mut e, c);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "c331_qinglong_card");
    assert!(!e.open_choice().unwrap().spec.options.contains(&O::Card(eq)));
    let _ = restore(&e, &[&p]);
    answer(&mut e, vec![O::HiddenCard(1)]);
    finish(&mut e);
    assert!(e.debug_equips(1).contains(&eq));
    assert!(e.state_core().has_skill(1, def));
}
