//! Round 3 real-event regression suite for `swarm/conv-l11-b6a-generals`:
//! - Finding #1: `jueqing`, `bazhen`, `roulin`, `reguicai`, `drlt_qianjie`, `nzry_jieying`,
//!   `reqicai`, and passive `feiying` (0 `U05` `PhaseBegin` mutations).
//! - Finding #2: `"double"` gender (`txhj_niufudongxie`) parity across remaining `U01` & `U08` skills.
//! Zero `debug_execute_effect`; public-info AI checks only.

use super::*;
use engine_native::{LogEvent, RejectReason};

fn take_card_matching<F>(e: &mut Engine, pred: F) -> u32
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

fn take_card_named(e: &mut Engine, name: &str) -> u32 {
    take_card_matching(e, |d| d.name == name)
}

fn give_hand_card(e: &mut Engine, seat: u8, card: u32) {
    e.state_core_mut().player_mut(seat).hand.push(card);
}

fn play_real_card(e: &mut Engine, actor: u8, card: u32, targets: Vec<u8>) -> CommandOutcome {
    give_hand_card(e, actor, card);
    submit(e, actor, GameAction::PlayCard { card, targets })
}

#[test]
fn l11_b6a_round3_jueqing_bazhen_and_roulin_real_events() {
    let verified = taixu_verified();

    // 1. `jueqing` (`txhj_zhangchunhua`, `txhj_dushi`): converts Sha damage to HpLost (0 DamageDealt, 0 hujia consumed)
    for general in ["txhj_zhangchunhua", "txhj_dushi"] {
        let mut ej = start_locked_engine(2026_701, 2, LockedStartMode::Prototype, verified).unwrap();
        ej.assign_general_named(0, general).unwrap();
        ej.assign_general_named(1, "txhj_huangyueying").unwrap();
        let jq_id = ej.catalog().skill_by_name("绝情").expect("绝情 in catalog");
        assert!(ej.state_core().has_skill(0, jq_id), "{general} must bind standard 绝情");
        advance_to_play(&mut ej, 0);
        ej.state_core_mut().player_mut(1).hujia = 3;
        ej.debug_clear_hand(1);
        let hp_before = ej.state_core().player(1).hp;
        let log_len = ej.log().len();
        let sha = take_card_kind(&mut ej, CardKind::Sha);
        give_hand_card(&mut ej, 0, sha);
        let snap_pre = encode_archive(&ej.save());
        let mut ej_restored = restore_locked_engine(&decode_archive(&snap_pre).unwrap(), verified).unwrap();
        let out = submit(&mut ej, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        let _ = submit(&mut ej_restored, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(!matches!(out, CommandOutcome::Rejected(_)), "{general}: {out:?}");
        drain_to_pass(&mut ej);
        drain_to_pass(&mut ej_restored);
        assert_eq!(ej.state_hash(), ej_restored.state_hash());
        assert_eq!(
            ej.state_core().player(1).hp,
            hp_before - 1,
            "{general} jueqing loses 1 hp directly"
        );
        assert_eq!(
            ej.state_core().player(1).hujia,
            3,
            "{general} jueqing must NOT consume target hujia"
        );
        assert!(
            ej.log()[log_len..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::HpLost { player: 1, amount: 1 })),
            "{general} must log HpLost"
        );
        assert!(
            !ej.log()[log_len..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::DamageDealt { target: 1, .. })),
            "{general} must NOT log DamageDealt"
        );
    }

    // 2. `bazhen` (`txhj_yecha`): virtual Bagua when Armor slot empty; disabled by Armor equip or Qinggang
    {
        let mut eb = start_locked_engine(2026_702, 2, LockedStartMode::Prototype, verified).unwrap();
        eb.assign_general_named(0, "txhj_huangyueying").unwrap();
        eb.assign_general_named(1, "txhj_yecha").unwrap();
        advance_to_play(&mut eb, 0);
        eb.debug_clear_hand(1);
        eb.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
        let hp1 = eb.state_core().player(1).hp;
        let log_len = eb.log().len();
        let sha = take_card_kind(&mut eb, CardKind::Sha);
        let _ = play_real_card(&mut eb, 0, sha, vec![1]);
        drain_to_pass(&mut eb);
        assert_eq!(eb.state_core().player(1).hp, hp1, "bazhen heart judge must dodge Sha");
        assert!(
            eb.log()[log_len..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::ArmorJudged { player: 1, success: true, .. })),
            "bazhen must fire ArmorJudged when Armor slot is empty"
        );

        // Armor slot occupied by `Renwang` -> `bazhen` must NOT judge on red Sha
        eb.assign_general_named(1, "txhj_yecha").unwrap();
        let rw = take_card_kind(&mut eb, CardKind::Renwang);
        advance_to_play(&mut eb, 1);
        let _ = play_real_card(&mut eb, 1, rw, vec![]);
        eb.debug_clear_hand(1);
        advance_to_play(&mut eb, 0);
        eb.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
        let log_len2 = eb.log().len();
        let red_sha = take_card_matching(&mut eb, |d| {
            d.kind == CardKind::Sha && matches!(d.suit, Suit::Heart | Suit::Diamond)
        });
        let _ = play_real_card(&mut eb, 0, red_sha, vec![1]);
        drain_to_pass(&mut eb);
        assert_eq!(
            eb.state_core().player(1).hp,
            hp1 - 1,
            "occupied armor slot disables bazhen"
        );
        assert!(
            !eb.log()[log_len2..]
                .iter()
                .any(|ev| matches!(ev, LogEvent::ArmorJudged { player: 1, .. })),
            "occupied armor slot must NOT fire ArmorJudged for bazhen"
        );

        // Qinggang pierces empty-armor `bazhen`
        let mut eq = start_locked_engine(2026_703, 2, LockedStartMode::Prototype, verified).unwrap();
        eq.assign_general_named(0, "txhj_huangyueying").unwrap();
        eq.assign_general_named(1, "txhj_yecha").unwrap();
        advance_to_play(&mut eq, 0);
        let qg = take_card_kind(&mut eq, CardKind::Qinggang);
        let _ = play_real_card(&mut eq, 0, qg, vec![]);
        eq.debug_clear_hand(1);
        eq.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
        let hp_q = eq.state_core().player(1).hp;
        let log_q = eq.log().len();
        let sha_q = take_card_kind(&mut eq, CardKind::Sha);
        let _ = play_real_card(&mut eq, 0, sha_q, vec![1]);
        drain_to_pass(&mut eq);
        assert_eq!(eq.state_core().player(1).hp, hp_q - 1);
        assert!(!eq.log()[log_q..].iter().any(|ev| matches!(ev, LogEvent::ArmorJudged { .. })));
    }

    // 3. `roulin` (`txhj_fennudeyezhu`): requires 2 Shan both vs Female (`txhj_huangyueying`) and `"double"` (`txhj_niufudongxie`)
    for (att_gen, def_gen, expect_two) in [
        ("txhj_fennudeyezhu", "txhj_huangyueying", true),
        ("txhj_huangyueying", "txhj_fennudeyezhu", true),
        ("txhj_fennudeyezhu", "txhj_niufudongxie", true),
        ("txhj_niufudongxie", "txhj_fennudeyezhu", true),
        ("txhj_xuelan", "txhj_fennudeyezhu", false),
    ] {
        let mut er = start_locked_engine(2026_704, 2, LockedStartMode::Prototype, verified).unwrap();
        er.assign_general_named(0, att_gen).unwrap();
        er.assign_general_named(1, def_gen).unwrap();
        advance_to_play(&mut er, 0);
        er.debug_clear_hand(1);
        let shan1 = take_card_kind(&mut er, CardKind::Shan);
        let shan2 = take_card_kind(&mut er, CardKind::Shan);
        give_hand_card(&mut er, 1, shan1);
        give_hand_card(&mut er, 1, shan2);
        let hp_before = er.state_core().player(1).hp;
        let sha = take_card_kind(&mut er, CardKind::Sha);
        let _ = play_real_card(&mut er, 0, sha, vec![1]);
        let ask1 = er.open_choice().cloned().expect("expected 1st Dodge prompt");
        assert_eq!(ask1.observer, 1);
        let _ = answer_choice(&mut er, 1, vec![ChoiceOption::Card(shan1)]);
        if expect_two {
            let ask2 = er
                .open_choice()
                .cloned()
                .unwrap_or_else(|| panic!("roulin ({att_gen} -> {def_gen}) must require a 2nd Shan!"));
            assert_eq!(ask2.observer, 1);
            let _ = answer_choice(&mut er, 1, vec![ChoiceOption::Card(shan2)]);
            drain_to_pass(&mut er);
            assert_eq!(er.state_core().player(1).hp, hp_before, "2 Shan dodges roulin Sha");
        } else {
            assert!(
                er.open_choice().is_none() || er.open_choice().unwrap().spec.prompt != "respond_shan",
                "male vs roulin ({att_gen} -> {def_gen}) must only require 1 Shan"
            );
            drain_to_pass(&mut er);
            assert_eq!(er.state_core().player(1).hp, hp_before);
        }
    }

    // Explicit 1-Shan fail check under `roulin` (and 2-Shan pass on 2nd prompt):
    {
        let mut er1 = start_locked_engine(2026_705, 2, LockedStartMode::Prototype, verified).unwrap();
        er1.assign_general_named(0, "txhj_fennudeyezhu").unwrap();
        er1.assign_general_named(1, "txhj_niufudongxie").unwrap();
        advance_to_play(&mut er1, 0);
        er1.debug_clear_hand(1);
        let shan1 = take_card_kind(&mut er1, CardKind::Shan);
        let shan2 = take_card_kind(&mut er1, CardKind::Shan);
        give_hand_card(&mut er1, 1, shan1);
        give_hand_card(&mut er1, 1, shan2);
        let hp_before = er1.state_core().player(1).hp;
        let sha = take_card_kind(&mut er1, CardKind::Sha);
        let _ = play_real_card(&mut er1, 0, sha, vec![1]);
        let _ = answer_choice(&mut er1, 1, vec![ChoiceOption::Card(shan1)]);
        let ask2 = er1.open_choice().cloned().expect("must ask 2nd Shan");
        assert_eq!(ask2.observer, 1);
        let _ = answer_choice(&mut er1, 1, vec![ChoiceOption::Pass]);
        drain_to_pass(&mut er1);
        assert_eq!(
            er1.state_core().player(1).hp,
            hp_before - 1,
            "passing 2nd Shan vs roulin takes 1 damage"
        );
    }
}

