use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

/// Upstream: `extension_skill.js:3614-3637` (`txhj_mouduan`) & `3764-3798` (`txhj_xuanlve`)
#[test]
fn test_txhj_mouduan_and_xuanlve_chain() {
    let mut e = start_versus(0xB5_0101, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "txhj_mouduan");
    grant_skill(&mut e, 1, "txhj_xuanlve");
    grant_skill(&mut e, 1, "_txhj_shenqi");
    grant_skill(&mut e, 1, "_txhj_XXXshenqi");
    let _eq1 = equip_named(&mut e, 1, "txhj_taijifuchen");
    assert!(has_skill(&e, 1, "buff_txhj_fuchendangmo"));
    let _h2 = place_kind(&mut e, 2, CardKind::Tao);

    // Negative: only 3 suits / 2 types -> End phase does NOT trigger `txhj_mouduan`.
    e.debug_set_turn(0, Phase::Play, 1);
    let b1 = place_kind_suit_rank(&mut e, 0, CardKind::Sha, Suit::Spade, 7);
    let b2 = place_kind_suit_rank(&mut e, 0, CardKind::Jiu, Suit::Heart, 3);
    let b3 = place_std_kind(&mut e, 0, CardKind::Dismantle);
    let _ = play_card(&mut e, 0, b1, vec![1]);
    drain(&mut e);
    let _ = play_card(&mut e, 0, b2, Vec::new());
    drain(&mut e);
    let _ = play_card(&mut e, 0, b3, vec![2]);
    drain(&mut e);
    advance_until_turn_phase(&mut e, 0, Phase::End);
    assert!(
        e.open_choice().is_none(),
        "3 suits & 2 types must not trigger txhj_mouduan"
    );

    // Positive: play 4th suit (`Diamond`) in Play -> End opens `txhj_mouduan:confirm` -> move equip from 1 -> 0!
    // Losing equip triggers `txhj_xuanlve` (on 1) & `_txhj_XXXshenqi` (removes `fuchendangmo`).
    let mut e2 = start_versus(0xB5_0102, &[&[0], &[1, 2]]);
    grant_skill(&mut e2, 0, "txhj_mouduan");
    grant_skill(&mut e2, 1, "txhj_xuanlve");
    grant_skill(&mut e2, 1, "_txhj_shenqi");
    grant_skill(&mut e2, 1, "_txhj_XXXshenqi");
    let eq1b = equip_named(&mut e2, 1, "txhj_taijifuchen");
    let _h2b = place_kind(&mut e2, 2, CardKind::Tao);
    let _h0_x = place_kind(&mut e2, 0, CardKind::Tao);
    e2.debug_set_turn(0, Phase::Play, 1);
    for (kind, suit, rank, tgt) in [
        (CardKind::Sha, Suit::Spade, 7, vec![1]),
        (CardKind::Jiu, Suit::Heart, 3, vec![]),
        (CardKind::Dismantle, Suit::Club, 4, vec![2]),
    ] {
        let c = if kind == CardKind::Dismantle {
            place_std_kind(&mut e2, 0, CardKind::Dismantle)
        } else {
            place_kind_suit_rank(&mut e2, 0, kind, suit, rank)
        };
        let _ = play_card(&mut e2, 0, c, tgt);
        drain(&mut e2);
    }
    let armor4 = place_std_kind(&mut e2, 0, CardKind::Armor);
    let _ = play_card(&mut e2, 0, armor4, Vec::new());
    drain(&mut e2);
    advance_until_turn_phase(&mut e2, 0, Phase::End);
    let mc = expect_pending(&e2, "txhj_mouduan:confirm");
    let _ = answer(&mut e2, &mc, vec![ChoiceOption::Confirm]);
    let mf = expect_pending(&e2, "txhj_mouduan:move_from");
    let _ = answer(&mut e2, &mf, vec![ChoiceOption::Player(1)]);
    let mk = expect_pending(&e2, "txhj_mouduan:move_card");
    let _ = answer(&mut e2, &mk, vec![ChoiceOption::Card(eq1b)]);
    let mt = expect_pending(&e2, "txhj_mouduan:move_to");
    let e2_live = save_restore_and_compare_pending(&e2, &mt, vec![ChoiceOption::Player(0)]);
    let _ = e2_live.state_hash();
    let _ = answer(&mut e2, &mt, vec![ChoiceOption::Player(0)]);

    // `txhj_xuanlve` fires on seat 1 losing equip -> seat 1 discards seat 0's hand card!
    let xt = expect_pending(&e2, "txhj_xuanlve:target");
    let _ = answer(&mut e2, &xt, vec![ChoiceOption::Player(0)]);
    let xc = expect_pending(&e2, "txhj_xuanlve:card");
    let _ = answer(&mut e2, &xc, vec![ChoiceOption::Card(_h0_x)]);
    drain(&mut e2);
    assert!(e2.state_core().player(0).equips.contains(&eq1b));
    assert!(!e2.state_core().player(1).equips.contains(&eq1b));
    assert!(!e2.state_core().player(0).hand.contains(&_h0_x));
}

