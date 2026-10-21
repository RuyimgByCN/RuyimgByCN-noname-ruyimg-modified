use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十四（规则 v131）：安娜（ow_ana）支援（zhiyuan，伤害前
// 置窗口触发）——你即将对一名角色造成伤害时（量>0）可确认：防止此伤
// 害，受伤者回复等量体力。mianzhen/aqianghua 登记暂缓（子技链）。
// ---------------------------------------------------------------------------

/// 支援：确认→伤害防止（体力不变）+受伤者回复 1（2→3）；取消→伤害
/// 照常（3→2）。DamageBefore 前置窗口首个真实消费者路径。
#[test]
fn ana_zhiyuan_cancel_damage_and_equal_recover() {
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

    let mut e = Engine::new_with_extensions(9951, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "zhiyuan");
    // 连弩解除限次（需连出两杀验证确认/取消两分支）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    for _ in 0..3 {
        e.debug_place(0, "sha");
    }
    e.debug_set_hp(1, 2);

    // ① 杀 1 号→支援确认门（OptionalGate）→Confirm：防止+回复 1
    let sha1 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha1,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c = e.open_choice().cloned().expect("支援确认门");
    assert_eq!(c.observer, 0, "确认门 observer=持有者");
    assert_eq!(
        c.spec.prompt, "trigger_confirm",
        "确认为触发门：{:?}",
        c.spec
    );
    let cid = c.id;
    drop(c);
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: cid,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "支援确认：{out:?}"
    );
    assert_eq!(e.debug_hp(1), 3, "伤害防止（2 不变）+回复 1（→3）");

    // ② 第二次杀→取消：伤害照常（3→2）
    let sha2 = e
        .debug_hand(0)
        .into_iter()
        .find(|c| e.catalog().cards[*c as usize].kind == CardKind::Sha)
        .expect("应仍有杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha2,
            targets: vec![1],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    // 确认门的取消=空 selection（Pass 非 options 成员）
    let out = answer_open(&mut e, Vec::new());
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    assert_eq!(e.debug_hp(1), 2, "取消后伤害照常（3→2）");
    assert!(e.open_choice().is_none(), "无残留挂起");
}
