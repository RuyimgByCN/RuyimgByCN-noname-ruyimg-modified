use super::common::*;
use engine_native::ai::BoundedPlanner;
use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase};

#[test]
fn l11_vm_fix_a_fuji_refankui_fenwei_kuangfu_qiaomeng() {
    // 1. fuji: Thunder damage triggers source.judge(); Black -> +1 dmg, Red -> source gains judge card
    let mut e = start_clean_4p_play(910_201);
    e.debug_grant(3, "fuji");
    let th_sha1 = place_kind(&mut e, 0, CardKind::LeiSha);
    let _j_black = e.debug_set_deck_top_by(|cd| matches!(cd.suit, Suit::Spade | Suit::Club));
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: th_sha1, targets: vec![1] });
    assert!(prompt_is(&e, "l11_vm_a_fuji_confirm"));
    let _ = answer(&mut e, 3, vec![ChoiceOption::Confirm]);
    drain_ai(&mut e, 8);
    assert_eq!(e.debug_hp(1), 3, "fuji black judge adds +1 thunder damage");

    let th_sha2 = place_kind(&mut e, 0, CardKind::LeiSha);
    let wq = place_kind(&mut e, 0, CardKind::Zhuge);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: wq, targets: vec![] });
    let j_red = e.debug_set_deck_top_by(|cd| matches!(cd.suit, Suit::Heart | Suit::Diamond));
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: th_sha2, targets: vec![1] });
    let _ = answer(&mut e, 3, vec![ChoiceOption::Confirm]);
    drain_ai(&mut e, 8);
    assert_eq!(e.debug_hp(1), 2);
    assert!(e.debug_hand(0).contains(&j_red), "fuji red judge gives judge card to source");

    // 2. refankui: damaged by N (Jiu + Sha = 2 dmg) steals 2 cards from source
    let mut e2 = start_clean_4p_play(910_202);
    e2.debug_grant(1, "refankui");
    let jiu = e2.debug_place(0, "jiu");
    let sha = e2.debug_place(0, "sha");
    let loot1 = e2.debug_place(0, "tao");
    let loot2 = e2.debug_place(0, "shan");
    e2.debug_clear_hand(1);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: jiu, targets: vec![] });
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
    assert!(prompt_is(&e2, "l11_vm_a_refankui_gain"));
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Card(loot1)]);
    assert!(prompt_is(&e2, "l11_vm_a_refankui_gain"));
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Card(loot2)]);
    assert!(e2.debug_hand(1).contains(&loot1));
    assert!(e2.debug_hand(1).contains(&loot2));

    // 3. fenwei: cancels selected targets of Wugu / SavageAttack; excludes Jiedao; once per game
    let mut e3 = start_clean_4p_play(910_203);
    e3.debug_grant(2, "fenwei");
    let aoe = place_kind(&mut e3, 0, CardKind::SavageAttack);
    e3.debug_clear_hand(1);
    e3.debug_clear_hand(2);
    e3.debug_clear_hand(3);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: aoe, targets: vec![] });
    assert!(prompt_is(&e3, "l11_vm_a_fenwei_targets"));
    let _ = answer(&mut e3, 2, vec![ChoiceOption::Player(2), ChoiceOption::Player(3)]);
    drain_ai(&mut e3, 16);
    assert_eq!((e3.debug_hp(1), e3.debug_hp(2), e3.debug_hp(3)), (4, 5, 5), "fenwei cancels only chosen seats 2 & 3");
    let wugu = place_kind(&mut e3, 0, CardKind::Wugu);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: wugu, targets: vec![] });
    assert!(!prompt_is(&e3, "l11_vm_a_fenwei_targets"), "fenwei is once per game");
    drain_ai(&mut e3, 16);

    // fenwei cancelling ALL targets prevents all damage
    let mut e3b = start_clean_4p_play(910_214);
    e3b.debug_grant(2, "fenwei");
    let aoe_all = place_kind(&mut e3b, 0, CardKind::SavageAttack);
    let _ = submit(&mut e3b, 0, GameAction::PlayCard { card: aoe_all, targets: vec![] });
    assert!(prompt_is(&e3b, "l11_vm_a_fenwei_targets"));
    let _ = answer(&mut e3b, 2, vec![ChoiceOption::Player(1), ChoiceOption::Player(2), ChoiceOption::Player(3)]);
    drain_ai(&mut e3b, 16);
    assert_eq!((e3b.debug_hp(1), e3b.debug_hp(2), e3b.debug_hp(3)), (5, 5, 5));

    // fenwei on 2P Taoyuan: caster included in effective targets (len == 2 >= 2) and selectable
    let mut e3c = start_clean_2p_play(910_216);
    e3c.debug_grant(1, "fenwei");
    e3c.debug_set_hp(0, 2);
    e3c.debug_set_hp(1, 2);
    let ty1 = place_kind(&mut e3c, 0, CardKind::Taoyuan);
    let _ = submit(&mut e3c, 0, GameAction::PlayCard { card: ty1, targets: vec![] });
    assert!(prompt_is(&e3c, "l11_vm_a_fenwei_targets"));
    let opts = e3c.open_choice().unwrap().spec.options.clone();
    assert!(opts.contains(&ChoiceOption::Player(0)));
    assert!(opts.contains(&ChoiceOption::Player(1)));
    let _ = answer(&mut e3c, 1, vec![ChoiceOption::Player(0)]);
    drain_ai(&mut e3c, 16);
    assert_eq!(e3c.debug_hp(0), 2, "cancelled Taoyuan caster must stay at 2 HP");
    assert_eq!(e3c.debug_hp(1), 3, "uncancelled Taoyuan target recovers +1 HP");

    // fenwei on 2P Taoyuan cancelling non-caster leaves caster recovering +1 HP
    let mut e3d = start_clean_2p_play(910_217);
    e3d.debug_grant(1, "fenwei");
    e3d.debug_set_hp(0, 2);
    e3d.debug_set_hp(1, 2);
    let ty2 = place_kind(&mut e3d, 0, CardKind::Taoyuan);
    let _ = submit(&mut e3d, 0, GameAction::PlayCard { card: ty2, targets: vec![] });
    assert!(prompt_is(&e3d, "l11_vm_a_fenwei_targets"));
    let _ = answer(&mut e3d, 1, vec![ChoiceOption::Player(1)]);
    drain_ai(&mut e3d, 16);
    assert_eq!(e3d.debug_hp(0), 3, "uncancelled Taoyuan caster recovers +1 HP");
    assert_eq!(e3d.debug_hp(1), 2, "cancelled seat 1 stays at 2 HP");

    // 4. kuangfu + qiaomeng (black Sha discards Horse -> gains Horse from discard)
    let mut e4 = start_clean_4p_play(910_204);
    e4.debug_grant(0, "kuangfu");
    e4.debug_grant(0, "qiaomeng");
    let eq_sword = place_kind(&mut e4, 1, CardKind::Qinggang);
    let eq_horse = place_slot(&mut e4, 1, EquipSlot::HorseMinus);
    e4.debug_take_owned_card_quiet(1, eq_sword);
    e4.state_core_mut().player_mut(1).equips.push(eq_sword);
    e4.debug_take_owned_card_quiet(1, eq_horse);
    e4.state_core_mut().player_mut(1).equips.push(eq_horse);
    let bsha = place_by_suit(&mut e4, 0, "sha", Suit::Spade);
    let _ = submit(&mut e4, 0, GameAction::PlayCard { card: bsha, targets: vec![1] });
    for _ in 0..6 {
        let Some(ch) = e4.open_choice() else { break; };
        match ch.spec.prompt {
            "l11_vm_a_kuangfu_equip" => { let _ = answer(&mut e4, 0, vec![ChoiceOption::Card(eq_sword)]); }
            "l11_vm_a_kuangfu_equip_self" => { let _ = answer(&mut e4, 0, vec![ChoiceOption::Confirm]); }
            "l11_vm_a_qiaomeng_discard" => { let _ = answer(&mut e4, 0, vec![ChoiceOption::Card(eq_horse)]); }
            _ => { drain_ai(&mut e4, 4); }
        }
    }
    assert!(e4.debug_equips(0).contains(&eq_sword), "kuangfu moved sword to own equip zone");
    assert!(e4.debug_hand(0).contains(&eq_horse), "qiaomeng gained discarded Chitu horse");
    assert!(e4.catalog().card_kind(eq_horse).equip_slot() == Some(EquipSlot::HorseMinus));

    // kuangfu with disabled Weapon slot directly discards target weapon; qiaomeng ignores Red Sha
    let mut e4b = start_clean_4p_play(910_215);
    e4b.debug_grant(0, "kuangfu");
    e4b.debug_grant(0, "qiaomeng");
    e4b.state_core_mut().vm_aux.flags.insert((0, "equip_slot_disabled:1".to_string()), 1);
    let eq_w2 = place_kind(&mut e4b, 1, CardKind::Qinggang);
    let eq_h2 = place_slot(&mut e4b, 1, EquipSlot::HorseMinus);
    e4b.debug_take_owned_card_quiet(1, eq_w2);
    e4b.state_core_mut().player_mut(1).equips.push(eq_w2);
    e4b.debug_take_owned_card_quiet(1, eq_h2);
    e4b.state_core_mut().player_mut(1).equips.push(eq_h2);
    let rsha = place_by_suit(&mut e4b, 0, "sha", Suit::Heart);
    let _ = submit(&mut e4b, 0, GameAction::PlayCard { card: rsha, targets: vec![1] });
    assert!(prompt_is(&e4b, "l11_vm_a_kuangfu_equip"));
    let _ = answer(&mut e4b, 0, vec![ChoiceOption::Card(eq_w2)]);
    assert!(!prompt_is(&e4b, "l11_vm_a_kuangfu_equip_self"), "disabled slot must skip self-equip ask");
    assert!(!prompt_is(&e4b, "l11_vm_a_qiaomeng_discard"), "Red Sha must NOT trigger qiaomeng");
    assert!(!e4b.debug_equips(0).contains(&eq_w2));
    assert!(!e4b.debug_equips(1).contains(&eq_w2));
    assert!(e4b.debug_equips(1).contains(&eq_h2));
}

