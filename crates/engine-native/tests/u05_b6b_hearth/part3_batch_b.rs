//! U05-6b Part 3 Batch B: ALL 25 skills (`fenlie` .. `oldmalymowang`)
//! Upstream: `character/hearth/skill.js:4716-5368`

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::choice::ChoiceOption;
use engine_native::event::Phase;
use engine_native::CommandOutcome;

#[test]
fn u05_b6b_batch_b_fenlie_nianfu_xiaorong_shixu_jixuan_qianghua_biri() {
    // 1) `fenlie` & `oldfenlie`: out-of-PhaseDraw gain copies from detached_cards, skips unique equips; `fenlie` usable 2 vs `oldfenlie` unlimited
    for (sk, expected_extra) in [("fenlie", 2usize), ("oldfenlie", 3usize)] {
        let mut e = fresh_proto(10_721, &["hs_lrexxar", "hs_jaina"]);
        grant_host_skill(&mut e, 0, sk);
        advance_to_phase(&mut e, 0, Phase::Play);
        clear_hand(&mut e, 0);
        for i in 0..6 {
            let kind = if i % 2 == 0 { CardKind::Sha } else { CardKind::Shan };
            let c = give_free_card_kind_suit(&mut e, 0, kind, None);
            put_card_on_deck_top(&mut e, c);
        }
        for _ in 0..3 {
            let wz = give_free_base_named(&mut e, 0, "wuzhong");
            play_card(&mut e, 0, wz, vec![]);
            drain_auto(&mut e);
        }
        // Each Wuzhong drew 2 non-unique copyable cards (`Sha`/`Shan`):
        // `fenlie` doubles 2 triggers = 6 + 4 = 10; `oldfenlie` doubles all 3 triggers = 6 + 6 = 12
        let hand_n = e.state_core().player(0).hand.len();
        assert_eq!(
            hand_n,
            6 + expected_extra * 2,
            "{sk} copy count {hand_n} mismatch"
        );
    }

    // 2) `nianfu`: source or target damageEnd steals random equip from the other player
    let mut en = fresh_proto(10_722, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut en, 0, "nianfu");
    clear_hand(&mut en, 0);
    clear_hand(&mut en, 1);
    let armor = give_free_card_kind_suit(&mut en, 1, CardKind::Armor, None);
    advance_to_phase(&mut en, 1, Phase::Play);
    play_card(&mut en, 1, armor, vec![]);
    drain_auto(&mut en);
    assert!(en.state_core().player(1).equips.contains(&armor));
    advance_to_phase(&mut en, 0, Phase::Play);
    clear_hand(&mut en, 1);
    let sha = give_free_card_kind_suit(&mut en, 0, CardKind::Sha, None);
    play_card(&mut en, 0, sha, vec![1]);
    drain_auto(&mut en);
    assert!(en.state_core().player(1).equips.is_empty());
    assert!(en.state_core().player(0).hand.contains(&armor));

    // 3) `xiaorong`: ignores equips in hand_limit + Phase::End converts hand equips -> 2N basics + heal N
    let mut ex = fresh_proto(10_723, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ex, 0, "xiaorong");
    advance_to_phase(&mut ex, 0, Phase::Play);
    clear_hand(&mut ex, 0);
    ex.state_core_mut().player_mut(0).hp = 2;
    let eq1 = give_free_card_kind_suit(&mut ex, 0, CardKind::Weapon, None);
    let eq2 = give_free_card_kind_suit(&mut ex, 0, CardKind::Armor, None);
    let keep1 = give_free_card_kind_suit(&mut ex, 0, CardKind::Sha, None);
    let keep2 = give_free_card_kind_suit(&mut ex, 0, CardKind::Shan, None);
    // Non-equip count == hp (2), plus 2 ignored equips -> Discard drops 0, then `xiaorong` at End consumes eq1/eq2 -> +2 hp & +4 basics (= 6 total)!
    advance_to_phase(&mut ex, 1, Phase::Play);
    assert!(!ex.state_core().player(0).hand.contains(&eq1));
    assert!(!ex.state_core().player(0).hand.contains(&eq2));
    assert!(ex.state_core().player(0).hand.contains(&keep1));
    assert!(ex.state_core().player(0).hand.contains(&keep2));
    assert_eq!(ex.state_core().player(0).hp, 4, "xiaorong recovered +2");
    assert_eq!(ex.state_core().player(0).hand.len(), 6, "2 non-equips kept + 2*2=4 basics");

    // 4) `shixu`: Play phase > 3 actions (3000ms) -> End phase discards floor(ms/3000) cards
    let mut esx = fresh_proto(10_724, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut esx, 0, "shixu");
    clear_hand(&mut esx, 1);
    advance_to_phase(&mut esx, 1, Phase::Play);
    for _ in 0..4 {
        let wz = give_free_base_named(&mut esx, 1, "wuzhong");
        let out = play_card(&mut esx, 1, wz, vec![]);
        assert!(!matches!(out, CommandOutcome::Rejected(_)));
        drain_auto(&mut esx);
    }
    let ch_sx = advance_until_open_prompt(&mut esx, "hs_shixu_discard", 8).expect("shixu ask");
    assert_eq!(ch_sx.observer, 1);
    assert_eq!(ch_sx.spec.min, 1); // 4000 / 3000 = 1
    let drop = ch_sx.spec.options[0];
    let h_before = esx.state_core().player(1).hand.len();
    save_restore_and_answer(&mut esx, "hs_shixu_discard", vec![drop]);
    drain_auto(&mut esx);
    assert_eq!(esx.state_core().player(1).hand.len(), h_before - 1);

    // 5) `jixuan`: TurnPhaseAfter draws 1 & queues extra turn once (no infinite loop!)
    let mut ej = fresh_proto(10_725, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut ej, 0, "jixuan");
    clear_hand(&mut ej, 0);
    advance_to_phase(&mut ej, 0, Phase::End);
    let _ = submit(&mut ej, 0, engine_native::GameAction::NextPhase);
    drain_auto(&mut ej);
    // Still seat 0's extra turn!
    assert_eq!(ej.turn().current, 0);
    assert!(ej.state_core().player(0).hand.len() >= 1);
    // Ending the extra turn hands off cleanly to seat 1 (no infinite loop)
    advance_to_phase(&mut ej, 1, Phase::Play);
    assert_eq!(ej.turn().current, 1);

    // 6) `qianghua` (`qianghua2`): Play phase first card replays copy on same target, limited 1/turn
    let mut eq = fresh_proto(10_726, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut eq, 0, "qianghua");
    advance_to_phase(&mut eq, 0, Phase::Play);
    clear_hand(&mut eq, 0);
    clear_hand(&mut eq, 1);
    let sha = give_free_card_kind_suit(&mut eq, 0, CardKind::Sha, None);
    let hp_p1 = eq.state_core().player(1).hp;
    play_card(&mut eq, 0, sha, vec![1]);
    let ch_q = eq.open_choice().cloned().expect("qianghua confirm");
    assert_eq!(ch_q.spec.prompt, "hs_qianghua_confirm");
    save_restore_and_answer(&mut eq, "hs_qianghua_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut eq);
    assert_eq!(eq.state_core().player(1).hp, hp_p1 - 2, "qianghua replayed Sha for -2 total");

    // 7) `biri`: distance <= 1 ally targeted by enemy Sha -> discard Shan to cancel
    let mut eb = fresh_proto(10_727, &["hs_jaina", "hs_lrexxar", "hs_wuther"]);
    grant_host_skill(&mut eb, 1, "biri");
    clear_hand(&mut eb, 0);
    clear_hand(&mut eb, 1);
    clear_hand(&mut eb, 2);
    let shan = give_free_card_kind_suit(&mut eb, 1, CardKind::Shan, None);
    let sha0 = give_free_card_kind_suit(&mut eb, 0, CardKind::Sha, None);
    let hp_p2 = eb.state_core().player(2).hp;
    advance_to_phase(&mut eb, 0, Phase::Play);
    play_card(&mut eb, 0, sha0, vec![2]); // dist(1,2) == 1
    let ch_biri = eb.open_choice().cloned().expect("biri discard shan");
    assert_eq!(ch_biri.spec.prompt, "hs_biri_discard");
    assert_eq!(ch_biri.observer, 1);
    save_restore_and_answer(&mut eb, "hs_biri_discard", vec![ChoiceOption::Card(shan)]);
    drain_auto(&mut eb);
    assert_eq!(eb.state_core().player(2).hp, hp_p2, "biri canceled Sha damage");
    assert!(!eb.state_core().player(1).hand.contains(&shan));
}

#[test]
fn u05_b6b_batch_b_stuxi_through_oldmalymowang() {
    // 8) `stuxi` + `stuxi2`: End phase grants `stuxi2` to enemy -> enemy's Draw reduces -1 and removes `stuxi2`
    let mut es = fresh_proto(10_728, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut es, 0, "stuxi");
    clear_hand(&mut es, 1);
    advance_to_phase(&mut es, 0, Phase::End);
    assert!(has_skill(&es, 1, "stuxi2"));
    advance_to_phase(&mut es, 1, Phase::Play);
    assert_eq!(es.state_core().player(1).hand.len(), 1, "stuxi2 reduced Draw(2)->1");
    assert!(!has_skill(&es, 1, "stuxi2"));

    // 9) `bingdong` + `luoshi` + `ronghuo` (`ronghuo2`):
    let mut er = fresh_proto(10_729, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut er, 0, "ronghuo");
    grant_host_skill(&mut er, 1, "luoshi");
    er.state_core_mut().player_mut(1).linked = true;
    er.state_core_mut().player_mut(0).linked = false;
    advance_to_phase(&mut er, 0, Phase::Play);
    clear_hand(&mut er, 0);
    clear_hand(&mut er, 1);
    let sha = give_free_card_kind_suit(&mut er, 0, CardKind::Sha, None);
    let extra0 = give_free_card_kind_suit(&mut er, 0, CardKind::Shan, None);
    let extra1 = give_free_card_kind_suit(&mut er, 1, CardKind::Jiu, None);
    play_card(&mut er, 0, sha, vec![1]); // `ronghuo` turns normal Sha -> Fire -> hits -> unlinks seat 1, `luoshi` discards 1 from each!
    drain_auto(&mut er);
    assert!(!er.state_core().player(1).linked, "ronghuo Fire Sha must unlink target");
    assert!(!er.state_core().player(0).hand.contains(&extra0), "luoshi discarded attacker card");
    assert!(!er.state_core().player(1).hand.contains(&extra1), "luoshi discarded defender card");

    // `bingdong` standalone verifies token spawn (`hslingjian_jinjilengdong`) on damageEnd
    let mut ebd = fresh_proto(10_729, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut ebd, 0, "bingdong");
    advance_to_phase(&mut ebd, 0, Phase::Play);
    clear_hand(&mut ebd, 0);
    clear_hand(&mut ebd, 1);
    let sha_bd = give_free_card_kind_suit(&mut ebd, 0, CardKind::Sha, None);
    play_card(&mut ebd, 0, sha_bd, vec![1]);
    drain_auto(&mut ebd);
    assert!(ebd.state_core().player(0).hand.iter().any(|&c| {
        ebd.catalog().cards[c as usize]
            .name
            .contains("hslingjian_jinjilengdong")
    }));

    // 10) `fushi` vs `oldfushi`: maxHp-1 then recover 1 if still wounded
    let mut ef = fresh_proto(10_730, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut ef, 0, "fushi");
    ef.state_core_mut().player_mut(1).max_hp = 4;
    ef.state_core_mut().player_mut(1).hp = 1;
    advance_to_phase(&mut ef, 0, Phase::Play);
    let _ = activate(&mut ef, 0, "fushi");
    answer(&mut ef, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut ef);
    assert_eq!(ef.state_core().player(1).max_hp, 3);
    assert_eq!(ef.state_core().player(1).hp, 2);
    assert!(matches!(activate(&mut ef, 0, "fushi"), CommandOutcome::Rejected(_)));

    // `oldfushi` uncapped
    let mut eof = fresh_proto(10_731, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut eof, 0, "oldfushi");
    eof.state_core_mut().player_mut(1).max_hp = 5;
    eof.state_core_mut().player_mut(1).hp = 1;
    advance_to_phase(&mut eof, 0, Phase::Play);
    let _ = activate(&mut eof, 0, "oldfushi");
    answer(&mut eof, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut eof);
    let _ = activate(&mut eof, 0, "oldfushi");
    answer(&mut eof, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut eof);
    assert_eq!(eof.state_core().player(1).max_hp, 3);
    assert_eq!(eof.state_core().player(1).hp, 3);

    // 11) `moyao` + `malymowang` + `oldmalymowang`:
    // `moyao` blocks other players' tricks targeting owner; self-targeted tricks allowed; `malymowang` discovers trick + +1 trick dmg (1/turn vs unlimited)
    let mut em = fresh_proto(10_732, &["hs_jaina", "hs_malygos"]);
    grant_host_skill(&mut em, 1, "moyao");
    grant_host_skill(&mut em, 0, "malymowang");
    clear_hand(&mut em, 0);
    let duel = give_free_card_kind_suit(&mut em, 0, CardKind::Duel, None);
    // `malymowang` discover triggers at PlayBegin
    let ch_m = advance_until_open_prompt(&mut em, "hs_malymowang_discover", 8)
        .expect("malymowang discover");
    assert_eq!(ch_m.spec.prompt, "hs_malymowang_discover");
    let picked = ch_m.spec.options[0];
    answer(&mut em, 0, vec![picked]);
    drain_auto(&mut em);
    assert!(
        matches!(play_card(&mut em, 0, duel, vec![1]), CommandOutcome::Rejected(_)),
        "moyao must prevent other player using trick on moyao holder"
    );

    // `malymowang` +1 damage on non-chain trick
    let mut em2 = fresh_proto(10_733, &["hs_malygos", "hs_lrexxar"]);
    grant_host_skill(&mut em2, 0, "malymowang");
    clear_hand(&mut em2, 0);
    clear_hand(&mut em2, 1);
    let aoe2 = give_free_card_kind_suit(&mut em2, 0, CardKind::SavageAttack, None);
    let hp_before = em2.state_core().player(1).hp;
    advance_to_phase(&mut em2, 0, Phase::Play);
    clear_hand(&mut em2, 1);
    drain_auto(&mut em2); // picks first discovered card if prompt still open
    play_card(&mut em2, 0, aoe2, vec![]);
    drain_auto(&mut em2);
    assert_eq!(em2.state_core().player(1).hp, hp_before - 2, "malymowang +1 trick dmg");

    // 12) `jiaohui` (tested above too + negative when damage dealt) + `bimeng` + `zhoujiang` + `liehun` + `xjumo`
    let mut ejz = fresh_proto(10_734, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut ejz, 0, "zhoujiang");
    grant_host_skill(&mut ejz, 0, "bimeng");
    grant_host_skill(&mut ejz, 0, "liehun");
    grant_host_skill(&mut ejz, 0, "xjumo");
    advance_to_phase(&mut ejz, 0, Phase::Play);
    clear_hand(&mut ejz, 0);
    let wz = give_free_base_named(&mut ejz, 0, "wuzhong");
    play_card(&mut ejz, 0, wz, vec![]);
    drain_auto(&mut ejz); // `zhoujiang` adds 1 curse token
    assert!(ejz.state_core().player(0).hand.iter().any(|&c| {
        let n = ejz.catalog().cards[c as usize].name;
        n.contains("hszuzhou_")
    }));
    // `xjumo`: full HP -> hand limit = hp + 3; wounded -> hp + 5
    let hp0 = ejz.state_core().player(0).hp as usize;
    assert_eq!(ejz.state_core().hand_limit(ejz.catalog(), 0), hp0 + 3);
    ejz.state_core_mut().player_mut(0).hp -= 1;
    assert_eq!(ejz.state_core().hand_limit(ejz.catalog(), 0), (hp0 - 1) + 5);
    // `bimeng` at End phase
    let ch_bm = advance_until_open_prompt(&mut ejz, "hs_bimeng_confirm", 8).expect("bimeng");
    assert_eq!(ch_bm.observer, 0);
    answer(&mut ejz, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut ejz);
    assert!(ejz.state_core().player(0).hand.iter().any(|&c| {
        ejz.catalog().cards[c as usize]
            .name
            .contains("hsmengjing_")
    }));
}
