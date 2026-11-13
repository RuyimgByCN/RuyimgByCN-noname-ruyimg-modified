//! 暗织的真实用牌/打出历史与目标免疫回归；仅布阵使用 debug 入口。
//! 预期取自 archive/upstream/noname/apps/core/extension/3D精选/character/skill.js:5796-5839。

use std::path::PathBuf;

use content_native_loader::{bridge_pack, decision34_pack9_for_coexistence};
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::{ExtensionDef, ExtensionPack, InitHook};
use engine_native::event::EventKind;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase,
};

fn probe(dsl: &str) -> ExtensionPack {
    let pkg = content_compiler::compile_source_named("dddanzhi-probe", dsl).unwrap();
    let skills = content_native_loader::bridge_vm_skill_decls(&pkg).unwrap();
    ExtensionPack {
        def: ExtensionDef {
            name: "dddanzhi-probe",
            version: 1,
            description: "真实事件回归探针",
        },
        generals: vec![],
        cards: vec![],
        host_skills: vec![],
        vm_skills: skills.decls,
        init_hook: InitHook::None,
        enabled: true,
    }
}

fn deck_first(e: &mut Engine, c: u32) {
    give(e, 0, c);
    e.state_core_mut().player_mut(0).hand.retain(|&id| id != c);
    e.state_core_mut().deck.insert(0, c);
}

fn liangce(e: &mut Engine) {
    let general = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).unwrap().0 == "wdb_zaozhirenjun")
        .unwrap();
    e.assign_general(2, general).unwrap();
    for skill in ["jianbi", "diyjuntun"] {
        e.debug_set_skill_enabled(2, skill, false);
    }
}

fn packs() -> (ExtensionPack, ExtensionPack) {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packages/extensions/d3-upstream-noname/assets/character.json");
    let source = std::fs::read_to_string(path).unwrap().replace("\r\n", "\n");
    let d3 = bridge_pack(&source).unwrap().pack;
    let rest = decision34_pack9_for_coexistence(&[&d3]);
    (d3, rest)
}

fn engine(packs: &[&ExtensionPack]) -> Engine {
    let mut e = Engine::new_with_extensions(2026111301, 3, packs).unwrap();
    let general = (0..e.catalog().general_count() as u16)
        .find(|&g| e.catalog().general_meta(g).unwrap().0 == "ddd_xuelingyun")
        .unwrap();
    e.assign_general(0, general).unwrap();
    e.debug_set_skill_enabled(0, "dddlianer", false);
    for p in 0..3 {
        e.debug_clear_hand(p);
        e.state_core_mut().player_mut(p).max_hp = 20;
        e.state_core_mut().player_mut(p).hp = 20;
    }
    e
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: e.revision() + 1_000_000,
        action,
    });
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    out
}

fn card(e: &Engine, kind: CardKind, black: bool) -> u32 {
    e.catalog()
        .cards
        .iter()
        .enumerate()
        .find(|(_, c)| c.kind == kind && matches!(c.suit, Suit::Spade | Suit::Club) == black)
        .map(|(id, _)| id as u32)
        .unwrap()
}

fn give(e: &mut Engine, player: u8, c: u32) {
    e.state_core_mut().deck.retain(|&id| id != c);
    e.state_core_mut().discard.retain(|&id| id != c);
    e.state_core_mut().table.retain(|&id| id != c);
    for p in 0..3 {
        e.state_core_mut().player_mut(p).hand.retain(|&id| id != c);
    }
    e.state_core_mut().player_mut(player).hand.push(c);
}

fn answer(e: &mut Engine, selection: Vec<ChoiceOption>) {
    let c = e.open_choice().unwrap().clone();
    submit(
        e,
        c.observer,
        GameAction::Answer {
            choice_id: c.id,
            selection,
        },
    );
}

fn pass_responses(e: &mut Engine) {
    for _ in 0..20 {
        let Some(c) = e.open_choice().cloned() else {
            return;
        };
        assert_ne!(
            c.spec.prompt, "dddanzhi_target",
            "unexpected turn-start question"
        );
        let selection = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            vec![]
        } else {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        };
        answer(e, selection);
    }
    panic!("responses did not settle");
}

