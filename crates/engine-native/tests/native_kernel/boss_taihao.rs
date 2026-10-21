use super::*;

/// 星宿阶段2系首片（规则 v238）：神恩（太昊/炎帝全局光环）——boss 方
/// 用牌无距离（隔位杀可指定；挑战者对照被拒）；挑战者侧摸牌 +1、手牌
/// 上限 +1（boss 侧不加）。太昊条目（群目录近似/hp6/[神裔,神恩]；青翼
/// 未落随下批）。
#[test]
fn boss_taihao_shenen_aura() {
    let mut e = Engine::new_boss_stage(58001, 4, 1, 3, 6, 4, 0);
    let taihao_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 14) as u16; // BOSS_GENERALS[14] 太昊
    e.debug_set_general_def(1, taihao_def);
    e.debug_set_hp(1, 6);
    assert_eq!(
        e.catalog().general_meta(taihao_def).map(|g| g.0),
        Some("太昊"),
        "太昊入目录"
    );
    let sdef = e.catalog().skill_by_name("神恩").expect("神恩入目录");
    let ydef = e.catalog().skill_by_name("神裔").expect("神裔入目录");
    assert!(e.state_core().skills.contains_key(&(1, sdef)));
    assert!(e.state_core().skills.contains_key(&(1, ydef)), "神裔共用");

    // ① boss 方无距离：1 号（boss）隔位杀 3 号 → 可指定
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 4);
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![3]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "神恩：boss 方无距离 {out:?}"
    );
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(3), 3, "隔位杀命中");

    // ② 挑战者对照：0 号隔位杀 2 号 → 距离拒（神恩不加持挑战者距离）
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    let sha2 = e.debug_place(0, "sha");
    let out = play(&mut e, sha2, vec![2]);
    assert!(
        !matches!(out, CommandOutcome::Accepted { .. }),
        "挑战者：隔位杀应被距离拒"
    );

    // ③ 挑战者摸牌 +1 / boss 侧不加：0 号（挑战者）下回合摸 3，1 号摸 2
    //    （当前在 0 号出牌阶段；先数现手牌再推进一整圈比较）
    let h0 = e.debug_hand(0).len();
    let _h1 = e.debug_hand(1).len();
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    next_phase(&mut e); // 0 Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // 0 Discard→End
    drain_choices(&mut e);
    advance_to_play(&mut e, 0); // 1(摸2)→2(摸3)→3(摸3)→0(摸3)
    assert_eq!(e.debug_hand(1).len(), 2, "boss 侧摸 2");
    assert_eq!(e.debug_hand(0).len(), 3, "挑战者摸 3（2+光环 1）");
    let _ = h0;

    // ④ 挑战者手牌上限 +1 / boss 侧不加
    let hp0 = e.debug_hp(0) as usize;
    let hp1 = e.debug_hp(1) as usize;
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        hp0 + 1,
        "挑战者上限=hp+1"
    );
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 1),
        hp1,
        "boss 侧上限=hp（无加成）"
    );

    // ⑤ 存档往返+守恒
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, sdef)),
        "神恩实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}
