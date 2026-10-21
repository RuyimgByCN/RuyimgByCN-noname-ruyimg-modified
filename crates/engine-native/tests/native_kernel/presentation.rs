use super::*;

// ---------------------------------------------------------------------------
// N10 第六十二批：表现事件层与渲染投影（ADR 0761）
// ---------------------------------------------------------------------------

/// 表现层可见性：他人摸牌仅数量（无牌名）；本人摸牌带牌名；
/// 观战者全部摸牌仅数量。渲染帧确定性且视图字段一致。
#[test]
fn presentation_visibility_and_render_determinism() {
    let mut e = Engine::new(971, 3);
    advance_to_play(&mut e, 0);
    // 座位 0 出杀攻 1（1 无牌 → 直接伤害路径由 1 出闪询问与否决定）
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let sha = e.debug_place(0, "sha");
    play(&mut e, sha, vec![1]);
    if e.open_choice().is_some() {
        answer(&mut e, ChoiceOption::Pass);
    }

    // 本人视图：自己摸牌带牌名
    let own = e.presentation_for(Some(0));
    assert!(
        own.iter().any(|ev| matches!(
            ev,
            engine_native::presentation::PresentationEvent::Drawn {
                player: 0,
                cards: Some(_),
                ..
            }
        )),
        "本人摸牌应携带牌名"
    );
    // 他人视图：座位 0 的摸牌仅数量
    let opp = e.presentation_for(Some(1));
    for ev in &opp {
        if let engine_native::presentation::PresentationEvent::Drawn { player, cards, .. } = ev {
            if *player != 1 {
                assert!(cards.is_none(), "他人摸牌不得携带牌名");
            }
        }
    }
    // 观战者：所有摸牌仅数量
    let spec = e.presentation_for(None);
    for ev in &spec {
        if let engine_native::presentation::PresentationEvent::Drawn { cards, .. } = ev {
            assert!(cards.is_none(), "观战者摸牌不得携带牌名");
        }
    }

    // 渲染帧：确定性 + 内容要素
    let f1 = e.render_frame(Some(1), 64);
    let f2 = e.render_frame(Some(1), 64);
    assert_eq!(f1, f2, "同输入渲染必须逐字节一致");
    assert!(f1.contains("座位0"), "帧应包含座位行");
    assert!(f1.contains("回合:0"), "帧应包含回合头");
    // 视图字段一致性：座位 0 的手牌数与权威一致
    let authoritative = e.debug_hand(0).len();
    assert!(
        f1.contains(&format!(
            "座位0 体力{}/4 手牌{}",
            e.debug_hp(0),
            authoritative
        )),
        "帧座位行应与权威状态一致"
    );

    // 终局帧：胜负横幅
    e.debug_set_hp(1, 1);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 1);
    e.debug_deal_damage(None, 0, 1, DamageNature::Normal);
    e.debug_deal_damage(None, 1, 1, DamageNature::Normal);
    let o = e.outcome().expect("原型模式 3 人局应可推到终局");
    let _ = o;
    e.debug_deal_damage(None, 2, 9, DamageNature::Normal);
    let f3 = e.render_frame(None, 64);
    assert!(f3.contains("终局"), "终局帧应含胜负横幅");
    assert!(matches!(e.outcome(), Some(o) if o.winner == Winner::Player(2)));
}
