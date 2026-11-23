//! U05-6b Part 5 Batch D: ALL 20 skills (`jingmeng` .. `anying`)
//! Upstream: `character/hearth/skill.js:5836-6303`

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::choice::ChoiceOption;
use engine_native::event::Phase;
use engine_native::CommandOutcome;

#[test]
fn u05_b6b_batch_d_jingmeng_through_dunji() {
    // 1) `jingmeng` + `hswuji`: first card played draws same-type from deck; PlayEnd draws N used cards!
    let mut e = fresh_proto(10_761, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut e, 0, "jingmeng");
    grant_host_skill(&mut e, 0, "hswuji");
    advance_to_phase(&mut e, 0, Phase::Play);
    clear_hand(&mut e, 0);
    clear_hand(&mut e, 1);
    let sha1 = give_free_card_kind_suit(&mut e, 0, CardKind::Sha, None);
    let wz2 = give_free_card_kind_suit(&mut e, 0, CardKind::Wuzhong, None);
    play_card(&mut e, 0, sha1, vec![1]);
    save_restore_and_answer(&mut e, "hs_jingmeng_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut e);
    assert!(e
        .state_core()
        .player(0)
        .hand
        .iter()
        .any(|&c| e.catalog().card_kind(c).is_basic_kind()));
    // 2nd card must NOT trigger jingmeng again
    play_card(&mut e, 0, wz2, vec![]);
    assert_ne!(e.open_choice().map(|c| c.spec.prompt), Some("hs_jingmeng_confirm"));
    drain_auto(&mut e);
    // End of Play triggers `hswuji` (draws 2 cards)
    let _ = submit(&mut e, 0, engine_native::GameAction::NextPhase);
    assert_eq!(e.open_choice().map(|c| c.spec.prompt), Some("hs_hswuji_confirm"));
    let h_before = e.state_core().player(0).hand.len();
    answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut e);
    assert_eq!(e.state_core().player(0).hand.len(), h_before + 2);

    // 2) `bingshuang` + `yanshu` + `oldyanshu` + `shengyan` (`shengyan2`)
    let mut eb = fresh_proto(10_762, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut eb, 0, "bingshuang");
    grant_host_skill(&mut eb, 0, "yanshu");
    grant_host_skill(&mut eb, 1, "shengyan");
    advance_to_phase(&mut eb, 0, Phase::Play);
    clear_hand(&mut eb, 0);
    clear_hand(&mut eb, 1);
    let aoe = give_free_card_kind_suit(&mut eb, 0, CardKind::SavageAttack, None);
    play_card(&mut eb, 0, aoe, vec![]);
    assert_eq!(
        eb.open_choice().map(|c| c.spec.prompt),
        Some("hs_bingshuang_confirm")
    );
    save_restore_and_answer(&mut eb, "hs_bingshuang_confirm", vec![ChoiceOption::Confirm]);
    drain_auto(&mut eb);
    assert!(eb.state_core().player(1).face_down);

    // `yanshu` on discarding non-basic
    eb.state_core_mut().player_mut(0).hp = 1;
    clear_hand(&mut eb, 0);
    let _non_basic = give_free_card_kind_suit(&mut eb, 0, CardKind::Wuzhong, None);
    let _armor_nb = give_free_card_kind_suit(&mut eb, 0, CardKind::Armor, None);
    let ch_ys = advance_until_open_prompt(&mut eb, "hs_yanshu_confirm", 8).expect("yanshu");
    answer(&mut eb, ch_ys.observer, vec![ChoiceOption::Confirm]);
    drain_auto(&mut eb);
    assert!(eb.state_core().player(0).hand.iter().any(|&c| {
        eb.catalog().cards[c as usize]
            .name
            .contains("liuxinghuoyu")
    }));

    // `shengyan` (`shengyan2` stops recursion): recover when wounded triggers extra +1 once per turn
    let mut esy = fresh_proto(10_763, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut esy, 0, "shengyan");
    clear_hand(&mut esy, 0);
    esy.state_core_mut().player_mut(0).hp = 1;
    let tao = give_free_card_kind_suit(&mut esy, 0, CardKind::Tao, None);
    advance_to_phase(&mut esy, 0, Phase::Play);
    play_card(&mut esy, 0, tao, vec![]);
    assert_eq!(
        esy.open_choice().map(|c| c.spec.prompt),
        Some("hs_shengyan_confirm")
    );
    answer(&mut esy, 0, vec![ChoiceOption::Confirm]);
    drain_auto(&mut esy);
    assert_eq!(esy.state_core().player(0).hp, 3, "Tao(+1) + shengyan(+1) once only");
    assert!(has_skill(&esy, 0, "shengyan2"));

    // 3) `liechao` + `qingliu` + `zhongjia` + `dunji`:
    let mut el = fresh_proto(10_764, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut el, 0, "liechao");
    grant_host_skill(&mut el, 0, "qingliu");
    grant_host_skill(&mut el, 0, "zhongjia");
    grant_host_skill(&mut el, 0, "dunji");
    advance_to_phase(&mut el, 0, Phase::Play);
    clear_hand(&mut el, 0);
    let _ = activate(&mut el, 0, "liechao");
    drain_auto(&mut el);
    assert!(el.state_core().player(0).face_down);
    assert_eq!(el.state_core().player(0).hand.len(), 4);
    // Fire Sha against `qingliu` is prevented!
    let mut eqz = fresh_proto(10_765, &["hs_jaina", "hs_lrexxar"]);
    grant_host_skill(&mut eqz, 1, "qingliu");
    grant_host_skill(&mut eqz, 1, "zhongjia");
    grant_host_skill(&mut eqz, 1, "dunji");
    clear_hand(&mut eqz, 0);
    clear_hand(&mut eqz, 1);
    let fsha = give_free_card_kind_suit(&mut eqz, 0, CardKind::FireSha, None);
    let hp_q = eqz.state_core().player(1).hp;
    advance_to_phase(&mut eqz, 0, Phase::Play);
    play_card(&mut eqz, 0, fsha, vec![1]);
    drain_auto(&mut eqz);
    assert_eq!(eqz.state_core().player(1).hp, hp_q, "qingliu prevented Fire dmg");
    // Normal Sha hits `zhongjia` -> +1 hujia -> seat 1 uses `dunji` spending hujia to damage seat 0
    advance_to_phase(&mut eqz, 1, Phase::Play);
    advance_to_phase(&mut eqz, 0, Phase::Play);
    let nsha = give_free_card_kind_suit(&mut eqz, 0, CardKind::Sha, None);
    clear_hand(&mut eqz, 1);
    play_card(&mut eqz, 0, nsha, vec![1]);
    drain_auto(&mut eqz);
    assert_eq!(eqz.state_core().player(1).hujia, 1, "zhongjia +1 hujia on >0 dmg");
    advance_to_phase(&mut eqz, 1, Phase::Play);
    let hp0_d = eqz.state_core().player(0).hp;
    let _ = activate(&mut eqz, 1, "dunji");
    answer(&mut eqz, 1, vec![ChoiceOption::Player(0)]);
    drain_auto(&mut eqz);
    assert_eq!(eqz.state_core().player(1).hujia, 0);
    assert_eq!(eqz.state_core().player(0).hp, hp0_d - 1);
}

