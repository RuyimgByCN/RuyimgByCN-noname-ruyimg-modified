//! U05-5a Round 2 审查问题（#48）真事件流回归测试（零 `debug_execute_effect`）。

use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, Phase};

fn place_cards_on_deck_top(
    core: &mut engine_native::state::GameStateCore,
    top_last: &[engine_native::ids::CardId],
) {
    for &c in top_last {
        core.deck.retain(|x| *x != c);
        core.discard.retain(|x| *x != c);
        for p in &mut core.players {
            p.hand.retain(|x| *x != c);
            p.equips.retain(|x| *x != c);
            p.judge.retain(|x| *x != c);
        }
    }
    core.deck.extend(top_last.iter().copied());
}

#[test]
fn u05_b5a_r2_hudiewu_disables_dieyi_equip2_and_fallback_armors_and_qinggang() {
    let (v, e0) = setup_gwent_locked(5571);
    let hudiewu = find_card(&e0, "gw_hudiewu");
    let baguas = find_cards_matching(&e0, |d| d.kind == CardKind::Armor, 2);
    assert_eq!(baguas.len(), 2, "需 2 张八卦阵验证首张替换与次张假标兜底");
    let (bagua1, bagua2) = (baguas[0], baguas[1]);
    let renwang = card_of_kind(&e0, CardKind::Renwang, None);
    let zhuge = card_of_kind(&e0, CardKind::Zhuge, None);
    let heart_judge = card_of_kind(&e0, CardKind::Tao, Some(Suit::Heart));
    let black_shas = find_cards_matching(
        &e0,
        |d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club),
        3,
    );
    assert_eq!(black_shas.len(), 3);

    // 1. 蝴蝶舞同时命中 1/2/3 号防具：1 号换为 gw_dieyi_equip2，2/3 号走 gw_dieyi_card:{c} 假标
    let mut e = inject_hand(&e0, &v, &[hudiewu], 0);
    e = inject_equip(&e, &v, bagua1, 1);
    e = inject_equip(&e, &v, bagua2, 2);
    e = inject_equip(&e, &v, renwang, 3);
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: hudiewu,
                targets: vec![]
            }
        ),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(
        e.save().core.players[1]
            .equips
            .iter()
            .any(|c| e.catalog().cards[*c as usize].name == "gw_dieyi_equip2")
    );
    assert!(
        e.save()
            .core
            .vm_aux
            .flags
            .contains_key(&(2, format!("gw_dieyi_card:{bagua2}")))
    );
    assert!(
        e.save()
            .core
            .vm_aux
            .flags
            .contains_key(&(3, format!("gw_dieyi_card:{renwang}")))
    );

    // 推进到 0 号下一次出牌阶段，装诸葛连弩+减一马，连出 3 张黑杀检验全部已变/假标防具均不再触发判定或免黑杀
    advance_to_seat_play(&mut e, 0);
    let horse_minus = card_of_kind(&e0, CardKind::HorseMinus, None);
    e = inject_equip(&e, &v, zhuge, 0);
    e = inject_equip(&e, &v, horse_minus, 0);
    let mut snap = e.save();
    place_cards_on_deck_top(&mut snap.core, &[heart_judge]);
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    e = inject_hand(&e, &v, &black_shas, 0);

    for (seat_idx, &sha_card) in [1u8, 2, 3].iter().zip(black_shas.iter()) {
        let mut s_top = e.save();
        place_cards_on_deck_top(&mut s_top.core, &[heart_judge]);
        e = content_native_loader::restore_locked_engine(&s_top, &v).unwrap();
        let logs_before = e.log().len();
        let out = submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: sha_card,
                targets: vec![*seat_idx],
            },
        );
        assert!(
            !matches!(out, CommandOutcome::Rejected(_)),
            "0 号出杀 seat {seat_idx} 不应被拒: {out:?}"
        );
        pass_window(&mut e);
        assert!(
            !e.log()[logs_before..]
                .iter()
                .any(|l| matches!(l, LogEvent::ArmorJudged { player, .. } if *player == *seat_idx)),
            "gw_dieyi_equip2 与假标防具不得触发八卦判定（seat {seat_idx}）"
        );
        assert_eq!(
            e.debug_hp(*seat_idx),
            3,
            "蝴蝶舞转化后的防具不得触发八卦/仁王抵挡黑杀（seat {seat_idx}）"
        );
    }
    assert_zone_conservation(&e);

    // 2. 假标青釭剑（>=2 号目标被兜底假标）：不再无视仁王盾
    let weapon1 = card_of_kind(&e0, CardKind::Weapon, None);
    let qinggang = card_of_kind(&e0, CardKind::Qinggang, None);
    let mut e_qg = inject_hand(&e0, &v, &[hudiewu], 0);
    e_qg = inject_equip(&e_qg, &v, weapon1, 1);
    e_qg = inject_equip(&e_qg, &v, qinggang, 2);
    submit(
        &mut e_qg,
        0,
        GameAction::PlayCard {
            card: hudiewu,
            targets: vec![],
        },
    );
    pass_window(&mut e_qg);
    assert!(
        e_qg.save()
            .core
            .vm_aux
            .flags
            .contains_key(&(2, format!("gw_dieyi_card:{qinggang}")))
    );
    advance_to_seat_play(&mut e_qg, 2);
    e_qg = inject_equip(&e_qg, &v, renwang, 3);
    e_qg = inject_hand(&e_qg, &v, &[black_shas[0]], 2);
    submit(
        &mut e_qg,
        2,
        GameAction::PlayCard {
            card: black_shas[0],
            targets: vec![3],
        },
    );
    pass_window(&mut e_qg);
    assert_eq!(
        e_qg.debug_hp(3),
        4,
        "假标 gw_dieyi_card:{{qinggang}} 不得无视 3 号未变形的仁王盾"
    );
    assert_zone_conservation(&e_qg);
}

