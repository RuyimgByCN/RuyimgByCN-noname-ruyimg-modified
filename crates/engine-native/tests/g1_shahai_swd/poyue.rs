use super::*;

/// T12 swd 武将面批 28：破月（拓跋玉儿 tag 288）——黑杀无视距离、红杀
/// 不计入出牌阶段限一次、红杀不可被闪避（poyue2 group 同承载）；niepan
/// 登记外部包技依赖不落（拓跋玉儿 2/3）。
#[test]
fn swd_generals_b28_poyue_black_range_red_free_direct() {
    use engine_native::catalog::Suit;
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
    let mut e = Engine::new_with_extensions(62011, 4, &[&pack]).expect("建局");
    e.debug_grant(0, "poyue");
    advance_to_play(&mut e, 0);

    let place_sha_color = |e: &mut Engine, p: u8, black: bool| {
        let id = e.debug_set_deck_top_by(move |d| {
            d.kind.is_sha_like()
                && if black {
                    matches!(d.suit, Suit::Spade | Suit::Club)
                } else {
                    matches!(d.suit, Suit::Heart | Suit::Diamond)
                }
        });
        {
            let core = e.state_core_mut();
            core.deck.retain(|x| *x != id);
            core.players[p as usize].hand.push(id);
        }
        id
    };

    // 正例 ①：黑杀打距离 2 的 p2（无距离技时本应被拒）
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let black_sha = place_sha_color(&mut e, 0, true);
    assert!(matches!(
        e.catalog().cards[black_sha as usize].suit,
        Suit::Spade | Suit::Club
    ));
    e.debug_set_hp(2, 8);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black_sha,
            targets: vec![2],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(2), 7, "破月：黑杀无视距离命中 p2");

    // 正例 ②：红杀连续两张不计限一次（第二张不再被拒）
    e.debug_set_hp(1, 8);
    let red1 = place_sha_color(&mut e, 0, false);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red1,
            targets: vec![1],
        },
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 7, "破月②：第一张红杀命中 p1");
    let red2 = place_sha_color(&mut e, 0, false);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red2,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "破月②：红杀不计限一次 {out:?}"
    );
    settle_optional_confirm(&mut e);
    assert_eq!(e.debug_hp(1), 6, "破月②：第二张红杀命中");

    // 正例 ③：红杀不可被闪避（p1 有闪仍直中）
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _shan_p1 = e.debug_place(1, "shan");
    let red3 = place_sha_color(&mut e, 0, false);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: red3,
            targets: vec![1],
        },
    );
    assert!(
        e.open_choice().is_none(),
        "破月③：红杀不出闪询问（directHit）"
    );
    assert_eq!(e.debug_hp(1), 5, "破月③：p1 有闪仍直中（6-1）");

    // 反例 ④：黑杀照常可被闪避（p2 有闪 → 闪窗出现）
    {
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("重置");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let _shan_p2 = e.debug_place(2, "shan");
    let black2 = place_sha_color(&mut e, 0, true);
    submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black2,
            targets: vec![2],
        },
    );
    let w = e.open_choice().map(|c| c.spec.prompt);
    assert_eq!(
        w.as_deref(),
        Some("respond_shan"),
        "破月反例：黑杀照常出闪询问"
    );

    // 反例 ⑤：撤销破月后黑杀打距离 2 被拒
    {
        let def_py = e.catalog().skill_by_name("poyue").expect("破月 def");
        let mut snap = e.save();
        snap.core.turn.sha_used_play_phase = false;
        snap.core.skills.remove(&(0, def_py));
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("撤销破月");
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let black3 = place_sha_color(&mut e, 0, true);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: black3,
            targets: vec![2],
        },
    );
    assert!(
        !matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "破月反例：无技黑杀距离 2 被拒 {out:?}"
    );
}
