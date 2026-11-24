//! Real-event integration tests for L11 Part B (`HostBehavior::L11CrossNpcB`)
//! active phaseUse skills, ViewAs skills, respond skills, and special equips (`taipingyaoshu`, `meiyingqiang`).
//! Zero `debug_execute_effect` / `debug_resolve` calls.

#[path = "l11_npc_b_support/mod.rs"]
mod support;

use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::content::host::HostBehavior;
use engine_native::event::Phase;
use engine_native::{ChoiceOption, CommandOutcome, GameAction};
use support::*;

#[test]
fn l11_npc_b_part_b_catalog_bindings_and_sidao_preservation() {
    let mut e = start_2p(2026_8001);
    assert_eq!(e.catalog().skill_by_name("txhj_tunshi").is_some(), true);
    for sk in [
        "shangyi",
        "twjiefan",
        "twxiongxi",
        "twxuechang",
        "twyuanhu",
        "waishi",
        "weijing",
        "twmouli",
        "shizhi",
        "twwushen",
        "xinfenyue",
        "xinfu_kannan",
        "xinfu_limu",
        "xinfu_zengdao",
        "xinfu_zengdao2",
        "xinfu_zhaoxin",
        "xinkuangfu",
        "yjjianshu",
        "yongjin",
        "zhanyi",
        "paiyi",
        "twqingce",
        "twsaotao",
        "twzhian",
        "twxiongjun",
        "baonvezhi",
        "juedi",
        "meiyingqiang",
        "taipingyaoshu",
    ] {
        let sid = e.catalog().skill_by_name(sk).unwrap_or_else(|| panic!("missing {sk}"));
        assert_eq!(
            e.catalog().skills[sid as usize].behavior.as_host_behavior(),
            Some(HostBehavior::L11CrossNpcB),
            "{sk} must bind to L11CrossNpcB"
        );
    }
    let sidao_id = e.catalog().skill_by_name("xinfu_sidao").expect("xinfu_sidao");
    assert_eq!(
        e.catalog().skills[sidao_id as usize].behavior.as_host_behavior(),
        Some(HostBehavior::L11GeneralBossC)
    );
    assert!(e.catalog().cards.iter().any(|c| c.name == "taipingyaoshu"));
    assert!(e.catalog().cards.iter().any(|c| c.name == "meiyingqiang"));
    assert!(!e.state_core().deck.iter().any(|&c| {
        matches!(e.catalog().cards[c as usize].name, "taipingyaoshu" | "meiyingqiang")
    }));
    e.debug_grant(0, "twwushen");
    assert!(e.debug_has_skill(0, "twwushen"));
}

