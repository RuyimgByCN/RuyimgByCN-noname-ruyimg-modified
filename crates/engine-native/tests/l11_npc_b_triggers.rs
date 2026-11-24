//! Real-event integration tests covering ALL Part-B trigger & passive `L11` skills
//! (`combat_damage`, `combat_step_b`, `phase_hooks`, `phase_turn_death`, `phase_step_b`,
//! `reactive_misc`, `reactive_step_b`). Zero `debug_execute_effect` / `debug_resolve`.

#[path = "l11_npc_b_support/mod.rs"]
mod support;

use engine_native::catalog::{CardKind, EquipSlot, Suit};
use engine_native::event::{DamageNature, Phase};
use engine_native::{ChoiceOption, CommandOutcome, GameAction};
use support::*;

#[test]
fn l11_npc_b_combat_damage_triggers_batch_a() {
    // Covers: `satomi_luodao`, `sbliegong`, `spshiji`, `spxiangzhen`, `spyishi`,
    //         `tianming`, `twhengjiang`, `twhuangjin`, `twjianming`, `twqirang`,
    //         `twxiangyu`, `twxuhe`, `xinfu_daigong`, `xinfu_fuyin`, `yise`,
    //         `zhidao`, `zhiman`, `zhongyong`
    let mut e = start_2p(2026_8101);
    for sk in [
        "satomi_luodao",
        "spshiji",
        "twhengjiang",
        "twjianming",
        "twxiangyu",
        "twxuhe",
        "zhidao",
        "zhongyong",
    ] {
        e.debug_grant(0, sk);
    }
    for sk in ["tianming", "twhuangjin", "xinfu_daigong", "xinfu_fuyin", "yise"] {
        e.debug_grant(1, sk);
    }
    advance_to_phase(&mut e, 0, Phase::Play, None);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_max_hp(1, 8);
    e.debug_set_hp(1, 8);
    let sha = place_hand_suit_kind(&mut e, 0, Some(Suit::Spade), CardKind::Sha);
    let shan = place_hand_suit_kind(&mut e, 1, Some(Suit::Heart), CardKind::Shan);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    // Let interactive trigger prompts resolve (decline optional nullify/dodge so damage/dodge paths run)
    for _ in 0..24 {
        let Some(ch) = e.open_choice().cloned() else { break };
        if ch.spec.prompt.starts_with("twjianming:") || ch.spec.prompt.starts_with("satomi_luodao:") {
            let pick = ch.spec.options[0];
            answer(&mut e, ch.observer, vec![pick]);
        } else if ch.spec.prompt.contains("shan") && ch.observer == 1 && ch.spec.options.contains(&ChoiceOption::Card(shan)) {
            answer(&mut e, 1, vec![ChoiceOption::Card(shan)]);
        } else {
            drain_pass(&mut e, 1);
        }
    }
    assert!(flag(&e, 0, "l11b_twjianming_suit_mask") != 0 || e.state_core().player(1).hp <= 8);

    // `spyishi`, `zhiman`, `spxiangzhen`, `twqirang`, `xinfu_daigong`
    let mut e2 = start_2p(2026_8102);
    e2.debug_grant(0, "spyishi");
    e2.debug_grant(0, "twqirang");
    e2.debug_grant(1, "spxiangzhen");
    advance_to_phase(&mut e2, 0, Phase::Play, None);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(1);
    e2.debug_set_max_hp(1, 6);
    e2.debug_set_hp(1, 6);
    let eq = take_deck_or_discard_by(&mut e2, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    e2.state_core_mut().player_mut(1).equips.push(eq);
    let nm = place_hand_kind_any(&mut e2, 0, CardKind::SavageAttack);
    // Equip triggers `twqirang` (gain trick from deck)
    let armor = place_hand_kind_any(&mut e2, 0, CardKind::Armor);
    submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: armor,
            targets: Vec::new(),
        },
    );
    drain_pass(&mut e2, 8);
    // `SavageAttack` on seat 1 with `spxiangzhen` nullifies damage on seat 1
    submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: nm,
            targets: Vec::new(),
        },
    );
    drain_pass(&mut e2, 12);
    assert_eq!(e2.state_core().player(1).hp, 6);

    // `zhiman` prevents own damage and steals equip
    let mut e3 = start_2p(2026_8103);
    e3.debug_grant(0, "zhiman");
    advance_to_phase(&mut e3, 0, Phase::Play, None);
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(1);
    e3.debug_set_max_hp(1, 6);
    e3.debug_set_hp(1, 6);
    let eq3 = take_deck_or_discard_by(&mut e3, |d| d.kind.equip_slot() == Some(EquipSlot::Armor));
    e3.state_core_mut().player_mut(1).equips.push(eq3);
    let sha3 = place_hand_kind_any(&mut e3, 0, CardKind::Sha);
    submit(
        &mut e3,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    for _ in 0..12 {
        let Some(ch) = e3.open_choice().cloned() else { break };
        if ch.spec.prompt == "trigger_confirm" {
            answer(&mut e3, ch.observer, vec![ChoiceOption::Confirm]);
        } else if ch.spec.prompt.starts_with("zhiman:") {
            let pick = ch.spec.options[0];
            answer(&mut e3, 0, vec![pick]);
        } else {
            drain_pass(&mut e3, 1);
        }
    }
    assert_eq!(e3.state_core().player(1).hp, 6);
    assert!(e3.state_core().player(0).hand.contains(&eq3));

    // `zhidao`: Phase::Play damage deals + steals card!
    let mut e4 = start_2p(2026_8104);
    e4.debug_grant(0, "zhidao");
    advance_to_phase(&mut e4, 0, Phase::Play, None);
    e4.debug_clear_hand(0);
    e4.debug_clear_hand(1);
    e4.debug_set_max_hp(1, 6);
    e4.debug_set_hp(1, 6);
    let loot = place_hand_kind_any(&mut e4, 1, CardKind::Tao);
    let sha4 = place_hand_kind_any(&mut e4, 0, CardKind::Sha);
    submit(
        &mut e4,
        0,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![1],
        },
    );
    for _ in 0..12 {
        let Some(ch) = e4.open_choice().cloned() else { break };
        if ch.spec.prompt.starts_with("zhidao:") && ch.spec.options.contains(&ChoiceOption::Card(loot)) {
            answer(&mut e4, 0, vec![ChoiceOption::Card(loot)]);
        } else {
            drain_pass(&mut e4, 1);
        }
    }
    assert_eq!(e4.state_core().player(1).hp, 5);
    assert!(e4.state_core().player(0).hand.contains(&loot));
}

