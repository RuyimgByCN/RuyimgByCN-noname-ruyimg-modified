//! U05-6b Round 2 Regression Suite: P0 & P1 Fixes (`qianghua`, `huanjue`, `oldyanshu`/`yanshu`,
//! `oldyulu`/`yulu`, `hanshuang`, `xiaorong`, `yuanzheng`).
//!
//! All tests use real formal `shahai-hearth` engine event paths (ZERO `debug_execute_effect`),
//! positive + negative + cancel + disposition + conservation + `BoundedPlanner` AI + `NGSA 107` save/restore.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::choice::{ChoiceKind, ChoiceOption};
use engine_native::event::Phase;
use engine_native::machine::LogEvent;

#[test]
fn u05_b6b_round2_p0_qianghua_wuzhong_virtual_and_named_tricks() {
    // 1A) `qianghua` (`character/hearth/skill.js:4931-4995`): Wuzhong in Play phase (`targets=[]`),
    // replayed copy enters `table -> discard` (never stuck in `hand`), 4 cards drawn total (`hand.len() == 4`), 1/turn.
    let mut ew = fresh_proto(20_701, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ew, 0, "qianghua");
    advance_to_phase(&mut ew, 0, Phase::Play);
    clear_hand(&mut ew, 0);
    clear_hand(&mut ew, 1);
    let before_w = total_tracked_cards(&ew);
    let wz = give_free_card_kind_suit(&mut ew, 0, CardKind::Wuzhong, None);
    play_card(&mut ew, 0, wz, vec![]);
    assert_eq!(
        ew.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm")
    );
    save_restore_and_answer(&mut ew, "hs_qianghua_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut ew);
    assert_eq!(
        ew.state_core().player(0).hand.len(),
        4,
        "original Wuzhong (+2) + qianghua copy (+2) = 4 cards, copy NOT stuck in hand"
    );
    assert!(!ew.state_core().player(0).hand.contains(&wz));
    assert!(ew.state_core().discard.contains(&wz));
    assert!(ew.state_core().table.is_empty());
    assert_eq!(flag(&ew, 0, "hs_b6b_cnt:qianghua"), 1);
    assert_eq!(total_tracked_cards(&ew), before_w);

    // Subsequent Sha in same Play phase does NOT re-trigger `qianghua` (1/turn limit)
    let hp1_w = ew.state_core().player(1).hp;
    clear_hand(&mut ew, 1);
    let sha_after = give_free_card_kind_suit(&mut ew, 0, CardKind::Sha, None);
    play_card(&mut ew, 0, sha_after, vec![1]);
    assert!(ew.open_choice().is_none(), "qianghua already used this turn");
    assert_eq!(ew.state_core().player(1).hp, hp1_w - 1);

    // 1B) `qianghua` virtual fallback (`card: None`): remove ALL `hs_wuzhong_*` and `hs_*sha*` from `detached_cards`
    let mut ev = fresh_proto(20_702, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ev, 0, "qianghua");
    advance_to_phase(&mut ev, 0, Phase::Play);
    clear_hand(&mut ev, 0);
    clear_hand(&mut ev, 1);
    let wz_detached: Vec<u32> = ev
        .state_core()
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| (ev.catalog().card_kind(c) == CardKind::Wuzhong).then_some(c))
        .collect();
    for c in wz_detached {
        ev.state_core_mut().detached_cards.retain(|&(x, _)| x != c);
        ev.state_core_mut().discard.push(c);
    }
    let before_v = total_tracked_cards(&ev);
    let wz_v = give_free_base_named(&mut ev, 0, "wuzhong");
    play_card(&mut ev, 0, wz_v, vec![]);
    assert_eq!(
        ev.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm")
    );
    save_restore_and_answer(&mut ev, "hs_qianghua_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut ev);
    assert_eq!(
        ev.state_core().player(0).hand.len(),
        4,
        "virtual Wuzhong fallback still resolves +2 -> 4 cards total; hand={:?}, logs={:?}",
        ev.state_core().player(0).hand,
        ev.log()
    );
    assert!(ev.state_core().discard.contains(&wz_v));
    assert_eq!(total_tracked_cards(&ev), before_v);

    // Virtual Sha fallback (`detached_cards` drained of all Sha):
    let mut evs = fresh_proto(20_703, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut evs, 0, "qianghua");
    advance_to_phase(&mut evs, 0, Phase::Play);
    clear_hand(&mut evs, 0);
    clear_hand(&mut evs, 1);
    let sha_det: Vec<u32> = evs
        .state_core()
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| evs.catalog().card_kind(c).is_sha_like().then_some(c))
        .collect();
    for c in sha_det {
        evs.state_core_mut().detached_cards.retain(|&(x, _)| x != c);
        evs.state_core_mut().discard.push(c);
    }
    let sha_v = give_free_card_kind_suit(&mut evs, 0, CardKind::Sha, None);
    let hp_v1 = evs.state_core().player(1).hp;
    play_card(&mut evs, 0, sha_v, vec![1]);
    save_restore_and_answer(&mut evs, "hs_qianghua_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut evs);
    assert_eq!(evs.state_core().player(1).hp, hp_v1 - 2);
    assert!(evs.state_core().player(0).hand.is_empty());
    assert!(evs.state_core().discard.contains(&sha_v));

    // 1C) `qianghua` named Hearth trick `shandianjian` (`CardKind::Wuzhong` in catalog):
    // must NOT be corrupted into plain `Wuzhong` draw-2 even when `detached_cards` has no extra `shandianjian`!
    let mut esd2 = fresh_proto(20_705, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut esd2, 0, "qianghua");
    advance_to_phase(&mut esd2, 0, Phase::Play);
    clear_hand(&mut esd2, 0);
    clear_hand(&mut esd2, 1);
    let sd_det2: Vec<u32> = esd2
        .state_core()
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| {
            esd2.catalog().cards[c as usize]
                .name
                .contains("shandianjian")
                .then_some(c)
        })
        .collect();
    for c in sd_det2 {
        esd2.state_core_mut().detached_cards.retain(|&(x, _)| x != c);
        esd2.state_core_mut().discard.push(c);
    }
    let sdj2 = give_card_named(&mut esd2, 0, "shandianjian");
    let h1 = give_free_card_kind_suit(&mut esd2, 0, CardKind::Sha, Some(Suit::Heart));
    let h2 = give_free_card_kind_suit(&mut esd2, 0, CardKind::Sha, Some(Suit::Heart));
    let _p1_card = give_free_card_kind_suit(&mut esd2, 1, CardKind::Shan, Some(Suit::Heart));
    let hp_sd2 = esd2.state_core().player(1).hp;
    let out_sd = play_card(&mut esd2, 0, sdj2, vec![1]);
    assert!(!matches!(out_sd, engine_native::CommandOutcome::Rejected(_)));
    assert_eq!(
        esd2.open_choice().map(|c| c.spec.prompt),
        Some("hs_shandianjian_discard")
    );
    answer(&mut esd2, 0, vec![ChoiceOption::Card(h1)]);
    assert_eq!(
        esd2.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm")
    );
    save_restore_and_answer(&mut esd2, "hs_qianghua_confirm", vec![ChoiceOption::Confirm]);
    assert_eq!(
        esd2.open_choice().map(|c| c.spec.prompt),
        Some("hs_shandianjian_discard"),
        "virtual shandianjian replay must invoke shandianjian (NOT draw-2 Wuzhong)"
    );
    answer(&mut esd2, 0, vec![ChoiceOption::Card(h2)]);
    drain_auto(&mut esd2);
    assert!(
        esd2.state_core().player(0).hand.is_empty(),
        "shandianjian must NOT fallback-draw 2 and both Heart discards resolved"
    );
    assert_eq!(
        esd2.state_core().player(1).hp,
        hp_sd2 - 2,
        "both shandianjian & virtual replay deal 1 Thunder dmg"
    );

    // 1D) `Tao` healed to full HP stops `qianghua` (`skill.js:4965`), while `shihuawuqi` at full HP replays!
    let mut et = fresh_proto(20_706, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut et, 0, "qianghua");
    advance_to_phase(&mut et, 0, Phase::Play);
    clear_hand(&mut et, 0);
    let max0 = et.state_core().player(0).max_hp as i32;
    et.state_core_mut().player_mut(0).hp = max0 - 1;
    let tao = give_free_card_kind_suit(&mut et, 0, CardKind::Tao, None);
    play_card(&mut et, 0, tao, vec![]);
    assert!(et.open_choice().is_none(), "full HP after Tao must not replay Tao");
    assert_eq!(et.state_core().player(0).hp, max0);

    let mut esw = fresh_proto(20_707, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut esw, 0, "qianghua");
    advance_to_phase(&mut esw, 0, Phase::Play);
    clear_hand(&mut esw, 0);
    let sw = give_card_named(&mut esw, 0, "shihuawuqi");
    play_card(&mut esw, 0, sw, vec![]);
    assert_eq!(
        esw.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm")
    );
    answer(&mut esw, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut esw);
    assert_eq!(
        flag(&esw, 0, "hs_shihuawuqi_used_turn"),
        2,
        "shihuawuqi replayed at full HP (+3 attack twice)"
    );

    // 1E) `wxuying` shadow `hsfashu_anyingjingxiang` copying `shandianjian` + `qianghua`:
    // shadow copy resolves as `shandianjian`, and `qianghua` preserves copied identity (`hs_wxuying_src_card`)!
    let mut ewx = fresh_proto(20_708, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ewx, 0, "qianghua");
    grant_host_skill(&mut ewx, 0, "wxuying");
    advance_to_phase(&mut ewx, 0, Phase::Play);
    clear_hand(&mut ewx, 0);
    clear_hand(&mut ewx, 1);
    let shadow_cid = give_card_named(&mut ewx, 0, "hsfashu_anyingjingxiang");
    let sdj_src = give_card_named(&mut ewx, 0, "shandianjian");
    let wh1 = give_free_card_kind_suit(&mut ewx, 0, CardKind::Sha, Some(Suit::Heart));
    let wh2 = give_free_card_kind_suit(&mut ewx, 0, CardKind::Sha, Some(Suit::Heart));
    let wh3 = give_free_card_kind_suit(&mut ewx, 0, CardKind::Sha, Some(Suit::Heart));
    let _p1_h = give_free_card_kind_suit(&mut ewx, 1, CardKind::Shan, Some(Suit::Heart));
    let hp_wx1 = ewx.state_core().player(1).hp;
    play_card(&mut ewx, 0, sdj_src, vec![1]);
    answer(&mut ewx, 0, vec![ChoiceOption::Card(wh1)]); // first shandianjian (-1)
    assert_eq!(
        ewx.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm")
    );
    answer(&mut ewx, 0, vec![]); // skip qianghua on first use so shadow copy can test it
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().player(1).hp, hp_wx1 - 1);
    let out_sh = play_card(&mut ewx, 0, shadow_cid, vec![1]);
    assert!(!matches!(out_sh, engine_native::CommandOutcome::Rejected(_)));
    answer(&mut ewx, 0, vec![ChoiceOption::Card(wh2)]); // shadow shandianjian (-1)
    assert_eq!(
        ewx.open_choice().map(|c| c.spec.prompt),
        Some("hs_qianghua_confirm")
    );
    save_restore_and_answer(&mut ewx, "hs_qianghua_confirm", vec![ChoiceOption::Confirm]);
    assert_eq!(
        ewx.open_choice().map(|c| c.spec.prompt),
        Some("hs_shandianjian_discard"),
        "qianghua on wxuying shadow must preserve copied shandianjian identity"
    );
    answer(&mut ewx, 0, vec![ChoiceOption::Card(wh3)]); // replayed shandianjian (-1)
    drain_auto(&mut ewx);
    assert_eq!(ewx.state_core().player(1).hp, hp_wx1 - 3);
}

