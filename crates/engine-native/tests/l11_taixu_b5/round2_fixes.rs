use super::common::*;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::{ChoiceOption, CommandOutcome, Engine, Phase};

fn place_from_any_zone_by_name(e: &mut Engine, player: u8, name: &str) -> u32 {
    let cid = e
        .state_core()
        .deck
        .iter()
        .chain(e.state_core().discard.iter())
        .copied()
        .chain(e.state_core().detached_cards.iter().map(|&(c, _)| c))
        .find(|&c| e.catalog().cards[c as usize].name == name)
        .unwrap_or_else(|| panic!("no card named {name} in deck/discard/detached"));
    remove_from_piles_to_hand(e, player, cid);
    cid
}

fn equip_from_any_zone_by_name(e: &mut Engine, actor: u8, name: &str) -> u32 {
    let prev = (e.turn().current, e.turn().phase, e.turn().round);
    if prev.0 != actor || prev.1 != Phase::Play {
        e.debug_set_turn(actor, Phase::Play, prev.2);
    }
    let cid = place_from_any_zone_by_name(e, actor, name);
    let out = play_card(e, actor, cid, Vec::new());
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "equip named {name} rejected: {out:?}"
    );
    drain(e);
    if e.turn().current != prev.0 || e.turn().phase != prev.1 || e.turn().round != prev.2 {
        e.debug_set_turn(prev.0, prev.1, prev.2);
    }
    cid
}

