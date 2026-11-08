//! Actual source ordering contracts; source and private caller proofs are in phase5 evidence.
use engine_native::content::host::HostBehavior as H;
use engine_native::event::Timing;
pub const DECLS: &[(H, Timing, i32, bool)] = &[
    (H::BossBaimang2, Timing::Normal, 1, false),
    (H::BossBaimang2x, Timing::Normal, -1000, false),
    (H::BossBaimang3, Timing::Normal, 1, false),
    (H::BossBaimang3x, Timing::Normal, -1000, false),
    (H::BossBaiyi, Timing::Normal, 0, true),
    (H::BossBaiyiThunder, Timing::Normal, 0, true),
    (H::BossBianshen3, Timing::Normal, 1, false),
    (H::BossBianshen3x, Timing::Normal, -1000, false),
    (H::BossBianshen4, Timing::Normal, 1, false),
    (H::BossBianshen4x, Timing::Normal, -1000, false),
    (H::BossChiyan2, Timing::Normal, 1, false),
    (H::BossChiyan2x, Timing::Normal, -1000, false),
    (H::BossChiyan3, Timing::Normal, 1, false),
    (H::BossChiyan3x, Timing::Normal, -1000, false),
    (H::BossChiyi, Timing::Normal, 0, true),
    (H::BossChiyiMark, Timing::Normal, 0, true),
    (H::BossHuanren, Timing::Normal, 0, true),
    (H::BossHuanrenDie, Timing::Normal, -999, false),
    (H::BossQingmu2, Timing::Normal, 1, false),
    (H::BossQingmu2x, Timing::Normal, -1000, false),
    (H::BossQingmu3, Timing::Normal, 1, false),
    (H::BossQingmu3x, Timing::Normal, -1000, false),
    (H::BossQingyi, Timing::Normal, 0, true),
    (H::BossXuanlin2, Timing::Normal, 1, false),
    (H::BossXuanlin2x, Timing::Normal, -1000, false),
    (H::BossXuanlin3, Timing::Normal, 1, false),
    (H::BossXuanlin3x, Timing::Normal, -1000, false),
    (H::GujianYunvyuanshen, Timing::Normal, 610, true),
    (H::GwentBirinState, Timing::Normal, 1, false),
    (H::GwentCiguState, Timing::Normal, 0, true),
    (H::GwentFengyinState, Timing::Normal, 0, true),
    (H::GwentKunenState, Timing::Normal, 0, true),
    (H::GwentQinpenState, Timing::Normal, 1, false),
    (H::JinlianzhuCancel, Timing::Normal, 0, false),
    (H::Kunlunjing, Timing::Normal, 1000, true),
    (H::MbQianlong, Timing::Normal, 600, true),
    (H::MbWeitong, Timing::Normal, 1000, true),
    (H::Miedao, Timing::Normal, 0, true),
    (H::Niepan, Timing::Normal, 0, true),
    (H::SanjianWeaponDamage, Timing::Normal, -25, true),
    (H::ShengkeCheck, Timing::Normal, 0, false),
    (H::Shishi, Timing::Normal, 0, true),
    (H::SwdBaishouzhihuHeld, Timing::Normal, 0, true),
    (H::SwdCangchizhibiHeld, Timing::Normal, 0, true),
    (H::SwdFengyinState, Timing::Normal, 0, true),
    (H::SwdGouhunluoSub, Timing::Normal, 0, true),
    (H::SwdHuanglinzhicongHeld, Timing::Normal, 0, true),
    (H::SwdJiguanyaoGain, Timing::Normal, 0, true),
    (H::SwdJiguanyuanGain, Timing::Normal, 0, true),
    (H::SwdJuxiCount, Timing::Normal, 0, true),
    (H::SwdLexue, Timing::Normal, 0, false),
    (H::SwdNigongAccumulate, Timing::Normal, -25, true),
    (H::SwdQinglongzhiguiHeld, Timing::Normal, 0, true),
    (H::SwdShencaojieGuard, Timing::Normal, 0, false),
    (H::SwdShenmiguoWindow, Timing::Normal, 0, false),
    (H::SwdShouyin, Timing::Normal, 0, true),
    (H::SwdSxianjingDamage, Timing::Normal, 0, true),
    (H::SwdSxianjingGain, Timing::Normal, 0, true),
    (H::SwdXuanwuzhihuangHeld, Timing::Normal, 0, true),
    (H::SwdXuanyingDamage, Timing::Normal, 0, true),
    (H::SwdYuchanSwap, Timing::Normal, -99, false),
    (H::SwdZhuquezhizhangHeld, Timing::Normal, 0, true),
    (H::WudianPaotaiFire, Timing::Normal, 0, true),
    (H::WudianQiangguGuard, Timing::Normal, 0, true),
    (H::WudianZhenchaDraw, Timing::Normal, 0, true),
    (H::XianjianChengxin, Timing::Normal, 0, true),
    (H::XianjianHushaCount, Timing::Normal, 0, true),
    (H::XianjianShuiyun5, Timing::Normal, 0, true),
    (H::Xiefang, Timing::Normal, -99, false),
    (H::XiguaXueliangFanbei, Timing::Normal, 1, false),
    (H::Xunmeng, Timing::Normal, 0, true),
    (H::Zaibian, Timing::Normal, 0, true),
    (H::ZhuWanzhongJi, Timing::Normal, 0, false),
];

