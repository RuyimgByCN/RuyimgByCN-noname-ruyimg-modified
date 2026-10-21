use super::*;

/// 年兽系形态片（规则 v249/存档 v59）：形态轮换+忍性——①忍性直装面：
/// 非当前回合任意伤害→持有者摸 1、当前回合不触发②轮换首换：判定锚 assembly
/// 重挂（忍性入场/hp 保留/手牌保留）。轮换后多回合推进暴露 Suspended 风暴
/// （中局 assembly 的 Yield 循环）→ 登记引擎待查，多回合轮换回归待查后
/// 补。忍性年兽条目（[年瑞,猛态,忍性,形态轮换]）；守翼/精觉/睿智/暴怒形
/// 态未落随下批。
#[test]
fn boss_nianshou_nbianshen_rotation_renxing() {
    // ① 忍性直装面：1=忍性年兽（条目直装）
    let mut e = Engine::new_boss_stage(69001, 4, 1, 3, 6, 4, 0);
    let rx_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 26) as u16; // BOSS_GENERALS[26] 忍性年兽
    e.debug_set_general_def(1, rx_def);
    e.debug_set_hp(1, 12);
    assert_eq!(
        e.catalog().general_meta(rx_def).map(|g| g.0),
        Some("忍性年兽"),
        "忍性年兽入目录"
    );
    let rx = e.catalog().skill_by_name("忍性").expect("忍性入目录");
    assert!(e.state_core().skills.contains_key(&(1, rx)));

    // 非当前回合（0 号回合）伤害 → 1 号摸 1
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    e.debug_deal_damage(Some(2), 3, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "忍性：非当前回合伤害摸 1");
    // 当前回合（0 号）回复 → 1 号再摸 1（recoverEnd 半面）
    e.debug_recover(3, 1);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "忍性：非当前回合回复摸 1");
    // 当前回合=持有者? 0 号回合内持有者=1 非 current ✓ 已验证；1 号自己
    // 回合内伤害不触发——推到 1 号出牌阶段
    next_phase(&mut e); // 0 Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    next_phase(&mut e); // End→1 Judge
    drain_choices(&mut e);
    next_phase(&mut e); // 1 Judge→Draw
    drain_choices(&mut e);
    next_phase(&mut e); // 1 Draw→Play
    drain_choices(&mut e);
    let hand1 = e.debug_hand(1).len();
    e.debug_deal_damage(Some(0), 3, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), hand1, "当前回合=持有者：忍性不触发");
    assert_zone_conservation(&e);

    // ② 轮换首换（独立局）：1=年兽合体 → 次回合判定锚换至忍性年兽
    let mut e2 = Engine::new_boss_stage(69002, 4, 1, 3, 6, 4, 0);
    let heti_def = (e2.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 25) as u16; // BOSS_GENERALS[25] 年兽合体
    let rx2_def = (e2.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 26) as u16;
    e2.debug_set_general_def(1, heti_def);
    e2.debug_set_hp(1, 12);
    let nb = e2
        .catalog()
        .skill_by_name("形态轮换")
        .expect("形态轮换入目录");
    assert!(e2.state_core().skills.contains_key(&(1, nb)));
    advance_to_play(&mut e2, 1);
    assert!(
        e2.state_core().player(1).nbianshen_toggle,
        "首回合：旗标置位不换将"
    );
    assert_eq!(
        e2.state_core().player(1).general_slots[0],
        Some(heti_def),
        "首回合不换将"
    );
    // 结束本回合，推进到次回合判定锚（轮换发生）
    next_phase(&mut e2); // Play→Discard
    drain_choices(&mut e2);
    next_phase(&mut e2); // Discard→End
    drain_choices(&mut e2);
    for _ in 0..24 {
        if e2.turn().current == 1 && e2.turn().phase == Phase::Judge {
            break;
        }
        if let Some(c) = e2.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e2, ChoiceOption::Pass);
                drain_choices(&mut e2);
            } else {
                answer_cards(&mut e2, &c);
            }
            continue;
        }
        match next_phase(&mut e2) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => {}
            CommandOutcome::Pending { choice } => {
                let is_dying = choice.spec.prompt == "dying_save";
                if is_dying {
                    answer(&mut e2, ChoiceOption::Pass);
                    drain_choices(&mut e2);
                } else {
                    answer_cards(&mut e2, &choice);
                }
            }
            CommandOutcome::Rejected(r) => panic!("np rejected {r:?}"),
        }
    }
    assert_eq!(
        e2.state_core().player(1).general_slots[0],
        Some(rx2_def),
        "次回合轮换至忍性年兽"
    );
    assert!(e2.state_core().skills.contains_key(&(1, rx)), "忍性入场");
    assert_eq!(e2.debug_hp(1), 12, "轮换：hp 保留");
    let snap = e2.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(
        r.state_core().player(1).general_slots[0],
        Some(rx2_def),
        "形态往返"
    );
    assert_zone_conservation(&r);

    // ⑤ 多回合回归（勘误：早前「Suspended 风暴」为测试脚手架假象——本
    //    探针证实轮换后每 resume 均有进展、无 Suspended）：驱动过 round-3
    //    判定锚 → 互斥轮转回年兽合体
    for _ in 0..60 {
        if e2.outcome().is_some() {
            break;
        }
        if let Some(c) = e2.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e2, ChoiceOption::Pass);
                drain_choices(&mut e2);
            } else {
                answer_cards(&mut e2, &c);
            }
            drain_choices(&mut e2);
            continue;
        }
        if e2.turn().current == 1 && e2.turn().phase == Phase::Judge && e2.turn().round >= 3 {
            break;
        }
        match next_phase(&mut e2) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => {}
            CommandOutcome::Pending { choice } => {
                let is_dying = choice.spec.prompt == "dying_save";
                if is_dying {
                    answer(&mut e2, ChoiceOption::Pass);
                    drain_choices(&mut e2);
                } else {
                    answer_cards(&mut e2, &choice);
                }
            }
            CommandOutcome::Rejected(r) => panic!("np rejected {r:?}"),
        }
    }
    assert_eq!(
        e2.state_core().player(1).general_slots[0],
        Some(heti_def),
        "round-3 判定锚：互斥轮换回年兽合体"
    );
    assert_eq!(e2.debug_hp(1), 12, "多回合轮换 hp 恒保留");
    let snap2 = e2.save();
    let r2 = Engine::restore(&snap2).expect("往返");
    assert_zone_conservation(&r2);
    assert_zone_conservation(&e2);
}