fn play(e: &mut Engine, actor: u8, c: u32, target: u8) {
    play_targets(e, actor, c, vec![target]);
}

fn play_targets(e: &mut Engine, actor: u8, c: u32, targets: Vec<u8>) {
    give(e, actor, c);
    let round = e.state_core().turn.round;
    e.debug_set_turn(actor, Phase::Play, round);
    e.state_core_mut().turn.sha_used_play_phase = false;
    submit(e, actor, GameAction::PlayCard { card: c, targets });
    pass_responses(e);
}

fn next_turn(e: &mut Engine) {
    e.debug_set_turn(1, Phase::End, 3);
    submit(e, 1, GameAction::NextPhase);
}

fn prepared(packs: &[&ExtensionPack]) -> Engine {
    let mut e = engine(packs);
    let c = card(&e, CardKind::Sha, true);
    play(&mut e, 1, c, 2);
    next_turn(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddanzhi_target");
    e
}

#[test]
fn dddanzhi_discard_without_history_and_last_red_do_not_trigger() {
    // skill.js:5800-5806：只检查最后一次 useCard/respond，弃牌不算历史。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut e = engine(&packs);
    let c = card(&e, CardKind::Sha, true);
    give(&mut e, 2, c);
    e.state_core_mut().player_mut(2).hand.retain(|&id| id != c);
    e.state_core_mut().discard.push(c);
    next_turn(&mut e);
    assert!(
        e.open_choice().is_none(),
        "discard alone must not offer 暗织"
    );

    let mut e = engine(&packs);
    let black = card(&e, CardKind::Sha, true);
    let red = card(&e, CardKind::Sha, false);
    play(&mut e, 1, black, 2);
    play(&mut e, 1, red, 2);
    assert!(!e.state_core().discard.is_empty());
    next_turn(&mut e);
    assert!(
        e.open_choice().is_none(),
        "last red use overrides earlier black use"
    );
}

#[test]
fn dddanzhi_last_response_color_overrides_used_card_color() {
    // skill.js:5800：respond 也参与全局历史，不能只监听 useCard。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    for response_black in [false, true] {
        let mut e = engine(&packs);
        let duel = card(&e, CardKind::Duel, !response_black);
        let response = card(&e, CardKind::Sha, response_black);
        give(&mut e, 1, duel);
        give(&mut e, 2, response);
        e.debug_set_turn(1, Phase::Play, 3);
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: duel,
                targets: vec![2],
            },
        );
        assert_eq!(e.open_choice().unwrap().spec.prompt, "duel_sha");
        answer(&mut e, vec![ChoiceOption::Card(response)]);
        pass_responses(&mut e);
        next_turn(&mut e);
        assert_eq!(e.open_choice().is_some(), response_black);
    }
}

#[test]
fn dddanzhi_target_choice_cancel_and_pending_archive_are_distinct() {
    // skill.js:5808-5819：可取消；所选目标获得技能，不能同时给施术者。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut pending = prepared(&packs);
    let snapshot = decode_archive(&encode_archive(&pending.save())).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
    assert_eq!(pending.state_hash(), restored.state_hash());
    assert_eq!(pending.open_choice(), restored.open_choice());
    let log_start = pending.log().len();
    answer(&mut pending, vec![ChoiceOption::Player(1)]);
    answer(&mut restored, vec![ChoiceOption::Player(1)]);
    let original = pending.save();
    let mut resumed = restored.save();
    assert_eq!(resumed.auth_generation, original.auth_generation + 1);
    resumed.auth_generation = original.auth_generation;
    assert_eq!(original, resumed);
    assert_eq!(&pending.log()[log_start..], restored.log());

    for selected in [Some(0), Some(1), Some(2), None] {
        let mut e = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
        answer(
            &mut e,
            selected.map(ChoiceOption::Player).into_iter().collect(),
        );
        for target in [0, 1] {
            let mut branch = Engine::restore_with_extensions(&e.save(), &packs).unwrap();
            let sha = card(&branch, CardKind::Sha, true);
            let before = branch.state_core().player(target).hp;
            play(&mut branch, 2, sha, target);
            assert_eq!(
                branch.state_core().player(target).hp,
                before - i32::from(selected != Some(target))
            );
        }
        let jiu = card(&e, CardKind::Jiu, true);
        play_targets(&mut e, 2, jiu, vec![]);
        assert_eq!(e.state_core().turn.jiu_active, selected != Some(2));
    }
}

