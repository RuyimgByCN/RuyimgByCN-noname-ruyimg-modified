use super::*;

/// T12 swd 武将面批㉙（玉澧灵心/喀玛渔人+伤逝白名单；天香 defer）：两将
/// 行为面 + 反例。
#[test]
fn swd_generals_b69_lingxin_yueren() {
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
    let mut e = Engine::new_with_extensions(62037, 4, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }

    // ① 灵心（玉澧）：阶段结束——牌堆顶三张红桃入手其余弃置
    e.debug_grant(0, "lingxin");
    // 压顶三张：红桃×2 + 黑♠×1（pop 序=压入逆序：最后压=先取）
    let h1 = {
        let h = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Heart
                ) && e.state_core().deck.contains(c)
            })
            .expect("红桃 1 在牌库");
        e.state_core_mut().deck.retain(|x| *x != h);
        e.state_core_mut().deck.push(h);
        h
    };
    let sp = {
        let sp = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                matches!(
                    e.catalog().cards[*c as usize].suit,
                    engine_native::catalog::Suit::Spade
                ) && e.state_core().deck.contains(c)
                    && e.catalog().card_kind(*c) != engine_native::catalog::CardKind::Sha
            })
            .expect("黑非杀在牌库");
        e.state_core_mut().deck.retain(|x| *x != sp);
        e.state_core_mut().deck.push(sp);
        sp
    };
    let h2 = {
        // 排除已压顶的 h1（同一张牌不可取两次——牌堆序随种子变化，须显式排除）
        let h = (0..e.catalog().cards.len() as u32)
            .find(|c| {
                *c != h1
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Heart
                    )
                    && e.state_core().deck.contains(c)
            })
            .expect("红桃 2 在牌库");
        e.state_core_mut().deck.retain(|x| *x != h);
        e.state_core_mut().deck.push(h);
        h
    };
    // 推进 p0 任一阶段结束（Play→Discard 触发 Play 阶段末灵心）
    let h0 = e.debug_hand(0).len();
    submit(&mut e, 0, GameAction::NextPhase);
    settle_optional_confirm(&mut e);
    assert!(
        e.debug_hand(0).contains(&h2)
            && e.debug_hand(0).contains(&h1)
            && !e.debug_hand(0).contains(&sp),
        "灵心：两红桃入手、黑牌弃置"
    );
    assert_eq!(e.debug_hand(0).len(), h0 + 2, "灵心：净增 2");
    let _ = sp;

    // ② 渔人（喀玛）：杀指定目标后判定——黑→弃目标 he 随机一张
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yueren");
    let sha2 = e.debug_place(0, "sha");
    let vic = e.debug_place(1, "tao");
    e.debug_set_hp(1, 4);
    // 判定堆顶黑（♠）
    e.debug_set_deck_top_by(|def| def.suit.is_spade());
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert!(
        !e.debug_hand(1).contains(&vic),
        "渔人：黑判→目标 he 随机弃 1"
    );
    assert_eq!(e.debug_hp(1), 3, "渔人：杀照常结算（p1 无闪受 1）");
    // 反例面：红判→摸 1（近似登记）
    next_p0_play_fn(&mut e);
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "yueren");
    let sha3 = e.debug_place(0, "sha");
    let h0b = e.debug_hand(0).len();
    e.debug_set_hp(1, 4);
    e.debug_set_deck_top_by(|def| def.suit.is_heart());
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    // 出杀 1 + 红判摸 1 → 手牌净不变
    assert_eq!(
        e.debug_hand(0).len(),
        h0b,
        "渔人：红判→摸 1（收回近似登记）"
    );
    assert_eq!(e.debug_hp(1), 3, "渔人：红判杀照常结算");
}
