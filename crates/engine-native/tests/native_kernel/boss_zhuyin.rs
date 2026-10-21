use super::*;

/// boss 内容落包首批（规则 v208）：烛阴（hp4）单技凶兽三面——
/// ①杀伤条件 +1（目标体力小于来源）；②与其他角色距离 -1；
/// ③不能被翻面；装配/存档往返。
#[test]
fn boss_zhuyin_xiongshou_semantics() {
    use engine_native::catalog::CardKind;
    let mut e = Engine::new_boss_stage(25001, 4, 1, 3, 6, 4, 0);
    // 烛阴 def = 动态尾段（base+启用扩展数 之后的第一条 boss 段条目）
    let zhuyin_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()) as u16;
    e.debug_set_general_def(1, zhuyin_def);
    // 满血装配（general_meta hp=4）
    assert_eq!(e.debug_hp(1), 4, "烛阴 4 血装配");
    // 技能实例已授
    let xdef = e.catalog().skill_by_name("凶兽").expect("凶兽应入目录");
    assert!(
        e.state_core().skills.contains_key(&(1, xdef)),
        "装配后凶兽技能实例应存在"
    );

    // ① 杀伤条件 +1：boss（4 血）用杀打 3 血目标 → 2 点伤害
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(2, 3);
    e.debug_set_deck_top_by(|d| d.suit.is_spade() || d.suit.is_club());
    let sha = e.debug_place(1, "sha");
    play(&mut e, sha, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 1, "凶兽：杀对体力小于来源者伤害 +1（3-2=1）");
    // 对照：无凶兽者（3 号，与 2 号相邻）出杀 → 伤害恒 1
    //（advance_to_play(1) 已处于 Play 会直接返回不重置杀限；0 与 2 相
    // 距 2 超出杀射程，故选 3 号）
    advance_to_play(&mut e, 3);
    e.debug_set_hp(2, 4);
    e.debug_clear_hand(3);
    e.debug_clear_hand(2); // 清目标手牌防闪避
    let sha2 = e.debug_place(3, "sha");
    play(&mut e, sha2, vec![2]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 3, "无凶兽不加成（4-1=3）");

    // ② 距离 -1：computed_distance(boss→3 号) = 座位距离-1
    let d = e.state_core().computed_distance(e.catalog(), 1, 3);
    assert!(d < 2, "凶兽进攻距离 -1（相邻座位 2→1）：{d}");

    // ③ 不能被翻面：对 boss 发翻面事件 → face_down 不变
    e.debug_flip(1);
    assert!(!e.state_core().player(1).face_down, "凶兽：不可被翻面");

    // 存档往返：技能实例与手牌一致
    let snap = e.save();
    let r = Engine::restore(&snap).expect("往返");
    assert!(
        r.state_core().skills.contains_key(&(1, xdef)),
        "技能实例往返"
    );
    let _ = CardKind::Sha;
    assert_zone_conservation(&r);
}
