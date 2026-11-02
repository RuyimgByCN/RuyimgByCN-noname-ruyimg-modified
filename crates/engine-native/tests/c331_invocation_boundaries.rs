use super::c331_genuine_invocations::{answer, emitter, fire, fixture};
use super::c331_source_predicates::{capture, finish, reject_restore_atomic, restore};
use engine_native::event::{EventData, NativeInvocationProgress as P, YuanceOption};
use engine_native::{ChoiceOption, Engine, MachineSnapshot};
fn paused_holder() -> (engine_native::content::extension::ExtensionPack, Engine) {
    let p = emitter();
    let mut e = fixture(&p, "章武");
    let a = e.debug_place(1, "sha");
    let b = e.debug_place(1, "shan");
    e.debug_place(2, "tao");
    e.debug_place(2, "sha");
    fire(&mut e);
    e.set_time_slice(Some(1));
    answer(&mut e, vec![ChoiceOption::Card(a), ChoiceOption::Card(b)]);
    for _ in 0..500 {
        if e.save().resolution.parks.iter().any(|x| {
            matches!(
                x.machine,
                MachineSnapshot::NativeInvocation {
                    progress: P::HolderAfterLog { count: 2 },
                    ..
                }
            )
        }) {
            return (p, e);
        }
        e.resume();
    }
    panic!("real paid two-card continuation");
}
#[test]
fn c9h_paid_count_mutation_rejected_typed_and_bytes() {
    let (p, e) = paused_holder();
    let _ = restore(&e, &[&p]);
    capture(&e, "c9h-paid-two-before-forgery");
    let mut bad = e.save();
    for x in &mut bad.resolution.parks {
        if let MachineSnapshot::NativeInvocation {
            progress: P::HolderAfterLog { count },
            ..
        } = &mut x.machine
        {
            *count = 1;
        }
    }
    reject_restore_atomic(&e, &[&p], &bad);
}
#[test]
fn c9h_admitted_participant_mutations_rejected() {
    let (p, e) = paused_holder();
    for source in [true, false] {
        let mut bad = e.save();
        for frame in &mut bad.resolution.stack {
            if let EventData::SkillInvocation(data) = &mut frame.data {
                if source {
                    data.source = Some(3);
                } else {
                    data.target = 2;
                }
            }
        }
        reject_restore_atomic(&e, &[&p], &bad);
    }
}
#[test]
fn c9h_cross_effect_body_progress_rejected() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    e.set_time_slice(Some(1));
    fire(&mut e);
    for _ in 0..500 {
        let mut bad = e.save();
        if let Some(park) = bad.resolution.parks.iter_mut().find(|x| {
            matches!(
                x.machine,
                MachineSnapshot::NativeInvocation {
                    progress: P::Body,
                    ..
                }
            )
        }) {
            if let MachineSnapshot::NativeInvocation { progress, .. } = &mut park.machine {
                *progress = P::HolderAfterLog { count: 2 };
            }
            let _ = restore(&e, &[&p]);
            reject_restore_atomic(&e, &[&p], &bad);
            return;
        }
        e.resume();
    }
    panic!("actual Jiding body entry");
}
#[test]
fn c9h_original_native_helper_ids_remain_stable() {
    let e = Engine::new(16001, 3);
    for (i, name) in [
        "native:liuli/ow/jidong2",
        "native:liuli/xianjian/chengxin2",
        "native:liuli/card-swd/chilongya",
        "native:liuli/card-swd/xingjunyan",
        "native:liuli/ow/liangou2",
        "native:liuli/swd/mailun31",
        "native:liuli/swd/mailun32",
        "native:liuli/swd/mailun41",
        "native:liuli/swd/mailun42",
        "native:liuli/swd/hutian2",
        "native:liuli/card-swd/shuchui2",
        "native:liuli/ow/paotai3",
        "native:liuli/card-swd/xixue",
        "native:liuli/card-swd/qinglonglingzhu",
        "native:liuli/swd/huanxia_expire",
    ]
    .iter()
    .enumerate()
    {
        assert_eq!(e.catalog().skill_name(215 + i as u16), *name);
    }
}
#[test]
fn c9h_sole_guicai_outside_kingdoms_cannot_pay_equipment() {
    let p = emitter();
    let mut e = fixture(&p, "远策");
    e.debug_grant(3, "鬼才");
    let c = e.debug_place(3, "armor");
    e.state_core_mut().player_mut(3).hand.retain(|x| *x != c);
    e.state_core_mut().player_mut(3).equips.push(c);
    fire(&mut e);
    answer(&mut e, vec![ChoiceOption::Control(YuanceOption::LoseHp)]);
    assert!(
        e.open_choice().is_none(),
        "sole GuiCai requires hand outside Kingdoms"
    );
    finish(&mut e);
}
#[test]
fn c9h_jiding_empty_deck_is_genuine_virtual_sha() {
    let p = emitter();
    let mut e = fixture(&p, "急叮");
    e.debug_place(2, "shan");
    let deck = std::mem::take(&mut e.state_core_mut().deck);
    e.state_core_mut().discard.extend(deck);
    let _ = restore(&e, &[&p]);
    let def = e.catalog().skill_by_name("急叮").unwrap();
    fire(&mut e);
    let c = e.open_choice().expect("actual dodge with empty deck");
    let selected = c
        .spec
        .options
        .iter()
        .find(|x| matches!(x, ChoiceOption::Card(_)))
        .copied()
        .unwrap();
    answer(&mut e, vec![selected]);
    finish(&mut e);
    assert!(!e.state_core().has_skill_any_state(0, def));
}
#[test]
fn c9h_paired_context_and_admission_cannot_retarget_canonical_cause() {
    let (p, e) = paused_holder();
    let _ = restore(&e, &[&p]);
    for source in [true, false] {
        let mut bad = e.save();
        for f in &mut bad.resolution.stack {
            if let EventData::SkillInvocation(c) = &mut f.data {
                if source {
                    c.source = Some(3)
                } else {
                    c.target = 2
                }
            }
        }
        for park in &mut bad.resolution.parks {
            if let MachineSnapshot::NativeInvocation {
                progress: P::AwaitInvocation { admission, .. },
                ..
            } = &mut park.machine
            {
                if source {
                    admission.source = Some(3)
                } else {
                    admission.target = 2
                }
            }
        }
        reject_restore_atomic(&e, &[&p], &bad);
    }
}
#[test]
fn c9h_jidian_black_waits_for_judge_then_logs_and_can_decline_nested() {
    let p = emitter();
    let mut e = fixture(&p, "极点");
    e.debug_grant(2, "极点");
    e.debug_set_deck_top_by(|c| c.suit == engine_native::catalog::Suit::Spade);
    fire(&mut e);
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![ChoiceOption::Player(0)]);
        let judged = branch
            .log()
            .iter()
            .position(|x| matches!(x, engine_native::LogEvent::JudgementFinalized { .. }))
            .unwrap();
        let logged = branch
            .log()
            .iter()
            .position(|x| {
                matches!(
                    x,
                    engine_native::LogEvent::SkillTriggered {
                        skill: "极点", ..
                    }
                )
            })
            .unwrap();
        assert!(
            judged < logged,
            "source target.judge completes before logSkillBegin"
        );
        assert_eq!(branch.debug_hp(0), 3);
        assert_eq!(
            branch.open_choice().unwrap().spec.prompt,
            "c331_jidian_target"
        );
        answer(branch, vec![]);
        finish(branch);
    }
    super::c331_source_predicates::same_final(&e, &copy);
}
pub(super) fn qinglong_ready() -> (
    engine_native::content::extension::ExtensionPack,
    Engine,
    u32,
) {
    let pack = super::c331_source_predicates::source_pack("shahai-swd-upstream-noname");
    let mut e = Engine::new_with_extensions(16009, 4, &[&pack]).unwrap();
    for p in 0..4 {
        e.debug_clear_hand(p);
        e.debug_set_hp(p, 4);
    }
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    super::c331_source_predicates::equip(&mut e, engine_native::CardKind::Qinglonglingzhu);
    e.debug_place(1, "tao");
    e.debug_place(1, "sha");
    let attack = e.debug_place(0, "fire_sha");
    (pack, e, attack)
}
fn qinglong_fixture() -> Engine {
    let (_, mut e, c) = qinglong_ready();
    super::c331_source_predicates::play(&mut e, c);
    e
}