#[test]
fn u05_b6b_round2_p0_huanjue_table_disposition_and_forced_equip_and_jiu() {
    // 2A) `huanjue` forced non-equip: `picked` enters `table -> discard` (NEVER stuck in `hand`!)
    let mut e = fresh_proto(20_711, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut e, 0, "huanjue");
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let _p1_keep = give_free_card_kind_suit(&mut e, 1, CardKind::Shan, None);
    let d_only = e
        .state_core()
        .detached_cards
        .iter()
        .find_map(|&(c, _)| (e.catalog().card_kind(c) == CardKind::Dismantle).then_some(c))
        .expect("hs_guohe in detached_cards");
    let moved_a: Vec<u32> = e
        .state_core()
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| (c != d_only).then_some(c))
        .collect();
    e.state_core_mut()
        .detached_cards
        .retain(|&(c, _)| c == d_only);
    e.state_core_mut().discard.extend(moved_a);
    let before = total_tracked_cards(&e);
    let sha = give_free_card_kind_suit(&mut e, 0, CardKind::Sha, None);
    play_card(&mut e, 0, sha, vec![1]);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hs_huanjue_pick")
    );
    save_restore_and_answer(&mut e, "hs_huanjue_pick", vec![ChoiceOption::Card(d_only)]);
    drain_auto(&mut e);
    assert!(e.state_core().player(0).hand.is_empty(), "picked Dismantle must NOT remain in hand");
    assert!(e.state_core().player(1).hand.is_empty(), "picked Dismantle resolved and discarded victim hand");
    assert!(e.state_core().discard.contains(&sha));
    assert!(e.state_core().discard.contains(&d_only));
    assert!(e.state_core().table.is_empty());
    assert_eq!(total_tracked_cards(&e), before);

    // 2B) `huanjue` forced Equip (`qinggang`): using self-single-target `Wuzhong` (`user == tgt`),
    // picking `hs_qinggang` installs directly into `player(0).equips` and keeps `hand` empty!
    let mut eq = fresh_proto(20_712, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut eq, 0, "huanjue");
    advance_to_phase(&mut eq, 0, Phase::Play);
    clear_hand(&mut eq, 0);
    let qg = eq
        .state_core()
        .detached_cards
        .iter()
        .find_map(|&(c, _)| eq.catalog().cards[c as usize].name.starts_with("hs_qinggang").then_some(c))
        .expect("hs_qinggang in detached_cards");
    let moved_q: Vec<u32> = eq
        .state_core()
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| (c != qg).then_some(c))
        .collect();
    eq.state_core_mut()
        .detached_cards
        .retain(|&(c, _)| c == qg);
    eq.state_core_mut().discard.extend(moved_q);
    let before_eq = total_tracked_cards(&eq);
    let wz = give_free_card_kind_suit(&mut eq, 0, CardKind::Wuzhong, None);
    play_card(&mut eq, 0, wz, vec![]);
    assert_eq!(
        eq.open_choice().map(|c| c.spec.prompt),
        Some("hs_huanjue_pick")
    );
    save_restore_and_answer(&mut eq, "hs_huanjue_pick", vec![ChoiceOption::Card(qg)]);
    drain_auto(&mut eq);
    assert!(
        eq.state_core().player(0).equips.contains(&qg),
        "huanjue forced equip must equip into equips zone"
    );
    assert!(
        eq.state_core().player(0).hand.is_empty(),
        "huanjue forced equip must NOT stay in hand or draw Wuzhong"
    );
    assert!(eq.state_core().table.is_empty());
    assert!(eq.state_core().discard.contains(&wz));
    assert!(eq
        .log()
        .iter()
        .any(|l| matches!(l, LogEvent::Equipped { player: 0, card, .. } if *card == qg)));
    assert_eq!(total_tracked_cards(&eq), before_eq);

    // 2C) `huanjue` forced `Jiu`: activates `jiu_active` & disposes `hs_jiu` into `discard`, `hand` empty!
    let mut ej = fresh_proto(20_713, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ej, 0, "huanjue");
    advance_to_phase(&mut ej, 0, Phase::Play);
    clear_hand(&mut ej, 0);
    let jiu_c = ej
        .state_core()
        .detached_cards
        .iter()
        .find_map(|&(c, _)| (ej.catalog().card_kind(c) == CardKind::Jiu).then_some(c))
        .expect("hs_jiu in detached_cards");
    let moved_j: Vec<u32> = ej
        .state_core()
        .detached_cards
        .iter()
        .filter_map(|&(c, _)| (c != jiu_c).then_some(c))
        .collect();
    ej.state_core_mut()
        .detached_cards
        .retain(|&(c, _)| c == jiu_c);
    ej.state_core_mut().discard.extend(moved_j);
    let wz_j = give_free_base_named(&mut ej, 0, "wuzhong");
    play_card(&mut ej, 0, wz_j, vec![]);
    assert_eq!(
        ej.open_choice().map(|c| c.spec.prompt),
        Some("hs_huanjue_pick")
    );
    save_restore_and_answer(&mut ej, "hs_huanjue_pick", vec![ChoiceOption::Card(jiu_c)]);
    drain_auto(&mut ej);
    assert!(ej.state_core().turn.jiu_active);
    assert!(ej.state_core().player(0).hand.is_empty());
    assert!(ej.state_core().discard.contains(&jiu_c));
    assert!(ej.state_core().table.is_empty());
}

