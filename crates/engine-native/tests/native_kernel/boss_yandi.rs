use super::*;

/// 星宿阶段2系纯入库批（规则 v243）：炎帝/少昊双条目（技面=神裔 v233+
/// 神恩光环 v238 全覆盖，尾巴赤翼/白翼=青翼同族已处置）——条目元数据+
/// 装配面；神恩光环随新携带者激活（既有臂通用性）。
#[test]
fn boss_yandi_shaohao_pure_entries() {
    let mut e = Engine::new_boss_stage(63001, 4, 1, 3, 6, 4, 0);
    let base = e.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len();
    let yandi_def = (base + 19) as u16; // BOSS_GENERALS[19] 炎帝
    let shaohao_def = (base + 20) as u16; // BOSS_GENERALS[20] 少昊
    assert_eq!(
        e.catalog().general_meta(yandi_def).map(|g| g.0),
        Some("炎帝"),
        "炎帝入目录"
    );
    assert_eq!(
        e.catalog().general_meta(shaohao_def).map(|g| g.0),
        Some("少昊"),
        "少昊入目录"
    );
    assert_eq!(e.catalog().general_meta(yandi_def).map(|g| g.3), Some(6));
    assert_eq!(e.catalog().general_meta(shaohao_def).map(|g| g.3), Some(6));
    // 装配面：神裔+神恩实例
    e.debug_set_general_def(1, yandi_def);
    let sdef = e.catalog().skill_by_name("神裔").expect("神裔");
    let snd = e.catalog().skill_by_name("神恩").expect("神恩");
    assert!(e.state_core().skills.contains_key(&(1, sdef)));
    assert!(e.state_core().skills.contains_key(&(1, snd)));
    // 光环激活（既有臂通用性）：炎帝=boss 侧无距离；挑战者摸牌 +1
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    e.debug_set_hp(3, 4);
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![3]);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "神恩光环：boss 方隔位杀可指定"
    );
    // 挑战者摸牌 +1（0 号下回合摸 3——清手后校验）
    next_phase(&mut e);
    drain_choices(&mut e);
    next_phase(&mut e);
    drain_choices(&mut e);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_clear_hand(3);
    advance_to_play(&mut e, 0);
    assert_eq!(e.debug_hand(0).len(), 3, "挑战者摸 3（2+光环 1）");
    // 存档往返
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, snd)),
        "神恩实例往返"
    );
    assert_zone_conservation(&r);
    assert_zone_conservation(&e);
}
