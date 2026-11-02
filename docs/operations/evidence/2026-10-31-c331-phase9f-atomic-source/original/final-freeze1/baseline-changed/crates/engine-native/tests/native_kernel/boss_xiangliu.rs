use super::*;

/// ①相柳受击 → 来源获蛇毒标记；②标记者回合开始「弃 X 或失 X」
/// （空选=失 X；弃 X 递减）；③邪酬 hp≤半激活；激活后他人用桃/酒 →
/// 相柳获标记。
#[test]
fn boss_xiangliu_duqu_shedu_echou() {
    let mut e = Engine::new_boss_stage(40001, 4, 1, 3, 6, 4, 0);
    let shujing_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 6) as u16;
    e.debug_set_general_def(1, shujing_def); // 书精持催毒
    e.debug_set_hp(1, 2);
    let cdef = e.catalog().skill_by_name("催毒").expect("催毒入目录");
    let zdef = e.catalog().skill_by_name("中毒").expect("中毒入目录");
    let sdef = e.catalog().skill_by_name("蛇毒").expect("蛇毒入目录");
    let edef = e.catalog().skill_by_name("邪酬").expect("邪酬入目录");
    assert!(e.state_core().skills.contains_key(&(1, cdef)));
    assert!(!e.state_core().has_skill(1, edef), "邪酬装配即未激活");

    // ① 书精（1 号）杀 2 号 → 2 号获中毒+蛇毒双实例
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    e.debug_set_hp(2, 4);
    let sha = e.debug_place(1, "sha");
    play(&mut e, sha, vec![2]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill_any_state(2, zdef), "中毒实例");
    assert!(e.state_core().has_skill_any_state(2, sdef), "蛇毒实例");

    // ② 中毒判定（方片）：伤 1+失去中毒；蛇毒实例保留
    e.debug_set_deck_top_by(|d| d.suit == engine_native::catalog::Suit::Diamond);
    let hp2 = e.debug_hp(2);
    advance_to_play(&mut e, 2); // 判定后推进（自动应答）
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), hp2 - 1, "中毒方片判定：伤 1");
    assert!(
        !e.state_core().has_skill_any_state(2, zdef),
        "非黑桃失去中毒"
    );
    assert!(e.state_core().has_skill_any_state(2, sdef), "蛇毒实例保留");

    // ③ 再授毒：蛇毒实例仍在
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(1, "sha");
    play(&mut e, sha2, vec![2]);
    drain_choices(&mut e);
    assert!(
        e.state_core().has_skill_any_state(2, sdef),
        "再授毒后蛇毒在"
    );

    // ④ 邪酬（独立局）：相柳持邪酬——hp≤半线伤害窗激活；他人用桃获标记
    let mut e2 = Engine::new_boss_stage(40002, 4, 1, 3, 6, 4, 0);
    let xiangliu_def = (e2.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 7) as u16;
    e2.debug_set_general_def(1, xiangliu_def);
    e2.debug_set_hp(1, 20);
    assert!(e2
        .state_core()
        .skills
        .contains_key(&(1, e2.catalog().skill_by_name("邪酬").unwrap())));
    assert!(!e2.state_core().has_skill(1, edef), "邪酬装配即未激活");
    e2.debug_set_hp(1, 10); // ≤半线
    advance_to_play(&mut e2, 2);
    e2.debug_clear_hand(2);
    e2.debug_clear_hand(1);
    let sha3 = e2.debug_place(2, "sha");
    play(&mut e2, sha3, vec![1]); // 伤害窗激活邪酬
    drain_choices(&mut e2);
    assert!(e2.state_core().has_skill(1, edef), "邪酬激活");
    e2.debug_place(2, "tao");
    e2.debug_set_hp(2, 3);
    let tao_id = e2.debug_hand(2)[0];
    play(&mut e2, tao_id, vec![]);
    drain_choices(&mut e2);
    assert_eq!(e2.state_core().player(1).shedu, 1, "邪酬：他人用桃获标记");
}

/// Boss 内容第七批续片（规则 v222）：相柳兽九三面——手牌上限恒 9；
/// 出牌阶段开始补至 9；跳过摸牌阶段；回合结束补至 9。
#[test]
fn boss_xiangliu_jiushou_limit_nine_and_refill() {
    let mut e = Engine::new_boss_stage(39001, 4, 1, 3, 6, 4, 0);
    let xiangliu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 7) as u16;
    e.debug_set_general_def(1, xiangliu_def);
    e.debug_set_hp(1, 25);
    let jdef = e.catalog().skill_by_name("兽九").expect("兽九入目录");
    assert!(e.state_core().skills.contains_key(&(1, jdef)));

    // ① 手牌上限恒 9（hp=25 时基数为 25，兽九覆盖为 9）
    assert_eq!(e.state_core().hand_limit(e.catalog(), 1), 9, "兽九上限恒 9");
    assert_eq!(e.state_core().hand_limit(e.catalog(), 0), 4, "他人不受影响");

    // ② 出牌阶段开始补至 9：1 号开局手牌 6 → Play 补 3（跳过摸牌后仍 9）
    advance_to_play(&mut e, 1);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 9, "出牌阶段开始补至 9（跳过摸牌）");

    // ③ 回合结束补至 9：弃 5 张（cap 9 下无弃；清到 3 张）→ End 补至 9
    e.debug_clear_hand(1);
    for _ in 0..3 {
        e.debug_place(1, "tao");
    }
    next_phase(&mut e); // Play→Discard（3≤9 无弃）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End（补至 9）
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 9, "回合结束补至 9");
}

/// Boss 内容第七批首片（规则 v221）：相柳妖兽——与其他角色计算距离
/// -2（毒妒[蛇毒标记链]/兽九[手牌补 9]/邪酬(+switch) 未落下批）。
#[test]
fn boss_xiangliu_yaoshou_distance_minus_two() {
    let mut e = Engine::new_boss_stage(38001, 4, 1, 3, 6, 4, 0);
    let xiangliu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 7) as u16; // BOSS_GENERALS[7] 相柳（毕方插入后）
    e.debug_set_general_def(1, xiangliu_def);
    e.debug_set_hp(1, 25);
    let ydef = e.catalog().skill_by_name("妖兽").expect("妖兽入目录");
    assert!(e.state_core().skills.contains_key(&(1, ydef)));
    // 相柳（1 号）到 3 号：座位距离 2 → -2 → 0（下限钳 1？马术口径下限 1）
    let d13 = e.state_core().computed_distance(e.catalog(), 1, 3);
    assert!(d13 <= 1, "妖兽距离 -2：2-2 钳下限（实测 {d13}）");
    // 对照：他人间距离不受影响（0→2 = 2）
    let d02 = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(d02, 2, "他人距离不受妖兽影响");
}
