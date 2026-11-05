//! Actual source ordering contracts; source and private caller proofs are in phase5 evidence.
use engine_native::content::host::HostBehavior as H;
use engine_native::event::Timing;
#[test]
fn phase5_all44_source_tuples_match_the_current_native_bindings() {
    for (host, timing, priority, selectable) in [
        (H::BossBaimang2, Timing::Normal, 1, false),
        (H::BossBaimang3, Timing::Normal, 1, false),
        (H::BossBaiyi, Timing::Normal, 0, true),
        (H::BossBaiyiThunder, Timing::Normal, 0, true),
        (H::BossBianshen3, Timing::Normal, 1, false),
        (H::BossBianshen4, Timing::Normal, 1, false),
        (H::BossChiyan2, Timing::Normal, 1, false),
        (H::BossChiyan3, Timing::Normal, 1, false),
        (H::BossChiyi, Timing::Normal, 0, true),
        (H::BossChiyiMark, Timing::Normal, 0, true),
        (H::BossHuanren, Timing::Normal, 0, true),
        (H::BossHuanrenDie, Timing::Normal, -999, false),
        (H::BossQingmu2, Timing::Normal, 1, false),
        (H::BossQingmu3, Timing::Normal, 1, false),
        (H::BossQingyi, Timing::Normal, 0, true),
        (H::BossXuanlin2, Timing::Normal, 1, false),
        (H::BossXuanlin3, Timing::Normal, 1, false),
        (H::GujianYunvyuanshen, Timing::Normal, 610, true),
        (H::GwentBirinState, Timing::Normal, 1, false),
        (H::GwentCiguState, Timing::Normal, 0, true),
        (H::GwentKunenState, Timing::Normal, 0, true),
        (H::GwentQinpenState, Timing::Normal, 1, false),
        (H::Miedao, Timing::Normal, 0, true),
        (H::Shishi, Timing::Normal, 0, true),
        (H::SwdBaishouzhihuHeld, Timing::Normal, 0, true),
        (H::SwdGouhunluoSub, Timing::Normal, 0, true),
        (H::SwdHuanglinzhicongHeld, Timing::Normal, 0, true),
        (H::SwdJiguanyaoGain, Timing::Normal, 0, true),
        (H::SwdJiguanyuanGain, Timing::Normal, 0, true),
        (H::SwdJuxiCount, Timing::Normal, 0, true),
        (H::SwdQinglongzhiguiHeld, Timing::Normal, 0, true),
        (H::SwdSxianjingDamage, Timing::Normal, 0, true),
        (H::SwdSxianjingGain, Timing::Normal, 0, true),
        (H::SwdXuanwuzhihuangHeld, Timing::Normal, 0, true),
        (H::SwdYuchanSwap, Timing::Normal, -99, false),
        (H::SwdZhuquezhizhangHeld, Timing::Normal, 0, true),
        (H::WudianPaotaiFire, Timing::Normal, 0, true),
        (H::WudianQiangguGuard, Timing::Normal, 0, true),
        (H::WudianZhenchaDraw, Timing::Normal, 0, true),
        (H::XianjianHushaCount, Timing::Normal, 0, true),
        (H::Xiefang, Timing::Normal, -99, false),
        (H::XiguaXueliangFanbei, Timing::Normal, 1, false),
        (H::Xunmeng, Timing::Normal, 0, true),
        (H::Zaibian, Timing::Normal, 0, true),
    ] {
        let m = host.trigger_meta().unwrap();
        assert_eq!(
            (m.timing, m.priority, m.order_selectable),
            (timing, priority, selectable),
            "{host:?}"
        );
    }
}

use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::content::host::HostSkillDecl;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
fn pack(event: &str, behavior: H, name: &'static str) -> ExtensionPack {
    let source=(0..3).map(|i|format!("skill c331_phase5_peer_{i}({}) locked {{ trigger {event} always {{ effect storage_set(self_skill,1,1); }} }}",420007000+i)).collect::<Vec<_>>().join("\n");
    let pkg = content_compiler::compile_source(&source).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "c331-phase5-order",
            version: 1,
            description: "actual source ordering",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![HostSkillDecl {
            name,
            behavior,
            presentation_binding: None,
        }],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&pkg)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}