/// Finding 1 (`extension_skill.js:299-350`): `_txhj_equip5` converts all 11 pool cards
/// (`taipingyaoshu`, 8 standard horses, `txhj_liulongcanjia`, `txhj_juechenjinge`) to `EquipSlot::Treasure`,
/// caps the whole pool at 2 equips (discarding oldest `[0]` on 3rd), and exposes them to `qilin_skill`.
#[test]
fn test_r2_f1_txhj_equip5_pool_and_qilin() {
    // Positive: `_txhj_equip5` maps horses/liulong/juechen/taipingyaoshu to Treasure and caps at 2.
    let mut e = start_versus_with_l28(0xB5_2001, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "_txhj_equip5");
    e.debug_set_turn(0, Phase::Play, 1);

    let horse_minus = equip_std_kind(&mut e, 0, CardKind::HorseMinus);
    let juechen = equip_named(&mut e, 0, "txhj_juechenjinge");
    assert_eq!(
        e.catalog().card_slot_of(e.state_core(), horse_minus),
        Some(EquipSlot::Treasure),
        "standard HorseMinus maps to Treasure when _txhj_equip5 is active"
    );
    assert_eq!(
        e.catalog().card_slot_of(e.state_core(), juechen),
        Some(EquipSlot::Treasure),
        "txhj_juechenjinge maps to Treasure when _txhj_equip5 is active"
    );
    assert_eq!(
        e.state_core().player(0).equips,
        vec![horse_minus, juechen],
        "2 equip5 cards coexist in Treasure pool"
    );

    // 3rd equip5 (`txhj_dinglanyemingzhu`) replaces oldest (`horse_minus`), capping pool at 2:
    let dinglan = equip_named(&mut e, 0, "txhj_dinglanyemingzhu");
    assert_eq!(
        e.state_core().player(0).equips,
        vec![juechen, dinglan],
        "3rd equip5 replaces oldest horse_minus"
    );

    // `th_taipingyaoshu` also shares the equip5 pool (replaces oldest `juechen`, cap 2):
    let taiping = equip_from_any_zone_by_name(&mut e, 0, "th_taipingyaoshu");
    assert_eq!(
        e.catalog().card_slot_of(e.state_core(), taiping),
        Some(EquipSlot::Treasure),
        "taipingyaoshu maps to Treasure under _txhj_equip5"
    );
    assert_eq!(
        e.state_core().player(0).equips,
        vec![dinglan, taiping],
        "taipingyaoshu shares 2-slot equip5 cap"
    );

    // `txhj_liulongcanjia` coexists in equip5 with `horse_plus`, while `txhj_baihu` stays HorsePlus & is blocked:
    let mut e_ll = start_versus(0xB5_2002, &[&[0], &[1]]);
    grant_skill(&mut e_ll, 0, "_txhj_equip5");
    e_ll.debug_set_turn(0, Phase::Play, 1);
    let liulong = equip_named(&mut e_ll, 0, "txhj_liulongcanjia");
    let hp_std = equip_std_kind(&mut e_ll, 0, CardKind::HorsePlus);
    assert_eq!(
        e_ll.catalog().card_slot_of(e_ll.state_core(), liulong),
        Some(EquipSlot::Treasure)
    );
    assert_eq!(
        e_ll.catalog().card_slot_of(e_ll.state_core(), hp_std),
        Some(EquipSlot::Treasure)
    );
    assert_eq!(
        e_ll.state_core().player(0).equips,
        vec![liulong, hp_std],
        "txhj_liulongcanjia + standard HorsePlus share 2-slot equip5"
    );
    let baihu = place_named(&mut e_ll, 0, "txhj_baihu");
    assert_eq!(
        e_ll.catalog().card_slot_of(e_ll.state_core(), baihu),
        Some(EquipSlot::HorsePlus),
        "txhj_baihu is NOT in _txhj_equip5 list and stays HorsePlus"
    );
    assert!(
        matches!(play_card(&mut e_ll, 0, baihu, Vec::new()), CommandOutcome::Rejected(_)),
        "txhj_liulongcanjia still blocks HorsePlus card txhj_baihu"
    );

    // `qilin_skill` targets converted equip5 (`txhj_juechenjinge` + `horse_minus`) on damage:
    let mut e_ql = start_versus_with_l28(0xB5_2003, &[&[0], &[1]]);
    grant_skill(&mut e_ql, 0, "_txhj_equip5");
    grant_skill(&mut e_ql, 1, "qilin_skill");
    let _range_wpn = equip_kind(&mut e_ql, 1, CardKind::Fangtian);
    let hm_q = equip_std_kind(&mut e_ql, 0, CardKind::HorseMinus);
    let jc_q = equip_named(&mut e_ql, 0, "txhj_juechenjinge");
    e_ql.debug_set_turn(1, Phase::Play, 1);
    let sha = place_kind(&mut e_ql, 1, CardKind::Sha);
    let sha_out = play_card(&mut e_ql, 1, sha, vec![0]);
    assert!(
        matches!(
            sha_out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "unexpected play_card outcome for qilin Sha: {sha_out:?}"
    );
    let q = expect_pending(&e_ql, "qilin_skill:discard_horse");
    assert!(
        q.spec.options.contains(&ChoiceOption::Card(hm_q))
            && q.spec.options.contains(&ChoiceOption::Card(jc_q)),
        "qilin_skill must target converted equip5 horses: {:?}",
        q.spec.options
    );
    let _ = answer(&mut e_ql, &q, vec![ChoiceOption::Card(jc_q)]);
    drain(&mut e_ql);
    assert!(!e_ql.state_core().player(0).equips.contains(&jc_q));
    assert!(e_ql.state_core().player(0).equips.contains(&hm_q));

    // Negative: without `_txhj_equip5`, standard HorsePlus & HorseMinus remain separate mount slots:
    let mut e_no = start_versus(0xB5_2004, &[&[0], &[1]]);
    let hp_n = equip_std_kind(&mut e_no, 0, CardKind::HorsePlus);
    let hm_n = equip_std_kind(&mut e_no, 0, CardKind::HorseMinus);
    assert_eq!(e_no.state_core().player(0).equips, vec![hp_n, hm_n]);
}

/// Finding 2 (`extension_skill.js:3627` & `1513-1628`): `txhj_mouduan` distinguishes `basic`/`trick`/`delay`/`equip`
/// (3 types across only 2 suits triggers), emits `Equipped` on `to` for `_txhj_shenqi`, and allows moving a 2nd
/// `equip5` onto `to` when `to` holds `_txhj_equip5` (rejecting when `to` already holds 2).
#[test]
fn test_r2_f2_txhj_mouduan_four_types_and_shenqi_move() {
    let mut e = start_versus(0xB5_2005, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "txhj_mouduan");
    grant_skill(&mut e, 0, "_txhj_equip5");
    grant_skill(&mut e, 0, "_txhj_shenqi");
    grant_skill(&mut e, 1, "_txhj_equip5");
    grant_skill(&mut e, 1, "_txhj_shenqi");
    grant_skill(&mut e, 1, "_txhj_XXXshenqi");
    grant_skill(&mut e, 1, "txhj_xuanlve");

    // Seat 0 already has 1 equip5 (`txhj_xuwangzhimian`), seat 1 has `txhj_juechenjinge` (shenqi -> `buff_txhj_wuzhongshengshan`).
    let xw = equip_named(&mut e, 0, "txhj_xuwangzhimian");
    let jc = equip_named(&mut e, 1, "txhj_juechenjinge");
    assert!(has_skill(&e, 1, "buff_txhj_wuzhongshengshan"));
    assert!(!has_skill(&e, 0, "buff_txhj_wuzhongshengshan"));
    let _dummy2 = place_kind(&mut e, 2, CardKind::Tao);

    // Play 3 cards using ONLY 2 suits (Spade & Club) but 3 card types: basic (Sha) + trick (Dismantle) + delay (Le)!
    e.debug_set_turn(0, Phase::Play, 1);
    let c_sha = place_kind_suit(&mut e, 0, CardKind::Sha, Suit::Spade);
    let c_dis = place_kind_suit(&mut e, 0, CardKind::Dismantle, Suit::Spade);
    let c_le = place_kind_suit(&mut e, 0, CardKind::Le, Suit::Club);
    let _ = play_card(&mut e, 0, c_sha, vec![2]);
    drain(&mut e);
    let _ = play_card(&mut e, 0, c_dis, vec![2]);
    drain(&mut e);
    let _ = play_card(&mut e, 0, c_le, vec![1]);
    drain(&mut e);

    advance_until_turn_phase(&mut e, 0, Phase::End);
    let mc = expect_pending(&e, "txhj_mouduan:confirm");
    let _ = answer(&mut e, &mc, vec![ChoiceOption::Confirm]);
    let mf = expect_pending(&e, "txhj_mouduan:move_from");
    assert!(mf.spec.options.contains(&ChoiceOption::Player(1)));
    let _ = answer(&mut e, &mf, vec![ChoiceOption::Player(1)]);
    let mk = expect_pending(&e, "txhj_mouduan:move_card");
    assert!(mk.spec.options.contains(&ChoiceOption::Card(jc)));
    let _ = answer(&mut e, &mk, vec![ChoiceOption::Card(jc)]);
    let mt = expect_pending(&e, "txhj_mouduan:move_to");
    assert!(
        mt.spec.options.contains(&ChoiceOption::Player(0)),
        "seat 0 with _txhj_equip5 and 1 equip5 must be eligible target for 2nd equip5: {:?}",
        mt.spec.options
    );
    let e_done = save_restore_and_compare_pending(&e, &mt, vec![ChoiceOption::Player(0)]);
    assert_eq!(e_done.state_core().player(0).equips, vec![xw, jc]);
    assert!(
        has_skill(&e_done, 0, "buff_txhj_wuzhongshengshan"),
        "_txhj_shenqi on target seat 0 must grant buff on moved equip via Equipped"
    );
    assert!(
        !has_skill(&e_done, 1, "buff_txhj_wuzhongshengshan"),
        "_txhj_XXXshenqi on source seat 1 must revoke buff on moved equip"
    );

    // Negative: when seat 0 already has 2 equip5s, moving a 3rd equip5 to seat 0 is disallowed:
    let mut e_full = start_versus(0xB5_2006, &[&[0], &[1, 2]]);
    grant_skill(&mut e_full, 0, "txhj_mouduan");
    grant_skill(&mut e_full, 0, "_txhj_equip5");
    grant_skill(&mut e_full, 1, "_txhj_equip5");
    let _e0a = equip_named(&mut e_full, 0, "txhj_xuwangzhimian");
    let _e0b = equip_named(&mut e_full, 0, "txhj_dinglanyemingzhu");
    let e1_eq = equip_named(&mut e_full, 1, "txhj_juechenjinge");
    let _d2 = place_kind(&mut e_full, 2, CardKind::Tao);
    e_full.debug_set_turn(0, Phase::Play, 1);
    let s1 = place_kind_suit(&mut e_full, 0, CardKind::Sha, Suit::Spade);
    let d1 = place_kind_suit(&mut e_full, 0, CardKind::Dismantle, Suit::Spade);
    let l1 = place_kind_suit(&mut e_full, 0, CardKind::Le, Suit::Club);
    let _ = play_card(&mut e_full, 0, s1, vec![2]);
    drain(&mut e_full);
    let _ = play_card(&mut e_full, 0, d1, vec![2]);
    drain(&mut e_full);
    let _ = play_card(&mut e_full, 0, l1, vec![1]);
    drain(&mut e_full);
    advance_until_turn_phase(&mut e_full, 0, Phase::End);
    let mc2 = expect_pending(&e_full, "txhj_mouduan:confirm");
    let _ = answer(&mut e_full, &mc2, vec![ChoiceOption::Confirm]);
    let mf2 = expect_pending(&e_full, "txhj_mouduan:move_from");
    let _ = answer(&mut e_full, &mf2, vec![ChoiceOption::Player(1)]);
    let mk2 = expect_pending(&e_full, "txhj_mouduan:move_card");
    let _ = answer(&mut e_full, &mk2, vec![ChoiceOption::Card(e1_eq)]);
    let mt2 = expect_pending(&e_full, "txhj_mouduan:move_to");
    assert!(
        !mt2.spec.options.contains(&ChoiceOption::Player(0)),
        "seat 0 with 2/2 equip5s must NOT receive a 3rd equip5"
    );
}

/// Finding 3 (`extension_skill.js:4338-4384`): `txhj_zhuwei` fires when `tiandu` declines
/// (and stays silent when `tiandu` gains), AND fires across all inline `draw_judge_card` paths
/// (`luoshen`, `ganglie`, `zj_leiji`, `bagua`, `tieji`, `tianxian`).
#[test]
fn test_r2_f3_txhj_zhuwei_tiandu_and_inline_judges() {
    // 3A-1: JudgePhaseMachine with BOTH `txhj_zhuwei` and `tiandu` -> decline `tiandu` -> `txhj_zhuwei` gains!
    let mut e = start_versus(0xB5_2007, &[&[0], &[1]]);
    grant_skill(&mut e, 0, "txhj_zhuwei");
    grant_skill(&mut e, 0, "天妒");
    let le = place_kind(&mut e, 0, CardKind::Le);
    e.state_core_mut().player_mut(0).hand.retain(|&c| c != le);
    e.state_core_mut().player_mut(0).judge.push(le);
    let j_decl = top_deck_suit(&mut e, Suit::Heart);
    e.debug_set_turn(0, Phase::Prepare, 1);
    let _ = next_phase(&mut e);
    let td = expect_pending(&e, "tian_du_gain");
    let _ = answer(&mut e, &td, Vec::new());
    let zg = expect_pending(&e, "txhj_zhuwei:confirm_gain");
    let _ = answer(&mut e, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e, "txhj_zhuwei:confirm_buff");
    let e_done = save_restore_and_compare_pending(&e, &zb, vec![ChoiceOption::Confirm]);
    assert!(
        e_done.state_core().player(0).hand.contains(&j_decl),
        "txhj_zhuwei must gain judge card after tiandu declines"
    );
    assert_eq!(flag(&e_done, 0, "skill_turn:l11_zhuwei_eff"), 1);

    // 3A-2: Accept `tiandu` -> `txhj_zhuwei` must NOT trigger (`open_choice().is_none()`).
    let mut e_acc = start_versus(0xB5_2008, &[&[0], &[1]]);
    grant_skill(&mut e_acc, 0, "txhj_zhuwei");
    grant_skill(&mut e_acc, 0, "天妒");
    let le2 = place_kind(&mut e_acc, 0, CardKind::Le);
    e_acc.state_core_mut().player_mut(0).hand.retain(|&c| c != le2);
    e_acc.state_core_mut().player_mut(0).judge.push(le2);
    let j_acc = top_deck_suit(&mut e_acc, Suit::Heart);
    e_acc.debug_set_turn(0, Phase::Prepare, 1);
    let _ = next_phase(&mut e_acc);
    let td2 = expect_pending(&e_acc, "tian_du_gain");
    let _ = answer(&mut e_acc, &td2, vec![ChoiceOption::Confirm]);
    assert!(
        e_acc.open_choice().is_none(),
        "when tiandu gains judge card, txhj_zhuwei must not open"
    );
    assert!(e_acc.state_core().player(0).hand.contains(&j_acc));
    assert_eq!(flag(&e_acc, 0, "skill_turn:l11_zhuwei_eff"), 0);

    // 3B-1: `luoshen` inline judge (red card enters discard -> `txhj_zhuwei` gains it):
    let mut e_ls = start_versus(0xB5_2009, &[&[0], &[1]]);
    grant_skill(&mut e_ls, 0, "洛神");
    grant_skill(&mut e_ls, 0, "txhj_zhuwei");
    let j_ls = top_deck_suit(&mut e_ls, Suit::Heart);
    e_ls.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_ls); // -> seat 0 Prepare -> triggers luo_shen
    let ls_c = expect_pending(&e_ls, "trigger_confirm");
    let _ = answer(&mut e_ls, &ls_c, vec![ChoiceOption::Confirm]);
    let zg = expect_pending(&e_ls, "txhj_zhuwei:confirm_gain");
    let _ = answer(&mut e_ls, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e_ls, "txhj_zhuwei:confirm_buff");
    let _ = answer(&mut e_ls, &zb, vec![ChoiceOption::Confirm]);
    assert!(e_ls.state_core().player(0).hand.contains(&j_ls));
    assert_eq!(flag(&e_ls, 0, "skill_turn:l11_zhuwei_eff"), 1);

    // 3B-2: `ganglie` inline judge -> `txhj_zhuwei` gains judge card:
    let mut e_gl = start_versus(0xB5_200A, &[&[0], &[1]]);
    grant_skill(&mut e_gl, 0, "刚烈");
    grant_skill(&mut e_gl, 0, "txhj_zhuwei");
    let j_gl = top_deck_suit(&mut e_gl, Suit::Heart);
    e_gl.debug_set_turn(1, Phase::Play, 1);
    let sha_gl = place_kind(&mut e_gl, 1, CardKind::Sha);
    let _ = play_card(&mut e_gl, 1, sha_gl, vec![0]);
    let gl_c = expect_pending(&e_gl, "trigger_confirm");
    let _ = answer(&mut e_gl, &gl_c, vec![ChoiceOption::Confirm]);
    let zg = expect_pending(&e_gl, "txhj_zhuwei:confirm_gain");
    let _ = answer(&mut e_gl, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e_gl, "txhj_zhuwei:confirm_buff");
    let _ = answer(&mut e_gl, &zb, vec![ChoiceOption::Confirm]);
    assert!(e_gl.state_core().player(0).hand.contains(&j_gl));
    assert_eq!(flag(&e_gl, 1, "skill_turn:l11_zhuwei_eff"), 1);

    // 3B-3: `zj_leiji` inline judge (`boss_leiji` `event.target.judge` -> target `seat 1`) -> `txhj_zhuwei` gains judge card:
    let mut e_zj = start_versus(0xB5_200B, &[&[0], &[1]]);
    grant_skill(&mut e_zj, 0, "雷击·张角");
    grant_skill(&mut e_zj, 1, "txhj_zhuwei");
    let shan_zj = place_kind(&mut e_zj, 0, CardKind::Shan);
    let j_zj = top_deck_suit(&mut e_zj, Suit::Heart);
    e_zj.debug_set_turn(1, Phase::Play, 1);
    let sha_zj = place_kind(&mut e_zj, 1, CardKind::Sha);
    let _ = play_card(&mut e_zj, 1, sha_zj, vec![0]);
    let rs = expect_pending(&e_zj, "respond_shan");
    let _ = answer(&mut e_zj, &rs, vec![ChoiceOption::Card(shan_zj)]);
    let zg = expect_pending(&e_zj, "txhj_zhuwei:confirm_gain");
    assert_eq!(zg.observer, 1);
    let _ = answer(&mut e_zj, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e_zj, "txhj_zhuwei:confirm_buff");
    let _ = answer(&mut e_zj, &zb, vec![ChoiceOption::Confirm]);
    assert!(e_zj.state_core().player(1).hand.contains(&j_zj));
    assert_eq!(flag(&e_zj, 1, "skill_turn:l11_zhuwei_eff"), 1);

    // 3B-4: `bagua` inline judge -> `txhj_zhuwei` gains judge card:
    let mut e_bg = start_versus(0xB5_200C, &[&[0], &[1]]);
    grant_skill(&mut e_bg, 0, "txhj_zhuwei");
    let _bg = equip_std_kind(&mut e_bg, 0, CardKind::Armor);
    let j_bg = top_deck_suit(&mut e_bg, Suit::Heart);
    e_bg.debug_set_turn(1, Phase::Play, 1);
    let sha_bg = place_kind(&mut e_bg, 1, CardKind::Sha);
    let _ = play_card(&mut e_bg, 1, sha_bg, vec![0]);
    let zg = expect_pending(&e_bg, "txhj_zhuwei:confirm_gain");
    let _ = answer(&mut e_bg, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e_bg, "txhj_zhuwei:confirm_buff");
    let _ = answer(&mut e_bg, &zb, vec![ChoiceOption::Confirm]);
    assert!(e_bg.state_core().player(0).hand.contains(&j_bg));

    // 3B-5: `tieji` inline judge -> `txhj_zhuwei` gains judge card:
    let mut e_tj = start_versus(0xB5_200D, &[&[0], &[1]]);
    grant_skill(&mut e_tj, 0, "铁骑");
    grant_skill(&mut e_tj, 0, "txhj_zhuwei");
    let j_tj = top_deck_suit(&mut e_tj, Suit::Club);
    e_tj.debug_set_turn(0, Phase::Play, 1);
    let sha_tj = place_kind(&mut e_tj, 0, CardKind::Sha);
    let _ = play_card(&mut e_tj, 0, sha_tj, vec![1]);
    let zg = expect_pending(&e_tj, "txhj_zhuwei:confirm_gain");
    let _ = answer(&mut e_tj, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e_tj, "txhj_zhuwei:confirm_buff");
    let _ = answer(&mut e_tj, &zb, vec![ChoiceOption::Confirm]);
    drain(&mut e_tj);
    assert!(e_tj.state_core().player(0).hand.contains(&j_tj));
    assert_eq!(flag(&e_tj, 0, "skill_turn:l11_zhuwei_eff"), 1);

    // 3B-6: `tianxian` inline judge (`owner` is target `seat 1`) -> `txhj_zhuwei` gains judge card:
    let mut e_tx = start_versus_with_xianjian(0xB5_200E, &[&[0], &[1]]);
    grant_skill(&mut e_tx, 0, "tianxian");
    grant_skill(&mut e_tx, 1, "txhj_zhuwei");
    let j_tx = top_deck_suit(&mut e_tx, Suit::Heart);
    e_tx.debug_set_turn(0, Phase::Play, 1);
    let sha_tx = place_kind(&mut e_tx, 0, CardKind::Sha);
    let _ = play_card(&mut e_tx, 0, sha_tx, vec![1]);
    let zg = expect_pending(&e_tx, "txhj_zhuwei:confirm_gain");
    assert_eq!(zg.observer, 1);
    let _ = answer(&mut e_tx, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&e_tx, "txhj_zhuwei:confirm_buff");
    let _ = answer(&mut e_tx, &zb, vec![ChoiceOption::Confirm]);
    drain(&mut e_tx);
    assert!(e_tx.state_core().player(1).hand.contains(&j_tx));
    assert_eq!(flag(&e_tx, 0, "skill_turn:l11_zhuwei_eff"), 1);
}

