use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::event::DamageNature;
use engine_native::rvm_bridge::modifiers::{
    eval_card_target_canceled, is_card_ignored_in_discard_phase,
};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension.js:4021-4180` (`cxy_BaoYing`, `cxyYangWu`, `cxyJingQi`, `cxyRuiQi`, `cxyJiaoXia`, `cxyTunJun`, `cxyFengYing`).
#[test]
fn l11_skills_c_baoying_through_fengying() {
    // 1. cxy_BaoYing (limited: dying friend recovers to 1 HP once)
    let mut e_by = start_versus(0x1601, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_by.debug_grant(0, "cxy_BaoYing");
    e_by.debug_grant(2, "咆哮");
    e_by.debug_set_hp(1, 1);
    e_by.debug_set_turn(2, Phase::Play, 1);
    let sha = place_kind_suit(&mut e_by, 2, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_by, 2, sha, vec![1]);
    let c_by = expect_pending(&e_by, "cxy_BaoYing:confirm");
    assert_eq!(c_by.observer, 0);
    let _ = answer(&mut e_by, &c_by, vec![ChoiceOption::Confirm]);
    drain(&mut e_by);
    assert_eq!(e_by.debug_hp(1), 1, "cxy_BaoYing saved friend seat 1 to 1 HP");
    // Second dying does NOT trigger cxy_BaoYing (limited once per game)
    let sha2 = place_kind_suit(&mut e_by, 2, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e_by, 2, sha2, vec![1]);
    drain(&mut e_by);
    assert!(!e_by.debug_alive(1), "cxy_BaoYing is limited once per game");

    // 2. cxyYangWu (Prepare: 1 dmg to all others, then lose 1 HP) & cxyTunJun (Round 1 TurnBegin: -1 max_hp, draw remaining max_hp)
    let mut e_yt = start_taixu(0x1602, 3);
    e_yt.debug_grant(0, "cxyYangWu");
    e_yt.debug_grant(0, "cxyTunJun");
    e_yt.debug_set_turn(2, Phase::End, 1);
    let _ = next_phase(&mut e_yt); // Seat 2 End -> Seat 0 TurnBegin -> cxyTunJun (-1 max_hp to 5, draw 5) -> Prepare -> cxyYangWu (dmg 1,2, lose 1 HP)
    drain(&mut e_yt);
    assert_eq!(e_yt.state_core().player(0).max_hp, 5);
    assert_eq!(e_yt.debug_hand(0).len(), 5, "cxyTunJun drew remaining max_hp = 5");
    assert_eq!(e_yt.debug_hp(1), 5);
    assert_eq!(e_yt.debug_hp(2), 5);
    assert_eq!(e_yt.debug_hp(0), 4, "max_hp clamped hp from 6 to 5 then YangWu lost 1 -> 4");

    // 3. cxyJingQi (-1 distance for allies), cxyRuiQi (+1 draw for allies), cxyJiaoXia (black cards ignored in discard), cxyFengYing (unique min enemy blocked)
    let mut e_au = start_versus(0x1603, &[&[0, 1], &[2, 3]], &[Faction::Wei, Faction::Shu]);
    e_au.debug_grant(0, "cxyJingQi");
    e_au.debug_grant(0, "cxyRuiQi");
    e_au.debug_grant(0, "cxyJiaoXia");
    e_au.debug_grant(0, "cxyFengYing");

    assert_eq!(e_au.state_core().computed_distance(e_au.catalog(), 1, 3), 1, "cxyJingQi -1 from ally 1");
    let black_c = place_kind_suit(&mut e_au, 1, CardKind::Sha, Suit::Spade);
    let red_c = place_kind_suit(&mut e_au, 1, CardKind::Sha, Suit::Heart);
    assert!(is_card_ignored_in_discard_phase(e_au.state_core(), e_au.catalog(), 1, black_c));
    assert!(!is_card_ignored_in_discard_phase(e_au.state_core(), e_au.catalog(), 1, red_c));

    // Ally seat 1 draws 2 + 1 (cxyRuiQi) = 3 in Draw phase
    e_au.debug_set_turn(1, Phase::Judge, 1);
    let _ = next_phase(&mut e_au);
    drain(&mut e_au);
    assert_eq!(e_au.debug_hand(1).len(), 5, "2 initial + 2 base + 1 cxyRuiQi = 5");

    // cxyFengYing: ally target 1 with strictly unique lowest HP (hp=1) cannot be targeted by enemy 2! Tie (hp=1 on both 1 & 2) or unique-min enemy targeting high-HP ally is NOT blocked!
    e_au.debug_set_hp(1, 1);
    let sha_2 = place_kind_suit(&mut e_au, 2, CardKind::Sha, Suit::Spade);
    assert!(eval_card_target_canceled(
        e_au.state_core(),
        e_au.catalog(),
        2,
        1,
        sha_2,
        CardKind::Sha,
        DamageNature::Normal,
    ));
    e_au.debug_set_hp(3, 1); // tie at hp=1 -> target 1 is no longer strictly unique minimum
    assert!(!eval_card_target_canceled(
        e_au.state_core(),
        e_au.catalog(),
        2,
        1,
        sha_2,
        CardKind::Sha,
        DamageNature::Normal,
    ));
}

/// Upstream: `extension.js:4181-4385` (`cxyFanGong`, `cxyJieLve`, `cxyMoQu`, `cxyYaoWu`, `cxyYingHun`, `cxyPoLu`).
#[test]
fn l11_skills_c_fangong_through_polu() {
    // 1. cxyFanGong + cxyJieLve + cxyMoQu: enemy attacks owner -> FanGong counter-Sha -> JieLve loots hand+equip+judge & links -> MoQu triggers on TurnEnd & friend damage!
    let mut e = start_versus(0x1604, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e.debug_grant(1, "cxyFanGong");
    e.debug_grant(1, "cxyJieLve");
    e.debug_grant(0, "cxyMoQu");
    e.debug_set_hp(0, 6);
    let _ = equip_kind(&mut e, 2, CardKind::Zhuge);
    let enemy_h = place_kind_suit(&mut e, 2, CardKind::Tao, Suit::Heart);
    let fg_sha = place_kind_suit(&mut e, 1, CardKind::Sha, Suit::Spade);
    let mq_card = place_kind(&mut e, 0, CardKind::Shan);
    let e_sha = place_kind_suit(&mut e, 2, CardKind::Sha, Suit::Diamond);
    e.debug_set_turn(2, Phase::Play, 1);
    let _ = play_card(&mut e, 2, e_sha, vec![1]);
    drain(&mut e);
    let mut e_fg = start_versus(0x1605, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_fg.debug_grant(1, "cxyFanGong");
    e_fg.debug_grant(1, "cxyJieLve");
    let eq2 = equip_kind(&mut e_fg, 2, CardKind::Zhuge);
    let eh2 = place_kind_suit(&mut e_fg, 2, CardKind::Tao, Suit::Heart);
    let fg2 = place_kind_suit(&mut e_fg, 1, CardKind::Sha, Suit::Spade);
    let es2 = place_kind_suit(&mut e_fg, 2, CardKind::Sha, Suit::Diamond);
    e_fg.debug_set_turn(2, Phase::Play, 1);
    let _ = play_card(&mut e_fg, 2, es2, vec![1]);
    let c_fg = answer_pass_until(&mut e_fg, "cxyFanGong:choose_sha");
    let _ = answer(&mut e_fg, &c_fg, vec![ChoiceOption::Card(fg2)]);
    let c_jl = answer_pass_until(&mut e_fg, "cxyJieLve:confirm");
    let _ = answer(&mut e_fg, &c_jl, vec![ChoiceOption::Confirm]);
    drain(&mut e_fg);
    assert!(e_fg.debug_hand(1).contains(&eh2), "cxyJieLve looted hand");
    assert!(e_fg.debug_hand(1).contains(&eq2), "cxyJieLve looted equip");
    assert!(e_fg.debug_linked(1), "cxyJieLve flipped owner link");
    assert_eq!(e_fg.debug_hp(2), 5);
    let _ = (enemy_h, fg_sha, mq_card);

    // cxyMoQu explicit test: TurnEnd with hand <= hp draws 2; friend taking damage forces 1 discard
    let mut e_mq = start_versus(0x1606, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_mq.debug_grant(0, "cxyMoQu");
    let d0 = place_kind(&mut e_mq, 0, CardKind::Shan);
    let _ = e_mq.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal);
    drain(&mut e_mq);
    assert!(!e_mq.debug_hand(0).contains(&d0), "cxyMoQu discarded 1 on ally damage");
    e_mq.debug_set_turn(0, Phase::End, 1);
    let _ = next_phase(&mut e_mq); // End -> TurnEndWindow (hand 0 <= hp 6) -> draws 2!
    drain(&mut e_mq);
    assert_eq!(e_mq.debug_hand(0).len(), 2, "cxyMoQu drew 2 when hand <= hp at TurnEnd");

    // 2. cxyYaoWu: Red Sha damage prompts attacker mode 0 (+1 HP) vs mode 1 (+1 draw); full HP draws 1
    let mut e_yw = start_taixu(0x1607, 2);
    e_yw.debug_grant(1, "cxyYaoWu");
    e_yw.debug_set_hp(0, 4);
    let rsha = place_kind_suit(&mut e_yw, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_yw, 0, rsha, vec![1]);
    let c_yw = expect_pending(&e_yw, "cxyYaoWu:choose_mode");
    assert_eq!(c_yw.observer, 0);
    assert_eq!(ai_plan(&e_yw, 0), GameAction::Answer {
        choice_id: c_yw.id,
        selection: vec![ChoiceOption::Tag(0)],
    });
    let _ = answer(&mut e_yw, &c_yw, vec![ChoiceOption::Tag(0)]);
    drain(&mut e_yw);
    assert_eq!(e_yw.debug_hp(0), 5, "injured attacker chose mode 0 to heal 1");

    // 3. cxyYingHun: Prepare when injured -> draw X / discard 1 (mode 0) vs draw 1 / discard X (mode 1)
    let mut e_yh = start_versus(0x1608, &[&[0], &[1]], &[Faction::Wei, Faction::Shu]);
    e_yh.debug_grant(0, "cxyYingHun");
    e_yh.debug_set_hp(0, 4); // X = 6 - 4 = 2
    let _ = place_kind(&mut e_yh, 1, CardKind::Shan);
    let _ = place_kind_suit(&mut e_yh, 1, CardKind::Tao, Suit::Heart);
    e_yh.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_yh); // Seat 1 End -> Seat 0 TurnBegin -> Prepare -> cxyYingHun
    let c_yt = expect_pending(&e_yh, "cxyYingHun:choose_target");
    let _ = answer(&mut e_yh, &c_yt, vec![ChoiceOption::Player(1)]);
    let c_ym = expect_pending(&e_yh, "cxyYingHun:choose_mode");
    assert_eq!(ai_plan(&e_yh, 0), GameAction::Answer {
        choice_id: c_ym.id,
        selection: vec![ChoiceOption::Tag(1)], // enemy target -> mode 1 (draw 1, discard X=2)
    });
    let e_yh_done = save_restore_and_compare_pending(&e_yh, &c_ym, vec![ChoiceOption::Tag(1)]);
    assert_eq!(e_yh_done.debug_hand(1).len(), 1, "2 initial + 1 draw - 2 discard = 1");

    // 4. cxyPoLu: death of self OR friend-slain enemy draws `cnt` (1st=1, 2nd=2) for friends!
    let mut e_pl = start_versus(0x1609, &[&[0, 1], &[2, 3]], &[Faction::Wei, Faction::Shu]);
    e_pl.debug_grant(0, "cxyPoLu");
    e_pl.debug_grant(1, "咆哮");
    e_pl.debug_set_hp(2, 1);
    e_pl.debug_set_hp(0, 1);
    e_pl.debug_set_turn(1, Phase::Play, 1);
    let sk1 = place_kind_suit(&mut e_pl, 1, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_pl, 1, sk1, vec![2]); // ally 1 kills enemy 2 (+3 2v2 reward) -> 1st cxyPoLu (cnt=1) draws 1 each for 0 and 1!
    drain(&mut e_pl);
    assert_eq!(e_pl.debug_hand(0).len(), 1);
    assert_eq!(e_pl.debug_hand(1).len(), 4);
    e_pl.debug_clear_hand(0);
    // Avoid random Tao in ally 1's 4-card hand auto-saving seat 0 while preserving 4-card count
    e_pl.debug_clear_hand(1);
    for _ in 0..4 {
        let _ = place_kind(&mut e_pl, 1, CardKind::Sha);
    }
    // Enemy 3 kills seat 0 (`cxyPoLu` self death -> cnt=2) -> alive friend seat 1 draws +2 = 6!
    e_pl.debug_set_turn(3, Phase::Play, 1);
    e_pl.state_core_mut().turn.sha_used_play_phase = false;
    let sk2 = place_kind_suit(&mut e_pl, 3, CardKind::Sha, Suit::Club);
    let out_pl = play_card(&mut e_pl, 3, sk2, vec![0]);
    assert!(
        matches!(out_pl, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "sk2 rejected: {out_pl:?}"
    );
    drain(&mut e_pl);
    assert!(!e_pl.debug_alive(0));
    assert_eq!(e_pl.debug_hand(1).len(), 6, "2nd cxyPoLu on own death draws 2 for alive friend");
}
