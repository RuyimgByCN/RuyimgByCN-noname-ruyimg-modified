use super::common::*;
use engine_native::catalog::{CardKind, Suit};
use engine_native::event::Phase;
use engine_native::state::Role;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction};

/// `cyqiaoxie` (`swd/skill.js:5-55`): 失去手牌装备触发，主动 useCard 装备不触发。
#[test]
fn u05_b7c_cyqiaoxie_hand_equip_vs_use_guard() {
    let (_pack, mut e) = swd_engine(705_001);
    e.debug_grant(0, "cyqiaoxie");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);

    // 1) 正例：主动 PlayCard 使用装备（event.type === "use"）不应误触发 cyqiaoxie
    let armor = find_card(&e, |k, _| k == CardKind::Armor);
    move_card_to_hand(&mut e, 0, armor);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: armor,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "主动装备应直接结算: {out:?}");
    assert!(e.open_choice().is_none(), "主动 useCard 不得误触发此桥邪");

    // 2) 正例：失去手牌中的装备牌（非 use）应触发此桥邪
    let eq_hand = find_card(&e, |k, _| k == CardKind::HorsePlus);
    move_card_to_hand(&mut e, 0, eq_hand);
    let _ = e.debug_take_owned_card(0, eq_hand);
    let c = e.open_choice().expect("失去手牌装备应开此桥邪选机关牌窗");
    assert_eq!(c.spec.prompt, "cyqiaoxie_card");
    drain_pass_or_first(&mut e);
}

