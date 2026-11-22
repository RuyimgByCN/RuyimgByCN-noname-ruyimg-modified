use engine_native::catalog::{CardKind, Faction, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

use super::common::*;

/// Upstream: `extension.js:3581-3765` (`diyfeihua`, `txclanxieshu`, `txshixin`, `txmouduan`, `txkaikang`).
#[test]
fn l11_skills_b_feihua_xieshu_shixin_mouduan_kaikang() {
    // 1. diyfeihua: Play begin randomly emits ArcheryAttack or virtual txhj_zhijizhibi (both arms over seeds)
    let (mut saw_aoe_dmg, mut saw_zhibi_draw) = (false, false);
    for seed in 0x1500u64..0x1514 {
        let mut ex = start_taixu(seed, 2);
        let _ = place_kind(&mut ex, 1, CardKind::Shan); // non-empty target hand lets virtual zhijizhibi draw 1
        ex.debug_grant(0, "diyfeihua");
        ex.debug_set_turn(0, Phase::Draw, 1);
        let _ = next_phase(&mut ex);
        drain(&mut ex);
        if ex.debug_hp(1) < 6 {
            saw_aoe_dmg = true;
        }
        if !ex.debug_hand(0).is_empty() {
            saw_zhibi_draw = true;
        }
        if saw_aoe_dmg && saw_zhibi_draw {
            break;
        }
    }
    assert!(saw_aoe_dmg && saw_zhibi_draw, "diyfeihua must cover both ArcheryAttack and virtual txhj_zhijizhibi");

    // 2. txclanxieshu: DamageEnd with `card` having name len (e.g. `sha` = 1) discards 1 -> draws lost_hp!
    let mut e_xs = start_taixu(0x1502, 2);
    e_xs.debug_grant(0, "txclanxieshu");
    e_xs.debug_set_hp(0, 4);
    let x_card = place_kind_suit(&mut e_xs, 0, CardKind::Shan, Suit::Heart);
    let s_hit = place_kind_suit(&mut e_xs, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_xs, 0, s_hit, vec![1]);
    let c_xs = expect_pending(&e_xs, "txclanxieshu:discard_cards");
    assert_eq!(c_xs.spec.min, 1);
    let e_xs_done = save_restore_and_compare_pending(&e_xs, &c_xs, vec![ChoiceOption::Card(x_card)]);
    assert_eq!(e_xs_done.debug_hand(0).len(), 2, "lost_hp = 6 - 4 = 2 cards drawn");

    // 3. txshixin & txshixin_red: Black cancels trick targeting owner; Red steals the trick after use!
    let mut e_sx = start_taixu(0x1503, 2);
    e_sx.debug_grant(1, "txshixin");
    let d1 = place_kind_suit(&mut e_sx, 0, CardKind::Dismantle, Suit::Spade);
    let _keep = place_kind_suit(&mut e_sx, 1, CardKind::Shan, Suit::Heart);
    let _ = top_deck_suit(&mut e_sx, Suit::Spade);
    let _ = play_card(&mut e_sx, 0, d1, vec![1]);
    let c_sx1 = expect_pending(&e_sx, "txshixin:confirm");
    let _ = answer(&mut e_sx, &c_sx1, vec![ChoiceOption::Confirm]);
    drain(&mut e_sx);
    assert_eq!(e_sx.debug_hand(1).len(), 1, "Black judge cancelled Dismantle");

    let d2 = place_kind_suit(&mut e_sx, 0, CardKind::Dismantle, Suit::Heart);
    let _ = top_deck_suit(&mut e_sx, Suit::Heart);
    let _ = play_card(&mut e_sx, 0, d2, vec![1]);
    let c_sx2 = expect_pending(&e_sx, "txshixin:confirm");
    let _ = answer(&mut e_sx, &c_sx2, vec![ChoiceOption::Confirm]);
    drain(&mut e_sx);
    assert!(e_sx.debug_hand(1).contains(&d2), "Red judge gained the trick card after settlement");

    // 4. txmouduan: End phase after using 3 card types moves field card (`equips`/`judge`)!
    let mut e_md = start_taixu(0x1504, 2);
    e_md.debug_grant(0, "txmouduan");
    let eq_src = equip_kind(&mut e_md, 1, CardKind::Zhuge);
    // Seat 0 uses Basic (`jiu`), Equip (`qinggang`), Trick (`wuzhong`) -> 3 types in Play!
    let jiu = place_kind_suit(&mut e_md, 0, CardKind::Jiu, Suit::Spade);
    let _ = play_card(&mut e_md, 0, jiu, Vec::new());
    drain(&mut e_md);
    let _ = equip_kind(&mut e_md, 0, CardKind::Armor);
    let wz = place_kind_suit(&mut e_md, 0, CardKind::Wuzhong, Suit::Heart);
    let _ = play_card(&mut e_md, 0, wz, Vec::new());
    drain(&mut e_md);
    e_md.debug_set_turn(0, Phase::Discard, 1);
    let _ = next_phase(&mut e_md); // End triggers txmouduan
    let c_md = expect_pending(&e_md, "txmouduan:confirm");
    let _ = answer(&mut e_md, &c_md, vec![ChoiceOption::Confirm]);
    let c_from = expect_pending(&e_md, "txmouduan:move_from");
    let _ = answer(&mut e_md, &c_from, vec![ChoiceOption::Player(1)]);
    let c_card = expect_pending(&e_md, "txmouduan:move_card");
    let _ = answer(&mut e_md, &c_card, vec![ChoiceOption::Card(eq_src)]);
    let c_to = expect_pending(&e_md, "txmouduan:move_to");
    let _ = answer(&mut e_md, &c_to, vec![ChoiceOption::Player(0)]);
    drain(&mut e_md);
    assert!(e_md.debug_equips(0).contains(&eq_src));
    assert!(e_md.debug_equips(1).is_empty());

    // 5. txkaikang: on Sha used -> draw 1 -> give 1 card -> target optionally equips it!
    let mut e_kk = start_taixu(0x1505, 2);
    e_kk.debug_grant(0, "txkaikang");
    let s_kk = place_kind_suit(&mut e_kk, 0, CardKind::Sha, Suit::Spade);
    let give_eq = place_kind(&mut e_kk, 0, CardKind::Armor);
    let _ = play_card(&mut e_kk, 0, s_kk, vec![1]);
    let c_kk = expect_pending(&e_kk, "txkaikang:confirm");
    let _ = answer(&mut e_kk, &c_kk, vec![ChoiceOption::Confirm]);
    let c_kg = expect_pending(&e_kk, "txkaikang:choose_card");
    let _ = answer(&mut e_kk, &c_kg, vec![ChoiceOption::Card(give_eq)]);
    let c_ke = expect_pending(&e_kk, "txkaikang:choose_equip");
    assert_eq!(c_ke.observer, 1);
    let _ = answer(&mut e_kk, &c_ke, vec![ChoiceOption::Confirm]);
    drain(&mut e_kk);
    assert!(e_kk.debug_equips(1).contains(&give_eq), "target equipped gifted equip via txkaikang");
}

/// Upstream: `extension.js:3766-4020` (`txhuangfu`, `txleizhen`, `txxiebi`, `txlianbi`, `txfeiyan`, `txleili`, `cxyMoJun`, `cxyKuangXi`, `txbaobian`).
#[test]
fn l11_skills_b_huangfu_through_txbaobian() {
    // 1. txhuangfu (immune Thunder) + txleizhen (Death deals 1 Thunder to killer) + txxiebi (clamps dmg > 1 to 1)
    let mut e_def = start_taixu(0x1506, 3);
    e_def.debug_grant(1, "txhuangfu");
    e_def.debug_grant(1, "txxiebi");
    e_def.debug_grant(2, "txleizhen");
    let jiu = place_kind_suit(&mut e_def, 0, CardKind::Jiu, Suit::Spade);
    let _ = play_card(&mut e_def, 0, jiu, Vec::new());
    let lsha = place_kind_suit(&mut e_def, 0, CardKind::LeiSha, Suit::Spade);
    let _ = play_card(&mut e_def, 0, lsha, vec![1]);
    drain(&mut e_def);
    assert_eq!(e_def.debug_hp(1), 6, "txhuangfu prevented Thunder damage");
    // Normal Sha with Jiu (2 dmg) in fresh turn -> txxiebi clamps to 1
    let mut e_xb = start_taixu(0x150c, 2);
    e_xb.debug_grant(1, "txxiebi");
    let jiu2 = place_kind(&mut e_xb, 0, CardKind::Jiu);
    let _ = play_card(&mut e_xb, 0, jiu2, Vec::new());
    let nsha = place_kind_suit(&mut e_xb, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_xb, 0, nsha, vec![1]);
    drain(&mut e_xb);
    assert_eq!(e_xb.debug_hp(1), 5, "txxiebi clamped 2 dmg to 1");
    // Kill seat 2 (`txleizhen`) -> killer 0 takes 1 Thunder
    e_def.debug_set_hp(2, 1);
    e_def.debug_grant(0, "咆哮");
    let ksha = place_kind_suit(&mut e_def, 0, CardKind::Sha, Suit::Club);
    let _ = play_card(&mut e_def, 0, ksha, vec![2]);
    drain(&mut e_def);
    assert!(!e_def.debug_alive(2));
    assert_eq!(e_def.debug_hp(0), 5, "txleizhen dealt 1 Thunder to killer 0");

    // 2. txlianbi: links self on TurnBegin/DamageAfter and keeps linked (`txlianbi_blocks_unlink`)
    let mut e_lb = start_taixu(0x1507, 2);
    e_lb.debug_grant(1, "txlianbi");
    e_lb.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e_lb);
    drain(&mut e_lb);
    assert!(e_lb.debug_linked(1), "txlianbi links owner at TurnBegin");
    // Fire damage to linked seat 1 stays linked because `txlianbi` prevents auto-unlink!
    e_lb.debug_set_turn(0, Phase::Play, 1);
    let fsha = place_kind(&mut e_lb, 0, CardKind::FireSha);
    let _ = play_card(&mut e_lb, 0, fsha, vec![1]);
    drain(&mut e_lb);
    assert_eq!(e_lb.debug_hp(1), 5);
    assert!(e_lb.debug_linked(1), "txlianbi must stay linked");

    // 3. txfeiyan: other uses single-target Sha & owner has attacker in range -> discard Sha on attacker & draw 2!
    let mut e_fy = start_taixu(0x1508, 3);
    e_fy.debug_grant(2, "txfeiyan"); // seat 2 has seat 0 in distance 1!
    let fy_sha = place_kind_suit(&mut e_fy, 2, CardKind::Sha, Suit::Spade);
    let s0_fy = place_kind_suit(&mut e_fy, 0, CardKind::Sha, Suit::Heart);
    let _ = play_card(&mut e_fy, 0, s0_fy, vec![1]);
    let c_fy = expect_pending(&e_fy, "txfeiyan:confirm");
    assert_eq!(c_fy.observer, 2);
    let _ = answer(&mut e_fy, &c_fy, vec![ChoiceOption::Confirm]);
    let c_fys = expect_pending(&e_fy, "txfeiyan:choose_sha");
    let _ = answer(&mut e_fy, &c_fys, vec![ChoiceOption::Card(fy_sha)]);
    drain(&mut e_fy);
    assert_eq!(e_fy.debug_hp(0), 5, "txfeiyan counter-Sha hit seat 0");
    assert_eq!(e_fy.debug_hand(2).len(), 2, "txfeiyan drew 2 after resolution");

    // 4. txleili & cxyMoJun
    let mut e_lm = start_versus(0x1509, &[&[0, 1], &[2]], &[Faction::Wei, Faction::Shu]);
    e_lm.debug_grant(0, "txleili");
    e_lm.debug_grant(1, "cxyMoJun");
    let _ = top_deck_suit(&mut e_lm, Suit::Spade); // black for cxyMoJun -> friends draw 1
    let s_lm = place_kind_suit(&mut e_lm, 0, CardKind::Sha, Suit::Spade);
    let _ = play_card(&mut e_lm, 0, s_lm, vec![1]); // hit ally 1 with Sha -> txleili can hit enemy 2 with 1 Thunder!
    let c_ll = expect_pending(&e_lm, "txleili:choose_target");
    assert!(!c_ll.spec.options.contains(&ChoiceOption::Player(1)), "must exclude damaged target & friends");
    let _ = answer(&mut e_lm, &c_ll, vec![ChoiceOption::Player(2)]);
    drain(&mut e_lm);
    assert_eq!(e_lm.debug_hp(2), 5, "txleili dealt 1 Thunder to enemy 2");
    assert_eq!(e_lm.debug_hand(0).len(), 1, "cxyMoJun black judge drew 1 for friend 0");
    assert_eq!(e_lm.debug_hand(1).len(), 1, "cxyMoJun black judge drew 1 for friend 1");

    // 5. cxyKuangXi: Play active loses 1 HP, deals 1 dmg; dying target seals cxyKuangXi for turn!
    let mut e_kx = start_taixu(0x150a, 2);
    e_kx.debug_grant(0, "cxyKuangXi");
    e_kx.debug_set_hp(1, 1);
    let _ = place_kind_suit(&mut e_kx, 1, CardKind::Tao, Suit::Heart); // seat 1 saves self -> dying triggered
    let _ = activate_skill(&mut e_kx, 0, "cxyKuangXi");
    let c_kx = expect_pending(&e_kx, "cxyKuangXi:choose_target");
    assert_eq!(ai_plan(&e_kx, 0), GameAction::Answer {
        choice_id: c_kx.id,
        selection: vec![ChoiceOption::Player(1)],
    });
    let _ = answer(&mut e_kx, &c_kx, vec![ChoiceOption::Player(1)]);
    // Seat 1 chooses Tao to survive dying:
    let c_tao = answer_pass_until(&mut e_kx, "dying_save");
    let tao1 = e_kx.debug_hand(1)[0];
    let _ = answer(&mut e_kx, &c_tao, vec![ChoiceOption::Card(tao1)]);
    drain(&mut e_kx);
    assert_eq!(e_kx.debug_hp(0), 5);
    assert!(
        matches!(
            activate_skill(&mut e_kx, 0, "cxyKuangXi"),
            CommandOutcome::Rejected(_)
        ),
        "cxyKuangXi must be sealed for rest of turn after triggering dying"
    );

    // 6. txbaobian: hp <= 3 gains `tiaoxin`, hp <= 2 gains `repaoxiao`/`paoxiao`, hp <= 1 gains `shensu`; heal > 3 removes them!
    let mut e_bb = start_taixu(0x150b, 2);
    e_bb.debug_grant(0, "txbaobian");
    e_bb.debug_set_hp(0, 2);
    e_bb.debug_set_turn(0, Phase::Judge, 1);
    let _ = next_phase(&mut e_bb);
    drain(&mut e_bb);
    assert!(e_bb.debug_has_skill(0, "tiaoxin"), "hp <= 3 -> tiaoxin");
    assert!(
        e_bb.debug_has_skill(0, "咆哮") || e_bb.debug_has_skill(0, "paoxiao"),
        "hp <= 2 -> paoxiao"
    );
    assert!(!e_bb.debug_has_skill(0, "shensu"), "hp 2 > 1 -> no shensu yet");
    e_bb.debug_set_turn(0, Phase::Play, 1);
    for _ in 0..2 {
        let tao_bb = place_kind(&mut e_bb, 0, CardKind::Tao);
        let _ = play_card(&mut e_bb, 0, tao_bb, Vec::new());
        drain(&mut e_bb);
    }
    assert_eq!(e_bb.debug_hp(0), 4);
    assert!(!e_bb.debug_has_skill(0, "tiaoxin"), "recovering > 3 removes tiaoxin");
    assert!(!e_bb.debug_has_skill(0, "咆哮") && !e_bb.debug_has_skill(0, "paoxiao"));
    let _ = e_bb.debug_deal_damage(None, 0, 3, DamageNature::Normal); // hp 1
    drain(&mut e_bb);
    assert!(e_bb.debug_has_skill(0, "shensu"), "hp <= 1 -> shensu");
}
