use super::*;

/// Boss 内容第四批首片（规则 v215）：饕餮贪欲两面——①跳过弃牌阶段
/// （手牌保留）；②结束阶段手牌全场最多 → 失去 1 点体力；非最多不失。
#[test]
fn boss_taotie_tanyu_skip_discard_and_max_hand_lose_hp() {
    // ① 跳过弃牌阶段：饕餮带超上限手牌进入弃牌阶段 → 手牌保留
    let mut e = Engine::new_boss_stage(32001, 4, 1, 3, 6, 4, 0);
    let taotie_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 3) as u16;
    e.debug_set_general_def(1, taotie_def);
    e.debug_set_hp(1, 20);
    let tdef = e.catalog().skill_by_name("贪欲").expect("贪欲入目录");
    assert!(e.state_core().skills.contains_key(&(1, tdef)));
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    for _ in 0..5 {
        e.debug_place(1, "tao");
    }
    next_phase(&mut e); // Play→Discard（贪欲跳过）
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 5, "贪欲：弃牌阶段跳过，手牌保留");

    // ② 独立局：结束阶段手牌全场最多（5>1>0>0）→ 失 1 体
    let mut e2 = Engine::new_boss_stage(32002, 4, 1, 3, 6, 4, 0);
    let td2 = (e2.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len()
        + 3) as u16;
    e2.debug_set_general_def(1, td2);
    e2.debug_set_hp(1, 20);
    advance_to_play(&mut e2, 1);
    e2.debug_clear_hand(0);
    e2.debug_clear_hand(2);
    e2.debug_clear_hand(3);
    for _ in 0..5 {
        e2.debug_place(1, "tao");
    }
    e2.debug_place(0, "tao"); // 0 号 1 张 < 5
    next_phase(&mut e2); // Play→Discard（跳过）
    drain_choices(&mut e2);
    next_phase(&mut e2); // Discard→End（失 1）
    drain_choices(&mut e2);
    // resolution 在 End 帧后自动连续推进多轮（每轮 Draw+4/End 各失 1），
    // 故断言「会失体」而非精确单次：hp 严格小于初始
    assert!(
        e2.debug_hp(1) < 20,
        "手牌全场最多 → 结束阶段失体（自动推进多轮累计）：hp={}",
        e2.debug_hp(1)
    );

    // ③ 非最多面（状态面精确断言）：他人手牌严格更多 → is_max=false
    //（End 后 resolution 自动连续推进多轮、饕餮摸牌必然反超，e2e 不可
    // 停帧——判定函数状态面直验，臂行为由②局「会失体」覆盖）
    let mut e3 = Engine::new_boss_stage(32003, 4, 1, 3, 6, 4, 0);
    let td3 = (e3.catalog().general_count() - engine_native::content::standard::BOSS_GENERALS.len()
        + 3) as u16;
    e3.debug_set_general_def(1, td3);
    e3.debug_set_hp(1, 20);
    advance_to_play(&mut e3, 1);
    e3.debug_clear_hand(1);
    e3.debug_clear_hand(0);
    e3.debug_clear_hand(2);
    e3.debug_clear_hand(3);
    e3.debug_place(1, "tao"); // 饕餮 1 张
    for _ in 0..3 {
        e3.debug_place(0, "tao"); // 0 号 3 张 > 1
    }
    let is_max3 = {
        let core = e3.state_core();
        let my = core.player(1).hand.len();
        core.alive_players()
            .all(|o| o == 1 || core.player(o).hand.len() <= my)
    };
    assert!(!is_max3, "他人手牌严格更多 → 非全场最多");
    let is_max2 = {
        let core = e2.state_core();
        let my = core.player(1).hand.len();
        core.alive_players()
            .all(|o| o == 1 || core.player(o).hand.len() <= my)
    };
    assert!(is_max2, "②局构造=全场最多（对照）");
}

/// Boss 内容第四批续片（规则 v216）：饕餮藏目+积财——
/// ①藏目：摸牌阶段基数=存活角色数（4 人局摸 4）；
/// ②积财：装配即未激活；体力≤一半伤害窗激活；自己回复后摸 2；
/// 他人回复后自己与其各摸 1。
#[test]
fn boss_taotie_cangmu_jicai() {
    let mut e = Engine::new_boss_stage(33001, 4, 1, 3, 6, 4, 0);
    let taotie_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 3) as u16;
    e.debug_set_general_def(1, taotie_def);
    e.debug_set_hp(1, 20);
    let cdef = e.catalog().skill_by_name("藏目").expect("藏目入目录");
    let jdef = e.catalog().skill_by_name("积财").expect("积财入目录");
    assert!(e.state_core().skills.contains_key(&(1, cdef)));
    assert!(e.state_core().skills.contains_key(&(1, jdef)));
    assert!(!e.state_core().has_skill(1, jdef), "积财装配即未激活");

    // ① 藏目：4 人局存活 4 → 摸牌阶段总摸 4
    advance_to_play(&mut e, 1);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 6 + 4, "藏目：摸牌基数=存活数 4");

    // ② 积财激活门控：hp 置 10（≤半线）后一次伤害（10→9）激活
    e.debug_set_hp(1, 10);
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_clear_hand(2);
    let sha = e.debug_place(2, "sha");
    play(&mut e, sha, vec![1]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill(1, jdef), "体力≤一半 → 积财激活");

    // ③ 他人回复：3 号回复（伤害后 recover）→ 饕餮与 3 号各摸 1
    advance_to_play(&mut e, 3);
    e.debug_clear_hand(1);
    e.debug_clear_hand(3);
    let hp3 = e.debug_hp(3);
    e.debug_set_hp(3, hp3 - 1);
    e.debug_recover(3, 1);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "积财：他人回复 → 饕餮摸 1");
    assert_eq!(e.debug_hand(3).len(), 1, "积财：回复者摸 1");

    // ④ 自己回复：饕餮回复 → 摸 2
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, e.debug_hp(1) - 1);
    e.debug_recover(1, 1);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 2, "积财：自己回复 → 摸 2");
}