#[test]
fn l11_vm_fix_a_wushen_shenxian_jiezi_bihuo() {
    // 1. wushen: Heart hand cards = distance-unlimited, count-unlimited, unrespondable Sha; Spade Sha limited
    let mut e = start_clean_4p_play(910_205);
    e.debug_grant(0, "wushen");
    let h1 = place_by_suit(&mut e, 0, "shan", Suit::Heart);
    let h2 = place_by_suit(&mut e, 0, "tao", Suit::Heart);
    let sp = place_by_suit(&mut e, 0, "sha", Suit::Spade);
    let shan2 = e.debug_place(2, "shan");
    assert!(!matches!(submit(&mut e, 0, GameAction::PlayCard { card: h1, targets: vec![2] }), CommandOutcome::Rejected(_)));
    assert_eq!(e.debug_hp(2), 4, "Heart Sha via wushen is unrespondable");
    assert!(e.debug_hand(2).contains(&shan2));
    assert!(!matches!(submit(&mut e, 0, GameAction::PlayCard { card: h2, targets: vec![2] }), CommandOutcome::Rejected(_)));
    assert_eq!(e.debug_hp(2), 3);
    assert!(!e.state_core().turn.sha_used_play_phase);
    // Spade Sha cannot reach seat 2 at dist 2
    assert!(matches!(submit(&mut e, 0, GameAction::PlayCard { card: sp, targets: vec![2] }), CommandOutcome::Rejected(_)));

    // wushen mandatory mod.cardname (Heart -> 'sha'):
    // (a) Wounded owner cannot use Heart Tao as Tao or Heart ArcheryAttack as ArcheryAttack; chanyuan HP==1 unlocks them
    let mut e_ws = start_clean_4p_play(910_218);
    e_ws.debug_grant(0, "wushen");
    e_ws.debug_set_hp(0, 3);
    let h_tao0 = place_by_suit(&mut e_ws, 0, "tao", Suit::Heart);
    assert!(
        matches!(submit(&mut e_ws, 0, GameAction::PlayCard { card: h_tao0, targets: vec![] }), CommandOutcome::Rejected(_)),
        "wushen Heart Tao cannot be played as Tao on self"
    );
    let h_wanjian0 = place_kind(&mut e_ws, 0, CardKind::ArcheryAttack);
    assert!(
        matches!(submit(&mut e_ws, 0, GameAction::PlayCard { card: h_wanjian0, targets: vec![] }), CommandOutcome::Rejected(_)),
        "wushen Heart ArcheryAttack becomes Sha and cannot be played without target"
    );
    e_ws.debug_grant(0, "chanyuan");
    e_ws.debug_set_hp(0, 1);
    assert!(!matches!(submit(&mut e_ws, 0, GameAction::PlayCard { card: h_tao0, targets: vec![] }), CommandOutcome::Rejected(_)));
    assert_eq!(e_ws.debug_hp(0), 2);

    // (b) Defender with wushen holding Heart Shan/Tao cannot dodge Sha or ArcheryAttack, nor save in Dying
    let mut e_ws_def = start_clean_4p_play(910_219);
    e_ws_def.debug_grant(1, "wushen");
    let h_shan1 = place_by_suit(&mut e_ws_def, 1, "shan", Suit::Heart);
    let s_atk = place_by_suit(&mut e_ws_def, 0, "sha", Suit::Spade);
    let _ = submit(&mut e_ws_def, 0, GameAction::PlayCard { card: s_atk, targets: vec![1] });
    assert!(e_ws_def.open_choice().is_none(), "wushen defender cannot dodge Sha with Heart Shan");
    assert_eq!(e_ws_def.debug_hp(1), 4);
    assert!(e_ws_def.debug_hand(1).contains(&h_shan1));

    let aoe_arrows = place_kind(&mut e_ws_def, 0, CardKind::ArcheryAttack);
    assert!(!matches!(
        submit(&mut e_ws_def, 0, GameAction::PlayCard { card: aoe_arrows, targets: vec![] }),
        CommandOutcome::Rejected(_)
    ));
    assert!(e_ws_def.open_choice().is_none(), "wushen defender cannot dodge ArcheryAttack with Heart Shan");
    assert_eq!(e_ws_def.debug_hp(1), 3);
    assert!(e_ws_def.debug_hand(1).contains(&h_shan1));

    e_ws_def.debug_set_hp(1, 1);
    let _h_tao1 = place_by_suit(&mut e_ws_def, 1, "tao", Suit::Heart);
    let wq_ws = place_kind(&mut e_ws_def, 0, CardKind::Zhuge);
    let _ = submit(&mut e_ws_def, 0, GameAction::PlayCard { card: wq_ws, targets: vec![] });
    let s_kill = place_by_suit(&mut e_ws_def, 0, "sha", Suit::Spade);
    let _ = submit(&mut e_ws_def, 0, GameAction::PlayCard { card: s_kill, targets: vec![1] });
    assert!(e_ws_def.open_choice().is_none());
    assert!(!e_ws_def.debug_alive(1), "wushen Heart Tao cannot save in Dying");

    // 2. shenxian: off-turn basic discard draws 1 once per turn; ignores Judge-zone basic discard via drlt_xiongluan
    let mut e2 = start_clean_4p_play(910_206);
    e2.debug_grant(2, "shenxian");
    e2.debug_grant(0, "drlt_xiongluan");
    let _j_basic = e2.debug_place_judge(0, "tao");
    let xl = e2.catalog().skill_by_name("drlt_xiongluan").unwrap();
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: xl });
    assert!(prompt_is(&e2, "l11_vm_a_xiongluan_target"));
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(3)]);
    assert!(e2.debug_judge(0).is_empty());
    assert!(!prompt_is(&e2, "l11_vm_a_shenxian_confirm"), "shenxian must ignore Judge-zone basic discard");
    let guohe = place_vanilla_dismantle(&mut e2, 0);
    let _b_sha = e2.debug_place(1, "sha");
    let _b_sha2 = e2.debug_place(1, "sha");
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: guohe, targets: vec![1] });
    assert!(prompt_is(&e2, "l11_vm_a_shenxian_confirm"));
    let h_pre = e2.debug_hand(2).len();
    let _ = answer(&mut e2, 2, vec![ChoiceOption::Confirm]);
    assert_eq!(e2.debug_hand(2).len(), h_pre + 1);
    assert_eq!(e2.debug_hand(1).len(), 1);
    let guohe2 = place_vanilla_dismantle(&mut e2, 0);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: guohe2, targets: vec![1] });
    assert!(!prompt_is(&e2, "l11_vm_a_shenxian_confirm"), "shenxian is once per turn");
    assert_eq!(e2.debug_hand(1).len(), 0);

    // 3. jiezi + bihuo (dyingSaved -> draw 3 & +alive_count distance until round start)
    let mut e3 = start_clean_4p_play(910_207);
    e3.debug_grant(0, "jiezi");
    e3.debug_grant(2, "bihuo");
    let _bing = e3.debug_place_judge(1, "bing");
    let _ = submit(&mut e3, 0, GameAction::NextPhase);
    let _ = submit(&mut e3, 0, GameAction::NextPhase);
    let _ = e3.debug_set_deck_top_by(|cd| cd.suit != Suit::Club);
    let h0_pre = e3.debug_hand(0).len();
    advance_to_phase(&mut e3, 1, Phase::Play);
    assert_eq!(e3.debug_hand(0).len(), h0_pre + 1, "jiezi draws 1 when opponent's draw phase is skipped");

    e3.debug_set_hp(3, 1);
    e3.debug_clear_hand(3);
    let tao3 = e3.debug_place(3, "tao");
    let wq1 = place_kind(&mut e3, 1, CardKind::Qinglong);
    let _ = submit(&mut e3, 1, GameAction::PlayCard { card: wq1, targets: vec![] });
    let killer = e3.debug_place(1, "sha");
    let h3_pre = e3.debug_hand(3).len();
    let _ = submit(&mut e3, 1, GameAction::PlayCard { card: killer, targets: vec![3] });
    assert_eq!(e3.open_choice().map(|c| c.observer), Some(3));
    let _ = answer(&mut e3, 3, vec![ChoiceOption::Card(tao3)]);
    assert!(prompt_is(&e3, "l11_vm_a_bihuo_confirm"));
    let _ = answer(&mut e3, 2, vec![ChoiceOption::Confirm]);
    assert_eq!(e3.debug_hand(3).len(), h3_pre + 2, "bihuo drew 3 (after consuming tao) for saved seat 3");
    assert_eq!(e3.state_core().computed_distance(e3.catalog(), 1, 3), 2 + 4);

    // bihuo distance bonus expires cleanly on next RoundStart
    advance_to_phase(&mut e3, 0, Phase::Play);
    assert!(e3.state_core().vm_aux.flags.get(&(3, "l11_bihuo_distance".to_string())).is_none());
    e3.state_core_mut().player_mut(1).equips.clear();
    e3.state_core_mut().player_mut(3).equips.clear();
    assert_eq!(e3.state_core().computed_distance(e3.catalog(), 1, 3), 2);
}

