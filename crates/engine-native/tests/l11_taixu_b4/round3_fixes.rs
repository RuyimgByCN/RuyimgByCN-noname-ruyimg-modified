//! Round 3 parity regression tests for verifier-l11 findings #1-#4 in board message #9,
//! plus other inline `draw_judge_card` + `to_discard` paths.

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::vocabulary::names::{BOSS_ZHONGDU, BOSS_ZJLEIJI, GANG_LIE, LUO_SHEN, TIE_JI};
use engine_native::{ChoiceOption, Phase};

#[test]
fn l11_b4_r3_luoshen_huangtianyili_zhizu_and_huangtianzhimin() {
    // 1. `luoshen` + `huangtianyili`: Heart 9 judge is overridden to Spade (black), so `luoshen`
    //    succeeds and Zhenji gains Heart 9 into hand, terminating cleanly when deck+discard empties.
    let mut e = start_versus(440, &[&[0], &[1]]);
    grant_skill(&mut e, 0, LUO_SHEN);
    grant_skill(&mut e, 0, "_buff_txhj_huangtianyili");
    let total_cards = e.state_core().deck.len() + e.state_core().discard.len();
    let heart9 = top_deck_suit_rank(&mut e, Suit::Heart, 9);
    e.debug_set_turn(1, Phase::End, 0);
    let _ = next_phase(&mut e);
    let ask = expect_pending(&e, "trigger_confirm");
    assert_eq!(ask.observer, 0);
    let _ = answer(&mut e, &ask, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert!(
        e.state_core().player(0).hand.contains(&heart9),
        "luoshen + huangtianyili must pass Heart 9 as black and gain it into hand"
    );
    assert!(
        !e.state_core().discard.contains(&heart9),
        "gained Heart 9 must not remain in discard"
    );
    assert_eq!(
        e.state_core().player(0).hand.len(),
        total_cards,
        "huangtianyili turns all remaining deck cards to Spade until deck/discard exhausts without panic"
    );

    // 1b. `luoshen` + `huangtianzhizu` (overrides to Spade 5): Diamond 4 passes as black!
    let mut ez = start_versus(454, &[&[0], &[1]]);
    grant_skill(&mut ez, 0, LUO_SHEN);
    grant_skill(&mut ez, 0, "_buff_txhj_huangtianzhizu");
    let dia4 = top_deck_suit_rank(&mut ez, Suit::Diamond, 4);
    ez.debug_set_turn(1, Phase::End, 0);
    let _ = next_phase(&mut ez);
    let ask_z = expect_pending(&ez, "trigger_confirm");
    let _ = answer(&mut ez, &ask_z, vec![ChoiceOption::Confirm]);
    drain(&mut ez);
    assert!(
        ez.state_core().player(0).hand.contains(&dia4),
        "luoshen + huangtianzhizu must pass Diamond 4 as Spade 5 and gain it into hand"
    );

    // 2. `luoshen` + `huangtianzhimin` (no `huangtianyili`): failed red judge enters discard and draws 1!
    let mut er = start_versus(441, &[&[0], &[1]]);
    grant_skill(&mut er, 0, LUO_SHEN);
    grant_skill(&mut er, 0, "_buff_txhj_huangtianzhimin");
    let red_card = top_deck_suit(&mut er, Suit::Heart);
    er.debug_set_turn(1, Phase::End, 0);
    let _ = next_phase(&mut er);
    let ask_r = expect_pending(&er, "trigger_confirm");
    let _ = answer(&mut er, &ask_r, vec![ChoiceOption::Confirm]);
    drain(&mut er);
    assert!(
        er.state_core().discard.contains(&red_card),
        "failed red luoshen judge card enters discard"
    );
    assert!(
        !er.state_core().player(0).hand.contains(&red_card),
        "failed red luoshen judge card is not gained directly"
    );
    assert_eq!(
        er.state_core().player(0).hand.len(),
        1,
        "buff_txhj_huangtianzhimin draws 1 when failed red luoshen judge enters discard"
    );

    // 3. `luoshen` + `huangtianzhimin` with black followed by red:
    //    black card goes to hand (does NOT trigger zhimin); red card goes to discard -> zhimin draws 1!
    let mut eb = start_versus(442, &[&[0], &[1]]);
    grant_skill(&mut eb, 0, LUO_SHEN);
    grant_skill(&mut eb, 0, "_buff_txhj_huangtianzhimin");
    let red_stop = top_deck_suit(&mut eb, Suit::Diamond);
    let black_win = top_deck_suit(&mut eb, Suit::Spade);
    eb.debug_set_turn(1, Phase::End, 0);
    let _ = next_phase(&mut eb);
    let ask_b = expect_pending(&eb, "trigger_confirm");
    let _ = answer(&mut eb, &ask_b, vec![ChoiceOption::Confirm]);
    drain(&mut eb);
    assert!(
        eb.state_core().player(0).hand.contains(&black_win),
        "black luoshen judge is gained into hand"
    );
    assert!(eb.state_core().discard.contains(&red_stop));
    assert_eq!(
        eb.state_core().player(0).hand.len(),
        2,
        "1 black gained + 1 drawn by huangtianzhimin on red stop (not 2 draws)"
    );
}

#[test]
fn l11_b4_r3_zj_leiji_uses_target_for_huangtian_and_zhimin() {
    // 1. Owner has `huangtianyili` + `huangtianzhimin`, target does NOT:
    //    target judges (mode/boss.js:7992 `event.target.judge(...)`), so Heart stays red -> no dmg, no draw!
    let mut eo = start_versus(443, &[&[0], &[1]]);
    grant_skill(&mut eo, 0, BOSS_ZJLEIJI);
    grant_skill(&mut eo, 0, "_buff_txhj_huangtianyili");
    grant_skill(&mut eo, 0, "_buff_txhj_huangtianzhimin");
    let sha_o = place_kind(&mut eo, 1, CardKind::Sha);
    let shan_o = place_kind(&mut eo, 0, CardKind::Shan);
    let _heart = top_deck_suit(&mut eo, Suit::Heart);
    eo.debug_set_turn(1, Phase::Play, 1);
    let _ = play_card(&mut eo, 1, sha_o, vec![0]);
    let resp_o = expect_pending(&eo, "respond_shan");
    let _ = answer(&mut eo, &resp_o, vec![ChoiceOption::Card(shan_o)]);
    drain(&mut eo);
    assert_eq!(
        eo.state_core().player(1).hp,
        6,
        "owner's huangtianyili must NOT turn target's zj_leiji judge black"
    );
    assert_eq!(
        eo.state_core().player(0).hand.len(),
        0,
        "owner must NOT draw from zj_leiji or target's judge zhimin when Heart stays red"
    );

    // 2. Target has `huangtianyili` + `huangtianzhimin`, owner has `zj_leiji` only:
    //    target's Heart is overridden to Spade -> 1 thunder dmg, owner draws 1, target draws 1 via zhimin!
    let mut et = start_versus(444, &[&[0], &[1]]);
    grant_skill(&mut et, 0, BOSS_ZJLEIJI);
    grant_skill(&mut et, 1, "_buff_txhj_huangtianyili");
    grant_skill(&mut et, 1, "_buff_txhj_huangtianzhimin");
    let sha_t = place_kind(&mut et, 1, CardKind::Sha);
    let shan_t = place_kind(&mut et, 0, CardKind::Shan);
    let _heart_t = top_deck_suit(&mut et, Suit::Heart);
    et.debug_set_turn(1, Phase::Play, 1);
    let _ = play_card(&mut et, 1, sha_t, vec![0]);
    let resp_t = expect_pending(&et, "respond_shan");
    let _ = answer(&mut et, &resp_t, vec![ChoiceOption::Card(shan_t)]);
    drain(&mut et);
    assert_eq!(
        et.state_core().player(1).hp,
        5,
        "target's huangtianyili turns Heart -> Spade in zj_leiji -> 1 thunder damage"
    );
    assert_eq!(
        et.state_core().player(0).hand.len(),
        1,
        "zj_leiji owner draws 1 on black hit"
    );
    assert_eq!(
        et.state_core().player(1).hand.len(),
        1,
        "target draws 1 via huangtianzhimin when zj_leiji judge card enters discard"
    );
}

#[test]
fn l11_b4_r3_tianxian_uses_target_for_huangtian_and_zhimin() {
    // 1. Source has `tianxian` + `huangtianyili`, target does NOT:
    //    `tianxian` calls `trigger.target.judge(...)`, so Heart stays red -> Sha resolves and hits!
    let mut es = start_versus_with_xianjian(445, &[&[0], &[1]]);
    grant_skill(&mut es, 0, "tianxian");
    grant_skill(&mut es, 0, "_buff_txhj_huangtianyili");
    let sha_s = place_kind(&mut es, 0, CardKind::Sha);
    let _heart_s = top_deck_suit(&mut es, Suit::Heart);
    es.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut es, 0, sha_s, vec![1]);
    drain(&mut es);
    assert_eq!(
        es.state_core().player(1).hp,
        5,
        "source's huangtianyili must NOT cancel own Sha via target-judged tianxian"
    );

    // 2. Target has `huangtianyili` + `huangtianzhimin`, source has `tianxian`:
    //    Heart -> Spade cancels Sha AND target draws 1 via huangtianzhimin!
    let mut et = start_versus_with_xianjian(446, &[&[0], &[1]]);
    grant_skill(&mut et, 0, "tianxian");
    grant_skill(&mut et, 1, "_buff_txhj_huangtianyili");
    grant_skill(&mut et, 1, "_buff_txhj_huangtianzhimin");
    let sha_t = place_kind(&mut et, 0, CardKind::Sha);
    let _heart_t = top_deck_suit(&mut et, Suit::Heart);
    et.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut et, 0, sha_t, vec![1]);
    drain(&mut et);
    assert_eq!(
        et.state_core().player(1).hp,
        6,
        "target's huangtianyili turns tianxian Heart -> Spade -> Sha cancelled"
    );
    assert_eq!(
        et.state_core().player(1).hand.len(),
        1,
        "target draws 1 via huangtianzhimin on settled tianxian judge"
    );

    // 3. Target has `huangtianzhimin` (red judge -> Sha continues): target draws BEFORE `respond_shan`!
    let mut ed = start_versus_with_xianjian(447, &[&[0], &[1]]);
    grant_skill(&mut ed, 0, "tianxian");
    grant_skill(&mut ed, 1, "_buff_txhj_huangtianzhimin");
    let sha_d = place_kind(&mut ed, 0, CardKind::Sha);
    // Stack deck from bottom-of-top up: 2nd card drawn = Shan, 1st card drawn = Heart judge.
    let drawn_shan = ed
        .state_core()
        .deck
        .iter()
        .copied()
        .find(|&c| ed.catalog().cards[c as usize].kind == CardKind::Shan)
        .expect("shan in deck");
    ed.state_core_mut().deck.retain(|&c| c != drawn_shan);
    ed.state_core_mut().deck.push(drawn_shan);
    let _heart_d = top_deck_suit(&mut ed, Suit::Heart);
    ed.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut ed, 0, sha_d, vec![1]);
    let ask_shan = expect_pending(&ed, "respond_shan");
    assert_eq!(ask_shan.observer, 1);
    assert!(
        ask_shan.spec.options.contains(&ChoiceOption::Card(drawn_shan)),
        "target must draw Shan via huangtianzhimin BEFORE respond_shan opens: options={:?}",
        ask_shan.spec.options
    );
    let _ = answer(&mut ed, &ask_shan, vec![ChoiceOption::Card(drawn_shan)]);
    drain(&mut ed);
    assert_eq!(ed.state_core().player(1).hp, 6, "dodged with zhimin-drawn Shan");
}