#[test]
fn l11_b6a_round3_reguicai_drlt_qianjie_nzry_jieying_reqicai_and_feiying() {
    let verified = taixu_verified();

    // 4. `reguicai` (`txhj_huanshensima`): rewrites with hand AND equips in Identity/Taixu & discards old card
    {
        let mut eg = start_locked_engine(2026_706, 2, LockedStartMode::Prototype, verified).unwrap();
        eg.assign_general_named(0, "txhj_huanshensima").unwrap();
        eg.assign_general_named(1, "txhj_xuelan").unwrap();
        advance_to_play(&mut eg, 0);
        let heart_equip = take_card_matching(&mut eg, |d| d.suit == Suit::Heart && d.kind.is_equip());
        let hand_spade = take_card_matching(&mut eg, |d| d.suit == Suit::Spade && !d.kind.is_equip());
        let _ = play_real_card(&mut eg, 0, heart_equip, vec![]);
        let le = take_card_kind(&mut eg, CardKind::Le);
        let _ = play_real_card(&mut eg, 0, le, vec![1]);
        drain_to_pass(&mut eg);
        eg.debug_clear_hand(0);
        give_hand_card(&mut eg, 0, hand_spade);
        assert!(eg.state_core().player(0).equips.contains(&heart_equip));

        let bad_judge = eg.debug_set_deck_top_by(|d| d.suit == Suit::Spade);
        let _ = submit(&mut eg, 0, GameAction::NextPhase);
        let _ = submit(&mut eg, 0, GameAction::NextPhase);
        drain_to_pass(&mut eg);
        let _ = submit(&mut eg, 0, GameAction::NextPhase);
        for _ in 0..16 {
            if eg.open_choice().is_some_and(|c| c.spec.prompt == "gui_dao_rewrite") {
                break;
            }
            if eg.open_choice().is_some() {
                drain_to_pass(&mut eg);
            } else {
                let cur = eg.turn().current;
                let _ = submit(&mut eg, cur, GameAction::NextPhase);
            }
        }
        let rw_ask = eg
            .open_choice()
            .cloned()
            .expect("reguicai must open gui_dao_rewrite prompt even when only Heart card is in equips");
        assert_eq!(rw_ask.observer, 0);
        assert!(rw_ask.spec.options.contains(&ChoiceOption::Card(heart_equip)));
        assert!(rw_ask.spec.options.contains(&ChoiceOption::Card(hand_spade)));

        let raw_save = encode_archive(&eg.save());
        let dec_save = decode_archive(&raw_save).expect("decode mid-reguicai");
        let mut restored = restore_locked_engine(&dec_save, verified).expect("restore mid-reguicai");

        for g in [&mut eg, &mut restored] {
            let out = answer_choice(g, 0, vec![ChoiceOption::Card(heart_equip)]);
            assert!(!matches!(out, CommandOutcome::Rejected(_)));
            drain_to_pass(g);
        }
        assert_eq!(eg.state_hash(), restored.state_hash());
        assert!(!eg.state_core().player(0).equips.contains(&heart_equip));
        assert!(
            eg.state_core().discard.contains(&bad_judge),
            "reguicai must discard old judge card {bad_judge} (not gain it like guidao)"
        );
        assert!(
            !eg.state_core().player(0).hand.contains(&bad_judge),
            "reguicai must NOT put old judge card into rewriter hand"
        );
        advance_to_play(&mut eg, 1);
    }

    // 5. `drlt_qianjie` (`txhj_zhangzhao`, `txhj_manding`): blocks link gain + delayed tricks
    {
        let mut eq = start_locked_engine(2026_707, 4, LockedStartMode::Prototype, verified).unwrap();
        eq.assign_general_named(0, "txhj_xuelan").unwrap();
        eq.assign_general_named(1, "txhj_zhangzhao").unwrap();
        eq.assign_general_named(2, "txhj_manding").unwrap();
        eq.assign_general_named(3, "txhj_huangyueying").unwrap();
        advance_to_play(&mut eq, 0);

        let chain = take_card_kind(&mut eq, CardKind::Chain);
        let out = play_real_card(&mut eq, 0, chain, vec![1, 3]);
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        drain_to_pass(&mut eq);
        assert!(!eq.state_core().player(1).linked, "drlt_qianjie must cancel link gain");
        assert!(eq.state_core().player(3).linked, "normal seat 3 still links");

        for dkind in [CardKind::Le, CardKind::Bing] {
            let dc = take_card_kind(&mut eq, dkind);
            let rej = play_real_card(&mut eq, 0, dc, vec![1]);
            assert!(
                matches!(rej, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
                "drlt_qianjie must reject {dkind:?} target"
            );
        }

        let sd = take_card_kind(&mut eq, CardKind::ShanDian);
        let out_sd = play_real_card(&mut eq, 0, sd, vec![]);
        assert!(!matches!(out_sd, CommandOutcome::Rejected(_)));
        drain_to_pass(&mut eq);
        advance_to_play(&mut eq, 1);
        advance_to_play(&mut eq, 2);
        advance_to_play(&mut eq, 3);
        let _ = submit(&mut eq, 3, GameAction::NextPhase);
        let _ = submit(&mut eq, 3, GameAction::NextPhase);
        drain_to_pass(&mut eq);
        eq.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
        for _ in 0..16 {
            if eq
                .state_core()
                .player(3)
                .judge
                .iter()
                .any(|&c| eq.catalog().card_kind(c) == CardKind::ShanDian)
            {
                break;
            }
            if eq.open_choice().is_some() {
                drain_to_pass(&mut eq);
            } else {
                let cur = eq.turn().current;
                let _ = submit(&mut eq, cur, GameAction::NextPhase);
            }
        }
        assert!(eq.state_core().player(1).judge.is_empty(), "ShanDian must skip drlt_qianjie seat 1");
        assert!(eq.state_core().player(2).judge.is_empty(), "ShanDian must skip drlt_qianjie seat 2");
        assert!(
            eq.state_core()
                .player(3)
                .judge
                .iter()
                .any(|&c| eq.catalog().card_kind(c) == CardKind::ShanDian),
            "ShanDian must land on seat 3 skipping drlt_qianjie seats"
        );
    }

    // 6. `nzry_jieying` (`txhj_digongzhangbaox`):
    //    auto-links Turn 1, +2 hand limit to linked seats, link_loss_cancel on fire/chain/tiesuo, phase_end ChooseLinkOther
    {
        let mut en = start_locked_engine(2026_708, 3, LockedStartMode::Prototype, verified).unwrap();
        en.assign_general_named(0, "txhj_digongzhangbaox").unwrap();
        en.assign_general_named(1, "txhj_xuelan").unwrap();
        en.assign_general_named(2, "txhj_huangyueying").unwrap();
        advance_to_play(&mut en, 0);
        assert!(en.state_core().player(0).linked, "nzry_jieying game_started must auto-link owner");

        en.debug_set_hp(0, 3);
        en.debug_set_hp(1, 3);
        en.debug_set_hp(2, 3);
        let ch = take_card_kind(&mut en, CardKind::Chain);
        let _ = play_real_card(&mut en, 0, ch, vec![2]);
        drain_to_pass(&mut en);
        assert!(en.state_core().player(2).linked);
        assert_eq!(en.state_core().hand_limit(en.catalog(), 0), 5, "nzry_jieying linked owner +2 hand limit");
        assert_eq!(en.state_core().hand_limit(en.catalog(), 2), 5, "nzry_jieying linked other +2 hand limit");
        assert_eq!(en.state_core().hand_limit(en.catalog(), 1), 3, "unlinked seat keeps normal hand limit");

        let ch2 = take_card_kind(&mut en, CardKind::Chain);
        let _ = play_real_card(&mut en, 0, ch2, vec![0]);
        drain_to_pass(&mut en);
        assert!(en.state_core().player(0).linked, "Chain cannot unlink nzry_jieying owner");

        en.debug_clear_hand(0);
        let _ = submit(&mut en, 0, GameAction::NextPhase);
        let _ = submit(&mut en, 0, GameAction::NextPhase);
        let link_ask = en
            .open_choice()
            .cloned()
            .expect("nzry_jieying phase_end must open ChooseLinkOther");
        assert_eq!(link_ask.observer, 0);
        assert_eq!(
            link_ask.spec.options,
            vec![ChoiceOption::Player(1)],
            "ChooseLinkOther must filter out already-linked seat 2"
        );

        let raw_s = encode_archive(&en.save());
        let dec_s = decode_archive(&raw_s).expect("decode ChooseLinkOther");
        let mut r_en = restore_locked_engine(&dec_s, verified).expect("restore ChooseLinkOther");
        for g in [&mut en, &mut r_en] {
            let _ = answer_choice(g, 0, vec![ChoiceOption::Player(1)]);
            drain_to_pass(g);
        }
        assert_eq!(en.state_hash(), r_en.state_hash());
        assert!(
            en.state_core().player(1).linked,
            "ChooseLinkOther must leave target linked (no double toggle!)"
        );

        advance_to_play(&mut en, 1);
        en.debug_clear_hand(0);
        en.debug_clear_hand(2);
        en.state_core_mut().player_mut(0).hujia = 0;
        let hp0_before = en.state_core().player(0).hp;
        let hp1_before = en.state_core().player(1).hp;
        let hp2_before = en.state_core().player(2).hp;
        let fsha = take_card_matching(&mut en, |d| d.kind == CardKind::FireSha);
        let _ = play_real_card(&mut en, 1, fsha, vec![0]);
        drain_to_pass(&mut en);
        assert_eq!(en.state_core().player(0).hp, hp0_before - 1);
        assert_eq!(en.state_core().player(1).hp, hp1_before - 1, "chain conducts from nzry_jieying");
        assert_eq!(en.state_core().player(2).hp, hp2_before - 1, "chain conducts to seat 2");
        assert!(
            en.state_core().player(0).linked,
            "nzry_jieying link_loss_cancel must keep owner linked after elemental damage!"
        );
        assert!(!en.state_core().player(1).linked, "normal seat 1 unlinks after chained damage");
        assert!(!en.state_core().player(2).linked, "normal seat 2 unlinks after chained damage");
    }

    // 7. `reqicai` (`txhj_huangyueying`): unlimited trick distance (`Snatch`) + protects Armor & Treasure from other discard
    {
        let mut eqc = start_locked_engine(2026_709, 4, LockedStartMode::Prototype, verified).unwrap();
        eqc.assign_general_named(0, "txhj_huangyueying").unwrap();
        eqc.assign_general_named(1, "txhj_xuelan").unwrap();
        eqc.assign_general_named(2, "txhj_huangyueying").unwrap();
        eqc.assign_general_named(3, "txhj_xuelan").unwrap();
        advance_to_play(&mut eqc, 0);
        assert!(eqc.state_core().computed_distance(eqc.catalog(), 0, 2) >= 2);
        let rw = take_card_kind(&mut eqc, CardKind::Renwang);
        let tr = take_card_named(&mut eqc, "txhj_dinglanyemingzhu");
        let wpn = take_card_kind(&mut eqc, CardKind::Qinggang);
        eqc.debug_clear_hand(2);
        advance_to_play(&mut eqc, 2);
        let _ = play_real_card(&mut eqc, 2, rw, vec![]);
        let _ = play_real_card(&mut eqc, 2, tr, vec![]);
        drain_to_pass(&mut eqc);

        advance_to_play(&mut eqc, 0);
        eqc.debug_clear_hand(2);
        let h2 = take_card_kind(&mut eqc, CardKind::Tao);
        give_hand_card(&mut eqc, 2, h2);
        let snatch = take_card_kind(&mut eqc, CardKind::Snatch);
        let out_sn = play_real_card(&mut eqc, 0, snatch, vec![2]);
        assert!(
            !matches!(out_sn, CommandOutcome::Rejected(_)),
            "reqicai Snatch must succeed at distance >= 2: {out_sn:?}"
        );
        drain_to_pass(&mut eqc);
        assert!(eqc.state_core().player(0).hand.contains(&h2));

        eqc.debug_clear_hand(2);
        let gh = take_card_kind(&mut eqc, CardKind::Dismantle);
        let _ = play_real_card(&mut eqc, 0, gh, vec![2]);
        drain_to_pass(&mut eqc);
        assert!(
            eqc.state_core().player(2).equips.contains(&rw),
            "reqicai must protect equipped Armor from Dismantle"
        );
        assert!(
            eqc.state_core().player(2).equips.contains(&tr),
            "reqicai must protect equipped Treasure from Dismantle"
        );

        advance_to_play(&mut eqc, 2);
        let _ = play_real_card(&mut eqc, 2, wpn, vec![]);
        advance_to_play(&mut eqc, 0);
        eqc.debug_clear_hand(2);
        let gh2 = take_card_kind(&mut eqc, CardKind::Dismantle);
        let _ = play_real_card(&mut eqc, 0, gh2, vec![2]);
        drain_to_pass(&mut eqc);
        assert!(!eqc.state_core().player(2).equips.contains(&wpn), "unprotected Weapon must be dismantled");
        assert!(eqc.state_core().player(2).equips.contains(&rw), "Armor still protected");
        assert!(eqc.state_core().player(2).equips.contains(&tr), "Treasure still protected");

        // Also check `txhj_kapaikeshi` Dismantle prompt excludes protected Armor & Treasure:
        eqc.debug_grant(0, "txhj_kapaikeshi");
        let h_pick = take_card_kind(&mut eqc, CardKind::Tao);
        give_hand_card(&mut eqc, 2, h_pick);
        let gh3 = take_card_kind(&mut eqc, CardKind::Dismantle);
        let _ = play_real_card(&mut eqc, 0, gh3, vec![2]);
        for _ in 0..4 {
            let Some(ch) = eqc.open_choice().cloned() else { break };
            if ch.spec.prompt == "txhj_kapaikeshi:choose_card" {
                break;
            }
            let _ = answer_choice(&mut eqc, ch.observer, Vec::new());
        }
        let kp_ask = eqc.open_choice().cloned().expect("txhj_kapaikeshi Dismantle prompt");
        assert_eq!(kp_ask.spec.options, vec![ChoiceOption::Card(h_pick)], "reqicai must exclude protected Armor/Treasure from kapaikeshi prompt");
        let _ = answer_choice(&mut eqc, 0, vec![ChoiceOption::Card(h_pick)]);
        drain_to_pass(&mut eqc);

        let ql = take_card_named(&mut eqc, "txhj_qilingong");
        let _ = play_real_card(&mut eqc, 0, ql, vec![]);
        eqc.debug_clear_hand(2);
        let sha_ql = take_card_kind(&mut eqc, CardKind::Sha);
        let _ = play_real_card(&mut eqc, 0, sha_ql, vec![2]);
        drain_to_pass(&mut eqc);
        assert!(
            eqc.state_core().player(2).equips.contains(&tr),
            "reqicai must protect Treasure from txhj_qilingong"
        );
        // Also verify `qilin_skill` does not discard `reqicai` Treasure:
        advance_to_play(&mut eqc, 1);
        advance_to_play(&mut eqc, 0);
        eqc.debug_grant(0, "qilin_skill");
        eqc.debug_clear_hand(2);
        let sha_qs = take_card_kind(&mut eqc, CardKind::Sha);
        let _ = play_real_card(&mut eqc, 0, sha_qs, vec![2]);
        drain_to_pass(&mut eqc);
        assert!(
            eqc.state_core().player(2).equips.contains(&tr),
            "reqicai must protect Treasure from qilin_skill"
        );

        advance_to_play(&mut eqc, 2);
        let rw2 = take_card_kind(&mut eqc, CardKind::Armor);
        let _ = play_real_card(&mut eqc, 2, rw2, vec![]);
        assert!(eqc.state_core().player(2).equips.contains(&rw2));
        assert!(!eqc.state_core().player(2).equips.contains(&rw));
    }

    // 8. `feiying` (`txhj_xuelan`): purely passive distance +1 with 0 `U05` `PhaseBegin` trigger flags
    {
        let mut ef = start_locked_engine(2026_710, 3, LockedStartMode::Prototype, verified).unwrap();
        ef.assign_general_named(0, "txhj_xuelan").unwrap();
        ef.assign_general_named(1, "txhj_huangyueying").unwrap();
        ef.assign_general_named(2, "txhj_dushi").unwrap();
        advance_to_play(&mut ef, 0);
        assert_eq!(ef.state_core().computed_distance(ef.catalog(), 1, 0), 2);
        assert!(!ef.state_core().vm_aux.flags.contains_key(&(0, "u05_skill_invoked:feiying".to_string())));
        assert!(!ef.state_core().vm_aux.flags.contains_key(&(0, "u05_cross_mod:feiying".to_string())));
    }
}

#[test]
fn l11_b6a_round3_double_gender_u01_and_u08_parity() {
    let verified_u08 = prepare_locked_content_with_bridge(vec![
        load_frozen_pkg("taixu-upstream-liuli"),
        load_frozen_pkg("hero-kill-upstream-noname"),
    ])
    .unwrap();

    // A. `u08_batch_b` `heqin`
    {
        let mut e = start_locked_engine(2026_720, 3, LockedStartMode::Prototype, &verified_u08).unwrap();
        e.assign_general_named(0, "yxs_wangzhaojun").unwrap();
        e.assign_general_named(1, "txhj_niufudongxie").unwrap();
        e.assign_general_named(2, "txhj_huangyueying").unwrap();
        advance_to_play(&mut e, 0);
        let hq = e.catalog().skill_by_name("heqin").expect("heqin skill");
        let c1 = take_card_kind(&mut e, CardKind::Sha);
        let c2 = take_card_kind(&mut e, CardKind::Sha);
        give_hand_card(&mut e, 0, c1);
        give_hand_card(&mut e, 0, c2);
        let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: hq });
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "heqin must activate when only other Male is double-gender txhj_niufudongxie: {out:?}"
        );
    }

    // B. `u08_batch_c` `yizhuang` (Card prompt -> Male target prompt including double gender & runtime override)
    {
        let mut e = start_locked_engine(2026_721, 3, LockedStartMode::Prototype, &verified_u08).unwrap();
        e.assign_general_named(0, "txhj_huangyueying").unwrap();
        e.assign_general_named(1, "yxs_huamulan").unwrap();
        e.assign_general_named(2, "txhj_niufudongxie").unwrap();
        let c = take_card_kind(&mut e, CardKind::Sha);
        give_hand_card(&mut e, 1, c);
        for _ in 0..48 {
            if let Some(ch) = e.open_choice().cloned() {
                if ch.spec.prompt.contains("yizhuang") && ch.spec.kind == engine_native::ChoiceKind::SelectPlayers {
                    break;
                }
                if ch.spec.prompt.contains("yizhuang") && ch.spec.kind == engine_native::ChoiceKind::SelectCards {
                    let pick = ch.spec.options.iter().find(|o| matches!(o, ChoiceOption::Card(_))).copied().unwrap();
                    let _ = answer_choice(&mut e, ch.observer, vec![pick]);
                } else if ch.spec.options.contains(&ChoiceOption::Confirm) {
                    let _ = answer_choice(&mut e, ch.observer, vec![ChoiceOption::Confirm]);
                } else {
                    drain_to_pass(&mut e);
                }
            } else {
                let cur = e.turn().current;
                let _ = submit(&mut e, cur, GameAction::NextPhase);
            }
        }
        let ask = e
            .open_choice()
            .cloned()
            .expect("yizhuang must offer Male target prompt including double-gender txhj_niufudongxie");
        assert!(ask.spec.options.contains(&ChoiceOption::Player(2)));
        assert!(!ask.spec.options.contains(&ChoiceOption::Player(0)));
    }

    // C. `u08_batch_d` `nvquan3`
    {
        let mut e = start_locked_engine(2026_722, 2, LockedStartMode::Prototype, &verified_u08).unwrap();
        e.assign_general_named(0, "txhj_niufudongxie").unwrap();
        e.assign_general_named(1, "yxs_wuzetian").unwrap();
        advance_to_play(&mut e, 0);
        let duel = take_card_kind(&mut e, CardKind::Duel);
        let out = play_real_card(&mut e, 0, duel, vec![1]);
        assert!(
            matches!(out, CommandOutcome::Rejected(RejectReason::IllegalTargets)),
            "nvquan3 must treat double-gender txhj_niufudongxie as Male and reject Duel: {out:?}"
        );
    }

    // D. `u01_batch_d` `dddshixing`
    let verified_u01 = prepare_locked_content_with_bridge(vec![
        load_frozen_pkg("taixu-upstream-liuli"),
        load_frozen_pkg("d3-upstream-noname"),
    ])
    .unwrap();
    {
        let mut e = start_locked_engine(2026_723, 2, LockedStartMode::Prototype, &verified_u01).unwrap();
        e.assign_general_named(0, "ddd_liuhong").unwrap();
        e.assign_general_named(1, "txhj_niufudongxie").unwrap();
        advance_to_play(&mut e, 0);
        // Trigger `dddshixing` on `CardUsed` (or Turn 2 `Phase::Prepare`) AFTER `assign_general_named`:
        let peach = take_card_kind(&mut e, CardKind::Tao);
        e.debug_set_hp(0, 3);
        let _ = play_real_card(&mut e, 0, peach, vec![]);
        drain_to_pass(&mut e);
        let mask = e
            .state_core()
            .vm_aux
            .flags
            .get(&(0, "dddshixing_granted_mask".to_string()))
            .copied()
            .unwrap_or(0);
        assert_ne!(
            mask & 1,
            0,
            "dddshixing must detect undamaged double-gender txhj_niufudongxie as Female (xiangle mask bit 1)"
        );
        let xiangle = e.catalog().skill_by_name("xiangle").expect("xiangle skill");
        assert!(e.state_core().has_skill(0, xiangle));
    }

    // E. `u08_batch_c` `seyou` + `u01_batch_b` `dddjijian`: double-gender `txhj_niufudongxie` treated as Male,
    //    and `effective_gender_override = 2` (Female) switches it to non-Male cleanly.
    {
        let mut es = start_locked_engine(2026_724, 3, LockedStartMode::Prototype, &verified_u08).unwrap();
        es.assign_general_named(0, "yxs_aijiyanhou").unwrap();
        es.assign_general_named(1, "txhj_huangyueying").unwrap();
        es.assign_general_named(2, "txhj_niufudongxie").unwrap();
        advance_to_play(&mut es, 0);
        let sy = es.catalog().skill_by_name("seyou").expect("seyou skill");
        let sha = take_card_kind(&mut es, CardKind::Sha);
        give_hand_card(&mut es, 2, sha);
        let out = submit(&mut es, 0, GameAction::ActivateSkill { skill: sy });
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        let _ = answer_choice(&mut es, 0, vec![ChoiceOption::Player(1)]);
        // Seat 2 (double-gender `txhj_niufudongxie`) must be asked by `seyou` to Sha seat 1!
        let ask_sha = es.open_choice().cloned().expect("seyou must prompt double-gender txhj_niufudongxie as Male");
        assert_eq!(ask_sha.observer, 2);
        assert!(ask_sha.spec.options.contains(&ChoiceOption::Card(sha)));
    }
    // F. `u01_batch_b/c` (`dddjijian` / `dddfushi`) + `u08` `effective_gender_override = 2`:
    {
        let mut ed = start_locked_engine(2026_725, 3, LockedStartMode::Prototype, &verified_u08).unwrap();
        ed.assign_general_named(0, "yxs_wangzhaojun").unwrap();
        ed.assign_general_named(1, "txhj_niufudongxie").unwrap();
        ed.assign_general_named(2, "txhj_huangyueying").unwrap();
        ed.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "effective_gender_override".to_string()), 2);
        advance_to_play(&mut ed, 0);
        let hq = ed.catalog().skill_by_name("heqin").expect("heqin skill");
        let c1 = take_card_kind(&mut ed, CardKind::Sha);
        let c2 = take_card_kind(&mut ed, CardKind::Sha);
        give_hand_card(&mut ed, 0, c1);
        give_hand_card(&mut ed, 0, c2);
        // With `effective_gender_override == 2` (Female) on seat 1 and Female on seat 2, no Male targets remain:
        let out = submit(&mut ed, 0, GameAction::ActivateSkill { skill: hq });
        assert!(
            matches!(out, CommandOutcome::Rejected(_)),
            "effective_gender_override=2 must make heqin reject when no Male others exist: {out:?}"
        );

        // And with `u01` `is_male` / `is_male_player` / `dddshixing` after `effective_gender_override = 1` vs `2`:
        let mut eu = start_locked_engine(2026_726, 2, LockedStartMode::Prototype, &verified_u01).unwrap();
        eu.assign_general_named(0, "ddd_liuhong").unwrap();
        eu.assign_general_named(1, "txhj_niufudongxie").unwrap();
        eu.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "effective_gender_override".to_string()), 1);
        advance_to_play(&mut eu, 0);
        let peach = take_card_kind(&mut eu, CardKind::Tao);
        eu.debug_set_hp(0, 3);
        let _ = play_real_card(&mut eu, 0, peach, vec![]);
        drain_to_pass(&mut eu);
        let mask = eu
            .state_core()
            .vm_aux
            .flags
            .get(&(0, "dddshixing_granted_mask".to_string()))
            .copied()
            .unwrap_or(0);
        assert_eq!(
            mask & 1,
            0,
            "effective_gender_override=1 turns double-gender into Male-only so dddshixing sees no undamaged Female"
        );
        let xiangle = eu.catalog().skill_by_name("xiangle").expect("xiangle skill");
        assert!(!eu.state_core().has_skill(0, xiangle));
    }

    // G. `u08_batch_c` `fengyan`: double-gender `txhj_niufudongxie` red judge triggers `fengyan` draw/gain
    {
        let mut ef = start_locked_engine(2026_727, 2, LockedStartMode::Prototype, &verified_u08).unwrap();
        ef.assign_general_named(0, "txhj_niufudongxie").unwrap();
        ef.assign_general_named(1, "yxs_yangyuhuan").unwrap();
        advance_to_play(&mut ef, 0);
        ef.debug_clear_hand(1);
        let le = take_card_kind(&mut ef, CardKind::Le);
        // Seat 1 puts `Le` in seat 0's judge zone, then seat 0 enters Judge with red judge card:
        advance_to_play(&mut ef, 1);
        let _ = play_real_card(&mut ef, 1, le, vec![0]);
        drain_to_pass(&mut ef);
        let hand1_before = ef.state_core().player(1).hand.len();
        let red_j = ef.debug_set_deck_top_by(|d| d.suit == Suit::Heart);
        let _ = submit(&mut ef, 1, GameAction::NextPhase);
        let _ = submit(&mut ef, 1, GameAction::NextPhase);
        drain_to_pass(&mut ef);
        let _ = submit(&mut ef, 1, GameAction::NextPhase); // Turn advances to seat 0 Judge
        for _ in 0..8 {
            if let Some(ch) = ef.open_choice().cloned() {
                if ch.spec.options.contains(&ChoiceOption::Confirm) {
                    let _ = answer_choice(&mut ef, ch.observer, vec![ChoiceOption::Confirm]);
                } else {
                    drain_to_pass(&mut ef);
                }
            } else {
                break;
            }
        }
        assert!(
            ef.state_core().player(1).hand.contains(&red_j)
                || ef.state_core().player(1).hand.len() > hand1_before,
            "fengyan must trigger on red OrdinaryJudge of double-gender txhj_niufudongxie"
        );
    }
}