#[test]
fn u05_b5a_r2_airuiting_and_huoge_enforce_hidden_kongcheng_weimu_and_cancel_mods() {
    let (v, e0) = setup_gwent_locked(5572);
    let airuiting = find_card(&e0, "gwmaoxian_airuiting");
    let black_sha = find_cards_matching(
        &e0,
        |d| d.kind == CardKind::Sha && matches!(d.suit, Suit::Spade | Suit::Club),
        1,
    )[0];
    let shans = find_cards_matching(&e0, |d| d.kind == CardKind::Shan, 6);

    // 1a. 艾瑞汀：2 号空城无手牌 + 3 号隐秘，仅 0 号可成为 1 号出杀目标
    let mut e_a1 = inject_hand(&e0, &v, &[airuiting], 0);
    e_a1 = inject_hand(&e_a1, &v, &[black_sha], 1);
    e_a1 = grant_skill(&e_a1, &v, "空城", 2);
    e_a1 = inject_hand(&e_a1, &v, &[shans[0]], 3);
    let mut s_a1 = e_a1.save();
    s_a1.core.players[3].yinmi_active = true;
    let mut e_a1 = content_native_loader::restore_locked_engine(&s_a1, &v).unwrap();
    submit(
        &mut e_a1,
        0,
        GameAction::PlayCard {
            card: airuiting,
            targets: vec![],
        },
    );
    pass_window(&mut e_a1);
    assert_eq!(e_a1.open_choice().unwrap().spec.prompt, "gwmaoxian_airuiting_sha");
    let (e_a1, _) = save_restore_answer(e_a1, &v, 1, vec![ChoiceOption::Card(black_sha)]);
    let tgt_ch = e_a1.open_choice().expect("airuiting_target choice").clone();
    assert_eq!(tgt_ch.spec.prompt, "gwmaoxian_airuiting_target");
    assert_eq!(
        tgt_ch.spec.options,
        vec![ChoiceOption::Player(0)],
        "艾瑞汀出杀目标必须排除空城(2)与隐秘(3)"
    );
    let (mut e_a1, _) = save_restore_answer(e_a1, &v, 1, vec![]);
    drain_default(&mut e_a1);
    assert_eq!(e_a1.debug_hp(1), 3);

    // 1b. 艾瑞汀 + `eval_card_target_canceled`（0 号对黑杀 molicha 免疫 + 2 空城 + 3 隐秘）→ 1 号无合法目标直接失 1 体力
    let mut e_a2 = inject_hand(&e0, &v, &[airuiting], 0);
    e_a2 = inject_hand(&e_a2, &v, &[black_sha], 1);
    e_a2 = grant_skill(&e_a2, &v, "空城", 2);
    let mut s_a2 = e_a2.save();
    s_a2.core
        .vm_aux
        .flags
        .insert((0, "u05_mod:molicha".to_string()), 1);
    s_a2.core.players[3].yinmi_active = true;
    let mut e_a2 = content_native_loader::restore_locked_engine(&s_a2, &v).unwrap();
    submit(
        &mut e_a2,
        0,
        GameAction::PlayCard {
            card: airuiting,
            targets: vec![],
        },
    );
    pass_window(&mut e_a2);
    assert!(
        e_a2.open_choice().is_none_or(|c| c.observer != 1),
        "全部射程内目标均被空城/隐秘/cancel 阻挡时不得询问 1 号出杀"
    );
    drain_default(&mut e_a2);
    assert_eq!(e_a2.debug_hp(1), 3);

    // 2. 霍格：黑决斗受帷幕排除、黑顺手/乐不思蜀受帷幕+谦逊+隐秘过滤、诱饵排除隐秘
    let huoge = find_card(&e0, "gwmaoxian_huoge");
    let black_duel = card_of_kind(&e0, CardKind::Duel, Some(Suit::Spade));
    let black_snatch = card_of_kind(&e0, CardKind::Snatch, Some(Suit::Spade));
    let black_le = find_cards_matching(
        &e0,
        |d| d.kind == CardKind::Le && matches!(d.suit, Suit::Spade | Suit::Club),
        1,
    )[0];
    let gw_youer = find_card(&e0, "gw_youer");
    let horse_minus = card_of_kind(&e0, CardKind::HorseMinus, None);

    let mut e_h = inject_hand(&e0, &v, &[huoge], 0);
    e_h = inject_equip(&e_h, &v, horse_minus, 0);
    e_h = grant_skill(&e_h, &v, "帷幕", 1);
    e_h = inject_hand(&e_h, &v, &[shans[0]], 1);
    e_h = inject_hand(&e_h, &v, &[shans[1]], 2);
    e_h = inject_hand(&e_h, &v, &[shans[2]], 3);
    let mut s_h = e_h.save();
    s_h.core
        .vm_aux
        .flags
        .insert((2, "u05_cross_mod:qianxun".to_string()), 1);
    s_h.core.players[3].yinmi_active = true;
    place_cards_on_deck_top(
        &mut s_h.core,
        &[shans[3], gw_youer, black_le, black_snatch, black_duel, black_sha],
    );
    let mut e_h = content_native_loader::restore_locked_engine(&s_h, &v).unwrap();
    submit(
        &mut e_h,
        0,
        GameAction::PlayCard {
            card: huoge,
            targets: vec![],
        },
    );
    pass_window(&mut e_h);
    let h_pick = e_h.open_choice().expect("huoge_pick").clone();
    assert_eq!(h_pick.spec.prompt, "gwmaoxian_huoge_pick");
    assert!(
        !h_pick.spec.options.contains(&ChoiceOption::Card(black_snatch)),
        "顺手被帷幕(1)+qianxun(2)+隐秘(3)全阻挡，不得出现在霍格可选牌中"
    );
    assert!(
        !h_pick.spec.options.contains(&ChoiceOption::Card(black_le)),
        "黑乐不思蜀被帷幕(1)+qianxun(2)+隐秘(3)全阻挡，不得出现在霍格可选牌中"
    );
    assert!(h_pick.spec.options.contains(&ChoiceOption::Card(black_duel)));
    assert!(h_pick.spec.options.contains(&ChoiceOption::Card(gw_youer)));
    submit(
        &mut e_h,
        0,
        GameAction::Answer {
            choice_id: h_pick.id,
            selection: vec![ChoiceOption::Card(black_duel), ChoiceOption::Card(gw_youer)],
        },
    );
    let duel_tgt = e_h.open_choice().expect("huoge duel target").clone();
    assert_eq!(duel_tgt.spec.prompt, "gwmaoxian_huoge_target");
    assert_eq!(
        duel_tgt.spec.options,
        vec![ChoiceOption::Player(2)],
        "黑决斗必须排除帷幕(1)与隐秘(3)"
    );
    submit(
        &mut e_h,
        0,
        GameAction::Answer {
            choice_id: duel_tgt.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    pass_window(&mut e_h);
    drain_default(&mut e_h);
    // 注：霍格使用 gw_youer 时，target_options 不得包含隐秘(3)；额外用单挑用例直断 gw_youer
    let mut e_y = inject_hand(&e0, &v, &[huoge], 0);
    e_y = inject_hand(&e_y, &v, &[shans[0]], 1);
    e_y = inject_hand(&e_y, &v, &[shans[1]], 3);
    let mut s_y = e_y.save();
    s_y.core.players[3].yinmi_active = true;
    place_cards_on_deck_top(
        &mut s_y.core,
        &[shans[2], shans[3], shans[4], shans[5], black_snatch, gw_youer],
    );
    let mut e_y = content_native_loader::restore_locked_engine(&s_y, &v).unwrap();
    submit(
        &mut e_y,
        0,
        GameAction::PlayCard {
            card: huoge,
            targets: vec![],
        },
    );
    pass_window(&mut e_y);
    let y_pick = e_y.open_choice().unwrap().id;
    submit(
        &mut e_y,
        0,
        GameAction::Answer {
            choice_id: y_pick,
            selection: vec![ChoiceOption::Card(gw_youer)],
        },
    );
    let youer_tgt = e_y.open_choice().expect("huoge gw_youer target").clone();
    assert!(
        !youer_tgt.spec.options.contains(&ChoiceOption::Player(3)),
        "others_with_cards 必须排除隐秘状态角色(3): {:?}",
        youer_tgt.spec.options
    );
    drain_default(&mut e_y);
    assert_zone_conservation(&e_y);

    // 2b. 霍格 + `gw_wuyao`：当射程内 1 号空城无手牌且 3 号隐秘（2 号距离 2 超射程），gw_wuyao 不可被选中
    let gw_wuyao = find_card(&e0, "gw_wuyao");
    let mut e_wy = inject_hand(&e0, &v, &[huoge], 0);
    e_wy = grant_skill(&e_wy, &v, "空城", 1);
    let mut s_wy = e_wy.save();
    s_wy.core.players[3].yinmi_active = true;
    place_cards_on_deck_top(
        &mut s_wy.core,
        &[shans[0], shans[1], shans[2], shans[3], shans[4], gw_wuyao],
    );
    let mut e_wy = content_native_loader::restore_locked_engine(&s_wy, &v).unwrap();
    submit(
        &mut e_wy,
        0,
        GameAction::PlayCard {
            card: huoge,
            targets: vec![],
        },
    );
    pass_window(&mut e_wy);
    assert!(
        e_wy.open_choice().is_none_or(|c| !c.spec.options.contains(&ChoiceOption::Card(gw_wuyao))),
        "无合法可杀目标时霍格不得允许选择 gw_wuyao"
    );
    drain_default(&mut e_wy);
    assert_zone_conservation(&e_wy);
}

#[test]
fn u05_b5a_r2_falanxisika_emits_card_used_and_ends_play_once() {
    let (v, e0) = setup_gwent_locked(5573);
    let fala = find_card(&e0, "gwmaoxian_falanxisika");
    let mut e = inject_hand(&e0, &v, &[fala], 0);
    assert!(matches!(
        submit(
            &mut e,
            0,
            GameAction::PlayCard {
                card: fala,
                targets: vec![]
            }
        ),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let pick_ch = e.open_choice().expect("falanxisika_pick").clone();
    assert_eq!(pick_ch.spec.prompt, "gwmaoxian_falanxisika_pick");
    let chosen_tag = pick_ch.spec.options[0];
    // 3a. 跨存档恢复：恢复后仅日志新增被施放的金卡法术 CardPlayed，total_used == 2
    let (mut e, _) = save_restore_answer(e, &v, 0, vec![chosen_tag]);
    drain_default(&mut e);

    let used_after_restore: Vec<&str> = e
        .log()
        .iter()
        .filter_map(|l| match l {
            LogEvent::CardPlayed { player: 0, card, .. } => Some(e.catalog().cards[*card as usize].name),
            _ => None,
        })
        .collect();
    assert_eq!(
        used_after_restore.len(),
        1,
        "恢复后应恰好触发 1 次所选金卡法术 CardUsed，实际={used_after_restore:?}"
    );
    assert!(
        used_after_restore[0].starts_with("gw_") && used_after_restore[0] != "gwmaoxian_falanxisika",
        "所选金卡法术须发出真实 CardUsed，实际={used_after_restore:?}"
    );
    assert_eq!(e.state_core().player(0).total_used, 2);
    assert!(is_vanished(&e, fala));
    assert!(e.save().core.table.is_empty());
    assert_eq!(
        e.turn().current,
        0,
        "不得重复触发 Phase::Play 结束而提前跳过弃牌阶段直接切回合"
    );
    assert_eq!(e.turn().phase, Phase::Discard);
    assert_zone_conservation(&e);

    // 3b. 无中断完整跑：按序发出 [gwmaoxian_falanxisika, <gold_spell>] 两次 CardUsed
    let mut e_full = inject_hand(&e0, &v, &[fala], 0);
    submit(
        &mut e_full,
        0,
        GameAction::PlayCard {
            card: fala,
            targets: vec![],
        },
    );
    pass_window(&mut e_full);
    let pick2 = e_full.open_choice().expect("falanxisika_pick full").clone();
    submit(
        &mut e_full,
        0,
        GameAction::Answer {
            choice_id: pick2.id,
            selection: vec![pick2.spec.options[0]],
        },
    );
    drain_default(&mut e_full);
    let used_full: Vec<&str> = e_full
        .log()
        .iter()
        .filter_map(|l| match l {
            LogEvent::CardPlayed { player: 0, card, .. } => {
                Some(e_full.catalog().cards[*card as usize].name)
            }
            _ => None,
        })
        .collect();
    assert_eq!(used_full.len(), 2, "完整战局须触发 2 次 CardUsed: {used_full:?}");
    assert_eq!(used_full[0], "gwmaoxian_falanxisika");
    assert!(used_full[1].starts_with("gw_") && used_full[1] != "gwmaoxian_falanxisika");
    assert_eq!(e_full.turn().phase, Phase::Discard);
    assert_zone_conservation(&e_full);
}
