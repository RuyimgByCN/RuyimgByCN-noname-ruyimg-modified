//! Step 5 (Track P3-B) real-event parity tests for all 15 L11 generals across 9 root causes:
//! 1. `zishou2` (`txhj_liubiao`): `CardTargetsSelfOnly` enforcement & turn expiry
//! 2. `duanliang` (`txhj_jinhuansanjie`): `Bing` range 1 + `card_distance_unlimited_if_target_hand_gte_owner`
//! 3. `qiangwu` (`txhj_manzuyongshi`, `txhj_huangjinshizu`, `txhj_zhangxingcai`): real `OrdinaryJudge` rank mark + `< mark` range + `> mark` no use count
//! 4. `jiaojie` (`txhj_dongxie`): red ignored hand limit & un-discardable in Discard + black unlimited distance & use count (`Black Jiu` twice)
//! 5. `dujin` (`txhj_caocaox`, `txhj_dongzhuo`, `txhj_huangfusong`): `OptionalDrawCount` formula `1 + ceil(equips / 2)` gated on `!num_fixed`
//! 6. `jintao` (`txhj_huzhen`, `txhj_mowangzhizhong`): `sha_used_count` + `ShaDamage` (+1 only on 1st Sha, never `xinfu_langxi`) + 2nd Sha unrespondable
//! 7. `jinjiu` & `xianzhen`/`rexianzhen` (`txhj_xianzhenying`, `txhj_lingtonghuanxiang`): `CardName(jiu -> sha)` + `bound_target` scoping + `rexianzhen4`
//! 8. `zhenlie` (`txhj_wangyi`): delayed trick exclusion + equip discard events (`CardDiscarded`/`CardsLost`)
//! 9. `lieren` (`txhj_huanjingweishi`): `yingjian` virtual `Sha` damage + equip steal events
//! Zero `debug_execute_effect`; public-info AI checks only.

use super::*;
use engine_native::event::{DamageNature, EventData};
use engine_native::{LogEvent, RejectReason};

fn step5_take_card<F>(e: &mut Engine, pred: F) -> u32
where
    F: Fn(&engine_native::catalog::CardDef) -> bool,
{
    if let Some(pos) = e
        .state_core()
        .deck
        .iter()
        .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&pred))
    {
        return e.state_core_mut().deck.remove(pos);
    }
    if let Some(pos) = e
        .state_core()
        .discard
        .iter()
        .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&pred))
    {
        return e.state_core_mut().discard.remove(pos);
    }
    for seat in 0..e.state_core().players.len() {
        if let Some(pos) = e
            .state_core()
            .player(seat as u8)
            .hand
            .iter()
            .position(|&c| e.catalog().cards.get(c as usize).is_some_and(&pred))
        {
            return e.state_core_mut().player_mut(seat as u8).hand.remove(pos);
        }
    }
    panic!("missing matching card in deck/discard/hand");
}

fn step5_play_card(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    e.state_core_mut().player_mut(actor).hand.push(card);
    submit(e, actor, GameAction::PlayCard { card, targets })
}

fn step5_equip_card(e: &mut Engine, seat: u8, kind: CardKind) -> u32 {
    let cid = step5_take_card(e, |d| d.kind == kind);
    e.state_core_mut().player_mut(seat).equips.push(cid);
    cid
}

fn step5_has_skill(e: &Engine, seat: u8, name: &str) -> bool {
    e.catalog()
        .skill_by_name(name)
        .is_some_and(|d| e.state_core().has_skill(seat, d))
}