/// Finding 4 (`extension_skill.js:4001, 4053`): out-of-turn `CardUsed` (e.g. `Tao` in `dying.rs:214`)
/// must trigger `boss_moyany` and `boss_danshu`, while in-turn `CardUsed` must NOT trigger either.
#[test]
fn test_r2_f4_boss_moyany_danshu_out_of_turn_card_used() {
    let mut e = start_versus(0xB5_200F, &[&[0, 2], &[1]]);
    grant_skill(&mut e, 0, "boss_moyany");
    grant_skill(&mut e, 0, "boss_danshu");
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 1);
    let tao = place_kind(&mut e, 0, CardKind::Tao);
    e.debug_set_turn(1, Phase::Play, 1);
    top_deck_suit(&mut e, Suit::Heart); // Red for boss_danshu
    top_deck_suit(&mut e, Suit::Diamond); // Red for boss_moyany (popped first)
    let sha = place_kind(&mut e, 1, CardKind::Sha);
    let _ = play_card(&mut e, 1, sha, vec![2]);
    let ds = expect_pending(&e, "dying_save");
    assert_eq!(ds.observer, 0);
    let _ = answer(&mut e, &ds, vec![ChoiceOption::Card(tao)]);

    // Out-of-turn `CardUsed(tao)` triggers `boss_moyany` + `boss_danshu` once each:
    let cm = expect_pending(&e, "boss_moyany:confirm");
    let _ = answer(&mut e, &cm, vec![ChoiceOption::Confirm]);
    let cmt = expect_pending(&e, "boss_moyany:target");
    let _ = answer(&mut e, &cmt, vec![ChoiceOption::Player(1)]);
    let cd = expect_pending(&e, "boss_danshu:confirm");
    let _ = answer(&mut e, &cd, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert_eq!(e.state_core().player(2).hp, 1, "ally saved by out-of-turn Tao");
    assert_eq!(e.state_core().player(1).hp, 4, "boss_moyany dealt 2 fire damage to seat 1");
    assert_eq!(e.state_core().player(0).hp, 5, "boss_danshu healed seat 0 (4 -> 5)");

    // Negative: in-turn `CardUsed(tao)` by seat 0 during seat 0's own Play phase must NOT trigger:
    e.debug_set_turn(0, Phase::Play, 2);
    let tao_in = place_kind(&mut e, 0, CardKind::Tao);
    let _ = play_card(&mut e, 0, tao_in, Vec::new());
    drain(&mut e);
    assert_eq!(e.state_core().player(0).hp, 6);
    assert_eq!(e.state_core().player(1).hp, 4, "in-turn Tao must NOT trigger boss_moyany");
}