/// `touxi` / `touxi2` (`swd/skill.js:3924-3969`): 黑判雷伤+touxi2 屏蔽，红判对方公开信息弃 he + 存读档恢复。
#[test]
fn u05_b7c_touxi_black_red_cooldown_ai_and_restore() {
    let (pack, mut e) = swd_engine(705_002);
    e.debug_grant(1, "touxi");
    set_roles_lord_rebel(&mut e);
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) 红判：p1 无 touxi2，p0 结束阶段触发 touxi（可选确认→红判→开 p0 自选弃 p1 一张 he）；
    //    并在 touxi_discard 挂起时测 AI 公开信息选装/blind 手牌及 pending save->restore->Answer。
    let p1_eq = find_card(&e, |k, _| k == CardKind::Armor);
    equip_slot_card(&mut e, 1, p1_eq);
    let p1_hand = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 1, p1_hand);
    let red_judge = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Heart));
    push_top_deck(&mut e, red_judge);

    // 进弃牌 → 结束 → 回合末窗（TurnEndWindow）
    for _ in 0..4 {
        if e.open_choice().is_some() {
            break;
        }
        let cur = e.turn().current;
        let _ = submit(&mut e, cur, GameAction::NextPhase);
    }
    let confirm = e.open_choice().expect("touxi 可选确认窗");
    assert_eq!(confirm.observer, 1);
    let (cid, obs) = (confirm.id, confirm.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let td = e.open_choice().expect("红判后应开 touxi_discard 窗");
    assert_eq!((td.observer, td.spec.prompt), (0, "touxi_discard"));
    // AI 仅按公开信息选择（p1 装备区甲对 p0 为敌方公开牌），并验证 pending 存读档
    let snap = e.save();
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复 pending");
    assert_eq!(restored.open_choice().unwrap().spec.prompt, "touxi_discard");
    assert_eq!(ai_answer_once(&mut restored).as_deref(), Some("touxi_discard"));
    assert!(
        restored.debug_equips(1).is_empty(),
        "AI 公开信息优先弃除敌方公开防具"
    );

    // 2) 黑判：p2 持 touxi，p0 结束时触发→1 雷伤+挂 touxi2；p1 结束时因 touxi2 屏蔽不触发；p2 回合开始后 touxi2 自消
    let (_, mut e2) = swd_engine(705_003);
    e2.debug_grant(2, "touxi");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let black_judge = find_card(&e2, |k, s| k == CardKind::Sha && matches!(s, Suit::Spade));
    push_top_deck(&mut e2, black_judge);
    let hp0_before = e2.debug_hp(0);
    for _ in 0..4 {
        if e2.open_choice().is_some() {
            break;
        }
        let cur = e2.turn().current;
        let _ = submit(&mut e2, cur, GameAction::NextPhase);
    }
    let confirm = e2.open_choice().expect("touxi 黑判确认");
    assert_eq!(confirm.observer, 2);
    let (cid, obs) = (confirm.id, confirm.observer);
    let _ = submit(
        &mut e2,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    drain_pass_or_first(&mut e2);
    assert_eq!(e2.debug_hp(0), hp0_before - 1, "touxi 黑判造成 1 点雷电伤害");
    let tx2_def = e2.catalog().skill_by_name("touxi2").expect("touxi2");
    assert!(e2.state_core().has_skill(2, tx2_def), "黑判后获得 touxi2 冷却");
    // 推进经 p1 完整回合（不得再触发 touxi）直至 p2 回合出牌阶段（touxi2 已在 p2 TurnBegin 自消）
    for _ in 0..16 {
        assert!(
            e2.open_choice().is_none(),
            "p2 带 touxi2 冷却期间 p1 结束不得触发 touxi"
        );
        if e2.turn().current == 2 && e2.turn().phase == Phase::Play {
            break;
        }
        let c = e2.turn().current;
        let _ = submit(&mut e2, c, GameAction::NextPhase);
    }
    assert!(!e2.state_core().has_skill(2, tx2_def), "p2 回合开始后移除 touxi2");
}

/// `kbolan` / `kbolan2` (`swd/skill.js:672-719`): 摸牌 +1 并自选置顶 + pending 恢复。
#[test]
fn u05_b7c_kbolan2_top_of_deck_restore_and_ai() {
    let (pack, mut e) = swd_engine(705_004);
    e.debug_grant(0, "kbolan");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let wuzhong = find_card(&e, |k, _| k == CardKind::Wuzhong);
    let keep = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 0, wuzhong);
    move_card_to_hand(&mut e, 0, keep);

    // 1) 无中生有摸 2 → 弹出 kbolan_confirm 可选确认 → Confirm 后 kbolan 加 1（共摸 3），drawEnd 开 kbolan2_top
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    let kc = e.open_choice().expect("wuzhong 后开 kbolan_confirm");
    assert_eq!(kc.spec.prompt, "kbolan_confirm");
    let (kcid, kobs) = (kc.id, kc.observer);
    let _ = submit(
        &mut e,
        kobs,
        GameAction::Answer {
            choice_id: kcid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let c = e.open_choice().expect("确认 kbolan 后开 kbolan2_top");
    assert_eq!(c.spec.prompt, "kbolan2_top");
    let snap = e.save();
    let mut restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复 kbolan2_top");
    let ch = restored.open_choice().expect("恢复后 kbolan2_top 仍在");
    assert_eq!(ch.spec.prompt, "kbolan2_top");
    let cid = ch.id;
    let _ = submit(
        &mut restored,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Card(keep)],
        },
    );
    assert_eq!(restored.debug_hand(0).len(), 3, "原手 1 + 摸 3 - 置顶 1 = 3");
    assert_eq!(
        restored.state_core().deck.last().copied(),
        Some(keep),
        "自选牌置于牌堆顶（下次摸牌即摸回）"
    );

    // 2) AI 选低价值手牌置顶
    let _ = ai_answer_once;
    assert_eq!(e.open_choice().unwrap().spec.prompt, "kbolan2_top");
    assert_eq!(ai_answer_once(&mut e).as_deref(), Some("kbolan2_top"));
    assert_eq!(e.debug_hand(0).len(), 3);
}

