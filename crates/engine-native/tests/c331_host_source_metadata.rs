//! C331 ordering profiles audited against the selected upstream source and calculator.
use engine_native::content::host::HostBehavior as H;
use engine_native::event::Timing;
#[test]
fn direct_source_profiles_keep_priority_silent_and_translated_choice_qualification() {
    for (host, timing, priority, selectable) in [
        (H::FzXinKuanggu, Timing::Normal, 0, true),
        (H::HlssFengqi, Timing::Normal, 0, true),
        (H::HlssXiangxing, Timing::Normal, 0, true),
        (H::HlssYueyin, Timing::First, 1, false),
        (H::HlssYueyin2, Timing::Normal, 2000, true),
        (H::MbcmQingzheng, Timing::Normal, 0, true),
        (H::NewMeibu, Timing::Normal, 0, true),
        (H::NewZhixi, Timing::First, 0, true),
        (H::Rangjie, Timing::Normal, 0, true),
        (H::SwdJiang, Timing::Normal, 0, true),
        (H::SwdLieren, Timing::Normal, 0, true),
        (H::SwdReyingzi, Timing::Normal, 0, true),
        (H::SwdZhuhai, Timing::Normal, 0, true),
        (H::Tianxiang, Timing::Normal, 0, true),
        (H::Tianxiang2, Timing::Normal, 0, false),
        (H::YunchouBeige, Timing::Normal, 0, true),
        (H::YunchouJiaoxia, Timing::Normal, 0, true),
        (H::Zhenlie, Timing::Normal, 0, true),
    ] {
        let meta = host.trigger_meta().unwrap();
        assert_eq!(
            (meta.timing, meta.priority, meta.order_selectable),
            (timing, priority, selectable),
            "{host:?}"
        );
    }
}
#[test]
fn actual_silent_first_handler_runs_before_the_normal_order_choice_and_restores() {
    use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
    use engine_native::content::host::HostSkillDecl;
    use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};
    let pkg=content_compiler::compile_source("skill c331_source_a(420006000) locked { trigger damage_before owner_is_target { effect storage_set(self_skill,1,1); } } skill c331_source_b(420006001) locked { trigger damage_before owner_is_target { effect storage_set(self_skill,1,1); } }").unwrap();
    let p = ExtensionPack {
        def: ExtensionDef {
            name: "c331-source-priority",
            version: 1,
            description: "actual source ordering",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![HostSkillDecl {
            name: "c331_source_yueyin",
            behavior: H::HlssYueyin,
            presentation_binding: None,
        }],
        vm_skills: content_native_loader::bridge_vm_skill_decls(&pkg)
            .unwrap()
            .decls,
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(331600, 4, &[&p]).unwrap();
    for name in ["c331_source_a", "c331_source_b", "c331_source_yueyin"] {
        e.debug_grant(1, name);
    }
    e.debug_deal_damage(Some(0), 1, 1, engine_native::event::DamageNature::Normal);
    let c = e.open_choice().unwrap().clone();
    assert_eq!(c.spec.prompt, "trigger_order");
    assert_eq!(
        c.spec.options.len(),
        2,
        "source silent firstDo handler cannot join the player's Normal group"
    );
    assert!(
        e.resolution()
            .stack
            .iter()
            .any(|f| matches!(&f.data,engine_native::event::EventData::Damage(d)if d.amount==2)),
        "first handler already changed the real damage before the order ask"
    );
    let snap =
        engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&e.save()))
            .unwrap();
    let mut r = Engine::restore_with_extensions(&snap, &[&p]).unwrap();
    let cursor = e.log().len();
    for engine in [&mut e, &mut r] {
        let out = engine.submit(GameCommand {
            actor: 1,
            expected_revision: engine.revision(),
            dedup_id: engine.save().last_dedup + 1,
            action: GameAction::Answer {
                choice_id: c.id,
                selection: vec![c.spec.options[0]],
            },
        });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
    }
    assert_eq!(&e.log()[cursor..], r.log());
    let expected = e.save();
    let mut actual = r.save();
    actual.auth_generation = expected.auth_generation;
    assert_eq!(actual, expected);
    assert_eq!(e.state_core().player(1).hp, 2);
}
