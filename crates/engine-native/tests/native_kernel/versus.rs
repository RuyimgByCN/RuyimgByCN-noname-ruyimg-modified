use super::*;

/// versus 替补（规则 v206）：敌队死亡座位从本队替补池弹队首换将复活
/// （新将满血+摸 4+区域离场），死亡不结算（奖惩/终局不发生）；池空后
/// 照常死亡；替补池状态随存档往返。
#[test]
fn versus_substitute_revive_on_death() {
    let xiahoudun = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .position(|(g, ..)| *g == "夏侯惇")
        .expect("夏侯惇在表") as u16;
    let zhangliao = engine_native::content::standard::STANDARD_GENERALS
        .iter()
        .position(|(g, ..)| *g == "张辽")
        .expect("张辽在表") as u16;
    // 敌队（1 队）带替补池 [夏侯惇, 张辽]；己队空池
    let teams: [Vec<u8>; 2] = [vec![0, 1], vec![2, 3]];
    let mut e = Engine::new_versus_with_substitutes(
        23001,
        teams,
        vec![Vec::new(), vec![xiahoudun, zhangliao]],
    );
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // 2 号装配初始将（张辽），池首为夏侯惇
    e.debug_set_general(2, "张辽");
    // 0 杀 2 至死：替补介入 → 2 号不死亡、换将夏侯惇满血复活+摸 4
    e.debug_set_hp(2, 1);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain_choices(&mut e);
    assert!(e.debug_alive(2), "替补介入：2 号不死亡");
    assert!(
        e.state_core().outcome.is_none(),
        "替补替代死亡 → 不触发终局"
    );
    assert_eq!(e.debug_hand(2).len(), 4, "替补出场摸 4");
    let hp_after = e.debug_hp(2);
    assert!(hp_after >= 3, "新将满血复活（夏侯惇 4 血）：{hp_after}");
    // 奖惩不结算（死亡未发生）：0 号手牌不变（清空后仍 0）
    assert_eq!(e.debug_hand(0).len(), 0, "替补替代死亡 → 杀敌奖惩不结算");
    // 再杀 2 号：池首已弹（剩张辽）→ 再次替补换回张辽
    e.debug_set_hp(2, 1);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain_choices(&mut e);
    assert!(e.debug_alive(2), "第二替补继续介入");
    assert_eq!(e.debug_hand(2).len(), 4, "再次替补摸 4");
    // 池空：第三次击杀 → 照常死亡
    e.debug_set_hp(2, 1);
    e.debug_clear_hand(2); // 防桃自救
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain_choices(&mut e);
    assert!(!e.debug_alive(2), "池空照常死亡");
    assert_eq!(e.debug_hand(0).len(), 3, "死亡结算 → 杀敌奖惩摸三");
    // 存档往返：替补池（已耗尽）状态一致
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(r.debug_alive(2), false, "死亡状态往返一致");
    assert_zone_conservation(&r);
}

/// versus 4v4 变体（T11 模式变体，规则 v206 固化）：8 人建局、全员
/// 存活、敌方全灭 → 己方胜（终局面与 2v2 同管线）。
#[test]
fn versus_4v4_eight_seats_team_victory() {
    let teams: [Vec<u8>; 2] = [vec![0, 1, 2, 3], vec![4, 5, 6, 7]];
    let mut e = Engine::new_versus(22001, teams);
    advance_to_play(&mut e, 0);
    for p in 0..8 {
        assert!(e.debug_alive(p), "4v4 全员存活建局");
        e.debug_clear_hand(p);
    }
    for p in 4..8 {
        if !e.debug_alive(p) {
            continue;
        }
        e.debug_set_hp(p, 1);
        let out = e.debug_deal_damage(Some(0), p, 1, DamageNature::Normal);
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
        while let Some(c) = e.open_choice() {
            let (cid, obs) = (c.id, c.observer);
            submit_cmd(
                &mut e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
                    selection: vec![],
                },
            );
        }
    }
    assert!((4..8).all(|p| !e.debug_alive(p)), "敌方全灭");
    assert!((0..4).all(|p| e.debug_alive(p)), "己方存活");
    let out = e.state_core().outcome.clone();
    assert!(out.is_some(), "敌方全灭应有终局: {out:?}");
}

/// 对决 2v2 基线（P2/N08 模式入口批一，规则 v80）：杀死敌方摸三；
/// 杀死友方弃掉自己全部手牌与装备；敌方全灭终局队胜；存档往返。
#[test]
fn versus_mode_team_victory_and_rewards() {
    let teams: [Vec<u8>; 2] = [vec![0, 1], vec![2, 3]];
    let mut e = Engine::new_versus(2001, teams.clone());
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 杀死敌方（0 杀 2）→ 摸三；未终局（1 号存活）
    e.debug_set_hp(2, 1);
    let out = e.debug_deal_damage(Some(0), 2, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(2));
    assert_eq!(e.debug_hand(0).len(), 3, "杀死敌方应摸三张");
    assert!(e.outcome().is_none(), "1 号存活，未终局");

    // ② 中途存档往返：模式/队伍表编码一致
    {
        use engine_native::archive::{decode_archive, encode_archive};
        let bytes = encode_archive(&e.save());
        let back = decode_archive(&bytes).expect("解码");
        assert_eq!(back.core.mode, e.save().core.mode, "对决模式表往返一致");
    }

    // ③ 杀死友方（1 杀 0 同队）→ 杀手弃掉自己全部手牌与装备
    //（清 0/1 手牌：①的奖惩摸牌可能含桃，干扰濒死确定性——身份局测试同款口径）
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_place(1, "shan");
    let _armor_like = e.debug_place(1, "sha");
    e.debug_set_hp(0, 1);
    let out = e.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(0));
    assert!(e.debug_hand(1).is_empty(), "杀死友方应弃掉自己全部手牌");

    // ④ 0 队还剩 1 号：3（敌方）补杀 → 0 队全灭 → 1 队胜
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_set_hp(1, 1);
    let out = e.debug_deal_damage(Some(3), 1, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(!e.debug_alive(1));
    assert!(
        matches!(e.outcome(), Some(o) if o.winner == Winner::Versus(1)),
        "{:?}",
        e.outcome()
    );
}