#[test]
fn dddanzhi_only_first_black_target_is_immune_and_red_does_not_consume() {
    // skill.js:5825-5836：黑色牌第一次成为目标时失效并移去临时技能。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    let before = e.state_core().player(1).hp;
    let red = card(&e, CardKind::Sha, false);
    let black = card(&e, CardKind::Sha, true);
    play(&mut e, 2, red, 1);
    assert_eq!(e.state_core().player(1).hp, before - 1);
    play(&mut e, 2, black, 1);
    assert_eq!(e.state_core().player(1).hp, before - 1);
    assert!(
        e.state_core().discard.contains(&black),
        "immune use still consumes its card"
    );
    play(&mut e, 2, black, 1);
    assert_eq!(e.state_core().player(1).hp, before - 2);
}

#[test]
fn dddanzhi_ward_expires_at_turn_end_and_survives_grantor_skill_loss() {
    // skill.js:5819：addTempSkill 的接收者独立持有效果，默认到本回合结束。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    e.debug_set_skill_enabled(0, "dddanzhi", false);
    let before = e.state_core().player(1).hp;
    let black = card(&e, CardKind::Sha, true);
    play(&mut e, 2, black, 1);
    assert_eq!(e.state_core().player(1).hp, before);

    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    e.debug_set_skill_enabled(0, "dddanzhi", false);
    e.debug_set_turn(2, Phase::End, 3);
    submit(&mut e, 2, GameAction::NextPhase);
    let before = e.state_core().player(1).hp;
    let black = card(&e, CardKind::Sha, true);
    play(&mut e, 2, black, 1);
    assert_eq!(e.state_core().player(1).hp, before - 1);
}

#[test]
fn dddanzhi_black_multitarget_card_only_loses_the_selected_target() {
    // skill.js:5831：只从 targets 中移去接收者，其他目标仍正常结算。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    let before = [e.state_core().player(0).hp, e.state_core().player(1).hp];
    let aoe = card(&e, CardKind::SavageAttack, true);
    play_targets(&mut e, 2, aoe, vec![]);
    assert_eq!(e.state_core().player(0).hp, before[0] - 1);
    assert_eq!(e.state_core().player(1).hp, before[1]);
    assert!(e.state_core().discard.contains(&aoe));
    let black = card(&e, CardKind::Sha, true);
    play(&mut e, 2, black, 1);
    assert_eq!(e.state_core().player(1).hp, before[1] - 1);

    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    let chain = card(&e, CardKind::Chain, true);
    play_targets(&mut e, 2, chain, vec![0, 1]);
    assert!(e.state_core().player(0).linked);
    assert!(!e.state_core().player(1).linked);
    assert!(e.state_core().discard.contains(&chain));
}

#[test]
fn dddanzhi_black_implicit_self_target_is_consumed_and_counted() {
    // skill.js:5825-5836：酒、闪电的隐式自身目标同样触发一次免疫。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    for kind in [CardKind::Jiu, CardKind::ShanDian] {
        let mut e = prepared(&packs);
        answer(&mut e, vec![ChoiceOption::Player(2)]);
        let c = card(&e, kind, true);
        let before = e.state_core().player(2).total_used;
        play_targets(&mut e, 2, c, vec![]);
        assert_eq!(e.state_core().player(2).total_used, before + 1);
        assert!(!e.state_core().turn.jiu_active);
        assert!(e.state_core().player(2).judge.is_empty());
        assert!(e.state_core().discard.contains(&c));
        assert!(!e.state_core().table.contains(&c));
        let jiu = card(&e, CardKind::Jiu, true);
        e.state_core_mut().turn.jiu_used = false;
        play_targets(&mut e, 2, jiu, vec![]);
        assert!(e.state_core().turn.jiu_active);
    }
}