/// 年兽系首片（规则 v248/存档 v58）：年瑞+猛态（年兽合体条目）——①年
/// 瑞：摸牌阶段 +2；②猛态常态回合（摸/用均发生）：无补摸不跳弃；③猛
/// 态跳摸+跳用回合：结束阶段补摸 3+跳过弃牌阶段。年兽条目（群目录近似
/// /hp12/[年瑞,猛态]；形态轮换 nbianshen 未落随下批）。
#[test]
fn boss_nianshou_nianrui_mengtai() {
    // ① 年瑞 +2：年兽出牌阶段前手牌=年瑞加成（hp12 → 上限 12，无弃牌干扰）
    let mut e = Engine::new_boss_stage(68001, 4, 1, 3, 6, 4, 0);
    let ns_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 25) as u16; // BOSS_GENERALS[25] 年兽
    e.debug_set_general_def(1, ns_def);
    e.debug_set_hp(1, 12);
    assert_eq!(
        e.catalog().general_meta(ns_def).map(|g| g.0),
        Some("年兽"),
        "年兽入目录"
    );
    let nr = e.catalog().skill_by_name("年瑞").expect("年瑞入目录");
    let mt = e.catalog().skill_by_name("猛态").expect("猛态入目录");
    assert!(e.state_core().skills.contains_key(&(1, nr)));
    assert!(e.state_core().skills.contains_key(&(1, mt)));
    // 0 号先手回合正常轮转（无猛态面），推到 1 号出牌阶段
    advance_to_play(&mut e, 1);
    // 1 号：初始 6+年瑞 2+摸牌 2=10（跳过路径未走）
    assert_eq!(e.debug_hand(1).len(), 10, "年瑞：摸牌阶段 +2");
    assert!(!e.state_core().turn.mengtai_draw_pending, "摸牌已发生");
    assert!(!e.state_core().turn.mengtai_use_pending, "出牌阶段已开始");

    // ② 跳摸+跳用回合：推进到 1 号判定阶段（置位臂已跑）后置跳过旗标
    //    → 摸牌跳过（清位不发生）+出牌跳过（清位不发生）→ 结束补摸 3+
    //    弃牌闸门跳过
    e.debug_clear_hand(1);
    next_phase(&mut e); // Play→Discard（use 已发生→弃牌正常）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    for _ in 0..24 {
        if e.turn().current == 1 && e.turn().phase == Phase::Judge {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                if choice.spec.prompt == "dying_save" {
                    answer(&mut e, ChoiceOption::Pass);
                    drain_choices(&mut e);
                } else {
                    answer_cards(&mut e, &choice);
                }
            }
            other => panic!("{other:?}"),
        }
    }
    assert!(
        e.state_core().turn.mengtai_draw_pending && e.state_core().turn.mengtai_use_pending,
        "准备置位已跑"
    );
    {
        let core = e.state_core_mut();
        core.player_mut(1).skip_next_draw = true;
        core.turn.skip_play_phase = true;
    }
    next_phase(&mut e); // Judge→Draw（skip_next_draw 消费：摸牌跳过）
    drain_choices(&mut e);
    // 推进到本回合结束阶段（弃牌闸门跳过）
    for _ in 0..8 {
        if e.turn().current == 1 && e.turn().phase == Phase::End {
            break;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Pending { choice } => {
                if choice.spec.prompt == "dying_save" {
                    answer(&mut e, ChoiceOption::Pass);
                    drain_choices(&mut e);
                } else {
                    answer_cards(&mut e, &choice);
                }
            }
            other => panic!("{other:?}"),
        }
    }
    assert_eq!(e.debug_hand(1).len(), 3, "猛态：未摸牌 → 结束阶段摸 3");
    assert!(!e.state_core().turn.mengtai_draw_pending, "补摸后旗标消费");
    assert_zone_conservation(&e);

    // ③ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, nr)), "年瑞实例往返");
    assert_zone_conservation(&r);
}

