use super::common::*;
use content_native_loader::{
    prepare_locked_content_with_bridge, start_locked_engine, LockedStartMode,
};
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent};

fn find_rank_card(e: &engine_native::Engine, min_rank: u8, max_rank: u8, exclude: &[u32]) -> u32 {
    (0..e.catalog().cards.len() as u32)
        .find(|&id| {
            !exclude.contains(&id)
                && matches!(
                    e.catalog().card_kind(id),
                    CardKind::Sha | CardKind::Shan | CardKind::Tao
                )
                && e.catalog().cards[id as usize].rank >= min_rank
                && e.catalog().cards[id as usize].rank <= max_rank
        })
        .expect("matching rank basic card")
}

#[test]
fn u05_b7b_r2_chilongya2_chilongfengxue_and_tianxianjiu_timing() {
    let (v, mut e) = start_shahai_swd_only(9401, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let txj_def = e.catalog().skill_by_name("tianxianjiu").expect("tianxianjiu");

    // 1. `tianxianjiu` stays active through Play -> Discard -> End (`phaseAfter` only),
    //    and expires at `TurnPhaseAfter` if no Sha is played.
    let txj1 = find_kind_nth(&e, CardKind::Tianxianjiu, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    force_move_card_to_hand(&mut e, &v, 0, txj1);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: txj1,
            targets: vec![],
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    drain_pass_all(&mut e);
    assert!(e.state_core().has_skill(0, txj_def));

    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::Discard);
    assert!(e.state_core().has_skill(0, txj_def), "must survive Play -> Discard");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    assert_eq!(e.turn().phase, Phase::End);
    assert!(e.state_core().has_skill(0, txj_def), "must survive Discard -> End");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_pass_all(&mut e);
    assert_eq!(e.turn().current, 1);
    assert!(!e.state_core().has_skill(0, txj_def), "must clear at TurnPhaseAfter");
    assert_eq!(total_card_set(&e), init_set);

    // 2a. `tianxianjiu` + hitting `Sha` (without `chilongya2`):
    //     `should_defer_sha_end` keeps `tianxianjiu` alive through `Damage` (+1 dmg -> 2 total),
    //     and `shaAfter` still expires `tianxianjiu` unconditionally afterward!
    let (v, mut e) = start_shahai_swd_only(9411, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    let txj_def = e.catalog().skill_by_name("tianxianjiu").expect("tianxianjiu");
    let cfx_def = e.catalog().skill_by_name("chilongfengxue").expect("chilongfengxue");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let txj2 = find_kind_nth(&e, CardKind::Tianxianjiu, 1);
    let sha1 = find_kind_nth(&e, CardKind::Sha, 0);
    force_move_card_to_hand(&mut e, &v, 0, txj2);
    force_move_card_to_hand(&mut e, &v, 0, sha1);

    let out2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: txj2,
            targets: vec![],
        },
    );
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "txj2 submit failed: {out2:?} at turn={:?}", e.turn());
    drain_pass_all(&mut e);
    assert!(e.state_core().has_skill(0, txj_def));

    let hp1_before = e.state_core().player(1).hp;
    let hand0_before = e.state_core().player(0).hand.len();
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
        hp1_before - 1,
        "tianxianjiu must deal base 1 Sha damage without Jiu +1"
    );
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand0_before - 1 + 2,
        "tianxianjiu must remain active through deferred Damage to draw 2 before shaAfter clears it"
    );
    assert!(
        !e.state_core().has_skill(0, txj_def),
        "tianxianjiu must expire at shaAfter even when Damage was dealt"
    );
    assert_eq!(total_card_set(&e), init_set);

    // 2b. `chilongya2` + `chilongfengxue` defer `sha_end_step` until AFTER `Damage`:
    let (v, mut e) = start_shahai_swd_only(9412, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    let sha_c = find_kind_nth(&e, CardKind::Sha, 0);
    force_move_card_to_hand(&mut e, &v, 0, sha_c);
    grant_skill(&mut e, &v, 0, "chilongya2");
    grant_skill(&mut e, &v, 1, "chilongfengxue");

    let hp1_before = e.state_core().player(1).hp;
    let hand0_after_play_before = 0usize;
    let hand1_before = e.state_core().player(1).hand.len();
    let log_mark = e.log().len();
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha_c,
            targets: vec![1],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        e.state_core().player(1).hp <= hp1_before - 1,
        "Sha with chilongya2 dealt damage"
    );
    assert!(
        !e.state_core().has_skill(0, cfx_def) && !e.state_core().has_skill(1, cfx_def),
        "chilongfengxue must settle and clear at sha_end_step after Damage"
    );
    assert_eq!(e.state_core().player(1).hand.len(), hand1_before + 1);
    assert!(
        e.state_core().player(0).hand.len() >= hand0_after_play_before,
        "owner hand conserved/drawn if chilongya2 proc'd"
    );
    let log_slice = &e.log()[log_mark..];
    let dmg_pos = log_slice
        .iter()
        .position(|ev| matches!(ev, LogEvent::DamageDealt { target: 1, .. }))
        .expect("damage dealt");
    let draw_pos = log_slice
        .iter()
        .position(|ev| matches!(ev, LogEvent::CardsDrawn { player: 1, cards } if cards.len() == 1))
        .expect("chilongfengxue draw");
    assert!(dmg_pos < draw_pos, "Damage must settle before sha_end_step draws");
    remove_skill(&mut e, &v, 0, "chilongya2");
    assert_eq!(total_card_set(&e), init_set);

    // 3. `Sha` with `Damage` emitted + deferred, then prevented inside `Damage` by `step_fengxueren` + `tianxianjiu`:
    //    `tianxianjiu` STILL expires unconditionally at `shaAfter`!
    let (v, mut e) = start_shahai_swd_only(9421, 2);
    let init_set = total_card_set(&e);
    let txj_def = e.catalog().skill_by_name("tianxianjiu").expect("tianxianjiu");
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    grant_skill(&mut e, &v, 0, "fengxueren");
    let txj_again = find_kind_nth(&e, CardKind::Tianxianjiu, 0);
    let second_sha = find_kind_nth(&e, CardKind::Sha, 1);
    force_move_card_to_hand(&mut e, &v, 0, txj_again);
    force_move_card_to_hand(&mut e, &v, 0, second_sha);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: txj_again,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(e.state_core().has_skill(0, txj_def));

    let hp1_prev = e.state_core().player(1).hp;
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: second_sha,
            targets: vec![1],
        },
    );
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_fengxueren_confirm");
    answer_open(&mut e, vec![ChoiceOption::Confirm]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_prev, "Fengxueren prevented damage");
    assert!(
        !e.state_core().has_skill(0, txj_def),
        "tianxianjiu must clear unconditionally at shaAfter even when Damage is prevented"
    );
    remove_skill(&mut e, &v, 0, "fengxueren");
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r2_lingjianduanzao_jiqi_yuchan_and_card_mods_cleanup() {
    let (v, mut e) = start_shahai_swd_only(9402, 2);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let duanzao_def = grant_skill(&mut e, &v, 0, "_lingjianduanzao");
    let hl_equip2_def = e
        .catalog()
        .skill_by_name("huanglinzhicong_equip2")
        .expect("huanglinzhicong_equip2");
    let cangchi_equip1_def = e
        .catalog()
        .skill_by_name("cangchizhibi_equip1")
        .expect("cangchizhibi_equip1");
    let yuchan_equip_def = grant_skill(&mut e, &v, 0, "g_yuchan_equip");

    let armor1 = find_kind(&e, CardKind::Baihupifeng);
    let armor2 = find_kind(&e, CardKind::Guangshatianyi);
    let huanglin = find_kind(&e, CardKind::Huanglinzhicong);
    let cangchi = find_kind(&e, CardKind::Cangchizhibi);
    let weapon1 = find_kind(&e, CardKind::Chilongya);
    let weapon2 = find_kind(&e, CardKind::Fengxueren);
    let yuchan_mat = find_kind(&e, CardKind::Yuchanqian);
    let spare_basic1 = find_kind_nth(&e, CardKind::Sha, 0);
    let spare_basic2 = find_kind_nth(&e, CardKind::Sha, 1);

    // 1. Forge `Huanglinzhicong` (jiqi) onto Armor (`armor1`, slot 2) -> grants `huanglinzhicong_equip2`
    clear_hand_to_discard(&mut e, &v, 0);
    force_move_card_to_hand(&mut e, &v, 0, armor1);
    force_move_card_to_hand(&mut e, &v, 0, huanglin);
    force_move_card_to_hand(&mut e, &v, 0, spare_basic1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: armor1,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);

    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_duanzao_equip");
    answer_open(&mut e, vec![ChoiceOption::Card(armor1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_duanzao_mat");
    answer_open(&mut e, vec![ChoiceOption::Card(huanglin)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_duanzao_target");
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);
    drain_pass_all(&mut e);

    assert_eq!(e.state_core().player(0).card_mods.get(&armor1), Some(&huanglin));
    assert!(
        e.state_core().has_skill(0, hl_equip2_def),
        "forging Huanglinzhicong onto Armor must grant huanglinzhicong_equip2"
    );

    // Also test Weapon forge with `Cangchizhibi` -> grants `cangchizhibi_equip1`
    force_move_card_to_hand(&mut e, &v, 0, weapon1);
    force_move_card_to_hand(&mut e, &v, 0, cangchi);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: weapon1,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    answer_open(&mut e, vec![ChoiceOption::Card(weapon1)]);
    answer_open(&mut e, vec![ChoiceOption::Card(cangchi)]);
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);
    drain_pass_all(&mut e);
    assert!(e.state_core().has_skill(0, cangchi_equip1_def));

    // Advance to TurnPhaseEnd: `huanglinzhicong_equip2` must fire and award +1 hujia!
    let hujia_before = e.state_core().player(0).hujia;
    for _ in 0..12 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "u05_huanglinzhicong_forging_discard" {
                answer_open(&mut e, vec![ChoiceOption::Card(spare_basic1)]);
                break;
            }
            let sel = if c.spec.allow_cancel {
                Vec::new()
            } else {
                c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
            };
            answer_open(&mut e, sel);
            continue;
        }
        let cur = e.turn().current;
        if cur != 0 {
            break;
        }
        let _ = submit(&mut e, 0, GameAction::NextPhase);
    }
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).hujia, hujia_before + 1);

    // 2. Replace `armor1` with `armor2` -> `armor1` leaves equips:
    //    `card_mods` & `declared_slots` cleaned and `huanglinzhicong_equip2` revoked!
    step_full_turn_to(&mut e, 0);
    force_move_card_to_hand(&mut e, &v, 0, armor2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: armor2,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(
        !e.state_core().player(0).card_mods.contains_key(&armor1),
        "card_mods must be cleaned when forged armor leaves equips"
    );
    assert!(
        !e.state_core().declared_slots.contains_key(&armor1),
        "declared_slots must be cleaned when forged armor leaves equips"
    );
    assert!(
        !e.state_core().has_skill(0, hl_equip2_def),
        "forged huanglinzhicong_equip2 must be revoked when forged armor leaves equips"
    );

    // 3. Replace `weapon1` with `weapon2` -> `cangchizhibi_equip1` revoked & `card_mods` cleaned.
    //    Then forge `Yuchanqian` onto `weapon2` -> `has_forged_yuchan_skill` detects `card_mods`!
    force_move_card_to_hand(&mut e, &v, 0, weapon2);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: weapon2,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(0).card_mods.contains_key(&weapon1));
    assert!(!e.state_core().has_skill(0, cangchi_equip1_def));

    clear_hand_to_discard(&mut e, &v, 0);
    force_move_card_to_hand(&mut e, &v, 0, yuchan_mat);
    force_move_card_to_hand(&mut e, &v, 0, spare_basic2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: duanzao_def });
    answer_open(&mut e, vec![ChoiceOption::Card(weapon2)]);
    answer_open(&mut e, vec![ChoiceOption::Card(yuchan_mat)]);
    answer_open(&mut e, vec![ChoiceOption::Player(0)]);
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(0).card_mods.get(&weapon2), Some(&yuchan_mat));

    let hand_before = e.state_core().player(0).hand.len();
    let out = submit(
        &mut e,
        0,
        GameAction::ActivateSkill {
            skill: yuchan_equip_def,
        },
    );
    assert!(!matches!(out, CommandOutcome::Rejected(_)));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_yuchan_equip_discard");
    answer_open(&mut e, vec![ChoiceOption::Card(spare_basic2)]);
    drain_pass_all(&mut e);
    assert_eq!(
        e.state_core().player(0).hand.len(),
        hand_before,
        "g_yuchan_equip discarded 1 basic card and drew 1"
    );

    // Replace `weapon2` with `weapon1` -> `card_mods` on `weapon2` cleared, `g_yuchan_equip` no longer triggers
    step_full_turn_to(&mut e, 0);
    force_move_card_to_hand(&mut e, &v, 0, weapon1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: weapon1,
            targets: vec![],
        },
    );
    drain_pass_all(&mut e);
    assert!(!e.state_core().player(0).card_mods.contains_key(&weapon2));
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r2_kongxin_does_not_consume_owner_play_phase_sha_quota() {
    let (v, mut e) = start_shahai_swd_only(9403, 3);
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    let kongxin_def = grant_skill(&mut e, &v, 0, "kongxin");
    clear_hand_to_discard(&mut e, &v, 0);
    clear_hand_to_discard(&mut e, &v, 1);
    clear_hand_to_discard(&mut e, &v, 2);

    let high = find_rank_card(&e, 11, 13, &[]);
    let sha = find_kind_nth(&e, CardKind::Sha, 0);
    let low = find_rank_card(&e, 1, 4, &[high, sha]);
    force_move_card_to_hand(&mut e, &v, 0, high);
    force_move_card_to_hand(&mut e, &v, 0, sha);
    force_move_card_to_hand(&mut e, &v, 1, low);

    let hp2_before = e.state_core().player(2).hp;
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: kongxin_def });
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_target");
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_pindian_own");
    answer_open(&mut e, vec![ChoiceOption::Card(high)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_pindian_target");
    answer_open(&mut e, vec![ChoiceOption::Card(low)]);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_kongxin_sha_target");
    answer_open(&mut e, vec![ChoiceOption::Player(2)]);
    drain_pass_all(&mut e);

    assert_eq!(e.state_core().player(2).hp, hp2_before - 1);
    assert!(
        !e.state_core().turn.sha_used_play_phase,
        "kongxin forcing player 1 to use Sha must NOT consume owner's Play-phase Sha quota"
    );

    let hp1_before = e.state_core().player(1).hp;
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "owner must still be allowed to play Play-phase Sha after kongxin: {out:?}"
    );
    drain_pass_all(&mut e);
    assert_eq!(e.state_core().player(1).hp, hp1_before - 1);
    assert!(e.state_core().turn.sha_used_play_phase);
    assert_eq!(total_card_set(&e), init_set);
}

