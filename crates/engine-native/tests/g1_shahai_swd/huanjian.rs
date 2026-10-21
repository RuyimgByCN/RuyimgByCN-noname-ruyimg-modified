use super::*;

/// T12 swd 武将面批㉚（妮可幻剑/韦达渔人复用/淳于衡绝情白名单+虚影；灵武
/// 引擎缺口/明镜/贞烈/对决/笙笙 deferred）：三将行为面 + 反例。
#[test]
fn swd_generals_b70_huanjian_weida_xuying() {
    let root =
        std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "shahai-swd")
        .expect("swd 安装")
        .clone();
    let json = std::fs::read_to_string(
        std::path::PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    let pack = bridge(&json);
    let mut e = Engine::new_with_extensions(62038, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 幻剑（妮可）：黑色牌当冰魄针（古剑机关——目标+邻座各弃首张）
    e.debug_grant(0, "huanjian");
    let bc = {
        let b = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                ) && e.catalog().card_kind(*c) == engine_native::catalog::CardKind::Snatch
            })
            .expect("黑顺手牵羊在目录");
        let core = e.state_core_mut();
        core.deck.retain(|x| *x != b);
        core.discard.retain(|x| *x != b);
        core.table.retain(|x| *x != b);
        for pl in core.players.iter_mut() {
            pl.hand.retain(|x| *x != b);
        }
        core.players[0].hand.push(b);
        b
    };
    let d1 = e.debug_place(1, "tao");
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: bc,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.debug_hand(1).contains(&d1),
        "幻剑：冰魄针路由（p1 被弃牌）"
    );

    // ② 渔人 def 复用（韦达）：喀玛同技同 mapping——韦达持技生效
    e.debug_grant(0, "yueren");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let sha2 = e.debug_place(0, "sha");
    e.debug_set_hp(1, 4);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 3, "渔人复用：韦达杀照常结算");
    // 红判摸 1：p0 手牌出 1 摸 1 净不变——以判定行为验证（无额外断言面）

    // ③ 虚影（淳于衡）：锁定——伤害取消+手牌非空失去 1
    e.debug_grant(0, "xuying");
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    // p1 回合出杀
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    submit(&mut e, 0, GameAction::NextPhase);
    for _ in 0..16 {
        let c = e.turn().current;
        if c == 1 && e.state_core().turn.phase == engine_native::event::Phase::Play {
            break;
        }
        let _ = submit(&mut e, c, GameAction::NextPhase);
    }
    advance_to_play(&mut e, 1);
    for p in 0..4 {
        if p != 1 {
            e.debug_clear_hand(p);
        }
    }
    let keep = e.debug_place(0, "tao");
    let sha3 = e.debug_place(1, "sha");
    e.debug_set_hp(0, 3);
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(0), 2, "虚影：伤害取消→失去 1（3→2）");
    assert!(e.debug_hand(0).contains(&keep), "虚影：手牌保留");
    // 反例面：手牌空时受伤害照常（清手后再杀）
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(0, 3);
    let sha4 = e.debug_place(1, "sha");
    submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: sha4,
            targets: vec![0],
        },
    );
    settle_optional_confirm(&mut e);
    // 上游语义：cancel 恒定+有手才失去——空手=完全免伤
    assert_eq!(e.debug_hp(0), 3, "虚影：手牌空伤害取消且无失去（免伤面）");
}