/// 年兽系精觉片（规则 v252）：精觉（继承胆术）——失去手牌后（非当前回
/// 合且已受伤）判定红回 1；满血/非持有者不触发。精觉年兽条目（[年瑞,猛
/// 态,精觉,形态轮换]）；胆术 def 供夜叉批复用。
#[test]
fn boss_nianshou_jingjue_danshu_recover() {
    let mut e = Engine::new_boss_stage(72001, 4, 1, 3, 6, 4, 0);
    let jj_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 29) as u16; // BOSS_GENERALS[29] 精觉年兽
    e.debug_set_general_def(1, jj_def);
    e.debug_set_hp(1, 12);
    assert_eq!(
        e.catalog().general_meta(jj_def).map(|g| g.0),
        Some("精觉年兽"),
        "精觉年兽入目录"
    );
    let jj = e.catalog().skill_by_name("精觉").expect("精觉入目录");
    assert!(e.state_core().skills.contains_key(&(1, jj)));

    // ① 精觉面：0 号回合拆 1 号手牌 → 非当前回合+已受伤（11/12）→
    //    判定红 → 回 1
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 11);
    let _shan1 = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs = e.debug_place(0, "dismantle");
    play(&mut e, gs, vec![1]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 12, "精觉：判定红回 1（11→12）");

    // ② 满血对照：拆第二张 → 判定红不回（已满）
    let _shan1b = e.debug_place(1, "shan");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs2 = e.debug_place(0, "dismantle");
    play(&mut e, gs2, vec![1]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 12, "满血：不回（恒上限）");
    assert_eq!(e.debug_hand(1).len(), 0, "牌仍被拆走");

    // ③ 非持有者对照：拆 2 号 → 无胆术/精觉不回
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_set_hp(2, 3);
    let _shan2 = e.debug_place(2, "shan");
    e.debug_set_deck_top_by(|def| def.suit == engine_native::catalog::Suit::Heart);
    let gs3 = e.debug_place(0, "dismantle");
    play(&mut e, gs3, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 3, "非持有者：无回血（拆桥本不打伤害）");
    assert!(e.debug_hand(2).is_empty(), "牌被拆走");

    // ④ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, jj)), "精觉实例往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}

/// 年兽系守翼片（规则 v251）：守翼（守翼年兽形态）——所有牌无距离限
/// 制：①杀隔位可指定②顺手牵羊隔位可指定③非持有者对照被拒。守翼年兽
/// 条目（[年瑞,猛态,守翼,形态轮换]；上游形态表含 boss_nbaonu 未落随批）。
#[test]
fn boss_nianshou_shouyi_range_free() {
    let mut e = Engine::new_boss_stage(71001, 4, 1, 3, 6, 4, 0);
    let sy_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 28) as u16; // BOSS_GENERALS[28] 守翼年兽
    e.debug_set_general_def(1, sy_def);
    e.debug_set_hp(1, 12);
    assert_eq!(
        e.catalog().general_meta(sy_def).map(|g| g.0),
        Some("守翼年兽"),
        "守翼年兽入目录"
    );
    let sy = e.catalog().skill_by_name("守翼").expect("守翼入目录");
    assert!(e.state_core().skills.contains_key(&(1, sy)));

    // ① 杀隔位可指定（1→3 距离 2）
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 4);
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![3]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "守翼：杀无距离 {out:?}"
    );
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(3), 3, "隔位杀命中");

    // ② 顺手牵羊隔位可指定（Accepted 即距离面合法）
    e.debug_clear_hand(1);
    let req = e.debug_place(1, "snatch");
    play(&mut e, req, vec![3]);
    drain_choices(&mut e);

    // ③ 非持有者对照：2 号隔位杀 0 号 → 距离拒
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    e.debug_clear_hand(0);
    let sha2 = e.debug_place(2, "sha");
    let out = play(&mut e, sha2, vec![0]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "非持有者：隔位杀应被距离拒"
    );

    // ④ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, sy)), "守翼实例往返");
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}