#[test]
fn dddanzhi_black_view_as_wugu_skips_only_immune_picker_after_restore() {
    // skill.js:5831：群体目标的移除也适用于黑色转化五谷；选择持久化。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    liangce(&mut e);
    let before: Vec<_> = (0..3)
        .map(|p| e.state_core().player(p).hand.len())
        .collect();
    let black = card(&e, CardKind::Sha, true);
    give(&mut e, 2, black);
    e.debug_set_turn(2, Phase::Play, 3);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: black,
            targets: vec![],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wugu_pick");
    let snapshot = decode_archive(&encode_archive(&e.save())).unwrap();
    let mut restored = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
    let log_start = e.log().len();
    pass_responses(&mut e);
    pass_responses(&mut restored);
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(&e.log()[log_start..], restored.log());
    assert_eq!(e.state_core().player(0).hand.len(), before[0] + 1);
    assert_eq!(e.state_core().player(1).hand.len(), before[1]);
    // 原生良策·余获将被跳过角色留下的牌交给持有者（kernel/tricks.rs）。
    assert_eq!(e.state_core().player(2).hand.len(), before[2] + 2);
    assert!(e.state_core().discard.contains(&black));
}

#[test]
fn dddanzhi_pre_fix_pending_ngsa_retains_content_lock_and_resumes() {
    // 修复前由真实用牌与 NextPhase 挂起导出的 NGSA107，非新实现重新造档。
    let bytes = include_bytes!("fixtures/u01-dddanzhi-pending-v107.ngsa");
    let snapshot = decode_archive(bytes).unwrap();
    assert_eq!(encode_archive(&snapshot), bytes);
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    use engine_native::content::extension::content_lock_id;
    assert_eq!(content_lock_id(&[&d3]), 1_553_573_260_256_134_951);
    assert_eq!(content_lock_id(&[&rest]), 9_097_112_637_251_845_257);
    assert_eq!(content_lock_id(&packs), 11_413_175_519_070_831_972);
    let mut e = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
    let mut roundtrip = e.save();
    assert_eq!(roundtrip.auth_generation, snapshot.auth_generation + 1);
    roundtrip.auth_generation = snapshot.auth_generation;
    assert_eq!(encode_archive(&roundtrip), bytes);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "dddanzhi_target");
    answer(&mut e, vec![ChoiceOption::Player(1)]);
    let before = e.state_core().player(1).hp;
    let black = card(&e, CardKind::Sha, true);
    play(&mut e, 0, black, 1);
    assert_eq!(e.state_core().player(1).hp, before);
    let before = e.state_core().player(0).hp;
    let jiu = card(&e, CardKind::Jiu, true);
    play_targets(&mut e, 0, jiu, vec![]);
    assert!(e.state_core().turn.jiu_active);
    assert_eq!(e.state_core().player(0).hp, before);
    println!(
        "old NGSA {} bytes: exact decode/encode and restored snapshot match; locks unchanged",
        bytes.len()
    );
}

#[test]
fn dddanzhi_wugu_without_skill_restores_held_reveal_cards() {
    let mut e = Engine::new(2026111301, 3);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    assert!(e.catalog().skill_by_name("dddanzhi").is_none());
    let wugu = card(&e, CardKind::Wugu, false);
    give(&mut e, 2, wugu);
    e.debug_set_turn(2, Phase::Play, 3);
    submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: wugu,
            targets: vec![],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "wugu_pick");
    let snapshot = decode_archive(&encode_archive(&e.save())).unwrap();
    let mut restored = Engine::restore(&snapshot).unwrap();
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(e.open_choice(), restored.open_choice());
    let log_start = e.log().len();
    pass_responses(&mut e);
    pass_responses(&mut restored);
    assert_eq!(e.state_hash(), restored.state_hash());
    assert_eq!(&e.log()[log_start..], restored.log());
    assert!((0..3).all(|p| e.state_core().player(p).hand.len() == 1));
    assert!(e.state_core().discard.contains(&wugu));
}

