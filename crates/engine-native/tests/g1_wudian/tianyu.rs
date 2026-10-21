use super::*;

/// 车阵（U07 技能批五，规则 v115）：田豫——锁定距离技：有装备时你算
/// 到他人 -1；目标无装备时其被算到 +1（computed_distance 直读）。
#[test]
fn tianyu_chezhen_distance_mod() {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let disc = discovered
        .into_iter()
        .find(|d| d.name == "wudian-upstream-noname")
        .expect("wudian 可发现");
    let json = std::fs::read_to_string(disc.root.join("assets/cards.json")).unwrap();
    let pack = bridge(&json);

    // 5 人局：对座距离 2（3 人局全员相邻会被距离下限 1 饱和）
    let mut e = Engine::new_with_extensions(8801, 5, &[&pack]).expect("建局");
    e.debug_grant(0, "chezhen");
    let qg = (0..e.catalog().cards.len() as u32)
        .find(|i| e.catalog().cards[*i as usize].kind == engine_native::catalog::CardKind::Qinggang)
        .expect("青釭剑在目录");
    {
        let mut snap = e.save();
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|c| *c != qg);
        }
        snap.core.deck.retain(|c| *c != qg);
        snap.core.discard.retain(|c| *c != qg);
        snap.core.table.retain(|c| *c != qg);
        snap.core.players[0].equips.push(qg);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("青釭注入");
    }

    // 验证一：0 有装备 → 车阵攻侧 -1；目标 2 无装备 → 被算到 +1（对座
    // 基线 2：关=2-1+1=2、开=2-1=1）
    e.debug_set_skill_enabled(0, "chezhen", false);
    let off = e.state_core().computed_distance(e.catalog(), 0, 2);
    e.debug_set_skill_enabled(0, "chezhen", true);
    let on = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert_eq!(off, 2, "关：车阵攻侧 -1 与守侧 +1 抵消");
    assert_eq!(on, 1, "开：有装备攻侧 -1（守侧 +1 仅对无装备目标）");

    // 验证二：2→1 无车阵参与（2 号无 chezhen）→ 基线 1（对座 5 人局
    // 2↔1 相邻）；1 号无装备不影响他人算向自己（守侧 +1 需车阵持有者
    // 参与判定，此处 b=1 持技方是 0 号非 1 号）
    let to_1 = e.state_core().computed_distance(e.catalog(), 2, 1);
    assert_eq!(to_1, 1, "2→1 相邻基线");

    // 验证三：把青釭转注给 1 号 → 1 号无装备条件消失 → 0 号技不再给
    // 他人算向 1 号 +1（0→1：基线 2，0 有装备 -1 → 1；无变化）
    {
        let mut snap = e.save();
        // 青釭在 0 号装备区——转注前先移除（全区域 purge）
        for p in snap.core.players.iter_mut() {
            p.equips.retain(|c| *c != qg);
            p.hand.retain(|c| *c != qg);
        }
        snap.core.deck.retain(|c| *c != qg);
        snap.core.discard.retain(|c| *c != qg);
        snap.core.table.retain(|c| *c != qg);
        snap.core.players[1].equips.push(qg);
        e = Engine::restore_with_extensions(&snap, &[&pack]).expect("青釭转注");
    }
    // 0 号已无装备 → 车阵攻侧条件（有装备）不成立：0→2 无 -1（1+1=2）
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 2),
        2,
        "0 号卸装备：攻侧 -1 消失（守侧 +1 因 2 有装备不触发）"
    );
    // 0→1：1 号有装备 → 0 号守侧不触发；0 号无装备 → 攻侧不触发 → 基线 1
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 0, 1),
        1,
        "0→1：双侧条件均不成立"
    );
    // 2→1：1 号有装备 → 0 号守侧不涉及；基线 1
    assert_eq!(
        e.state_core().computed_distance(e.catalog(), 2, 1),
        1,
        "2→1：1 号有装备无守侧加成"
    );
}
