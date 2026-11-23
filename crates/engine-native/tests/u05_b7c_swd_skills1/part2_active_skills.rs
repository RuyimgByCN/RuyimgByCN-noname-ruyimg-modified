use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};

/// `jiefen` & `aojian` & `bingjian` & `huodan` & `zhanxing` & `jianji`: 取消退回 once_used 与延后弃牌。
#[test]
fn u05_b7c_cancel_refunds_and_delayed_discard() {
    let (_pack, mut e) = swd_engine(705_020);
    e.debug_grant(0, "jiefen");
    e.debug_grant(0, "aojian");
    e.debug_grant(0, "bingjian");
    e.debug_grant(0, "huodan");
    e.debug_grant(0, "zhanxing");
    e.debug_grant(0, "jianji");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 布阵：p0 体力 2/4（aojian X=2），有手牌与武器（攻击范围 >=2），p1/p2 有手牌
    e.debug_set_hp(0, 2);
    let w = weapon_with_range(&e, 2);
    equip_slot_card(&mut e, 0, w);
    let bsha = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Spade));
    let r_tao = find_card(&e, |k, s| k == CardKind::Tao && matches!(s, Suit::Heart));
    let spare = find_card(&e, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e, 0, bsha);
    move_card_to_hand(&mut e, 0, r_tao);
    move_card_to_hand(&mut e, 0, spare);
    let p1_c = find_card(&e, |k, _| k == CardKind::Wuzhong);
    move_card_to_hand(&mut e, 1, p1_c);
    let p2_c = find_card(&e, |k, _| k == CardKind::Jiu);
    move_card_to_hand(&mut e, 2, p2_c);

    // 1) aojian：选 2 牌后取消 aojian_targets → 退回 once_used 且不弃牌，再重发并选用攻击距离 <=1 目标（p2 距离 2 不在列）
    let aj = e.catalog().skill_by_name("aojian").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: aj });
    let ch = e.open_choice().unwrap();
    assert_eq!(ch.spec.prompt, "aojian_discard");
    assert_eq!(ch.spec.max, 2);
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(bsha), ChoiceOption::Card(r_tao)] });
    let tgt = e.open_choice().unwrap();
    assert_eq!(tgt.spec.prompt, "aojian_targets");
    assert_eq!((tgt.spec.min, tgt.spec.max), (2, 2));
    assert!(!tgt.spec.options.contains(&ChoiceOption::Player(2)), "奥剑按攻击距离<=1，即使持武器也不含距离 2 的 2 号位");
    assert!(tgt.spec.options.contains(&ChoiceOption::Player(1)), "距离 1 的 1 号位应在列");
    // 取消 aojian_targets
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert_eq!(e.debug_hand(0).len(), 3, "取消奥剑目标窗不得预先弃牌");
    assert!(!e.state_core().skills.get(&(0, aj)).unwrap().once_used, "取消奥剑退回 once_used");
    // 再次发动成功击中攻击距离 1 的 1 号位
    let out_retry = submit(&mut e, 0, GameAction::ActivateSkill { skill: aj });
    assert!(matches!(out_retry, CommandOutcome::Pending { .. }));
    let ch = e.open_choice().unwrap();
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(bsha)] });
    let tgt = e.open_choice().unwrap();
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(e.debug_hp(1), 3, "奥剑取消后重发按攻击距离<=1命中 1 号位");

    // 2) jianji：选装备后取消 jianji_target → 不弃装备、退 once_used；提交时无视防具（穿藤甲/仁王照常杀中）
    let jj = e.catalog().skill_by_name("jianji").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jj });
    let jc = e.open_choice().unwrap();
    assert_eq!(jc.spec.prompt, "jianji_pick");
    let (cid, obs) = (jc.id, jc.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(w)] });
    let jt = e.open_choice().unwrap();
    assert_eq!(jt.spec.prompt, "jianji_target");
    let (cid, obs) = (jt.id, jt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert!(e.debug_equips(0).contains(&w), "取消剑击目标不得预先弃装备");
    assert!(!e.state_core().skills.get(&(0, jj)).unwrap().once_used, "取消剑击退回 once_used");

    // 给 p1 装八卦阵（CardKind::Armor）并持闪 + 牌堆顶放红桃，sword attack from jianji ignores armor -> 跳过八卦判定直入 respond_shan
    let bagua = find_card(&e, |k, _| k == CardKind::Armor);
    equip_slot_card(&mut e, 1, bagua);
    let p1_shan = find_card(&e, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e, 1, p1_shan);
    let h1 = find_card(&e, |k, s| k == CardKind::Tao && matches!(s, Suit::Heart));
    let h2 = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Heart));
    push_top_deck(&mut e, h1);
    push_top_deck(&mut e, h2);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jj });
    let jc = e.open_choice().unwrap();
    let (cid, obs) = (jc.id, jc.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(w)] });
    let jt = e.open_choice().unwrap();
    let (cid, obs) = (jt.id, jt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let r = e.open_choice().expect("剑击无视防具跳过八卦直入 p1 闪询问");
    assert_eq!((r.observer, r.spec.prompt), (1, "respond_shan"));
    assert_eq!(e.state_core().deck.last().copied(), Some(h1), "跳过八卦阵不应消耗牌堆顶红桃判定牌");
    drain_pass_or_first(&mut e);
    assert_eq!(
        e.state_core().vm_aux.flags.get(&(0, "u05_swd_jianji_unequip".to_string())).copied().unwrap_or(0),
        0,
        "剑击结算后必须清除 u05_swd_jianji_unequip"
    );

    // 3) bingjian / huodan / zhanxing / jiefen 各自取消退回 once_used
    for (sk, p_prompt) in [("bingjian", "bingjian_discard"), ("huodan", "huodan_pick"), ("zhanxing", "zhanxing_discard")] {
        let def = e.catalog().skill_by_name(sk).unwrap();
        if let Some(ch_pre) = e.open_choice() {
            let _ = ch_pre;
        }
        // ensure p0 has a black sha, red heart, equip
        let bs2 = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Club));
        move_card_to_hand(&mut e, 0, bs2);
        let r2 = find_card(&e, |k, s| k == CardKind::Tao && matches!(s, Suit::Diamond));
        move_card_to_hand(&mut e, 0, r2);
        let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: def });
        let c = e.open_choice().expect(p_prompt);
        assert_eq!(c.spec.prompt, p_prompt);
        let (cid, obs) = (c.id, c.observer);
        let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
        assert!(!e.state_core().skills.get(&(0, def)).unwrap().once_used, "{sk} 取消必退 once_used");
    }
    // 4) jiefen 取消目标窗退 once_used 且支持 pending 恢复
    let jf = e.catalog().skill_by_name("jiefen").unwrap();
    // 设 p1 手牌 6 > p0
    let mut extras = Vec::new();
    for k in [CardKind::Sha, CardKind::Shan, CardKind::Tao, CardKind::Wuzhong] {
        let c = find_card(&e, |ck, _| ck == k);
        extras.push(c);
    }
    for c in extras {
        move_card_to_hand(&mut e, 1, c);
    }
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: jf });
    let jft = e.open_choice().unwrap();
    let (cid, obs) = (jft.id, jft.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert!(!e.state_core().skills.get(&(0, jf)).unwrap().once_used, "分舵取消必退 once_used");
}