/// Upstream: `extension_skill.js:3645-3728` (`olshanjia`)
#[test]
fn test_olshanjia_lost_equips_and_free_sha() {
    // Case A: X=0 lost equips -> discard 3; discarding all equips grants free distance-less Sha!
    let mut e = start_versus(0xB5_0103, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "olshanjia");
    let eq_hand1 = place_kind(&mut e, 0, CardKind::Weapon);
    let eq_hand2 = place_kind(&mut e, 0, CardKind::Armor);
    let _eq_e = equip_kind(&mut e, 0, CardKind::HorsePlus);
    // Stack deck top 3 cards so drawn cards + hand have equips (or verify equip-only discards!).
    for _ in 0..3 {
        let top_w = e.debug_set_deck_top_by(|cd| {
            matches!(cd.kind, CardKind::Weapon | CardKind::Armor | CardKind::HorseMinus)
        });
        let _ = top_w;
    }
    e.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut e); // Draw -> Play (`olshanjia` phaseUseBegin)
    let c_conf = expect_pending(&e, "olshanjia:confirm");
    let _ = answer(&mut e, &c_conf, vec![ChoiceOption::Confirm]);
    let c_disc = expect_pending(&e, "olshanjia:discard");
    assert_eq!(c_disc.spec.min, 3);
    // AI prefers equips first to unlock virtual Sha.
    let plan = ai_plan(&e, 0);
    assert!(matches!(plan, GameAction::Answer { .. }));
    let _ = answer(
        &mut e,
        &c_disc,
        vec![
            ChoiceOption::Card(eq_hand1),
            ChoiceOption::Card(eq_hand2),
            ChoiceOption::Card(_eq_e),
        ],
    );
    let c_sha = expect_pending(&e, "olshanjia:sha_target");
    assert!(c_sha.spec.options.contains(&ChoiceOption::Player(2)), "olshanjia virtual Sha ignores distance");
    let e_done = save_restore_and_compare_pending(&e, &c_sha, vec![ChoiceOption::Player(2)]);
    assert_eq!(e_done.state_core().player(2).hp, 5);
    assert!(!e_done.turn().sha_used_play_phase, "olshanjia Sha must not consume Play phase Sha quota");
    assert_eq!(
        flag(&e_done, 0, "l11_olshanjia_lost_equips"),
        3,
        "self-olshanjia discarding 2 hand equips + 1 equip slot counts exactly 3 for next turn"
    );

    // Case B: Playing an equip card via `PlayCard` (`useCard`) does NOT count toward `X`;
    // dismantling an enemy's equip via `Guohe` counts `1` (NO double count across CardsDiscarded + CardsLost)!
    let mut ex = start_versus(0xB5_0104, &[&[0], &[1]]);
    grant_skill(&mut ex, 0, "olshanjia");
    let _eq_play = equip_kind(&mut ex, 0, CardKind::Weapon);
    assert_eq!(flag(&ex, 0, "l11_olshanjia_lost_equips"), 0, "equipping via useCard must NOT increment X");
    let _eq_victim = equip_kind(&mut ex, 0, CardKind::Armor);
    ex.debug_clear_hand(0);
    ex.debug_set_turn(1, Phase::Play, 1);
    let guohe = place_std_kind(&mut ex, 1, CardKind::Dismantle);
    let _ = play_card(&mut ex, 1, guohe, vec![0]);
    drain(&mut ex);
    assert_eq!(flag(&ex, 0, "l11_olshanjia_lost_equips"), 1);

    advance_until_turn_phase(&mut ex, 0, Phase::Draw);
    let _ = next_phase(&mut ex); // -> Play
    let conf2 = expect_pending(&ex, "olshanjia:confirm");
    let _ = answer(&mut ex, &conf2, vec![ChoiceOption::Confirm]);
    let disc2 = expect_pending(&ex, "olshanjia:discard");
    assert_eq!(disc2.spec.min, 2, "X=1 reduces discard from 3 to 2");
    // Discard basic card -> no Sha prompt!
    let basic = ex.state_core().player(0).hand[0];
    let other = ex.state_core().player(0).hand[1];
    let _ = answer(&mut ex, &disc2, vec![ChoiceOption::Card(basic), ChoiceOption::Card(other)]);
    drain(&mut ex);
    assert!(ex.open_choice().is_none());

    // Case C: Hand-equip losses via Discard Phase & hand `Dismantle` reduce `3 - X` -> `min == 1`,
    // and non-Snatch `CardsMoved` (`txyirang`) counts every moved equip toward `X`!
    let mut eh = start_versus(0xB5_010D, &[&[0], &[1]]);
    grant_skill(&mut eh, 0, "olshanjia");
    eh.debug_set_hp(0, 1);
    let h_eq1 = place_kind(&mut eh, 0, CardKind::Weapon);
    let _h_keep = place_kind(&mut eh, 0, CardKind::Sha);
    eh.debug_set_turn(0, Phase::Play, 1);
    let _ = next_phase(&mut eh); // -> Discard (limit 1, 2 in hand -> discard h_eq1)
    let dp = expect_pending(&eh, "discard_to_limit");
    let _ = answer(&mut eh, &dp, vec![ChoiceOption::Card(h_eq1)]);
    drain(&mut eh);
    assert_eq!(flag(&eh, 0, "l11_olshanjia_lost_equips"), 1, "Discard Phase hand equip counts once");

    // Seat 1 Dismantles another hand equip of seat 0 -> `2` -> `olshanjia:discard` requires `min == 1`!
    eh.debug_clear_hand(0);
    let _h_eq2 = place_kind(&mut eh, 0, CardKind::Armor);
    eh.debug_set_turn(1, Phase::Play, 1);
    let d_h = place_std_kind(&mut eh, 1, CardKind::Dismantle);
    let _ = play_card(&mut eh, 1, d_h, vec![0]);
    drain(&mut eh);
    assert_eq!(flag(&eh, 0, "l11_olshanjia_lost_equips"), 2, "hand Dismantle of equip counts once");
    eh.debug_set_turn(0, Phase::Draw, 2);
    let _ = next_phase(&mut eh); // -> Play (`olshanjia`)
    let oc = expect_pending(&eh, "olshanjia:confirm");
    let _ = answer(&mut eh, &oc, vec![ChoiceOption::Confirm]);
    let od = expect_pending(&eh, "olshanjia:discard");
    assert_eq!(od.spec.min, 1, "X=2 (Discard Phase + hand Dismantle) reduces discard 3 -> 1");
    drain(&mut eh);

    // Non-Snatch `CardsMoved` (`txyirang` moving 3 hand equips to seat 1) -> `X = 3` -> skips discard straight to `sha_target`!
    let mut ey = start_versus(0xB5_010F, &[&[0], &[1]]);
    ey.debug_set_max_hp(1, 8);
    grant_skill(&mut ey, 0, "txyirang");
    let _m1 = place_kind(&mut ey, 0, CardKind::Weapon);
    let _m2 = place_kind(&mut ey, 0, CardKind::Armor);
    let _m3 = place_kind(&mut ey, 0, CardKind::HorsePlus);
    ey.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut ey); // -> Play (`txyirang`)
    let yg = expect_pending(&ey, "txyirang:choose_target");
    let _ = answer(&mut ey, &yg, vec![ChoiceOption::Player(1)]);
    drain(&mut ey);
    assert_eq!(flag(&ey, 0, "l11_olshanjia_lost_equips"), 3, "non-Snatch CardsMoved equips counted once each");
    grant_skill(&mut ey, 0, "olshanjia");
    ey.debug_set_turn(0, Phase::Draw, 2);
    let _ = next_phase(&mut ey); // -> Play (`olshanjia`, X=3 -> need=0)
    let oc2 = expect_pending(&ey, "olshanjia:confirm");
    let _ = answer(&mut ey, &oc2, vec![ChoiceOption::Confirm]);
    let osha = expect_pending(&ey, "olshanjia:sha_target");
    let _ = answer(&mut ey, &osha, Vec::new());
    drain(&mut ey);
}

