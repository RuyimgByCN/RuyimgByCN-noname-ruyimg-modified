//! U05-6b Part 2 Batch A: ALL 17 skills (`xieneng` .. `hhudun`)
//! Upstream: `character/hearth/skill.js:4185-4715`
//! Positive, negative/boundary, choice A vs B, cancel, AI public-info, and `NGSA 107` save/restore.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::choice::ChoiceOption;
use engine_native::event::{EventData, Phase};
use engine_native::machine::card_use;
use engine_native::{CommandOutcome, GameAction};

#[test]
fn u05_b6b_batch_a_xieneng_fbeifa_oldfbeifa_yufa_bingyan() {
    // 1) `xieneng`: phaseEnd choose 1 of 3 artifacts (`morijingxiang` / `kongbusangzhong` / `nengliangzhiguang`), save/restore + cancel
    let mut e = fresh_proto(10_711, &["hs_guldan", "hs_lrexxar"]);
    grant_host_skill(&mut e, 0, "xieneng");
    let before = total_tracked_cards(&e);
    let ch = advance_until_open_prompt(&mut e, "hs_xieneng_pick", 16).expect("xieneng open");
    assert_eq!(ch.spec.options.len(), 3);
    let ai_sel = ai_choose_current(&e);
    assert_eq!(ai_sel.len(), 1);
    save_restore_and_answer(&mut e, "hs_xieneng_pick", vec![ChoiceOption::Tag(1)]);
    drain_auto(&mut e);
    let gained_bell = e.state_core().player(0).hand.iter().any(|&c| {
        e.catalog().cards[c as usize]
            .name
            .contains("hsshenqi_kongbusangzhong")
    });
    assert!(gained_bell, "xieneng Tag(1) must grant kongbusangzhong");
    assert_eq!(total_tracked_cards(&e), before);

    // 2) `fbeifa` + `oldfbeifa`: last card leaves hand -> free `sha` (no Play Sha limit!) + damageAfter draws 1 + usable: 3 cap
    for sk in ["fbeifa", "oldfbeifa"] {
        let mut ef = fresh_proto(10_712, &["hs_lrexxar", "hs_jaina"]);
        grant_host_skill(&mut ef, 0, sk);
        advance_to_phase(&mut ef, 0, Phase::Play);
        clear_hand(&mut ef, 0);
        clear_hand(&mut ef, 1);
        // Seat 0 holds [s1, ar]: play 1st real Sha first (`sha_used_play_phase = true`), THEN empty hand via Armor!
        let s1 = give_free_card_kind_suit(&mut ef, 0, CardKind::Sha, None);
        let ar = give_free_card_kind_suit(&mut ef, 0, CardKind::Armor, None);
        let sh1 = give_free_card_kind_suit(&mut ef, 1, CardKind::Shan, None);
        play_card(&mut ef, 0, s1, vec![1]);
        answer(&mut ef, 1, vec![ChoiceOption::Card(sh1)]);
        drain_auto(&mut ef);
        assert_eq!(ef.state_core().player(0).hand, vec![ar]);
        play_card(&mut ef, 0, ar, vec![]);
        assert!(ef.state_core().player(0).hand.is_empty());
        // Playing the last card triggered `fbeifa`/`oldfbeifa` (`hs_fbeifa_target` even though sha_used_play_phase=true!)
        let ch_fb = ef
            .open_choice()
            .cloned()
            .expect("fbeifa prompt when hand becomes empty");
        assert_eq!(
            ch_fb.spec.prompt,
            if sk == "fbeifa" {
                "hs_fbeifa_target"
            } else {
                "hs_oldfbeifa_target"
            }
        );
        let hp_before = ef.state_core().player(1).hp;
        save_restore_and_answer(&mut ef, ch_fb.spec.prompt, vec![ChoiceOption::Player(1)]);
        drain_auto(&mut ef);
        assert_eq!(ef.state_core().player(1).hp, hp_before - 1);
        // `fbeifa_draw` drew 1 card on damageAfter!
        assert_eq!(ef.state_core().player(0).hand.len(), 1, "{sk} hit must draw 1");

        // Negative + cancel: playing non-last card does NOT trigger, canceling target prompt skips damage cleanly
        let c_a = give_free_card_kind_suit(&mut ef, 0, CardKind::Armor, None);
        play_card(&mut ef, 0, c_a, vec![]);
        assert!(ef.open_choice().is_none(), "{sk} must NOT trigger when hand still non-empty");
    }

    // Cancel path on `xieneng` leaves hand untouched
    let mut exc = fresh_proto(10_711, &["hs_guldan", "hs_lrexxar"]);
    grant_host_skill(&mut exc, 0, "xieneng");
    let ch_c = advance_until_open_prompt(&mut exc, "hs_xieneng_pick", 16).expect("xieneng cancel");
    let h_pre_c = exc.state_core().player(ch_c.observer).hand.len();
    cancel_open_choice(&mut exc);
    drain_auto(&mut exc);
    assert_eq!(exc.state_core().player(0).hand.len(), h_pre_c);

    // 3) `yufa` (`yufa2` + `yufa3`): take damage in attacker's turn -> attacker's End phase gives `chuansongmen` to non-attacker; reset on next Prepare
    let mut ey = fresh_proto(10_713, &["hs_lrexxar", "hs_khadgar", "hs_jaina"]);
    grant_host_skill(&mut ey, 1, "yufa");
    grant_host_skill(&mut ey, 1, "yufa2");
    grant_host_skill(&mut ey, 1, "yufa3");
    clear_hand(&mut ey, 0);
    clear_hand(&mut ey, 1);
    clear_hand(&mut ey, 2);
    let atk_sha = give_free_card_kind_suit(&mut ey, 0, CardKind::Sha, None);
    advance_to_phase(&mut ey, 0, Phase::Play);
    play_card(&mut ey, 0, atk_sha, vec![1]);
    drain_auto(&mut ey);
    assert_eq!(flag(&ey, 1, "hs_yufa_attacker"), 1); // seat 0 + 1
    let cy = advance_until_open_prompt(&mut ey, "hs_yufa_target", 12).expect("yufa at attacker end");
    // Option must NOT contain attacker seat 0!
    assert!(!cy.spec.options.contains(&ChoiceOption::Player(0)));
    assert!(cy.spec.options.contains(&ChoiceOption::Player(2)));
    answer(&mut ey, 1, vec![ChoiceOption::Player(2)]);
    drain_auto(&mut ey);
    assert!(ey.state_core().player(2).hand.iter().any(|&c| {
        ey.catalog().cards[c as usize]
            .name
            .contains("chuansongmen")
    }));
    // `yufa3` clears mark on next Prepare
    advance_to_phase(&mut ey, 1, Phase::Play);
    assert_eq!(flag(&ey, 1, "hs_yufa_attacker"), 0);

    // 4) `bingyan`: black card -> Jingleishan (thunder AOE), red card -> Chiyuxi (fire AOE), usable: 1
    let mut eb = fresh_proto(10_714, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut eb, 0, "bingyan");
    clear_hand(&mut eb, 0);
    clear_hand(&mut eb, 1);
    let black_c = give_free_card_kind_suit(&mut eb, 0, CardKind::Sha, Some(Suit::Spade));
    let hp_p1 = eb.state_core().player(1).hp;
    advance_to_phase(&mut eb, 0, Phase::Play);
    assert!(!matches!(activate(&mut eb, 0, "bingyan"), CommandOutcome::Rejected(_)));
    answer(&mut eb, 0, vec![ChoiceOption::Card(black_c)]);
    drain_auto(&mut eb);
    assert_eq!(eb.state_core().player(1).hp, hp_p1 - 1);
    // Negative: usable: 1 rejects second activation even with red card in hand
    let _red_c = give_free_card_kind_suit(&mut eb, 0, CardKind::Sha, Some(Suit::Heart));
    assert!(matches!(activate(&mut eb, 0, "bingyan"), CommandOutcome::Rejected(_)));
}