/// `tiebi` (`swd/skill.js:847-873`): computed_distance (+1/-1 马)、h-only 过滤与拒绝取消。
#[test]
fn u05_b7c_tiebi_computed_distance_and_cancel() {
    let (_pack, mut e) = swd_engine(705_005);
    e.debug_grant(2, "tiebi");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) p2 到 p1 本为距离 1，给 p0 装 2 射程武器保持 p0 可杀 p1，同时给 p1 装备 +1 马 →
    //    computed_distance(p2, p1) == 2，不得触发铁壁
    let p0_wpn = weapon_with_range(&e, 2);
    equip_slot_card(&mut e, 0, p0_wpn);
    let horse_p = find_card(&e, |k, _| k == CardKind::HorsePlus);
    equip_slot_card(&mut e, 1, horse_p);
    let blk_hand = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Spade));
    move_card_to_hand(&mut e, 2, blk_hand);
    let sha = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Heart));
    move_card_to_hand(&mut e, 0, sha);
    let out1 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(!matches!(out1, CommandOutcome::Rejected(_)), "p0 装武器可杀 +1 马目标: {out1:?}");
    assert!(
        e.open_choice().map(|c| c.spec.prompt != "tiebi_discard").unwrap_or(true),
        "+1 马令 p2->p1 距离>1 时不应开 tiebi_discard"
    );
    drain_pass_or_first(&mut e);

    // 2) 给 p2 装备 -1 马 → computed_distance(p2, p1) 降回 1；p2 可以取消 tiebi_discard 而不加护甲
    let horse_m = find_card(&e, |k, _| k == CardKind::HorseMinus);
    equip_slot_card(&mut e, 2, horse_m);
    let sha2 = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Diamond));
    move_card_to_hand(&mut e, 0, sha2);
    // 重置本回合出杀计数再发一杀
    let mut snap = e.save();
    snap.core.turn.sha_used_play_phase = false;
    let mut e = Engine::restore_with_extensions(&snap, &[&_pack]).unwrap();
    let out2 = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(!matches!(out2, CommandOutcome::Rejected(_)), "p0 第二杀合法: {out2:?}");
    let c = e.open_choice().expect("恢复距离 1 后应开 tiebi_discard");
    assert_eq!(c.spec.prompt, "tiebi_discard");
    assert!(c.spec.allow_cancel);
    let (cid, obs) = (c.id, c.observer);
    let _ = submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: Vec::new(),
        },
    );
    assert_eq!(e.state_core().players[1].hujia, 0, "取消铁壁不加护甲");
    assert!(e.debug_hand(2).contains(&blk_hand), "取消铁壁保留黑牌");
}

/// `lingyue` (`swd/skill.js:1560`) & `linyun`/`linyun2` (`swd/skill.js:2854`): shaBegin 顺序、仁王盾与反悔/闪。
#[test]
fn u05_b7c_lingyue_renwang_and_linyun2_shan() {
    let (_pack, mut e) = swd_engine(705_006);
    e.debug_grant(0, "lingyue");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // 1) 目标持仁王盾时，黑杀先被仁王盾无效，不进入 lingyue
    let rw = find_card(&e, |k, _| k == CardKind::Renwang);
    equip_slot_card(&mut e, 1, rw);
    let t1 = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 1, t1);
    let bsha = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Club));
    move_card_to_hand(&mut e, 0, bsha);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: bsha, targets: vec![1] });
    assert!(e.open_choice().is_none(), "仁王盾先挡黑杀，不触发 lingyue");

    // 2) 红杀不被仁王盾挡：开 lingyue_confirm，确认后 p1 自选弃手或装备（选弃装备 rw）
    let rsha = find_card(&e, |k, s| k == CardKind::Sha && matches!(s, Suit::Heart));
    move_card_to_hand(&mut e, 0, rsha);
    let mut snap = e.save();
    snap.core.turn.sha_used_play_phase = false;
    let mut e = Engine::restore_with_extensions(&snap, &[&_pack]).unwrap();
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: rsha, targets: vec![1] });
    let lc = e.open_choice().expect("lingyue_confirm");
    assert_eq!(lc.spec.prompt, "lingyue_confirm");
    let (cid, obs) = (lc.id, lc.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Confirm] });
    let ld = e.open_choice().expect("lingyue_discard");
    assert_eq!((ld.observer, ld.spec.prompt), (1, "lingyue_discard"));
    let (cid, obs) = (ld.id, ld.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(rw)] });
    assert!(e.debug_equips(1).is_empty(), "lingyue 目标自选弃掉装备区仁王盾");

    // 3) linyun2：目标有闪时开 linyun2_shan；A 分支弃闪仍需正常结算（有第二闪可再闪），B 分支取消则 directHit
    let (_pack2, mut e2) = swd_engine(705_007);
    e2.debug_grant(0, "linyun");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let m1 = find_card(&e2, |k, _| k == CardKind::Tao);
    let m2 = find_card(&e2, |k, _| k == CardKind::Wuzhong);
    move_card_to_hand(&mut e2, 0, m1);
    move_card_to_hand(&mut e2, 0, m2);
    let p1_shan = find_card(&e2, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e2, 1, p1_shan);
    let hp1 = e2.debug_hp(1);
    let ly_def = e2.catalog().skill_by_name("linyun").expect("linyun");
    let _ = submit(&mut e2, 0, GameAction::ActivateSkill { skill: ly_def });
    let p_pick = e2.open_choice().unwrap();
    let (cid, obs) = (p_pick.id, p_pick.observer);
    let _ = submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(m1), ChoiceOption::Card(m2)] });
    let p_tgt = e2.open_choice().unwrap();
    let (cid, obs) = (p_tgt.id, p_tgt.observer);
    let _ = submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    let l2 = e2.open_choice().expect("目标有闪时应开 linyun2_shan");
    assert_eq!((l2.observer, l2.spec.prompt), (1, "linyun2_shan"));
    // 取消 linyun2_shan → directHit，不开常规 respond_shan 直接命中
    let (cid, obs) = (l2.id, l2.observer);
    let _ = submit(&mut e2, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert!(e2.open_choice().is_none(), "取消 linyun2_shan 后 directHit 跳过 respond_shan");
    assert_eq!(e2.debug_hp(1), hp1 - 1, "取消 linyun2_shan 直击命中");
    assert!(e2.debug_hand(1).contains(&p1_shan), "未打出的闪仍在目标手");
}

