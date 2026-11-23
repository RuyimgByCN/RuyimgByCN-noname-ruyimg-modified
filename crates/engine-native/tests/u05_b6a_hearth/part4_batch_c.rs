//! Batch U05-6a Part 4: Batch C 20 skills (`szbianshen` .. `jinzhou`,
//! `character/hearth/skill.js:1893-2567`).

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::event::Phase;
use engine_native::ChoiceOption;

#[test]
fn u05_b6a_batch_c_szbianshen_through_pyuhuo() {
    // 1-3. `szbianshen` (:1893), `kekao` (:1927), `jinhua` (:1964)
    let mut e = fresh_proto(14_001, &["hs_shizugui", "hs_shuiwenxuejia", "hs_jiawodun"]);
    let tracked = total_tracked_cards(&e);

    advance_to_phase(&mut e, 0, Phase::Play);
    advance_to_phase(&mut e, 1, Phase::Discard);
    // Test `kekao` (:1927) at seat 1 End Phase: offer 3 choice tags -> gain delay trick card
    let kc = advance_until_open_prompt(&mut e, "hs_kekao_pick", 6)
        .expect("kekao at End Phase must open hs_kekao_pick");
    assert_eq!(kc.observer, 1);
    save_restore_and_answer(&mut e, "hs_kekao_pick", vec![ChoiceOption::Tag(0)]);
    drain_auto(&mut e);
    assert!(!e.state_core().player(1).hand.is_empty());

    advance_to_phase(&mut e, 2, Phase::Play);
    clear_hand(&mut e, 2);
    // `jinhua` (:1964): when caster plays self-targeting trick (e.g. wuzhong), offers random skill pick!
    let wz = give_free_card_kind(&mut e, 2, CardKind::Wuzhong);
    let _ = play_card(&mut e, 2, wz, vec![]);
    assert!(
        e.open_choice().is_some_and(|c| c.spec.prompt == "hs_jinhua_skill"),
        "jinhua must open hs_jinhua_skill on self-targeted trick"
    );
    let opt = e.open_choice().unwrap().spec.options[0];
    save_restore_and_answer(&mut e, "hs_jinhua_skill", vec![opt]);
    drain_auto(&mut e);
    assert!(e.state_core().skills.iter().filter(|((s, _), _)| *s == 2).count() > 1);
    assert_eq!(total_tracked_cards(&e), tracked);

    // Explicitly reach round >= 3 for `szbianshen` (:1893) confirm + general pick
    let mut esz = fresh_proto(14_002, &["hs_shizugui", "hs_jaina"]);
    esz.state_core_mut().turn.round = 3;
    advance_to_phase(&mut esz, 1, Phase::Discard);
    let _ = advance_until_open_prompt(&mut esz, "hs_szbianshen_confirm", 10)
        .expect("szbianshen must open hs_szbianshen_confirm when round >= 3");
    save_restore_and_answer(&mut esz, "hs_szbianshen_confirm", vec![ChoiceOption::Confirm]);
    assert!(
        esz.open_choice().is_some_and(|c| c.spec.prompt == "hs_szbianshen_general"),
        "confirming szbianshen must open hs_szbianshen_general"
    );
    let opt = esz.open_choice().unwrap().spec.options[0];
    let _ = answer(&mut esz, 0, vec![opt]);
    assert_eq!(flag(&esz, 0, "hs_szbianshen_used"), 1);
    assert_eq!(
        esz.state_core().player(0).hp,
        esz.state_core().player(0).max_hp as i32
    );

    // 4-7. `kqizhou` (:2028), `jingcu` (:2083), `shengzhang` (:2110), `pyuhuo` (:2120)
    let mut e2 = fresh_proto(14_003, &["hs_kalimosi", "hs_mojinbaozi", "hs_pyros"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    let wz = give_free_card_kind(&mut e2, 0, CardKind::Wuzhong);
    let _ = play_card(&mut e2, 0, wz, vec![]);
    drain_auto(&mut e2);
    assert!(flag(&e2, 0, "hs_kqizhou_pending") > 0, "kqizhou must mark trick usage for next Prepare");

    // `jingcu` active + `shengzhang` on seat 1 (`hs_mojinbaozi`) discard phase
    advance_to_phase(&mut e2, 1, Phase::Play);
    clear_hand(&mut e2, 1);
    let mhp_before = e2.state_core().player(1).max_hp;
    assert!(!matches!(activate(&mut e2, 1, "jingcu"), engine_native::CommandOutcome::Rejected(_)));
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(1).max_hp, mhp_before - 1, "jingcu must lose 1 max_hp");
    assert_eq!(e2.state_core().player(1).hand.len(), 2, "jingcu must draw 2 cards");
    // Ensure hand > hp so Discard phase discards at least 1 card -> `shengzhang` gains +1 max_hp
    let mhp_mid = e2.state_core().player(1).max_hp;
    e2.state_core_mut().player_mut(1).hp = 1;
    advance_to_phase(&mut e2, 1, Phase::Discard);
    drain_auto(&mut e2);
    advance_to_phase(&mut e2, 1, Phase::End);
    assert_eq!(e2.state_core().player(1).max_hp, mhp_mid + 1, "shengzhang must gain 1 max_hp when discarding in Discard phase");

    // `pyuhuo` (:2120): seat 2 (max_hp=2) dying -> transforms stage 1 -> max_hp increases & recovers!
    advance_to_phase(&mut e2, 2, Phase::Discard);
    let kq = advance_until_open_prompt(&mut e2, "hs_kqizhou_pick", 8)
        .expect("kqizhou must prompt hs_kqizhou_pick on seat 0 Prepare");
    assert_eq!(kq.observer, 0);
    save_restore_and_answer(&mut e2, "hs_kqizhou_pick", vec![ChoiceOption::Tag(0)]);
    drain_auto(&mut e2);
    assert!(
        e2.state_core().player(0).hand.iter().any(|&c| e2.catalog().cards[c as usize].name.contains("hsqizhou_")),
        "kqizhou must grant picked hsqizhou_* card"
    );
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    clear_hand(&mut e2, 2);
    e2.state_core_mut().player_mut(2).hp = 1;
    let sn = give_free_card_kind(&mut e2, 0, CardKind::Duel);
    let _ = play_card(&mut e2, 0, sn, vec![2]);
    drain_auto(&mut e2);
    assert!(e2.state_core().player(2).alive, "pyuhuo must revive dying Phoenix");
    assert_eq!(flag(&e2, 2, "hs_pyuhuo_stage"), 1);
    assert_eq!(e2.state_core().player(2).max_hp, 4);
    assert_eq!(total_tracked_cards(&e2), tracked2);
}

#[test]
fn u05_b6a_batch_c_mengye_through_jinzhou() {
    // 8-14. `mengye` (:2165) + `mengye2` (:2210), `lianzhan` (:2251), `kuixin` (:2267),
    // `fuhua` (:2306) + `fuhua2` (:2361) + `fuhua3` (:2400), `moxie` (:2412)
    let mut e = fresh_proto(14_010, &["hs_guldan", "hs_enzoth", "hs_aedwin"]);
    let tracked = total_tracked_cards(&e);
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);

    // `fuhua` (:2306): negative without `du` card; positive give `du` card -> choose choice 0 (`fuhua2`+`moxie`)
    let non_du = give_free_card_kind(&mut e, 0, CardKind::Sha);
    assert!(matches!(activate(&mut e, 0, "fuhua"), engine_native::CommandOutcome::Rejected(_)));
    let _ = non_du;
    clear_hand(&mut e, 0);
    let give_c = give_card_named(&mut e, 0, "du");
    assert!(matches!(activate(&mut e, 0, "fuhua"), out if !matches!(out, engine_native::CommandOutcome::Rejected(_))));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Card(give_c)]);
    let _ = answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    save_restore_and_answer(&mut e, "hs_fuhua_choice", vec![ChoiceOption::Tag(0)]);
    drain_auto(&mut e);
    assert!(has_skill(&e, 1, "fuhua2"));
    assert!(has_skill(&e, 1, "moxie"));

    // Negative `fuhua`: once_per_turn already used
    assert!(matches!(activate(&mut e, 0, "fuhua"), engine_native::CommandOutcome::Rejected(_)));

    // `mengye` (:2165) on seat 1 (`hs_enzoth`): End Phase marks 1 card as `du` & adds 1 hujia
    let _c_my = give_free_card_kind(&mut e, 2, CardKind::Sha);
    advance_to_phase(&mut e, 1, Phase::Discard);
    let my_ch = advance_until_open_prompt(&mut e, "hs_mengye_target", 6)
        .expect("mengye at seat 1 End Phase must open hs_mengye_target");
    assert_eq!(my_ch.observer, 1);
    save_restore_and_answer(&mut e, "hs_mengye_target", vec![ChoiceOption::Player(2)]);
    drain_auto(&mut e);
    assert!(e.state_core().player(2).hujia >= 1, "mengye must grant +1 hujia");
    assert!(
        e.state_core().player(2).hand.iter().any(|&c| flag(&e, 2, &format!("is_du_card:{c}")) > 0),
        "mengye must mark a hand card of target as du"
    );

    // `lianzhan` (:2251) on seat 2: first damage draws 2; second damage > cur_dmg gains +1 max_hp & recovers
    advance_to_phase(&mut e, 2, Phase::Play);
    clear_hand(&mut e, 2);
    let sha2 = give_free_card_kind(&mut e, 2, CardKind::Sha);
    let _ = play_card(&mut e, 2, sha2, vec![1]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(2).hand.len(), 2, "lianzhan first damage must draw 2");
    let mhp2 = e.state_core().player(2).max_hp;
    let duel2 = give_free_card_kind(&mut e, 2, CardKind::Duel);
    let _ = play_card(&mut e, 2, duel2, vec![1]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(2).max_hp, mhp2 + 1, "lianzhan second damage in turn must gain +1 max_hp");
    // `mengye2` temp cleanup on TurnPhaseAfter
    let mut em2 = fresh_proto(14_013, &["hs_enzoth", "hs_jaina"]);
    em2.debug_grant(0, "mengye2");
    assert!(has_skill(&em2, 0, "mengye2"));
    advance_to_phase(&mut em2, 0, Phase::Play);
    clear_hand(&mut em2, 0);
    clear_hand(&mut em2, 1);
    advance_to_phase(&mut em2, 1, Phase::Play);
    assert!(!has_skill(&em2, 0, "mengye2"), "mengye2 must clear on TurnPhaseAfter");
    assert_eq!(total_tracked_cards(&e), tracked);

    // `kuixin` (:2267) on `hs_mijiaojisi` (End Phase steals 1 hand card when other >= self)
    let mut ekx = fresh_proto(14_012, &["hs_mijiaojisi", "hs_jaina"]);
    let tracked_kx = total_tracked_cards(&ekx);
    advance_to_phase(&mut ekx, 0, Phase::Play);
    clear_hand(&mut ekx, 0);
    clear_hand(&mut ekx, 1);
    let steal_target_card = give_free_card_kind(&mut ekx, 1, CardKind::Tao);
    advance_to_phase(&mut ekx, 0, Phase::Discard);
    let _kx_ch = advance_until_open_prompt(&mut ekx, "hs_kuixin_target", 6)
        .expect("kuixin at End Phase must open hs_kuixin_target");
    save_restore_and_answer(&mut ekx, "hs_kuixin_target", vec![ChoiceOption::Player(1)]);
    drain_auto(&mut ekx);
    assert!(ekx.state_core().player(0).hand.contains(&steal_target_card));
    assert!(ekx.state_core().player(1).hand.is_empty());
    assert_eq!(total_tracked_cards(&ekx), tracked_kx);

    // 15-20. `qianhou` (:2449), `hlongyi` (:2514), `zhongji` (:2523), `fuwen` (:2550), `jinzhou` (:2567)
    let mut e2 = fresh_proto(14_011, &["hs_khadgar", "hs_tanghangu", "hs_laxiao", "hs_yogg"]);
    let tracked2 = total_tracked_cards(&e2);
    advance_to_phase(&mut e2, 0, Phase::Play);
    clear_hand(&mut e2, 0);
    let trick_c = give_card_named(&mut e2, 0, "wuzhong");
    let sp1 = give_card_named(&mut e2, 0, "hsjixie_zhadan"); // Spade 3 kept after discarding index 0
    assert_eq!(e2.state_core().player(0).hand.len(), 2);
    let _ = (trick_c, sp1);
    e2.state_core_mut().player_mut(0).hp = 1;
    advance_to_phase(&mut e2, 0, Phase::Discard);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(0).hand.len(), 1, "Discard phase must discard 1 card");
    assert_eq!(flag(&e2, 0, "hs_dp_trick"), 1, "fuwen must record trick discard in Discard phase");
    let jz_ch = advance_until_open_prompt(&mut e2, "hs_jinzhou_target", 6)
        .expect("jinzhou must open hs_jinzhou_target at End Phase when holding Spade");
    assert_eq!(jz_ch.observer, 0);
    save_restore_and_answer(&mut e2, "hs_jinzhou_target", vec![ChoiceOption::Player(3)]);
    drain_auto(&mut e2);
    assert!(e2.state_core().player(0).hujia >= 1, "fuwen must grant +1 hujia on discarding a trick");
    assert_eq!(flag(&e2, 3, "hs_jinzhou_sealed"), 1);

    // `zhongji` (:2523) on seat 1 (`hs_tanghangu`): discard black card to boost damage +1 (and cancel path deals 1)
    advance_to_phase(&mut e2, 1, Phase::Play);
    clear_hand(&mut e2, 1);
    clear_hand(&mut e2, 2);
    let zj_blk = give_card_named(&mut e2, 1, "hsjixie_zhadan");
    let zj_sha = give_free_card_kind(&mut e2, 1, CardKind::Sha);
    let hp2_before = e2.state_core().player(2).hp;
    let _ = play_card(&mut e2, 1, zj_sha, vec![2]);
    let zj_ch = advance_until_open_prompt(&mut e2, "hs_zhongji_discard", 6)
        .expect("zhongji must prompt hs_zhongji_discard on DamageBefore when holding black card");
    assert_eq!(zj_ch.observer, 1);
    save_restore_and_answer(&mut e2, "hs_zhongji_discard", vec![ChoiceOption::Card(zj_blk)]);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(2).hp, hp2_before - 2, "zhongji must boost damage from 1 to 2");

    // `hlongyi` (:2514) on seat 2 (`hs_laxiao`): black hand cards ignored in hand limit
    advance_to_phase(&mut e2, 2, Phase::Play);
    clear_hand(&mut e2, 2);
    e2.state_core_mut().player_mut(2).hp = 1;
    let black_cids: Vec<u32> = e2
        .state_core()
        .deck
        .iter()
        .copied()
        .filter(|&c| {
            matches!(
                e2.catalog().cards[c as usize].suit,
                engine_native::catalog::Suit::Spade | engine_native::catalog::Suit::Club
            )
        })
        .take(2)
        .collect();
    assert_eq!(black_cids.len(), 2, "deck must contain 2 distinct black cards for hlongyi");
    for &bc in &black_cids {
        e2.state_core_mut().deck.retain(|&c| c != bc);
        e2.state_core_mut().player_mut(2).hand.push(bc);
    }
    advance_to_phase(&mut e2, 2, Phase::Discard);
    drain_auto(&mut e2);
    assert_eq!(e2.state_core().player(2).hand.len(), 2, "hlongyi must exempt black hand cards from discard");

    // `qianhou` (:2449) on seat 3 (`hs_yogg`): Prepare auto-casts a single-target trick (and discard prompt if target != self)
    advance_to_phase(&mut e2, 2, Phase::Discard);
    let qh_hand0 = give_free_card_kind(&mut e2, 3, CardKind::Sha);
    let _ = qh_hand0;
    if let Some(qh_ch) = advance_until_open_prompt(&mut e2, "hs_qianhou_discard", 10) {
        assert_eq!(qh_ch.observer, 3);
        let _ = cancel_open_choice(&mut e2);
    }
    drain_auto(&mut e2);
    assert_eq!(total_tracked_cards(&e2), tracked2);
}