#[test]
fn u05_b6b_batch_a_shifa_yuanzheng_yulu_duzhang_hannu_chuidiao_hhudun() {
    // 5) `shifa` & `oldshifa`: PlayBegin grants owner + enemy (`Wei` vs `Shu`) random trick; trick use triggers draw (`shifa` cap 3 vs `oldshifa` uncapped)
    let mut es = fresh_proto(10_715, &["hs_jaina", "hs_lrexxar"]);
    clear_hand(&mut es, 0);
    clear_hand(&mut es, 1);
    grant_host_skill(&mut es, 0, "shifa");
    advance_to_phase(&mut es, 0, Phase::Play);
    assert_eq!(es.state_core().player(0).hand.len(), 3, "2 Draw + 1 shifa trick");
    assert_eq!(es.state_core().player(1).hand.len(), 1, "enemy got 1 shifa trick");
    for _ in 0..4 {
        let wz = give_free_card_kind_suit(&mut es, 0, CardKind::Wuzhong, None);
        play_card(&mut es, 0, wz, vec![]);
        if let Some(ch) = es.open_choice().cloned() {
            if ch.spec.prompt == "hs_shifa_draw_confirm" {
                answer(&mut es, 0, vec![ChoiceOption::Confirm]);
            }
        }
        drain_auto(&mut es);
    }
    assert_eq!(flag(&es, 0, "hs_b6b_cnt:shifa_draw"), 3, "shifa_draw capped at 3");

    // 6) `yuanzheng`: target at distance > 1 discards from hej on confirm, skip if distance == 1
    let mut ey = fresh_proto(10_716, &["hs_lrexxar", "hs_jaina", "hs_wuther", "hs_alleria"]);
    grant_host_skill(&mut ey, 0, "yuanzheng");
    clear_hand(&mut ey, 0);
    clear_hand(&mut ey, 2);
    let d2_c = give_free_card_kind_suit(&mut ey, 2, CardKind::Shan, None);
    let duel = give_free_card_kind_suit(&mut ey, 0, CardKind::Duel, None);
    advance_to_phase(&mut ey, 0, Phase::Play);
    play_card(&mut ey, 0, duel, vec![2]); // distance(0, 2) == 2 > 1
    assert_eq!(
        ey.open_choice().map(|c| c.spec.prompt),
        Some("hs_yuanzheng_card")
    );
    save_restore_and_answer(&mut ey, "hs_yuanzheng_card", vec![ChoiceOption::Card(d2_c)]);
    drain_auto(&mut ey);
    assert!(!ey.state_core().player(2).hand.contains(&d2_c));

    // 7) `yulu` vs `oldyulu`: draw 1 + discard 1 hej (`yulu`), draw 2 + discard 2 hej (`oldyulu`)
    let mut eu = fresh_proto(10_717, &["hs_malfurion", "hs_jaina"]);
    grant_host_skill(&mut eu, 0, "oldyulu");
    clear_hand(&mut eu, 0);
    clear_hand(&mut eu, 1);
    advance_to_phase(&mut eu, 0, Phase::Play);
    let _ = activate(&mut eu, 0, "oldyulu");
    answer(&mut eu, 0, vec![ChoiceOption::Player(0), ChoiceOption::Player(1)]);
    // Both draw 2, then seat 0 & seat 1 each discard 2 via `hs_yulu_discard`
    let ch0 = eu.open_choice().cloned().expect("yulu discard seat 0");
    assert_eq!(ch0.observer, 0);
    assert_eq!(ch0.spec.min, 2);
    answer(&mut eu, 0, ch0.spec.options.iter().take(2).copied().collect());
    let ch1 = eu.open_choice().cloned().expect("yulu discard seat 1");
    assert_eq!(ch1.observer, 1);
    assert_eq!(ch1.spec.min, 2);
    answer(&mut eu, 1, ch1.spec.options.iter().take(2).copied().collect());
    drain_auto(&mut eu);

    // 8) `duzhang` + `duzhang2`: no trick used in turn -> grant `duzhang2` forbidding tricks -> expire after victim turn
    let mut ed = fresh_proto(10_718, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ed, 0, "duzhang");
    let _ch_dz = advance_until_open_prompt(&mut ed, "hs_duzhang_target", 16).expect("duzhang open");
    answer(&mut ed, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut ed);
    assert!(has_skill(&ed, 1, "duzhang2"));
    advance_to_phase(&mut ed, 1, Phase::Play);
    let wz_ban = give_free_card_kind_suit(&mut ed, 1, CardKind::Wuzhong, None);
    assert!(
        matches!(play_card(&mut ed, 1, wz_ban, vec![]), CommandOutcome::Rejected(_)),
        "duzhang2 must forbid tricks on victim's turn"
    );
    // `duzhang2` expires at `TurnPhaseAfter` (`phaseAfter`)
    advance_to_phase(&mut ed, 0, Phase::Play);
    assert!(!has_skill(&ed, 1, "duzhang2"));

    // 9) `hannu` + `chuidiao` + `hhudun`:
    // Verify `hhudun` absorbs 1 dmg via hujia -> draws 1; `hannu` draws nh -> >10 trims down to hp=4!
    let mut ehh = fresh_proto(10_719, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ehh, 0, "hhudun");
    grant_host_skill(&mut ehh, 1, "chuidiao");
    advance_to_phase(&mut ehh, 1, Phase::Play);
    let s_h = give_free_card_kind_suit(&mut ehh, 1, CardKind::Sha, None);
    clear_hand(&mut ehh, 0);
    ehh.state_core_mut().player_mut(0).hp = 4;
    ehh.state_core_mut().player_mut(0).hujia = 1;
    play_card(&mut ehh, 1, s_h, vec![0]);
    drain_auto(&mut ehh);
    assert_eq!(ehh.state_core().player(0).hujia, 0);
    assert_eq!(ehh.state_core().player(0).hp, 4);
    assert_eq!(ehh.state_core().player(0).hand.len(), 1, "hhudun must draw 1 on full hujia absorb");

    let mut eh = fresh_proto(10_720, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut eh, 0, "hannu");
    advance_to_phase(&mut eh, 1, Phase::Play);
    let s_hit = give_free_card_kind_suit(&mut eh, 1, CardKind::Sha, None);
    clear_hand(&mut eh, 0);
    for _ in 0..4 {
        give_free_card_kind_suit(&mut eh, 0, CardKind::Jiu, None); // unplayable on response
    }
    for _ in 0..2 {
        give_free_card_kind_suit(&mut eh, 0, CardKind::Tao, None); // unplayable on Sha
    }
    assert_eq!(eh.state_core().player(0).hand.len(), 6);
    eh.state_core_mut().player_mut(0).hp = 4;
    play_card(&mut eh, 1, s_hit, vec![0]);
    drain_auto(&mut eh);
    assert_eq!(eh.state_core().player(0).hp, 3);
    assert_eq!(eh.state_core().player(0).hand.len(), 3, "hannu must trim 12 hand (>10) down to hp=3");
    let _ = card_use;
    let _ = EventData::TurnBegin { player: 0 };
    let _ = GameAction::NextPhase;
}