#[test]
fn u05_b6b_round2_p0_oldyanshu_and_yanshu_outside_phase_discard_and_unbounded() {
    // 3A) `oldyanshu` triggers via skill discards (`oldzhanhou` / `fengxing` in `discard_card_from_zone`)
    // AND card `Dismantle` on equip, generating > 4 `liuxinghuoyu` tokens unboundedly while vanishing when used/discarded.
    let mut e = fresh_proto(20_721, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut e, 0, "oldyanshu");
    grant_host_skill(&mut e, 0, "oldzhanhou");
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);

    let mut gained_lx = Vec::new();
    for round_i in 0..6 {
        let arm = give_free_card_kind_suit(&mut e, 0, CardKind::Armor, None);
        let _ = activate(&mut e, 0, "oldzhanhou");
        assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("hs_zhanhou_card"));
        answer(&mut e, 0, vec![ChoiceOption::Card(arm)]);
        assert_eq!(
            e.open_choice().map(|c| c.spec.prompt),
            Some("hs_oldyanshu_confirm"),
            "round {round_i}: non-basic skill discard outside Phase::Discard must fire oldyanshu"
        );
        if round_i == 0 {
            save_restore_and_answer(&mut e, "hs_oldyanshu_confirm", vec![ChoiceOption::Confirm]);
        } else {
            answer(&mut e, 0, vec![ChoiceOption::Confirm]);
        }
        drain_auto(&mut e);
        let lx = *e
            .state_core()
            .player(0)
            .hand
            .iter()
            .find(|&&c| e.catalog().cards[c as usize].name.contains("liuxinghuoyu"))
            .expect("oldyanshu must grant liuxinghuoyu (> 4 times)");
        gained_lx.push(lx);
    }
    assert_eq!(gained_lx.len(), 6, "unbounded >4 liuxinghuoyu creations succeeded");

    // 3B) Batch-discarding generated `liuxinghuoyu` tokens via `yulu` triggers `oldyanshu` ONCE per batch
    // AND discarded `liuxinghuoyu` tokens vanish back into `detached_cards`!
    grant_host_skill(&mut e, 0, "yulu");
    let _ = activate(&mut e, 0, "yulu");
    answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
    let ch_yl = e.open_choice().cloned().expect("yulu discard");
    assert_eq!(ch_yl.spec.prompt, "hs_yulu_discard");
    answer(&mut e, 0, vec![ChoiceOption::Card(gained_lx[0])]);
    assert_eq!(
        e.open_choice().map(|c| c.spec.prompt),
        Some("hs_oldyanshu_confirm"),
        "yulu discard_cards_from_zone of liuxinghuoyu must trigger oldyanshu"
    );
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e);
    assert!(
        !e.state_core()
            .discard
            .iter()
            .any(|&c| e.catalog().cards[c as usize].name.contains("liuxinghuoyu")),
        "discarded Hearth liuxinghuoyu tokens must vanish into detached_cards, never clog discard"
    );

    // 3C) `yanshu` (`usable: 1`): triggers ONCE on non-basic skill discard, ignores 2nd in same turn
    let mut ey = fresh_proto(20_722, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ey, 0, "yanshu");
    grant_host_skill(&mut ey, 0, "oldzhanhou");
    advance_to_phase(&mut ey, 0, Phase::Play);
    clear_hand(&mut ey, 0);
    let a1 = give_free_card_kind_suit(&mut ey, 0, CardKind::Armor, None);
    let _ = activate(&mut ey, 0, "oldzhanhou");
    answer(&mut ey, 0, vec![ChoiceOption::Card(a1)]);
    assert_eq!(
        ey.open_choice().map(|c| c.spec.prompt),
        Some("hs_yanshu_confirm")
    );
    answer(&mut ey, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut ey);
    assert_eq!(flag(&ey, 0, "hs_b6b_cnt:yanshu"), 1);

    let a2 = give_free_card_kind_suit(&mut ey, 0, CardKind::Armor, None);
    let _ = activate(&mut ey, 0, "oldzhanhou");
    answer(&mut ey, 0, vec![ChoiceOption::Card(a2)]);
    assert!(ey.open_choice().is_none(), "yanshu limited to 1/turn");

    // 3D) `Dismantle` (`guohe`) discarding an equip triggers `oldyanshu`:
    let mut ed = fresh_proto(20_723, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ed, 1, "oldyanshu");
    advance_to_phase(&mut ed, 1, Phase::Play);
    clear_hand(&mut ed, 1);
    let eq1 = give_free_card_kind_suit(&mut ed, 1, CardKind::Armor, None);
    play_card(&mut ed, 1, eq1, vec![]);
    drain_auto(&mut ed);
    assert!(ed.state_core().player(1).equips.contains(&eq1));
    advance_to_phase(&mut ed, 0, Phase::Play);
    clear_hand(&mut ed, 0);
    clear_hand(&mut ed, 1);
    let gh = give_free_card_kind_suit(&mut ed, 0, CardKind::Dismantle, None);
    play_card(&mut ed, 0, gh, vec![1]);
    assert_eq!(
        ed.open_choice().map(|c| c.spec.prompt),
        Some("hs_oldyanshu_confirm"),
        "Dismantle discarding equip must emit CardsDiscarded and trigger oldyanshu"
    );
    answer(&mut ed, 1, vec![ChoiceOption::Confirm]);
    drain_auto(&mut ed);
    assert!(ed.state_core().player(1).hand.iter().any(|&c| {
        ed.catalog().cards[c as usize]
            .name
            .contains("liuxinghuoyu")
    }));

    // 3E) `discard_from_hand` (`machine.rs:671`) via `Linghunzhihuo` random self-hand discard outside `Phase::Discard`:
    let mut elh = fresh_proto(20_724, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut elh, 0, "oldyanshu");
    advance_to_phase(&mut elh, 0, Phase::Play);
    clear_hand(&mut elh, 0);
    clear_hand(&mut elh, 1);
    let _arm_disc = give_free_card_kind_suit(&mut elh, 0, CardKind::Armor, None);
    let lhz = give_free_card_kind_suit(&mut elh, 0, CardKind::Linghunzhihuo, None);
    play_card(&mut elh, 0, lhz, vec![1]);
    assert_eq!(
        elh.open_choice().map(|c| c.spec.prompt),
        Some("hs_oldyanshu_confirm"),
        "kernel discard_from_hand on non-basic must trigger oldyanshu in Hearth"
    );
    answer(&mut elh, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut elh);
    assert!(elh.state_core().player(0).hand.iter().any(|&c| {
        elh.catalog().cards[c as usize]
            .name
            .contains("liuxinghuoyu")
    }));
}

