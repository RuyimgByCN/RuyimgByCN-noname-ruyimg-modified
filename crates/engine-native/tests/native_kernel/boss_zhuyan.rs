use super::*;

/// 朱厌批末片（规则 v226/存档 v51）：巨源——回合结束记录体力；出牌
/// 阶段体力低于记录时【杀】可额外 1 目标；体力≥记录时单目标。
#[test]
fn boss_zhuyan_juyuan_storage_and_extra_target() {
    // Prototype 局验证巨源两面（End 臂纯查询不受 Boss 模式影响；
    // boss_stage 局 first=0 与虚失翻面臂的自动推进交互独立登记）
    let mut e = Engine::new(45001, 4);
    let zhuyan_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 9) as u16;
    e.debug_set_general_def(1, zhuyan_def);
    e.debug_set_hp(1, 25);
    let jdef = e.catalog().skill_by_name("巨源").expect("巨源入目录");
    assert!(e.state_core().skills.contains_key(&(1, jdef)));
    assert_eq!(e.debug_hp(1), 25, "朱厌 25 血装配");
    assert!(e.state_core().player(1).juyuan_storage == 0, "初始未记录");

    // ① 回合结束记录体力 25（1 号出牌阶段 NextPhase 跳到下家 Draw，
    //    其回合的 End begin 帧（含巨源记录臂）在快进链中提交）
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    next_phase(&mut e); // Play→Discard（虚失未激活不翻面）
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End
    drain_choices(&mut e);
    next_phase(&mut e); // End→收束→phaseAfter（K53 巨源记录臂）
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(1).juyuan_storage,
        25,
        "回合结束记录 25（hp={}, phase={:?})",
        e.debug_hp(1),
        e.turn().phase
    );

    // ② 体力掉到 20（<记录 25）→ 杀可双目标
    e.debug_set_hp(1, 20);
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    e.debug_clear_hand(0);
    e.debug_clear_hand(2);
    e.debug_set_hp(0, 4);
    e.debug_set_hp(2, 4);
    let sha = e.debug_place(1, "sha");
    let out = play(&mut e, sha, vec![0, 2]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "体力<记录 → 杀可双目标：{out:?}"
    );
    drain_choices(&mut e);

    // ③ 记录刷新：回合结束后 juyuan=当前体力（K53：phaseAfter 段记录，
    // 多推一次收束）
    next_phase(&mut e);
    drain_choices(&mut e);
    next_phase(&mut e);
    drain_choices(&mut e);
    next_phase(&mut e);
    drain_choices(&mut e);
    assert_eq!(
        e.state_core().player(1).juyuan_storage,
        e.debug_hp(1),
        "回合结束刷新记录=当前体力"
    );

    // ④ 存档往返（v51）
    let snap = e.save();
    let r = Engine::restore(&snap).expect("v51 往返");
    assert_eq!(
        r.state_core().player(1).juyuan_storage,
        e.state_core().player(1).juyuan_storage,
        "juyuan_storage 往返一致"
    );
    assert_zone_conservation(&r);
}

/// 朱厌批续片（规则 v225）：虚失——体力首次≤一半激活；出牌阶段结束
/// 自翻面；翻回正面时全场其他角色各受 1（随机恒 1 近似）。
#[test]
fn boss_zhuyan_xushi_flip_and_aoe() {
    let mut e = Engine::new_boss_stage(43001, 4, 1, 3, 6, 4, 0);
    let zhuyan_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 9) as u16;
    e.debug_set_general_def(1, zhuyan_def);
    e.debug_set_hp(1, 25);
    let xdef = e.catalog().skill_by_name("虚失").expect("虚失入目录");
    assert!(e.state_core().skills.contains_key(&(1, xdef)));
    assert!(!e.state_core().has_skill(1, xdef), "虚失装配即未激活");

    // 激活门控：hp 置 12（≤12 半线）→ 一次伤害（12→11）激活
    e.debug_set_hp(1, 12);
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    e.debug_clear_hand(1);
    let sha = e.debug_place(2, "sha");
    play(&mut e, sha, vec![1]);
    drain_choices(&mut e);
    assert!(e.state_core().has_skill(1, xdef), "体力≤一半 → 虚失激活");

    // ① 朱厌自己回合的弃牌阶段开始 → 自翻面
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    next_phase(&mut e); // 1 Play→Discard（虚失自翻面）
    drain_choices(&mut e);
    assert!(
        e.state_core().player(1).face_down,
        "虚失：朱厌出牌阶段结束自翻面"
    );

    // ② 翻面者回合跳过+翻回正面 → 全场其他角色各受 1
    let hp0 = e.debug_hp(0);
    let hp2 = e.debug_hp(2);
    let hp3 = e.debug_hp(3);
    advance_to_play(&mut e, 1); // 下一轮朱厌 Judge begin：翻回+全场伤
    drain_choices(&mut e);
    assert!(
        !e.state_core().player(1).face_down,
        "翻面者回合跳过时翻回正面"
    );
    assert!(
        e.debug_hp(0) < hp0 && e.debug_hp(2) < hp2 && e.debug_hp(3) < hp3,
        "虚失翻正：全场其他角色各受 1（{}→{}、{}→{}、{}→{}）",
        hp0,
        e.debug_hp(0),
        hp2,
        e.debug_hp(2),
        hp3,
        e.debug_hp(3)
    );
}

/// 朱厌批首片（规则 v224）：兵先——他人回合未出杀（出牌阶段口径）→
/// 其结束阶段被朱厌虚拟杀（受 1 或响应）；出了杀则不被打。
#[test]
fn boss_zhuyan_bingxian_virtual_sha_on_no_sha_turn() {
    let mut e = Engine::new_boss_stage(42001, 4, 1, 3, 6, 4, 0);
    let zhuyan_def = (e.catalog().general_count()
        - engine_native::content::standard::BOSS_GENERALS.len()
        + 9) as u16; // BOSS_GENERALS[9] 朱厌
    e.debug_set_general_def(1, zhuyan_def);
    e.debug_set_hp(1, 25);
    let bdef = e.catalog().skill_by_name("兵先").expect("兵先入目录");
    assert!(e.state_core().skills.contains_key(&(1, bdef)));

    // ① 2 号回合未出杀 → 其结束阶段被朱厌虚拟杀（-1）
    advance_to_play(&mut e, 2);
    e.debug_clear_hand(2);
    e.debug_set_hp(2, 4);
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End（兵先虚拟杀）
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(2), 3, "兵先：未出杀回合结束被虚拟杀（4-1）");

    // ② 出了杀的回合 → 不被打：0 号回合出杀打 3 号后 End
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 4);
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![3]); // 0 杀 3（出杀）
    drain_choices(&mut e);
    let hp0_before = e.debug_hp(0);
    next_phase(&mut e); // Play→Discard
    drain_choices(&mut e);
    next_phase(&mut e); // Discard→End（已出杀 → 兵先不触发）
    drain_choices(&mut e);
    assert_eq!(e.debug_hp(0), hp0_before, "出杀回合 → 兵先不触发");
}