/// `duishi` (`swd/skill.js:3786`) & `mingfu` (`swd/skill.js:3844`) & `nlianji` (`swd/skill.js:3971`)
/// & `yuchen` & `pozhou2` & `lmazui..3` & `hutian` & `huanxing`: 核心规则与存读档。
#[test]
fn u05_b7c_duishi_mingfu_nlianji_yuchen_pozhou2_lmazui_huanxing_hutian() {
    let (pack, mut e) = swd_engine(705_021);
    e.debug_grant(0, "duishi");
    e.debug_grant(0, "mingfu");
    e.debug_grant(0, "lmazui");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) duishi：A 选 p1 同花色弃牌（置 duishi3），p0 同回合仍可对 p2 发动（未中 duishi2），
    //    直到从 p2 获得 1 牌置 duishi2 后本回合才禁用 duishi；且对 p1 不可再选
    let h1 = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Spade));
    let h2 = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Club));
    let p1_spade = find_card(&e, |k, s| k == CardKind::Dismantle && matches!(s, Suit::Spade));
    let p2_heart = find_card(&e, |k, s| k == CardKind::Tao && matches!(s, Suit::Heart));
    move_card_to_hand(&mut e, 0, h1);
    move_card_to_hand(&mut e, 0, h2);
    move_card_to_hand(&mut e, 1, p1_spade);
    move_card_to_hand(&mut e, 2, p2_heart);
    let ds = e.catalog().skill_by_name("duishi").unwrap();
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: ds });
    let c = e.open_choice().unwrap();
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(h1)] });
    let c = e.open_choice().unwrap();
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let c = e.open_choice().unwrap();
    assert_eq!(c.spec.prompt, "duishi_discard_same_suit");
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(p1_spade)] });
    // 同回合可再发动 duishi，且目标列不包含 p1（已 duishi3），仅含 p2
    let out2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: ds });
    assert!(matches!(out2, CommandOutcome::Pending { .. }), "未获得牌时对诗本回合可再发: {out2:?}");
    let c = e.open_choice().unwrap();
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(h2)] });
    let dt = e.open_choice().unwrap();
    assert!(!dt.spec.options.contains(&ChoiceOption::Player(1)), "已弃同花色的 p1 本回合受 duishi3 排除");
    assert!(dt.spec.options.contains(&ChoiceOption::Player(2)));
    let (cid, obs) = (dt.id, dt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(2)] });
    // p2 无梅花 → 开 duishi_gain，pending 存读档恢复
    let snap = e.save();
    let mut er = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复 duishi_gain");
    let dg = er.open_choice().expect("恢复 duishi_gain");
    assert_eq!(dg.spec.prompt, "duishi_gain");
    let (cid, obs) = (dg.id, dg.observer);
    let _ = submit(&mut er, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(p2_heart)] });
    assert!(er.debug_hand(0).contains(&p2_heart), "duishi_gain 获得目标手牌");
    let h_extra = find_card(&er, |k, _| k == CardKind::Jiu);
    move_card_to_hand(&mut er, 0, h_extra);
    let out_blocked = submit(&mut er, 0, GameAction::ActivateSkill { skill: ds });
    assert!(matches!(out_blocked, CommandOutcome::Rejected(_)), "获得牌后置 duishi2 本回合禁发");

    // 2) mingfu：支持装备区梅花且可指定自己；不入弃堆而以 Guiyoujie 声明置入判定区
    let club_eq = find_card(&er, |k, s| k.equip_slot().is_some() && matches!(s, Suit::Club));
    equip_slot_card(&mut er, 0, club_eq);
    let mf = er.catalog().skill_by_name("mingfu").unwrap();
    let disc_before = er.debug_discard_len();
    let _ = submit(&mut er, 0, GameAction::ActivateSkill { skill: mf });
    let mp = er.open_choice().unwrap();
    assert!(mp.spec.options.contains(&ChoiceOption::Card(club_eq)), "命符允许装备区梅花");
    let (cid, obs) = (mp.id, mp.observer);
    let _ = submit(&mut er, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(club_eq)] });
    let mt = er.open_choice().unwrap();
    assert!(!mt.spec.options.contains(&ChoiceOption::Player(0)), "命符不得指定自己");
    let (cid, obs) = (mt.id, mt.observer);
    let _ = submit(&mut er, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    assert_eq!(er.debug_discard_len(), disc_before, "discard:false 不进入弃牌堆");
    assert_eq!(er.state_core().players[1].judge, vec![club_eq]);
    assert_eq!(er.state_core().declared_kinds.get(&club_eq), Some(&CardKind::Guiyoujie));

    // 3) lmazui + lmazui2 + lmazui3：伤前 -1 降至 0（DamageZero）时仍在结算后归还且可共存 off_court
    let (_pack2, mut e3) = swd_engine(705_022);
    e3.debug_grant(0, "lmazui");
    advance_to_play(&mut e3, 0);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    let m_blk = find_card(&e3, |k, s| k == CardKind::Dismantle && matches!(s, Suit::Spade | Suit::Club));
    move_card_to_hand(&mut e3, 0, m_blk);
    // 先放另一张 off_court 牌（模拟与枕流共存）
    let other_off = find_card(&e3, |k, _| k == CardKind::Jiu);
    e3.state_core_mut().players[1].off_court.push(other_off);
    let lm = e3.catalog().skill_by_name("lmazui").unwrap();
    let _ = submit(&mut e3, 0, GameAction::ActivateSkill { skill: lm });
    let c = e3.open_choice().unwrap();
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(&mut e3, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(m_blk)] });
    let c = e3.open_choice().unwrap();
    assert!(c.spec.options.contains(&ChoiceOption::Player(1)), "off_court 非空时只要未带 lmazui2 仍可选");
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(&mut e3, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    // 让 p1 对 p2 造成 1 点伤害 → lmazui2 减 1 归 0 触发 DamageZero → lmazui3 归还 m_blk，不动 other_off
    let hp2 = e3.debug_hp(2);
    let _ = e3.debug_deal_damage(Some(1), 2, 1, engine_native::event::DamageNature::Normal);
    assert_eq!(e3.debug_hp(2), hp2, "麻醉减 1 使 1 伤归零（DamageZero）");
    assert!(e3.debug_hand(1).contains(&m_blk), "lmazui3 在 DamageZero 后归还所挂黑牌");
    assert!(e3.state_core().players[1].off_court.contains(&other_off), "共存其他 off_court 牌不受波及");

    // 4) yuchen（排除自身+目标自选弃 he）、nlianji 平局保留 throw_card、hutian 选自己、pozhou2 多目标、huanxing
    let (_pack3, mut e4) = swd_engine(705_023);
    e4.debug_grant(1, "yuchen");
    e4.debug_grant(0, "hutian");
    e4.debug_grant(0, "nlianji");
    e4.debug_grant(0, "pozhou2");
    advance_to_play(&mut e4, 0);
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    let duel_0 = find_card(&e4, |k, _| k == CardKind::Duel);
    let blk_1 = find_card(&e4, |k, s| k == CardKind::Sha && matches!(s, Suit::Club));
    let drop_a = find_card(&e4, |k, _| k == CardKind::Tao);
    let drop_b = find_card(&e4, |k, _| k == CardKind::Jiu);
    move_card_to_hand(&mut e4, 0, duel_0);
    move_card_to_hand(&mut e4, 0, drop_a);
    move_card_to_hand(&mut e4, 0, drop_b);
    move_card_to_hand(&mut e4, 1, blk_1);
    let _ = submit(&mut e4, 0, GameAction::PlayCard { card: duel_0, targets: vec![1] });
    let rs = e4.open_choice().expect("决斗响应黑杀");
    let (cid, obs) = (rs.id, rs.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(blk_1)] });
    let tc = e4.open_choice().expect("p1 回合外打黑杀响应触发 yuchen 确认");
    assert_eq!((tc.observer, tc.spec.prompt), (1, "trigger_confirm"));
    let (cid, obs) = (tc.id, tc.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let yt = e4.open_choice().expect("yuchen_target");
    assert_eq!((yt.observer, yt.spec.prompt), (1, "yuchen_target"));
    assert!(!yt.spec.options.contains(&ChoiceOption::Player(1)), "浴尘不可选自己");
    let (cid, obs) = (yt.id, yt.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(0)] });
    let yd = e4.open_choice().expect("yuchen_discard 由技能持有者（p1）选择弃目标牌");
    assert_eq!((yd.observer, yd.spec.prompt), (1, "yuchen_discard"));
    let (cid, obs) = (yd.id, yd.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(drop_b)] });
    drain_pass_or_first(&mut e4);
    assert!(!e4.debug_hand(0).contains(&drop_b) && e4.debug_hand(0).contains(&drop_a));

    // nlianji 平局：throw_card 进弃牌堆（owner 失去 throw_card，无人受伤）
    let rank5_a = (0..e4.catalog().cards.len() as u32).find(|c| e4.catalog().cards[*c as usize].rank == 5).unwrap();
    let rank5_b = (rank5_a + 1..e4.catalog().cards.len() as u32).find(|c| e4.catalog().cards[*c as usize].rank == 5).unwrap();
    move_card_to_hand(&mut e4, 1, rank5_a);
    move_card_to_hand(&mut e4, 2, rank5_b);
    let nlj = e4.catalog().skill_by_name("nlianji").unwrap();
    let _ = submit(&mut e4, 0, GameAction::ActivateSkill { skill: nlj });
    for (p_expected, sel) in [
        ("nlianji_throw", vec![ChoiceOption::Card(drop_a)]),
        ("nlianji_target0", vec![ChoiceOption::Player(1)]),
        ("nlianji_target1", vec![ChoiceOption::Player(2)]),
        ("nlianji_t0_pick", vec![ChoiceOption::Card(rank5_a)]),
        ("nlianji_t1_pick", vec![ChoiceOption::Card(rank5_b)]),
    ] {
        let ch = e4.open_choice().unwrap();
        assert_eq!(ch.spec.prompt, p_expected);
        let (cid, obs) = (ch.id, ch.observer);
        let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: sel });
    }
    assert!(
        !e4.debug_hand(0).contains(&drop_a) && e4.state_core().discard.contains(&drop_a),
        "nlianji 平局将 throw_card 弃入弃牌堆且无人受伤"
    );

    // pozhou2 多目标（封印 p1 与 p2，扣 2 计数）
    e4.state_core_mut().players[0].pozhou_count = 2;
    let pz2 = e4.catalog().skill_by_name("pozhou2").unwrap();
    let _ = submit(&mut e4, 0, GameAction::ActivateSkill { skill: pz2 });
    let pc = e4.open_choice().unwrap();
    assert_eq!(pc.spec.prompt, "pozhou_target");
    assert_eq!(pc.spec.max, 2);
    let (cid, obs) = (pc.id, pc.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1), ChoiceOption::Player(2)] });
    assert_eq!(e4.state_core().player(0).pozhou_count, 0, "多目标破舟按目标人数扣除计数");
    let yc_def = e4.catalog().skill_by_name("yuchen").unwrap();
    assert!(!e4.state_core().skills.get(&(1, yc_def)).unwrap().enabled);

    // hutian 阶段结束选自己
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    move_card_to_hand(&mut e4, 0, drop_a);
    for _ in 0..4 {
        if e4.open_choice().is_some() {
            break;
        }
        let cur = e4.turn().current;
        let _ = submit(&mut e4, cur, GameAction::NextPhase);
    }
    let htc = e4.open_choice().expect("hutian trigger_confirm");
    assert_eq!((htc.observer, htc.spec.prompt), (0, "trigger_confirm"));
    let (cid, obs) = (htc.id, htc.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let ht = e4.open_choice().expect("hutian 选牌窗");
    assert_eq!(ht.spec.prompt, "hutian_cards");
    let (cid, obs) = (ht.id, ht.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(drop_a)] });
    let htt = e4.open_choice().expect("hutian_target");
    assert!(htt.spec.options.contains(&ChoiceOption::Player(0)), "护天必须允许选择自己");
    let (cid, obs) = (htt.id, htt.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(0)] });
}

