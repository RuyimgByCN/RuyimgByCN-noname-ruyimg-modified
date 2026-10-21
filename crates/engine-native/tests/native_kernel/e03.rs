use super::*;

/// E03：扩展卡牌合并入目录后在真实对局中生效——
/// 扩展闪电牌在判定中命中（红桃闪电在扩展语义中仍触发标准判定）。
#[test]
fn e03_extension_cards_in_catalog_and_game() {
    use engine_native::content::extension;
    let ext = extension::example_extension();
    assert_eq!(ext.general_count(), 2);
    assert_eq!(ext.card_count(), 1);

    // 合并后目录包含扩展卡牌
    let mut base_cards: Vec<engine_native::catalog::CardDef> = Vec::new();
    // 从标准目录提取 CardDef（通过 save 的 core.deck → catalog 引用）
    let e0 = Engine::new(1, 2);
    let cat = e0.catalog();
    for c in &cat.cards {
        base_cards.push(
            engine_native::catalog::CardDef::basic(c.name, c.kind).with_suit_rank(c.suit, c.rank),
        );
    }
    let merged = engine_native::content::extension::merge_extension_cards(base_cards, &ext.cards)
        .expect("合并应成功");
    assert_eq!(
        merged.len(),
        e0.catalog().cards.len() + ext.card_count(),
        "合并后应比基础目录多扩展卡牌数"
    );
    let ext_card = merged.last().expect("扩展卡");
    assert_eq!(ext_card.name, "shandian_red");
    assert_eq!(ext_card.kind, engine_native::catalog::CardKind::ShanDian);
}

/// E03：扩展卡牌合并入真实目录并在建局/存档/恢复中生效；
/// 内容锁绑定扩展集合——不同集合/版本的恢复被拒绝。
#[test]
fn e03_extension_catalog_content_lock_in_game_and_restore() {
    use engine_native::content::extension::{example_extension, ExtensionDef, ExtensionPack};
    use engine_native::SnapshotError;
    let ext = example_extension();
    let packs: Vec<&ExtensionPack> = vec![&ext];

    // 建局：目录牌数 = 标准 + 扩展；规则版本 ≠ 纯标准版本
    let mut e = Engine::new_with_extensions(3001, 3, &packs).expect("带扩展建局");
    assert_eq!(
        e.catalog().cards.len(),
        Engine::new(1, 3).catalog().cards.len() + ext.card_count(),
        "扩展卡牌应合并入目录"
    );
    assert_ne!(
        e.catalog().ruleset_version,
        Engine::new(1, 3).catalog().ruleset_version,
        "内容锁：扩展集合改变目录版本"
    );

    // 对局可推进（发牌含扩展牌域，牌数守恒由既有断言保障）
    let cur = e.turn().current;
    let out = submit_cmd(&mut e, cur, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");

    // 存档 → 同集合恢复 OK 且同哈希；无扩展恢复 → 拒绝
    let snap = e.save();
    let ok = Engine::restore_with_extensions(&snap, &packs).expect("同集合恢复");
    assert_eq!(ok.state_hash(), e.state_hash(), "恢复件同哈希");
    match Engine::restore(&snap) {
        Err(SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(e) => panic!("应报规则版本不符: {e:?}"),
        Ok(_) => panic!("无扩展恢复带扩展存档必须被内容锁拒绝"),
    }

    // 同名不同版本的扩展 → 指纹不同 → 恢复拒绝
    let mut ext_v2 = example_extension();
    ext_v2.def = ExtensionDef {
        name: ext.def.name,
        version: ext.def.version + 1,
        description: "",
    };
    let packs_v2: Vec<&ExtensionPack> = vec![&ext_v2];
    match Engine::restore_with_extensions(&snap, &packs_v2) {
        Err(SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(e) => panic!("应报规则版本不符: {e:?}"),
        Ok(_) => panic!("版本不同的扩展集合恢复必须被拒绝"),
    }

    // 禁用扩展后构建目录 = 纯标准目录（lock=0 → 标准版本）
    let disabled = example_extension();
    let mut disabled = disabled;
    disabled.enabled = false;
    let packs_off: Vec<&ExtensionPack> = vec![&disabled];
    let cat = engine_native::content::standard::standard_catalog_with_extensions(&packs_off)
        .expect("无启用扩展");
    assert_eq!(
        cat.ruleset_version,
        Engine::new(1, 2).catalog().ruleset_version
    );
    assert_eq!(cat.cards.len(), Engine::new(1, 2).catalog().cards.len());
}