/// Upstream: `extension_skill.js:3735-3892,3899-3997,4134-4140`
/// (`boss_xuanlei`, `boss_baolian`, `dz_gs_yinfu`, `boss_leili`, `boss_fengxing`, `boss_yushou`, `boss_xixing`, `boss_modao`)
#[test]
fn test_boss_turn_phase_skills_batch() {
    // 1. `boss_xuanlei`, `boss_modao`, `dz_gs_yinfu`, `boss_fengxing`, `boss_xixing`, `boss_baolian`, `boss_yushou`.
    let mut e = start_versus(0xB5_0105, &[&[0, 3], &[1, 2]]);
    grant_skill(&mut e, 0, "boss_xuanlei");
    grant_skill(&mut e, 0, "boss_modao");
    grant_skill(&mut e, 0, "dz_gs_yinfu");
    grant_skill(&mut e, 0, "boss_fengxing");
    grant_skill(&mut e, 0, "boss_xixing");
    grant_skill(&mut e, 0, "boss_baolian");
    e.debug_set_hp(0, 4); // damaged=2 >= round 1 -> yinfu recovers 1 (4 -> 5)!
    // Put Le into seat 1 (enemy) judge, seat 3 (ally) judge, and link seat 2 (enemy).
    let le1 = place_kind(&mut e, 1, CardKind::Le);
    let le3 = place_kind(&mut e, 3, CardKind::Le);
    e.state_core_mut().player_mut(1).hand.retain(|&c| c != le1);
    e.state_core_mut().player_mut(1).judge.push(le1);
    e.state_core_mut().player_mut(3).hand.retain(|&c| c != le3);
    e.state_core_mut().player_mut(3).judge.push(le3);
    e.state_core_mut().player_mut(2).linked = true;

    e.debug_set_turn(3, Phase::End, 1);
    let _ = next_phase(&mut e); // TurnBegin(0) -> xuanlei damages 1 (not ally 3!), yinfu heals 4->5, fengxing prompts!
    let fx = expect_pending(&e, "boss_fengxing:target");
    assert!(!fx.spec.options.contains(&ChoiceOption::Player(3)), "ally excluded");
    let _ = answer(&mut e, &fx, vec![ChoiceOption::Player(1)]);
    // TurnBegin commits directly into Phase::Prepare (`boss_modao` +2 hand & `boss_xixing:target`)!
    let xx = expect_pending(&e, "boss_xixing:target");
    assert_eq!(e.state_core().player(1).hp, 4, "1 took xuanlei (1) + fengxing Sha (1)");
    assert_eq!(e.state_core().player(3).hp, 6, "ally immune to xuanlei");
    assert_eq!(e.state_core().player(0).hp, 5, "yinfu healed 1 on TurnBegin");
    assert_eq!(xx.spec.options, vec![ChoiceOption::Player(2)], "only linked seat 2 eligible");
    let mut e_xx = save_restore_and_compare_pending(&e, &xx, vec![ChoiceOption::Player(2)]);
    assert_eq!(e_xx.state_core().player(2).hp, 5);
    assert_eq!(e_xx.state_core().player(0).hp, 6, "boss_xixing heals 1 after thunder damage");
    assert_eq!(e_xx.state_core().player(0).hand.len(), 2, "boss_modao drew 2");

    // 3rd `dz_gs_yinfu` removes skill; `boss_baolian` draws 2 at Phase::End.
    e_xx.state_core_mut().vm_aux.flags.insert((0, "l11_yinfu_uses".to_string()), 2);
    e_xx.debug_set_hp(0, 3);
    e_xx.debug_set_turn(3, Phase::End, 2);
    let _ = next_phase(&mut e_xx);
    drain(&mut e_xx);
    assert!(!has_skill(&e_xx, 0, "dz_gs_yinfu"), "3 uses removes dz_gs_yinfu");

    // `boss_baolian`:
    advance_until_turn_phase(&mut e_xx, 0, Phase::Discard);
    drain(&mut e_xx);
    let hand_pre = e_xx.state_core().player(0).hand.len();
    let _ = next_phase(&mut e_xx); // -> End (`boss_baolian`)
    drain(&mut e_xx);
    assert_eq!(e_xx.state_core().player(0).hand.len(), hand_pre + 2);

    // 2. `boss_yushou` & `boss_leili`:
    let mut ey = start_versus(0xB5_0106, &[&[0, 3], &[1, 2]]);
    grant_skill(&mut ey, 0, "boss_yushou");
    grant_skill(&mut ey, 0, "boss_leili");
    ey.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut ey); // -> Play (`boss_yushou:confirm`)
    let yc = expect_pending(&ey, "boss_yushou:confirm");
    let _ = answer(&mut ey, &yc, vec![ChoiceOption::Confirm]);
    drain(&mut ey);
    assert_eq!(ey.state_core().player(1).hp, 5);
    assert_eq!(ey.state_core().player(2).hp, 5);
    assert_eq!(ey.state_core().player(3).hp, 6, "ally excluded from boss_yushou Nanman");

    // `boss_leili` on Sha hitting 1 -> targets ONLY 2 (not 1, not ally 3)!
    let sha_l = place_kind(&mut ey, 0, CardKind::Sha);
    let _ = play_card(&mut ey, 0, sha_l, vec![1]);
    let cl = expect_pending(&ey, "boss_leili:target");
    assert_eq!(cl.spec.options, vec![ChoiceOption::Player(2)]);
    let ey_done = save_restore_and_compare_pending(&ey, &cl, vec![ChoiceOption::Player(2)]);
    assert_eq!(ey_done.state_core().player(2).hp, 4, "boss_leili dealt 1 thunder damage");
}

