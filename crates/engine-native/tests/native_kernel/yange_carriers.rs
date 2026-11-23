use super::*;
use engine_native::catalog::{CardDef, EquipSlot};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::state::SkillInstance;

#[test]
fn yange_cloned_equip_dismantle_snatch_and_sha_respond() {
    let pack = ExtensionPack {
        def: ExtensionDef {
            name: "yange-carrier-test-ext",
            version: 1,
            description: "test pack for yange cloned equip & dizaizhen",
        },
        generals: Vec::new(),
        cards: vec![
            CardDef::armor("yange_test_armor").with_equip_skills(&["闭月"]),
            CardDef::basic("yange_test_dizaizhen", CardKind::Dizaizhen),
        ],
        host_skills: Vec::new(),
        vm_skills: Vec::new(),
        init_hook: InitHook::None,
        enabled: true,
    };
    let mut e = Engine::new_with_extensions(2026_9491, 3, &[&pack]).expect("ext engine");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let armor_tmpl = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "yange_test_armor")
        .expect("yange_test_armor template") as u32;
    let dz_tmpl = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.name == "yange_test_dizaizhen")
        .expect("yange_test_dizaizhen template") as u32;
    let shan_tmpl = e
        .catalog()
        .cards
        .iter()
        .position(|c| c.kind == CardKind::Shan)
        .expect("standard Shan template") as u32;
    let biyue_skill = e.catalog().skill_by_name("闭月").expect("闭月 skill");
    let c_eq = e.debug_place(1, "sha");
    let c_shan = e.debug_place(1, "sha");
    let c_dz = e.debug_place(0, "sha");
    let dism = e.debug_place(0, "dismantle");
    let snatch = e.debug_place(0, "snatch");
    {
        let mut snap = e.save();
        snap.core.players[1].hand.retain(|&c| c != c_eq && c != c_shan);
        snap.core.deck.push(c_shan);
        snap.core.players[1].equips.push(c_eq);
        snap.core.skills.insert(
            (1, biyue_skill),
            SkillInstance {
                owner: 1,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        snap.core
            .vm_aux
            .flags
            .insert((0, format!("gj_yange_clone:{c_eq}")), armor_tmpl + 1);
        snap.core.declared_kinds.insert(c_eq, CardKind::Armor);
        snap.core.declared_slots.insert(c_eq, EquipSlot::Armor);

        // Also prepare cloned `Dizaizhen` on `Sha` carrier `c_dz` in 0.hand:
        snap.core
            .vm_aux
            .flags
            .insert((0, format!("gj_yange_clone:{c_dz}")), dz_tmpl + 1);
        snap.core.declared_kinds.insert(c_dz, CardKind::Dizaizhen);

        e = Engine::restore_with_extensions(&snap, &[&pack])
            .expect("restore Dismantle cloned equip setup");
    }
    assert!(e.debug_has_skill(1, "闭月"));

    // 1) Play cloned `Dizaizhen` on `Sha` carrier -> resolves and clears `table` (`tricks.rs:124`)!
    let _ = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: c_dz,
            targets: vec![],
        },
    );
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = answer(&mut e, ChoiceOption::Pass);
        } else {
            break;
        }
    }
    assert!(
        e.state_core().table.is_empty(),
        "cloned Dizaizhen must be removed from table after resolving"
    );
    assert!(
        e.state_core().discard.contains(&c_dz),
        "cloned Dizaizhen moved to discard"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, format!("gj_yange_clone:{c_dz}")))
            .copied()
            .unwrap_or(0),
        0,
        "gj_yange_clone cleared after Dizaizhen settles"
    );

    // 2) Dismantle cloned equip on `Sha` carrier -> MUST revoke `"闭月"` skill!
    e.debug_clear_hand(1);
    let out_d = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dism,
            targets: vec![1],
        },
    );
    assert!(!matches!(out_d, CommandOutcome::Rejected(_)), "Dismantle play: {out_d:?}");
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = answer(&mut e, ChoiceOption::Pass);
        } else {
            break;
        }
    }
    assert!(
        !e.state_core().player(1).equips.contains(&c_eq),
        "Dismantle removed cloned equip from 1.equips"
    );
    assert!(
        !e.debug_has_skill(1, "闭月"),
        "Dismantle on cloned equip on Sha carrier MUST revoke equip skills"
    );

    // 3) Snatch cloned equip on `Sha` carrier -> MUST revoke skill, move to 0.hand, AND clear `gj_yange_clone` flags!
    {
        let mut snap = e.save();
        snap.core.discard.retain(|&c| c != c_eq);
        snap.core.players[1].equips.push(c_eq);
        snap.core.skills.insert(
            (1, biyue_skill),
            SkillInstance {
                owner: 1,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        snap.core
            .vm_aux
            .flags
            .insert((0, format!("gj_yange_clone:{c_eq}")), armor_tmpl + 1);
        snap.core.declared_kinds.insert(c_eq, CardKind::Armor);
        snap.core.declared_slots.insert(c_eq, EquipSlot::Armor);
        e = Engine::restore_with_extensions(&snap, &[&pack])
            .expect("restore Snatch cloned equip setup");
    }
    e.debug_clear_hand(1);
    let out_s = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: snatch,
            targets: vec![1],
        },
    );
    assert!(!matches!(out_s, CommandOutcome::Rejected(_)), "Snatch play: {out_s:?}");
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "nullify_window" {
            let _ = answer(&mut e, ChoiceOption::Pass);
        } else {
            break;
        }
    }
    assert!(
        e.state_core().player(0).hand.contains(&c_eq),
        "Snatch moved cloned equip to thief 0.hand"
    );
    assert!(
        !e.debug_has_skill(1, "闭月"),
        "Snatch on cloned equip on Sha carrier MUST revoke equip skills"
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, format!("gj_yange_clone:{c_eq}")))
            .copied()
            .unwrap_or(0),
        0,
        "Snatch MUST clear gj_yange_clone flag in thief's hand"
    );
    assert!(!e.state_core().declared_kinds.contains_key(&c_eq));
    assert!(!e.state_core().declared_slots.contains_key(&c_eq));

    // 4) `ShaResolveMachine` + `snap/asks/sha.rs` offering `c_shan` (cloned `Shan` on `Sha` carrier)!
    let real_sha = e.debug_place(0, "sha");
    {
        let mut snap = e.save();
        for p in &mut snap.core.players {
            p.hand.retain(|&c| c != c_shan);
            p.equips.retain(|&c| c != c_shan);
            p.judge.retain(|&c| c != c_shan);
            p.off_court.retain(|&c| c != c_shan);
        }
        snap.core.deck.retain(|&c| c != c_shan);
        snap.core.discard.retain(|&c| c != c_shan);
        snap.core.table.retain(|&c| c != c_shan);
        snap.core.players[1].hand.push(c_shan);
        snap.core
            .vm_aux
            .flags
            .insert((0, format!("gj_yange_clone:{c_shan}")), shan_tmpl + 1);
        snap.core.declared_kinds.insert(c_shan, CardKind::Shan);
        e = Engine::restore_with_extensions(&snap, &[&pack])
            .expect("restore ShaResolve cloned Shan setup");
    }
    let hp1_pre = e.debug_hp(1);
    let _ = submit_cmd(
        &mut e,
        0,
        GameAction::PlayCard {
            card: real_sha,
            targets: vec![1],
        },
    );
    let sha_ask = e.open_choice().cloned().expect("respond_shan");
    assert_eq!(sha_ask.spec.prompt, "respond_shan");
    assert!(
        sha_ask.spec.options.contains(&ChoiceOption::Card(c_shan)),
        "respond_shan must offer cloned Shan on Sha carrier"
    );
    let mut restored = Engine::restore_with_extensions(&e.save(), &[&pack])
        .expect("restore parked on respond_shan");
    let r_ask = restored.open_choice().cloned().expect("restored respond_shan");
    assert!(
        r_ask.spec.options.contains(&ChoiceOption::Card(c_shan)),
        "snapshot ask derivation (snap/asks/sha.rs) must preserve cloned Shan option"
    );
    let _ = answer(&mut restored, ChoiceOption::Card(c_shan));
    assert_eq!(restored.debug_hp(1), hp1_pre, "cloned Shan dodged Sha cleanly");
    assert!(restored.state_core().table.is_empty());
}