fn submit(e: &mut Engine, actor: u8, action: GameAction) {
    let result = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.save().last_dedup + 1,
        action,
    });
    assert!(!matches!(result, CommandOutcome::Rejected(_)), "{result:?}");
}
fn answer(e: &mut Engine) {
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.prompt, "trigger_order");
    assert!(matches!(c.spec.options[0], ChoiceOption::Skill(_)));
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection: vec![c.spec.options[0]],
        },
    );
}
fn restore(e: &Engine, p: &ExtensionPack) -> Engine {
    let decoded =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    Engine::restore_with_extensions(&decoded, &[p]).unwrap()
}
fn same(e: &Engine, r: &Engine) {
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
    assert_eq!(e.state_hash(), r.state_hash());
}
fn roundtrip_answer(e: &mut Engine, p: &ExtensionPack) {
    let mut r = restore(e, p);
    same(e, &r);
    let cursor = e.log().len();
    answer(e);
    answer(&mut r);
    same(e, &r);
    assert_eq!(&e.log()[cursor..], r.log());
}
fn grant_peers(e: &mut Engine) {
    for i in 0..3 {
        e.debug_grant(1, &format!("c331_phase5_peer_{i}"));
    }
}

#[test]
fn phase5_boss_silent_normal_waits_for_earlier_owner_instead_of_running_first() {
    use engine_native::BossStageKind;
    let p = pack("death", H::BossBaimang2, "c331_phase5_boss_hide");
    let mut e = Engine::new_boss_stage_with_extensions(331701, 4, 0, 0, 0, 0, 1, &[&p]).unwrap();
    e.debug_set_turn(1, Phase::Play, 1);
    for owner in 0..4 {
        e.debug_clear_hand(owner);
    }
    e.debug_set_hp(0, 1);
    grant_peers(&mut e);
    e.debug_grant(0, "c331_phase5_boss_hide");
    e.debug_deal_damage(Some(1), 0, 1, engine_native::event::DamageNature::Normal);
    for options in [3, 2] {
        assert_eq!(e.open_choice().unwrap().spec.options.len(), options);
        assert_eq!(
            e.state_core().boss_stage().unwrap().kind,
            BossStageKind::None,
            "silent dieBegin has no firstDo: the later owner's Normal bucket must wait"
        );
        assert!(!e.log().iter().any(|event| matches!(
            event,
            engine_native::LogEvent::SkillTriggered {
                skill: "白芒", ..
            }
        )));
        roundtrip_answer(&mut e, &p);
    }
    assert!(e.open_choice().is_none());
    assert_eq!(
        e.state_core().boss_stage().unwrap().kind,
        BossStageKind::SixiangChain
    );
    assert!(e.log().iter().any(|event| matches!(
        event,
        engine_native::LogEvent::SkillTriggered {
            skill: "白芒",
            owner: 0, .. }
    )));
}

#[test]
fn phase5_yuchan_normal_swaps_real_card_before_the_next_owner_instead_of_waiting_last() {
    use engine_native::catalog::{CardDef, CardKind};
    let mut p = pack("card_used", H::SwdYuchanSwap, "c331_phase5_yuchan");
    p.cards = vec![
        CardDef::basic("c331_yuchan_qian", CardKind::Yuchanqian),
        CardDef::basic("c331_yuchan_kun", CardKind::Yuchankun),
    ];
    let mut e = Engine::new_with_extensions(331702, 4, &[&p]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    for owner in 0..4 {
        e.debug_clear_hand(owner);
    }
    e.debug_set_hp(0, 3);
    grant_peers(&mut e);
    e.debug_grant(0, "c331_phase5_yuchan");
    let qian = e.debug_place(0, "c331_yuchan_qian");
    let tao = e.debug_place(0, "tao");
    let kun = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "c331_yuchan_kun")
        .unwrap() as u32;
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: tao,
            targets: vec![],
        },
    );
    for options in [3, 2] {
        assert_eq!(e.open_choice().unwrap().spec.options.len(), options);
        assert!(!e.debug_hand(0).contains(&qian),"source useCardAfter has no lastDo: owner0's Normal branch must run before owner1's bucket");
        assert!(e.debug_hand(0).contains(&kun));
        roundtrip_answer(&mut e, &p);
    }
    assert!(e.open_choice().is_none());
    assert!(e.debug_hand(0).contains(&kun));
}