/// Upstream: `extension_skill.js:4000-4082,4223-4413`
/// (`boss_moyany`, `boss_danshu`, `boss_jueji`, `boss_mojian`, `boss_lianyu`, `boss_skonghun`, `txhj_zhuwei`, `boss_zhennu`)
#[test]
fn test_boss_moyany_danshu_jueji_lianyu_skonghun_zhuwei_zhennu() {
    // 1. Out-of-turn `CardsLost` triggers `boss_moyany` (red -> 2 fire) & `boss_danshu` (red -> recover 1) ONCE!
    let mut e = start_versus(0xB5_0107, &[&[0], &[1, 2]]);
    grant_skill(&mut e, 0, "boss_moyany");
    grant_skill(&mut e, 0, "boss_danshu");
    e.debug_set_hp(0, 4);
    let _h0 = place_kind(&mut e, 0, CardKind::Tao);
    e.debug_set_turn(1, Phase::Play, 1);
    top_deck_suit(&mut e, Suit::Heart); // Red for boss_danshu
    top_deck_suit(&mut e, Suit::Diamond); // Red for boss_moyany (popped first!)
    let guohe = place_std_kind(&mut e, 1, CardKind::Dismantle);
    let _ = play_card(&mut e, 1, guohe, vec![0]);
    let cm = expect_pending(&e, "boss_moyany:confirm");
    let _ = answer(&mut e, &cm, vec![ChoiceOption::Confirm]);
    let cmt = expect_pending(&e, "boss_moyany:target");
    let _ = answer(&mut e, &cmt, vec![ChoiceOption::Player(2)]);
    let cd = expect_pending(&e, "boss_danshu:confirm");
    let _ = answer(&mut e, &cd, vec![ChoiceOption::Confirm]);
    drain(&mut e);
    assert!(e.open_choice().is_none(), "no duplicate trigger from CardsDiscarded");
    assert_eq!(e.state_core().player(2).hp, 4, "boss_moyany red judge dealt 2 fire damage");
    assert_eq!(e.state_core().player(0).hp, 5, "boss_danshu red judge healed 1");

    // 1b. Non-Guohe hand discard (`cixiong_skill`) & equip-zone `Dismantle` both trigger ONCE (`boss_moyany` + `boss_danshu`):
    let mut e_cx = start_versus(0xB5_010E, &[&[0], &[1, 2]]);
    grant_skill(&mut e_cx, 1, "cixiong_skill");
    grant_skill(&mut e_cx, 0, "boss_moyany");
    grant_skill(&mut e_cx, 0, "boss_danshu");
    e_cx.debug_set_hp(0, 4);
    e_cx.state_core_mut().vm_aux.flags.insert((0, "effective_gender_override".to_string()), 2);
    e_cx.state_core_mut().vm_aux.flags.insert((1, "effective_gender_override".to_string()), 1);
    let _boss_h = place_kind(&mut e_cx, 0, CardKind::Tao);
    e_cx.debug_set_turn(1, Phase::Play, 1);
    top_deck_suit(&mut e_cx, Suit::Heart); // Red for boss_danshu
    top_deck_suit(&mut e_cx, Suit::Diamond); // Red for boss_moyany
    let cx_sha = place_kind(&mut e_cx, 1, CardKind::Sha);
    let _ = play_card(&mut e_cx, 1, cx_sha, vec![0]);
    let ccx = expect_pending(&e_cx, "cixiong_skill:confirm");
    let _ = answer(&mut e_cx, &ccx, vec![ChoiceOption::Confirm]);
    let cdx = expect_pending(&e_cx, "cixiong_skill:target_discard");
    let _ = answer(&mut e_cx, &cdx, vec![ChoiceOption::Card(_boss_h)]);
    let cm2 = expect_pending(&e_cx, "boss_moyany:confirm");
    let _ = answer(&mut e_cx, &cm2, vec![ChoiceOption::Confirm]);
    let cmt2 = expect_pending(&e_cx, "boss_moyany:target");
    let _ = answer(&mut e_cx, &cmt2, vec![ChoiceOption::Player(2)]);
    let cd2 = expect_pending(&e_cx, "boss_danshu:confirm");
    let _ = answer(&mut e_cx, &cd2, vec![ChoiceOption::Confirm]);
    drain(&mut e_cx);
    assert_eq!(e_cx.state_core().player(2).hp, 4, "cixiong hand discard triggered boss_moyany once");
    assert_eq!(e_cx.state_core().player(0).hp, 4, "boss_danshu healed 1 (4 -> 3 Sha -> 4)");

    // Equip-zone `Dismantle` triggers `boss_moyany` ONCE (NO double trigger across CardsLost + CardsDiscarded):
    e_cx.debug_clear_hand(0);
    let _boss_eq = equip_kind(&mut e_cx, 0, CardKind::Armor);
    top_deck_suit(&mut e_cx, Suit::Club); // Black for boss_danshu
    top_deck_suit(&mut e_cx, Suit::Spade); // Black for boss_moyany
    let d_eq = place_std_kind(&mut e_cx, 1, CardKind::Dismantle);
    let _ = play_card(&mut e_cx, 1, d_eq, vec![0]);
    let cm3 = expect_pending(&e_cx, "boss_moyany:confirm");
    let _ = answer(&mut e_cx, &cm3, vec![ChoiceOption::Confirm]);
    let cd3 = expect_pending(&e_cx, "boss_danshu:confirm");
    let _ = answer(&mut e_cx, &cd3, vec![ChoiceOption::Confirm]);
    drain(&mut e_cx);
    assert!(e_cx.open_choice().is_none(), "equip Dismantle must NOT double-trigger boss_moyany/boss_danshu");

    // Hand `Snatch` (`CardsMoved`) also triggers `boss_moyany` & `boss_danshu` once!
    e_cx.debug_clear_hand(0);
    let _boss_h2 = place_kind(&mut e_cx, 0, CardKind::Sha);
    top_deck_suit(&mut e_cx, Suit::Spade);
    top_deck_suit(&mut e_cx, Suit::Club);
    let sn_h = place_std_kind(&mut e_cx, 1, CardKind::Snatch);
    let _ = play_card(&mut e_cx, 1, sn_h, vec![0]);
    let cm4 = expect_pending(&e_cx, "boss_moyany:confirm");
    let _ = answer(&mut e_cx, &cm4, vec![ChoiceOption::Confirm]);
    let cd4 = expect_pending(&e_cx, "boss_danshu:confirm");
    let _ = answer(&mut e_cx, &cd4, vec![ChoiceOption::Confirm]);
    drain(&mut e_cx);
    assert!(e_cx.open_choice().is_none(), "hand Snatch CardsMoved triggers once cleanly");

    // 2. `boss_jueji`: injured enemy draws 1 fewer; `boss_mojian` / `boss_lianyu` / `boss_skonghun` / `boss_zhennu`.
    let mut eb = start_versus(0xB5_0108, &[&[0], &[1, 2]]);
    grant_skill(&mut eb, 0, "boss_jueji");
    grant_skill(&mut eb, 0, "boss_mojian");
    grant_skill(&mut eb, 0, "boss_lianyu");
    grant_skill(&mut eb, 0, "boss_zhennu");
    eb.debug_set_hp(1, 5); // injured enemy -> triggers boss_jueji
    eb.debug_set_turn(1, Phase::Judge, 1);
    let _ = next_phase(&mut eb);
    let cj = expect_pending(&eb, "boss_jueji:confirm");
    let _ = answer(&mut eb, &cj, vec![ChoiceOption::Confirm]);
    drain(&mut eb);
    assert_eq!(eb.state_core().player(1).hand.len(), 1, "boss_jueji reduced draw 2 -> 1");

    // Seat 0 `boss_zhennu` (Prepare), `boss_mojian` (Play), `boss_lianyu` (End):
    eb.debug_set_turn(2, Phase::End, 1);
    advance_until_turn_phase(&mut eb, 0, Phase::Judge);
    drain(&mut eb);
    assert_eq!(eb.state_core().player(1).hp, 4, "boss_zhennu hit seat 1");
    assert_eq!(eb.state_core().player(2).hp, 5, "boss_zhennu hit seat 2");

    advance_until_turn_phase(&mut eb, 0, Phase::Draw);
    let _ = next_phase(&mut eb);
    let cmj = expect_pending(&eb, "boss_mojian:confirm");
    let _ = answer(&mut eb, &cmj, vec![ChoiceOption::Confirm]);
    drain(&mut eb);
    assert_eq!(eb.state_core().player(1).hp, 3);
    assert_eq!(eb.state_core().player(2).hp, 4);

    advance_until_turn_phase(&mut eb, 0, Phase::Discard);
    let _ = next_phase(&mut eb);
    let cly = expect_pending(&eb, "boss_lianyu:confirm");
    let _ = answer(&mut eb, &cly, vec![ChoiceOption::Confirm]);
    drain(&mut eb);
    assert_eq!(eb.state_core().player(1).hp, 2);
    assert_eq!(eb.state_core().player(2).hp, 3);

    // `boss_skonghun`: damaged (2) >= enemies (2) -> deals 1 Thunder to both in forward order -> recovers 2!
    let mut esk = start_versus(0xB5_0109, &[&[0], &[1, 2]]);
    grant_skill(&mut esk, 0, "boss_skonghun");
    esk.debug_set_hp(0, 4);
    esk.debug_set_turn(0, Phase::Draw, 1);
    let _ = next_phase(&mut esk); // -> Play
    drain(&mut esk);
    assert_eq!(esk.state_core().player(1).hp, 5);
    assert_eq!(esk.state_core().player(2).hp, 5);
    assert_eq!(esk.state_core().player(0).hp, 6, "boss_skonghun healed 2 after damaging 2 enemies");

    // 3. `txhj_zhuwei`: settled judge card -> confirm_gain puts judge card in hand + confirm_buff +1 hand & +1 Sha limit!
    let mut ezw = start_versus(0xB5_010A, &[&[0], &[1]]);
    grant_skill(&mut ezw, 0, "txhj_zhuwei");
    ezw.debug_set_hp(0, 1); // hand limit 1 without zhuwei
    let le = place_kind(&mut ezw, 0, CardKind::Le);
    ezw.state_core_mut().player_mut(0).hand.retain(|&c| c != le);
    ezw.state_core_mut().player_mut(0).judge.push(le);
    let j_card = top_deck_suit(&mut ezw, Suit::Heart);
    ezw.debug_set_turn(0, Phase::Prepare, 2);
    let _ = next_phase(&mut ezw); // -> Judge -> Le settles!
    let zg = expect_pending(&ezw, "txhj_zhuwei:confirm_gain");
    let _ = answer(&mut ezw, &zg, vec![ChoiceOption::Confirm]);
    let zb = expect_pending(&ezw, "txhj_zhuwei:confirm_buff");
    let mut ezw = save_restore_and_compare_pending(&ezw, &zb, vec![ChoiceOption::Confirm]);
    assert!(ezw.state_core().player(0).hand.contains(&j_card), "txhj_zhuwei gained judge card");
    assert_eq!(flag(&ezw, 0, "skill_turn:l11_zhuwei_eff"), 1);

    // Extra +1 Sha limit in Play phase:
    advance_until_turn_phase(&mut ezw, 0, Phase::Play);
    let z_s1 = place_kind(&mut ezw, 0, CardKind::Sha);
    let z_s2 = place_kind(&mut ezw, 0, CardKind::Sha);
    let _ = play_card(&mut ezw, 0, z_s1, vec![1]);
    drain(&mut ezw);
    let out2 = play_card(&mut ezw, 0, z_s2, vec![1]);
    assert!(matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
}

/// Upstream: `extension_skill.js:3843-3872` (`boss_lingfeng`) & `4416-4509` (`txhj_chuhai`, `txhj_chuhai2`)
#[test]
fn test_boss_lingfeng_save_restore_and_chuhai_pindian() {
    // 1. `boss_lingfeng`: Confirm + diff color -> save/restore mid `target` + cancel -> still gains both cards!
    let mut el = start_versus(0xB5_010B, &[&[0], &[1]]);
    grant_skill(&mut el, 0, "boss_lingfeng");
    top_deck_suit(&mut el, Suit::Heart);
    top_deck_suit(&mut el, Suit::Spade);
    el.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut el);
    let lc = expect_pending(&el, "boss_lingfeng:confirm");
    let _ = answer(&mut el, &lc, vec![ChoiceOption::Confirm]);
    let lt = expect_pending(&el, "boss_lingfeng:target");
    let el_cancel = save_restore_and_compare_pending(&el, &lt, Vec::new());
    assert_eq!(el_cancel.state_core().player(1).hp, 6, "cancel keeps enemy hp unchanged");
    assert_eq!(el_cancel.state_core().player(0).hand.len(), 2, "still gains both revealed cards");

    // 2. `txhj_chuhai` + `txhj_chuhai_add` + `txhj_chuhai2`!
    let mut ec = start_versus(0xB5_010C, &[&[0], &[1]]);
    grant_skill(&mut ec, 0, "txhj_chuhai");
    ec.debug_set_turn(0, Phase::Play, 1);
    let my_p = place_kind_suit_rank(&mut ec, 0, CardKind::Sha, Suit::Spade, 7);
    let their_p = place_kind_suit_rank(&mut ec, 1, CardKind::Sha, Suit::Heart, 10);
    // Notice: 7 + (4 - 0 equips) = 11 > 10 (`txhj_chuhai_add`)!! Plus seat 1 also holds a Trick in hand!
    let _their_trick = place_kind(&mut ec, 1, CardKind::Dismantle);

    // Cancel target selection refunds `once_per_turn`!
    let act0 = activate_skill(&mut ec, 0, "txhj_chuhai");
    assert!(matches!(act0, CommandOutcome::Pending { .. }));
    let ct0 = expect_pending(&ec, "txhj_chuhai:target");
    let _ = answer(&mut ec, &ct0, Vec::new());
    drain(&mut ec);

    // Re-activate after cancel:
    let act1 = activate_skill(&mut ec, 0, "txhj_chuhai");
    assert!(matches!(act1, CommandOutcome::Pending { .. }));
    let ct1 = expect_pending(&ec, "txhj_chuhai:target");
    let _ = answer(&mut ec, &ct1, vec![ChoiceOption::Player(1)]);
    let cp_owner = expect_pending(&ec, "txhj_chuhai:pindian_card");
    let _ = answer(&mut ec, &cp_owner, vec![ChoiceOption::Card(my_p)]);
    let cp_tgt = expect_pending(&ec, "txhj_chuhai:target_pindian_card");
    let mut ec = save_restore_and_compare_pending(&ec, &cp_tgt, vec![ChoiceOption::Card(their_p)]);
    assert!(
        has_skill(&ec, 0, "txhj_chuhai2"),
        "txhj_chuhai_add boosted 7+4=11 > 10 -> won pindian & granted txhj_chuhai2"
    );
    assert!(
        ec.state_core()
            .player(0)
            .hand
            .iter()
            .any(|&c| ec.catalog().card_kind(c).equip_slot().is_none()
                && !ec.catalog().card_kind(c).is_basic_kind()),
        "gained Trick card from pile matching enemy remaining hand type"
    );

    // Dealing damage to seat 1 in same Play phase triggers `txhj_chuhai2` equipping an empty slot!
    let sha_c = place_kind(&mut ec, 0, CardKind::Sha);
    let _ = play_card(&mut ec, 0, sha_c, vec![1]);
    drain(&mut ec);
    assert!(
        !ec.state_core().player(0).equips.is_empty(),
        "txhj_chuhai2 equipped card from pile into empty slot"
    );

    // Leaving Play phase revokes `txhj_chuhai2` (`phaseUseEnd`).
    let _ = next_phase(&mut ec);
    drain(&mut ec);
    assert!(!has_skill(&ec, 0, "txhj_chuhai2"), "txhj_chuhai2 expires at phaseUseEnd");
}