#[test]
fn u05_b6b_batch_d_fengxing_through_anying() {
    // 4) `fengxing` (`fengxing2`): off-turn loss -> temp fengxing2 + discard 1 he -> draw 2 (once per turn, non-recursive!)
    let mut efx = fresh_proto(10_766, &["hs_lrexxar", "hs_jaina"]);
    grant_host_skill(&mut efx, 1, "fengxing");
    clear_hand(&mut efx, 0);
    clear_hand(&mut efx, 1);
    let shan1 = give_free_card_kind_suit(&mut efx, 1, CardKind::Shan, None);
    let spare1 = give_free_card_kind_suit(&mut efx, 1, CardKind::Jiu, None);
    let sha0 = give_free_card_kind_suit(&mut efx, 0, CardKind::Sha, None);
    advance_to_phase(&mut efx, 0, Phase::Play);
    play_card(&mut efx, 0, sha0, vec![1]);
    answer(&mut efx, 1, vec![ChoiceOption::Card(shan1)]); // losing Shan off-turn triggers fengxing
    let ch_fx = efx.open_choice().cloned().expect("fengxing discard");
    assert_eq!(ch_fx.spec.prompt, "hs_fengxing_discard");
    assert!(has_skill(&efx, 1, "fengxing2"), "fengxing2 added BEFORE discard ask");
    save_restore_and_answer(&mut efx, "hs_fengxing_discard", vec![ChoiceOption::Card(spare1)]);
    drain_auto(&mut efx);
    assert_eq!(efx.state_core().player(1).hand.len(), 2, "fengxing drew 2");

    // 5) `fengxian` + `zhanhou` (`oldzhanhou`) + `shijie`
    let mut efn = fresh_proto(10_767, &["hs_wuther", "hs_jgarrosh"]);
    grant_host_skill(&mut efn, 0, "fengxian");
    advance_to_phase(&mut efn, 0, Phase::Play);
    clear_hand(&mut efn, 0);
    clear_hand(&mut efn, 1);
    let c0 = give_free_card_kind_suit(&mut efn, 0, CardKind::Sha, None);
    let c1 = give_free_card_kind_suit(&mut efn, 1, CardKind::Shan, None);
    let _ = activate(&mut efn, 0, "fengxian");
    answer(&mut efn, 0, vec![ChoiceOption::Card(c0)]);
    answer(&mut efn, 1, vec![ChoiceOption::Card(c1)]);
    drain_auto(&mut efn);
    assert!(efn.state_core().player(0).hand.is_empty());
    assert!(efn.state_core().player(1).hand.is_empty());

    // `oldzhanhou` uncapped
    let mut eoz = fresh_proto(10_768, &["hs_jgarrosh", "hs_wuther"]);
    grant_host_skill(&mut eoz, 0, "oldzhanhou");
    clear_hand(&mut eoz, 0);
    let a1 = give_free_card_kind_suit(&mut eoz, 0, CardKind::Armor, None);
    let a2 = give_free_card_kind_suit(&mut eoz, 0, CardKind::Armor, None);
    advance_to_phase(&mut eoz, 0, Phase::Play);
    let _ = activate(&mut eoz, 0, "oldzhanhou");
    answer(&mut eoz, 0, vec![ChoiceOption::Card(a1)]);
    let _ = activate(&mut eoz, 0, "oldzhanhou");
    answer(&mut eoz, 0, vec![ChoiceOption::Card(a2)]);
    drain_auto(&mut eoz);
    assert_eq!(eoz.state_core().player(0).hujia, 2);

    // 6) `shengguang` + `anying` + `xinci`: red heal -> awaken black loses `shengguang`, gains `xinci`, black -> loseHp
    let mut esa = fresh_proto(10_769, &["hs_anduin", "hs_lrexxar"]);
    grant_host_skill(&mut esa, 0, "shengguang");
    grant_host_skill(&mut esa, 0, "anying");
    clear_hand(&mut esa, 0);
    let blk1 = give_free_card_kind_suit(&mut esa, 0, CardKind::Sha, Some(Suit::Spade));
    let blk2 = give_free_card_kind_suit(&mut esa, 0, CardKind::Sha, Some(Suit::Club));
    let hp_t = esa.state_core().player(1).hp;
    advance_to_phase(&mut esa, 0, Phase::Play);
    let _ = activate(&mut esa, 0, "anying");
    answer(&mut esa, 0, vec![ChoiceOption::Card(blk1)]);
    drain_auto(&mut esa);
    assert!(!has_skill(&esa, 0, "shengguang"));
    assert!(has_skill(&esa, 0, "xinci"));
    assert!(matches!(activate(&mut esa, 0, "anying"), CommandOutcome::Rejected(_)));
    let _ = activate(&mut esa, 0, "xinci");
    answer(&mut esa, 0, vec![ChoiceOption::Card(blk2)]);
    answer(&mut esa, 0, vec![ChoiceOption::Player(1)]);
    drain_auto(&mut esa);
    assert_eq!(esa.state_core().player(1).hp, hp_t - 1);
}