#[test]
fn dddanzhi_no_ward_virtual_group_subset_keeps_global_recipients() {
    // rvm_ops/commit_ext.rs 的虚拟牌允许目标子集；未消费免疫时保留原机器全局范围。
    for (kind, name) in [(CardKind::Wugu, "wugu"), (CardKind::Taoyuan, "taoyuan")] {
        let pack = probe(&format!("skill ddd_subset_probe(9900991) {{ trigger phase_any_end owner_is_target {{ effect use_virtual_card_on(self, {name}, self); }} }}"));
        let mut e = Engine::new_with_extensions(2026111301, 3, &[&pack]).unwrap();
        for p in 0..3 {
            e.debug_clear_hand(p);
            e.state_core_mut().player_mut(p).hp = 2;
        }
        e.debug_grant(0, "ddd_subset_probe");
        e.debug_set_turn(0, Phase::Play, 3);
        submit(&mut e, 0, GameAction::NextPhase);
        pass_responses(&mut e);
        assert!(e.log().iter().any(|log| matches!(log,
            LogEvent::VirtualCardPlayed { kind: actual, targets, .. }
                if *actual == kind && targets == &[0])));
        if kind == CardKind::Wugu {
            assert!((0..3).all(|p| e.state_core().player(p).hand.len() == 1));
        } else {
            assert_eq!(e.state_core().player(0).hp, 2);
            assert_eq!(e.state_core().player(1).hp, 3);
            assert_eq!(e.state_core().player(2).hp, 3);
        }
    }
}

#[test]
fn dddanzhi_final_targets_include_after_trigger_additions() {
    // skill.js:5831：追加目标同样适用；先移去全部初始目标不能跳过后来追加者。
    let (d3, rest) = packs();
    let probe = probe("skill ddd_add_probe(9900992) { trigger card_used owner_is_target { effect choose_add_current_sha_target(self, event_card); } }");
    let packs = [&d3, &rest, &probe];
    for initial in [0, 1] {
        let mut e = prepared(&packs);
        answer(&mut e, vec![ChoiceOption::Player(1)]);
        e.debug_grant(2, "ddd_add_probe");
        let before = [e.state_core().player(0).hp, e.state_core().player(1).hp];
        let black = card(&e, CardKind::Sha, true);
        give(&mut e, 2, black);
        e.debug_set_turn(2, Phase::Play, 3);
        submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: black,
                targets: vec![initial],
            },
        );
        assert_eq!(e.open_choice().unwrap().spec.prompt, "rvm_choice");
        answer(&mut e, vec![ChoiceOption::Player(1 - initial)]);
        pass_responses(&mut e);
        assert_eq!(
            e.state_core().player(0).hp,
            before[0] - 1,
            "initial={initial}"
        );
        assert_eq!(e.state_core().player(1).hp, before[1], "initial={initial}");
        assert!(e.state_core().discard.contains(&black));
        assert!(e.log().iter().any(|log| matches!(
            log,
            LogEvent::SkillTriggered {
                skill: "ddd_add_probe",
                ..
            }
        )));
    }
}

#[test]
fn dddanzhi_actual_equipment_use_updates_last_color() {
    // skill.js:5800 的 useCard 包括装备牌；直接移装备不属于使用。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    for black in [false, true] {
        let mut e = engine(&packs);
        let sha = card(&e, CardKind::Sha, !black);
        play(&mut e, 1, sha, 2);
        let equipment = card(&e, CardKind::Zhuge, black);
        play_targets(&mut e, 1, equipment, vec![]);
        assert!(e.state_core().player(1).equips.contains(&equipment));
        next_turn(&mut e);
        assert_eq!(e.open_choice().is_some(), black);
    }
}