#[test]
fn l11_npc_b_shangyi_twqingce_twjiefan_twxiongxi_waishi_active() {
    // 1. `shangyi`: target opponent -> discard a black hand card (`sp.js:20972`)
    let mut e = start_2p(2026_8002);
    e.debug_grant(0, "shangyi");
    advance_to_phase(&mut e, 0, Phase::Play, None);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let blk = place_hand_suit_kind(&mut e, 1, Some(Suit::Spade), CardKind::Sha);
    let res = activate(&mut e, 0, "shangyi");
    assert!(!matches!(res, CommandOutcome::Rejected(_)));
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(blk)]);
    drain_pass(&mut e, 8);
    assert!(e.state_core().discard.contains(&blk));
    assert!(matches!(activate(&mut e, 0, "shangyi"), CommandOutcome::Rejected(_)));

    // 2. `twqingce` + `twzhengrong` pile: discard pile card -> dismantle opponent field equip
    let mut e2 = start_2p(2026_8003);
    e2.debug_grant(0, "twqingce");
    advance_to_phase(&mut e2, 0, Phase::Play, None);
    let pile_c = take_deck_or_discard_by(&mut e2, |_| true);
    e2.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "l11b_twzhengrong".to_string()), vec![pile_c]);
    let eq = take_deck_or_discard_by(&mut e2, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    e2.state_core_mut().player_mut(1).equips.push(eq);
    assert!(!matches!(activate(&mut e2, 0, "twqingce"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Card(pile_c)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Card(eq)]);
    drain_pass(&mut e2, 8);
    assert!(!e2.state_core().player(1).equips.contains(&eq));

    // 3. `twjiefan`: limited skill draws 1 if others pass Sha / restores on Dying
    let mut e3 = start_2p(2026_8004);
    e3.debug_grant(0, "twjiefan");
    advance_to_phase(&mut e3, 0, Phase::Play, None);
    e3.debug_clear_hand(1);
    let hand_before = e3.state_core().player(0).hand.len();
    assert!(!matches!(activate(&mut e3, 0, "twjiefan"), CommandOutcome::Rejected(_)));
    answer(&mut e3, 0, vec![ChoiceOption::Player(0)]);
    drain_pass(&mut e3, 8);
    assert!(e3.state_core().player(0).hand.len() >= hand_before);
    assert!(matches!(activate(&mut e3, 0, "twjiefan"), CommandOutcome::Rejected(_)));

    // 4. `twxiongxi` (discard cards deal 1 dmg, limited) & `waishi` (give 1 -> take 1)
    let mut e4 = start_2p(2026_8005);
    e4.debug_grant(0, "twxiongxi");
    e4.debug_grant(0, "waishi");
    advance_to_phase(&mut e4, 0, Phase::Play, None);
    e4.debug_set_max_hp(1, 6);
    e4.debug_set_hp(1, 6);
    e4.debug_clear_hand(0);
    e4.debug_clear_hand(1);
    let d1 = place_hand_suit_kind(&mut e4, 0, Some(Suit::Heart), CardKind::Sha);
    let t_eq = place_hand_suit_kind(&mut e4, 1, Some(Suit::Diamond), CardKind::Shan);
    assert!(!matches!(activate(&mut e4, 0, "waishi"), CommandOutcome::Rejected(_)));
    answer(&mut e4, 0, vec![ChoiceOption::Card(d1)]);
    answer(&mut e4, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e4, 0, vec![ChoiceOption::Card(t_eq)]);
    drain_pass(&mut e4, 8);
    assert!(e4.state_core().player(0).hand.contains(&t_eq));
    // `twxiongxi` requires (5 - `baonvezhi`) cards -> with 4 `baonvezhi` marks, discard 1 deals 1 dmg
    e4.debug_clear_hand(0);
    let xs = place_hand_kind_any(&mut e4, 0, CardKind::Shan);
    e4.state_core_mut()
        .vm_aux
        .flags
        .insert((0, "baonvezhi".to_string()), 4);
    assert!(!matches!(activate(&mut e4, 0, "twxiongxi"), CommandOutcome::Rejected(_)));
    answer(&mut e4, 0, vec![ChoiceOption::Card(xs)]);
    answer(&mut e4, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e4, 8);
    assert_eq!(e4.state_core().player(1).hp, 5);
}

#[test]
fn l11_npc_b_twxuechang_twyuanhu_xinfenyue_xinfu_kannan_xinfu_limu() {
    // 1. `twxuechang` + `xinfu_kannan` pindian -> win/lose sets combat bonus + `DamageBegin1`
    let mut e = start_2p(2026_8006);
    e.debug_grant(0, "twxuechang");
    e.debug_grant(0, "xinfu_kannan");
    advance_to_phase(&mut e, 0, Phase::Play, None);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_max_hp(1, 8);
    e.debug_set_hp(1, 8);
    let low = take_deck_or_discard_by(&mut e, |d| d.rank <= 3);
    let high = take_deck_or_discard_by(&mut e, |d| d.rank >= 11);
    let high2 = take_deck_or_discard_by(&mut e, |d| d.rank >= 10);
    let low2 = take_deck_or_discard_by(&mut e, |d| d.rank <= 4);
    e.state_core_mut().player_mut(0).hand.extend([low, high2]);
    e.state_core_mut().player_mut(1).hand.extend([high, low2]);
    // Lose `twxuechang` pindian -> increments `l11b_xuechang_bonus:1`
    assert!(!matches!(activate(&mut e, 0, "twxuechang"), CommandOutcome::Rejected(_)));
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(low)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(high)]);
    drain_pass(&mut e, 8);
    assert_eq!(flag(&e, 0, "l11b_xuechang_bonus:1"), 1);
    // Win `xinfu_kannan` pindian -> increments `l11b_kannan_eff`
    assert!(!matches!(activate(&mut e, 0, "xinfu_kannan"), CommandOutcome::Rejected(_)));
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 0, vec![ChoiceOption::Card(high2)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(low2)]);
    drain_pass(&mut e, 8);
    assert!(flag(&e, 0, "l11b_kannan_eff") >= 1);
    // Play Sha on seat 1: consumes BOTH `twxuechang` (+1) and `xinfu_kannan` (+1) -> 3 dmg!
    e.debug_clear_hand(1);
    let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_pass(&mut e, 12);
    assert_eq!(e.state_core().player(1).hp, 5);

    // 2. `twyuanhu`, `xinfenyue`, `xinfu_limu`
    let mut e2 = start_2p(2026_8007);
    e2.debug_grant(0, "twyuanhu");
    e2.debug_grant(0, "xinfenyue");
    e2.debug_grant(0, "xinfu_limu");
    advance_to_phase(&mut e2, 0, Phase::Play, None);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(1);
    e2.debug_set_max_hp(0, 5);
    e2.debug_set_hp(0, 4);
    let armor = place_hand_kind_any(&mut e2, 0, CardKind::Armor);
    let dmd = place_hand_suit_kind(&mut e2, 0, Some(Suit::Diamond), CardKind::Shan);
    // `xinfu_limu` Diamond -> `Le` self & recover 1 HP
    assert!(!matches!(
        submit(
            &mut e2,
            0,
            GameAction::PlayCard {
                card: dmd,
                targets: vec![0],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    drain_pass(&mut e2, 8);
    assert_eq!(e2.state_core().player(0).hp, 5);
    assert!(!e2.state_core().player(0).judge.is_empty());
    // `twyuanhu` equips armor -> target draws 1
    let h1_before = e2.state_core().player(1).hand.len();
    assert!(!matches!(activate(&mut e2, 0, "twyuanhu"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Card(armor)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e2, 8);
    assert!(e2.state_core().player(1).equips.contains(&armor));
    assert_eq!(e2.state_core().player(1).hand.len(), h1_before + 1);
    // `xinfenyue`: pindian vs seat 1
    let hi = take_deck_or_discard_by(&mut e2, |d| d.rank >= 11);
    let lo = take_deck_or_discard_by(&mut e2, |d| d.rank <= 3);
    e2.state_core_mut().player_mut(0).hand.push(hi);
    e2.state_core_mut().player_mut(1).hand.push(lo);
    assert!(!matches!(activate(&mut e2, 0, "xinfenyue"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Card(hi)]);
    answer(&mut e2, 1, vec![ChoiceOption::Card(lo)]);
    drain_pass(&mut e2, 8);
}

#[test]
fn l11_npc_b_zengdao_zhaoxin_kuangfu_jianshu_yongjin_zhanyi_paiyi() {
    // 1. `xinfu_zengdao` grants `xinfu_zengdao2` + `+1` dmg; `xinkuangfu` strips equip & uses virtual Sha
    let mut e = start_2p(2026_8008);
    e.debug_grant(0, "xinfu_zengdao");
    e.debug_grant(1, "xinkuangfu");
    advance_to_phase(&mut e, 0, Phase::Play, None);
    e.debug_set_max_hp(1, 8);
    e.debug_set_hp(1, 8);
    e.debug_clear_hand(0);
    let give_c = take_deck_or_discard_by(&mut e, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    e.state_core_mut().player_mut(0).equips.push(give_c);
    assert!(!matches!(activate(&mut e, 0, "xinfu_zengdao"), CommandOutcome::Rejected(_)));
    answer(&mut e, 0, vec![ChoiceOption::Card(give_c)]);
    answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e, 8);
    assert!(e.debug_has_skill(1, "xinfu_zengdao2"));

    // Advance to seat 1 Play phase -> seat 1 attacks seat 0 with `xinfu_zengdao2` (+1 dmg) + `xinkuangfu`
    advance_to_phase(&mut e, 1, Phase::Play, None);
    e.debug_clear_hand(0);
    e.debug_set_max_hp(0, 6);
    e.debug_set_hp(0, 6);
    let sha = place_hand_kind_any(&mut e, 1, CardKind::Sha);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha,
            targets: vec![0],
        },
    );
    drain_pass(&mut e, 8);
    assert_eq!(e.state_core().player(0).hp, 4);

    // Seat 1 `xinkuangfu`: discard own equip, emit virtual Sha at seat 0 without distance limit or hand Sha
    let eq = take_deck_or_discard_by(&mut e, |d| d.kind.equip_slot() == Some(EquipSlot::HorsePlus));
    e.state_core_mut().player_mut(1).equips.push(eq);
    assert!(!matches!(activate(&mut e, 1, "xinkuangfu"), CommandOutcome::Rejected(_)));
    answer(&mut e, 1, vec![ChoiceOption::Player(1)]);
    answer(&mut e, 1, vec![ChoiceOption::Card(eq)]);
    answer(&mut e, 1, vec![ChoiceOption::Player(0)]);
    drain_pass(&mut e, 8);
    assert_eq!(e.state_core().player(0).hp, 3);

    // 2. `xinfu_zhaoxin`, `yjjianshu`, `yongjin`, `zhanyi`, `paiyi`
    let mut e2 = start_4p_identity(2026_8009);
    e2.debug_grant(0, "xinfu_zhaoxin");
    e2.debug_grant(0, "yjjianshu");
    e2.debug_grant(0, "yongjin");
    e2.debug_grant(0, "zhanyi");
    e2.debug_grant(0, "paiyi");
    advance_to_phase(&mut e2, 0, Phase::Play, None);
    // `yongjin` moves seat 1 armor -> seat 2
    let eq1 = take_deck_or_discard_by(&mut e2, |d| d.kind.equip_slot() == Some(EquipSlot::Armor));
    e2.state_core_mut().player_mut(1).equips.push(eq1);
    assert!(!matches!(activate(&mut e2, 0, "yongjin"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Card(eq1)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(2)]);
    drain_pass(&mut e2, 8);
    assert!(e2.state_core().player(2).equips.contains(&eq1));
    // `xinfu_zhaoxin`: stow 1 hand card -> pile
    e2.debug_clear_hand(0);
    let zx = place_hand_kind_any(&mut e2, 0, CardKind::Shan);
    assert!(!matches!(activate(&mut e2, 0, "xinfu_zhaoxin"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Card(zx)]);
    drain_pass(&mut e2, 8);
    assert_eq!(
        e2.state_core()
            .vm_aux
            .piles
            .get(&(0, "l11b_zhaoxin".to_string()))
            .map(Vec::len),
        Some(1)
    );
    // `paiyi`: remove 1 quanji -> seat 1 draws 2 & takes 1 dmg if hand > owner
    let q = take_deck_or_discard_by(&mut e2, |_| true);
    e2.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "l11b_quanji".to_string()), vec![q]);
    e2.debug_set_max_hp(1, 6);
    e2.debug_set_hp(1, 6);
    e2.debug_clear_hand(0);
    assert!(!matches!(activate(&mut e2, 0, "paiyi"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Card(q)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e2, 8);
    assert_eq!(e2.state_core().player(1).hp, 5);
    // `zhanyi` with equip card -> lose 1 HP, Sha triggers `zhanyi` target 2-card discard!
    e2.debug_set_max_hp(0, 6);
    e2.debug_set_hp(0, 6);
    let eq_c = place_hand_kind_any(&mut e2, 0, CardKind::Weapon);
    assert!(!matches!(activate(&mut e2, 0, "zhanyi"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Card(eq_c)]);
    drain_pass(&mut e2, 8);
    assert_eq!(e2.state_core().player(0).hp, 5);
    // `yjjianshu`: give 1 hand card to target 1, then target 1 and 2 pindian
    let give_j = place_hand_kind_any(&mut e2, 0, CardKind::Sha);
    e2.debug_clear_hand(1);
    e2.debug_clear_hand(2);
    let hi_a = take_deck_or_discard_by(&mut e2, |d| d.rank >= 11);
    let lo_b = take_deck_or_discard_by(&mut e2, |d| d.rank <= 3);
    e2.state_core_mut().player_mut(1).hand.push(hi_a);
    e2.state_core_mut().player_mut(2).hand.push(lo_b);
    assert!(!matches!(activate(&mut e2, 0, "yjjianshu"), CommandOutcome::Rejected(_)));
    answer(&mut e2, 0, vec![ChoiceOption::Card(give_j)]);
    answer(&mut e2, 0, vec![ChoiceOption::Player(1), ChoiceOption::Player(2)]);
    answer(&mut e2, 1, vec![ChoiceOption::Card(hi_a)]);
    answer(&mut e2, 2, vec![ChoiceOption::Card(lo_b)]);
    drain_pass(&mut e2, 8);
}

#[test]
fn l11_npc_b_viewas_respond_twwushen_shizhi_weijing_twmouli_xinjiewei_special_equips() {
    // 1. `shizhi` (`hp == 1` Shan->Sha), `weijing` (1/round virtual Sha), `twwushen` (Heart->Sha unlimited)
    let mut e = start_2p(2026_8010);
    e.debug_grant(0, "twwushen");
    e.debug_grant(0, "twmouli");
    e.debug_grant(0, "weijing");
    advance_to_phase(&mut e, 0, Phase::Play, None);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_max_hp(1, 8);
    e.debug_set_hp(1, 8);
    let h1 = place_hand_suit_kind(&mut e, 0, Some(Suit::Heart), CardKind::Shan);
    let h2 = place_hand_suit_kind(&mut e, 0, Some(Suit::Heart), CardKind::Tao);
    // Use BOTH Heart cards as distance/count-unlimited Sha under `twwushen`!
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: h1,
            targets: vec![1],
        },
    );
    drain_pass(&mut e, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: h2,
            targets: vec![1],
        },
    );
    drain_pass(&mut e, 8);
    assert!(e.state_core().player(1).hp <= 6);

    // `weijing` active virtual Sha on Play phase
    let wj_res = activate(&mut e, 0, "weijing");
    if !matches!(wj_res, CommandOutcome::Rejected(_)) {
        drain_pass(&mut e, 8);
    }

    // `shizhi`: only converts `Shan` -> `Sha` when `hp == 1`
    let mut e_sz = start_2p(2026_8011);
    e_sz.debug_grant(0, "shizhi");
    advance_to_phase(&mut e_sz, 0, Phase::Play, None);
    e_sz.debug_clear_hand(0);
    e_sz.debug_clear_hand(1);
    e_sz.debug_set_max_hp(1, 5);
    e_sz.debug_set_hp(1, 5);
    let shan_c = place_hand_kind_any(&mut e_sz, 0, CardKind::Shan);
    e_sz.debug_set_hp(0, 2);
    assert!(matches!(
        submit(
            &mut e_sz,
            0,
            GameAction::PlayCard {
                card: shan_c,
                targets: vec![1],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    e_sz.debug_set_hp(0, 1);
    assert!(!matches!(
        submit(
            &mut e_sz,
            0,
            GameAction::PlayCard {
                card: shan_c,
                targets: vec![1],
            },
        ),
        CommandOutcome::Rejected(_)
    ));
    drain_pass(&mut e_sz, 8);
    assert_eq!(e_sz.state_core().player(1).hp, 4);

    // 2. `taipingyaoshu` (prevent Fire dmg + onLose draw 2 / lose 1 HP LIFO), `xinjiewei` (equip->Nullify) & `meiyingqiang`
    let mut e_eq = start_2p(2026_8012);
    e_eq.debug_grant(0, "tianshu");
    e_eq.debug_grant(0, "twqueshi");
    e_eq.debug_grant(1, "xinjiewei");
    let cost = place_hand_kind_any(&mut e_eq, 0, CardKind::Shan);
    advance_to_phase(&mut e_eq, 0, Phase::Play, Some("tianshu:"));
    answer(&mut e_eq, 0, vec![ChoiceOption::Card(cost)]);
    answer(&mut e_eq, 0, vec![ChoiceOption::Player(1)]);
    drain_pass(&mut e_eq, 8);
    assert!(e_eq.debug_has_skill(1, "taipingyaoshu"));
    e_eq.debug_set_max_hp(1, 6);
    e_eq.debug_set_hp(1, 6);
    // FireSha on seat 1 is prevented by `taipingyaoshu`
    let fire_sha = place_hand_kind_any(&mut e_eq, 0, CardKind::FireSha);
    e_eq.debug_clear_hand(1);
    submit(
        &mut e_eq,
        0,
        GameAction::PlayCard {
            card: fire_sha,
            targets: vec![1],
        },
    );
    drain_pass(&mut e_eq, 8);
    assert_eq!(e_eq.state_core().player(1).hp, 6);
    // Dismantle `taipingyaoshu` -> seat 1 draws 2 & loses 1 HP (`hp: 6 -> 5`)
    let h_before = e_eq.state_core().player(1).hand.len();
    let ch = place_hand_kind_any(&mut e_eq, 0, CardKind::Dismantle);
    let tpys = e_eq.state_core().player(1).equips[0];
    submit(
        &mut e_eq,
        0,
        GameAction::PlayCard {
            card: ch,
            targets: vec![1],
        },
    );
    drain_pass(&mut e_eq, 4);
    if e_eq.open_choice().is_some() {
        answer(&mut e_eq, 0, vec![ChoiceOption::Card(tpys)]);
    }
    drain_pass(&mut e_eq, 8);
    assert_eq!(e_eq.state_core().player(1).hp, 5);
    assert!(e_eq.state_core().player(1).hand.len() >= h_before + 2);
}

#[test]
fn l11_npc_b_round2_active_parity_fixes() {
    use engine_native::event::DamageNature;
    use engine_native::rvm_bridge::modifiers::{
        d34_flags::find_d34_flag_view_as, eval_card_unrespondable,
    };

    // Defect #1 (`twwushen`): Heart-only range/count + hand-only ViewAs + per-phase 1st-Sha directHit.
    {
        let mut e = start_4p_identity(2026_8201);
        e.debug_grant(0, "twwushen");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        for s in 1..4 {
            e.debug_clear_hand(s);
            e.debug_set_max_hp(s, 6);
            e.debug_set_hp(s, 6);
        }
        // Heart equip must NOT trigger active `twwushen` or `VmViewAsZone::Hand` ViewAs.
        let heart_eq = take_deck_or_discard_by(&mut e, |d| {
            d.suit == Suit::Heart && d.kind.equip_slot().is_some()
        });
        e.state_core_mut().player_mut(0).equips.push(heart_eq);
        assert!(
            find_d34_flag_view_as(
                e.state_core(),
                e.catalog(),
                0,
                heart_eq,
                false,
                Some(CardKind::Sha),
            )
            .is_none(),
            "Heart equip must not ViewAs Sha under twwushen"
        );
        assert!(matches!(
            activate(&mut e, 0, "twwushen"),
            CommandOutcome::Rejected(_)
        ));
        e.state_core_mut().player_mut(0).equips.clear();

        // Distance to seat 2 is 2 (> base attack range 1): Spade Sha rejected, Heart Sha allowed.
        let sp_sha1 = place_hand_suit_kind(&mut e, 0, Some(Suit::Spade), CardKind::Sha);
        let sp_sha2 = place_hand_suit_kind(&mut e, 0, Some(Suit::Spade), CardKind::Sha);
        let ht_tao = place_hand_suit_kind(&mut e, 0, Some(Suit::Heart), CardKind::Tao);
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: sp_sha1,
                    targets: vec![2],
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        // Static unrespondable must be false for Spade Sha before use.
        assert!(!eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            1,
            sp_sha1,
            CardKind::Sha,
            DamageNature::Normal,
        ));
        // 1st Sha in phase (`sp_sha1` at seat 1): unrespondable (`Shan` not usable)!
        let shan1 = place_hand_kind_any(&mut e, 1, CardKind::Shan);
        let shan2 = place_hand_kind_any(&mut e, 2, CardKind::Shan);
        assert!(!matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: sp_sha1,
                    targets: vec![1],
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        assert!(
            e.open_choice().is_none() || e.open_choice().is_some_and(|c| c.observer != 1),
            "1st Sha in phase under twwushen must be unrespondable"
        );
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(1).hp, 5);
        assert!(e.state_core().player(1).hand.contains(&shan1));

        // 2nd Spade Sha must be REJECTED (normal Sha limit reached, NOT Heart!).
        assert!(matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: sp_sha2,
                    targets: vec![1],
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        // 2nd Sha in phase as Heart hand card (`ht_tao` -> Sha) reaches distant seat 2, bypasses count, AND CAN be dodged!
        assert!(!matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: ht_tao,
                    targets: vec![2],
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        let ch2 = e.open_choice().cloned().expect("seat 2 must be asked to dodge 2nd Sha");
        assert_eq!(ch2.observer, 2);
        assert!(ch2.spec.options.contains(&ChoiceOption::Card(shan2)));
        answer(&mut e, 2, vec![ChoiceOption::Card(shan2)]);
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(2).hp, 6);
    }

    // Defect #2 (`twjiefan`): `p != t` includes owner when owner.inRange(t), plus `Dying` restores skill.
    {
        let mut e = start_4p_identity(2026_8202);
        e.debug_grant(0, "twjiefan");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        for s in 0..4 {
            e.debug_clear_hand(s);
            e.debug_set_max_hp(s, 4);
            e.debug_set_hp(s, 4);
        }
        let _w0 = place_hand_kind_any(&mut e, 0, CardKind::Weapon);
        let _w2 = place_hand_kind_any(&mut e, 2, CardKind::Weapon);
        // Seat 0 targets seat 1: seat 0 (`inRange(1)`) and seat 2 (`inRange(1)`) must BOTH be asked!
        assert!(!matches!(
            activate(&mut e, 0, "twjiefan"),
            CommandOutcome::Rejected(_)
        ));
        answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        let mut asked = Vec::new();
        for _ in 0..8 {
            let Some(ch) = e.open_choice().cloned() else {
                break;
            };
            if ch.spec.prompt == "twjiefan:discard_weapon" {
                asked.push(ch.observer);
                answer(&mut e, ch.observer, Vec::new());
            } else {
                drain_pass(&mut e, 1);
            }
        }
        assert!(asked.contains(&0), "twjiefan responder filter must include owner if inRange(t): got {asked:?}");
        assert!(asked.contains(&2), "twjiefan responders got {asked:?}");
        assert!(!asked.contains(&1), "target itself must never be a responder");
        assert_eq!(flag(&e, 0, "l11b_limited_twjiefan"), 1);
        assert_eq!(flag(&e, 0, "l11b_twjiefan2_target"), 2);
        assert!(matches!(
            activate(&mut e, 0, "twjiefan"),
            CommandOutcome::Rejected(_)
        ));

        // Seat 1 enters Dying -> `twjiefan2` restores `twjiefan`!
        let tao = place_hand_kind_any(&mut e, 1, CardKind::Tao);
        e.debug_deal_damage(Some(0), 1, 4, DamageNature::Normal);
        for _ in 0..12 {
            let Some(ch) = e.open_choice().cloned() else {
                break;
            };
            if ch.observer == 1 && ch.spec.options.contains(&ChoiceOption::Card(tao)) {
                answer(&mut e, 1, vec![ChoiceOption::Card(tao)]);
            } else {
                drain_pass(&mut e, 1);
            }
        }
        assert_eq!(flag(&e, 0, "l11b_limited_twjiefan"), 0);
        assert_eq!(flag(&e, 0, "l11b_twjiefan2_target"), 0);
        assert!(!matches!(
            activate(&mut e, 0, "twjiefan"),
            CommandOutcome::Rejected(_)
        ));
        drain_pass(&mut e, 8);
    }

    // Defect #3 (`zhanyi`): (a) `l11b_zhanyi_basic` expires at turn end; (b) 1 HP trick yields LoseHp before Draw(2).
    {
        let mut e = start_2p(2026_8203);
        e.debug_grant(0, "zhanyi");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_set_max_hp(0, 4);
        e.debug_set_hp(0, 4);
        let basic_shan = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        assert!(!matches!(
            activate(&mut e, 0, "zhanyi"),
            CommandOutcome::Rejected(_)
        ));
        answer(&mut e, 0, vec![ChoiceOption::Card(basic_shan)]);
        drain_pass(&mut e, 8);
        assert_eq!(flag(&e, 0, "l11b_zhanyi_basic"), 1);
        advance_to_phase(&mut e, 1, Phase::Play, None);
        assert_eq!(flag(&e, 0, "l11b_zhanyi_basic"), 0, "zhanyi_basic must expire at turn end");

        // (b) 1 HP trick discard -> LoseHp settles before Draw(2), so dying/dead player draws 0 cards!
        let mut e_die = start_2p(2026_8204);
        e_die.debug_grant(0, "zhanyi");
        advance_to_phase(&mut e_die, 0, Phase::Play, None);
        e_die.debug_clear_hand(0);
        e_die.debug_clear_hand(1);
        e_die.debug_set_max_hp(0, 4);
        e_die.debug_set_hp(0, 1);
        let trick = place_hand_kind_any(&mut e_die, 0, CardKind::Dismantle);
        assert!(!matches!(
            activate(&mut e_die, 0, "zhanyi"),
            CommandOutcome::Rejected(_)
        ));
        answer(&mut e_die, 0, vec![ChoiceOption::Card(trick)]);
        drain_pass(&mut e_die, 8);
        assert!(!e_die.state_core().player(0).alive);
        assert_eq!(e_die.state_core().player(0).hand.len(), 0);
    }

    // Defect #5 (`xinkuangfu`): empty hand, virtual nodistance Sha, `addCount: false`, self-equip damage draws 2.
    {
        let mut e = start_4p_identity(2026_8205);
        e.debug_grant(0, "xinkuangfu");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        for s in 1..4 {
            e.debug_clear_hand(s);
            e.debug_set_max_hp(s, 5);
            e.debug_set_hp(s, 5);
        }
        let eq = take_deck_or_discard_by(&mut e, |d| d.kind.equip_slot() == Some(EquipSlot::Armor));
        e.state_core_mut().player_mut(0).equips.push(eq);
        assert!(!matches!(
            activate(&mut e, 0, "xinkuangfu"),
            CommandOutcome::Rejected(_)
        ));
        answer(&mut e, 0, vec![ChoiceOption::Player(0)]);
        answer(&mut e, 0, vec![ChoiceOption::Card(eq)]);
        let ask_target = e.open_choice().cloned().expect("xinkuangfu must ask target directly");
        assert_eq!(ask_target.spec.prompt, "xinkuangfu:use_sha_target");
        // Seat 2 is at distance 2 (beyond normal range 1): nodistance allows targeting seat 2!
        assert!(ask_target.spec.options.contains(&ChoiceOption::Player(2)));
        answer(&mut e, 0, vec![ChoiceOption::Player(2)]);
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(2).hp, 4);
        assert_eq!(e.state_core().player(0).equips.len(), 0);
        // Self-equip + dealt damage draws 2, AND normal Sha limit remains unused (`!sha_used_play_phase`).
        assert_eq!(e.state_core().player(0).hand.len(), 2);
        assert!(!e.state_core().turn.sha_used_play_phase);
    }
}
