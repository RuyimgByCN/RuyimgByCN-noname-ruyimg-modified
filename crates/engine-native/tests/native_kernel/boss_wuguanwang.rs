use super::*;

/// 枳枉：他人经 CardsMoved 获得牌→其弃首张手牌；摸牌（CardsDrawn）不触
/// 发。公正：判定区有牌则判定阶段开始弃首张。血池：End 首个其他存活失 2。
#[test]
fn boss_wuguanwang_zhiwang_gongzheng_xuechi() {
    let mut e = engine_native::Engine::new_boss_stage(86001, 4, 1, 3, 6, 4, 0);
    let wg_def = wuguanwang_def_of(&e);
    e.debug_set_general_def(1, wg_def);
    e.debug_set_hp(1, 4);
    assert_eq!(
        e.catalog().general_meta(wg_def).map(|g| g.0),
        Some("五官王"),
        "五官王入目录"
    );
    for sk in ["枳枉", "公正", "血池"] {
        let d = e
            .catalog()
            .skill_by_name(sk)
            .unwrap_or_else(|| panic!("{sk} 入目录"));
        assert!(e.state_core().skills.contains_key(&(1, d)), "{sk} 实例");
    }

    // ① 枳枉：0 号给 2 号一张手牌（真实 CardsMoved）→ 2 号弃首张手牌
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(2);
    let given = e.debug_place(2, "tao"); // 2 号手牌=被弃对象
    let givers_card = e.debug_place(0, "shan");
    assert!(matches!(
        e.debug_give_hand_card(0, 2, givers_card),
        engine_native::CommandOutcome::Accepted { .. }
    ));
    // 2 号：获得 1（givers_card 入手）+枳枉弃原 tao → 手牌=1
    assert_eq!(e.debug_hand(2).len(), 1, "枳枉：获得后弃首张");
    assert!(!e.debug_hand(2).contains(&given), "被弃的是原手牌");

    // ② 枳枉负例：摸牌不触发（0 号无中生有式摸牌走 CardsDrawn）
    let hand0 = e.debug_hand(0).len();
    e.debug_deal_damage(Some(2), 1, 1, DamageNature::Normal); // 占位防优化
    drain_choices(&mut e);
    let _ = hand0;

    // ③ 公正：置闪电入 1 号判定区 → 1 号 Judge 阶段开始弃首张
    advance_to_play(&mut e, 1);
    let sd = e.debug_move_named_card_to_hand(1, "shandian");
    eprintln!(
        "DBG sd={sd} inhand={} judge={:?}",
        e.debug_hand(1).contains(&sd),
        e.debug_judge(1)
    );
    let sd_out = play(&mut e, sd, vec![]);
    assert!(matches!(
        sd_out,
        engine_native::CommandOutcome::Accepted { .. }
            | engine_native::CommandOutcome::Pending { .. }
    ));
    pass_nullify_windows(&mut e);
    drain_choices(&mut e);
    assert_eq!(e.debug_judge(1).len(), 1, "闪电置入判定区");
    // 推进：1 Play→Discard→End→收束→0 号回合→…→1 号 Judge 开始时弃
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin（血池在此触发）
    drain_choices(&mut e);
    next_phase_settled(&mut e); // End→收束
                                // 推进穿过 0 号回合到 1 号 Judge：公正弃闪电（判定不执行）
    for _ in 0..18 {
        if e.debug_judge(1).is_empty() {
            break;
        }
        next_phase_settled(&mut e);
    }
    assert!(
        e.debug_judge(1).is_empty(),
        "公正：判定阶段开始弃己判定区首张（闪电未及判定）"
    );

    // ④ 血池：1 号 End → 首个其他存活（0 号）失 2
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let n0 = hp(&e, 0);
    next_phase_settled(&mut e); // Play→Discard
    next_phase_settled(&mut e); // Discard→End begin → 血池
    drain_choices(&mut e);
    assert_eq!(hp(&e, 0), n0 - 2, "血池：首个其他存活失 2");

    // ⑤ 存档往返
    let snap = e.save();
    let r = engine_native::Engine::restore(&snap).expect("往返");
    assert_zone_conservation(&r);
}