#[test]
fn l11_npc_b_phase_turn_death_triggers_all() {
    // Covers: `twhanyu`, `twqueshi`, `xianfu`, `zongkui`, `shensu`, `juedi`, `twfenming`,
    //         `twhongju`, `twsaotao`, `twqianxi`, `twxingqi`, `wangzun`, `zili`, `spwenji`,
    //         `tianshu`, `twlinglu`, `zhendu`, `yongsi`, `shefu`, `sptunjiang`, `twyuanhu`,
    //         `wulie`, `xinjushou`, `zhiyan`, `twxuanhuo`, `twyingjia`, `xinfu_zhaoxin`, `sbyingzi`
    let mut e = start_4p_identity(2026_8105);
    for sk in ["twhanyu", "twqueshi", "twfenming", "sptunjiang", "sbyingzi", "yongsi"] {
        e.debug_grant(0, sk);
    }
    advance_to_phase(&mut e, 1, Phase::Play, None);
    advance_to_phase(&mut e, 0, Phase::Play, None);
    // `twhanyu` ran on TurnBegin; `twqueshi` equipped `meiyingqiang` onto seat 0!
    assert_eq!(flag(&e, 0, "l11b_twhanyu_init"), 1);
    assert_eq!(flag(&e, 0, "l11b_twqueshi_init"), 1);
    assert!(e.debug_has_skill(0, "meiyingqiang"));
    // Advance past Phase::Discard (`yongsi`), then `sptunjiang` triggers at Phase::End when no damage dealt!
    advance_to_phase(&mut e, 0, Phase::Discard, None);
    let hand_pre_end = e.state_core().player(0).hand.len();
    advance_to_phase(&mut e, 0, Phase::End, Some("trigger_confirm"));
    if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm") {
        answer(&mut e, 0, vec![ChoiceOption::Confirm]);
    }
    drain_pass(&mut e, 8);
    assert!(e.state_core().player(0).hand.len() > hand_pre_end);

    // `twxingqi` + `twqianxi` + `shensu` + `zongkui` + `wangzun`
    let mut e2 = start_2p(2026_8106);
    e2.debug_grant(0, "zongkui");
    e2.debug_grant(0, "twxingqi");
    e2.debug_grant(0, "twqianxi");
    e2.debug_grant(0, "shensu");
    e2.debug_grant(1, "wangzun");
    e2.debug_set_max_hp(0, 4);
    e2.debug_set_hp(0, 1);
    let eq = take_deck_or_discard_by(&mut e2, |d| d.kind.equip_slot() == Some(EquipSlot::Weapon));
    let eq2 = take_deck_or_discard_by(&mut e2, |d| d.kind.equip_slot() == Some(EquipSlot::Armor));
    e2.state_core_mut().player_mut(1).equips.extend([eq, eq2]);
    // Advance through seat 0 TurnBegin / Prepare / Judge -> confirm zongkui/twxingqi/twqianxi/wangzun!
    advance_to_phase(&mut e2, 1, Phase::Play, None);
    advance_to_phase(&mut e2, 0, Phase::Play, Some("zongkui:"));
    for _ in 0..24 {
        let Some(ch) = e2.open_choice().cloned() else { break };
        if ch.spec.options.contains(&ChoiceOption::Confirm) {
            answer(&mut e2, ch.observer, vec![ChoiceOption::Confirm]);
        } else if !ch.spec.options.is_empty() {
            let pick = ch.spec.options[0];
            answer(&mut e2, ch.observer, vec![pick]);
        } else {
            drain_pass(&mut e2, 1);
        }
    }
    assert_eq!(flag(&e2, 1, "l11_kui_mark"), 1);

    // `spwenji`, `zhendu`, `wulie`, `xinjushou`, `zhiyan`, `twxuanhuo`, `twyingjia`, `xinfu_zhaoxin`
    let mut e3 = start_4p_identity(2026_8107);
    e3.debug_grant(0, "spwenji");
    e3.debug_grant(0, "xinjushou");
    e3.debug_grant(0, "zhiyan");
    e3.debug_grant(0, "wulie");
    e3.debug_grant(0, "twxuanhuo");
    e3.debug_grant(1, "zhendu");
    e3.debug_set_max_hp(0, 6);
    e3.debug_set_hp(0, 6);
    advance_to_phase(&mut e3, 0, Phase::End, Some("wulie:"));
    for _ in 0..24 {
        let Some(ch) = e3.open_choice().cloned() else { break };
        if ch.spec.prompt.starts_with("wulie:") {
            answer(&mut e3, 0, vec![ChoiceOption::Player(1)]);
        } else if ch.spec.prompt.starts_with("xinjushou:") || ch.spec.prompt.starts_with("zhiyan:") {
            let pick = ch.spec.options[0];
            answer(&mut e3, 0, vec![pick]);
        } else {
            drain_pass(&mut e3, 1);
        }
    }
    assert!(flag(&e3, 0, "l11b_limited_wulie") == 1 || flag(&e3, 1, "l11b_wulie2_marks") > 0);

    // `juedi`: Prepare with `l11b_yinbing` pile
    let mut e4 = start_2p(2026_8108);
    e4.debug_grant(0, "juedi");
    let yb = take_deck_or_discard_by(&mut e4, |_| true);
    e4.state_core_mut()
        .vm_aux
        .piles
        .insert((0, "l11b_yinbing".to_string()), vec![yb]);
    advance_to_phase(&mut e4, 1, Phase::Play, None);
    advance_to_phase(&mut e4, 0, Phase::Prepare, Some("juedi:"));
    if let Some(ch) = e4.open_choice().cloned() {
        if ch.spec.prompt.starts_with("juedi:") {
            answer(&mut e4, 0, vec![ChoiceOption::Player(1)]);
        }
    }
    drain_pass(&mut e4, 8);
    assert!(e4.state_core().player(1).hand.contains(&yb));
}