/// `huanxing` / `huanxing2` (`swd/skill.js:3039`): unique/上轮同名禁止、精准回收授予及 pending 存读档。
#[test]
fn u05_b7c_huanxing_unique_prev_and_exact_remove() {
    let (pack, mut e) = swd_engine(705_025);
    e.debug_grant(0, "huanxing");
    let base = engine_native::content::standard::STANDARD_GENERALS.len() as u16;
    let total = base + pack.generals.len() as u16;
    let male_gen = (base..total)
        .find(|g| {
            let meta = e.catalog().general_meta(*g).unwrap();
            meta.4 == engine_native::content::standard::Gender::Male
                && meta.2.iter().any(|s| *s == "mingfu")
        })
        .unwrap_or_else(|| {
            (base..total)
                .find(|g| e.catalog().general_meta(*g).unwrap().4 == engine_native::content::standard::Gender::Male)
                .expect("男性扩展将")
        });
    let male_alt = (0..base)
        .find(|g| {
            let meta = e.catalog().general_meta(*g).unwrap();
            meta.4 == engine_native::content::standard::Gender::Male && !meta.2.is_empty()
        })
        .expect("男性标准将");
    e.state_core_mut().players[1].general_slots[0] = Some(male_gen);
    e.state_core_mut().players[2].general_slots[0] = Some(male_alt);

    // 预授 p0 一项 p1/p2 同名技能之一，确保 huanxing2 受伤时仅移除本次授予项
    let alt_skills = e.catalog().general_meta(male_alt).unwrap().2.clone();
    let keep_skill = alt_skills[0];
    e.debug_grant(0, keep_skill);

    advance_to_play(&mut e, 0);
    let mut found_hx = false;
    for _ in 0..48 {
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "huanxing_discard" {
                found_hx = true;
                break;
            }
            if c.observer == 0 && c.spec.options.contains(&ChoiceOption::Confirm) {
                let _ = submit(
                    &mut e,
                    0,
                    GameAction::Answer {
                        choice_id: c.id,
                        selection: vec![ChoiceOption::Confirm],
                    },
                );
                continue;
            }
            drain_pass_or_first(&mut e);
            continue;
        }
        if e.turn().current == 0 && e.turn().phase == engine_native::event::Phase::Play {
            e.debug_clear_hand(0);
            let k = find_card(&e, |ck, _| ck == CardKind::Tao);
            move_card_to_hand(&mut e, 0, k);
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    assert!(found_hx, "p0 回合开始应触发 huanxing_discard");
    let hx_c = e.open_choice().unwrap().spec.options[0];
    let (cid, obs) = {
        let c = e.open_choice().unwrap();
        (c.id, c.observer)
    };
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![hx_c] });
    let snap = e.save();
    let mut er = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复 huanxing_target");
    let ht = er.open_choice().expect("恢复 huanxing_target");
    assert_eq!(ht.spec.prompt, "huanxing_target");
    assert!(ht.spec.options.contains(&ChoiceOption::Player(1)));
    let (cid, obs) = (ht.id, ht.observer);
    let _ = submit(&mut er, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });

    let keep_def = er.catalog().skill_by_name(keep_skill).unwrap();
    let prev_before = er
        .state_core()
        .vm_aux
        .flags
        .get(&(0, "swd_huanxing_prev_gid".to_string()))
        .copied()
        .unwrap_or(0);
    assert_eq!(prev_before, u32::from(male_gen) + 1, "幻形在位期间必须记录本次复制武将编号");
    let _ = er.debug_deal_damage(None, 0, 1, engine_native::event::DamageNature::Normal);
    drain_pass_or_first(&mut er);
    assert!(er.state_core().has_skill(0, keep_def), "huanxing2 仅精准移除本次授予技能，保留原有同名技");
    assert!(!er.debug_has_skill(0, "mingfu"), "huanxing2 受伤回收复制的 mingfu");

    let prev_flag = er
        .state_core()
        .vm_aux
        .flags
        .get(&(0, "swd_huanxing_prev_gid".to_string()))
        .copied()
        .unwrap_or(0);
    assert_eq!(prev_flag, 0, "huanxing2 受伤回收后必须清除 swd_huanxing_prev_gid（swd/skill.js:3129）");
}
