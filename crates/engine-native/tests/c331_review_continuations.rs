use super::c331_genuine_invocations::{answer, emitter, fire, fixture};
use super::c331_invocation_boundaries::qinglong_ready;
use super::c331_source_predicates::{finish, restore, same_final};
use engine_native::{ChoiceOption as O, Engine, MachineSnapshot};
use engine_native::event::{EventData, NativeInvocationProgress as P};

#[test]
fn c9j_jiding_genuine_zhenlie_removed_target_restores_and_continues() {
    let mut pack=emitter(); pack.host_skills.push(engine_native::content::host::HostSkillDecl{name:"zhenlie",behavior: engine_native::content::host::HostBehavior::Zhenlie}); let mut e=fixture(&pack,"急叮");
    e.debug_grant(2,"zhenlie"); e.debug_place(0,"sha");
    e.set_time_slice(Some(1)); fire(&mut e);
    for _ in 0..1600 {
        if let Some(c)=e.open_choice().cloned(){
            if c.spec.prompt=="zhenlie_discard" {
                assert!(e.save().resolution.stack.iter().any(|f|matches!(&f.data,EventData::CardUsed(u)if u.use_as==Some(engine_native::CardKind::LeiSha)&&u.targets.is_empty())));
                for mutation in 0..7 {
                    let mut bad=e.save();
                    let invocation=bad.resolution.stack.iter_mut().find_map(|f| match &mut f.data { EventData::SkillInvocation(d) if !d.jiding_changes.is_empty()=>Some(d), _=>None }).expect("real receipt");
                    match mutation {
                        0=>invocation.jiding_changes.clear(),
                        1=>invocation.jiding_changes[0].def=invocation.def,
                        2=>invocation.jiding_changes[0].actor=3,
                        3=>invocation.jiding_changes[0].card=invocation.window,
                        4=>invocation.jiding_changes[0].before_target=1,
                        5=>invocation.jiding_changes.push(invocation.jiding_changes[0].clone()),
                        _=>if let engine_native::event::JidingCardCost::Zhenlie{lose_hp}=&mut invocation.jiding_changes[0].cost { *lose_hp=invocation.window; },
                    }
                    super::c331_source_predicates::reject_restore_atomic(&e,&[&pack],&bad);
                }
                let mut typed=Engine::restore_with_extensions(&e.save(),&[&pack]).expect("genuine removed-target typed restore");
                let mut bytes=restore(&e,&[&pack]);
                for branch in [&mut e,&mut typed,&mut bytes]{let card=branch.open_choice().unwrap().spec.options[0];answer(branch,vec![card]);finish(branch);}
                same_final(&e,&typed); same_final(&e,&bytes);return;
            }
            let selection=if c.spec.prompt=="trigger_confirm"&&c.observer==2{vec![O::Confirm]}else if c.spec.allow_cancel{vec![]}else{c.spec.options.iter().take(c.spec.min).copied().collect()};
            answer(&mut e,selection);
        }else{e.resume();}
    }
    panic!("actual Zhenlie discard pause not reached");
}

#[test]
fn c9j_qinglong_custody_redaction_preserves_full_physical_ownership() {
    let mut paired_public = None;
    for slot in [0,1] {
        let(pack,mut e,c)=qinglong_ready(); super::c331_source_predicates::play(&mut e,c);
        e.set_time_slice(Some(1));answer(&mut e,vec![O::HiddenCard(slot)]);
        for _ in 0..1000 {
            if e.save().resolution.parks.iter().any(|p|matches!(p.machine,MachineSnapshot::NativeInvocation{progress:P::QinglongLoss{..},..})) {
                for seat in [0,1,2,3] {
                    let mut red=e.clone();engine_native::ai::redact_opponent_hands(&mut red,seat);
                    let mut typed=Engine::restore_with_extensions(&red.save(),&[&pack]).expect("redacted custody owns each physical card once");
                    let mut bytes=restore(&red,&[&pack]);
                    let before=format!("{:?}{:?}{:?}{:?}",red.view_for(seat),red.choice_view_for(Some(seat)),red.event_view_for(seat),red.state_core().rng);
                    finish(&mut red);finish(&mut typed);finish(&mut bytes);same_final(&red,&typed);same_final(&red,&bytes);
                    if seat==3 {
                        let public=(before,format!("{:?}{:?}{:?}{:?}",red.view_for(seat),red.choice_view_for(Some(seat)),red.event_view_for(seat),red.state_core().rng));
                        if let Some(expected)=&paired_public {assert_eq!(&public,expected,"nonparticipant paired hidden custody worlds");} else {paired_public=Some(public);}
                    }
                }
                break;
            }
            e.resume();
        }
    }
}

