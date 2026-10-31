use super::*;

/// 四象系玄武首片（规则 v237/存档 v54）：灵渠——①受伤>1 点归零（终值
/// 口径：1+凶兽=2 亦归零）；②每次受伤摸 1+受击计数（0 点伤害照常）；
/// ③手牌上限+计数。玄武条目（群目录近似/hp4/[神裔,灵渠]；boss_xuanlin2
/// 玄麟=关卡流程面登记不落内核）。
#[test]
fn boss_xuanwu_lingqu_damage_counter() {
    let mut e = Engine::new_boss_stage(57001, 4, 1, 3, 6, 4, 0);
    let xuanwu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 13) as u16; // BOSS_GENERALS[13] 玄武
    e.debug_set_general_def(1, xuanwu_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(xuanwu_def).map(|g| g.0),
        Some("玄武"),
        "玄武入目录"
    );
    let ldef = e.catalog().skill_by_name("灵渠").expect("灵渠入目录");
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, ldef)));
    assert!(e.state_core().skills.contains_key(&(1, sdef)), "神裔共用");

    // ① 大于 1 点伤害归零：3 点直伤 → hp 不变；摸 1+计数 1（0 点照常）
    e.debug_clear_hand(1);
    let h1 = e.debug_hand(1).len();
    e.debug_deal_damage(Some(0), 1, 3, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 4, "灵渠：3 点伤害归零");
    assert_eq!(e.state_core().player(1).lingqu_count, 1, "0 点伤害仍计数");
    assert_eq!(e.debug_hand(1).len(), h1 + 1, "0 点伤害仍摸 1");

    // ② 1 点伤害照常：hp-1、再摸 1、计数 2
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 3, "1 点伤害照常结算");
    assert_eq!(e.state_core().player(1).lingqu_count, 2);
    assert_eq!(e.debug_hand(1).len(), h1 + 2);

    // ③ 增幅终值口径：1 点基础+赤龙牙类增幅终值>1 → 归零（借火杀+焚天
    //    不可得，直接 2 点直伤验证终值门槛）
    e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 3, "2 点伤害归零");
    assert_eq!(e.state_core().player(1).lingqu_count, 3);

    // ④ 手牌上限+计数：hp3 基数 3 + 计数 3 = 6
    let lim = e.state_core().hand_limit(e.catalog(), 1);
    assert_eq!(lim, 6, "手牌上限 = hp 3 + 灵渠计数 3");

    // ⑤ 存档 v54 往返：计数跨存档
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert_eq!(r.state_core().player(1).lingqu_count, 3, "灵渠计数入存档");
    assert!(
        r.state_core().skills.contains_key(&(1, ldef)),
        "灵渠实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}
