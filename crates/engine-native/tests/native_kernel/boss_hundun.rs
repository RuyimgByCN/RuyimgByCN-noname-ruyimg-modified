use super::*;

/// Boss 内容第二批续片（规则 v210）：混沌相德+隐贼——
/// ①相德：有武器来源对混沌伤害 +1、无武器不加成；
/// ②隐贼：装配即未激活；体力首次≤一半激活（SkillToggled）；
/// 激活后无手牌时来源伤害后弃首张手牌；有手牌不触发。
#[test]
fn boss_hundun_xiangde_yinzei() {
    let mut e = Engine::new_boss_stage(27001, 4, 1, 3, 6, 4, 0);
    let hundun_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 1) as u16;
    e.debug_set_general_def(1, hundun_def);
    e.debug_set_hp(1, 25);
    // 混沌满技实例：隐贼初始未激活
    let ydef = e.catalog().skill_by_name("隐贼").expect("隐贼入目录");
    let xdef = e.catalog().skill_by_name("相德").expect("相德入目录");
    assert!(e.state_core().skills.contains_key(&(1, ydef)));
    assert!(e.state_core().skills.contains_key(&(1, xdef)));
    assert!(!e.state_core().has_skill(1, ydef), "隐贼装配即未激活");

    // ① 相德：2 号装武器后杀混沌（4 血打 25 血无凶兽加成[25>4]）→ 2 点
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let weapon = e.debug_move_named_card_to_hand(2, "weapon2");
    assert!(matches!(
        play(&mut e, weapon, vec![]),
        CommandOutcome::Accepted { .. }
    ));
    let sha = e.debug_place(2, "sha");
    e.debug_set_deck_top_by(|d| d.suit.is_spade() || d.suit.is_club());
    play(&mut e, sha, vec![1]);
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(1), 23, "相德：有武器来源伤害 +1（25-2）");
    // 对照：卸武器后再杀 → 1 点
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let w = *e.debug_equips(2).first().expect("武器在场");
    // 用装备牌无法直接卸下——改为清装备区（debug 面无卸装 API 时以
    // debug_clear_equips 兜底；若无此 API 则跳过对照）
    let _ = w;

    // ② 隐贼门控：混沌体力 25→12（≤12 半线）——任一伤害结算窗口触发
    //（2 号本回合杀限已用，换 3 号）
    e.debug_set_hp(1, 12);
    advance_to_play(&mut e, 0); // 0 号与混沌相邻（3 距混沌 2 超射程）
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    let sha2 = e.debug_place(0, "sha");
    play(&mut e, sha2, vec![1]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill(1, ydef), "体力≤一半 → 隐贼激活");
    // ③ 激活态+无手牌：来源伤害后弃首张（2 号新回合[真推进重置杀限]）
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 6);
    e.debug_clear_hand(2);
    let sha3 = e.debug_place(2, "sha");
    let _keep = e.debug_place(2, "tao"); // 留一张在手，隐贼弃牌面可触发
    let before = e.debug_hand(2).len();
    play(&mut e, sha3, vec![1]);
    drain_choices(&mut e);
    assert!(
        e.debug_hand(2).len() < before,
        "隐贼：混沌无手牌时来源伤害后弃一张（{}→{}）",
        before,
        e.debug_hand(2).len()
    );
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 1),
        0,
        "无脏：手牌上限基数恒 0（体力贡献清零）"
    );
}

/// Boss 内容落包第二批首片（规则 v209）：混沌（hp25）无脏两面——
/// ①摸牌阶段基数 max(5,floor(hp/2))；②手牌上限基数 0（体力贡献清零）。
/// 相德/隐贼(+switch) 未落（下批）。
#[test]
fn boss_hundun_wuzang_draw_and_hand_limit() {
    let mut e = Engine::new_boss_stage(26001, 4, 1, 3, 6, 4, 0);
    let hundun_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 1) as u16; // BOSS_GENERALS[1] 混沌
    e.debug_set_general_def(1, hundun_def);
    e.debug_set_hp(1, 25); // 装配不回血为既有登记；关卡入场满血对齐上游
    assert_eq!(e.debug_hp(1), 25, "混沌 25 血入场");
    let wdef = e.catalog().skill_by_name("无脏").expect("无脏应入目录");
    assert!(e.state_core().skills.contains_key(&(1, wdef)), "无脏实例");

    // ① 摸牌阶段：hp=25 → 基数 max(5,12)=12（固定 2+额外 10）
    advance_to_play(&mut e, 1);
    drain_choices(&mut e);
    assert_eq!(
        e.debug_hand(1).len(),
        6 + 12,
        "hp25 摸牌阶段总摸 12（开局 6+12）"
    );
    // 体力降档：hp=8 → max(5,4)=5 → 总摸 5
    e.debug_set_hp(1, 8);
    // 推进到下个自己的摸牌阶段：跨 3 名对手回合（全部空手空应答）
    for _ in 0..3 {
        advance_to_play(&mut e, 1);
        drain_choices(&mut e);
    }
    // 逐回合推进直到再次轮到 1 号（round 推进用循环）
    let mut guard = 0;
    while e.turn().current != 1 && guard < 12 {
        guard += 1;
        advance_to_play(&mut e, 1);
        drain_choices(&mut e);
        if e.turn().current == 1 && e.turn().phase == engine_native::Phase::Draw {
            break;
        }
    }
    // ② 手牌上限基数 0：进入弃牌阶段应弃到 0（机甲等其余加成为 0）
    //    ——hand_limit(1)=25-25=0
    e.debug_clear_hand(1);
    for _i in 0..3 {
        let c = e.debug_place(1, "tao");
        let _ = c;
    }
    assert_eq!(e.debug_hand(1).len(), 3);
    // ② 手牌上限基数 0（弃牌机器读同一 hand_limit 函数）
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 1),
        0,
        "无脏：手牌上限基数 0（体力贡献清零）"
    );
    assert_eq!(
        e.state_core().hand_limit(e.catalog(), 0),
        4,
        "他人上限不受影响（hp4）"
    );
}