#[test]
fn c9j_nested_source_death_sixiang_revival_restores() {
    for expands in [false,true] { check_source_revival(expands); }
}
fn check_source_revival(expands: bool) {
    let pack=emitter();
    let mut e=fixture(&pack,"急叮");
    e.debug_grant(2,"回火");
    e.debug_grant(0,if expands {"青木"} else {"青木3"});
    e.debug_grant(0,if expands {"青木·转阶"} else {"青木3·转阶"});
    e.debug_set_hp(2,1);e.debug_set_hp(0,3);
    e.set_time_slice(Some(1));fire(&mut e);
    let mut died=false;
    for _ in 0..3000 {
        died |= !e.state_core().player(2).alive;
        let s=e.save();
        if died && e.state_core().player(2).alive
            && s.resolution.stack.iter().any(|f|matches!(&f.data,EventData::Damage(d)if d.source.is_none()&&d.settlement.cleared_source==Some(2)))
            && s.resolution.stack.iter().any(|f|matches!(f.data,EventData::SkillInvocation(_))) {
            assert!(e.log().iter().any(|l|matches!(l,engine_native::LogEvent::BossRevived{player:2})));
            let mut typed=Engine::restore_with_extensions(&s,&[&pack]).expect("historical source remains admitted after native revival");
            let mut bytes=restore(&e,&[&pack]);
            let mut aborted=Engine::restore_with_extensions(&s,&[&pack]).unwrap();
            aborted.abort_segment().expect("restored extension checkpoint abort");
            let mut expected=*s.segment_checkpoint.clone().unwrap();
            let mut actual=aborted.save();actual.auth_generation=expected.auth_generation;
            assert_eq!(actual,expected,"abort removes appended seats, ownership, continuations and RNG");
            assert!(aborted.log().is_empty());
            if expands {
                assert_eq!(s.core.players.len(),6);
                for mutation in 0..4 {
                    let mut bad=s.clone();let checkpoint=bad.segment_checkpoint.as_mut().unwrap();
                    match mutation {
                        0=>checkpoint.core.players=s.core.players.clone(),
                        1=>checkpoint.core.turn.current=5,
                        2=>checkpoint.core.revision=s.core.revision+1,
                        _=>{checkpoint.core.players[0].hand.push(checkpoint.core.deck[0]);},
                    }
                    super::c331_source_predicates::reject_restore_atomic(&e,&[&pack],&bad);
                }
            }
            expected.auth_generation=actual.auth_generation;
            finish(&mut e);finish(&mut typed);finish(&mut bytes);
            same_final(&e,&typed);same_final(&e,&bytes);return;
        }
        if e.open_choice().is_some(){answer(&mut e,vec![]);}else{e.resume();}
    }
    panic!("genuine nested death then Sixiang revival not reached");
}

#[test]
fn c9j_hwendao_hand_and_equipment_origin_visibility_survives_restore() {
    let pack=super::c331_source_predicates::source_pack("shahai-swd-upstream-noname");
    for public in [false,true] {
        let mut e=Engine::new_with_extensions(19003,3,&[&pack]).unwrap();
        for seat in 0..3 {e.debug_clear_hand(seat);}
        e.debug_set_turn(1,engine_native::Phase::Play,1);e.debug_grant(0,"hwendao");
        let response=e.debug_place(0,"shan");
        let acquired=e.debug_place(1,"zhuge");
        assert_ne!(e.catalog().cards[response as usize].suit,e.catalog().cards[acquired as usize].suit);
        if public {e.state_core_mut().player_mut(1).hand.retain(|c|*c!=acquired);e.state_core_mut().player_mut(1).equips.push(acquired);}
        let attack=e.debug_place(1,"sha");
        let outcome=e.submit(engine_native::GameCommand{actor:1,expected_revision:e.revision(),dedup_id:e.save().last_dedup+1,action:engine_native::GameAction::PlayCard{card:attack,targets:vec![0]}});
        assert!(!matches!(outcome,engine_native::CommandOutcome::Rejected(_)));
        answer(&mut e,vec![O::Card(response)]);
        assert_eq!(e.open_choice().unwrap().spec.prompt,"trigger_confirm");
        let mut typed=Engine::restore_with_extensions(&e.save(),&[&pack]).unwrap();
        let mut bytes=restore(&e,&[&pack]);
        for branch in [&mut e,&mut typed,&mut bytes] {
            answer(branch,vec![O::Confirm]);finish(branch);
            assert!(branch.debug_hand(0).contains(&acquired));
            assert!(branch.log().iter().any(|l|matches!(l,engine_native::LogEvent::CardsTaken{from:1,to:0,cards,public:p}if cards==&[acquired]&&*p==public)));
            assert_eq!(branch.event_view_for(2).iter().any(|l|matches!(l,engine_native::view::EventView::Public(engine_native::LogEvent::CardsTaken{cards,..})if cards==&[acquired])),public);
            for seat in [0,1] {assert!(branch.event_view_for(seat).iter().any(|l|matches!(l,engine_native::view::EventView::Public(engine_native::LogEvent::CardsTaken{cards,..})if cards==&[acquired])));}
        }
        same_final(&e,&typed);same_final(&e,&bytes);
    }
}