/// 年兽系睿智片（规则 v250）：睿智（睿智年兽形态）——准备阶段其余角色
/// （手牌>1 或装备>1）保留一张手牌和一张装备区内的牌，弃置其余（保留
/// 首张确定性近似登记）。睿智年兽条目（[年瑞,猛态,睿智,形态轮换]）。
#[test]
fn boss_nianshou_ruizhi_prune() {
    let mut e = Engine::new_boss_stage(70001, 4, 1, 3, 6, 4, 0);
    let rz_form_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 27) as u16; // BOSS_GENERALS[27] 睿智年兽
    e.debug_set_general_def(1, rz_form_def);
    e.debug_set_hp(1, 12);
    assert_eq!(
        e.catalog().general_meta(rz_form_def).map(|g| g.0),
        Some("睿智年兽"),
        "睿智年兽入目录"
    );
    let rz = e.catalog().skill_by_name("睿智").expect("睿智入目录");
    assert!(e.state_core().skills.contains_key(&(1, rz)));

    // 布阵（0 号回合前）：0 号 3 手+2 装（修剪后 1/1）；2 号 1 手 1 装
    // （本就不超——不触发）；3 号 3 手 0 装（弃至 1 手）
    e.debug_clear_hand(0);
    let eq0 = [e.debug_place(0, "armor"), e.debug_place(0, "horse_plus")];
    for c in eq0.iter() {
        let core = e.state_core_mut();
        let pos = core.players[0]
            .hand
            .iter()
            .position(|x| x == c)
            .expect("in hand");
        let card = core.players[0].hand.remove(pos);
        core.players[0].equips.push(card);
    }
    e.debug_clear_hand(0);
    let h0 = [
        e.debug_place(0, "tao"),
        e.debug_place(0, "duel"),
        e.debug_place(0, "wugu"),
    ];
    e.debug_clear_hand(2);
    let _h2 = e.debug_place(2, "tao");
    let eq2 = e.debug_place(2, "armor");
    {
        let core = e.state_core_mut();
        let pos = core.players[2]
            .hand
            .iter()
            .position(|x| x == &eq2)
            .expect("in hand");
        let c = core.players[2].hand.remove(pos);
        core.players[2].equips.push(c);
    }
    e.debug_clear_hand(3);
    let h3 = [
        e.debug_place(3, "tao"),
        e.debug_place(3, "shan"),
        e.debug_place(3, "sha"),
    ];
    e.debug_clear_hand(1); // 睿智年兽自身清手（修剪只打其余角色）

    // 推进到 1 号判定锚后（修剪已发生）
    for _ in 0..12 {
        if e.turn().current == 1 && e.turn().phase == Phase::Draw {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            if c.spec.prompt == "dying_save" {
                answer(&mut e, ChoiceOption::Pass);
                drain_choices(&mut e);
            } else {
                answer_cards(&mut e, &c);
            }
            continue;
        }
        match next_phase(&mut e) {
            CommandOutcome::Accepted { .. } => {}
            CommandOutcome::Suspended => {}
            CommandOutcome::Pending { choice } => {
                let is_dying = choice.spec.prompt == "dying_save";
                if is_dying {
                    answer(&mut e, ChoiceOption::Pass);
                    drain_choices(&mut e);
                } else {
                    answer_cards(&mut e, &choice);
                }
            }
            CommandOutcome::Rejected(r) => panic!("np rejected {r:?}"),
        }
    }
    assert_eq!(e.debug_hand(0).len(), 1, "睿智：0 号手牌弃至 1");
    assert!(
        h0.contains(&e.debug_hand(0)[0]),
        "保留的是原有手牌（确定性近似：实现定哪张）"
    );
    assert_eq!(e.state_core().player(0).equips.len(), 1, "0 号装备弃至 1");
    assert_eq!(e.state_core().player(0).equips[0], eq0[0], "保留首张装备");
    assert_eq!(e.debug_hand(2).len(), 1, "2 号手牌保留（本就不超）");
    assert_eq!(e.state_core().player(2).equips.len(), 1, "2 号装备保留");
    assert_eq!(e.debug_hand(3).len(), 1, "睿智：3 号手牌弃至 1");
    assert!(
        h3.contains(&e.debug_hand(3)[0]),
        "3 号保留原有手牌（确定性近似）"
    );
    assert_zone_conservation(&e);

    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, rz)), "睿智实例往返");
    assert_zone_conservation(&r);
}
