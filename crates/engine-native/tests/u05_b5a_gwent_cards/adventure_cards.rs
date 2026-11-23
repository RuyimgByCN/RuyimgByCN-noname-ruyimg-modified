//! `character/gwent/card.js` 全 26 张牌（`gw_xianzumaijiu`、20 张 `gwmaoxian_*`、
//! `gwjinli_jiu`、`gw_wuyao`、`gw_lang`、`gw_dudayuanshuai1/2`）真实事件测试。

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, Phase, RejectReason};

#[test]
fn u05_b5a_adv_special_cards_wuyao_lang_xianzumaijiu_tokens() {
    let (v, e0) = setup_gwent_locked(5530);

    // 1. 禁止直接出牌的派生/不可出牌：`gwjinli_jiu`, `gw_dudayuanshuai1`, `gw_dudayuanshuai2`
    for tok in ["gwjinli_jiu", "gw_dudayuanshuai1", "gw_dudayuanshuai2"] {
        let cid = find_card(&e0, tok);
        let mut e = inject_hand(&e0, &v, &[cid], 0);
        assert!(
            matches!(
                submit(&mut e, 0, GameAction::PlayCard { card: cid, targets: vec![] }),
                CommandOutcome::Rejected(RejectReason::WrongPhase)
            ),
            "{tok} 不应在出牌阶段被直接打出"
        );
    }

    // 2. 巫妖药水 (`character/gwent/card.js:925`): 当【杀】打出与响应，且 vanish=true
    let wuyao = find_card(&e0, "gw_wuyao");
    let mut e = inject_hand(&e0, &v, &[wuyao], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: wuyao, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hp(1), 3, "gw_wuyao 当【杀】造成 1 伤");
    assert!(is_vanished(&e, wuyao));

    // 决斗响应时 gw_wuyao 亦作为【杀】响应
    let duel = card_of_kind(&e0, CardKind::Duel, None);
    let mut e = inject_hand(&e0, &v, &[duel], 0);
    e = inject_hand(&e, &v, &[wuyao], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: duel, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let c = e.open_choice().expect("1 号决斗应询问出杀");
    assert!(c.spec.options.contains(&ChoiceOption::Card(wuyao)), "gw_wuyao 可响应决斗打杀");
    let cid = c.id;
    submit(&mut e, 1, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(wuyao)] });
    drain_default(&mut e);
    assert!(is_vanished(&e, wuyao));

    // 3. 狼 (`character/gwent/card.js:938`): 当【酒】增伤 +1 与濒死自救
    let lang = find_card(&e0, "gw_lang");
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let mut e = inject_hand(&e0, &v, &[lang, sha], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: lang, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(is_vanished(&e, lang));
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hp(1), 2, "gw_lang 作为【酒】使下次杀伤害+1");

    // 4. 先祖麦酒 (`character/gwent/card.js:9`): 无杀伤加成；用杀命中后自己与友军(0,2)各摸 1；每阶段限 1 次 + 已持技禁重放
    let maijiu = find_card(&e0, "gw_xianzumaijiu");
    let aozu = find_card(&e0, "gw_aozuzhilei");
    let mut e = inject_hand(&e0, &v, &[maijiu, sha, aozu], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: maijiu, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(is_vanished(&e, maijiu));
    // 再次出先祖麦酒被限次/重复拒绝
    let mut e = inject_hand(&e, &v, &[maijiu], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: maijiu, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::CardLimit)
    ));
    let h0_b = e.debug_hand(0).len();
    let h2_b = e.debug_hand(2).len();
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hp(1), 3, "先祖麦酒不叠加酒伤害加成");
    assert_eq!(e.debug_hand(0).len(), h0_b, "打出1杀后触发先祖麦酒摸1 → 净+0");
    assert_eq!(e.debug_hand(2).len(), h2_b + 1, "友方忠臣 2 号一同摸 1");
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_adv_gwmaoxian_first_half() {
    let (v, e0) = setup_gwent_locked(5531);
    let sha = card_of_kind(&e0, CardKind::Sha, None);

    // 1. 伊欧菲斯 (`character/gwent/card.js:119`): 互杀 + vanish + 结束 Play
    let yiou = find_card(&e0, "gwmaoxian_yioufeisi");
    let mut e = inject_hand(&e0, &v, &[yiou], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: yiou, targets: vec![1, 3] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.debug_hp(1) < 4 || e.debug_hp(3) < 4);
    assert!(is_vanished(&e, yiou));
    assert_ne!(e.turn().phase, Phase::Play);

    // 2. 罗契 (`character/gwent/card.js:158`): 其余角色出杀询问 + 存档恢复 + cancel 分支
    let luoqi = find_card(&e0, "gwmaoxian_luoqi");
    let mut e = inject_hand(&e0, &v, &[luoqi], 0);
    e = inject_hand(&e, &v, &[sha], 2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: luoqi, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "gwmaoxian_luoqi_sha");
    assert_eq!(c.observer, 2);
    let (mut e_yes, out) = save_restore_answer(e.clone(), &v, 2, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e_yes);
    assert!(e_yes.debug_hp(1) <= 2, "罗契自己出杀+友军补杀共2伤");
    let (mut e_no, _) = save_restore_answer(e, &v, 2, vec![]);
    pass_window(&mut e_no);
    assert_eq!(e_no.debug_hp(1), 3, "cancel 跳过补杀仅 1 伤");

    // 3. 杰洛特 / 叶奈法 / 特莉丝 / 亨赛特 / 弗尔泰斯特 / 拉多维德 / 恩希尔
    let jieluote = find_card(&e0, "gwmaoxian_jieluote");
    let mut e = inject_hand(&e0, &v, &[jieluote], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: jieluote, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hp(1), 2, "杰洛特命中最高体力>2造成2伤");

    for (name, tgt) in [
        ("gwmaoxian_yenaifa", vec![]),
        ("gwmaoxian_telisi", vec![]),
        ("gwmaoxian_hengsaite", vec![]),
        ("gwmaoxian_fuertaisite", vec![0, 2]),
        ("gwmaoxian_laduoweide", vec![1]),
    ] {
        let cid = find_card(&e0, name);
        let mut e = inject_hand(&e0, &v, &[cid], 0);
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: cid, targets: tgt }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        pass_window(&mut e);
        assert!(is_vanished(&e, cid), "{name} vanish");
        assert_ne!(e.turn().phase, Phase::Play, "{name} 结束 Play");
        assert_zone_conservation(&e);
    }

    // 恩希尔 (`character/gwent/card.js:438`): 手牌数差>1 拒绝，差<=1 交换
    let enxier = find_card(&e0, "gwmaoxian_enxier");
    let extra = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 3);
    let mut e_bad = inject_hand(&e0, &v, &[enxier, extra[0], extra[1]], 0);
    assert!(matches!(
        submit(&mut e_bad, 0, GameAction::PlayCard { card: enxier, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    let mut e = inject_hand(&e0, &v, &[enxier, extra[0]], 0);
    e = inject_hand(&e, &v, &[extra[1]], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: enxier, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.debug_hand(0).contains(&extra[1]) && e.debug_hand(1).contains(&extra[0]));
}

#[test]
fn u05_b5a_adv_gwmaoxian_second_half() {
    let (v, e0) = setup_gwent_locked(5532);
    let shas = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 4);

    // 1. 符里斯 + 卡尔维特 + 布兰王(A vs Cancel) + 奎特 + 哈洛 + 达贡
    let fulisi = find_card(&e0, "gwmaoxian_fulisi");
    let mut e = inject_hand(&e0, &v, &[fulisi], 0);
    e = inject_hand(&e, &v, &[shas[0]], 1);
    e = inject_hand(&e, &v, &[shas[1]], 3);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: fulisi, targets: vec![1, 3] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gwmaoxian_fulisi_discard");
    let (mut e, _) = save_restore_answer(
        e,
        &v,
        0,
        vec![ChoiceOption::Card(shas[0]), ChoiceOption::Card(shas[1])],
    );
    pass_window(&mut e);
    assert!(e.debug_hand(1).is_empty() && e.debug_hand(3).is_empty());

    let bulan = find_card(&e0, "gwmaoxian_bulanwang");
    let mut e = inject_hand(&e0, &v, &[bulan, shas[0], shas[1]], 0);
    e.debug_set_hp(0, 1); // 手牌上限=1，用以验证 skip_discard_phase 跳过弃牌保留全部手牌
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: bulan, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gwmaoxian_bulanwang_discard");
    let (mut e_a, _) = save_restore_answer(
        e.clone(),
        &v,
        0,
        vec![ChoiceOption::Card(shas[0]), ChoiceOption::Card(shas[1])],
    );
    pass_window(&mut e_a);
    assert_eq!(e_a.debug_hand(0).len(), 4, "布兰王弃 2 摸 4 且跳过弃牌阶段保留 4 张(>hp=1)");

    let (mut e_cancel, _) = save_restore_answer(e, &v, 0, vec![]);
    pass_window(&mut e_cancel);
    assert_eq!(e_cancel.debug_hand(0).len(), 2, "布兰王 cancel 保留 2 张(>hp=1)且跳过弃牌");

    for (name, tgt) in [
        ("gwmaoxian_kaerweite", vec![1]),
        ("gwmaoxian_kuite", vec![1]),
        ("gwmaoxian_haluo", vec![]),
        ("gwmaoxian_dagong", vec![1]),
    ] {
        let cid = find_card(&e0, name);
        let mut e = inject_hand(&e0, &v, &[cid], 0);
        if matches!(name, "gwmaoxian_kaerweite" | "gwmaoxian_kuite") {
            e = inject_hand(&e, &v, &[shas[0]], 1);
        }
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: cid, targets: tgt }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        drain_default(&mut e);
        assert!(is_vanished(&e, cid), "{name} vanish");
        assert_zone_conservation(&e);
    }

    // 2. 盖尔 (Branch 0 增益 vs Branch 1 减损) + 艾瑞汀 + 艾思妮 + 法兰西丝卡 + 霍格
    let gaier = find_card(&e0, "gwmaoxian_gaier");
    let mut e = inject_hand(&e0, &v, &[gaier], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: gaier, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gwmaoxian_gaier_branch");
    let (mut e0_a, _) = save_restore_answer(e.clone(), &v, 0, vec![ChoiceOption::Tag(0)]);
    pass_window(&mut e0_a);
    assert_eq!(e0_a.save().core.players[1].max_hp, 5);
    let (mut e0_b, _) = save_restore_answer(e, &v, 0, vec![ChoiceOption::Tag(1)]);
    pass_window(&mut e0_b);
    assert_eq!((e0_b.debug_hp(1), e0_b.save().core.players[1].max_hp), (3, 3));

    let airuiting = find_card(&e0, "gwmaoxian_airuiting");
    let mut e = inject_hand(&e0, &v, &[airuiting], 0);
    e = inject_hand(&e, &v, &[shas[0]], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: airuiting, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gwmaoxian_airuiting_sha");
    let (mut e, _) = save_restore_answer(e, &v, 1, vec![]);
    drain_default(&mut e);
    assert_eq!(e.debug_hp(1), 3, "拒绝出杀时艾瑞汀令其失去 1 体力");

    for name in ["gwmaoxian_aisinie", "gwmaoxian_falanxisika", "gwmaoxian_huoge"] {
        let cid = find_card(&e0, name);
        let mut e = inject_hand(&e0, &v, &[cid], 0);
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: cid, targets: vec![] }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        pass_window(&mut e);
        if let Some(c) = e.open_choice().cloned() {
            let pick = c.spec.options[0];
            let (mut e_r, _) = save_restore_answer(e, &v, c.observer, vec![pick]);
            drain_default(&mut e_r);
            assert!(is_vanished(&e_r, cid));
            assert_zone_conservation(&e_r);
        } else {
            drain_default(&mut e);
            assert!(is_vanished(&e, cid));
            assert_zone_conservation(&e);
        }
    }
}

#[test]
fn u05_b5a_round2_huoge_fixes_and_dudayuanshuai1() {
    let (v, e0) = setup_gwent_locked(5533);
    let huoge = find_card(&e0, "gwmaoxian_huoge");

    // #4(a)/(b)/(c): 霍格牌堆顶放入 Equip、Tao、Sha、gw_lang，依次验证真实装备、桌牌不重复、gw_lang 入 vanish 且 table 空
    let weapon = card_of_kind(&e0, CardKind::Weapon, None);
    let tao = card_of_kind(&e0, CardKind::Tao, None);
    let gw_lang = find_card(&e0, "gw_lang");
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let mut e = inject_hand(&e0, &v, &[huoge], 0);
    e.debug_set_hp(0, 2);
    let mut snap = e.save();
    for c in [weapon, tao, gw_lang, sha] {
        snap.core.deck.retain(|x| *x != c);
        snap.core.discard.retain(|x| *x != c);
        for p in &mut snap.core.players {
            p.hand.retain(|x| *x != c);
        }
    }
    // 栈顶 pop：先 pop 进 peeked，因此最后放入 vec 尾的会在顶 6
    let padding = find_cards_matching(&e0, |d| d.kind == CardKind::Shan, 2);
    for c in &padding {
        snap.core.deck.retain(|x| *x != *c);
    }
    snap.core.deck.extend([padding[0], padding[1], gw_lang, sha, tao, weapon]);
    let mut e = content_native_loader::restore_locked_engine(&snap, &v).unwrap();
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: huoge, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gwmaoxian_huoge_pick");
    let c_id = e.open_choice().unwrap().id;
    // 选 Equip(weapon) + gw_lang
    assert!(matches!(
        submit(&mut e, 0, GameAction::Answer {
            choice_id: c_id,
            selection: vec![ChoiceOption::Card(weapon), ChoiceOption::Card(gw_lang)],
        }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.save().core.players[0].equips.contains(&weapon), "霍格选择的装备必须真正进入 equips");
    assert!(e.save().core.table.is_empty(), "霍格使用 Equip/gw_lang 后 table 不得残留");
    assert!(is_vanished(&e, gw_lang), "霍格使用 gw_lang 须移入 u05_vanished 且不留 discard");
    assert_zone_conservation(&e);

    // 验证 Tao + Sha via Huoge
    let mut e_ts = inject_hand(&e0, &v, &[huoge], 0);
    e_ts.debug_set_hp(0, 2);
    let mut snap_ts = e_ts.save();
    for c in [tao, sha, padding[0], padding[1]] {
        snap_ts.core.deck.retain(|x| *x != c);
        snap_ts.core.discard.retain(|x| *x != c);
    }
    snap_ts.core.deck.extend([padding[0], padding[1], padding[0], padding[1], sha, tao]);
    // padding 不得重复入 deck（保持唯一性）：改取另一组牌补齐 6 张
    snap_ts.core.deck.truncate(snap_ts.core.deck.len() - 4);
    let more_shan = find_cards_matching(&e0, |d| d.kind == CardKind::Shan, 4);
    for c in &more_shan {
        snap_ts.core.deck.retain(|x| *x != *c);
        snap_ts.core.discard.retain(|x| *x != *c);
    }
    snap_ts.core.deck.extend([more_shan[0], more_shan[1], more_shan[2], more_shan[3], sha, tao]);
    let mut e_ts = content_native_loader::restore_locked_engine(&snap_ts, &v).unwrap();
    submit(&mut e_ts, 0, GameAction::PlayCard { card: huoge, targets: vec![] });
    pass_window(&mut e_ts);
    let pick_id = e_ts.open_choice().unwrap().id;
    submit(&mut e_ts, 0, GameAction::Answer {
        choice_id: pick_id,
        selection: vec![ChoiceOption::Card(tao), ChoiceOption::Card(sha)],
    });
    pass_window(&mut e_ts);
    let tgt_id = e_ts.open_choice().unwrap().id;
    assert_eq!(e_ts.open_choice().unwrap().spec.prompt, "gwmaoxian_huoge_target");
    submit(&mut e_ts, 0, GameAction::Answer { choice_id: tgt_id, selection: vec![ChoiceOption::Player(1)] });
    pass_window(&mut e_ts);
    assert_eq!(e_ts.debug_hp(0), 3, "霍格用桃回 1 血");
    assert_eq!(e_ts.debug_hp(1), 3, "霍格用杀伤 1 号 1 血");
    assert!(e_ts.save().core.table.is_empty(), "霍格用桃与杀后 table 为空不重复");
    assert_zone_conservation(&e_ts);

    // #4(d): Dismantle / Snatch / Le 目标过滤（仅 3 号有手牌、distance=2 不可顺、已有 Le 不再选）
    let chai = card_of_kind(&e0, CardKind::Dismantle, None);
    let shun = card_of_kind(&e0, CardKind::Snatch, None);
    let le = card_of_kind(&e0, CardKind::Le, None);
    let shas2 = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 6);
    let mut e_f = inject_hand(&e0, &v, &[huoge], 0);
    e_f = inject_hand(&e_f, &v, &[shas2[0]], 2); // 距离 0 号=2（不能顺手）
    e_f = inject_judge(&e_f, &v, le, 1); // 1 号已有乐
    let le2 = find_cards_matching(&e0, |d| d.kind == CardKind::Le, 2)[1];
    let mut snap_f = e_f.save();
    for c in [chai, shun, le2, shas2[1], shas2[2], shas2[3]] {
        snap_f.core.deck.retain(|x| *x != c);
        snap_f.core.discard.retain(|x| *x != c);
    }
    snap_f.core.deck.extend([shas2[1], shas2[2], shas2[3], shun, chai, le2]);
    let mut e_f = content_native_loader::restore_locked_engine(&snap_f, &v).unwrap();
    submit(&mut e_f, 0, GameAction::PlayCard { card: huoge, targets: vec![] });
    pass_window(&mut e_f);
    let opts_f = e_f.open_choice().unwrap().spec.options.clone();
    assert!(!opts_f.contains(&ChoiceOption::Card(shun)), "距离 2 有牌、距离 1 无牌时顺手不得入选");
    assert!(opts_f.contains(&ChoiceOption::Card(chai)), "2 号有牌时拆桥可选");
    let pid = e_f.open_choice().unwrap().id;
    submit(&mut e_f, 0, GameAction::Answer { choice_id: pid, selection: vec![ChoiceOption::Card(chai), ChoiceOption::Card(le2)] });
    let chai_tgts = e_f.open_choice().unwrap().spec.options.clone();
    assert!(!chai_tgts.contains(&ChoiceOption::Player(3)), "空区 3 号不得进拆桥目标");
    assert!(chai_tgts.contains(&ChoiceOption::Player(2)));
    let tid1 = e_f.open_choice().unwrap().id;
    submit(&mut e_f, 0, GameAction::Answer { choice_id: tid1, selection: vec![ChoiceOption::Player(2)] });
    pass_window(&mut e_f);
    let le_tgts = e_f.open_choice().unwrap().spec.options.clone();
    assert!(!le_tgts.contains(&ChoiceOption::Player(1)), "已有乐的 1 号不得进乐目标");

    // #4(e): 霍格第 1 张打金卡(干旱)后，第 2 张(杀)必须继续执行完毕才结束 Play 阶段
    let ganhan = find_card(&e0, "gw_ganhan");
    let e_g = inject_hand(&e0, &v, &[huoge], 0);
    let mut snap_g = e_g.save();
    for c in [ganhan, sha, more_shan[0], more_shan[1], more_shan[2], more_shan[3]] {
        snap_g.core.deck.retain(|x| *x != c);
        snap_g.core.discard.retain(|x| *x != c);
    }
    snap_g.core.deck.extend([more_shan[0], more_shan[1], more_shan[2], more_shan[3], sha, ganhan]);
    let mut e_g = content_native_loader::restore_locked_engine(&snap_g, &v).unwrap();
    submit(&mut e_g, 0, GameAction::PlayCard { card: huoge, targets: vec![] });
    pass_window(&mut e_g);
    let g_id = e_g.open_choice().unwrap().id;
    submit(&mut e_g, 0, GameAction::Answer {
        choice_id: g_id,
        selection: vec![ChoiceOption::Card(ganhan), ChoiceOption::Card(sha)],
    });
    assert_eq!(e_g.turn().phase, Phase::Play, "霍格第 1 张金卡不得提前结束 Play 阻断第 2 张杀");
    assert_eq!(e_g.open_choice().unwrap().spec.prompt, "gwmaoxian_huoge_target");
    let sha_t = e_g.open_choice().unwrap().id;
    submit(&mut e_g, 0, GameAction::Answer { choice_id: sha_t, selection: vec![ChoiceOption::Player(1)] });
    pass_window(&mut e_g);
    assert_eq!(e_g.debug_hp(1), 2, "1 号先因干旱上限-1(4->3)，再吃第 2 张杀降为 2");
    assert_ne!(e_g.turn().phase, Phase::Play, "霍格第 2 张用完后才结束 Play 阶段");
    assert_zone_conservation(&e_g);

    // #12: `gw_dudayuanshuai1` 真事件栈结算（带 `gw_dudayuanshuai1_evt` 取消来牌并获入手；无标不取消）
    let duda1 = find_card(&e0, "gw_dudayuanshuai1");
    let mut core_a = inject_hand(&e0, &v, &[sha], 0).save().core;
    core_a.players[1].hand.push(duda1);
    core_a.deck.retain(|x| *x != duda1);
    core_a.discard.retain(|x| *x != duda1);
    let mut res_a = engine_native::Resolution::new(128);
    let mut log_a = Vec::new();
    let inc_id = engine_native::resolution::push_frame(
        &mut res_a,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(sha, 0, vec![1])),
    );
    let _ = engine_native::resolution::run(&mut core_a, &mut res_a, e0.catalog(), &mut log_a, Some(2));
    core_a.vm_aux.flags.insert((1, format!("gw_dudayuanshuai1_evt:{inc_id}")), 1);
    engine_native::resolution::push_frame(
        &mut res_a,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(duda1, 1, vec![])),
    );
    let out_a = engine_native::resolution::run(&mut core_a, &mut res_a, e0.catalog(), &mut log_a, None);
    assert_eq!(out_a, engine_native::resolution::StepOutcome::Done);
    assert_eq!(core_a.players[1].hp, 4, "gw_dudayuanshuai1 取消来牌杀使 1 号不受伤");
    assert!(core_a.players[1].hand.contains(&sha), "gw_dudayuanshuai1 将来牌收进自己手牌");
    assert!(
        core_a.vm_aux.piles.get(&(0, "u05_vanished".to_string())).is_some_and(|p| p.contains(&duda1)),
        "gw_dudayuanshuai1 vanish 进入 u05_vanished"
    );
    assert!(core_a.table.is_empty());

    // 无 gw_dudayuanshuai1_evt 时不取消来牌
    let mut core_b = inject_hand(&e0, &v, &[sha], 0).save().core;
    core_b.players[1].hand.push(duda1);
    core_b.deck.retain(|x| *x != duda1);
    core_b.discard.retain(|x| *x != duda1);
    let mut res_b = engine_native::Resolution::new(128);
    let mut log_b = Vec::new();
    let _ = engine_native::resolution::push_frame(
        &mut res_b,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(sha, 0, vec![1])),
    );
    let _ = engine_native::resolution::run(&mut core_b, &mut res_b, e0.catalog(), &mut log_b, Some(2));
    engine_native::resolution::push_frame(
        &mut res_b,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(duda1, 1, vec![])),
    );
    let mut out_b = engine_native::resolution::run(&mut core_b, &mut res_b, e0.catalog(), &mut log_b, None);
    while out_b == engine_native::resolution::StepOutcome::Pending {
        res_b.open_choice = None;
        res_b.answer = Some(Vec::new());
        out_b = engine_native::resolution::run(&mut core_b, &mut res_b, e0.catalog(), &mut log_b, None);
    }
    assert_eq!(out_b, engine_native::resolution::StepOutcome::Done);
    assert_eq!(core_b.players[1].hp, 3, "无 gw_dudayuanshuai1_evt 时杀不被取消并命中 1 号");

    let mut snap_a = e0.save();
    snap_a.core = core_a;
    let e_a_cons = content_native_loader::restore_locked_engine(&snap_a, &v).unwrap();
    assert_zone_conservation(&e_a_cons);

    let mut snap_b = e0.save();
    snap_b.core = core_b;
    let e_b_cons = content_native_loader::restore_locked_engine(&snap_b, &v).unwrap();
    assert_zone_conservation(&e_b_cons);

    // 死前已出牌入 table 但所有者在结算时已死亡：gw_dudayuanshuai1 不得从 table 拿走并丢牌
    let mut core_c = inject_hand(&e0, &v, &[sha], 0).save().core;
    core_c.players[1].hand.push(duda1);
    core_c.deck.retain(|x| *x != duda1);
    core_c.discard.retain(|x| *x != duda1);
    let mut res_c = engine_native::Resolution::new(128);
    let mut log_c = Vec::new();
    let _ = engine_native::resolution::push_frame(
        &mut res_c,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(sha, 0, vec![2])),
    );
    let _ = engine_native::resolution::run(&mut core_c, &mut res_c, e0.catalog(), &mut log_c, Some(2));
    engine_native::resolution::push_frame(
        &mut res_c,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(duda1, 1, vec![])),
    );
    let _ = engine_native::resolution::run(&mut core_c, &mut res_c, e0.catalog(), &mut log_c, Some(1));
    core_c.players[1].alive = false;
    core_c.players[1].hp = 0;
    let mut out_c = engine_native::resolution::run(&mut core_c, &mut res_c, e0.catalog(), &mut log_c, None);
    while out_c == engine_native::resolution::StepOutcome::Pending {
        res_c.open_choice = None;
        res_c.answer = Some(Vec::new());
        out_c = engine_native::resolution::run(&mut core_c, &mut res_c, e0.catalog(), &mut log_c, None);
    }
    assert_eq!(out_c, engine_native::resolution::StepOutcome::Done);
    assert!(core_c.players[1].hand.is_empty());
    let mut snap_c = e0.save();
    snap_c.core = core_c;
    let e_c_cons = content_native_loader::restore_locked_engine(&snap_c, &v).unwrap();
    assert_zone_conservation(&e_c_cons);

    // #280(1)+(2): 霍格 + AOE (`SavageAttack` + `ArcheryAttack`) 须冻结全体他家目标并实际结算伤害；
    // 霍格 + `Jiedao`（无人持武器时不得入选项、无 panic；有人持武器+射程内目标时两步选目标并完整结算）
    let nanman = card_of_kind(&e0, CardKind::SavageAttack, None);
    let wanjian = card_of_kind(&e0, CardKind::ArcheryAttack, None);
    let e_aoe = inject_hand(&e0, &v, &[huoge], 0);
    let mut snap_aoe = e_aoe.save();
    for c in [nanman, wanjian, more_shan[0], more_shan[1], more_shan[2], more_shan[3]] {
        snap_aoe.core.deck.retain(|x| *x != c);
        snap_aoe.core.discard.retain(|x| *x != c);
    }
    snap_aoe.core.deck.extend([more_shan[0], more_shan[1], more_shan[2], more_shan[3], wanjian, nanman]);
    let mut e_aoe = content_native_loader::restore_locked_engine(&snap_aoe, &v).unwrap();
    submit(&mut e_aoe, 0, GameAction::PlayCard { card: huoge, targets: vec![] });
    pass_window(&mut e_aoe);
    let aoe_pick_id = e_aoe.open_choice().unwrap().id;
    let aoe_opts = e_aoe.open_choice().unwrap().spec.options.clone();
    assert!(aoe_opts.contains(&ChoiceOption::Card(nanman)) && aoe_opts.contains(&ChoiceOption::Card(wanjian)));
    submit(&mut e_aoe, 0, GameAction::Answer {
        choice_id: aoe_pick_id,
        selection: vec![ChoiceOption::Card(nanman), ChoiceOption::Card(wanjian)],
    });
    pass_window(&mut e_aoe);
    assert_eq!(e_aoe.debug_hp(1), 2, "霍格打出南蛮+万箭须对无杀闪 1 号共造成 2 点伤害");
    assert_eq!(e_aoe.debug_hp(2), 2);
    assert_eq!(e_aoe.debug_hp(3), 2);
    assert!(e_aoe.save().core.table.is_empty());
    assert_zone_conservation(&e_aoe);

    // `Jiedao` 无人持武器：禁止入霍格选牌池（否则 card_use 触发 jiedao two targets panic）
    let jiedao = card_of_kind(&e0, CardKind::Jiedao, None);
    let e_j0 = inject_hand(&e0, &v, &[huoge], 0);
    let mut snap_j0 = e_j0.save();
    for c in [jiedao, sha, more_shan[0], more_shan[1], more_shan[2], more_shan[3]] {
        snap_j0.core.deck.retain(|x| *x != c);
        snap_j0.core.discard.retain(|x| *x != c);
    }
    snap_j0.core.deck.extend([more_shan[0], more_shan[1], more_shan[2], more_shan[3], sha, jiedao]);
    let mut e_j0 = content_native_loader::restore_locked_engine(&snap_j0, &v).unwrap();
    submit(&mut e_j0, 0, GameAction::PlayCard { card: huoge, targets: vec![] });
    pass_window(&mut e_j0);
    let j0_opts = e_j0.open_choice().unwrap().spec.options.clone();
    assert!(!j0_opts.contains(&ChoiceOption::Card(jiedao)), "无人持武器时借刀杀人不得出现在霍格选项中");
    let j0_id = e_j0.open_choice().unwrap().id;
    submit(&mut e_j0, 0, GameAction::Answer { choice_id: j0_id, selection: vec![] });
    pass_window(&mut e_j0);
    assert_zone_conservation(&e_j0);

    // `Jiedao` 有武器持有者(1 号持武器, 2 号在距离 1 射程内)：两步选 holder->victim 含存档恢复
    let five_shan = find_cards_matching(&e0, |d| d.kind == CardKind::Shan, 5);
    let mut e_j1 = inject_hand(&e0, &v, &[huoge], 0);
    e_j1 = inject_equip(&e_j1, &v, weapon, 1);
    let mut snap_j1 = e_j1.save();
    for &c in five_shan.iter().chain(std::iter::once(&jiedao)) {
        snap_j1.core.deck.retain(|x| *x != c);
        snap_j1.core.discard.retain(|x| *x != c);
    }
    snap_j1.core.deck.extend([five_shan[0], five_shan[1], five_shan[2], five_shan[3], five_shan[4], jiedao]);
    let mut e_j1 = content_native_loader::restore_locked_engine(&snap_j1, &v).unwrap();
    submit(&mut e_j1, 0, GameAction::PlayCard { card: huoge, targets: vec![] });
    pass_window(&mut e_j1);
    let j1_pick = e_j1.open_choice().unwrap().id;
    assert!(e_j1.open_choice().unwrap().spec.options.contains(&ChoiceOption::Card(jiedao)));
    submit(&mut e_j1, 0, GameAction::Answer { choice_id: j1_pick, selection: vec![ChoiceOption::Card(jiedao)] });
    assert_eq!(e_j1.open_choice().unwrap().spec.prompt, "gwmaoxian_huoge_target");
    let (e_j1, _) = save_restore_answer(e_j1, &v, 0, vec![ChoiceOption::Player(1)]);
    assert_eq!(e_j1.open_choice().unwrap().spec.prompt, "gwmaoxian_huoge_target");
    let (mut e_j1, _) = save_restore_answer(e_j1, &v, 0, vec![ChoiceOption::Player(2)]);
    pass_window(&mut e_j1);
    assert_eq!(e_j1.open_choice().unwrap().spec.prompt, "jiedao_sha_or_give");
    let jd_id = e_j1.open_choice().unwrap().id;
    submit(&mut e_j1, 1, GameAction::Answer { choice_id: jd_id, selection: vec![ChoiceOption::Tag(0)] });
    pass_window(&mut e_j1);
    assert_eq!(e_j1.debug_hp(2), 3, "霍格借刀杀人令 1 号对 2 号造成 1 点伤害");
    assert!(e_j1.save().core.table.is_empty());
    assert_zone_conservation(&e_j1);
}