#[test]
fn dddanzhi_direct_equipment_move_neither_changes_history_nor_consumes_ward() {
    // 强制入装备区是移牌：commit_combat.rs 的既有直接装备原语不发 useCard。
    let (d3, rest) = packs();
    let probe = probe("skill ddd_equip_probe(9900994) { trigger phase_any_end owner_is_target { effect equip_weapon_from_draw_pile(self); } }");
    let packs = [&d3, &rest, &probe];
    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    let red = card(&e, CardKind::Sha, false);
    play(&mut e, 2, red, 0);
    let black = card(&e, CardKind::Zhangba, true);
    give(&mut e, 2, black);
    e.state_core_mut()
        .player_mut(2)
        .hand
        .retain(|&c| c != black);
    e.state_core_mut().deck.push(black);
    e.debug_grant(2, "ddd_equip_probe");
    submit(&mut e, 2, GameAction::NextPhase);
    pass_responses(&mut e);
    assert_eq!(e.state_core().player(2).equips, vec![black]);
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "dddanzhi_last_black".into())),
        Some(&0)
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(2, "dddanzhi_immune".into())),
        Some(&1)
    );
    let jiu = card(&e, CardKind::Jiu, true);
    play_targets(&mut e, 2, jiu, vec![]);
    assert!(!e.state_core().turn.jiu_active);
    assert!(e.state_core().player(2).equips.contains(&black));
}

#[test]
fn dddanzhi_nullified_equipment_keeps_old_slot_and_has_no_equip_after() {
    // skill.js:5831 移去自身目标，装备效果及 equipAfter 不执行；使用历史仍是黑色。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    let mut e = prepared(&packs);
    answer(&mut e, vec![ChoiceOption::Player(2)]);
    let red = card(&e, CardKind::Zhuge, false);
    play_targets(&mut e, 2, red, vec![]);
    e.debug_grant(2, "dddjiexing");
    e.state_core_mut()
        .vm_aux
        .flags
        .insert((2, "dddjiexing_used".into()), 1);
    let black = card(&e, CardKind::Zhangba, true);
    let log_start = e.log().len();
    play_targets(&mut e, 2, black, vec![]);
    assert_eq!(e.state_core().player(2).equips, vec![red]);
    assert!(e.state_core().discard.contains(&black));
    assert!(!e.debug_has_skill(2, "丈八蛇矛"));
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(2, "dddjiexing_used".into())),
        Some(&1)
    );
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "dddanzhi_last_black".into())),
        Some(&1)
    );
    assert!(!e.log()[log_start..].iter().any(|log| matches!(
        log,
        LogEvent::Equipped { .. }
            | LogEvent::SkillTriggered {
                skill: "dddjiexing",
                ..
            }
    )));
    play_targets(&mut e, 2, black, vec![]);
    assert_eq!(e.state_core().player(2).equips, vec![black]);
    assert!(e.debug_has_skill(2, "丈八蛇矛"));
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(2, "dddjiexing_used".into())),
        Some(&0)
    );
}

#[test]
fn dddanzhi_nested_use_and_response_do_not_get_overwritten_by_parent() {
    // 外层红决斗先Commit，奇佐黑色副本/打出后，旧CardUsed追色分支不能回写红色。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    for respond in [false, true] {
        let mut e = engine(&packs);
        e.debug_grant(1, "奇佐");
        let carrier = card(&e, CardKind::Jiu, true);
        deck_first(&mut e, carrier);
        let response = e
            .catalog()
            .cards
            .iter()
            .enumerate()
            .find(|(id, c)| {
                *id as u32 != carrier
                    && c.kind == CardKind::Sha
                    && matches!(c.suit, Suit::Spade | Suit::Club)
            })
            .map(|(id, _)| id as u32)
            .unwrap();
        if respond {
            give(&mut e, 2, response);
        }
        let red = card(&e, CardKind::Duel, false);
        give(&mut e, 1, red);
        e.debug_set_turn(1, Phase::Play, 3);
        submit(
            &mut e,
            1,
            GameAction::PlayCard {
                card: red,
                targets: vec![2],
            },
        );
        if respond {
            assert_eq!(e.open_choice().unwrap().spec.prompt, "duel_sha");
            answer(&mut e, vec![ChoiceOption::Card(response)]);
        }
        pass_responses(&mut e);
        assert!(e.state_core().discard.contains(&carrier));
        assert_eq!(
            e.state_core()
                .vm_aux
                .flags
                .get(&(0, "dddanzhi_last_black".into())),
            Some(&1)
        );
        next_turn(&mut e);
        assert_eq!(e.open_choice().unwrap().spec.prompt, "dddanzhi_target");
    }
}