#[test]
fn u05_b6b_round2_p1_yulu_draw_order_and_hanshuang_turnover_before_losehp() {
    // 4) `oldyulu` draw order across multiple targets [0, 1, 2] MUST be [0, 1, 2] (not LIFO [2, 1, 0])!
    let mut ey = fresh_proto(20_731, &["hs_malfurion", "hs_jaina", "hs_wuther"]);
    grant_host_skill(&mut ey, 0, "oldyulu");
    advance_to_phase(&mut ey, 0, Phase::Play);
    clear_hand(&mut ey, 0);
    clear_hand(&mut ey, 1);
    clear_hand(&mut ey, 2);
    let log_start = ey.log().len();
    let _ = activate(&mut ey, 0, "oldyulu");
    answer(
        &mut ey,
        0,
        vec![
            ChoiceOption::Player(0),
            ChoiceOption::Player(1),
            ChoiceOption::Player(2),
        ],
    );
    let draw_order: Vec<u8> = ey.log()[log_start..]
        .iter()
        .filter_map(|l| match l {
            LogEvent::CardsDrawn { player, cards } if !cards.is_empty() => Some(*player),
            _ => None,
        })
        .collect();
    assert_eq!(
        draw_order,
        vec![0, 1, 2],
        "oldyulu must execute draws in selected order [0, 1, 2]"
    );
    drain_auto(&mut ey);

    // 5) `hanshuang` causal sequence proof: TurnOvered MUST finish BEFORE `LoseHp` executes!
    // Put seat 0 at `hp = 1` holding `Tao`: when `LoseHp(0, 1)` runs, `dying_save` opens.
    // At the exact moment `dying_save` opens, seat 1 MUST ALREADY be `face_down == true`!
    let mut eh = fresh_proto(20_732, &["hs_jaina", "hs_lrexxar", "hs_wuther"]);
    grant_host_skill(&mut eh, 0, "hanshuang");
    advance_to_phase(&mut eh, 0, Phase::Play);
    clear_hand(&mut eh, 0);
    clear_hand(&mut eh, 1);
    eh.state_core_mut().player_mut(0).hp = 1;
    eh.state_core_mut().player_mut(1).hujia = 0;
    let tao_save = give_free_base_named(&mut eh, 0, "tao");
    let sha = give_free_card_kind_suit(&mut eh, 0, CardKind::Sha, Some(Suit::Spade));
    play_card(&mut eh, 0, sha, vec![1]);
    assert_eq!(
        eh.open_choice().map(|c| c.spec.prompt),
        Some("dying_save"),
        "hanshuang LoseHp drops seat 0 to dying AFTER TurnOvered finishes; p0_hp={} p1_fd={} p1_hp={} logs={:?}",
        eh.state_core().player(0).hp,
        eh.state_core().player(1).face_down,
        eh.state_core().player(1).hp,
        eh.log()
    );
    assert!(
        eh.state_core().player(1).face_down,
        "at Dying(LoseHp) pause time, target seat 1 MUST already have turned over"
    );
    assert!(eh
        .log()
        .iter()
        .any(|l| matches!(l, LogEvent::HpLost { player: 0, amount: 1 })));
    save_restore_and_answer(&mut eh, "dying_save", vec![ChoiceOption::Card(tao_save)]);
    drain_auto(&mut eh);
    assert!(eh.state_core().player(0).alive);
    assert_eq!(eh.state_core().player(0).hp, 1);
    assert!(eh.state_core().player(1).face_down);
    assert!(eh
        .log()
        .iter()
        .any(|l| matches!(l, LogEvent::HpRecovered { player: 0, amount: 1, .. })));
}

