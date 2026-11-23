use super::common::*;
use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, LockedStartMode,
};
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, RejectReason};

#[test]
fn u05_b7b_r3_tianxianjiu_decoupled_from_jiu_and_caoyao_self_heal() {
    let (v, mut e) = start_shahai_swd_only(9501, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let txj1 = find_kind_nth(&e, CardKind::Tianxianjiu, 0);
    let txj2 = find_kind_nth(&e, CardKind::Tianxianjiu, 1);
    let jiu = find_kind(&e, CardKind::Jiu);
    let sha1 = find_kind_nth(&e, CardKind::Sha, 0);
    let sha2 = find_kind_nth(&e, CardKind::Sha, 1);
    let caoyao = find_kind(&e, CardKind::Caoyao);

    // 1. Caoyao self-heal (#56 note): damaged seat 0 can target self.
    clear_hand_to_discard(&mut e, &v, 0);
    e.debug_set_hp(0, 2);
    force_move_card_to_hand(&mut e, &v, 0, caoyao);
    let cy_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: caoyao,
            targets: vec![0],
        },
    );
    assert!(
        !matches!(cy_out, CommandOutcome::Rejected(_)),
        "Caoyao must allow self-heal: {cy_out:?}"
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hp, 3, "Caoyao healed self +1 HP");

    // 2. Tianxianjiu alone: does NOT set jiu_used/jiu_active, deals base 1 Sha damage + draws 2!
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    e.debug_set_hp(1, 4);
    force_move_card_to_hand(&mut e, &v, 0, txj1);
    force_move_card_to_hand(&mut e, &v, 0, txj2);
    force_move_card_to_hand(&mut e, &v, 0, jiu);
    force_move_card_to_hand(&mut e, &v, 0, sha1);

    let out_txj = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: txj1,
            targets: vec![],
        },
    );
    assert!(!matches!(out_txj, CommandOutcome::Rejected(_)));
    drain_pass_all(&mut e);
    assert!(!e.state_core().turn.jiu_used, "tianxianjiu must NOT consume jiu_used");
    assert!(!e.state_core().turn.jiu_active, "tianxianjiu must NOT set jiu_active (+1 Sha dmg)");

    // Second tianxianjiu while skill active / usable=1 spent must be rejected
    let out_txj2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: txj2,
            targets: vec![],
        },
    );
    assert!(
        matches!(out_txj2, CommandOutcome::Rejected(RejectReason::CardLimit)),
        "second tianxianjiu same phase must be rejected: {out_txj2:?}"
    );

    // Real Sha with tianxianjiu only -> 1 damage (NOT 2!), draws 2!
    let hand_before_sha = e.state_core().player(0).hand.len();
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        3,
        "tianxianjiu must add 0 Sha damage (hp 4 -> 3)"
    );
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before_sha - 1 + 2,
        "tianxianjiu draws 2 on Sha damage"
    );

    // Same turn after tianxianjiu: Jiu is STILL playable independently!
    let out_jiu = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: jiu,
            targets: vec![],
        },
    );
    assert!(
        !matches!(out_jiu, CommandOutcome::Rejected(_)),
        "Jiu must still be usable after Tianxianjiu in same turn: {out_jiu:?}"
    );
    drain_pass_all(&mut e);
    assert!(e.state_core().turn.jiu_active);

    // Next turn: Jiu first -> Tianxianjiu STILL usable -> Sha deals 2 dmg AND draws 2!
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    e.debug_set_hp(1, 4);
    force_move_card_to_hand(&mut e, &v, 0, jiu);
    force_move_card_to_hand(&mut e, &v, 0, txj1);
    force_move_card_to_hand(&mut e, &v, 0, sha2);

    let out_jiu2 = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
    assert!(
        !matches!(out_jiu2, CommandOutcome::Rejected(_)),
        "Jiu must succeed on next turn {:?}: {out_jiu2:?}",
        e.turn()
    );
    drain_pass_all(&mut e);
    let out_txj_after_jiu = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: txj1,
            targets: vec![],
        },
    );
    assert!(
        !matches!(out_txj_after_jiu, CommandOutcome::Rejected(_)),
        "Tianxianjiu must still be usable after Jiu in same turn: {out_txj_after_jiu:?}"
    );
    drain_pass_all(&mut e);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, 2, "Jiu +1 makes Sha deal 2 damage");
    assert_eq!(e.state_core().player(0).hand.len(), 2, "Tianxianjiu draws 2 alongside Jiu");
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r3_lingjiandai_gains_3_hslingjian() {
    let (v, mut e) = start_shahai_swd_only(9502, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);

    let ljd = find_card_name(&e, "lingjiandai");
    force_move_card_to_hand(&mut e, &v, 0, ljd);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: ljd,
            targets: vec![],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "{out:?}");
    drain_pass_all(&mut e);
    let gained = &e.state_core().player(0).hand;
    assert_eq!(gained.len(), 3, "lingjiandai must gain 3 Hslingjian cards, got {:?}", gained);
    assert!(
        gained.iter().all(|&c| e.catalog().card_kind(c) == CardKind::Hslingjian),
        "all 3 gained cards must be Hslingjian"
    );
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r3_yinmilichang_equip1_and_equip2_temp_expiration() {
    let (v, mut e) = start_shahai_swd_only(9503, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // Part A: `hslingjian_yinmilichang_equip1` -> tempHide expires at TARGET's phaseBeginStart!
    grant_skill(&mut e, &v, 0, "hslingjian_yinmilichang_equip1");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    clear_hand_to_discard(&mut e, &v, 2);
    let sha1 = find_kind_nth(&e, CardKind::Sha, 0);
    let sha2 = find_kind_nth(&e, CardKind::Sha, 1);
    force_move_card_to_hand(&mut e, &v, 0, sha1);
    force_move_card_to_hand(&mut e, &v, 2, sha2);

    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_yinmi_equip1_target");
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().player(2).yinmi_active, "equip1 tempHide active on target 2");

    // Seat 2 survives through seat 0's turnEnd and into seat 1's turn!
    step_full_turn_to(&mut e, 1);
    assert!(
        e.state_core().player(2).yinmi_active,
        "equip1 tempHide on seat 2 must persist through seat 0 phaseAfter until seat 2 turn begins"
    );
    // Once seat 2's turn begins (`phaseBeginStart`), `yinmi_active` expires!
    step_full_turn_to(&mut e, 2);
    assert!(
        !e.state_core().player(2).yinmi_active,
        "equip1 tempHide must expire at target's phaseBeginStart"
    );

    // Part B: `hslingjian_yinmilichang_equip2` -> addTempSkill expires at current turn's phaseAfter!
    remove_skill(&mut e, &v, 0, "hslingjian_yinmilichang_equip1");
    grant_skill(&mut e, &v, 1, "hslingjian_yinmilichang_equip2");
    let _ = submit(
        &mut e,
        2,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert!(e.state_core().player(1).yinmi_active, "equip2 grants qianxing after damage");

    // End seat 2's turn -> advancing to seat 0's turn passes seat 2's `phaseAfter`!
    step_full_turn_to(&mut e, 0);
    assert!(
        !e.state_core().player(1).yinmi_active,
        "equip2 temporary qianxing must expire at current turn's phaseAfter"
    );
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r3_duanzao_excludes_unique_nopower_reforge_and_grants_yuchan() {
    let (v, mut e) = start_shahai_swd_only(9504, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let duanzao_def = grant_skill(&mut e, &v, 0, "_lingjianduanzao");
    clear_hand_to_discard(&mut e, &v, 0);

    let xyj = find_kind(&e, CardKind::Xuanyuanjian);
    let dhz = find_kind(&e, CardKind::Donghuangzhong);
    let fxq = find_card_name(&e, "fuxiqin");
    let normal_wep = find_kind(&e, CardKind::Weapon);
    let mat1 = find_card_name(&e, "xingtigaizao");
    let mat2 = find_card_name(&e, "shijianhuisu");
    let yuchan = find_kind(&e, CardKind::Yuchanqian);

    // 1. Only unique/nopower artifacts + material -> _lingjianduanzao cannot activate
    force_move_card_to_hand(&mut e, &v, 0, xyj);
    force_move_card_to_hand(&mut e, &v, 0, dhz);
    force_move_card_to_hand(&mut e, &v, 0, fxq);
    force_move_card_to_hand(&mut e, &v, 0, mat1);
    let out_blocked = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    assert!(
        matches!(out_blocked, CommandOutcome::Rejected(_)),
        "unique/nopower artifacts must NOT be forgeable: {out_blocked:?}"
    );

    // 2. Add a normal weapon -> ONLY normal_wep offered; unique/nopower still excluded!
    force_move_card_to_hand(&mut e, &v, 0, normal_wep);
    let out_ok = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    assert!(!matches!(out_ok, CommandOutcome::Rejected(_)));
    let ch = e.open_choice().expect("u05_duanzao_equip");
    assert_eq!(ch.spec.prompt, "u05_duanzao_equip");
    assert!(!ch.spec.options.contains(&ChoiceOption::Card(xyj)), "Xuanyuanjian excluded");
    assert!(!ch.spec.options.contains(&ChoiceOption::Card(dhz)), "Donghuangzhong excluded");
    assert!(!ch.spec.options.contains(&ChoiceOption::Card(fxq)), "Fuxiqin excluded");
    assert!(ch.spec.options.contains(&ChoiceOption::Card(normal_wep)));
    answer_open(&mut e, vec![ChoiceOption::Card(normal_wep)]);
    answer_open(&mut e, vec![ChoiceOption::Card(mat1)]);
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);
    drain_pass_all(&mut e);

    // 3. Already-forged `normal_wep` in equips must NOT be re-forgeable!
    force_move_card_to_hand(&mut e, &v, 0, mat2);
    let out_reforge = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    assert!(
        matches!(out_reforge, CommandOutcome::Rejected(_)),
        "already-forged equip must NOT be re-forgeable: {out_reforge:?}"
    );

    // 4. Forging with `Yuchanqian` without pre-granting `g_yuchan_equip`:
    //    `forged_lingjian_skill_name` must resolve non-None & auto-grant `g_yuchan_equip`!
    let normal_armor = find_kind(&e, CardKind::Baihupifeng);
    force_move_card_to_hand(&mut e, &v, 0, normal_armor);
    force_move_card_to_hand(&mut e, &v, 0, yuchan);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    answer_open(&mut e, vec![ChoiceOption::Card(normal_armor)]);
    answer_open(&mut e, vec![ChoiceOption::Card(yuchan)]);
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);
    drain_pass_all(&mut e);
    assert!(
        engine_native::kernel::forged_lingjian_skill_name(e.state_core(), e.catalog(), 0, normal_armor).is_some(),
        "forged_lingjian_skill_name must not return None for Yuchan* forged equip"
    );
    let g_yc_eq = e.catalog().skill_by_name("g_yuchan_equip").expect("g_yuchan_equip");
    assert!(e.state_core().has_skill(0, g_yc_eq), "forging Yuchan* grants g_yuchan_equip");
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r3_jiguanyaoshu_skill_only_triggers_on_equip_zone_loss() {
    let (v, mut e) = start_shahai_swd_only(9505, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    grant_skill(&mut e, &v, 1, "jiguanyaoshu_skill");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);

    let wep_hand = find_kind_nth(&e, CardKind::Weapon, 0);
    let wep_equip = find_kind_nth(&e, CardKind::Weapon, 1);
    let dis1 = find_kind_nth(&e, CardKind::Dismantle, 0);
    let dis2 = find_kind_nth(&e, CardKind::Dismantle, 1);

    // 1. Hand-zone equip discarded off-turn: must NOT trigger jiguanyaoshu_skill!
    force_move_card_to_hand(&mut e, &v, 1, wep_hand);
    force_move_card_to_hand(&mut e, &v, 0, dis1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dis1,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        e.state_core().player(1).hand.is_empty(),
        "hand-zone equip discard must NOT trigger jiguanyaoshu_skill, hand={:?}",
        e.state_core().player(1).hand
    );

    // 2. Equip-zone equip discarded off-turn: MUST trigger jiguanyaoshu_skill!
    advance_to_play(&mut e, 1);
    force_move_card_to_hand(&mut e, &v, 1, wep_equip);
    let _ = submit(&mut e, 1, GameAction::PlayCard { card: wep_equip, targets: vec![] });
    drain_pass_all(&mut e);
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    force_move_card_to_hand(&mut e, &v, 0, dis2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: dis2,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hand.len(), 1);
    assert_eq!(e.catalog().card_kind(e.state_core().player(1).hand[0]), CardKind::Hslingjian);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r3_sadengjinhuan_sha_hit_jiu_guiyan_and_vm_yuchan_shan() {
    let (v, mut e) = start_shahai_swd_only(9506, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // Part A: standalone `sadengjinhuan` + `Jiu` -> dodged -> red judge -> no extra shan ->
    // real `shaHit` deals 2 damage (`Jiu` +1 preserved)!
    grant_skill(&mut e, &v, 0, "sadengjinhuan");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let jiu = find_kind(&e, CardKind::Jiu);
    let sha = find_kind_nth(&e, CardKind::Sha, 0);
    let shan1 = find_kind_nth(&e, CardKind::Shan, 0);
    force_move_card_to_hand(&mut e, &v, 0, jiu);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    force_move_card_to_hand(&mut e, &v, 1, shan1);
    e.debug_set_hp(1, 4);
    e.debug_set_deck_top_by(|d| d.suit.is_heart());

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
    drain_pass_all(&mut e);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    answer_open(&mut e, vec![ChoiceOption::Card(shan1)]); // seat 1 dodges
    assert_eq!(e.open_choice().unwrap().spec.prompt, "sadengjinhuan_judge_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().player(1).hp,
        2,
        "sadengjinhuan hit must trigger shaHit preserving Jiu +1 damage (4 -> 2)"
    );

    // Part B: `sadengjinhuan_extra_shan` includes VM view-as `Shan` & `Longdan + Yuchanqian`!
    step_full_turn_to(&mut e, 0);
    grant_skill(&mut e, &v, 1, engine_native::content::standard::LONG_DAN);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let sha2 = find_kind_nth(&e, CardKind::Sha, 1);
    let shan2 = find_kind_nth(&e, CardKind::Shan, 1);
    let yc_qian = find_kind(&e, CardKind::Yuchanqian);
    force_move_card_to_hand(&mut e, &v, 0, sha2);
    force_move_card_to_hand(&mut e, &v, 1, shan2);
    force_move_card_to_hand(&mut e, &v, 1, yc_qian);
    e.debug_set_hp(1, 4);
    e.debug_set_deck_top_by(|d| d.suit.is_heart());

    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    answer_open(&mut e, vec![ChoiceOption::Card(shan2)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "sadengjinhuan_judge_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    let extra = e.open_choice().expect("sadengjinhuan_extra_shan").clone();
    assert_eq!(extra.spec.prompt, "sadengjinhuan_extra_shan");
    assert!(
        extra.spec.options.contains(&ChoiceOption::Card(yc_qian)),
        "sadengjinhuan_extra_shan must allow longdan + Yuchanqian as Shan: {:?}",
        extra.spec.options
    );
    answer_open(&mut e, vec![ChoiceOption::Card(yc_qian)]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, 4, "extra Shan prevented damage");
    assert_eq!(total_card_set(&e), init_set);

    // Part C: `sadengjinhuan_extra_shan` also includes VM `view_as Shan` hand & equip cards!
    let swd_cards_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("read swd cards.json");
    let swd_cards_pack = content_native_loader::bridge_pack(&swd_cards_json)
        .expect("bridge swd cards")
        .pack;
    let vm_pkg = content_compiler::compile_source_named(
        "test_vm_shan",
        "skill test_vm_shan(9506001) { view_as shan response heart from hand_equip; }",
    )
    .expect("compile test_vm_shan");
    let vm_decls = content_native_loader::bridge_vm_skill_decls(&vm_pkg)
        .expect("bridge test_vm_shan")
        .decls;
    let vm_pack = engine_native::content::extension::ExtensionPack {
        def: engine_native::content::extension::ExtensionDef {
            name: "test-vm-shan-pack",
            version: 1,
            description: "vm view_as shan test",
        },
        generals: Vec::new(),
        cards: Vec::new(),
        host_skills: Vec::new(),
        vm_skills: vm_decls,
        init_hook: engine_native::content::extension::InitHook::None,
        enabled: true,
    };
    let packs = [&swd_cards_pack, &vm_pack];
    let mut evm = engine_native::Engine::new_with_extensions(9510, 2, &packs).expect("vm engine");
    advance_to_play(&mut evm, 0);
    let vm_skill_def = evm
        .catalog()
        .skill_by_name("test_vm_shan")
        .expect("test_vm_shan");
    {
        let mut snap = evm.save();
        for p in 0..2 {
            let drain: Vec<u32> = snap.core.players[p].hand.drain(..).collect();
            snap.core.discard.extend(drain);
            let eqs: Vec<u32> = snap.core.players[p].equips.drain(..).collect();
            snap.core.discard.extend(eqs);
        }
        snap.core.skills.insert(
            (1, vm_skill_def),
            engine_native::state::SkillInstance {
                owner: 1,
                enabled: true,
                once_used: false,
                expires_end_of_turn: false,
                rounds: 0,
            },
        );
        evm = engine_native::Engine::restore_with_extensions(&snap, &packs).expect("restore vm");
    }
    let sadeng = find_kind(&evm, CardKind::Sadengjinhuan);
    let vm_sha = find_kind(&evm, CardKind::Sha);
    let vm_shan = find_kind(&evm, CardKind::Shan);
    let heart_tao = find_kind_suit(&evm, CardKind::Tao, |s| s == engine_native::catalog::Suit::Heart)
        .expect("heart tao");
    let heart_equip = (0..evm.catalog().cards.len() as u32)
        .find(|&id| {
            let cd = &evm.catalog().cards[id as usize];
            cd.kind.is_equip() && cd.suit == engine_native::catalog::Suit::Heart
        })
        .expect("heart equip");
    let move_hand = |eng: &mut engine_native::Engine, seat: usize, cid: u32| {
        let mut snap = eng.save();
        snap.core.deck.retain(|&c| c != cid);
        snap.core.discard.retain(|&c| c != cid);
        snap.core.table.retain(|&c| c != cid);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|&c| c != cid);
            pl.equips.retain(|&c| c != cid);
        }
        snap.core.players[seat].hand.push(cid);
        *eng = engine_native::Engine::restore_with_extensions(&snap, &packs).expect("move");
    };
    move_hand(&mut evm, 0, sadeng);
    let _ = submit(&mut evm, 0, GameAction::PlayCard { card: sadeng, targets: vec![] });
    move_hand(&mut evm, 0, vm_sha);
    move_hand(&mut evm, 1, vm_shan);
    {
        let mut snap = evm.save();
        snap.core.deck.retain(|&c| c != heart_equip && c != heart_tao);
        snap.core.discard.retain(|&c| c != heart_equip && c != heart_tao);
        snap.core.table.retain(|&c| c != heart_equip && c != heart_tao);
        for pl in snap.core.players.iter_mut() {
            pl.hand.retain(|&c| c != heart_equip && c != heart_tao);
            pl.equips.retain(|&c| c != heart_equip && c != heart_tao);
        }
        snap.core.players[1].equips.push(heart_equip);
        snap.core.players[1].hand.push(heart_tao);
        evm = engine_native::Engine::restore_with_extensions(&snap, &packs).expect("equip");
    }
    evm.debug_set_deck_top_by(|d| d.suit.is_heart());
    let _ = submit(&mut evm, 0, GameAction::PlayCard { card: vm_sha, targets: vec![1] });
    answer_open(&mut evm, vec![ChoiceOption::Card(vm_shan)]);
    assert_eq!(evm.open_choice().unwrap().spec.prompt, "sadengjinhuan_judge_confirm");
    answer_open(&mut evm, vec![ChoiceOption::Confirm]);
    let vm_extra = evm.open_choice().expect("sadengjinhuan_extra_shan").clone();
    assert_eq!(vm_extra.spec.prompt, "sadengjinhuan_extra_shan");
    assert!(
        vm_extra.spec.options.contains(&ChoiceOption::Card(heart_tao))
            && vm_extra.spec.options.contains(&ChoiceOption::Card(heart_equip)),
        "sadengjinhuan_extra_shan must include VM view_as Shan hand & equip options: {:?}",
        vm_extra.spec.options
    );
    answer_open(&mut evm, vec![ChoiceOption::Card(heart_equip)]);
    drain_pass_all(&mut evm);
    assert_eq!(evm.state_core().player(1).hp, 4, "VM view_as equip Shan prevented damage");
    assert!(!evm.state_core().player(1).equips.contains(&heart_equip));
}

#[test]
fn u05_b7b_r3_jinjilengdong_equip1_equip2_draw_before_turnover_and_guiyanfadao_shahit() {
    let verified = prepare_locked_content_with_bridge(vec![
        freeze_formal_package("shahai-swd-upstream-noname"),
        freeze_formal_package("longxue-upstream-liuli"),
    ])
    .expect("prepare shahai + longxue");
    let mut e = start_locked_engine(9507, 2, LockedStartMode::Prototype, &verified)
        .expect("start dual-pack engine");
    for seat in 0..2 {
        e.debug_remove_general(seat);
        e.debug_set_face_down(seat, false);
    }
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // Part A: `hslingjian_jinjilengdong_equip1` -> target draws 2 BEFORE TurnOver!
    grant_skill(&mut e, &verified, 0, "hslingjian_jinjilengdong_equip1");
    grant_skill(&mut e, &verified, 1, "mbcmjiushi");
    clear_hand_to_discard(&mut e, &verified, 0);
    clear_hand_to_discard(&mut e, &verified, 1);
    let sha1 = find_kind_nth(&e, CardKind::Sha, 0);
    force_move_card_to_hand(&mut e, &verified, 0, sha1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha1, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_jileng_equip1_confirm");
    let mark1 = e.log().len();
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    let c1 = e.open_choice().expect("target mbcmjiushi on TurnOvered").clone();
    assert_eq!(c1.observer, 1);
    assert!(e.state_core().player(1).face_down);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        2,
        "equip1 MUST finish draw(2) BEFORE TurnOver fires"
    );
    assert!(
        e.log()[mark1..].iter().any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 1, cards } if cards.len() == 2)),
        "draw(2) must precede TurnOver"
    );
    answer_open(&mut e, Vec::new());
    drain_pass_all(&mut e);

    // Part B: `hslingjian_jinjilengdong_equip2` -> source draws 2 BEFORE TurnOver!
    remove_skill(&mut e, &verified, 0, "hslingjian_jinjilengdong_equip1");
    grant_skill(&mut e, &verified, 1, "hslingjian_jinjilengdong_equip2");
    grant_skill(&mut e, &verified, 0, "mbcmjiushi");
    e.debug_set_face_down(0, false);
    e.debug_set_face_down(1, false);
    step_full_turn_to(&mut e, 0);
    clear_hand_to_discard(&mut e, &verified, 0);
    clear_hand_to_discard(&mut e, &verified, 1);
    let sha2 = find_kind_nth(&e, CardKind::Sha, 1);
    force_move_card_to_hand(&mut e, &verified, 0, sha2);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha2, targets: vec![1] });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_jileng_equip2_confirm");
    let mark2 = e.log().len();
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    let c2 = e.open_choice().expect("source mbcmjiushi on TurnOvered").clone();
    assert_eq!(c2.observer, 0);
    assert!(e.state_core().player(0).face_down);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        2,
        "equip2 MUST finish draw(2) BEFORE TurnOver fires"
    );
    assert!(
        e.log()[mark2..].iter().any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 0, cards } if cards.len() == 2)),
        "draw(2) must precede TurnOver"
    );
    answer_open(&mut e, Vec::new());
    drain_pass_all(&mut e);
    assert_eq!(total_card_set(&e), init_set);

    // Part C: `guiyanfadao` hooks `shaHit` BEFORE `Hanbing` (`damageBegin2`), NOT `DamageBefore` after `Hanbing`!
    let (v2, mut e2) = start_shahai_swd_only(9508, 2);
    let init2 = total_card_set(&e2);
    advance_to_play(&mut e2, 0);
    grant_skill(&mut e2, &v2, 0, "guiyanfadao");
    clear_hand_to_discard(&mut e2, &v2, 0);
    clear_hand_to_discard(&mut e2, &v2, 1);
    let hanbing = find_kind(&e2, CardKind::Hanbing);
    let sha_a = find_kind_nth(&e2, CardKind::Sha, 0);
    let sha_b = find_kind_nth(&e2, CardKind::Sha, 1);
    let filler = find_kind(&e2, CardKind::Tao);
    force_move_card_to_hand(&mut e2, &v2, 0, hanbing);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: hanbing, targets: vec![] });
    drain_pass_all(&mut e2);
    force_move_card_to_hand(&mut e2, &v2, 0, sha_a);
    force_move_card_to_hand(&mut e2, &v2, 1, filler);
    {
        let mut snap = e2.save();
        snap.core.players[1].hp = 4;
        snap.core.players[1].hujia = 2;
        e2 = content_native_loader::restore_locked_engine(&snap, &v2).unwrap();
    }
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    assert_eq!(
        e2.open_choice().expect("shaHit prompt").spec.prompt,
        "u05_guiyanfadao_confirm",
        "guiyanfadao (shaHit) must open BEFORE hanbing_prevent_confirm (damageBegin2)"
    );
    answer_open(&mut e2, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e2);
    assert_eq!(e2.state_core().player(1).hp, 3);
    assert_eq!(e2.state_core().player(1).hujia, 2);
    assert_eq!(e2.state_core().player(1).hand.len(), 1, "Hanbing skipped when Guiyanfadao unhurt=true");

    // Decline guiyanfadao -> Hanbing CAN still fire!
    step_full_turn_to(&mut e2, 0);
    force_move_card_to_hand(&mut e2, &v2, 0, sha_b);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_b, targets: vec![1] });
    assert_eq!(e2.open_choice().unwrap().spec.prompt, "u05_guiyanfadao_confirm");
    answer_open(&mut e2, Vec::new());
    assert_eq!(
        e2.open_choice().expect("hanbing after declined guiyan").spec.prompt,
        "hanbing_prevent_confirm"
    );
    drain_pass_all(&mut e2);
    assert_eq!(total_card_set(&e2), init2);
}