#[test]
fn l11_vm_fix_a_fenyin_jianying_kaikang_lihuo_manji_ai_and_archive() {
    // 1. fenyin & jianying
    let mut e = start_clean_4p_play(910_208);
    e.debug_grant(0, "fenyin");
    e.debug_grant(0, "jianying");
    let r_sp = place_by_suit(&mut e, 0, "sha", Suit::Heart);
    let r_sp2 = place_by_suit(&mut e, 0, "tao", Suit::Heart); // same suit Heart & red -> jianying yes, fenyin no
    e.debug_set_hp(0, 3);
    e.debug_clear_hand(1);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: r_sp, targets: vec![1] });
    drain_ai(&mut e, 4);
    let h0 = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: r_sp2, targets: vec![] });
    assert!(prompt_is(&e, "l11_vm_a_jianying_confirm"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hand(0).len(), h0, "spent r_sp2, jianying drew 1 -> same count; fenyin didn't prompt");

    let blk_sp = place_by_suit(&mut e, 0, "jiu", Suit::Spade);
    let h0b = e.debug_hand(0).len();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: blk_sp, targets: vec![] });
    assert!(prompt_is(&e, "l11_vm_a_fenyin_confirm"));
    let _ = answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    assert_eq!(e.debug_hand(0).len(), h0b, "fenyin drew 1 on alternating Black after Red");

    // 2. kaikang + lihuo (convert normal Sha -> extra target -> LoseHp(1) after dealing damage)
    let mut e2 = start_clean_4p_play(910_209);
    e2.debug_grant(0, "lihuo");
    e2.debug_grant(1, "kaikang");
    let sha_n = place_kind(&mut e2, 0, CardKind::Sha);
    let eq_gift = place_kind(&mut e2, 1, CardKind::Qinggang);
    e2.debug_clear_hand(3);
    let _ = submit(&mut e2, 0, GameAction::PlayCard { card: sha_n, targets: vec![1] });
    assert!(prompt_is(&e2, "l11_vm_a_lihuo_convert"));
    // NGSA roundtrip mid-lihuo choice
    let mut e2 = roundtrip_ngsa(&e2, taixu_verified());
    let _ = answer(&mut e2, 0, vec![ChoiceOption::Confirm]);
    if prompt_is(&e2, "l11_vm_a_lihuo_extra") {
        let _ = answer(&mut e2, 0, vec![ChoiceOption::Player(3)]);
    }
    assert!(prompt_is(&e2, "l11_vm_a_kaikang_confirm"));
    let _ = answer(&mut e2, 1, vec![ChoiceOption::Confirm]);
    drain_ai(&mut e2, 8);
    assert_eq!(e2.debug_hp(0), 4, "lihuo lost 1 HP after converted Sha dealt damage");
    assert_eq!(e2.debug_hp(3), 4, "lihuo extra target took fire damage");
    let _ = eq_gift;

    // Kaikang give equip -> target equips it
    let mut e3 = start_clean_4p_play(910_210);
    e3.debug_grant(0, "kaikang");
    let eq2 = place_kind(&mut e3, 0, CardKind::Qinggang);
    let sha_kk = e3.debug_place(0, "sha");
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: sha_kk, targets: vec![1] });
    assert!(prompt_is(&e3, "l11_vm_a_kaikang_confirm"));
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Confirm]);
    assert!(prompt_is(&e3, "l11_vm_a_kaikang_give"));
    let _ = answer(&mut e3, 0, vec![ChoiceOption::Card(eq2)]);
    assert!(prompt_is(&e3, "l11_vm_a_kaikang_use_equip"));
    let _ = answer(&mut e3, 1, vec![ChoiceOption::Confirm]);
    drain_ai(&mut e3, 8);
    assert!(e3.debug_equips(1).contains(&eq2), "kaikang target equipped gifted Qinggang");

    // Native FireSha under lihuo: extra target allowed WITHOUT losing 1 HP after damage!
    let mut e2b = start_clean_4p_play(910_213);
    e2b.debug_grant(0, "lihuo");
    let fsha = place_kind(&mut e2b, 0, CardKind::FireSha);
    let play_out = submit(&mut e2b, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    assert!(!matches!(play_out, CommandOutcome::Rejected(_)), "play fsha rejected: {play_out:?}");
    assert!(prompt_is(&e2b, "l11_vm_a_lihuo_extra"));
    let ans_out = answer(&mut e2b, 0, vec![ChoiceOption::Player(3)]);
    assert!(!matches!(ans_out, CommandOutcome::Rejected(_)), "lihuo_extra rejected: {ans_out:?}");
    drain_ai(&mut e2b, 32);
    assert_eq!(e2b.debug_hp(0), 5, "native FireSha via lihuo2 must NOT cost 1 HP");
    assert_eq!(
        e2b.debug_hp(1),
        4,
        "hp1={}, hp3={}, open={:?}",
        e2b.debug_hp(1),
        e2b.debug_hp(3),
        e2b.open_choice().map(|c| (c.observer, c.spec.prompt)),
    );
    assert_eq!(e2b.debug_hp(3), 4);

    // 3. manji + AI public-info rollout
    let mut e4 = start_clean_4p_play(910_211);
    e4.debug_grant(2, "manji");
    e4.debug_set_hp(2, 2);
    e4.debug_set_hp(0, 3);
    let h2_pre = e4.debug_hand(2).len();
    // Trigger kunfen LoseHp on seat 0 cleanly via End phase:
    e4.debug_grant(0, "kunfen");
    let _ = submit(&mut e4, 0, GameAction::NextPhase);
    let _ = submit(&mut e4, 0, GameAction::NextPhase);
    drain_ai(&mut e4, 16);
    assert_eq!(e4.debug_hp(2), 3, "manji recovered 1 HP (2 <= 3 -> 3)");
    assert_eq!(e4.debug_hand(2).len(), h2_pre + 1, "new HP 3 >= target HP 2 -> also drew 1");

    // Verify AI BoundedPlanner cleanly answers hidden enemy hand choices without peeking
    let mut e5 = start_clean_4p_play(910_212);
    e5.debug_grant(0, "wurong");
    let wr5 = e5.catalog().skill_by_name("wurong").unwrap();
    let _p0_c = e5.debug_place(0, "tao");
    let _p1_c1 = e5.debug_place(1, "shan");
    let _p1_c2 = e5.debug_place(1, "tao");
    let ai = BoundedPlanner::new(16);
    let _ = submit(&mut e5, 0, GameAction::ActivateSkill { skill: wr5 });
    for _ in 0..8 {
        if let Some(ch) = e5.open_choice() {
            let obs = ch.observer;
            let act = e5.plan_action(obs, &ai).action;
            let res = submit(&mut e5, obs, act);
            assert!(!matches!(res, CommandOutcome::Rejected(_)));
        } else {
            break;
        }
    }
    let _ = DamageNature::Normal;
}