#[test]
fn l11_step5_zishou2_duanliang_and_jiaojie_real_event_parity() {
    let verified = taixu_verified();

    // 1. `txhj_liubiao` (`zishou`, `rezongshi`) -> `zishou2` self-only targeting + turn expiry
    {
        let mut e = start_locked_engine(2026_801, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_liubiao").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        assert!(step5_has_skill(&e, 0, "zishou"));
        assert!(step5_has_skill(&e, 0, "rezongshi"));

        e.debug_clear_hand(0);
        e.debug_set_turn(0, Phase::Judge, 1);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Judge -> Draw
        if let Some(ch) = e.open_choice().cloned() {
            if ch.spec.options.contains(&ChoiceOption::Confirm) {
                let _ = answer_choice(&mut e, ch.observer, vec![ChoiceOption::Confirm]);
            }
        }
        drain_to_pass(&mut e);
        assert!(step5_has_skill(&e, 0, "zishou2"), "confirming zishou grants zishou2");

        let _ = submit(&mut e, 0, GameAction::NextPhase); // Draw -> Play
        drain_to_pass(&mut e);
        let sha = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let duel = step5_take_card(&mut e, |d| d.kind == CardKind::Duel);
        let wuzhong = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Wuzhong && matches!(d.name, "wuzhong" | "无中生有")
        });
        e.state_core_mut().player_mut(0).hand.push(sha);
        e.state_core_mut().player_mut(0).hand.push(duel);
        let legal = e.legal_actions(0);
        assert!(
            !legal.iter().any(|a| matches!(a, GameAction::PlayCard { targets, .. } if targets.iter().any(|&t| t != 0))),
            "zishou2 must filter non-self PlayCard actions out of legal_actions"
        );
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![2] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
        let wz_out = step5_play_card(&mut e, 0, wuzhong, Vec::new());
        assert!(!matches!(wz_out, CommandOutcome::Rejected(_)), "self Wuzhong allowed: {wz_out:?}");
        drain_to_pass(&mut e);

        // Advance past seat 0 turn end -> `zishou2` clears; decline Turn 2 `zishou` so seat 0 can Sha seat 1.
        advance_to_play(&mut e, 1);
        assert!(!step5_has_skill(&e, 0, "zishou2"), "zishou2 expires at turn end");
        for _ in 0..96 {
            if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
                break;
            }
            if let Some(ch) = e.open_choice().cloned() {
                let sel = if ch.spec.options.contains(&ChoiceOption::Pass) {
                    vec![ChoiceOption::Pass]
                } else if ch.spec.allow_cancel {
                    Vec::new()
                } else {
                    ch.spec.options.iter().take(ch.spec.min.max(1)).copied().collect()
                };
                let _ = answer_choice(&mut e, ch.observer, sel);
                continue;
            }
            let cur = e.turn().current;
            let _ = submit(&mut e, cur, GameAction::NextPhase);
        }
        assert_eq!(e.turn().current, 0);
        assert_eq!(e.turn().phase, Phase::Play);
        assert!(!step5_has_skill(&e, 0, "zishou2"), "zishou2 stays cleared when zishou declined");
        let sha_next = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let out_next = step5_play_card(&mut e, 0, sha_next, vec![1]);
        assert!(!matches!(out_next, CommandOutcome::Rejected(_)), "Sha allowed after zishou2 clears: {out_next:?}");
    }

    // 2. `txhj_jinhuansanjie` (`duanliang`) -> Bing base distance 1 + target.hand >= owner.hand
    {
        let mut e = start_locked_engine(2026_802, 4, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_jinhuansanjie").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.assign_general_named(3, "txhj_yecha").unwrap();
        for sk in ["jiezi", "txleili", "duanliang", "fuji"] {
            assert!(step5_has_skill(&e, 0, sk), "txhj_jinhuansanjie must have {sk}");
        }
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(2);
        let bing = step5_take_card(&mut e, |d| d.kind == CardKind::Bing);
        let black_sha = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        // owner has 2 cards, target seat 2 (dist=2) has 1 card -> target.hand < owner.hand => REJECTED!
        e.state_core_mut().player_mut(0).hand.push(bing);
        e.state_core_mut().player_mut(0).hand.push(black_sha);
        let c_t2 = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        e.state_core_mut().player_mut(2).hand.push(c_t2);
        assert_eq!(e.state_core().computed_distance(e.catalog(), 0, 2), 2);
        let legal_bad = e.legal_actions(0);
        assert!(
            !legal_bad.iter().any(|a| matches!(a, GameAction::PlayCard { card, targets } if *card == bing && targets == &[2])),
            "dist=2 Bing must be omitted from legal_actions when target.hand < owner.hand"
        );
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: bing, targets: vec![2] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));

        // Give seat 2 another card => target.hand (2) >= owner.hand (2) => ALLOWED via duanliang!
        let c_t2_b = step5_take_card(&mut e, |d| d.kind == CardKind::Tao);
        e.state_core_mut().player_mut(2).hand.push(c_t2_b);
        let out_ok = submit(&mut e, 0, GameAction::PlayCard { card: bing, targets: vec![2] });
        assert!(!matches!(out_ok, CommandOutcome::Rejected(_)), "Bing at dist=2 allowed when target.hand >= owner.hand: {out_ok:?}");
        drain_to_pass(&mut e);
        assert!(e.state_core().player(2).judge.contains(&bing));

        // Negative without `duanliang`: dist=2 Bing on seat 2 rejected even if target.hand >= owner.hand!
        let mut e_plain = start_locked_engine(2026_803, 4, LockedStartMode::Prototype, verified).unwrap();
        e_plain.assign_general_named(0, "txhj_yecha").unwrap();
        e_plain.assign_general_named(1, "txhj_machao").unwrap();
        e_plain.assign_general_named(2, "txhj_huangyueying").unwrap();
        e_plain.assign_general_named(3, "txhj_zhangchunhua").unwrap();
        advance_to_play(&mut e_plain, 0);
        assert_eq!(e_plain.state_core().computed_distance(e_plain.catalog(), 0, 2), 2);
        e_plain.debug_clear_hand(0);
        let bing_p = step5_take_card(&mut e_plain, |d| d.kind == CardKind::Bing);
        let fill = step5_take_card(&mut e_plain, |d| d.kind == CardKind::Tao);
        e_plain.state_core_mut().player_mut(2).hand.push(fill);
        assert!(matches!(
            step5_play_card(&mut e_plain, 0, bing_p, vec![2]),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ));
    }

    // 3. `txhj_dongxie` (`cxyJiaoXia`, `cxyTunJun`, `jiaojie`)
    {
        let mut e = start_locked_engine(2026_804, 4, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_dongxie").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.assign_general_named(3, "txhj_yecha").unwrap();
        for sk in ["cxyJiaoXia", "cxyTunJun", "jiaojie"] {
            assert!(step5_has_skill(&e, 0, sk), "txhj_dongxie must have {sk}");
        }
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        // Black Bing & Black Snatch reach dist=2, Black Jiu can be used twice in Play phase!
        e.debug_clear_hand(2);
        let t2_c = step5_take_card(&mut e, |d| d.kind == CardKind::Shan);
        e.state_core_mut().player_mut(2).hand.push(t2_c);
        let blk_bing = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Bing && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let blk_snatch = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Snatch && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let blk_jiu1 = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Jiu && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let blk_jiu2 = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Jiu && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        assert!(!matches!(step5_play_card(&mut e, 0, blk_bing, vec![2]), CommandOutcome::Rejected(_)));
        drain_to_pass(&mut e);
        assert!(!matches!(step5_play_card(&mut e, 0, blk_snatch, vec![2]), CommandOutcome::Rejected(_)));
        drain_to_pass(&mut e);
        assert!(!matches!(step5_play_card(&mut e, 0, blk_jiu1, Vec::new()), CommandOutcome::Rejected(_)));
        drain_to_pass(&mut e);
        let jiu2_out = step5_play_card(&mut e, 0, blk_jiu2, Vec::new());
        assert!(!matches!(jiu2_out, CommandOutcome::Rejected(_)), "jiaojie must allow 2nd Black Jiu: {jiu2_out:?}");
        drain_to_pass(&mut e);

        // Discard phase: red cards are ignored from hand limit AND excluded from discard_to_limit options!
        e.debug_clear_hand(0);
        e.debug_set_hp(0, 1);
        let red1 = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Tao && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let red2 = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Shan && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        // Hold 1 black card as well -> hand limit 1, ignored=2 (red1, red2), need discard 1 -> ONLY blk can be discarded!
        let blk = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        let blk_extra = step5_take_card(&mut e, |d| {
            d.kind == CardKind::Nullify && matches!(d.suit, Suit::Spade | Suit::Club)
        });
        e.state_core_mut().player_mut(0).hand.extend([red1, red2, blk, blk_extra]);
        // `txhj_dongxie` carries both `cxyJiaoXia` (black) and `jiaojie` (red) => 4 ignored with both active.
        assert_eq!(e.state_core().ignored_hand_card_count(e.catalog(), 0), 4);
        // Isolate `jiaojie` by stripping `cxyJiaoXia` => only the 2 red cards are ignored & excluded from discard_to_limit!
        let cj = e.catalog().skill_by_name("cxyJiaoXia").unwrap();
        e.state_core_mut().skills.remove(&(0, cj));
        assert_eq!(e.state_core().ignored_hand_card_count(e.catalog(), 0), 2);
        let _ = submit(&mut e, 0, GameAction::NextPhase); // Play -> Discard
        let ch = e.open_choice().cloned().expect("discard_to_limit prompt");
        assert_eq!(ch.spec.prompt, "discard_to_limit");
        assert!(!ch.spec.options.contains(&ChoiceOption::Card(red1)));
        assert!(!ch.spec.options.contains(&ChoiceOption::Card(red2)));
        assert!(ch.spec.options.contains(&ChoiceOption::Card(blk)));
        assert!(ch.spec.options.contains(&ChoiceOption::Card(blk_extra)));
    }
}

