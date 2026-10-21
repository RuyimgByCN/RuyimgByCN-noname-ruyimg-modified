use super::*;

/// 凌波：打出闪后摸 2；娇孝：成为红色牌目标摸 1（黑牌对照不摸）。
#[test]
fn boss_zhenji_lingbo_jiaoxia_draws() {
    let mut e = engine_native::Engine::new_boss_stage(83001, 4, 1, 3, 6, 4, 0);
    let zj_def = zhenji_def_of(&e);
    e.debug_set_general_def(1, zj_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(zj_def).map(|g| g.0),
        Some("甄姬"),
        "甄姬入目录"
    );
    for sk in ["踏水", "凌波", "娇孝", "芳华"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    // ① 凌波：0 出杀打 1（持闪）→ 闪响应后摸 2
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(1);
    let shan = e.debug_place(1, "shan");
    let sha = e.debug_place(0, "sha");
    assert!(matches!(
        play(&mut e, sha, vec![1]),
        engine_native::CommandOutcome::Pending { .. }
    ));
    assert_eq!(prompt_of(&e), "respond_shan");
    assert!(matches!(
        answer(&mut e, ChoiceOption::Card(shan)),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    assert_eq!(e.debug_hand(1).len(), 2, "凌波：闪打出后摸 2");
    assert_eq!(hp(&e, 1), 4, "闪响应免伤");

    // ② 娇孝正例：甄姬回合自置红闪电（shandian_safe=♥12，按名精确布阵）
    // → 娇孝摸 1（无懈窗放行；自置=隐式自目标等价分支；显式 targets 面=
    // 红乐/红杀同理——桃园/五谷/无中为隐式全员不触发，登记）
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let red_sd = e.debug_move_named_card_to_hand(1, "shandian_safe");
    assert!(matches!(
        play(&mut e, red_sd, vec![]),
        engine_native::CommandOutcome::Accepted { .. }
            | engine_native::CommandOutcome::Pending { .. }
    ));
    pass_nullify_windows(&mut e);
    drain_choices(&mut e);
    assert_eq!(e.debug_hand(1).len(), 1, "娇孝：红色牌目标摸 1");

    // ③ 娇孝负例：甄姬出黑杀打 0（同回合闪电非杀不占限次）→ 伤害 1 不摸
    e.debug_clear_hand(0);
    let black_sha = place_sha_of_color(&mut e, 1, true);
    assert!(matches!(
        play(&mut e, black_sha, vec![0]),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert_eq!(hp(&e, 0), 3, "黑杀伤害 1");
    // 踏水不额外触发（黑杀 ✓ 翻面 0 号——非娇孝面）
    assert_eq!(e.debug_hand(1).len(), 1, "黑牌目标非己：娇孝不摸");
    assert_zone_conservation(&e);

    // ④ 往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}

/// 踏水：黑牌后座位序首个其他存活翻面；芳华：结束阶段翻面者各失 1。
#[test]
fn boss_zhenji_tashui_flip_and_fanghua() {
    let mut e = engine_native::Engine::new_boss_stage(83002, 4, 1, 3, 6, 4, 0);
    let zj_def = zhenji_def_of(&e);
    e.debug_set_general_def(1, zj_def);
    e.debug_set_hp(1, 4);
    advance_to_play(&mut e, 1);

    // ① 踏水：甄姬出黑杀打 0（无闪）→ 0 号被翻面
    e.debug_clear_hand(0);
    let black_sha = place_sha_of_color(&mut e, 1, true);
    let b_out = play(&mut e, black_sha, vec![0]);
    assert!(matches!(
        b_out,
        engine_native::CommandOutcome::Accepted { .. }
    ));
    drain_choices(&mut e);
    assert!(e.state_core().player(0).face_down, "踏水：0 号翻面");
    assert_eq!(hp(&e, 0), 3, "杀伤害 1（无娇孝：0 号非持有者）");

    // ② 芳华：甄姬 End → 翻面者（0 号）失 1
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → 芳华
    drain_choices(&mut e);
    assert_eq!(hp(&e, 0), 2, "芳华：翻面者失 1");
    assert!(e.state_core().player(0).face_down, "芳华不翻回");
    assert!(!e.state_core().player(2).face_down && !e.state_core().player(3).face_down);

    // ③ 往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert!(r.state_core().player(0).face_down);
    assert_zone_conservation(&r);
}