/// `lingxian` (`swd/skill.js:2671`) & `guaili` (`swd/skill.js:2600`) & `bingfeng2` & `yudun`: 边界校验。
#[test]
fn u05_b7c_lingxian_guaili_bingfeng2_yudun() {
    // 1) bingfeng2：只锁手牌，不锁装备区转化
    let (_pack, mut e) = swd_engine(705_008);
    e.debug_grant(0, "bingfeng2");
    e.debug_grant(0, "huanxia");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let h_sha = find_card(&e, |k, _| k == CardKind::Sha);
    move_card_to_hand(&mut e, 0, h_sha);
    let out_hand = submit(&mut e, 0, GameAction::PlayCard { card: h_sha, targets: vec![1] });
    assert!(matches!(out_hand, CommandOutcome::Rejected(_)), "bingfeng2 禁止使用手牌");
    let red_eq = find_card(&e, |k, s| k.equip_slot().is_some() && matches!(s, Suit::Heart | Suit::Diamond));
    equip_slot_card(&mut e, 0, red_eq);
    let out_eq = submit(&mut e, 0, GameAction::PlayCard { card: red_eq, targets: vec![1] });
    assert!(
        matches!(out_eq, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "bingfeng2 不禁装备区 huanxia 转化: {out_eq:?}"
    );
    drain_pass_or_first(&mut e);

    // 2) yudun 封印豁免：有 yudun 时禁打锦囊，被 tieji 封印后恢复可用
    let (_pack2, mut e2) = swd_engine(705_009);
    e2.debug_grant(0, "yudun");
    advance_to_play(&mut e2, 0);
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let wz = find_card(&e2, |k, _| k == CardKind::Wuzhong);
    move_card_to_hand(&mut e2, 0, wz);
    let out_ban = submit(&mut e2, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    assert!(matches!(out_ban, CommandOutcome::Rejected(_)), "yudun 未被封印时禁出锦囊");
    e2.state_core_mut().vm_aux.flags.insert((0, "sbtieji_target_sealed".to_string()), 1);
    let out_ok = submit(&mut e2, 0, GameAction::PlayCard { card: wz, targets: vec![] });
    assert!(
        matches!(out_ok, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "yudun 被封印后可用锦囊: {out_ok:?}"
    );
    drain_pass_or_first(&mut e2);

    // 3) guaili：+1 伤害后结算期手牌>2 时开 guaili2_discard 自选弃 2；且不触发于铁索传导伤害
    let (_pack3, mut e3) = swd_engine(705_010);
    e3.debug_grant(0, "guaili");
    advance_to_play(&mut e3, 0);
    for p in 0..4 {
        e3.debug_clear_hand(p);
    }
    let fsha = find_card(&e3, |k, _| k == CardKind::FireSha);
    move_card_to_hand(&mut e3, 0, fsha);
    let k1 = find_card(&e3, |k, _| k == CardKind::Tao);
    let k2 = find_card(&e3, |k, _| k == CardKind::Shan);
    let k3 = find_card(&e3, |k, _| k == CardKind::Jiu);
    move_card_to_hand(&mut e3, 0, k1);
    move_card_to_hand(&mut e3, 0, k2);
    move_card_to_hand(&mut e3, 0, k3);
    e3.state_core_mut().players[1].linked = true;
    e3.state_core_mut().players[2].linked = true;
    let hp1 = e3.debug_hp(1);
    let hp2 = e3.debug_hp(2);
    let _ = submit(&mut e3, 0, GameAction::PlayCard { card: fsha, targets: vec![1] });
    // 先跳过 p1 出闪窗
    let sh = e3.open_choice().unwrap();
    let (cid, obs) = (sh.id, sh.observer);
    let _ = submit(&mut e3, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Pass] });
    let g2 = e3.open_choice().expect("手牌>2 应开 guaili2_discard");
    assert_eq!(g2.spec.prompt, "guaili2_discard");
    assert_eq!((g2.spec.min, g2.spec.max), (2, 2));
    let (cid, obs) = (g2.id, g2.observer);
    let _ = submit(&mut e3, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(k1), ChoiceOption::Card(k3)] });
    drain_pass_or_first(&mut e3);
    assert_eq!(e3.debug_hp(1), hp1 - 2, "主目标受乖戾 +1 = 2 点火伤");
    assert_eq!(e3.debug_hp(2), hp2 - 2, "铁索传导 2 伤但不应再触发乖戾加成（非 3）");
    assert_eq!(e3.debug_hand(0), vec![k2], "guaili2 精准弃掉所选两张手牌");

    // 4) lingxian：取消不摸牌，AI 优先选公开友军（p1 忠臣 → p3 公开主公，距离 2 > 1）
    let (_pack4, mut e4) = swd_engine(705_011);
    e4.debug_grant(1, "lingxian");
    e4.state_core_mut().mode = engine_native::state::GameMode::Identity {
        roles: vec![Role::Rebel, Role::Loyalist, Role::Rebel, Role::Lord],
    };
    advance_to_play(&mut e4, 0);
    for p in 0..4 {
        e4.debug_clear_hand(p);
    }
    let sha_a = find_card(&e4, |k, s| k == CardKind::Sha && matches!(s, Suit::Spade));
    let shan_1 = find_card(&e4, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e4, 0, sha_a);
    move_card_to_hand(&mut e4, 1, shan_1);
    let _ = submit(&mut e4, 0, GameAction::PlayCard { card: sha_a, targets: vec![1] });
    let r = e4.open_choice().unwrap();
    let (cid, obs) = (r.id, r.observer);
    let _ = submit(&mut e4, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(shan_1)] });
    let lx = e4.open_choice().expect("lingxian_target");
    assert_eq!(lx.spec.prompt, "lingxian_target");
    assert_eq!(ai_answer_once(&mut e4).as_deref(), Some("lingxian_target"));
    assert_eq!(e4.debug_hand(3).len(), 1, "AI (Loyalist p1) 应选公开友军 Lord p3 摸 1");
    assert_eq!(e4.debug_hand(1).len(), 1, "p1 自身摸 1");
}