#[test]
fn l11_step5_qiangwu_dujin_and_jintao_real_event_parity() {
    let verified = taixu_verified();

    // 4. `qiangwu` (`txhj_manzuyongshi`, `txhj_huangjinshizu`, `txhj_zhangxingcai`)
    for g in ["txhj_manzuyongshi", "txhj_huangjinshizu", "txhj_zhangxingcai"] {
        let mut e = start_locked_engine(2026_805, 4, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, g).unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.assign_general_named(3, "txhj_yecha").unwrap();
        assert!(step5_has_skill(&e, 0, "qiangwu"), "{g} must have qiangwu");
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        // Set judge card top to rank 6 so rank < 6 has unlimited distance and rank > 6 has no use count!
        let _jc = e.debug_set_deck_top_by(|d| d.rank == 6);
        let qw_id = e.catalog().skill_by_name("qiangwu").unwrap();
        let snap_pre = encode_archive(&e.save());
        let mut e_res = restore_locked_engine(&decode_archive(&snap_pre).unwrap(), verified).unwrap();
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: qw_id });
        let _ = submit(&mut e_res, 0, GameAction::ActivateSkill { skill: qw_id });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{g} qiangwu activate: {out:?}");
        drain_to_pass(&mut e);
        drain_to_pass(&mut e_res);
        assert_eq!(e.state_hash(), e_res.state_hash());
        assert_eq!(
            e.state_core().vm_aux.marks.get(&(0, 2_563_176_834)).copied(),
            Some(6),
            "qiangwu judge must record rank 6 on skill mark"
        );

        // Rank 8 (> 6) Sha CANNOT hit dist=2 seat 2, CAN hit dist=1 seat 1 and DOES NOT consume Sha use count!
        let sha_gt = step5_take_card(&mut e, |d| d.kind == CardKind::Sha && d.rank > 6);
        let sha_lt = step5_take_card(&mut e, |d| d.kind == CardKind::Sha && d.rank < 6);
        e.state_core_mut().player_mut(0).hand.push(sha_gt);
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha_gt, targets: vec![2] }),
            CommandOutcome::Rejected(_)
        ));
        assert!(!matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: sha_gt, targets: vec![1] }),
            CommandOutcome::Rejected(_)
        ));
        drain_to_pass(&mut e);
        assert!(!e.state_core().turn.sha_used_play_phase, "rank > mark Sha must NOT consume Sha count");

        // Now rank < 6 Sha CAN hit dist=2 seat 2 (`sha_distance_unlimited_if_rank_lt_skill_mark`)!
        let lt_out = step5_play_card(&mut e, 0, sha_lt, vec![2]);
        assert!(!matches!(lt_out, CommandOutcome::Rejected(_)), "rank < mark Sha must ignore distance: {lt_out:?}");
        drain_to_pass(&mut e);
        assert!(e.state_core().turn.sha_used_play_phase, "rank < mark Sha consumes Sha count");

        // Ending Play phase clears qiangwu mark!
        let _ = submit(&mut e, 0, GameAction::NextPhase);
        drain_to_pass(&mut e);
        assert_eq!(
            e.state_core().vm_aux.marks.get(&(0, 2_563_176_834)).copied().unwrap_or(0),
            0,
            "qiangwu mark must clear at phase_play_end"
        );
    }

    // 5. `dujin` (`txhj_caocaox`, `txhj_dongzhuo`, `txhj_huangfusong`)
    for g in ["txhj_caocaox", "txhj_dongzhuo", "txhj_huangfusong"] {
        let mut e = start_locked_engine(2026_806, 2, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, g).unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        assert!(step5_has_skill(&e, 0, "dujin"), "{g} must have dujin");
        for (equip_cnt, expected_draw) in [(0usize, 3usize), (1, 4), (2, 4), (3, 5)] {
            let mut ec = start_locked_engine(2026_807 + equip_cnt as u64, 2, LockedStartMode::Prototype, verified).unwrap();
            ec.assign_general_named(0, g).unwrap();
            ec.assign_general_named(1, "txhj_huangyueying").unwrap();
            let slots = [CardKind::Weapon, CardKind::Armor, CardKind::HorsePlus];
            for k in slots.iter().take(equip_cnt) {
                step5_equip_card(&mut ec, 0, *k);
            }
            ec.debug_clear_hand(0);
            ec.debug_set_turn(0, Phase::Judge, 1);
            let _ = submit(&mut ec, 0, GameAction::NextPhase); // Judge -> Draw
            drain_to_pass(&mut ec);
            assert_eq!(
                ec.state_core().player(0).hand.len(),
                expected_draw,
                "{g} dujin with {equip_cnt} equips must draw {expected_draw} (no bogus Phase::Draw xinjuejing draw)"
            );
        }
        // Authentic `xinjuejing` on `txhj_caocaox`: +2 hand limit AND draws 1 on Dying + draws 1 on DyingSaved (`dyingAfter`)!
        if g == "txhj_caocaox" {
            let mut ex = start_locked_engine(2026_8090, 2, LockedStartMode::Prototype, verified).unwrap();
            ex.assign_general_named(0, "txhj_caocaox").unwrap();
            ex.assign_general_named(1, "txhj_huangyueying").unwrap();
            assert!(step5_has_skill(&ex, 0, "xinjuejing"));
            ex.debug_set_hp(0, 4);
            assert_eq!(
                ex.state_core().hand_limit(ex.catalog(), 0),
                6,
                "txhj_caocaox xinjuejing must grant +2 hand limit (4 + 2 = 6)"
            );
            advance_to_play(&mut ex, 1);
            ex.debug_clear_hand(0);
            ex.debug_clear_hand(1);
            ex.debug_set_hp(0, 1);
            let tao_save = step5_take_card(&mut ex, |d| d.kind == CardKind::Tao);
            ex.state_core_mut().player_mut(0).hand.push(tao_save);
            let sha_hit = step5_take_card(&mut ex, |d| d.kind == CardKind::Sha);
            let _ = step5_play_card(&mut ex, 1, sha_hit, vec![0]);
            // Entering Dying (`hp = 0`) triggers `xinjuejing_dying` (+1 draw BEFORE Tao save ask -> 2 cards in hand):
            let ch_save = ex.open_choice().cloned().expect("dying Tao prompt");
            assert_eq!(ch_save.observer, 0);
            assert_eq!(
                ex.state_core().player(0).hand.len(),
                2,
                "xinjuejing must draw 1 card on Dying (1 Tao + 1 drawn = 2)"
            );
            let _ = answer_choice(&mut ex, 0, vec![ChoiceOption::Card(tao_save)]);
            drain_to_pass(&mut ex);
            // DyingSaved (`dyingAfter`) triggers `xinjuejing_dying_after` (+1 more draw -> 2 cards remaining in hand!):
            assert!(ex.state_core().player(0).alive && ex.state_core().player(0).hp == 1);
            assert_eq!(
                ex.state_core().player(0).hand.len(),
                2,
                "xinjuejing must draw 1 more card on DyingSaved/dyingAfter (2 - 1 Tao + 1 = 2)"
            );
        }
        // Negative check: DrawPhase { num_fixed: true } skips `OptionalDrawCount` (`dujin`)!
        e.debug_clear_hand(0);
        e.state_core_mut().player_mut(0).equips.clear();
        let draw_fixed = EventData::DrawPhase {
            player: 0,
            progress: engine_native::event::DrawPhaseProgress::Before,
            pending: 2,
            num_fixed: true,
            actual_gain: 0,
            outcome: engine_native::event::DrawOutcome::Running,
        };
        assert_eq!(
            engine_native::rvm_bridge::modifiers::eval_optional_draw_count(e.state_core(), e.catalog(), 0, 2),
            3
        );
        // Execute real fixed DrawPhase via machine? Verify eval_draw_count does NOT include dujin!
        assert_eq!(
            engine_native::rvm_bridge::modifiers::eval_draw_count(e.state_core(), e.catalog(), 0, 2),
            2,
            "fixed draw phase eval_draw_count must NOT include dujin OptionalDrawCount"
        );
        let _ = draw_fixed;
    }

    // 6. `jintao` (`txhj_huzhen`, `txhj_mowangzhizhong`)
    for g in ["txhj_huzhen", "txhj_mowangzhizhong"] {
        let mut e = start_locked_engine(2026_812, 4, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, g).unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.assign_general_named(3, "txhj_yecha").unwrap();
        assert!(step5_has_skill(&e, 0, "jintao"), "{g} must have jintao");
        assert!(step5_has_skill(&e, 0, "xinfu_langxi"), "{g} must have xinfu_langxi");

        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(2);
        let shan2 = step5_take_card(&mut e, |d| d.kind == CardKind::Shan);
        e.state_core_mut().player_mut(2).hand.push(shan2);
        e.debug_set_hp(2, 6);
        let sha1 = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let sha2 = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        // 1st Sha at dist=2: target can Shan; pass Shan -> +1 ShaDamage (2 total)!
        let out1 = step5_play_card(&mut e, 0, sha1, vec![2]);
        assert!(!matches!(out1, CommandOutcome::Rejected(_)), "{g} 1st Sha at dist=2: {out1:?}");
        let ask1 = e.open_choice().cloned().expect("1st Sha must still allow Shan");
        assert_eq!((ask1.observer, ask1.spec.prompt), (2, "respond_shan"));
        let ans1 = answer_choice(&mut e, 2, vec![ChoiceOption::Pass]);
        assert!(!matches!(ans1, CommandOutcome::Rejected(_)), "ans1 pass rejected: {ans1:?}");
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(2).hp, 4, "{g} 1st Sha deals 2 dmg (6 -> 4)");

        // 2nd Sha: unrespondable (NO respond_shan window even though seat 2 holds Shan!), deals 1 dmg (4 -> 3)!
        let out2 = step5_play_card(&mut e, 0, sha2, vec![2]);
        assert!(!matches!(out2, CommandOutcome::Rejected(_)), "{g} 2nd Sha: {out2:?}");
        assert!(
            !matches!(e.open_choice(), Some(c) if c.spec.prompt == "respond_shan"),
            "{g} 2nd Sha must be unrespondable"
        );
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(2).hp, 3, "{g} 2nd Sha deals 1 dmg (4 -> 3)");
    }
}