/// Finding 5 (`extension_skill.js:785-792`): under L11 Taixu context, `wushuangfangtianji_skill`
/// triggers on chained Sha damage (`d.from_chain == true`).
#[test]
fn test_r2_f5_wushuangfangtianji_chained_sha_damage() {
    let mut e = start_versus_with_l28(0xB5_2010, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "wushuangfangtianji_skill");
    e.state_core_mut().player_mut(1).linked = true;
    e.state_core_mut().player_mut(2).linked = true;
    let c1 = place_kind(&mut e, 1, CardKind::Tao);
    let c2 = place_kind(&mut e, 2, CardKind::Jiu);
    e.debug_set_turn(0, Phase::Play, 1);
    let fire_sha = place_kind(&mut e, 0, CardKind::FireSha);
    let _ = play_card(&mut e, 0, fire_sha, vec![1]);

    // Primary Sha hit on seat 1:
    let m1 = expect_pending(&e, "wushuangfangtianji:choose_mode");
    let _ = answer(&mut e, &m1, vec![ChoiceOption::Tag(1)]);
    let d1 = expect_pending(&e, "wushuangfangtianji:discard_card");
    let _ = answer(&mut e, &d1, vec![ChoiceOption::Card(c1)]);

    // Chained Sha damage (`d.from_chain == true`) on seat 2 MUST ALSO trigger:
    let m2 = expect_pending(&e, "wushuangfangtianji:choose_mode");
    let _ = answer(&mut e, &m2, vec![ChoiceOption::Tag(1)]);
    let d2 = expect_pending(&e, "wushuangfangtianji:discard_card");
    let _ = answer(&mut e, &d2, vec![ChoiceOption::Card(c2)]);
    drain(&mut e);
    assert!(e.state_core().player(1).hand.is_empty());
    assert!(e.state_core().player(2).hand.is_empty());
    assert_eq!(e.state_core().player(1).hp, 5);
    assert_eq!(e.state_core().player(2).hp, 5);
}

