//! A reachable VM local caches an opponent's private hand color before HOST_CHOICE.
use engine_native::ai::BoundedPlanner;
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook, VmSkillDecl};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use rule_vm::bytecode::{caps, effect_tag, op, query_tag};

fn constant(code: &mut Vec<u8>, value: u32) {
    code.push(op::CONST_U32);
    code.extend(value.to_le_bytes());
}

pub fn pack() -> ExtensionPack {
    let mut code = vec![];
    constant(&mut code, 3); // rule seat 3 = native opponent 2
    constant(&mut code, 1); // red
    code.extend([op::HOST_QUERY, query_tag::HAND_COLOR_COUNT, op::STORE, 0]);
    constant(&mut code, 0);
    constant(&mut code, 1);
    code.extend([
        op::HOST_CHOICE,
        2,
        op::STORE,
        1,
        op::LOAD,
        1,
        op::JUMP_IF_FALSE,
    ]);
    let jump = code.len();
    code.extend([0, 0]);
    constant(&mut code, 1);
    constant(&mut code, 3);
    code.extend([
        op::LOAD,
        0,
        op::HOST_EFFECT,
        effect_tag::DEAL_DAMAGE as u8,
        op::HALT,
    ]);
    let destination = code.len();
    code[jump..jump + 2].copy_from_slice(&((destination - jump - 2) as i16).to_le_bytes());
    constant(&mut code, 1);
    constant(&mut code, 4);
    constant(&mut code, 1);
    code.extend([op::HOST_EFFECT, effect_tag::DEAL_DAMAGE as u8, op::HALT]);
    ExtensionPack {
        def: ExtensionDef {
            name: "nc10-opaque",
            version: 1,
            description: "cached hidden color",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        init_hook: InitHook::None,
        enabled: true,
        vm_skills: vec![VmSkillDecl {
            name: "nc10_opaque".into(),
            module: rule_vm::BytecodeModule {
                name: "nc10-opaque".into(),
                code,
                local_count: 2,
                max_stack: 8,
                entry_pc: 0,
                logic_hash: [41; 32],
                caps_mask: caps::RUN_QUERIES | caps::REQUEST_CHOICE | caps::EMIT_EFFECTS,
            },
            trigger: engine_native::catalog::TriggerMeta {
                event: engine_native::EventKind::PhaseEnd,
                timing: engine_native::Timing::Normal,
                priority: 0,
                optional: false,
                extra_events: vec![],
            },
            filter: engine_native::rvm_bridge::VmFilterPolicy::Always,
        }],
    }
}

pub fn fixture(pack: &ExtensionPack, red: bool) -> Engine {
    let mut e = Engine::new_with_extensions(991_100, 4, &[pack]).unwrap();
    e.debug_set_turn(0, Phase::Play, 1);
    e.debug_grant(0, "nc10_opaque");
    for index in 0..4 {
        let deck_index = e
            .state_core()
            .deck
            .iter()
            .position(|id| {
                let suit = e.catalog().cards[*id as usize].suit;
                matches!(
                    suit,
                    engine_native::catalog::Suit::Heart | engine_native::catalog::Suit::Diamond
                ) == red
            })
            .unwrap();
        let core = e.state_core_mut();
        std::mem::swap(&mut core.players[2].hand[index], &mut core.deck[deck_index]);
    }
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: 1,
        action: GameAction::NextPhase,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_choice");
    Engine::restore_with_extensions(&e.save(), &[pack]).unwrap();
    e
}

