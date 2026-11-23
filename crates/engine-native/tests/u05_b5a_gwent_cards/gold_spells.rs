//! `card/gwent/card.js:100..860` 12 张金卡法术真实事件与结束出牌阶段/入 `u05_vanished` 验收测试。

use super::common::*;
use engine_native::catalog::CardKind;
use engine_native::{ChoiceOption, CommandOutcome, GameAction, LogEvent, Phase, RejectReason};

#[test]
fn u05_b5a_gold_hudiewu_yigeniyin_leizhoushu_aerdeyin() {
    let (v, e0) = setup_gwent_locked(5510);

    // 1. 蝴蝶舞 (`card/gwent/card.js:100`): 无他人装备/判定牌拒绝；有装备/判定牌替换为 gw_dieyi_* 并结束出牌阶段
    let hudiewu = find_card(&e0, "gw_hudiewu");
    let mut e_bad = inject_hand(&e0, &v, &[hudiewu], 0);
    assert!(matches!(
        submit(&mut e_bad, 0, GameAction::PlayCard { card: hudiewu, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    let weapon = card_of_kind(&e0, CardKind::Weapon, None);
    let le = card_of_kind(&e0, CardKind::Le, None);
    let mut e = inject_hand(&e0, &v, &[hudiewu], 0);
    e = inject_equip(&e, &v, weapon, 1);
    e = inject_judge(&e, &v, le, 2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: hudiewu, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_ne!(e.turn().phase, Phase::Play, "金卡法术结算后结束出牌阶段");
    assert!(is_vanished(&e, hudiewu), "蝴蝶舞 vanish=true 进入 u05_vanished");
    assert!(
        e.save().core.players[1]
            .equips
            .iter()
            .any(|c| e.catalog().cards[*c as usize].name == "gw_dieyi_equip1"),
        "1 号武器替换为 gw_dieyi_equip1"
    );
    assert!(
        e.save().core.players[2]
            .judge
            .iter()
            .any(|c| e.catalog().cards[*c as usize].name == "gw_dieyi_judge"),
        "2 号判定牌替换为 gw_dieyi_judge"
    );
    assert_zone_conservation(&e);

    // 2. 伊格尼印 (`card/gwent/card.js:158`): 最高体力敌方与全场最高体力者受火伤
    let yigeni = find_card(&e0, "gw_yigeniyin");
    let mut e = inject_hand(&e0, &v, &[yigeni], 0);
    e.debug_set_hp(1, 5);
    e.debug_set_hp(3, 3);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: yigeni, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.debug_hp(1) < 5, "伊格尼印对最高体力敌方造成火伤");
    assert!(
        e.log()
            .iter()
            .any(|l| matches!(l, LogEvent::DamageDealt { target: 1, amount: 1, .. })),
        "伊格尼印造成伤害日志"
    );
    assert!(is_vanished(&e, yigeni));
    assert_ne!(e.turn().phase, Phase::Play);
    assert_zone_conservation(&e);

    // 3. 雷咒术 (`card/gwent/card.js:245`) & 阿尔德印 (`card/gwent/card.js:293`)
    let lei = find_card(&e0, "gw_leizhoushu");
    let aer = find_card(&e0, "gw_aerdeyin");
    let mut e = inject_hand(&e0, &v, &[lei], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: lei, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(is_vanished(&e, lei));
    assert_ne!(e.turn().phase, Phase::Play);
    let lei_def = e.catalog().skill_by_name("gw_leizhoushu").unwrap();
    assert!(e.save().core.skills.contains_key(&(0, lei_def)));

    // 阿尔德印 (`card/gwent/card.js:312`): 对随机正面敌方造成 1 伤并翻面，授予 `gw_aerdeyin`
    let mut e = inject_hand(&e0, &v, &[aer], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: aer, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let aer_def = e.catalog().skill_by_name("gw_aerdeyin").unwrap();
    let hit = [1u8, 3u8].into_iter().find(|&p| e.save().core.players[p as usize].face_down);
    assert!(hit.is_some(), "阿尔德印应命中一名正面敌方并翻面");
    assert!(e.save().core.skills.contains_key(&(hit.unwrap(), aer_def)));
    assert!(is_vanished(&e, aer));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_gold_ansha_xinsheng_niuqu_zhongmo_ganhan() {
    let (v, e0) = setup_gwent_locked(5511);

    // 1. 暗杀 (`card/gwent/card.js:356`): 无 1 血敌方拒绝；命中 1 血敌方直接死亡
    let ansha = find_card(&e0, "gw_ansha");
    let mut e = inject_hand(&e0, &v, &[ansha], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: ansha, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    e.debug_set_hp(1, 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: ansha, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(!e.save().core.players[1].alive, "暗杀直接击毙 1 血敌方");
    assert!(is_vanished(&e, ansha));
    assert_zone_conservation(&e);

    // 2. 扭曲之境 (`card/gwent/card.js:514`): 全员等血拒绝；不等血对调最高最低（含极限恢复）
    let niuqu = find_card(&e0, "gw_niuquzhijing");
    let mut e = inject_hand(&e0, &v, &[niuqu], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: niuqu, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    e.debug_set_hp(1, 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: niuqu, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(is_vanished(&e, niuqu));
    assert_ne!(e.turn().phase, Phase::Play);
    assert_zone_conservation(&e);

    // 3. 新生 (`card/gwent/card.js:398`): 两段询问 PickTag -> save_restore_answer -> SelectPlayers
    let xinsheng = find_card(&e0, "gw_xinsheng");
    let mut e = inject_hand(&e0, &v, &[xinsheng], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: xinsheng, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_xinsheng_general");
    let (e, out) = save_restore_answer(e, &v, 0, vec![ChoiceOption::Tag(0)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_xinsheng_target");
    let (mut e, out) = save_restore_answer(e, &v, 0, vec![ChoiceOption::Player(1)]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert!(is_vanished(&e, xinsheng));
    assert_ne!(e.turn().phase, Phase::Play);
    assert_zone_conservation(&e);

    // 4. 终末之战 (`card/gwent/card.js:609`) & 干旱 (`card/gwent/card.js:661`)
    let zhongmo = find_card(&e0, "gw_zhongmozhizhan");
    let extra = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 2);
    let mut e = inject_hand(&e0, &v, &[zhongmo, extra[0]], 0);
    e = inject_hand(&e, &v, &[extra[1]], 1);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: zhongmo, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(e.debug_hand(0).is_empty() && e.debug_hand(1).is_empty(), "终末之战清空全场手牌");
    assert!(is_vanished(&e, zhongmo));
    assert_zone_conservation(&e);

    let ganhan = find_card(&e0, "gw_ganhan");
    let mut e = inject_hand(&e0, &v, &[ganhan], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: ganhan, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert!(is_vanished(&e, ganhan));
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_gold_huangjiashenpan_tunshi_chongci() {
    let (v, e0) = setup_gwent_locked(5512);

    // 1. 皇家审判 (`card/gwent/card.js:737`): PickTag 从牌堆/弃堆选金法术入手 + save->restore
    let huangjia = find_card(&e0, "gw_huangjiashenpan");
    let mut e = inject_hand(&e0, &v, &[huangjia], 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: huangjia, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.open_choice().unwrap().spec.prompt, "gw_huangjiashenpan_pick");
    let first_tag = e.open_choice().unwrap().spec.options[0];
    let (mut e, out) = save_restore_answer(e, &v, 0, vec![first_tag]);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(0).len(), 1, "皇家审判获得 1 张金卡法术");
    assert!(is_vanished(&e, huangjia));
    assert_ne!(e.turn().phase, Phase::Play);
    assert_zone_conservation(&e);

    // 2. 吞噬 (`card/gwent/card.js:779`): maxHp<=1 或敌无非装备非charlotte技能时拒绝；成功时夺技能、自降 1 上限+掉 1 血、目标+1 上限+回 1
    let tunshi = find_card(&e0, "gw_tunshi");
    let mut e_bad = inject_hand(&e0, &v, &[tunshi], 0);
    assert!(matches!(
        submit(&mut e_bad, 0, GameAction::PlayCard { card: tunshi, targets: vec![] }),
        CommandOutcome::Rejected(RejectReason::IllegalTargets)
    ));
    let mut e = grant_skill(&e0, &v, "gw_aerdeyin", 1);
    e = inject_hand(&e, &v, &[tunshi], 0);
    e.debug_set_hp(1, 2);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: tunshi, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    let aer_def = e.catalog().skill_by_name("gw_aerdeyin").unwrap();
    assert!(e.save().core.skills.contains_key(&(0, aer_def)), "吞噬偷得敌方技能");
    assert!(!e.save().core.skills.contains_key(&(1, aer_def)));
    assert_eq!(e.save().core.players[0].max_hp, 3);
    assert_eq!(e.debug_hp(0), 3, "吞噬先掉1体力再减1上限应为3/3");
    assert_eq!(e.save().core.players[1].max_hp, 5);
    assert_eq!(e.debug_hp(1), 3);
    assert!(is_vanished(&e, tunshi));
    assert_zone_conservation(&e);

    // 3. 冲刺 (`card/gwent/card.js:844`): 弃手+装换回同类别牌 + 1 银/铜法术
    let chongci = find_card(&e0, "gw_chongci");
    let sha = card_of_kind(&e0, CardKind::Sha, None);
    let weapon = card_of_kind(&e0, CardKind::Weapon, None);
    let mut e = inject_hand(&e0, &v, &[chongci, sha], 0);
    e = inject_equip(&e, &v, weapon, 0);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: chongci, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(e.debug_hand(0).len(), 2, "换取 1 张基本牌 + 1 张非金法术");
    assert_eq!(e.save().core.players[0].equips.len(), 1, "重新装备 1 件同槽位装备");
    assert!(is_vanished(&e, chongci));
    assert_ne!(e.turn().phase, Phase::Play);
    assert_zone_conservation(&e);
}

#[test]
fn u05_b5a_round2_gold_lifo_zhongmo_chongci_regressions() {
    let (v, e0) = setup_gwent_locked(5513);

    // #1 + #2: `gw_yigeniyin` CDP (`[4, 5, 4, 3] -> [3, 3, 3, 3]`)：step 0 先伤最高敌方，再重算全场最高hp并伤 [0,1,2]
    let yigeni = find_card(&e0, "gw_yigeniyin");
    let mut e = inject_hand(&e0, &v, &[yigeni], 0);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(1, 5);
    e.debug_set_hp(2, 4);
    e.debug_set_hp(3, 3);
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: yigeni, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e);
    assert_eq!(
        (e.debug_hp(0), e.debug_hp(1), e.debug_hp(2), e.debug_hp(3)),
        (3, 3, 3, 3),
        "伊格尼印 step 0 将 1 号从 5 压到 4 后，step 2 重算全场最高=4 并命中 0,1,2"
    );
    assert_ne!(e.turn().phase, Phase::Play);
    assert_zone_conservation(&e);

    // #2: 金卡结算发射 `Damage` 后须先结算 `Dying`/救人并存档恢复，再结束 `Phase::Play` 进入 `Phase::Discard`
    let jieluote = find_card(&e0, "gwmaoxian_jieluote");
    let shas = find_cards_matching(&e0, |d| d.kind == CardKind::Sha, 3);
    let tao = card_of_kind(&e0, CardKind::Tao, None);
    let mut e_j = inject_hand(&e0, &v, &[jieluote, shas[0], shas[1]], 0);
    e_j = inject_hand(&e_j, &v, &[tao], 1); // 1 号持桃触发真实濒死救援窗口（Save->Restore->Pass 后才死亡并结束 Play）
    e_j.debug_set_hp(0, 1);
    e_j.debug_set_hp(1, 1);
    assert!(matches!(
        submit(&mut e_j, 0, GameAction::PlayCard { card: jieluote, targets: vec![1] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    assert_eq!(e_j.turn().phase, Phase::Play, "杰洛特伤害尚在结算中，Play 阶段不得抢先结束进入 Discard");
    let dying_ask = e_j.open_choice().cloned().expect("1 号进入濒死救援询问");
    assert_eq!(dying_ask.spec.prompt, "dying_save");
    let (mut e_j, _) = save_restore_answer(e_j, &v, dying_ask.observer, vec![ChoiceOption::Pass]);
    drain_default(&mut e_j);
    assert!(!e_j.debug_alive(1), "杰洛特伤害完整结算击毙 1 号");
    assert_ne!(e_j.turn().phase, Phase::Play, "濒死与死亡结算完毕后才结束 Play 阶段");
    assert_zone_conservation(&e_j);

    // #6: `gw_zhongmozhizhan` 对 `lose(cards)._triggered = null`（真事件栈零 `CardsLost`），同时清理 `declared_slots`
    let zhongmo = find_card(&e0, "gw_zhongmozhizhan");
    let weapon = card_of_kind(&e0, CardKind::Weapon, None);
    let mut e_z = inject_hand(&e0, &v, &[zhongmo], 0);
    e_z = inject_hand(&e_z, &v, &[shas[0]], 1);
    e_z = inject_equip(&e_z, &v, weapon, 1);
    let mut snap = e_z.save();
    snap.core.declared_slots.insert(weapon, engine_native::catalog::EquipSlot::Armor);
    let mut core_z = snap.core;
    let mut res_z = engine_native::Resolution::new(128);
    let mut log_z = Vec::new();
    engine_native::resolution::push_frame(
        &mut res_z,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(zhongmo, 0, vec![])),
    );
    let mut saw_cards_lost = false;
    loop {
        if res_z.stack.iter().any(|f| matches!(f.data, engine_native::EventData::CardsLost { .. })) {
            saw_cards_lost = true;
        }
        match engine_native::resolution::run(&mut core_z, &mut res_z, e_z.catalog(), &mut log_z, Some(1)) {
            engine_native::resolution::StepOutcome::Done => break,
            engine_native::resolution::StepOutcome::BudgetExhausted
            | engine_native::resolution::StepOutcome::Yielded => continue,
            engine_native::resolution::StepOutcome::Fault => panic!("unexpected fault"),
            engine_native::resolution::StepOutcome::Pending => {
                let c = res_z.open_choice.take().unwrap();
                res_z.answer = Some(if c.spec.options.contains(&ChoiceOption::Pass) { vec![ChoiceOption::Pass] } else { vec![] });
            }
        }
    }
    assert!(!saw_cards_lost, "终末之战 lose()._triggered=null 不得产生任何 CardsLost 帧");
    assert!(core_z.players[1].hand.is_empty() && core_z.players[1].equips.is_empty());
    assert!(!core_z.declared_slots.contains_key(&weapon), "declared_slots 已清理");
    let mut snap_done = e_z.save();
    snap_done.core = core_z;
    let e_z_done = content_native_loader::restore_locked_engine(&snap_done, &v).unwrap();
    assert_zone_conservation(&e_z_done);

    // #7: `gw_chongci` 结尾触发 `player.tempHide()` (`yinshen_active == true`) 使下家无法用【杀】指定
    let chongci = find_card(&e0, "gw_chongci");
    let mut e_c = inject_hand(&e0, &v, &[chongci], 0);
    assert!(matches!(
        submit(&mut e_c, 0, GameAction::PlayCard { card: chongci, targets: vec![] }),
        CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
    ));
    pass_window(&mut e_c);
    assert!(e_c.save().core.players[0].yinshen_active, "冲刺后 0 号潜行置位");
    advance_to_seat_play(&mut e_c, 1);
    let mut e_c = inject_hand(&e_c, &v, &[shas[0]], 1);
    assert!(
        matches!(
            submit(&mut e_c, 1, GameAction::PlayCard { card: shas[0], targets: vec![0] }),
            CommandOutcome::Rejected(RejectReason::IllegalTargets)
        ),
        "0 号潜行期间 1 号不得对其使用【杀】"
    );
    assert_zone_conservation(&e_c);

    // #280(3): `gw_chongci` `DrawSettle` 期间若使用者已死亡不得 pull 卡到死人/装入死人装备区；
    // 若使用者存活且中途装备了同槽位牌，复装时须正确替换旧装备不占双槽
    let armors = find_cards_matching(&e0, |d| d.kind == CardKind::Armor, 2);
    let mut e_cd = inject_hand(&e0, &v, &[chongci, shas[0]], 0);
    e_cd = inject_equip(&e_cd, &v, armors[0], 0);
    let mut core_cd = e_cd.save().core;
    let mut res_cd = engine_native::Resolution::new(128);
    let mut log_cd = Vec::new();
    engine_native::resolution::push_frame(
        &mut res_cd,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(chongci, 0, vec![])),
    );
    let _ = engine_native::resolution::run(&mut core_cd, &mut res_cd, e0.catalog(), &mut log_cd, Some(2));
    // 在 yield 后（DrawSettle 运行前）模拟 0 号死亡
    core_cd.players[0].alive = false;
    core_cd.players[0].hp = 0;
    let mut out_cd = engine_native::resolution::run(&mut core_cd, &mut res_cd, e0.catalog(), &mut log_cd, None);
    while out_cd == engine_native::resolution::StepOutcome::Pending {
        res_cd.open_choice = None;
        res_cd.answer = Some(Vec::new());
        out_cd = engine_native::resolution::run(&mut core_cd, &mut res_cd, e0.catalog(), &mut log_cd, None);
    }
    assert_eq!(out_cd, engine_native::resolution::StepOutcome::Done);
    assert!(core_cd.players[0].hand.is_empty() && core_cd.players[0].equips.is_empty(), "死后不得抓牌或上装备");
    let mut snap_cd = e0.save();
    snap_cd.core = core_cd;
    let e_cd_done = content_native_loader::restore_locked_engine(&snap_cd, &v).unwrap();
    assert_zone_conservation(&e_cd_done);

    // 存活场景下，yield 期间若槽位被先放入一件同槽位防具，冲刺复装须替换旧防具仅留 1 件
    let mut e_cs = inject_hand(&e0, &v, &[chongci], 0);
    e_cs = inject_equip(&e_cs, &v, armors[0], 0);
    let mut core_cs = e_cs.save().core;
    let mut res_cs = engine_native::Resolution::new(128);
    let mut log_cs = Vec::new();
    engine_native::resolution::push_frame(
        &mut res_cs,
        engine_native::EventData::CardUsed(engine_native::machine::card_use(chongci, 0, vec![])),
    );
    let _ = engine_native::resolution::run(&mut core_cs, &mut res_cs, e0.catalog(), &mut log_cs, Some(2));
    // 手动将 armors[1] 从 deck/discard 提前放到 0 号防具槽，模拟 yield 中途上装备
    core_cs.deck.retain(|x| *x != armors[1]);
    core_cs.discard.retain(|x| *x != armors[1]);
    core_cs.players[0].equips.push(armors[1]);
    let mut out_cs = engine_native::resolution::run(&mut core_cs, &mut res_cs, e0.catalog(), &mut log_cs, None);
    while out_cs == engine_native::resolution::StepOutcome::Pending {
        res_cs.open_choice = None;
        res_cs.answer = Some(Vec::new());
        out_cs = engine_native::resolution::run(&mut core_cs, &mut res_cs, e0.catalog(), &mut log_cs, None);
    }
    assert_eq!(out_cs, engine_native::resolution::StepOutcome::Done);
    assert_eq!(core_cs.players[0].equips.len(), 1, "同槽位仅保留 1 件装备不得双防具占槽");
    let mut snap_cs = e0.save();
    snap_cs.core = core_cs;
    let e_cs_done = content_native_loader::restore_locked_engine(&snap_cs, &v).unwrap();
    assert_zone_conservation(&e_cs_done);
}
