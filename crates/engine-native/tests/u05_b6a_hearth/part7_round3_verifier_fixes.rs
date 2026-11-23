//! Round 3 failing-first parity tests for all 8 defects reported by verifier-u05-b (#6).
//!
//! Uses real formal `shahai-hearth-upstream-noname` content, real `GameAction` events,
//! ZERO `debug_execute_effect`, public-info `BoundedPlanner`, card conservation,
//! and binary `NGSA 107` save/restore.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, RejectReason};

fn give_spade_card(e: &mut Engine, seat: u8) -> u32 {
    let cid = {
        let cat = e.catalog();
        let core = e.state_core();
        core.deck
            .iter()
            .chain(core.discard.iter())
            .chain(core.detached_cards.iter().map(|(c, _)| c))
            .copied()
            .find(|&id| cat.cards[id as usize].suit == Suit::Spade)
            .expect("spade card in deck/discard/detached")
    };
    e.state_core_mut().deck.retain(|&c| c != cid);
    e.state_core_mut().discard.retain(|&c| c != cid);
    e.state_core_mut().detached_cards.retain(|&(c, _)| c != cid);
    e.state_core_mut().player_mut(seat).hand.push(cid);
    cid
}

#[test]
fn r3_defect1_hsbaowu_huangjinyuanhou_only_loses_hand_and_draws_hand_plus_one() {
    let mut e = fresh_proto(7101, &["hs_malfurion", "hs_magni"]);
    advance_to_phase(&mut e, 0, Phase::Play);

    let weapon = give_free_card_kind(&mut e, 0, CardKind::Weapon);
    assert!(!matches!(
        play_card(&mut e, 0, weapon, vec![]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);

    let armor = give_free_card_kind(&mut e, 0, CardKind::Armor);
    assert!(!matches!(
        play_card(&mut e, 0, armor, vec![]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);

    let le = give_free_card_kind(&mut e, 0, CardKind::Le);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != le);
    e.state_core_mut().player_mut(0).judge.push(le);

    clear_hand(&mut e, 0);
    let extra1 = give_free_card_kind(&mut e, 0, CardKind::Shan);
    let extra2 = give_free_card_kind(&mut e, 0, CardKind::Tao);
    let monkey = give_card_named(&mut e, 0, "hsbaowu_huangjinyuanhou");

    let total_before = total_tracked_cards(&e);
    let out = play_card(&mut e, 0, monkey, vec![]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play monkey: {out:?}");
    drain_auto(&mut e);

    let p0 = e.state_core().player(0);
    assert_eq!(
        p0.equips,
        vec![weapon, armor],
        "hsbaowu_huangjinyuanhou must NOT lose equipped cards"
    );
    assert_eq!(
        p0.judge,
        vec![le],
        "hsbaowu_huangjinyuanhou must NOT lose judge-zone cards"
    );
    assert_eq!(
        p0.hand.len(),
        3,
        "lost 2 remaining hand cards -> must draw 1 + 2 = 3 cards"
    );
    assert!(!p0.hand.contains(&extra1) && !p0.hand.contains(&extra2));
    assert!(e.state_core().discard.contains(&extra1) && e.state_core().discard.contains(&extra2));
    assert_eq!(total_tracked_cards(&e), total_before);
}

#[test]
fn r3_defect2_zengli_allows_equal_hp_and_strict_min_hp_targets() {
    // Case A: All seats have equal HP = 3 -> target.isMin() is false for all others.
    let mut e = fresh_proto(7102, &["hs_blingtron", "hs_khadgar", "hs_tyrande"]);
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(e.state_core().player(0).hp, 3);
    assert_eq!(e.state_core().player(1).hp, 3);
    assert_eq!(e.state_core().player(2).hp, 3);

    let total_before = total_tracked_cards(&e);
    let out = activate(&mut e, 0, "zengli");
    assert!(
        !matches!(out, CommandOutcome::Rejected(_)),
        "zengli must be usable when all players have equal HP: {out:?}"
    );
    let ch = e.open_choice().cloned().expect("expected hs_zengli_target prompt");
    assert_eq!(ch.spec.prompt, "hs_zengli_target");
    assert!(ch.spec.options.contains(&ChoiceOption::Player(1)));
    assert!(ch.spec.options.contains(&ChoiceOption::Player(2)));
    let ai_sel = ai_choose_current(&e);
    assert!(!ai_sel.is_empty());
    save_restore_and_answer(&mut e, "hs_zengli_target", vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e);
    assert!(!e.state_core().player(0).equips.is_empty());
    assert!(!e.state_core().player(1).equips.is_empty());
    assert_eq!(total_tracked_cards(&e), total_before);

    // Case B: Only other alive player has strictly lowest HP (2 vs 3).
    let mut e2 = fresh_proto(7103, &["hs_blingtron", "hs_khadgar"]);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 1);
    let sha = give_free_card_kind(&mut e2, 0, CardKind::Sha);
    assert!(!matches!(
        play_card(&mut e2, 0, sha, vec![1]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hp, 3);
    assert_eq!(e2.state_core().player(1).hp, 2);

    let out2 = activate(&mut e2, 0, "zengli");
    assert!(
        !matches!(out2, CommandOutcome::Rejected(_)),
        "zengli must still allow targeting the strictly min-HP player: {out2:?}"
    );
    let ch2 = e2.open_choice().cloned().expect("expected hs_zengli_target");
    assert_eq!(ch2.spec.prompt, "hs_zengli_target");
    assert_eq!(ch2.spec.options, vec![ChoiceOption::Player(1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e2);
}

#[test]
fn r3_defect3_hszuzhou_nvwudexuetu_seals_non_locked_hearth_skills_including_optional_false() {
    let mut e = fresh_proto(7104, &["hs_malfurion", "hs_khadgar", "hs_bannabusi"]);
    advance_to_phase(&mut e, 0, Phase::Play);

    let xuetu1 = give_card_named(&mut e, 0, "hszuzhou_nvwudexuetu");
    let out = play_card(&mut e, 0, xuetu1, vec![1]);
    assert!(!matches!(out, CommandOutcome::Rejected(_)), "play xuetu1: {out:?}");
    drain_auto(&mut e);
    assert_eq!(flag(&e, 1, "hs_fengyin_xuetu"), 1);
    assert!(has_skill(&e, 1, "zhoujiang"));
    assert!(
        e.catalog().is_tieji_sealed(e.state_core(), 1),
        "hs_fengyin_xuetu must seal target's non-locked skills"
    );

    // Seat 1's turn before phaseAfter: non-locked active skill (`midian`) and non-locked
    // trigger skill with `optional: false` (`jinzhou`) must both be sealed.
    advance_to_phase(&mut e, 1, Phase::Play);
    let _spade = give_spade_card(&mut e, 1);
    let _extra = give_free_card_kind(&mut e, 1, CardKind::Shan);
    let midian_blocked = activate(&mut e, 1, "midian");
    assert!(
        matches!(midian_blocked, CommandOutcome::Rejected(_)),
        "midian must be sealed by hs_fengyin_xuetu, got {midian_blocked:?}"
    );
    let midian_def = e.catalog().skill_by_name("midian").expect("midian skill");
    assert!(
        !matches!(ai_plan_action(&e, 1), GameAction::ActivateSkill { skill } if skill == midian_def),
        "AI must not attempt sealed active skill"
    );

    // Advance from Play through Discard -> End -> Seat 2: `jinzhou` must NOT trigger while sealed!
    loop {
        if e.turn().current == 2 && e.open_choice().is_none() {
            break;
        }
        if let Some(ch) = e.open_choice().cloned() {
            assert_ne!(
                ch.spec.prompt, "hs_jinzhou_target",
                "non-locked `jinzhou` must be sealed by hs_fengyin_xuetu"
            );
            drain_auto(&mut e);
        } else {
            let cur = e.turn().current;
            let _ = submit(&mut e, cur, GameAction::NextPhase);
        }
    }
    assert_eq!(flag(&e, 1, "hs_fengyin_xuetu"), 0);
    assert!(!has_skill(&e, 1, "zhoujiang"));
    assert!(!e.catalog().is_tieji_sealed(e.state_core(), 1));

    // Verify locked (`forced: true`) skill `qingtian` on Seat 2 STILL triggers while `hs_fengyin_xuetu == 1`
    // (`qingtian` triggers when its owner takes damage and strictly has unique max HP).
    advance_to_phase(&mut e, 0, Phase::Play);
    let xuetu2 = give_card_named(&mut e, 0, "hszuzhou_nvwudexuetu");
    assert!(!matches!(
        play_card(&mut e, 0, xuetu2, vec![2]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);
    assert_eq!(flag(&e, 2, "hs_fengyin_xuetu"), 1);

    clear_hand(&mut e, 2);
    let hp_before = e.state_core().player(2).hp;
    let sha_p0 = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(!matches!(
        play_card(&mut e, 0, sha_p0, vec![2]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);
    assert_eq!(
        hp_before - e.state_core().player(2).hp,
        2,
        "locked skill qingtian must still deal +1 damage under hs_fengyin_xuetu"
    );

    // On Seat 1's next unsealed turn, `midian` and `jinzhou` work again.
    advance_to_phase(&mut e, 1, Phase::Play);
    let _spade2 = give_spade_card(&mut e, 1);
    let midian_ok = activate(&mut e, 1, "midian");
    assert!(
        !matches!(midian_ok, CommandOutcome::Rejected(_)),
        "midian must work after hs_fengyin_xuetu clears: {midian_ok:?}"
    );
    cancel_open_choice(&mut e);
    drain_auto(&mut e);

    let ch = advance_until_open_prompt(&mut e, "hs_jinzhou_target", 6)
        .expect("jinzhou must prompt once unsealed");
    assert!(!ch.spec.options.is_empty());
    cancel_open_choice(&mut e);
}

#[test]
fn r3_defect4_jinzhou_excludes_sealed_players() {
    // 3 players: Seat 0 (khadgar) seals Seat 1 with hszuzhou_nvwudexuetu, then enters Phase::End.
    let mut e = fresh_proto(7105, &["hs_khadgar", "hs_malfurion", "hs_magni"]);
    advance_to_phase(&mut e, 0, Phase::Play);
    let xuetu = give_card_named(&mut e, 0, "hszuzhou_nvwudexuetu");
    assert!(!matches!(
        play_card(&mut e, 0, xuetu, vec![1]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);
    clear_hand(&mut e, 0);
    let _spade = give_spade_card(&mut e, 0);

    let ch = advance_until_open_prompt(&mut e, "hs_jinzhou_target", 6)
        .expect("expected hs_jinzhou_target with Seat 2 still unsealed");
    assert!(
        !ch.spec.options.contains(&ChoiceOption::Player(1)),
        "jinzhou options must exclude already-sealed Seat 1: {:?}",
        ch.spec.options
    );
    assert_eq!(ch.spec.options, vec![ChoiceOption::Player(2)]);
    let ai_sel = ai_choose_current(&e);
    assert_eq!(ai_sel, vec![ChoiceOption::Player(2)]);
    save_restore_and_answer(&mut e, "hs_jinzhou_target", vec![ChoiceOption::Player(2)]);
    drain_auto(&mut e);
    assert_eq!(flag(&e, 2, "hs_jinzhou_sealed"), 1);

    // 2 players: the only other player (Seat 1) is already sealed by `hs_fengyin_xuetu` -> no prompt!
    let mut e2 = fresh_proto(7106, &["hs_khadgar", "hs_malfurion"]);
    advance_to_phase(&mut e2, 0, Phase::Play);
    let xuetu2 = give_card_named(&mut e2, 0, "hszuzhou_nvwudexuetu");
    assert!(!matches!(
        play_card(&mut e2, 0, xuetu2, vec![1]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e2);
    clear_hand(&mut e2, 0);
    let _spade2 = give_spade_card(&mut e2, 0);

    loop {
        if e2.turn().current == 1 && e2.open_choice().is_none() {
            break;
        }
        if let Some(ch) = e2.open_choice().cloned() {
            assert_ne!(
                ch.spec.prompt, "hs_jinzhou_target",
                "jinzhou must not prompt when all other players are sealed"
            );
            drain_auto(&mut e2);
        } else {
            let cur = e2.turn().current;
            let _ = submit(&mut e2, cur, GameAction::NextPhase);
        }
    }

    // Also verify `hs_jinzhou_sealed` itself excludes a target in `jinzhou`.
    let mut e3 = fresh_proto(7107, &["hs_khadgar", "hs_malfurion"]);
    advance_to_phase(&mut e3, 0, Phase::Play);
    clear_hand(&mut e3, 0);
    let _spade3 = give_spade_card(&mut e3, 0);
    set_flag(&mut e3, 1, "hs_jinzhou_sealed", 1);
    loop {
        if e3.turn().current == 1 && e3.open_choice().is_none() {
            break;
        }
        if let Some(ch) = e3.open_choice().cloned() {
            assert_ne!(
                ch.spec.prompt, "hs_jinzhou_target",
                "jinzhou must not prompt when the only other player has hs_jinzhou_sealed"
            );
            drain_auto(&mut e3);
        } else {
            let cur = e3.turn().current;
            let _ = submit(&mut e3, cur, GameAction::NextPhase);
        }
    }
}

#[test]
fn r3_defect5_zhaomingdan_allows_equip_or_judge_without_hand_and_rejects_empty_hej() {
    let mut e = fresh_proto(7108, &["hs_malfurion", "hs_magni"]);
    advance_to_phase(&mut e, 1, Phase::Play);
    let eq = give_free_card_kind(&mut e, 1, CardKind::Armor);
    assert!(!matches!(
        play_card(&mut e, 1, eq, vec![]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);

    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 1);
    assert!(e.state_core().player(1).hand.is_empty());
    assert_eq!(e.state_core().player(1).equips, vec![eq]);

    let zmd1 = give_card_named(&mut e, 0, "zhaomingdan");
    let total_before = total_tracked_cards(&e);
    let out1 = play_card(&mut e, 0, zmd1, vec![1]);
    assert!(
        !matches!(out1, CommandOutcome::Rejected(_)),
        "zhaomingdan must accept target with empty hand but 1 equip: {out1:?}"
    );
    save_restore_and_answer(&mut e, "hs_zhaomingdan_discard", vec![ChoiceOption::Card(eq)]);
    drain_auto(&mut e);
    assert!(e.state_core().player(1).equips.is_empty());
    assert_eq!(e.state_core().player(1).hand.len(), 1);
    assert_eq!(total_tracked_cards(&e), total_before);

    // Empty h/e/j -> must reject with IllegalTargets.
    clear_hand(&mut e, 1);
    let zmd2 = give_card_named(&mut e, 0, "zhaomingdan");
    let rej = play_card(&mut e, 0, zmd2, vec![1]);
    assert!(
        matches!(rej, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
        "expected IllegalTargets for empty hej, got {rej:?}"
    );

    // 0 hand, 0 equips, 1 judge -> must succeed.
    let le = give_free_card_kind(&mut e, 1, CardKind::Le);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != le);
    e.state_core_mut().player_mut(1).judge.push(le);
    let out2 = play_card(&mut e, 0, zmd2, vec![1]);
    assert!(
        !matches!(out2, CommandOutcome::Rejected(_)),
        "zhaomingdan must accept target with empty hand+equips but 1 judge: {out2:?}"
    );
    answer(&mut e, 0, vec![ChoiceOption::Card(le)]);
    drain_auto(&mut e);
    assert!(e.state_core().player(1).judge.is_empty());
    assert_eq!(e.state_core().player(1).hand.len(), 1);
    assert_eq!(total_tracked_cards(&e), total_before);
}

#[test]
fn r3_defect6a_shihuawuqi_once_per_turn_and_persists_until_turn_end() {
    let mut e = fresh_proto(7109, &["hs_malfurion", "hs_magni"]);
    advance_to_phase(&mut e, 0, Phase::Play);
    let sw1 = give_card_named(&mut e, 0, "shihuawuqi");
    // Ensure a distinct second shihuawuqi-like check: play sw1, check limit, move sw1 back after turn
    // Wait: check if catalog has a second shihuawuqi or test after recovering sw1 via discard->hand next turn;
    // Actually we can temporarily create/reuse sw1 ONLY after it's discarded? For limit within same turn,
    // test calling play_card again after `sw1` goes to discard if moved back or put a second `Tao`? No,
    // let's check using `chuansongmen` OR move `sw1` back to hand after resolving in same Play phase?
    // Wait: let's check `play_card` rejects a second shihuawuqi cleanly if a 2nd card exists or wait:
    let out1 = play_card(&mut e, 0, sw1, vec![]);
    assert!(!matches!(out1, CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert_eq!(flag(&e, 0, "hs_shihuawuqi_used_turn"), 1);

    // Also test a 2nd `shihuawuqi` cast attempt in the same turn is rejected with CardLimit.
    let second_sw = (0..e.catalog().cards.len() as u32)
        .find(|&id| id != sw1 && e.catalog().cards[id as usize].name.starts_with("shihuawuqi"))
        .unwrap_or(sw1);
    e.state_core_mut().discard.retain(|&c| c != second_sw);
    e.state_core_mut().deck.retain(|&c| c != second_sw);
    for p in 0..2 {
        e.state_core_mut().player_mut(p).hand.retain(|&c| c != second_sw);
    }
    e.state_core_mut().player_mut(0).hand.push(second_sw);
    let rej = play_card(&mut e, 0, second_sw, vec![]);
    assert!(
        matches!(rej, CommandOutcome::Rejected(RejectReason::CardLimit)),
        "2nd shihuawuqi in same turn must reject with CardLimit, got {rej:?}"
    );

    // Must persist after Phase::Play ends (through Phase::Discard) until turn end.
    advance_to_phase(&mut e, 0, Phase::Discard);
    assert_eq!(
        flag(&e, 0, "hs_shihuawuqi_used_turn"),
        1,
        "hs_shihuawuqi_used_turn must persist through Discard/End of the same turn"
    );

    // Next turn of Seat 0: cleared and usable again.
    advance_to_phase(&mut e, 0, Phase::Play);
    assert_eq!(flag(&e, 0, "hs_shihuawuqi_used_turn"), 0);
    let out2 = play_card(&mut e, 0, second_sw, vec![]);
    assert!(
        !matches!(out2, CommandOutcome::Rejected(_)),
        "shihuawuqi must be usable again on next turn: {out2:?}"
    );
}

#[test]
fn r3_defect6b_chuansongmen_usability_gate_and_empty_deck_reshuffle() {
    let mut e = fresh_proto(7110, &["hs_malfurion", "hs_magni"]);
    advance_to_phase(&mut e, 0, Phase::Play);

    // 1. After Sha limit used in Play phase, chuansongmen drawing a Sha must NOT prompt and must discard csm.
    clear_hand(&mut e, 1);
    let sha1 = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(!matches!(
        play_card(&mut e, 0, sha1, vec![1]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);

    let sha2 = give_free_card_kind(&mut e, 0, CardKind::Sha);
    put_card_on_deck_top(&mut e, sha2);
    let csm1 = give_card_named(&mut e, 0, "chuansongmen");
    let total_before = total_tracked_cards(&e);
    let out1 = play_card(&mut e, 0, csm1, vec![]);
    assert!(!matches!(out1, CommandOutcome::Rejected(_)));
    assert!(
        e.open_choice().is_none(),
        "chuansongmen must NOT prompt to use a 2nd Sha when Sha limit is already used"
    );
    assert!(e.state_core().player(0).hand.contains(&sha2));
    assert!(!e.state_core().player(0).hand.contains(&csm1));
    assert!(e.state_core().discard.contains(&csm1));
    assert_eq!(total_tracked_cards(&e), total_before);

    // 2. Empty deck + non-empty discard containing usable Sha (next turn) -> reshuffles & prompts!
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    let usable_sha = give_free_card_kind(&mut e, 0, CardKind::Sha);
    // Park other deck/discard cards in Seat 1's off_court to avoid triggering Wuxie and leave only `usable_sha` in discard.
    let old_discard: Vec<u32> = e.state_core_mut().discard.drain(..).collect();
    let old_deck: Vec<u32> = e.state_core_mut().deck.drain(..).collect();
    e.state_core_mut().player_mut(1).off_court.extend(old_discard);
    e.state_core_mut().player_mut(1).off_court.extend(old_deck);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != usable_sha);
    e.state_core_mut().discard.push(usable_sha);

    let csm2 = give_card_named(&mut e, 0, "chuansongmen");
    let total_before2 = total_tracked_cards(&e);
    let out2 = play_card(&mut e, 0, csm2, vec![]);
    assert!(!matches!(out2, CommandOutcome::Rejected(_)));
    let ch = e
        .open_choice()
        .cloned()
        .expect("chuansongmen with empty deck & reshuffled usable Sha must prompt");
    assert_eq!(ch.spec.prompt, "hs_chuansongmen_use");
    assert!(ch.spec.options.contains(&ChoiceOption::Card(usable_sha)));
    save_restore_and_answer(&mut e, "hs_chuansongmen_use", vec![ChoiceOption::Card(usable_sha)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&csm2),
        "chuansongmen returns to hand when the drawn usable card is used"
    );
    assert!(has_skill(&e, 0, "chuansongmen2"));
    assert_eq!(total_tracked_cards(&e), total_before2);
}

#[test]
fn r3_defect6c_xiyong_empty_deck_reshuffle_and_end_phase_sha_usable() {
    let mut e = fresh_proto(7111, &["hs_fachaotuteng", "hs_magni"]);
    advance_to_phase(&mut e, 0, Phase::Play);

    // Use Play-phase Sha limit first, and keep hand small so Discard phase does not drop extra cards.
    clear_hand(&mut e, 1);
    let sha1 = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(!matches!(
        play_card(&mut e, 0, sha1, vec![1]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);
    clear_hand(&mut e, 0);

    let end_sha = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let old_discard: Vec<u32> = e.state_core_mut().discard.drain(..).collect();
    let old_deck: Vec<u32> = e.state_core_mut().deck.drain(..).collect();
    e.state_core_mut().player_mut(1).hand.extend(old_discard);
    e.state_core_mut().player_mut(1).hand.extend(old_deck);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != end_sha);
    e.state_core_mut().discard.push(end_sha);

    let total_before = total_tracked_cards(&e);
    let ch = advance_until_open_prompt(&mut e, "hs_xiyong_use", 8)
        .expect("xiyong must reshuffle empty deck and allow Sha in Phase::End");
    assert!(ch.spec.options.contains(&ChoiceOption::Card(end_sha)));
    assert!(!ai_choose_current(&e).is_empty());
    save_restore_and_answer(&mut e, "hs_xiyong_use", vec![ChoiceOption::Card(end_sha)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut e);
    assert_eq!(total_tracked_cards(&e), total_before);
}

#[test]
fn r3_defect7_qingtian_requires_strictly_unique_max_hp() {
    let mut e = fresh_proto(7112, &["hs_bannabusi", "hs_malfurion"]);
    // Seat 0 (`qingtian`, max_hp 14) starts strictly max HP (> Seat 1 HP 4): each Sha takes +1 = 2 dmg.
    // Over 5 turns, Seat 0 drops 14 -> 12 -> 10 -> 8 -> 6 -> 4 (now tied with Seat 1 HP == 4).
    for step in 0..5 {
        advance_to_phase(&mut e, 1, Phase::Play);
        clear_hand(&mut e, 0);
        let before = e.state_core().player(0).hp;
        let sha = give_free_card_kind(&mut e, 1, CardKind::Sha);
        assert!(!matches!(
            play_card(&mut e, 1, sha, vec![0]),
            CommandOutcome::Rejected(_)
        ));
        drain_auto(&mut e);
        assert_eq!(
            before - e.state_core().player(0).hp,
            2,
            "step {step}: qingtian must deal +1 damage when Seat 0 strictly has unique max HP"
        );
        let _ = submit(&mut e, 1, GameAction::NextPhase);
        drain_auto(&mut e);
    }
    assert_eq!(e.state_core().player(0).hp, 4);
    assert_eq!(e.state_core().player(1).hp, 4);

    // Tied max HP (4 == 4): `qingtian` must NOT add +1 damage (takes 1 dmg: 4 -> 3).
    advance_to_phase(&mut e, 1, Phase::Play);
    clear_hand(&mut e, 0);
    let sha_tied = give_free_card_kind(&mut e, 1, CardKind::Sha);
    assert!(!matches!(
        play_card(&mut e, 1, sha_tied, vec![0]),
        CommandOutcome::Rejected(_)
    ));
    drain_auto(&mut e);
    assert_eq!(
        e.state_core().player(0).hp,
        3,
        "qingtian must NOT trigger when max HP is tied (4 vs 4)"
    );
}

#[test]
fn r3_defect8a_shandianjian_allows_self_target_and_empty_post_play_hand() {
    let mut e = fresh_proto(7113, &["hs_malfurion", "hs_magni"]);
    advance_to_phase(&mut e, 0, Phase::Play);

    // 1. Self-target with 1 extra card -> auto-shows it, prompts hs_shandianjian_discard;
    //    discarding it deals 1 Thunder damage to self.
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sdj = give_card_named(&mut e, 0, "shandianjian");
    let extra1 = give_free_card_kind(&mut e, 0, CardKind::Shan);
    let hp0 = e.state_core().player(0).hp;
    let out1 = play_card(&mut e, 0, sdj, vec![0]);
    assert!(
        !matches!(out1, CommandOutcome::Rejected(_)),
        "shandianjian self-target must be allowed: {out1:?}"
    );
    let ch = e.open_choice().cloned().expect("expected hs_shandianjian_discard");
    assert_eq!(ch.spec.prompt, "hs_shandianjian_discard");
    answer(&mut e, 0, vec![ChoiceOption::Card(extra1)]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp0 - 1);
    assert!(e.state_core().discard.contains(&extra1));

    // 2. Self-target declining hs_shandianjian_discard -> 0 damage, sets hs_temp_huogong2.
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sdj2 = give_card_named(&mut e, 0, "shandianjian");
    let extra2 = give_free_card_kind(&mut e, 0, CardKind::Shan);
    let hp1 = e.state_core().player(0).hp;
    let out_m = play_card(&mut e, 0, sdj2, vec![0]);
    assert!(!matches!(out_m, CommandOutcome::Rejected(_)));
    cancel_open_choice(&mut e);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp1);
    assert!(e.state_core().player(0).hand.contains(&extra2));

    // 3. Self-target when shandianjian is the ONLY card in hand -> finishes cleanly without damage.
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sdj_only = give_card_named(&mut e, 0, "shandianjian");
    let hp_before = e.state_core().player(0).hp;
    let out2 = play_card(&mut e, 0, sdj_only, vec![0]);
    assert!(!matches!(out2, CommandOutcome::Rejected(_)));
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hp, hp_before);
    assert!(e.state_core().discard.contains(&sdj_only));
}

#[test]
fn r3_defect8b_qianfu2_targetable_by_all_and_blocks_own_cards() {
    let mut e = fresh_proto(7114, &["hs_malfurion", "hs_selajin"]);
    // Trigger `qianfu` -> `qianfu2` on Seat 1 via real damage.
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 1);
    let jiu = give_free_card_kind(&mut e, 0, CardKind::Jiu);
    let sha1 = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(!matches!(play_card(&mut e, 0, jiu, vec![]), CommandOutcome::Rejected(_)));
    assert!(!matches!(play_card(&mut e, 0, sha1, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);

    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 1);
    let sha2 = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(!matches!(play_card(&mut e, 0, sha2, vec![1]), CommandOutcome::Rejected(_)));
    drain_auto(&mut e);

    assert!(has_skill(&e, 1, "qianfu2"));
    assert_eq!(flag(&e, 1, "hs_qianfu2_active"), 1);
    assert_eq!(e.state_core().player(1).hp, 1);

    // Give Seat 1 Shan, Sha, and Tao to verify cardRespondable & cardSavable are all blocked.
    let _p1_shan = give_free_card_kind(&mut e, 1, CardKind::Shan);
    let _p1_sha = give_free_card_kind(&mut e, 1, CardKind::Sha);
    let _p1_tao = give_free_card_kind(&mut e, 1, CardKind::Tao);

    // 1. Targetable by normal Sha and Duel; Seat 1 gets NO Shan/Sha prompts, takes 0 non-fire damage.
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    let norm_sha = give_free_card_kind(&mut e, 0, CardKind::Sha);
    let out_norm = play_card(&mut e, 0, norm_sha, vec![1]);
    assert!(
        !matches!(out_norm, CommandOutcome::Rejected(_)),
        "qianfu2 player must remain targetable by normal Sha: {out_norm:?}"
    );
    assert!(
        e.open_choice().is_none(),
        "qianfu2 blocks Shan response; got {:?}",
        e.open_choice().map(|c| &c.spec.prompt)
    );
    assert_eq!(e.state_core().player(1).hp, 1);

    let duel = give_free_card_kind(&mut e, 0, CardKind::Duel);
    let out_duel = play_card(&mut e, 0, duel, vec![1]);
    assert!(
        !matches!(out_duel, CommandOutcome::Rejected(_)),
        "qianfu2 player must remain targetable by Duel: {out_duel:?}"
    );
    assert!(
        e.open_choice().is_none(),
        "qianfu2 blocks Sha response in Duel; got {:?}",
        e.open_choice().map(|c| &c.spec.prompt)
    );
    assert_eq!(e.state_core().player(1).hp, 1);

    // 2. Targetable by FireSha; Seat 1 cannot respond with Shan, cannot save with Tao -> dies!
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    drain_auto(&mut e);
    advance_to_phase(&mut e, 0, Phase::Play);
    let fire_sha = give_free_card_kind(&mut e, 0, CardKind::FireSha);
    let out_fire = play_card(&mut e, 0, fire_sha, vec![1]);
    assert!(
        !matches!(out_fire, CommandOutcome::Rejected(_)),
        "qianfu2 player must be targetable by FireSha: {out_fire:?}"
    );
    assert!(
        e.open_choice().is_none(),
        "qianfu2 blocks Shan/Dying Tao; got {:?}",
        e.open_choice().map(|c| &c.spec.prompt)
    );
    assert!(!e.state_core().player(1).alive);
}
