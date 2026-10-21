use super::*;

/// 绝情：张春华造成伤害→目标失去等量体力（无 DamageDealt/HpLost 面）；
/// 对照：普通来源伤害照常；无心伤害半：张春华受伤→失去 1（非伤害路径）。
#[test]
fn boss_zhangchunhua_jueqing_wuxin_damage_faces() {
    let mut e = engine_native::Engine::new_boss_stage(81001, 4, 1, 3, 6, 4, 0);
    let zch_def = zhangchunhua_def_of(&e);
    e.debug_set_general_def(1, zch_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(zch_def).map(|g| g.0),
        Some("张春华"),
        "张春华入目录"
    );
    let jq = e.catalog().skill_by_name("绝情").expect("绝情入目录");
    let wx = e.catalog().skill_by_name("无心").expect("无心入目录");
    let ss = e.catalog().skill_by_name("伤逝").expect("伤逝入目录");
    for sk in [jq, wx, ss] {
        assert!(e.state_core().skills.contains_key(&(1, sk)), "实例齐备");
    }

    // ① 绝情：1 号出杀打 0 号（无闪）→ 0 号失去 1（非伤害路径）
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(0);
    let sha = e.debug_place(1, "sha");
    assert!(matches!(
        play(&mut e, sha, vec![0]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(hp(&e, 0), 3, "绝情：目标失去 1");
    assert_eq!(
        count_log(&e, |l| matches!(l, LogEvent::DamageDealt { .. })),
        0,
        "无伤害日志"
    );
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::HpLost {
            player: 0,
            amount: 1
        }
    )));

    // ② 对照：0 号出杀打 3 号（相邻普通来源）→ 伤害路径照常
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(3);
    let sha2 = e.debug_place(0, "sha");
    assert!(matches!(
        play(&mut e, sha2, vec![3]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    // 注：advance_to_play(0) 途中 1 号 End——shangshix2 已先全体-1（4→3）
    assert_eq!(hp(&e, 3), 2, "shangshix2 先-1+普通伤害 1");
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::DamageDealt {
            target: 3,
            amount: 1,
            ..
        }
    )));

    // ③ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert!(r.state_core().skills.contains_key(&(1, jq)));
    assert_zone_conservation(&r);
}

/// 无心伤害半（独立局）：0 号出杀打张春华→失去 1（非伤害路径；局内无
/// shangshix2 跨回合累积——1 号 End 未到）。
#[test]
fn boss_zhangchunhua_wuxin_damage_only() {
    let mut e = engine_native::Engine::new_boss_stage(81003, 4, 1, 3, 6, 4, 0);
    let zch_def = zhangchunhua_def_of(&e);
    e.debug_set_general_def(1, zch_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    let sha = e.debug_place(0, "sha");
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(hp(&e, 1), 3, "无心：张春华失去 1");
    assert_eq!(
        count_log(&e, |l| matches!(l, LogEvent::DamageDealt { target: 1, .. })),
        0,
        "无心伤害不走伤害路径"
    );
    assert!(log_has(&e, |l| matches!(
        l,
        LogEvent::HpLost {
            player: 1,
            amount: 1
        }
    )));
    assert_zone_conservation(&e);
}

/// 无心延时半：张春华不可用乐/兵（他人照常）；伤逝补牌（受伤后 0→4）；
/// shangshix2：结束阶段 hp>1 全体（含自己）各失去 1。
#[test]
fn boss_zhangchunhua_wuxin_delay_and_shangshix() {
    let mut e = engine_native::Engine::new_boss_stage(81002, 4, 1, 3, 6, 4, 0);
    let zch_def = zhangchunhua_def_of(&e);
    e.debug_set_general_def(1, zch_def);
    e.debug_set_hp(1, 4);

    // ① 无心延时半：张春华用乐→拒
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let le = e.debug_place(1, "le");
    let out = play(&mut e, le, vec![0]);
    assert!(
        !matches!(out, engine_native::CommandOutcome::Accepted { .. }),
        "无心：张春华不可用乐 {out:?}"
    );

    // ② 他人用乐照常（0 号对 2 号；无懈窗放行）
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    let le2 = e.debug_place(0, "le");
    let out = play(&mut e, le2, vec![2]);
    assert!(
        matches!(
            out,
            engine_native::CommandOutcome::Accepted { .. }
                | engine_native::CommandOutcome::Pending { .. }
        ),
        "他人用乐照常 {out:?}"
    );
    pass_nullify_windows(&mut e);
    assert_zone_conservation(&e);

    // ③ 伤逝补牌：1 号清手后受伤（经无心转失去）→ 手牌补至 4
    e.debug_clear_hand(1);
    assert_eq!(e.debug_hand(1).len(), 0);
    e.debug_deal_damage(Some(0), 1, 1, DamageNature::Normal);
    drain_choices(&mut e);
    // 注：advance_to_play(0) 途中经过 1 号 End——shangshix2 已先全体-1
    //（4→3），故此处为 3-1=2
    assert_eq!(hp(&e, 1), 2, "shangshix2 先-1+无心转化-1");
    assert_eq!(e.debug_hand(1).len(), 4, "伤逝：手牌补至 4");

    // ④ shangshix2：1 号回合结束阶段（hp=3>1）→ 全体存活各失去 1
    advance_to_play(&mut e, 1);
    let hps: Vec<i32> = (0..4u8).map(|p| hp(&e, p)).collect();
    e.debug_clear_hand(1);
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → shangshix2
    drain_choices(&mut e);
    for p in 0..4u8 {
        assert_eq!(hp(&e, p), hps[p as usize] - 1, "shangshix2：{p} 号失去 1");
    }
    // 伤逝自补：自己 hp 变化后手牌 0→4
    assert_eq!(e.debug_hand(1).len(), 4, "伤逝自触发补牌");

    // ⑤ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}