/// `yeying` / `guiying` / `fengze` / `huanxia` / `poxiao` / `yaotong`:
/// `PlayCard` 不得劫持自然可打出的黑色顺/杀/酒/装备；须经 `ActivateSkill` 主动转化且限次与取消返还正确。
#[test]
fn u05_b7c_play_rules_viewas_and_no_hijack() {
    let (_pack, mut e) = swd_engine(705_012);
    e.debug_grant(0, "yeying");
    e.debug_grant(0, "guiying");
    e.debug_grant(0, "fengze");
    e.debug_grant(0, "huanxia");
    e.debug_grant(0, "yaotong");
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(0, 3);

    // 1) 无目标自用牌（桃/酒）、黑色装备与黑色顺手牵羊经 PlayCard 均按本牌结算（不被 yeying/guiying/fengze 劫持）
    let tao = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 0, tao);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
    drain_pass_or_first(&mut e);
    assert_eq!(e.debug_hp(0), 4, "无目标自用桃不被转化劫持");

    let blk_eq = find_card(&e, |k, s| k.equip_slot().is_some() && matches!(s, Suit::Spade | Suit::Club));
    move_card_to_hand(&mut e, 0, blk_eq);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: blk_eq, targets: vec![] });
    drain_pass_or_first(&mut e);
    assert!(e.debug_equips(0).contains(&blk_eq), "手牌黑色装备入槽而不被 fengze 劫持");

    let blk_jiu = find_card(&e, |k, s| k == CardKind::Jiu && matches!(s, Suit::Spade | Suit::Club));
    move_card_to_hand(&mut e, 0, blk_jiu);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: blk_jiu, targets: vec![] });
    drain_pass_or_first(&mut e);
    assert!(e.state_core().turn.jiu_used, "黑酒走 PlayCard 按酒结算而不被 fengze 劫持");

    let p1_c = find_card(&e, |k, _| k == CardKind::Shan);
    move_card_to_hand(&mut e, 1, p1_c);
    let b_snatch = find_card(&e, |k, s| k == CardKind::Snatch && matches!(s, Suit::Spade | Suit::Club));
    move_card_to_hand(&mut e, 0, b_snatch);
    let _ = submit(&mut e, 0, GameAction::PlayCard { card: b_snatch, targets: vec![1] });
    drain_pass_or_first(&mut e);
    assert!(e.debug_hand(0).contains(&p1_c), "黑色顺手牵羊按顺手牵羊偷牌，不被 yeying/guiying 劫持");
    let yeying_def = e.catalog().skill_by_name("yeying").expect("yeying");
    let fengze_def = e.catalog().skill_by_name("fengze").expect("fengze");
    assert!(!e.state_core().skills.get(&(0, yeying_def)).unwrap().once_used, "未发动夜影不得置 once_used");
    assert!(!e.state_core().skills.get(&(0, fengze_def)).unwrap().once_used, "未发动凤泽不得置 once_used");

    // 2) 通过 ActivateSkill 显式发动 yeying（以装备区黑装当乾坤镖）与取消返还，及再次发动被拒
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: yeying_def });
    let ch = e.open_choice().expect("yeying_pick");
    assert_eq!(ch.spec.prompt, "yeying_pick");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: Vec::new() });
    assert!(!e.state_core().skills.get(&(0, yeying_def)).unwrap().once_used, "夜影取消必退 once_used");
    let p1_keep = find_card(&e, |k, _| k == CardKind::Tao);
    move_card_to_hand(&mut e, 1, p1_keep);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: yeying_def });
    let ch = e.open_choice().expect("yeying_pick 2");
    let (cid, obs) = (ch.id, ch.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Card(blk_eq)] });
    let tgt = e.open_choice().expect("yeying_target");
    let (cid, obs) = (tgt.id, tgt.observer);
    let _ = submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: vec![ChoiceOption::Player(1)] });
    drain_pass_or_first(&mut e);
    assert!(e.state_core().skills.get(&(0, yeying_def)).unwrap().once_used, "夜影发动后置 once_used = true");
    assert!(!e.debug_equips(0).contains(&blk_eq), "黑装备已转化为乾坤镖消耗");
    assert!(!e.debug_hand(1).contains(&p1_keep), "夜影乾坤镖命中 p1 弃牌");
    let out2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: yeying_def });
    assert!(matches!(out2, CommandOutcome::Rejected(_)), "夜影每回合一次再发动拒: {out2:?}");
}