#[test]
fn u05_b7b_r3_qiankundai_kongdongyin_and_mid_resolution_draw() {
    let (v, mut e) = start_shahai_swd_only(9509, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);

    let kyd = find_kind(&e, CardKind::Kongdongyin);
    let qkd = find_kind(&e, CardKind::Qiankundai);
    // Equip both Kongdongyin + Qiankundai onto seat 1 (`hp = 1`, `fankui` on seat 1
    // to inspect mid-resolution hand count right after Kongdongyin discards equips!).
    {
        let mut snap = e.save();
        snap.core.deck.retain(|x| *x != kyd && *x != qkd);
        snap.core.discard.retain(|x| *x != kyd && *x != qkd);
        snap.core.players[1].equips.push(kyd);
        snap.core.players[1].equips.push(qkd);
        snap.core.players[1].hp = 1;
        for sk in ["kongdongyin", "qiankundai", engine_native::content::standard::FAN_KUI] {
            let def = e.catalog().skill_by_name(sk).expect(sk);
            snap.core.skills.insert(
                (1, def),
                engine_native::state::SkillInstance {
                    owner: 1,
                    enabled: true,
                    once_used: false,
                    expires_end_of_turn: false,
                    rounds: 0,
                },
            );
        }
        e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    }

    let jiu = find_kind(&e, CardKind::Jiu);
    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &v, 0, jiu);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
    drain_pass_all(&mut e);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    // Now paused MID-RESOLUTION inside Sha/Damage (either kongdongyin_confirm / fankui / etc.)
    // Answer Kongdongyin if prompted, stopping at Fankui trigger_confirm (before outer Sha settles!).
    for _ in 0..8 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            // Mid-resolution check! Outer `CardUsed(Sha)` is STILL on stack (`outer_busy = true`),
            // yet Kongdongyin (1) + Qiankundai.onLose (1) = 2 cards MUST ALREADY be in hand!
            assert_eq!(
                e.state_core().player(1).hand.len(),
                2,
                "Qiankundai.onLose + Kongdongyin must draw 2 BEFORE outer Sha settles"
            );
            break;
        }
        let sel = if c.spec.options.contains(&ChoiceOption::Confirm) {
            vec![ChoiceOption::Confirm]
        } else {
            Vec::new()
        };
        answer_open(&mut e, sel);
    }
    drain_pass_all(&mut e);
    assert!(e.state_core().player(1).alive);
    assert_eq!(
        e.state_core().player(1).hand.len(),
        2,
        "Kongdongyin (1) + Qiankundai.onLose (1) = 2 cards total"
    );
    assert_eq!(total_card_set(&e), init_set);
}