#[test]
fn u05_b6b_round2_p1_xiaorong_over_20_and_yuanzheng_hej_cancel_and_ai() {
    // 6) `xiaorong`: > 20 basics across repeated End conversions + recycling from `discard` + strict conservation
    let mut ex = fresh_proto(20_741, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ex, 0, "xiaorong");
    let before_ex = total_tracked_cards(&ex);
    let mut total_drawn_basics = 0usize;
    for cycle in 0..8 {
        advance_to_phase(&mut ex, 0, Phase::Play);
        clear_hand(&mut ex, 0);
        clear_hand(&mut ex, 1);
        for _ in 0..3 {
            let _ = give_free_card_kind_suit(&mut ex, 0, CardKind::Armor, None);
        }
        ex.state_core_mut().player_mut(0).hp = 1;
        advance_to_phase(&mut ex, 0, Phase::End);
        advance_to_phase(&mut ex, 1, Phase::Play);
        // Check basics now in discard vs drawn in seat 0 hand on End:
        let drawn_here = ex.state_core().player(0).hand.len();
        assert_eq!(
            drawn_here, 6,
            "cycle {cycle}: 3 equips -> 6 basics (48 total > 20 without starving)"
        );
        total_drawn_basics += drawn_here;
        clear_hand(&mut ex, 0);
        assert_eq!(total_tracked_cards(&ex), before_ex);
    }
    assert_eq!(total_drawn_basics, 48);

    // 7A) `yuanzheng`: interactive `ChoiceKind::SelectCards` on target `"hej"` (`judge + equips + hand`),
    // picking judge/equip card removes that exact card and preserves others.
    let mut eyz = fresh_proto(20_742, &["hs_lrexxar", "hs_jaina", "hs_wuther", "hs_alleria"]);
    grant_host_skill(&mut eyz, 0, "yuanzheng");
    advance_to_phase(&mut eyz, 0, Phase::Play);
    clear_hand(&mut eyz, 0);
    clear_hand(&mut eyz, 2);
    let p2_hand = give_free_card_kind_suit(&mut eyz, 2, CardKind::Shan, None);
    let p2_armor = give_free_card_kind_suit(&mut eyz, 2, CardKind::Armor, None);
    let p2_le = give_free_card_kind_suit(&mut eyz, 2, CardKind::Le, None);
    eyz.state_core_mut().player_mut(2).hand.retain(|&c| c != p2_armor && c != p2_le);
    eyz.state_core_mut().player_mut(2).equips.push(p2_armor);
    eyz.state_core_mut().player_mut(2).judge.push(p2_le);
    let duel = give_free_card_kind_suit(&mut eyz, 0, CardKind::Duel, None);
    play_card(&mut eyz, 0, duel, vec![2]);
    let ch = eyz.open_choice().cloned().expect("yuanzheng select hej");
    assert_eq!(ch.spec.prompt, "hs_yuanzheng_card");
    assert_eq!(ch.spec.kind, ChoiceKind::SelectCards);
    assert!(ch.spec.allow_cancel);
    assert_eq!(ch.observer, 0);
    assert!(ch.spec.options.contains(&ChoiceOption::Card(p2_le)));
    assert!(ch.spec.options.contains(&ChoiceOption::Card(p2_armor)));
    assert!(ch.spec.options.contains(&ChoiceOption::Card(p2_hand)));

    // 7B) AI (`BoundedPlanner`) picks public enemy equip (`p2_armor`) cleanly without peeking hand!
    assert_eq!(ai_choose_current(&eyz), vec![ChoiceOption::Card(p2_armor)]);

    // Pick `p2_le` from judge via NGSA 107 save/restore:
    save_restore_and_answer(&mut eyz, "hs_yuanzheng_card", vec![ChoiceOption::Card(p2_le)]);
    drain_auto(&mut eyz);
    assert!(!eyz.state_core().player(2).judge.contains(&p2_le));
    assert!(eyz.state_core().player(2).equips.contains(&p2_armor));
    assert!(eyz.state_core().player(2).hand.contains(&p2_hand));

    // 7C) Cancel branch leaves target cards untouched
    let mut eyzc = fresh_proto(20_743, &["hs_lrexxar", "hs_jaina", "hs_wuther", "hs_alleria"]);
    grant_host_skill(&mut eyzc, 0, "yuanzheng");
    advance_to_phase(&mut eyzc, 0, Phase::Play);
    clear_hand(&mut eyzc, 0);
    clear_hand(&mut eyzc, 2);
    let keep2 = give_free_card_kind_suit(&mut eyzc, 2, CardKind::Shan, None);
    let duel_c = give_free_card_kind_suit(&mut eyzc, 0, CardKind::Duel, None);
    play_card(&mut eyzc, 0, duel_c, vec![2]);
    assert_eq!(
        eyzc.open_choice().map(|c| c.spec.prompt),
        Some("hs_yuanzheng_card")
    );
    cancel_open_choice(&mut eyzc);
    drain_auto(&mut eyzc);
    assert!(eyzc.state_core().player(2).hand.contains(&keep2));
}