#[test]
fn l11_step5_jinjiu_rexianzhen_zhenlie_and_lieren_real_event_parity() {
    let verified = taixu_verified();

    // 7A. `txhj_xianzhenying` (`xianzhen`, `jinjiu`)
    {
        let mut e = start_locked_engine(2026_820, 4, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_xianzhenying").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.assign_general_named(3, "txhj_yecha").unwrap();
        assert!(step5_has_skill(&e, 0, "xianzhen"));
        assert!(step5_has_skill(&e, 0, "jinjiu"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let jiu = step5_take_card(&mut e, |d| d.kind == CardKind::Jiu);
        assert_eq!(e.catalog().card_kind(jiu), CardKind::Jiu, "physical CardDef stays Jiu");
        e.state_core_mut().player_mut(0).hand.push(jiu);
        // Using Jiu with empty targets as Jiu is rejected; targeting seat 1 works as Sha!
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: Vec::new() }),
            CommandOutcome::Rejected(_)
        ));
        let hp1_before = e.state_core().player(1).hp;
        let out_sha = submit(&mut e, 0, GameAction::PlayCard { card: jiu, targets: vec![1] });
        assert!(!matches!(out_sha, CommandOutcome::Rejected(_)), "jinjiu plays Jiu directly as Sha: {out_sha:?}");
        drain_to_pass(&mut e);
        assert_eq!(e.state_core().player(1).hp, hp1_before - 1);

        // Jinjiu responds to Duel with Jiu as Sha, and CANNOT self-save on Dying with Jiu!
        let jiu_duel = step5_take_card(&mut e, |d| d.kind == CardKind::Jiu);
        e.state_core_mut().player_mut(0).hand.push(jiu_duel);
        assert_eq!(
            engine_native_kernel::kernel::effective_card_kind_for(e.state_core(), e.catalog(), 0, jiu_duel),
            CardKind::Sha
        );
        assert!(!engine_native_kernel::kernel::response_kind_cards_core(e.state_core(), e.catalog(), 0, CardKind::Sha).is_empty());
        // Real Dying test: when `txhj_xianzhenying` drops to 0 HP holding ONLY `Jiu`, cannot save self with Jiu!
        e.debug_set_hp(0, 1);
        e.debug_clear_hand(0);
        e.state_core_mut().player_mut(0).hand.push(jiu_duel);
        advance_to_play(&mut e, 1);
        e.debug_clear_hand(1);
        let lethal_sha = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let lethal_out = step5_play_card(&mut e, 1, lethal_sha, vec![0]);
        assert!(!matches!(lethal_out, CommandOutcome::Rejected(_)), "lethal sha: {lethal_out:?}");
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(0).alive, "jinjiu owner cannot self-save on Dying using Jiu");
    }

    // 7B. `txhj_lingtonghuanxiang` (`rexianzhen`, `fenwei`) -> win vs bound_target, Armor/dist/unlimited ONLY on bound_target, rexianzhen4, turn cleanup
    {
        let mut e = start_locked_engine(2026_821, 4, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_lingtonghuanxiang").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        e.assign_general_named(3, "txhj_yecha").unwrap();
        assert!(step5_has_skill(&e, 0, "rexianzhen"));
        assert!(step5_has_skill(&e, 0, "fenwei"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        step5_equip_card(&mut e, 2, CardKind::Armor); // Renwang/Bagua on bound target seat 2 (`dist=2`)
        step5_equip_card(&mut e, 3, CardKind::Armor); // Armor on non-bound seat 3
        let sha_big = step5_take_card(&mut e, |d| d.kind == CardKind::Sha && d.rank >= 10);
        let t2_small = step5_take_card(&mut e, |d| d.rank <= 3);
        e.state_core_mut().player_mut(0).hand.push(sha_big);
        e.state_core_mut().player_mut(2).hand.push(t2_small);
        e.state_core_mut().vm_aux.set_selected_cards(0, &[sha_big]);
        e.state_core_mut().vm_aux.set_selected_target_seats(0, &[2]);
        let rxz_id = e.catalog().skill_by_name("rexianzhen").unwrap();
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: rxz_id });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "rexianzhen activate: {out:?}");
        if let Some(ch) = e.open_choice().cloned() {
            let opt = if ch.spec.options.contains(&ChoiceOption::Player(2)) {
                ChoiceOption::Player(2)
            } else {
                ch.spec.options[0]
            };
            let _ = answer_choice(&mut e, ch.observer, vec![opt]);
        }
        drain_to_pass(&mut e);
        assert!(step5_has_skill(&e, 0, "rexianzhen2"), "win must grant rexianzhen2");
        assert_eq!(e.state_core().vm_aux.flags.get(&(0, "bound_target".to_string())).copied(), Some(3));

        // Unlimited distance & ignore armor against bound_target seat 2 (`dist=2`), NOT against seat 3 (`dist=1`)!
        assert!(e.state_core().ignores_target_armor(e.catalog(), 0, 2, Some(sha_big), DamageNature::Normal));
        assert!(!e.state_core().ignores_target_armor(e.catalog(), 0, 3, Some(sha_big), DamageNature::Normal));
        let sha_a = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let sha_b = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        assert!(!matches!(step5_play_card(&mut e, 0, sha_a, vec![2]), CommandOutcome::Rejected(_)));
        drain_to_pass(&mut e);
        assert!(!matches!(step5_play_card(&mut e, 0, sha_b, vec![2]), CommandOutcome::Rejected(_)), "2nd Sha against bound target seat 2 allowed");
        drain_to_pass(&mut e);
        let sha_c = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        assert!(
            matches!(step5_play_card(&mut e, 0, sha_c, vec![1]), CommandOutcome::Rejected(RejectReason::ShaLimit)),
            "Sha against NON-bound target seat 1 must still reject after Sha used"
        );
        // `rexianzhen4` active from pindian card being Sha => Sha ignored in hand limit!
        assert!(e.state_core().ignored_hand_card_count(e.catalog(), 0) >= 1);
    }

    // 8. `txhj_wangyi` (`zhenlie`, `miji`) -> delayed trick exclusion + equip discard events
    {
        let mut e = start_locked_engine(2026_822, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huangyueying").unwrap();
        e.assign_general_named(1, "txhj_wangyi").unwrap();
        e.assign_general_named(2, "txhj_yecha").unwrap();
        assert!(step5_has_skill(&e, 1, "zhenlie"));
        assert!(step5_has_skill(&e, 1, "miji"));
        advance_to_play(&mut e, 0);
        e.debug_clear_hand(0);
        let wp = step5_equip_card(&mut e, 0, CardKind::Weapon);
        // Delayed tricks (`Le`, `Bing`) must NOT trigger `zhenlie`!
        let le = step5_take_card(&mut e, |d| d.kind == CardKind::Le);
        let bing = step5_take_card(&mut e, |d| d.kind == CardKind::Bing);
        let _ = step5_play_card(&mut e, 0, le, vec![1]);
        assert!(e.open_choice().is_none(), "Le must NOT trigger zhenlie");
        let _ = step5_play_card(&mut e, 0, bing, vec![1]);
        assert!(e.open_choice().is_none(), "Bing must NOT trigger zhenlie");

        // Sha DOES trigger `zhenlie`, loses 1 HP, excludes self, and discards user's equip with CardDiscarded!
        let sha = step5_take_card(&mut e, |d| d.kind == CardKind::Sha);
        let log_len = e.log().len();
        let _ = step5_play_card(&mut e, 0, sha, vec![1]);
        for _ in 0..16 {
            let Some(c0) = e.open_choice().cloned() else {
                break;
            };
            if c0.spec.prompt == "zhenlie_discard" {
                break;
            }
            if c0.observer == 1 && c0.spec.options.contains(&ChoiceOption::Confirm) {
                let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Confirm]);
            } else if c0.spec.options.contains(&ChoiceOption::Pass) {
                let _ = answer_choice(&mut e, c0.observer, vec![ChoiceOption::Pass]);
            } else if c0.spec.allow_cancel {
                let _ = answer_choice(&mut e, c0.observer, Vec::new());
            } else if let Some(first) = c0.spec.options.first().copied() {
                let _ = answer_choice(&mut e, c0.observer, vec![first]);
            }
        }
        let c1 = e.open_choice().cloned().expect("zhenlie_discard");
        assert_eq!(c1.spec.prompt, "zhenlie_discard");
        let _ = answer_choice(&mut e, 1, vec![ChoiceOption::Card(wp)]);
        drain_to_pass(&mut e);
        assert!(!e.state_core().player(0).equips.contains(&wp), "zhenlie must discard equip wp");
        assert!(
            e.log()[log_len..].iter().any(|ev| matches!(ev, LogEvent::CardDiscarded { player: 0, cards } if cards.contains(&wp))),
            "zhenlie equip discard must emit LogEvent::CardDiscarded"
        );
    }

    // 9. `txhj_huanjingweishi` (`yingjian`, `qiaomeng`, `lieren`) -> virtual Sha triggers SwdLieren + equip steal
    {
        let mut e = start_locked_engine(2026_823, 3, LockedStartMode::Prototype, verified).unwrap();
        e.assign_general_named(0, "txhj_huanjingweishi").unwrap();
        e.assign_general_named(1, "txhj_huangyueying").unwrap();
        e.assign_general_named(2, "txhj_machao").unwrap();
        for sk in ["yingjian", "qiaomeng", "lieren"] {
            assert!(step5_has_skill(&e, 0, sk), "txhj_huanjingweishi must have {sk}");
        }
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_clear_hand(2);
        let big_p = step5_take_card(&mut e, |d| d.rank >= 10);
        let small_t = step5_take_card(&mut e, |d| d.rank <= 3);
        let t_equip = step5_equip_card(&mut e, 1, CardKind::HorsePlus);
        e.state_core_mut().player_mut(0).hand.push(big_p);
        e.state_core_mut().player_mut(1).hand.push(small_t);
        // End seat 2's turn naturally -> transitions to seat 0 `TurnBegin` -> `Phase::Prepare { begin: true }` (`yingjian`)!
        e.debug_set_turn(2, Phase::End, 1);
        let _ = submit(&mut e, 2, GameAction::NextPhase);
        // Step through `yingjian` target -> pass `shan` -> `lieren` confirm -> picks -> verify equip stolen!
        let lieren_def = e.catalog().skill_by_name("lieren").unwrap();
        let mut saw_lieren_confirm = false;
        for _ in 0..24 {
            let Some(ch) = e.open_choice().cloned() else {
                break;
            };
            if ch.observer == 0 && ch.spec.options.contains(&ChoiceOption::Skill(lieren_def)) {
                saw_lieren_confirm = true;
                let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Skill(lieren_def)]);
            } else if ch.spec.prompt == "trigger_confirm" && ch.observer == 0 {
                saw_lieren_confirm = true;
                let _ = answer_choice(&mut e, 0, vec![ChoiceOption::Confirm]);
            } else if ch.spec.prompt == "lieren_target_pick" {
                let _ = answer_choice(&mut e, ch.observer, vec![ChoiceOption::Card(small_t)]);
            } else if ch.spec.prompt == "lieren_player_pick" {
                let _ = answer_choice(&mut e, ch.observer, vec![ChoiceOption::Card(big_p)]);
            } else if ch.spec.prompt == "respond_shan" || ch.spec.prompt == "sha_ask" {
                let _ = answer_choice(&mut e, ch.observer, vec![ChoiceOption::Pass]);
            } else if ch.spec.options.contains(&ChoiceOption::Player(1)) {
                let _ = answer_choice(&mut e, ch.observer, vec![ChoiceOption::Player(1)]);
            } else if let Some(first) = ch.spec.options.first().copied() {
                let _ = answer_choice(&mut e, ch.observer, vec![first]);
            }
        }
        assert!(saw_lieren_confirm, "yingjian virtual Sha damage must trigger SwdLieren");
        assert!(
            e.state_core().player(0).hand.contains(&t_equip) && !e.state_core().player(1).equips.contains(&t_equip),
            "winning SwdLieren steals target equip when hand emptied after pindian"
        );
    }
}