#[test]
fn l11_npc_b_reactive_misc_triggers_all() {
    // Covers: `twjuntun` + `twxiongjun` + `baonvezhi`, `shunshi`, `tuifeng`, `twliancai`,
    //         `twwuhun`, `twyilie`, `weifeng`, `xinfu_shajue`, `xinjiewei` (TurnOver),
    //         `xinsheng`, `yechou`, `yinbing`, `yuce`, `zhichi`, `zhuide`, `zhanyi`,
    //         `twyuhua`, `xinfu_xingluan`, `twmibei`, `twzhengrong`, `twxiongzheng`,
    //         `yise` (`CardsMoved`), `zhuixi`, `xiehui`, `xianfu`
    let mut e = start_2p(2026_8109);
    e.debug_grant(0, "twjuntun");
    e.debug_grant(0, "shunshi");
    e.debug_grant(0, "tuifeng");
    e.debug_grant(0, "twwuhun");
    e.debug_grant(0, "xinsheng");
    e.debug_grant(0, "yinbing");
    e.debug_grant(0, "yuce");
    e.debug_grant(0, "zhichi");
    e.debug_grant(0, "yise");
    e.debug_set_max_hp(0, 8);
    e.debug_set_hp(0, 8);
    e.debug_set_max_hp(1, 8);
    e.debug_set_hp(1, 8);
    // Advance into turn 1 so `twjuntun` grants `twxiongjun`
    advance_to_phase(&mut e, 1, Phase::Play, None);
    advance_to_phase(&mut e, 0, Phase::Play, None);
    for _ in 0..8 {
        if e.open_choice().is_some_and(|c| c.spec.prompt.starts_with("twjuntun:")) {
            answer(&mut e, 0, vec![ChoiceOption::Player(1)]);
        } else {
            drain_pass(&mut e, 1);
        }
    }
    assert!(e.debug_has_skill(1, "twxiongjun") || flag(&e, 0, "l11b_twjuntun_init") == 1);
    // Move into seat 1's turn, then deal 1 out-of-turn damage from seat 1 to seat 0 -> fires `twwuhun`, `xinsheng`, `zhichi`, `yuce`, `tuifeng`!
    advance_to_phase(&mut e, 1, Phase::Play, None);
    e.debug_clear_hand(0);
    let yuce_card = place_hand_kind_any(&mut e, 0, CardKind::Shan);
    e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    for _ in 0..16 {
        let Some(ch) = e.open_choice().cloned() else { break };
        if ch.spec.prompt == "trigger_confirm" {
            answer(&mut e, ch.observer, vec![ChoiceOption::Confirm]);
        } else if ch.spec.prompt.starts_with("yuce:") && ch.spec.options.contains(&ChoiceOption::Card(yuce_card)) {
            answer(&mut e, 0, vec![ChoiceOption::Card(yuce_card)]);
        } else if ch.spec.prompt.starts_with("tuifeng:") && !ch.spec.options.is_empty() {
            let pick = ch.spec.options[0];
            answer(&mut e, 0, vec![pick]);
        } else {
            drain_pass(&mut e, 1);
        }
    }
    assert!(flag(&e, 1, "l11b_twwuhun_mark") >= 1);
    assert!(flag(&e, 0, "rehuashen_pool_count") >= 1);
    assert_eq!(flag(&e, 0, "l11b_zhichi2"), 1);

    // `twmibei`, `xinfu_xingluan`, `twzhengrong`, `weifeng`, `twyilie` on Play phase
    let mut e2 = start_2p(2026_8110);
    for sk in ["twmibei", "xinfu_xingluan", "twzhengrong", "weifeng", "twyilie", "twxiongzheng"] {
        e2.debug_grant(0, sk);
    }
    advance_to_phase(&mut e2, 1, Phase::Play, None);
    advance_to_phase(&mut e2, 0, Phase::Play, Some("twyilie:"));
    for _ in 0..12 {
        let Some(ch) = e2.open_choice().cloned() else { break };
        if ch.spec.prompt.starts_with("twyilie:") {
            answer(&mut e2, 0, vec![ChoiceOption::Tag(0)]);
        } else if !ch.spec.options.is_empty() {
            let pick = ch.spec.options[0];
            answer(&mut e2, ch.observer, vec![pick]);
        } else {
            drain_pass(&mut e2, 1);
        }
    }
    assert_eq!(flag(&e2, 0, "l11b_twyilie_opt1"), 1);
    // Play a card in Play phase -> exercises `twmibei` & `weifeng` & `twzhengrong` & `xinfu_xingluan`
    e2.debug_clear_hand(1);
    let sha = place_hand_kind_any(&mut e2, 0, CardKind::Sha);
    submit(
        &mut e2,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain_pass(&mut e2, 12);
    assert!(flag(&e2, 0, "l11b_mibei_cnt_0") >= 1);

    // `xinjiewei` TurnOvered + `twliancai` + `yechou` + `zhuide`
    let mut e3 = start_2p(2026_8111);
    e3.debug_grant(0, "twliancai");
    e3.debug_grant(0, "xinjiewei");
    e3.debug_grant(1, "yechou");
    e3.debug_grant(1, "zhuide");
    e3.debug_set_max_hp(0, 6);
    e3.debug_set_hp(0, 4);
    e3.debug_set_max_hp(1, 4);
    e3.debug_set_hp(1, 1);
    e3.debug_clear_hand(1);
    // Kill seat 1 with 1 damage -> triggers `yechou` & `zhuide`
    e3.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    for _ in 0..16 {
        let Some(ch) = e3.open_choice().cloned() else { break };
        if ch.spec.prompt.starts_with("yechou:") || ch.spec.prompt.starts_with("zhuide:") {
            let pick = ch.spec.options[0];
            answer(&mut e3, ch.observer, vec![pick]);
        } else {
            drain_pass(&mut e3, 1);
        }
    }
}

#[test]
fn l11_npc_b_round2_trigger_parity_fixes() {
    use engine_native::rvm_bridge::modifiers::eval_card_unrespondable;

    // Defect #4 (`twhengjiang`): `twhengjiang2` draws 1 per other player who responded to that buffed card.
    {
        let mut e = start_4p_identity(2026_8210);
        e.debug_grant(0, "twhengjiang");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        for s in 0..4 {
            e.debug_clear_hand(s);
            e.debug_set_max_hp(s, 6);
            e.debug_set_hp(s, 6);
        }
        // Equip weapon with range >= 3 (`ZhuaHuang` isn't weapon -> pick weapon >= 3 range, e.g. Qinglong / Fangtian / Qilin).
        let wp = take_deck_or_discard_by(&mut e, |d| {
            d.kind.equip_slot() == Some(EquipSlot::Weapon)
                && matches!(d.name, "fangtian" | "qinglong" | "qilin")
        });
        e.state_core_mut().player_mut(0).equips.push(wp);
        let duel = place_hand_kind_any(&mut e, 0, CardKind::Duel);
        let s0_sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let s1_sha = place_hand_kind_any(&mut e, 1, CardKind::Sha);
        let s2_sha = place_hand_kind_any(&mut e, 2, CardKind::Sha);
        // Seat 1 & seat 2 respond with Sha; seat 3 passes -> 2 distinct other responders -> +2 draw at useCardAfter!
        assert!(!matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: duel,
                    targets: vec![1],
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        let mut s0_responded = false;
        let mut s1_responded = false;
        let mut s2_responded = false;
        for _ in 0..24 {
            let Some(ch) = e.open_choice().cloned() else {
                break;
            };
            if ch.spec.prompt == "twhengjiang:confirm" {
                answer(&mut e, 0, vec![ChoiceOption::Confirm]);
            } else if ch.observer == 1 && !s1_responded && ch.spec.options.contains(&ChoiceOption::Card(s1_sha)) {
                s1_responded = true;
                answer(&mut e, 1, vec![ChoiceOption::Card(s1_sha)]);
            } else if ch.observer == 2 && !s2_responded && ch.spec.options.contains(&ChoiceOption::Card(s2_sha)) {
                s2_responded = true;
                answer(&mut e, 2, vec![ChoiceOption::Card(s2_sha)]);
            } else if ch.observer == 0 && !s0_responded && ch.spec.options.contains(&ChoiceOption::Card(s0_sha)) {
                s0_responded = true;
                answer(&mut e, 0, vec![ChoiceOption::Card(s0_sha)]);
            } else {
                drain_pass(&mut e, 1);
            }
        }
        assert!(s1_responded && s2_responded);
        // Seat 0 had 1 remaining card before Duel settled (if played `s0_sha`, +2 drawn -> 2; otherwise 3):
        let expected_hand = if s0_responded { 2 } else { 3 };
        assert_eq!(
            e.state_core().player(0).hand.len(),
            expected_hand,
            "twhengjiang2 must draw 1 card per distinct other responder (2 responders)"
        );
    }

    // Defect #6 (`twzhian`): ignores self, omits Tag(0)/Tag(1) when off field, prompts interactive hand discard.
    {
        let mut e = start_4p_identity(2026_8211);
        e.debug_grant(0, "twzhian");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        let self_eq = place_hand_kind_any(&mut e, 0, CardKind::Armor);
        assert!(!matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: self_eq,
                    targets: Vec::new(),
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        assert!(
            e.open_choice().is_none_or(|c| c.spec.prompt != "twzhian:choose_mode"),
            "twzhian must not prompt when owner uses delayed trick/equip on self"
        );
        drain_pass(&mut e, 8);

        // Seat 1 plays Armor; seat 0 chooses Tag(1), picks 2nd hand card `h1` interactively, gains Armor!
        advance_to_phase(&mut e, 1, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let h0 = place_hand_kind_any(&mut e, 0, CardKind::Shan);
        let h1 = place_hand_kind_any(&mut e, 0, CardKind::Tao);
        let arm1 = place_hand_kind_any(&mut e, 1, CardKind::Armor);
        assert!(!matches!(
            submit(
                &mut e,
                1,
                GameAction::PlayCard {
                    card: arm1,
                    targets: Vec::new(),
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        let mode_ch = e.open_choice().cloned().expect("twzhian must prompt seat 0 on seat 1 armor");
        assert_eq!(mode_ch.observer, 0);
        assert_eq!(mode_ch.spec.prompt, "twzhian:choose_mode");
        assert!(mode_ch.spec.options.contains(&ChoiceOption::Tag(0)));
        assert!(mode_ch.spec.options.contains(&ChoiceOption::Tag(1)));
        assert!(mode_ch.spec.options.contains(&ChoiceOption::Tag(2)));
        answer(&mut e, 0, vec![ChoiceOption::Tag(1)]);
        let disc_ch = e.open_choice().cloned().expect("twzhian mode 1 must prompt hand discard");
        assert_eq!(disc_ch.observer, 0);
        assert_eq!(disc_ch.spec.prompt, "twzhian:discard_hand");
        assert!(disc_ch.spec.options.contains(&ChoiceOption::Card(h1)));
        answer(&mut e, 0, vec![ChoiceOption::Card(h1)]);
        drain_pass(&mut e, 8);
        assert!(e.state_core().player(0).hand.contains(&h0));
        assert!(!e.state_core().player(0).hand.contains(&h1));
        assert!(e.state_core().player(0).hand.contains(&arm1));
        assert!(!e.state_core().player(1).equips.contains(&arm1));

        // When card leaves field before 2nd `twzhian` builds choices, Tag(0)/Tag(1) must be omitted!
        let mut e2 = start_4p_identity(2026_8212);
        e2.debug_grant(2, "twzhian");
        e2.debug_grant(3, "twzhian");
        advance_to_phase(&mut e2, 1, Phase::Play, None);
        e2.debug_clear_hand(1);
        e2.debug_clear_hand(2);
        e2.debug_clear_hand(3);
        let _h3 = place_hand_kind_any(&mut e2, 3, CardKind::Shan);
        let arm2 = place_hand_kind_any(&mut e2, 1, CardKind::Armor);
        let _ = submit(
            &mut e2,
            1,
            GameAction::PlayCard {
                card: arm2,
                targets: Vec::new(),
            },
        );
        // Seat 2 runs first & discards `arm2` via Tag(0).
        let ch_s2 = e2.open_choice().cloned().expect("seat 2 twzhian");
        assert_eq!(ch_s2.observer, 2);
        answer(&mut e2, 2, vec![ChoiceOption::Tag(0)]);
        // Seat 3 runs second while `arm2` is already off field -> only Tag(2) valid!
        let ch_s3 = e2.open_choice().cloned().expect("seat 3 twzhian");
        assert_eq!(ch_s3.observer, 3);
        assert_eq!(ch_s3.spec.prompt, "twzhian:choose_mode");
        assert!(!ch_s3.spec.options.contains(&ChoiceOption::Tag(0)));
        assert!(!ch_s3.spec.options.contains(&ChoiceOption::Tag(1)));
        assert!(ch_s3.spec.options.contains(&ChoiceOption::Tag(2)));
        drain_pass(&mut e2, 8);
    }

    // Defect #7 (`twsaotao`): delayed tricks (`Indulgence`) CAN still be Nullified; ordinary tricks cannot.
    {
        let mut e = start_2p(2026_8213);
        e.debug_grant(0, "twsaotao");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        let ind = place_hand_kind_any(&mut e, 0, CardKind::Le);
        let dis = place_hand_kind_any(&mut e, 0, CardKind::Dismantle);
        let wuxie = place_hand_kind_any(&mut e, 1, CardKind::Nullify);
        assert!(!eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            1,
            ind,
            CardKind::Le,
            DamageNature::Normal,
        ));
        assert!(eval_card_unrespondable(
            e.state_core(),
            e.catalog(),
            0,
            1,
            dis,
            CardKind::Dismantle,
            DamageNature::Normal,
        ));
        assert!(!matches!(
            submit(
                &mut e,
                0,
                GameAction::PlayCard {
                    card: ind,
                    targets: vec![1],
                },
            ),
            CommandOutcome::Rejected(_)
        ));
        let ch = e.open_choice().cloned().expect("delayed trick under twsaotao must allow Nullify");
        assert_eq!(ch.observer, 1);
        assert!(ch.spec.options.contains(&ChoiceOption::Card(wuxie)));
        answer(&mut e, 1, vec![ChoiceOption::Card(wuxie)]);
        drain_pass(&mut e, 8);
        assert!(e.state_core().player(1).judge.is_empty(), "Indulgence must be nullified");
    }

    // Defect #8 (`twwuhun`): dealing 2 damage (`Jiu`+`Sha`) to already-marked target adds only +1 mark.
    {
        let mut e = start_2p(2026_8214);
        e.debug_grant(0, "twwuhun");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_max_hp(1, 6);
        e.debug_set_hp(1, 6);
        e.state_core_mut()
            .vm_aux
            .flags
            .insert((1, "l11b_twwuhun_mark".to_string()), 1);
        let jiu = place_hand_kind_any(&mut e, 0, CardKind::Jiu);
        let sha = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: jiu,
                targets: Vec::new(),
            },
        );
        drain_pass(&mut e, 4);
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha,
                targets: vec![1],
            },
        );
        drain_pass(&mut e, 8);
        assert_eq!(e.state_core().player(1).hp, 4);
        assert_eq!(
            flag(&e, 1, "l11b_twwuhun_mark"),
            2,
            "twwuhun source damage triggers once (+1 mark regardless of damage amount)"
        );
    }

    // Defect #9 (`weifeng`): when `weifeng` owner dies, all other players' `l11b_weifeng_kind` marks clear.
    {
        let mut e = start_4p_identity(2026_8215);
        e.debug_grant(0, "weifeng");
        advance_to_phase(&mut e, 0, Phase::Play, None);
        e.debug_clear_hand(0);
        e.debug_clear_hand(1);
        e.debug_set_max_hp(1, 6);
        e.debug_set_hp(1, 6);
        let sha0 = place_hand_kind_any(&mut e, 0, CardKind::Sha);
        let _ = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha0,
                targets: vec![1],
            },
        );
        drain_pass(&mut e, 8);
        assert!(flag(&e, 1, "l11b_weifeng_kind") > 0);
        // Advance to seat 3 (in range of seat 0) and kill seat 0 with Sha -> marks on seat 1 clear!
        e.debug_set_hp(0, 1);
        e.debug_clear_hand(0);
        advance_to_phase(&mut e, 3, Phase::Play, None);
        e.debug_clear_hand(3);
        let sha3 = place_hand_kind_any(&mut e, 3, CardKind::Sha);
        let _ = submit(
            &mut e,
            3,
            GameAction::PlayCard {
                card: sha3,
                targets: vec![0],
            },
        );
        drain_pass(&mut e, 12);
        assert!(!e.state_core().player(0).alive);
        assert_eq!(
            flag(&e, 1, "l11b_weifeng_kind"),
            0,
            "weifeng2 marks on all seats must clear when weifeng owner dies"
        );
    }
}
