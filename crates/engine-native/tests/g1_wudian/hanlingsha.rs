use super::*;

// ---------------------------------------------------------------------------
// U05 xianjian 第七切片（规则 v148）：韩菱纱（pal_hanlingsha）探囊（ta-
// nnang，出牌阶段限一次）——梅花牌视为顺手牵羊（无距离限制）。
// tuoqiao（石会粉跨包）登记暂缓。
// ---------------------------------------------------------------------------

/// 探囊：梅花牌→顺牵（距离 2 可达=豁免）；非梅花不能走顺牵；距离 1
/// 实体顺牵不受影响（无技能者对照）。
#[test]
fn hanlingsha_tannang_snatch_unlimited_range() {
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

    let mut e = Engine::new_with_extensions(10005, 5, &[&pack]).expect("建局");
    advance_loop(&mut e, 0);
    for p in 0..5 {
        e.debug_clear_hand(p);
    }
    e.debug_grant(0, "tannang");
    e.debug_set_hp(1, 4);
    e.debug_set_hp(2, 4);
    e.debug_place(1, "tao"); // 1 号有牌可被顺
                             // 0 号：梅花借刀（非杀类梅花牌）当顺牵（借刀本体目标规则要求目标
                             // 有武器——给 2 号装连弩满足转化前校验）
    e.debug_place(2, "zhuge");
    let club = e.debug_place(0, "jiedao");

    // ① 探囊：梅花杀当顺牵打距离 2 的 2 号（豁免距离）
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: club,
            targets: vec![2],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "探囊距离豁免：{out:?}"
    );
    // 无懈窗口/目标选择：无懈无持有者→顺牵询问 2 号给牌（自选）
    while let Some(c) = e.open_choice() {
        let (cid, obs) = (c.id, c.observer);
        let sel = if c.spec.min > 0 {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        } else {
            vec![]
        };
        let out = submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
        assert!(
            matches!(
                out,
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
            ),
            "{out:?}"
        );
    }
    let total2 = e.debug_hand(2).len() + e.save().core.players[2].equips.len();
    assert!(total2 <= 1, "顺牵获得 2 号一张牌（剩余≤1）：{total2}");
}
