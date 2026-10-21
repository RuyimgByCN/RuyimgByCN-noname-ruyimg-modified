use super::*;

/// 星宿阶段2系绝红片（规则 v244）：绝红（共工）——准备阶段敌方全体逐
/// 个：有装备 → 装备全弃（撤技能+入弃堆）；无装备有手牌 → 随机弃一张
/// （首张确定性近似登记）；无牌者不触发。共工条目（群目录近似/hp5/[神
/// 裔,绝红]；玄麟3 已处置关卡面）。
#[test]
fn boss_shuishengonggong_juehong_strip_enemies() {
    let mut e = Engine::new_boss_stage(64001, 4, 1, 3, 6, 4, 0);
    let gg_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 21) as u16; // BOSS_GENERALS[21] 共工
    e.debug_set_general_def(1, gg_def);
    e.debug_set_hp(1, 5);
    assert_eq!(
        e.catalog().general_meta(gg_def).map(|g| g.0),
        Some("共工"),
        "共工入目录"
    );
    let jdef = e.catalog().skill_by_name("绝红").expect("绝红入目录");
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, jdef)));
    assert!(e.state_core().skills.contains_key(&(1, sdef)), "神裔共用");

    // 布阵（0 号回合前）：0 号带装备、2 号仅手牌、3 号空牌
    let armor = e.debug_place(0, "armor");
    {
        let core = e.state_core_mut();
        let pos = core.players[0]
            .hand
            .iter()
            .position(|c| *c == armor)
            .expect("armor in hand");
        let c = core.players[0].hand.remove(pos);
        core.players[0].equips.push(c);
    }
    e.debug_clear_hand(0);
    e.debug_clear_hand(3);
    let h2_before = e.debug_hand(2).clone();
    assert!(!h2_before.is_empty(), "2 号应有手牌");

    // 推进到 1 号（共工）出牌阶段：其准备阶段（判定锚）触发绝红
    advance_to_play(&mut e, 1);
    assert!(
        e.state_core().player(0).equips.is_empty(),
        "绝红：0 号装备全弃"
    );
    assert_eq!(
        e.debug_hand(2).len(),
        h2_before.len() - 1,
        "绝红：2 号弃一张手牌"
    );
    assert_eq!(e.debug_hand(2), h2_before[1..], "随机→首张确定性近似");
    assert!(e.debug_hand(3).is_empty(), "无牌者不触发");
    assert!(e.debug_hp(3) == 4 || e.debug_hp(3) == 3, "3 号存活不受影响");
    // 弃牌入弃堆（守恒）
    assert_zone_conservation(&e);

    // 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, jdef)),
        "绝红实例往返"
    );
    assert!(r.state_core().player(0).equips.is_empty());
    assert_zone_conservation(&r);
}
