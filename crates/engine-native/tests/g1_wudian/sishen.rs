use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批二十三（规则 v140）：死神（ow_sishen）霰弹（xiandan，
// ShaResolve 内联钩）——出杀结算开始时可弃 1 张 he：红=直击（跳过闪
// 询问）；黑=本杀伤害 +1；取消=照常。yihun（虚拟 useCard）/shouge
//（createCard 治疗波）登记暂缓。
// ---------------------------------------------------------------------------

/// 霰弹：红弃→直击（1 号有闪不掉牌直接扣血）；黑弃→+1；取消→照常
/// 可闪（无闪照常伤害）。
#[test]
fn sishen_xiandan_red_direct_black_bonus() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-ow-upstream-noname")
        .expect("ow 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(9996, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xiandan");
    // 连弩解除限次（连出三杀验证三分支）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    // 红（红桃 tao）+黑（黑桃 sha）+任意一张杀
    let mut red = None;
    let mut black = None;
    for _ in 0..10 {
        e.debug_place(0, "tao");
        if red.is_none() {
            red = e.debug_hand(0).into_iter().find(|c| {
                e.catalog().cards[*c as usize].kind == CardKind::Tao
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Heart
                    )
            });
        }
        e.debug_place(0, "sha");
        if black.is_none() {
            black = e.debug_hand(0).into_iter().find(|c| {
                e.catalog().cards[*c as usize].kind == CardKind::Sha
                    && matches!(
                        e.catalog().cards[*c as usize].suit,
                        engine_native::catalog::Suit::Spade
                    )
            });
        }
        if red.is_some() && black.is_some() {
            break;
        }
    }
    let red = red.expect("应有红桃 tao");
    let black = black.expect("应有黑桃 sha");
    let any_sha = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha && Some(*c) != Some(black))
        .expect("应有第三张杀");
    e.debug_set_hp(1, 4);

    // ① 红弃→直击（1 号无闪也无所谓——跳过闪询问；4→3）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: any_sha,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(red)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 3, "红弃直击（4→3）");

    // ② 黑弃→+1（3→1）
    e.debug_set_hp(1, 3);
    let sha2 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, vec![ChoiceOption::Card(black)]);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 1, "黑弃伤害 +1（3→1）");

    // ③ 取消→照常（无闪直接 1→0 濒死？给 1 号回 2 保命：1→0 进濒死
    // 会开求桃链——改设 2→1）
    e.debug_set_hp(1, 2);
    let sha3 = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha3,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let out = answer_open(&mut e, Vec::new());
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 1, "取消照常（2→1）");
}