pub const REMAINING_28: &[H] = &[];

pub const LIULI_COMPAT_DELTAS: &[(H, i32, i32)] = &[
    (H::SanjianWeaponDamage, -25, -30),
    (H::SwdNigongAccumulate, -25, 0),
];

#[test]
fn phase5_all44_source_tuples_match_the_current_native_bindings() {
    for &(host, timing, priority, selectable) in DECLS {
        let m = host.trigger_meta().unwrap();
        assert_eq!(
            (m.timing, m.priority, m.order_selectable),
            (timing, priority, selectable),
            "{host:?}"
        );
        assert_eq!(
            host.source_order(),
            Some((timing, priority, selectable)),
            "{host:?}"
        );
    }
    assert_eq!(REMAINING_28.len(), 0);
    for &(host, core_priority, liuli_priority) in LIULI_COMPAT_DELTAS {
        assert_ne!(core_priority, liuli_priority);
        assert_eq!(host.trigger_meta().unwrap().priority, core_priority);
    }
    let remaining: Vec<_> = H::ALL
        .iter()
        .copied()
        .filter(|h| h.trigger_meta().is_some() && h.source_order().is_none())
        .collect();
    assert!(
        remaining.is_empty(),
        "all trigger-bearing HostBehavior variants must have source_order: {remaining:?}"
    );
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

#[test]
fn phase5_xianjian_chengxin_and_shuiyun5_admit_self_dying_rescue() {
    let p1 = pack("dying", H::XianjianChengxin, "c331_phase5_chengxin");
    let mut e = Engine::new_with_extensions(331703, 3, &[&p1]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 1);
    e.debug_grant(1, "c331_phase5_chengxin");
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().expect("chengxin self-dying confirm").clone();
    assert_eq!(c.observer, 1);
    assert_eq!(c.spec.prompt, "trigger_confirm");
    submit(
        &mut e,
        1,
        GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.debug_hp(1), 1);
    assert!(e.state_core().player(1).chengxin_active);

    let p2 = pack("dying", H::XianjianShuiyun5, "c331_phase5_shuiyun5");
    let mut e2 = Engine::new_with_extensions(331704, 3, &[&p2]).unwrap();
    e2.debug_set_turn(0, Phase::Play, 1);
    for p in 0..3 {
        e2.debug_clear_hand(p);
    }
    e2.debug_set_hp(1, 1);
    let stored = e2.debug_place(1, "sha");
    e2.state_core_mut().player_mut(1).hand.retain(|c| *c != stored);
    e2.state_core_mut().player_mut(1).off_court.push(stored);
    e2.debug_grant(1, "c331_phase5_shuiyun5");
    e2.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e2.open_choice().expect("shuiyun5 self-dying confirm").clone();
    assert_eq!(c.observer, 1);
    assert_eq!(c.spec.prompt, "trigger_confirm");
    submit(
        &mut e2,
        1,
        GameAction::Answer {
            choice_id: c.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c2 = e2.open_choice().expect("shuiyun5 off_court card pick").clone();
    submit(
        &mut e2,
        1,
        GameAction::Answer {
            choice_id: c2.id,
            selection: vec![ChoiceOption::Card(stored)],
        },
    );
    assert_eq!(e2.debug_hp(1), 1);
    assert_eq!(e2.state_core().player(1).shuiyun_count, 1);
}