#[test]
fn dddanzhi_sibling_duels_record_execution_order_instead_of_frame_ids() {
    // wn_dutan 的多目标决斗逆序压栈；后一场无色useCard在前场黑色respond后开始。
    let (d3, rest) = packs();
    let probe = probe(include_str!(
        "../../../content/original/ported/offline/wn_dutan.skill"
    ));
    let packs = [&d3, &rest, &probe];
    let mut e = engine(&packs);
    e.debug_grant(1, "wn_dutan");
    let black = card(&e, CardKind::Sha, true);
    give(&mut e, 2, black);
    e.state_core_mut()
        .vm_aux
        .set_selected_target_seats(1, &[2, 0]);
    e.debug_set_turn(1, Phase::Play, 3);
    let skill = e.catalog().skill_by_name("wn_dutan").unwrap();
    submit(&mut e, 1, GameAction::ActivateSkill { skill });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "duel_sha");
    assert_eq!(e.open_choice().unwrap().observer, 2);
    answer(&mut e, vec![ChoiceOption::Card(black)]);
    pass_responses(&mut e);
    assert!(e.log().iter().any(
        |log| matches!(log, LogEvent::CardResponded { card, player: 2, .. } if *card == black)
    ));
    assert_eq!(
        e.state_core()
            .vm_aux
            .flags
            .get(&(0, "dddanzhi_last_black".into())),
        Some(&0)
    );
    next_turn(&mut e);
    assert!(
        e.open_choice().is_none(),
        "末场无色决斗不能被较大frameID的先前响应盖过"
    );
}

#[test]
fn dddanzhi_black_fengze_taoyuan_restores_nullify_window_and_carrier() {
    // play_rules.rs:305 凤泽黑牌转桃园；skill.js:5831 只移去受保护目标。
    use engine_native::content::host::{HostBehavior, HostSkillDecl};
    let (d3, rest) = packs();
    let fengze = ExtensionPack {
        def: ExtensionDef {
            name: "dddanzhi-fengze",
            version: 1,
            description: "既有凤泽宿主行为",
        },
        generals: vec![],
        cards: vec![],
        vm_skills: vec![],
        host_skills: vec![HostSkillDecl {
            name: "fengze",
            behavior: HostBehavior::SwdFengze,
            presentation_binding: None,
        }],
        init_hook: InitHook::None,
        enabled: true,
    };
    let packs = [&d3, &rest, &fengze];
    for nullify in [false, true] {
        let mut e = prepared(&packs);
        answer(&mut e, vec![ChoiceOption::Player(1)]);
        e.debug_grant(2, "fengze");
        for p in 0..3 {
            e.state_core_mut().player_mut(p).hp = 10;
        }
        let black = card(&e, CardKind::Jiu, true);
        let counter = card(&e, CardKind::Nullify, true);
        give(&mut e, 2, black);
        give(&mut e, 0, counter);
        e.debug_set_turn(2, Phase::Play, 3);
        submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: black,
                targets: vec![],
            },
        );
        assert_eq!(e.open_choice().unwrap().spec.prompt, "nullify_window");
        let snapshot = decode_archive(&encode_archive(&e.save())).unwrap();
        let mut restored = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
        assert_eq!(e.open_choice(), restored.open_choice());
        let start = e.log().len();
        if nullify {
            answer(&mut e, vec![ChoiceOption::Card(counter)]);
            answer(&mut restored, vec![ChoiceOption::Card(counter)]);
        }
        pass_responses(&mut e);
        pass_responses(&mut restored);
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(&e.log()[start..], restored.log());
        assert_eq!(e.state_core().player(0).hp, if nullify { 10 } else { 11 });
        assert_eq!(e.state_core().player(1).hp, 10);
        // 现有原生桃园只恢复其他角色，本批保持该已存在的目标模型。
        assert_eq!(e.state_core().player(2).hp, 10);
        assert!(e.state_core().discard.contains(&black));
        assert!(!e.state_core().table.contains(&black));
        assert!(!e
            .state_core()
            .vm_aux
            .flags
            .keys()
            .any(|(_, k)| k.starts_with("dddanzhi_targets:")));
        Engine::restore_with_extensions(
            &decode_archive(&encode_archive(&e.save())).unwrap(),
            &packs,
        )
        .unwrap();
    }
}