#[test]
fn c9j_duishi_actual_hand_and_equipment_acquisitions_keep_visibility() {
    let pack=super::c331_source_predicates::source_pack("shahai-swd-upstream-noname");
    let mut seen=[false;2];
    for seed in 19004..19020 {
        let mut e=Engine::new_with_extensions(seed,3,&[&pack]).unwrap();
        for seat in 0..3 {e.debug_clear_hand(seat);}
        e.debug_set_turn(0,engine_native::Phase::Play,1);e.debug_grant(0,"duishi");
        let cost=e.debug_place(0,"tao");let hand=e.debug_place(1,"sha");let equip=e.debug_place(1,"zhuge");
        e.state_core_mut().player_mut(1).hand.retain(|c|*c!=equip);e.state_core_mut().player_mut(1).equips.push(equip);
        let skill=e.catalog().skill_by_name("duishi").unwrap();
        let out=e.submit(engine_native::GameCommand{actor:0,expected_revision:e.revision(),dedup_id:e.save().last_dedup+1,action:engine_native::GameAction::ActivateSkill{skill}});
        assert!(!matches!(out,engine_native::CommandOutcome::Rejected(_)));
        answer(&mut e,vec![O::Card(cost)]);answer(&mut e,vec![O::Player(1)]);finish(&mut e);
        let (taken,public)=e.log().iter().find_map(|l|if let engine_native::LogEvent::CardsTaken{from:1,to:0,cards,public}=l {Some((cards[0],*public))} else {None}).expect("actual acquisition");
        assert_eq!(public,taken==equip);assert!(taken==hand||taken==equip);seen[public as usize]=true;
        assert_eq!(e.event_view_for(2).iter().any(|l|matches!(l,engine_native::view::EventView::Public(engine_native::LogEvent::CardsTaken{cards,..})if cards==&[taken])),public);
        let mut typed=Engine::restore_with_extensions(&e.save(),&[&pack]).unwrap();let mut bytes=restore(&e,&[&pack]);
        finish(&mut typed);finish(&mut bytes);same_final(&e,&typed);same_final(&e,&bytes);
        if seen==[true,true] {break;}
    }
    assert_eq!(seen,[true,true]);
}

#[test]
fn c9j_jiding_tongji_and_yihua_restored_continuations() {
    for transfer in [false,true] {
        let mut pack=emitter();
        pack.host_skills.push(engine_native::content::host::HostSkillDecl {name:"yihua",behavior:engine_native::content::host::HostBehavior::Yihua});
        let mut e=fixture(&pack,"急叮");
        let actor=if transfer {3} else {2};
        if transfer { e.debug_grant(3,"同疾"); e.debug_place(2,"sha"); }
        e.debug_grant(actor,"yihua");
        e.debug_place(actor,"sha"); e.debug_place(actor,"shan");
        e.set_time_slice(Some(1)); fire(&mut e);
        let mut checked=false;
        for _ in 0..2000 {
            let s=e.save();
            if s.resolution.stack.iter().any(|f|matches!(&f.data,EventData::SkillInvocation(d)if d.jiding_changes.iter().any(|c|matches!(c.cost,engine_native::event::JidingCardCost::Yihua{..})))) {
                let mut typed=Engine::restore_with_extensions(&s,&[&pack]).expect("real paid reflection");
                let mut bytes=restore(&e,&[&pack]);
                if transfer {
                    let changes=s.resolution.stack.iter().find_map(|f|if let EventData::SkillInvocation(d)=&f.data {(!d.jiding_changes.is_empty()).then_some(&d.jiding_changes)} else {None}).unwrap();
                    assert_eq!(changes.len(),2,"actual transfer then reflection");
                    let mut bad=s.clone();
                    for f in &mut bad.resolution.stack {if let EventData::SkillInvocation(d)=&mut f.data {if d.jiding_changes.len()==2 {d.jiding_changes.swap(0,1);}}}
                    super::c331_source_predicates::reject_restore_atomic(&e,&[&pack],&bad);
                }
                finish(&mut e);finish(&mut typed);finish(&mut bytes);same_final(&e,&typed);same_final(&e,&bytes);
                checked=true;break;
            }
            if let Some(c)=e.open_choice().cloned() {
                let selection=match c.spec.prompt {
                    "trigger_confirm"=>vec![O::Confirm],
                    "yihua_discard"=>c.spec.options.iter().take(2).copied().collect(),
                    "tongji_transfer_discard"=>vec![c.spec.options[0]],
                    _=>if c.spec.allow_cancel {vec![]} else {c.spec.options.iter().take(c.spec.min).copied().collect()},
                };
                answer(&mut e,selection);
            } else {e.resume();}
        }
        assert!(checked,"paid source-specific reflection transfer={transfer}");
    }
}
