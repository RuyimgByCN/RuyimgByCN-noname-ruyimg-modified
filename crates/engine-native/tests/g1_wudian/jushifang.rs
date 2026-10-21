use super::*;

/// 械磐：弃牌阶段弃掉最后一张手牌基本牌→询问弃一张手牌→实体发现
/// 随机装备入手（可取消=放弃）；装备来自牌堆/弃堆实体池（非凭空造牌）。
#[test]
fn jushifang_xiepan_discard_and_discover_equip() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "shaai-xianjian-upstream-noname")
        .expect("xianjian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    let mut e = Engine::new_with_extensions(10059, 3, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "xiepan");
    e.debug_set_max_hp(0, 1);
    e.debug_set_hp(0, 1);
    let sha = e.debug_place(0, "sha");
    let wuzhong = e.debug_place(0, "wuzhong");

    // Play→Discard：弃牌阶段询问弃 1（手牌 2-上限 1）→ 弃基本牌 sha
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let c = e.open_choice().expect("弃牌阶段挂起");
    assert_eq!(c.spec.prompt, "discard_to_limit");
    let picks: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .copied()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == sha))
        .collect();
    assert_eq!(picks.len(), 1);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: picks,
        },
    );

    // 械磐挂起：弃最后一张基本牌+手牌仍非空（wuzhong）→ 弃 1 获随机装备
    let c = e.open_choice().expect("械磐弃牌挂起");
    assert_eq!(
        c.spec.prompt,
        engine_native::content::host::WUDIAN_XJ_XIEPAN_CARDS
    );
    assert_eq!((c.spec.min, c.spec.max), (1, 1));
    assert!(c.spec.allow_cancel);
    let picks: Vec<ChoiceOption> = c
        .spec
        .options
        .iter()
        .copied()
        .filter(|o| matches!(o, ChoiceOption::Card(x) if *x == wuzhong))
        .collect();
    assert_eq!(picks.len(), 1);
    let (cid, obs) = (c.id, c.observer);
    submit(
        &mut e,
        obs,
        GameAction::Answer {
            choice_id: cid,
            selection: picks,
        },
    );

    // 断言：手牌=1 张装备实体（来自牌堆/弃堆）；sha+wuzhong 均入弃堆
    let hand = e.debug_hand(0);
    assert_eq!(hand.len(), 1, "弃 1 获 1");
    let got = hand[0];
    assert!(
        e.catalog().card_kind(got).equip_slot().is_some(),
        "获得牌为装备实体"
    );
    assert!(
        e.state_core().discard.contains(&sha) && e.state_core().discard.contains(&wuzhong),
        "sha+弃牌均入弃牌堆"
    );
}
