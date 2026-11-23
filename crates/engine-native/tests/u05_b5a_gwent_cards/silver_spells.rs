//! `card/gwent/card.js:930..1455` 13 张银卡法术（含 `gw_fuyuan`）正反用例与存档恢复校验。

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, Phase, RejectReason};

#[test]
fn u05_b5a_silver_youer_tongdi_fuyuan_zhuoshao() {
    let (v, e0) = setup_gwent_locked(5520);

    // 1. 诱饵 (`card/gwent/card.js:930`): 空手/自己拒绝；非空将目标手牌转我手且记 gw_youer_pile，回合结束交回并 vanish
    let youer = find_card(&e0, "gw_youer");
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let mut e_bad = inject_hand(&e0, &v, &[youer], 0);
    assert!(matches!(
        submit(&mut e_bad, 0, GameAction::PlayCard { card: youer, targets: vec![1] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    let mut e = inject_hand(&e0, &v, &[youer], 0);
    e = inject_hand(&e, &v, &[sha], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: youer, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.turn().phase, Phase::Play, "银卡法术不结束出牌阶段");
    assert!(is_vanished(&e, youer), "诱饵 vanish=true");
    assert_eq!(e.debug_hand(0).len(), 1, "诱饵令自己摸 1");
    assert!(
        e.state_core()
            .vm_aux
            .piles
            .get(&(1, "gw_youer_pile".to_string()))
            .is_some_and(|p| p.contains(&sha))
            && e.debug_hand(1).is_empty(),
        "诱饵将 1 号手牌移至 gw_youer_pile"
    );
    advance_past_turn(&mut e, 0);
    assert!(e.debug_hand(1).contains(&sha), "回合结束 1 号取回原手牌");
    assert_zone_conservation(&e);

    // 2. 通敌 (`card/gwent/card.js:968`): 选 1 张获得，目标摸 1 + save->restore
    let tongdi = find_card(&e0, "gw_tongdi");
    let mut e = inject_hand(&e0, &v, &[tongdi], 0);
    e = inject_hand(&e, &v, &[sha], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: tongdi, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_tongdi_pick");
    let (mut e, out) = save_restore_answer(e, &v, 0, vec![ChoiceOption::Card(sha)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert!(e.debug_hand(0).contains(&sha));
    assert_eq!(e.debug_hand(1).len(), 1);
    assert!(is_vanished(&e, tongdi));
    assert_zone_conservation(&e);

    // 3. 复原 (`card/gwent/card.js:1001`): 非濒死 Play 不可用；濒死可用救回并令濒死者摸 1
    let fuyuan = find_card(&e0, "gw_fuyuan");
    let aozu = find_card(&e0, "gw_aozuzhilei");
    let mut e = inject_hand(&e0, &v, &[fuyuan, aozu], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: fuyuan, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::WrongPhase)
    ));
    e.debug_set_hp(1, 1);
    e.debug_set_hp(0, 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: aozu, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    // 1 号濒死询问 0 号（`dying_save`）
    while let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "dying_save" && c.observer == 0 && c.spec.options.contains(&ChoiceOption::Card(fuyuan)) {
            submit(&mut e, 0, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(fuyuan)] });
        } else {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
                vec![ChoiceOption::Pass]
            } else {
                vec![]
            };
            submit(&mut e, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
        }
    }
    assert!(e.save().core.players[1].alive && e.debug_hp(1) >= 1, "复原救回濒死者");
    assert!(e.debug_hand(1).len() >= 2, "复原摸 1 + 奥祖之雷摸 1");
    assert!(is_vanished(&e, fuyuan));
    assert_zone_conservation(&e);

    // 4. 灼烧 (`card/gwent/card.js:1027`): 非最高体力拒绝；最多体力的多个目标均受 1 火伤
    let zhuoshao = find_card(&e0, "gw_zhuoshao");
    let mut e = inject_hand(&e0, &v, &[zhuoshao], 0);
    e.debug_set_hp(1, 5);
    e.debug_set_hp(3, 5);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zhuoshao, targets: vec![2] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zhuoshao, targets: vec![1, 3] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!((e.debug_hp(1), e.debug_hp(3)), (4, 4));
    assert!(
        e.log()
            .iter()
            .any(|l| matches!(l, LogEvent::DamageDealt { target: 1, amount: 1, .. })),
        "灼烧造成伤害日志"
    );
    assert!(is_vanished(&e, zhuoshao));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_silver_butianshu_shizizhaohuan_zuihou_ziran_weather_nest() {
    let (v, e0) = setup_gwent_locked(5521);

    // 1. 步天术 (`card/gwent/card.js:1057`): 挂 1 延时锦囊到目标判定区 + 排重 + save->restore
    let butian = find_card(&e0, "gw_butianshu");
    let le = card_of_kind(&e0, CardKind::Le, None);
    let mut e = inject_hand(&e0, &v, &[butian], 0);
    e = inject_judge(&e, &v, le, 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: butian, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "gw_butianshu_delay");
    assert!(!c.spec.options.contains(&ChoiceOption::Tag(0)), "已有乐不思蜀时应排除 tag 0");
    let pick = c.spec.options[0];
    let (mut e, out) = save_restore_answer(e, &v, 0, vec![pick]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert_eq!(e.save().core.players[1].judge.len(), 2);
    assert!(is_vanished(&e, butian));

    // 2. 十字召唤 (`card/gwent/card.js:1137`) & 最后愿望 (`card/gwent/card.js:1189`)
    let shizi = find_card(&e0, "gw_shizizhaohuan");
    let zuihou = find_card(&e0, "gw_zuihouyuanwang");
    let mut e = inject_hand(&e0, &v, &[shizi, zuihou], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: shizi, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(0).len(), 3, "十字召唤获 2 牌（zuihou+2）");
    assert!(is_vanished(&e, shizi));
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zuihou, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_zuihouyuanwang_discard");
    let h0 = e.debug_hand(0);
    assert_eq!(h0.len(), 6, "2 张起手 + 4 人场摸 4 = 6");
    let (mut e, out) = save_restore_answer(
        e,
        &v,
        0,
        h0[..4].iter().copied().map(ChoiceOption::Card).collect(),
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2);
    assert!(is_vanished(&e, zuihou));
    assert_zone_conservation(&e);

    // 3. 自然馈赠 (`card/gwent/card.js:1228`): A（无目标瘟疫 tag=10）vs B（有目标昆恩 tag=8 + save->restore）
    let ziran = find_card(&e0, "gw_zirankuizeng");
    let mut e_a = inject_hand(&e0, &v, &[ziran], 0);
    e_a.debug_set_hp(1, 1);
    assert!(matches!(
        submit(&mut e_a, 0, GameAction::PlayCard { card: ziran, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_a);
    let (mut e_a, _) = save_restore_answer(e_a, &v, 0, vec![ChoiceOption::Tag(10)]); // gw_wenyi
    pass_window(&mut e_a);
    assert_eq!(e_a.debug_hp(1), 0 - 0, "虚拟瘟疫令 1 号失去 1 血（或进入濒死）");
    drain_default(&mut e_a);
    assert!(is_vanished(&e_a, ziran));

    let mut e_b = inject_hand(&e0, &v, &[ziran], 0);
    assert!(matches!(
        submit(&mut e_b, 0, GameAction::PlayCard { card: ziran, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_b);
    let (e_b, _) = save_restore_answer(e_b, &v, 0, vec![ChoiceOption::Tag(8)]); // gw_kunenfayin
    assert_eq!(e_b.open_choice().unwrap().spec.prompt, "gw_zirankuizeng_target");
    let (mut e_b, _) = save_restore_answer(e_b, &v, 0, vec![ChoiceOption::Player(0)]);
    pass_window(&mut e_b);
    let kunen_def = e_b.catalog().skill_by_name(engine_native::content::host::GW_KUNEN_STATE_KEY).unwrap();
    assert!(e_b.save().core.skills.contains_key(&(0, kunen_def)), "自然馈赠虚拟昆恩法印生效");

    // 4. 怒海风暴 / 白霜 / 雹暴术 / 怪物巢穴 (`card/gwent/card.js:1345..1455`)：此 4 张不 vanish
    let nuhai = find_card(&e0, "gw_nuhaifengbao");
    let baishuang = find_card(&e0, "gw_baishuang");
    let baobao = find_card(&e0, "gw_baobaoshu");
    let chaoxue = find_card(&e0, "gw_guaiwuchaoxue");
    let mut e = inject_hand(&e0, &v, &[nuhai, baishuang, baobao, chaoxue], 0);
    for (card, tgts) in [
        (nuhai, vec![1]),
        (baishuang, vec![1, 3]),
        (baobao, vec![1]),
        (chaoxue, vec![]),
    ] {
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card, targets: tgts }),
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ));
        pass_window(&mut e);
        assert!(!is_vanished(&e, card), "后 4 张银法术无 vanish=true");
    }
    // 怪物巢穴每阶段限用 1 次
    let chaoxue2 = find_card(&e0, "gw_guaiwuchaoxue");
    let mut e = inject_hand(&e, &v, &[chaoxue2], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: chaoxue2, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::CardLimit)
    ));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_round2_youer_tongdi_baobaoshu_shizizhaohuan_regressions() {
    let (v, e0) = setup_gwent_locked(5522);

    // #10 `gw_youer`: CardsLost 带 Hand origin 且在自己摸 1 前 yield（CardsLost 先结算完并出栈，随后才发射 Draw）
    let youer = find_card(&e0, "gw_youer");
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let mut core_y = inject_hand(&inject_hand(&e0, &v, &[youer], 0), &v, &[sha], 1).save().core;
    let mut res_y = engine_native::Resolution::new(128);
    let mut log_y = Vec::new();
    engine_native::resolution::push_frame(
        &mut res_y,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(youer, 0, vec![1])),
    );
    let mut saw_hand_lost = false;
    let mut saw_draw_before_lost_settled = false;
    let mut saw_draw_after_lost_settled = false;
    loop {
        let has_lost = res_y.stack.iter().any(|f| {
            matches!(
                &f.data,
                engine_native::EventData::CardsLost { player: 1, origins, .. }
                if origins == &vec![engine_native::event::LostOrigin::Hand]
            )
        });
        let has_draw = res_y.stack.iter().any(|f| matches!(f.data, engine_native::EventData::Draw { player: 0, requested: 1, .. }));
        if has_lost {
            saw_hand_lost = true;
            if has_draw {
                saw_draw_before_lost_settled = true;
            }
        } else if saw_hand_lost && has_draw {
            saw_draw_after_lost_settled = true;
        }
        match engine_native::resolution::run(&mut core_y, &mut res_y, e0.catalog(), &mut log_y, Some(1)) {
            engine_native::resolution::StepOutcome::Done => break,
            engine_native::resolution::StepOutcome::BudgetExhausted
            | engine_native::resolution::StepOutcome::Yielded => continue,
            engine_native::resolution::StepOutcome::Fault => panic!("unexpected fault"),
            engine_native::resolution::StepOutcome::Pending => {
                let c = res_y.open_choice.take().unwrap();
                res_y.answer = Some(if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] } else { vec![] });
            }
        }
    }
    assert!(saw_hand_lost, "诱饵 CardsLost 须携带 LostOrigin::Hand");
    assert!(!saw_draw_before_lost_settled, "诱饵 CardsLost 结算完前不得提前压入摸 1（LIFO）");
    assert!(saw_draw_after_lost_settled, "诱饵 CardsLost 结算完毕后才由所有者摸 1");
    assert_eq!(core_y.players[0].hand.len(), 1, "诱饵后 0 号摸 1");
    assert!(core_y.vm_aux.piles.get(&(1, "gw_youer_pile".to_string())).is_some_and(|p| p.contains(&sha)));
    let mut snap_y = e0.save();
    snap_y.core = core_y;
    let e_y_done = content_native_loader::restore_locked_engine(&snap_y, &v).unwrap();
    assert_zone_conservation(&e_y_done);

    // #10 `gw_tongdi`: 优先检索普通 CardKind::Sha（跳过牌堆顶部 FireSha/LeiSha）并在 CardsMoved 后结算 CardsDrawn
    let tongdi = find_card(&e0, "gw_tongdi");
    let tao = card_of_kind(&e0, CardKind::Tao, None);
    let fire_sha = card_of_kind(&e0, CardKind::FireSha, None);
    let plain_sha = card_of_kind(&e0, CardKind::Sha, None);
    let mut e_t = inject_hand(&e0, &v, &[tongdi], 0);
    e_t = inject_hand(&e_t, &v, &[tao], 1);
    let mut snap_t = e_t.save();
    // 只留 1 张 FireSha 与 1 张普通 Sha 在牌堆中，确保通敌仍优先普通 Sha
    for c in snap_t.core.deck.split_off(0) {
        if c == fire_sha || c == plain_sha {
            snap_t.core.deck.push(c);
        } else {
            snap_t.core.discard.push(c);
        }
    }
    if !snap_t.core.deck.contains(&fire_sha) {
        snap_t.core.discard.retain(|x| *x != fire_sha);
        snap_t.core.deck.push(fire_sha);
    }
    if !snap_t.core.deck.contains(&plain_sha) {
        snap_t.core.discard.retain(|x| *x != plain_sha);
        snap_t.core.deck.insert(0, plain_sha);
    }
    let mut e_t = content_native_loader::restore_locked_engine(&snap_t, &v).unwrap();
    assert!(matches!(
        submit(&mut e_t, 0, GameAction::PlayCard { card: tongdi, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_t);
    let td_id = e_t.open_choice().unwrap().id;
    submit(&mut e_t, 0, GameAction::Answer {
        choice_id: td_id,
        selection: vec![ChoiceOption::Card(tao)],
    });
    pass_window(&mut e_t);
    assert_eq!(e_t.debug_hand(1), vec![plain_sha], "通敌优先获得普通【杀】而非火杀");
    assert_zone_conservation(&e_t);

    // #10 `gw_baobaoshu`: 濒死用 `gw_fuyuan` / `gw_xianzumaijiu` 自救不得触发雹暴术弃牌，用 `gw_lang`（autoViewAs: jiu）则触发
    let baobao = find_card(&e0, "gw_baobaoshu");
    let fuyuan = find_card(&e0, "gw_fuyuan");
    let aozu = find_card(&e0, "gw_aozuzhilei");
    let lang = find_card(&e0, "gw_lang");
    let mut e_b = inject_hand(&e0, &v, &[baobao, aozu], 0);
    e_b = inject_hand(&e_b, &v, &[fuyuan, sha], 1);
    e_b.debug_set_hp(0, 1);
    e_b.debug_set_hp(1, 1);
    assert!(matches!(
        submit(&mut e_b, 0, GameAction::PlayCard { card: baobao, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_b);
    assert!(matches!(
        submit(&mut e_b, 0, GameAction::PlayCard { card: aozu, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    while let Some(c) = e_b.open_choice().cloned() {
        assert_ne!(c.spec.prompt, "gw_baobaoshu_discard", "gw_fuyuan 濒死自救不应触发雹暴术");
        if c.spec.prompt == "dying_save" && c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(fuyuan)) {
            submit(&mut e_b, 1, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(fuyuan)] });
        } else {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] } else { vec![] };
            submit(&mut e_b, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
        }
    }
    assert!(e_b.debug_alive(1), "gw_fuyuan 濒死自救成功");
    assert!(e_b.debug_hand(1).contains(&sha), "原手牌 sha 未被雹暴术误弃");

    // 对比组：`gw_lang` 濒死自救视作基本牌【酒】，必须触发雹暴术
    let mut e_l = inject_hand(&e0, &v, &[baobao, aozu], 0);
    e_l = inject_hand(&e_l, &v, &[lang, sha], 1);
    e_l.debug_set_hp(0, 1);
    e_l.debug_set_hp(1, 1);
    assert!(matches!(
        submit(&mut e_l, 0, GameAction::PlayCard { card: baobao, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_l);
    assert!(matches!(
        submit(&mut e_l, 0, GameAction::PlayCard { card: aozu, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    let mut saw_baobao = false;
    while let Some(c) = e_l.open_choice().cloned() {
        if c.spec.prompt == "gw_baobaoshu_discard" {
            saw_baobao = true;
        }
        if c.spec.prompt == "dying_save" && c.observer == 1 && c.spec.options.contains(&ChoiceOption::Card(lang)) {
            submit(&mut e_l, 1, GameAction::Answer { choice_id: c.id, selection: vec![ChoiceOption::Card(lang)] });
        } else {
            let sel = if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] }
            else if c.spec.allow_cancel { vec![] }
            else { vec![c.spec.options[0]] };
            submit(&mut e_l, c.observer, GameAction::Answer { choice_id: c.id, selection: sel });
        }
    }
    assert!(saw_baobao, "gw_lang (autoViewAs: jiu) 濒死自救应触发雹暴术弃牌");
    assert_zone_conservation(&e_l);

    // #11 `gw_shizizhaohuan`: 仅搜 `deck`（牌堆空、弃牌堆有决斗与杀时获得 0 张）
    let shizi = find_card(&e0, "gw_shizizhaohuan");
    let e_s = inject_hand(&e0, &v, &[shizi], 0);
    let mut snap_s = e_s.save();
    let drained = std::mem::take(&mut snap_s.core.deck);
    snap_s.core.discard.extend(drained);
    let mut e_s = content_native_loader::restore_locked_engine(&snap_s, &v).unwrap();
    assert!(matches!(
        submit(&mut e_s, 0, GameAction::PlayCard { card: shizi, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_s);
    assert!(e_s.debug_hand(0).is_empty(), "牌堆空而弃牌堆满时十字召唤获得 0 张");
    assert_zone_conservation(&e_s);
}