#[test]
fn c9h_qinglong_opens_optional_private_card_choice() {
    let e = qinglong_fixture();
    let c = e
        .open_choice()
        .expect("real gainPlayerCard choice instead of forced random gain");
    assert_eq!(c.spec.prompt, "c331_qinglong_card");
    let hidden = e.debug_hand(1);
    assert!(
        c.spec
            .options
            .iter()
            .all(|o| !matches!(o,ChoiceOption::Card(id)if hidden.contains(id))),
        "foreign hand identities must stay in private binding"
    );
}
#[test]
fn c9h_qinglong_private_transfer_masks_other_seat_events() {
    let mut e = qinglong_fixture();
    let selected = e.open_choice().unwrap().spec.options[0];
    answer(&mut e, vec![selected]);
    finish(&mut e);
    assert!(
        !e.event_view_for(3).iter().any(|v| matches!(
            v,
            engine_native::view::EventView::Public(engine_native::LogEvent::CardsGiven { .. })
        )),
        "nonparticipants must not learn transferred foreign hand IDs"
    );
}
#[test]
fn c9h_qinglong_hidden_selection_cancel_restore_and_forgery() {
    for cancel in [true, false] {
        let (p, mut e, c) = qinglong_ready();
        super::c331_source_predicates::play(&mut e, c);
        let h = e.debug_hand(1).to_vec();
        let before_rng = format!("{:?}", e.state_core().rng);
        let mut copy = restore(&e, &[&p]);
        let mut bad = e.save();
        for park in &mut bad.resolution.parks {
            if let MachineSnapshot::NativeInvocation {
                progress: P::QinglongAsk { hand, .. },
                ..
            } = &mut park.machine
            {
                hand.swap(0, 1);
            }
        }
        reject_restore_atomic(&e, &[&p], &bad);
        for branch in [&mut e, &mut copy] {
            let raw = branch.submit(engine_native::GameCommand {
                actor: 0,
                expected_revision: branch.revision(),
                dedup_id: branch.save().last_dedup + 1,
                action: engine_native::GameAction::Answer {
                    choice_id: branch.open_choice().unwrap().id,
                    selection: vec![ChoiceOption::Card(h[1])],
                },
            });
            assert!(
                matches!(raw, engine_native::CommandOutcome::Rejected(_)),
                "physical hidden candidate must not bypass slot binding"
            );
            let selection = if cancel {
                vec![]
            } else {
                vec![ChoiceOption::HiddenCard(1)]
            };
            answer(branch, selection);
            finish(branch);
            assert_eq!(format!("{:?}", branch.state_core().rng), before_rng);
            assert_eq!(branch.debug_hand(0).contains(&h[1]), !cancel);
            for seat in 1..4 {
                assert!(!matches!(
                    branch.choice_view_for(Some(seat)),
                    Some(engine_native::view::ChoiceView::Own { .. })
                ));
            }
            if !cancel {
                assert!(branch.event_view_for(3).iter().any(|x| matches!(
                    x,
                    engine_native::view::EventView::CardsGivenHidden {
                        from: 1,
                        to: 0,
                        count: 1
                    }
                )));
            }
        }
        super::c331_source_predicates::same_final(&e, &copy);
    }
}
#[test]
fn c9h_qinglong_selected_public_equipment_revokes_grant_after_loss() {
    let (p, mut e, c) = qinglong_ready();
    e.debug_set_turn(1, engine_native::Phase::Play, 1);
    let eq = e.debug_place(1, "zhangba");
    let out = e.submit(engine_native::GameCommand {
        actor: 1,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action: engine_native::GameAction::PlayCard {
            card: eq,
            targets: vec![],
        },
    });
    assert!(
        matches!(out, engine_native::CommandOutcome::Accepted { .. }),
        "{out:?}"
    );
    e.debug_set_turn(0, engine_native::Phase::Play, 1);
    super::c331_source_predicates::play(&mut e, c);
    assert!(e
        .open_choice()
        .unwrap()
        .spec
        .options
        .contains(&ChoiceOption::Card(eq)));
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![ChoiceOption::Card(eq)]);
        finish(branch);
        assert!(!branch.debug_equips(1).contains(&eq));
        assert!(branch.debug_hand(0).contains(&eq));
        let def = branch.catalog().skill_by_name("丈八蛇矛").unwrap();
        assert!(!branch.state_core().has_skill(1, def));
    }
    super::c331_source_predicates::same_final(&e, &copy);
}
#[test]
fn c9h_qinglong_self_source_uses_owned_cards_and_cancel_is_atomic() {
    let (p, mut e, _) = qinglong_ready();
    e.debug_deal_damage(Some(0), 0, 1, engine_native::event::DamageNature::Thunder);
    let c = e
        .open_choice()
        .expect("pinned source does not exclude self-source");
    assert_eq!(c.spec.prompt, "c331_qinglong_card");
    assert!(c
        .spec
        .options
        .iter()
        .all(|o| !matches!(o, ChoiceOption::HiddenCard(_))));
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![]);
        finish(branch);
    }
    super::c331_source_predicates::same_final(&e, &copy);
}
#[test]
fn c9h_qinglong_paid_loss_restores_after_provider_removal() {
    let (p, mut e, c) = qinglong_ready();
    super::c331_source_predicates::play(&mut e, c);
    let chosen = e.debug_hand(1)[1];
    e.set_time_slice(Some(1));
    answer(&mut e, vec![ChoiceOption::HiddenCard(1)]);
    let provider = std::mem::take(&mut e.state_core_mut().player_mut(0).equips);
    e.state_core_mut().discard.extend(provider);
    for _ in 0..300 {
        if e.save().resolution.parks.iter().any(|x| {
            matches!(
                x.machine,
                MachineSnapshot::NativeInvocation {
                    progress: P::QinglongLoss { .. },
                    ..
                }
            )
        }) {
            super::c331_source_predicates::capture(&e, "c9h-qinglong-paid-loss");
            let mut copy = restore(&e, &[&p]);
            finish(&mut e);
            finish(&mut copy);
            super::c331_source_predicates::same_final(&e, &copy);
            assert!(e.debug_hand(0).contains(&chosen));
            return;
        }
        e.resume();
    }
    panic!("actual paid loss continuation");
}
#[test]
fn c9h_guicai_kingdoms_accepts_equipment_while_nonkingdoms_excludes_it() {
    let p = emitter();
    let mut e = fixture(&p, "远策");
    e.state_core_mut().mode = engine_native::state::GameMode::Kingdoms;
    e.debug_grant(3, "鬼才");
    let eq = e.debug_place(3, "guanshi");
    e.state_core_mut().player_mut(3).hand.retain(|x| *x != eq);
    e.state_core_mut().player_mut(3).equips.push(eq);
    fire(&mut e);
    answer(&mut e, vec![ChoiceOption::Control(YuanceOption::LoseHp)]);
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "gui_dao_rewrite");
    assert!(c.spec.options.contains(&ChoiceOption::Card(eq)));
    let mut copy = restore(&e, &[&p]);
    for branch in [&mut e, &mut copy] {
        answer(branch, vec![ChoiceOption::Card(eq)]);
        finish(branch);
        assert!(!branch.debug_equips(3).contains(&eq));
    }
    super::c331_source_predicates::same_final(&e, &copy);
}