#[test]
fn u05_b7b_r2_lifo_turnover_before_draw_jinjilengdong_equip5_and_fengxueren() {
    let verified = prepare_locked_content_with_bridge(vec![
        freeze_formal_package("shahai-swd-upstream-noname"),
        freeze_formal_package("longxue-upstream-liuli"),
    ])
    .expect("prepare shahai + longxue");
    let mut e = start_locked_engine(9404, 2, LockedStartMode::Prototype, &verified)
        .expect("start dual-pack engine");
    for seat in 0..2 {
        e.debug_remove_general(seat);
        e.debug_set_face_down(seat, false);
    }
    let init_set = total_card_set(&e);
    advance_to_play(&mut e, 0);

    // Part A: `hslingjian_jinjilengdong_equip5`:
    //   Exact LIFO settlement order must be: owner TurnOver -> target TurnOver -> owner draw 2 -> target draw 2.
    grant_skill(&mut e, &verified, 0, "hslingjian_jinjilengdong_equip5");
    grant_skill(&mut e, &verified, 0, "mbcmjiushi");
    grant_skill(&mut e, &verified, 1, "mbcmjiushi");
    clear_hand_to_discard(&mut e, &verified, 0);
    clear_hand_to_discard(&mut e, &verified, 1);

    // Advance from Play through Discard -> End -> TurnEndWindow to reach `u05_jileng_equip5_target`
    for _ in 0..16 {
        if let Some(c) = e.open_choice() {
            if c.spec.prompt == "u05_jileng_equip5_target" {
                break;
            }
            drain_pass_all(&mut e);
            continue;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert_eq!(
        e.open_choice().expect("jileng_equip5 target ask").spec.prompt,
        "u05_jileng_equip5_target"
    );
    let mark_a = e.log().len();
    answer_open(&mut e, vec![ChoiceOption::Player(1)]);

    // 1st TurnOvered probe: owner (seat 0) must ALREADY be face_down, target (seat 1) NOT yet, 0 draws so far!
    let c1 = e.open_choice().expect("owner mbcmjiushi on TurnOvered").clone();
    assert_eq!(c1.observer, 0, "owner must flip BEFORE target");
    assert!(e.state_core().player(0).face_down);
    assert!(!e.state_core().player(1).face_down);
    assert!(
        !e.log()[mark_a..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsDrawn { .. })),
        "no draw may happen before owner TurnOver completes"
    );
    answer_open(&mut e, Vec::new());

    // 2nd TurnOvered probe: target (seat 1) must NOW be face_down too, STILL 0 draws!
    let c2 = e.open_choice().expect("target mbcmjiushi on TurnOvered").clone();
    assert_eq!(c2.observer, 1, "target must flip after owner and BEFORE any draw");
    assert!(e.state_core().player(0).face_down);
    assert!(e.state_core().player(1).face_down);
    assert!(
        !e.log()[mark_a..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsDrawn { .. })),
        "no draw may happen before BOTH turnovers complete"
    );
    let ans2 = answer_open(&mut e, Vec::new());
    assert!(
        !matches!(ans2, CommandOutcome::Rejected(_)),
        "ans2 rejected: {ans2:?}, open={:?}",
        e.open_choice()
    );
    drain_pass_all(&mut e);

    let slice_a = &e.log()[mark_a..];
    let d0 = slice_a
        .iter()
        .position(|ev| matches!(ev, LogEvent::CardsDrawn { player: 0, cards } if cards.len() == 2))
        .unwrap_or_else(|| panic!("owner draw 2 missing in {slice_a:?}"));
    let d1 = slice_a
        .iter()
        .position(|ev| matches!(ev, LogEvent::CardsDrawn { player: 1, cards } if cards.len() == 2))
        .unwrap_or_else(|| panic!("target draw 2 missing in {slice_a:?}"));
    assert!(d0 < d1, "owner draw 2 must pop before target draw 2");
    remove_skill(&mut e, &verified, 0, "hslingjian_jinjilengdong_equip5");
    assert_eq!(total_card_set(&e), init_set);

    // Part B: `fengxueren`:
    //   Exact LIFO order must be: prevent damage + target TurnOver -> THEN target draw 1.
    let mut e = start_locked_engine(9405, 2, LockedStartMode::Prototype, &verified)
        .expect("start dual-pack engine for fengxueren");
    for seat in 0..2 {
        e.debug_remove_general(seat);
        e.debug_set_face_down(seat, false);
    }
    let init_set_b = total_card_set(&e);
    grant_skill(&mut e, &verified, 0, "fengxueren");
    grant_skill(&mut e, &verified, 1, "mbcmjiushi");
    advance_to_play(&mut e, 0);
    clear_hand_to_discard(&mut e, &verified, 0);
    clear_hand_to_discard(&mut e, &verified, 1);

    let sha = find_kind(&e, CardKind::Sha);
    force_move_card_to_hand(&mut e, &verified, 0, sha);
    let hp1_before = e.state_core().player(1).hp;
    let sha_out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(!matches!(sha_out, CommandOutcome::Rejected(_)));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "u05_fengxueren_confirm");
    let mark_b = e.log().len();
    answer_open(&mut e, vec![ChoiceOption::Confirm]);

    // TurnOvered probe for target (seat 1): must already be face_down, 0 draws so far!
    let cb = e.open_choice().expect("target mbcmjiushi on fengxueren TurnOvered").clone();
    assert_eq!(cb.observer, 1);
    assert!(e.state_core().player(1).face_down);
    assert!(e.state_core().player(1).hand.is_empty(), "fengxueren must turn over before draw 1");
    assert!(
        !e.log()[mark_b..]
            .iter()
            .any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 1, .. }))
    );
    let ans2 = answer_open(&mut e, Vec::new());
    assert!(
        !matches!(ans2, CommandOutcome::Rejected(_)),
        "ans2 rejected: {ans2:?}, open={:?}",
        e.open_choice()
    );
    drain_pass_all(&mut e);

    assert_eq!(e.state_core().player(1).hp, hp1_before);
    assert_eq!(e.state_core().player(1).hand.len(), 1);
    assert!(e.log()[mark_b..]
        .iter()
        .any(|ev| matches!(ev, LogEvent::DamagePrevented { target: 1, .. })));
    assert!(e.log()[mark_b..]
        .iter()
        .any(|ev| matches!(ev, LogEvent::CardsDrawn { player: 1, cards } if cards.len() == 1)));
    assert_eq!(total_card_set(&e), init_set_b);
}