/// Finding 6 (`extension_skill.js:3931-3969, 4248-4263`):
/// - `boss_yushou` & `boss_mojian` filters/targets exclude `guofengyupao` via `eval_card_target_canceled`
/// - `boss_fengxing` `sha_target_valid` excludes canceled targets (e.g. unarmored `txmanjia`)
/// - `ShaResolveMachine` honors `eval_card_target_canceled` on virtual `Sha`.
#[test]
fn test_r2_f6_fengxing_yushou_mojian_target_cancel() {
    // 6A-1: Single enemy with `guofengyupao` -> `boss_yushou` & `boss_mojian` must NOT prompt at Play start:
    let mut e1 = start_versus_with_l28(0xB5_2011, &[&[0], &[1]]);
    grant_skill(&mut e1, 0, "boss_yushou");
    grant_skill(&mut e1, 0, "boss_mojian");
    grant_skill(&mut e1, 1, "guofengyupao");
    e1.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e1); // -> Play
    assert!(
        e1.open_choice().is_none(),
        "boss_yushou/boss_mojian must not prompt when all enemies hold guofengyupao"
    );
    assert_eq!(e1.state_core().player(1).hp, 6);

    // 6A-2: Two enemies `[1, 2]` where 1 holds `guofengyupao` and 2 does not -> hits ONLY 2:
    let mut e2 = start_versus_with_l28(0xB5_2012, &[&[0], &[1, 2]]);
    grant_skill(&mut e2, 0, "boss_yushou");
    grant_skill(&mut e2, 0, "boss_mojian");
    grant_skill(&mut e2, 1, "guofengyupao");
    e2.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e2); // -> Play
    let c_ys = expect_pending(&e2, "boss_yushou:confirm");
    let _ = answer(&mut e2, &c_ys, vec![ChoiceOption::Confirm]);
    let c_mj = expect_pending(&e2, "boss_mojian:confirm");
    let _ = answer(&mut e2, &c_mj, vec![ChoiceOption::Confirm]);
    drain(&mut e2);
    assert_eq!(e2.state_core().player(1).hp, 6, "guofengyupao seat 1 untouched");
    assert_eq!(e2.state_core().player(2).hp, 4, "unprotected enemy 2 took Nanman + Wanjian");

    // 6B-1: `boss_fengxing` excludes `eval_card_target_canceled` enemy (unarmored `txmanjia` -> normal Sha canceled):
    let mut e_fx = start_versus(0xB5_2013, &[&[0], &[1, 2]]);
    grant_skill(&mut e_fx, 0, "boss_fengxing");
    grant_skill(&mut e_fx, 1, "txmanjia");
    e_fx.debug_set_turn(2, Phase::End, 1);
    let _ = next_phase(&mut e_fx); // -> seat 0 TurnBegin
    let ft = expect_pending(&e_fx, "boss_fengxing:target");
    assert!(
        !ft.spec.options.contains(&ChoiceOption::Player(1))
            && ft.spec.options.contains(&ChoiceOption::Player(2)),
        "boss_fengxing:target must exclude unarmored txmanjia seat 1: {:?}",
        ft.spec.options
    );
    drain(&mut e_fx);

    // And if seat 1 (`txmanjia`) is the only enemy, `boss_fengxing` must NOT prompt:
    let mut e_fx_none = start_versus(0xB5_2014, &[&[0], &[1]]);
    grant_skill(&mut e_fx_none, 0, "boss_fengxing");
    grant_skill(&mut e_fx_none, 1, "txmanjia");
    e_fx_none.debug_set_turn(1, Phase::End, 1);
    let _ = next_phase(&mut e_fx_none);
    assert!(
        e_fx_none.open_choice().is_none(),
        "boss_fengxing must not prompt when only enemy cancels Sha"
    );

    // 6B-2: Real-event virtual `Sha` (via `txhj_yanSkill2`) entering `ShaResolveMachine` against
    // an attacker holding unarmored `txmanjia` is canceled cleanly without `respond_shan` or damage:
    let mut e_vs = start_versus(0xB5_2015, &[&[0], &[1]]);
    grant_skill(&mut e_vs, 0, "txmanjia");
    grant_skill(&mut e_vs, 1, "txhj_yanSkill2");
    let _shan0 = place_kind(&mut e_vs, 0, CardKind::Shan);
    e_vs.debug_set_turn(0, Phase::Play, 1);
    let s_hit = place_kind(&mut e_vs, 0, CardKind::Sha);
    let _ = play_card(&mut e_vs, 0, s_hit, vec![1]);
    drain(&mut e_vs);
    assert_eq!(e_vs.state_core().player(1).hp, 5, "seat 1 took primary Sha and fired virtual Sha");
    assert_eq!(
        e_vs.state_core().player(0).hp,
        6,
        "ShaResolveMachine must cancel virtual Sha against unarmored txmanjia seat 0"
    );
    assert_eq!(
        e_vs.state_core().player(0).hand.len(),
        1,
        "seat 0 did not have to spend Shan on canceled virtual Sha"
    );
}