#[test]
fn dddanzhi_protects_phase_end_card_until_phase_after() {
    // addTempSkill 默认到phaseAfter；phaseEnd内黑色牌仍在本回合。
    let (d3, rest) = packs();
    for after in [false, true] {
        let mut probe = probe("skill ddd_end_probe(9900993) { trigger turn_ended owner_is_target { if mark_count(self, 9900993) == 0 { effect add_mark(self, 9900993, 1); effect use_virtual_card_on(self, juedou, 1); } } }");
        probe.vm_skills[0].trigger.event = if after {
            EventKind::TurnPhaseAfter
        } else {
            EventKind::TurnPhaseEnd
        };
        probe.vm_skills[0].filter = engine_native::rvm_bridge::VmFilterPolicy::OwnerIsTarget;
        let packs = [&d3, &rest, &probe];
        let mut e = prepared(&packs);
        answer(&mut e, vec![ChoiceOption::Player(0)]);
        e.debug_set_skill_enabled(0, "dddanzhi", false);
        e.debug_grant(2, "ddd_end_probe");
        e.debug_grant(2, "奇佐");
        let black = card(&e, CardKind::Jiu, true);
        deck_first(&mut e, black);
        let before = e.state_core().player(0).hp;
        e.debug_set_turn(2, Phase::End, 3);
        submit(&mut e, 2, GameAction::NextPhase);
        pass_responses(&mut e);
        assert_eq!(
            e.state_core().player(0).hp,
            before - if after { 2 } else { 1 },
            "after={after}"
        );
        assert!(e.state_core().discard.contains(&black));
        assert!(!e
            .state_core()
            .vm_aux
            .flags
            .contains_key(&(0, "dddanzhi_immune".into())));
    }
}

#[test]
fn dddanzhi_wugu_pending_init_restores_and_nullification_cleans_marker() {
    // 五谷开无懈窗时尚未亮牌；旧snapshot的空cards/pickers明确表示Init。
    let (d3, rest) = packs();
    let packs = [&d3, &rest];
    for nullify in [false, true] {
        let mut e = prepared(&packs);
        answer(&mut e, vec![ChoiceOption::Player(1)]);
        liangce(&mut e);
        let nullify_card = card(&e, CardKind::Nullify, true);
        give(&mut e, 0, nullify_card);
        let black = card(&e, CardKind::Sha, true);
        give(&mut e, 2, black);
        e.debug_set_turn(2, Phase::Play, 3);
        submit(
            &mut e,
            2,
            GameAction::PlayCard {
                card: black,
                targets: vec![],
            },
        );
        assert_eq!(e.open_choice().unwrap().spec.prompt, "nullify_window");
        assert!(e
            .state_core()
            .vm_aux
            .flags
            .keys()
            .any(|(_, k)| k.starts_with("dddanzhi_targets:")));
        let snapshot = decode_archive(&encode_archive(&e.save())).unwrap();
        let mut restored = Engine::restore_with_extensions(&snapshot, &packs).unwrap();
        assert_eq!(e.open_choice(), restored.open_choice());
        let start = e.log().len();
        if nullify {
            for branch in [&mut e, &mut restored] {
                answer(branch, vec![ChoiceOption::Card(nullify_card)]);
            }
        }
        pass_responses(&mut e);
        pass_responses(&mut restored);
        assert_eq!(e.state_hash(), restored.state_hash());
        assert_eq!(&e.log()[start..], restored.log());
        assert!(e.state_core().discard.contains(&black));
        assert!(!e
            .state_core()
            .vm_aux
            .flags
            .keys()
            .any(|(_, k)| k.starts_with("dddanzhi_targets:")));
        assert_eq!(e.state_core().player(1).hand.len(), 0);
        let finished = decode_archive(&encode_archive(&e.save())).unwrap();
        Engine::restore_with_extensions(&finished, &packs).unwrap();
    }
}
