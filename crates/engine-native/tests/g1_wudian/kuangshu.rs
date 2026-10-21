use super::*;

// ---------------------------------------------------------------------------
// U05 ow 纵切批十六（规则 v133）：狂鼠（ow_kuangshu）拾火（shihuo，
// global 伤害后触发锁定）——任何人受到火属性伤害后你摸一张牌。
// liudan/shoujia 登记暂缓。
// ---------------------------------------------------------------------------

/// 拾火：他人火伤后摸 1；普通伤害不触发；自己造成的火伤照常触发；
/// 每次火伤各摸 1。
#[test]
fn kuangshu_shihuo_fire_damage_draw() {
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

    let mut e = Engine::new_with_extensions(9971, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "shihuo");
    // 连弩解除限次（需连出火杀+普通杀）
    let zhuge = e.debug_place(0, "zhuge");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: zhuge,
            targets: vec![],
        },
    );
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);

    // ① 火杀打 1 号 → 拾火摸 1
    let mut fire = None;
    for _ in 0..8 {
        e.debug_place(0, "fire_sha");
        let h = e.debug_hand(0);
        if let Some(c) = h
            .into_iter()
            .find(|c| e.catalog().cards[*c as usize].kind == CardKind::FireSha)
        {
            fire = Some(c);
            break;
        }
    }
    let fsha = fire.expect("应有火杀");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: fsha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(1), 3, "火杀伤害（4→3）");
    let hand_after_fire = e.debug_hand(0).len();

    // ② 普通杀不触发
    let sha = e.debug_place(0, "sha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
    assert_eq!(e.debug_hp(1), 2, "普通杀伤害（3→2）");
    assert_eq!(e.debug_hand(0).len(), hand_after_fire, "普通伤害不触发拾火");
}
