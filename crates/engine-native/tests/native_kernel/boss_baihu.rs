use super::*;

/// 四象系末片首刀（规则 v235）：狂笑（白虎）——【杀】无距离限制（隔
/// 位目标可指定）且目标数上限无限（三目标齐中）；非持有者对照多目标被
/// 拒。白虎条目（群目录近似/hp4/[神裔,狂笑]；boss_baimang2 白芒=关卡流
/// 程面登记不落内核）。
#[test]
fn boss_baihu_kuangxiao_sha_range_and_targets() {
    let mut e = Engine::new_boss_stage(55001, 4, 1, 3, 6, 4, 0);
    let baihu_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 11) as u16; // BOSS_GENERALS[11] 白虎
    e.debug_set_general_def(1, baihu_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(baihu_def).map(|g| g.0),
        Some("白虎"),
        "白虎入目录"
    );
    let kdef = e.catalog().skill_by_name("狂笑").expect("狂笑入目录");
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, kdef)));
    assert!(e.state_core().skills.contains_key(&(1, sdef)), "神裔共用");

    // ① 无距离：1 号隔位（距离 2>攻击范围 1）杀 3 号 → 可指定
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 4);
    let dist = e.state_core().computed_distance(e.catalog(), 1, 3);
    assert!(dist > 1, "隔位距离应大于 1（实测 {dist}）");
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![3]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "狂笑：无距离限制 {out:?}"
    );
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(3), 3, "隔位杀命中");

    // ② 目标数无限：三目标齐指定 → 全部命中（新回合避开杀限；全清手防闪）
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    let sha2 = e.debug_place(1, "sha");
    let out = play(&mut e, sha2, vec![0, 2, 3]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "狂笑：目标数无限 {out:?}"
    );
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), 3, "目标 0 受 1");
    assert_eq!(e.debug_hp(2), 3, "目标 2 受 1");
    assert_eq!(e.debug_hp(3), 2, "目标 3 累计受 1");

    // ③ 对照（非持有者）：多目标杀被拒
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let sha3 = e.debug_place(0, "sha");
    let out = play(&mut e, sha3, vec![1, 2, 3]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "非持有者：多目标杀应被拒"
    );

    // ④ 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, kdef)),
        "狂笑实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}