#[test]
fn l11_b4_r3_on_judge_settled_ganglie_tieji_and_other_inline_paths() {
    // 1. `gang_lie` + `huangtianzhimin` (probe-l11-4-v2 scenario): Heart judge -> no dmg, owner draws 1!
    let mut eg = start_versus(448, &[&[0], &[1]]);
    grant_skill(&mut eg, 0, GANG_LIE);
    grant_skill(&mut eg, 0, "_buff_txhj_huangtianzhimin");
    let sha_g = place_kind(&mut eg, 1, CardKind::Sha);
    let _heart_g = top_deck_suit(&mut eg, Suit::Heart);
    eg.debug_set_turn(1, Phase::Play, 1);
    let _ = play_card(&mut eg, 1, sha_g, vec![0]);
    let gl_confirm = expect_pending(&eg, "trigger_confirm");
    assert_eq!(gl_confirm.observer, 0);
    let _ = answer(&mut eg, &gl_confirm, vec![ChoiceOption::Confirm]);
    drain(&mut eg);
    assert_eq!(eg.state_core().player(1).hp, 6, "Heart ganglie deals no dmg");
    assert_eq!(
        eg.state_core().player(0).hand.len(),
        1,
        "ganglie Heart judge enters discard -> huangtianzhimin draws 1"
    );

    // 2. `gang_lie` failed Heart check with >=2 source hand cards:
    //    `huangtianzhimin` draws 1 for owner BEFORE `discard_to_limit` opens, and save/restore matches!
    let mut eg2 = start_versus(449, &[&[0], &[1]]);
    grant_skill(&mut eg2, 0, GANG_LIE);
    grant_skill(&mut eg2, 0, "_buff_txhj_huangtianzhimin");
    let _h1 = place_kind(&mut eg2, 1, CardKind::Tao);
    let _h2 = place_kind(&mut eg2, 1, CardKind::Tao);
    let sha_g2 = place_kind(&mut eg2, 1, CardKind::Sha);
    let _spade_g2 = top_deck_suit(&mut eg2, Suit::Spade);
    eg2.debug_set_turn(1, Phase::Play, 1);
    let _ = play_card(&mut eg2, 1, sha_g2, vec![0]);
    let gl2_confirm = expect_pending(&eg2, "trigger_confirm");
    let _ = answer(&mut eg2, &gl2_confirm, vec![ChoiceOption::Confirm]);
    let disc_ask = expect_pending(&eg2, "discard_to_limit");
    assert_eq!(disc_ask.observer, 1);
    assert_eq!(
        eg2.state_core().player(0).hand.len(),
        1,
        "ganglie Spade judge triggers huangtianzhimin draw BEFORE discard_to_limit opens"
    );
    let eg2 = save_restore_and_compare_pending(&eg2, &disc_ask, Vec::new());
    assert_eq!(eg2.state_core().player(1).hp, 5, "cancelling discard_to_limit deals 1 ganglie dmg");

    // 3. `tieji` + `huangtianzhimin`: both red hit and black miss draw 1 BEFORE `respond_shan`!
    let mut et_red = start_versus(450, &[&[0], &[1]]);
    grant_skill(&mut et_red, 0, TIE_JI);
    grant_skill(&mut et_red, 0, "_buff_txhj_huangtianzhimin");
    let _shan_1 = place_kind(&mut et_red, 1, CardKind::Shan);
    let sha_tr = place_kind(&mut et_red, 0, CardKind::Sha);
    let _heart_tr = top_deck_suit(&mut et_red, Suit::Heart);
    et_red.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut et_red, 0, sha_tr, vec![1]);
    drain(&mut et_red);
    assert_eq!(et_red.state_core().player(1).hp, 5, "red tieji blocks Shan");
    assert_eq!(
        et_red.state_core().player(0).hand.len(),
        1,
        "tieji red judge enters discard -> huangtianzhimin draws 1"
    );

    let mut et_blk = start_versus(451, &[&[0], &[1]]);
    grant_skill(&mut et_blk, 0, TIE_JI);
    grant_skill(&mut et_blk, 0, "_buff_txhj_huangtianzhimin");
    let _shan_2 = place_kind(&mut et_blk, 1, CardKind::Shan);
    let sha_tb = place_kind(&mut et_blk, 0, CardKind::Sha);
    let _spade_tb = top_deck_suit(&mut et_blk, Suit::Spade);
    et_blk.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut et_blk, 0, sha_tb, vec![1]);
    let ask_tb = expect_pending(&et_blk, "respond_shan");
    assert_eq!(
        et_blk.state_core().player(0).hand.len(),
        1,
        "tieji black judge draws via huangtianzhimin BEFORE respond_shan opens"
    );
    let _ = answer(&mut et_blk, &ask_tb, vec![ChoiceOption::Pass]);
    drain(&mut et_blk);

    // 4. Extra inline `draw_judge_card` + `to_discard` paths:
    //    (a) `BOSS_ZHONGDU` (`phase/judge.rs`) respects `huangtianyili` + `huangtianzhimin`.
    let mut ezd = start_versus(452, &[&[0], &[1]]);
    grant_skill(&mut ezd, 0, BOSS_ZHONGDU);
    grant_skill(&mut ezd, 0, "_buff_txhj_huangtianyili");
    grant_skill(&mut ezd, 0, "_buff_txhj_huangtianzhimin");
    let _heart_zd = top_deck_suit(&mut ezd, Suit::Heart);
    ezd.debug_set_turn(1, Phase::End, 0);
    let _ = next_phase(&mut ezd); // 1:End -> 0:Prepare (runs begin_prepare -> boss_zhongdu)
    drain(&mut ezd);
    assert_eq!(
        ezd.state_core().player(0).hp,
        6,
        "huangtianyili turns zhongdu Heart -> Spade (no damage)"
    );
    assert!(has_skill(&ezd, 0, BOSS_ZHONGDU), "Spade keeps BOSS_ZHONGDU");
    assert_eq!(
        ezd.state_core().player(0).hand.len(),
        1,
        "boss_zhongdu judge in discard triggers huangtianzhimin"
    );

    //    (b) `linglongshimandai_skill` (`l28_shengong_equips.rs`) respects `huangtianzhimin`.
    let mut ell = start_versus_with_l28_wudian(453, &[&[0], &[1]]);
    grant_skill(&mut ell, 1, "_buff_txhj_huangtianzhimin");
    let _ll = equip_catalog_named(&mut ell, 1, "linglongshimandai");
    let sha_ll = place_kind(&mut ell, 0, CardKind::Sha);
    let _heart_ll = top_deck_suit(&mut ell, Suit::Heart);
    ell.debug_set_turn(0, Phase::Play, 1);
    let _ = play_card(&mut ell, 0, sha_ll, vec![1]);
    let ll_ask = expect_pending(&ell, "linglongshimandai:confirm");
    let _ = answer(&mut ell, &ll_ask, vec![ChoiceOption::Confirm]);
    drain(&mut ell);
    assert_eq!(ell.state_core().player(1).hp, 6, "Heart linglong cancels Sha");
    assert_eq!(
        ell.state_core().player(1).hand.len(),
        1,
        "linglongshimandai judge card in discard triggers huangtianzhimin"
    );
}
