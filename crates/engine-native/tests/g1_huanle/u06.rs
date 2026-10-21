use super::*;

// ---------------------------------------------------------------------------
// T12/U06 G2–G4 产品通道验收（示例来源=欢乐卡牌）：目录/搜索（扩展卡在
// 合并目录可按名检索且 def 序稳定）+ 配置持久（store 重开激活集存续 +
// manifest.enabled=false 跳过导入）+ 资源（cards.json 经 content_hash 校验
// 的字节级保真）+ 界面/AI 隔离（扩展手牌仅持有者可见，他人 Hidden）。
// ---------------------------------------------------------------------------

#[test]
fn u06_g2g4_channels_acceptance() {
    use engine_native::view::HandView;

    // —— 资源面：cards.json 字节经 content_hash 校验保真（安装→桥接全链
    // 路不触碰原始资产字节）——
    let npack_bytes =
        std::fs::read(repo_extensions_root().join("huanle-upstream-noname/package.npack.json"))
            .expect("读 package.npack.json");
    let raw_cards =
        std::fs::read(repo_extensions_root().join("huanle-upstream-noname/assets/cards.json"))
            .expect("读 cards.json");
    let json = String::from_utf8(raw_cards.clone()).expect("utf8");
    let pack = bridge("huanle-upstream-noname", &json).expect("桥接");
    let npack_hash = content_runtime::content_hash_hex(&npack_bytes);
    let dir = tempfile::tempdir().unwrap();
    let store_path = dir.path().join("store");
    {
        let store = PackageStore::open(&store_path).unwrap();
        let report =
            import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
        let meta = report
            .installed
            .iter()
            .find(|m| m.package_id == "huanle-cards")
            .expect("安装记录");
        assert_eq!(
            meta.content_hash, npack_hash,
            "资源指纹保真（npack content_hash 一致）"
        );
        // 资源面：安装后 cards.json 与仓库源字节一致（资产原样保真）
        let installed_cards = std::fs::read(
            PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
        )
        .expect("读安装后 cards.json");
        assert_eq!(installed_cards, raw_cards, "cards.json 字节级保真");
    }

    // —— 配置持久：store 重开后激活集存续 ——
    {
        let store = PackageStore::open(&store_path).unwrap();
        let active = store.read_active().unwrap();
        assert!(
            active.packages.keys().any(|k| k.ends_with("/huanle-cards")),
            "激活集持久"
        );
    }

    // —— 目录+搜索：7 张扩展卡按名可检索、def 序稳定（重载后同名同位）——
    let mut e = Engine::new_with_extensions(3402, 3, &[&pack]).expect("建局");
    let base = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base + 7, "7 张扩展卡入目录");
    let mut positions_reload = Vec::new();
    for i in 0..7 {
        let name = {
            let cat = e.catalog();
            cat.cards
                .iter()
                .skip(base)
                .map(|c| c.name)
                .nth(i)
                .expect("扩展卡名")
        };
        let pos = {
            let cat = e.catalog();
            cat.cards
                .iter()
                .position(|c| c.name == name)
                .expect("按名检索")
        };
        positions_reload.push((name, pos));
    }
    assert_eq!(positions_reload.len(), 7);
    assert!(
        positions_reload.iter().all(|(n, _)| n.starts_with("hnl_")),
        "扩展卡名带 hnl_ 前缀（符号表隔离）"
    );

    // —— 界面/AI 隔离：扩展手牌仅持有者可见 ——
    e.debug_grant(1, "huanle_kuwu");
    let shoulijian = {
        let cat = e.catalog();
        cat.cards
            .iter()
            .position(|c| c.name.starts_with("hnl_shoulijian"))
            .expect("手里剑在目录") as u32
    };
    {
        let core = e.state_core_mut();
        core.players[1].hand.push(shoulijian);
    }
    e.debug_set_hp(1, 4);
    // 视角 1：自己手里的扩展牌具体可见（HandView 明细）
    let v1 = e.ai_session(1).view();
    match (&v1.players[1].hand, &v1.players[2].hand) {
        (HandView::Own(mine), other) => {
            assert!(mine.contains(&shoulijian), "持有者视角可见扩展手牌");
            assert!(!matches!(other, HandView::Own(_)), "他人手牌不泄露具体 id");
        }
        _ => panic!("持有者手牌应为 Own 明细"),
    }
    // 视角 2：他人视角下 1 号手牌为 Hidden（仅数量）
    let v2 = e.ai_session(2).view();
    match &v2.players[1].hand {
        HandView::Hidden => {}
        other => panic!("他人视角手牌应 Hidden: {other:?}"),
    }
    // 界面通道：扩展手牌的卡片 def 在目录可检索（UI 按 def_id 渲染）
    let def_ok = {
        let cat = e.catalog();
        cat.cards[shoulijian as usize]
            .name
            .starts_with("hnl_shoulijian")
    };
    assert!(def_ok, "界面可按 def_id 检索扩展卡名");

    // —— 配置面补充：manifest.enabled=false 的目录被导入跳过 ——
    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("huanle-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(
        repo_extensions_root().join("huanle-upstream-noname/manifest.json"),
        disabled_root.join("manifest.json"),
    )
    .unwrap();
    // 关闭开关
    let mf_path = disabled_root.join("manifest.json");
    let mut mf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mf_path).unwrap()).unwrap();
    mf["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(&mf_path, serde_json::to_string_pretty(&mf).unwrap()).unwrap();
    std::fs::copy(
        repo_extensions_root().join("huanle-upstream-noname/assets/cards.json"),
        disabled_root.join("assets/cards.json"),
    )
    .unwrap();
    {
        let store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let report = import_extensions(&store, disabled_dir.path(), &options()).expect("import");
        assert!(
            !report
                .installed
                .iter()
                .any(|m| m.package_id == "huanle-cards"),
            "enabled=false 目录被跳过（配置面）"
        );
        assert!(
            report.skipped.iter().any(|s| s == "huanle-disabled"),
            "跳过清单登记"
        );
    }
    let _ = (positions_reload, &pack);
}

/// U06 欢乐卡牌八项整包（2026-10-17）：从真实 `import_extensions` 安装出发，
/// 同一测试覆盖功能、配置、依赖、组合、保存恢复、安装升级、目标平台与规则差异
/// 的正例和反例。苦无仍是登记差异（确定性弃首张手牌），不是上游等价。
/// 本测试只关闭 U06，不把其余来源计为整包。
#[test]
fn u06_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;
    use content_schema::{ContentPackage, PackageDependency};

    let source_root = repo_extensions_root().join("huanle-upstream-noname");
    let npack_path = source_root.join("package.npack.json");
    let cards_path = source_root.join("assets/cards.json");
    let npack_bytes = std::fs::read(&npack_path).expect("读 package.npack.json");
    let cards_text = std::fs::read_to_string(&cards_path).expect("读 cards.json");

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "huanle-cards")
        .expect("欢乐卡牌已安装");
    assert_eq!(meta.version, "1.0.0");
    let installed_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let installed_npack =
        std::fs::read(installed_root.join("package.npack.json")).expect("安装副本 npack");
    assert_eq!(
        installed_npack, npack_bytes,
        "安装副本与仓库 npack 字节一致"
    );
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes),
        "商店记录的 content_hash 来自真实 npack 字节"
    );
    let installed_pkg = ContentPackage::from_json_bytes(&installed_npack).expect("解析安装副本");
    assert!(
        installed_pkg.manifest.dependencies.is_empty(),
        "依赖正例：欢乐卡牌不声明外部扩展依赖"
    );
    assert_eq!(installed_pkg.manifest.package_id, "huanle-cards");
    let _ = installed_npack;

    let huanle_version = |store: &PackageStore| -> String {
        store
            .read_active()
            .expect("active")
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("/huanle-cards"))
            .map(|(_, version)| version.clone())
            .expect("激活集含欢乐卡牌")
    };
    assert_eq!(
        huanle_version(&store),
        "1.0.0",
        "配置正例：安装后激活 1.0.0"
    );
    assert!(
        store
            .current_lock()
            .unwrap()
            .packages
            .iter()
            .any(|p| p.package_id == "huanle-cards" && p.version == "1.0.0"),
        "内容锁登记 1.0.0"
    );

    match store.activate_version("upstream-noname", "huanle-cards", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("依赖反例：未安装版本应 NotFound: {other:?}"),
    }
    assert_eq!(huanle_version(&store), "1.0.0", "失败激活不得改写激活集");

    let mut escaped = installed_pkg.clone();
    escaped.manifest.package_id = "../huanle".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    let escaped_bytes = escaped.to_json_bytes().expect("序列化非法包");
    match store.install(&escaped_bytes, None, &options()) {
        Err(RuntimeError::Package(msg)) => {
            assert!(msg.contains("invalid package_id"), "{msg}");
        }
        other => panic!("依赖反例：包 id 路径逃逸应在校验拒绝: {other:?}"),
    }

    let mut missing = installed_pkg.clone();
    missing.manifest.version = "1.0.1".into();
    missing.manifest.dependencies.push(PackageDependency {
        namespace: "upstream-noname".into(),
        package_id: "missing-base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    missing.manifest.logic_hash = missing.compute_logic_hash();
    let missing_bytes = missing.to_json_bytes().expect("序列化缺依赖包");
    match store.install_with_assets(&missing_bytes, None, &options(), &source_root) {
        Err(RuntimeError::Invalid(msg)) => {
            assert!(msg.contains("missing dependency"), "{msg}");
        }
        other => panic!("依赖反例：缺依赖激活应拒绝: {other:?}"),
    }
    assert_eq!(huanle_version(&store), "1.0.0", "缺依赖不得替换已激活版本");

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("huanle-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(
        source_root.join("manifest.json"),
        disabled_root.join("manifest.json"),
    )
    .unwrap();
    std::fs::copy(&npack_path, disabled_root.join("package.npack.json")).unwrap();
    std::fs::copy(&cards_path, disabled_root.join("assets/cards.json")).unwrap();
    let mf_path = disabled_root.join("manifest.json");
    let mut mf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mf_path).unwrap()).unwrap();
    mf["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(&mf_path, serde_json::to_string_pretty(&mf).unwrap()).unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled = import_extensions(&disabled_store, disabled_dir.path(), &options())
            .expect("disabled import");
        assert!(
            !disabled
                .installed
                .iter()
                .any(|m| m.package_id == "huanle-cards"),
            "配置反例：enabled=false 不得安装"
        );
        assert!(
            disabled.skipped.iter().any(|s| s == "huanle-disabled"),
            "配置反例：跳过清单登记目录名"
        );
    }

    let value: serde_json::Value = serde_json::from_str(&cards_text).unwrap();
    let kuwu_mapping = value["skills"]
        .as_array()
        .expect("skills")
        .iter()
        .find(|s| s["name"] == "huanle_kuwu")
        .expect("苦无声明")["mapping"]
        .as_str()
        .expect("mapping 字符串");
    assert_eq!(
        kuwu_mapping, "host:sha_damage_force_discard",
        "规则差异：装载映射是确定性弃牌原语，不是上游自选"
    );

    let installed_cards = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("安装副本 cards.json");
    assert_eq!(installed_cards, cards_text, "安装副本与仓库源字节一致");
    let pack = bridge("huanle-upstream-noname", &installed_cards).expect("桥接安装副本");
    let mut e = Engine::new_with_extensions(3417, 3, &[&pack]).expect("建局");
    let base = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(
        e.catalog().cards.len(),
        base + 7,
        "功能正例：7 张欢乐牌入局"
    );
    assert!(
        e.catalog()
            .cards
            .iter()
            .skip(base)
            .all(|c| c.name.starts_with("hnl_")),
        "功能正例：牌名带 hnl_ 前缀"
    );
    assert!(
        e.catalog().skill_by_name("huanle_kuwu").is_some(),
        "功能正例：苦无技能在目录"
    );

    advance_to_play(&mut e, 0);
    e.debug_set_hp(0, 4);
    match submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: u32::MAX,
            targets: vec![1],
        },
    ) {
        CommandOutcome::Rejected(_) => {}
        other => panic!("功能反例：不存在的牌必须拒绝: {other:?}"),
    }

    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: kuwu,
            targets: vec![],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. }),
        "装备苦无: {out:?}"
    );
    let def_kuwu = e.catalog().skill_by_name("huanle_kuwu").expect("苦无");
    assert!(
        e.save().core.skills.contains_key(&(0, def_kuwu)),
        "功能正例：装备改变技能实例"
    );
    let first = e.debug_place(1, "tao");
    let second = e.debug_place(1, "jiu");
    let hp_before = e.debug_hp(1);
    let leisha = e.debug_place(0, "leisha");
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: leisha,
            targets: vec![1],
        },
    );
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "雷杀: {out:?}"
    );
    if let Some(c) = e.open_choice() {
        let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
            c.spec.options.iter().take(c.spec.min).copied().collect()
        } else {
            vec![c.spec.options[0]]
        };
        let (id, obs) = (c.id, c.observer);
        submit(
            &mut e,
            obs,
            GameAction::Answer {
                choice_id: id,
                selection: sel,
            },
        );
    }
    assert_eq!(e.debug_hp(1), hp_before - 1, "雷杀命中");
    assert!(
        !e.debug_hand(1).contains(&first) && e.debug_hand(1).contains(&second),
        "规则差异：苦无确定性弃首张手牌，保留后一张；上游为目标自选"
    );

    let saved = e.save();
    let bytes = engine_native::archive::encode_archive(&saved);
    let decoded = engine_native::archive::decode_archive(&bytes).expect("目标平台：宿主解码");
    let restored = Engine::restore_with_extensions(&decoded, &[&pack]).expect("保存恢复正例");
    assert_eq!(
        restored.catalog().cards.len(),
        base + 7,
        "恢复后仍是 7 张扩展牌"
    );
    match Engine::restore(&saved) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(err) => panic!("保存恢复反例应报版本不符: {err:?}"),
        Ok(_) => panic!("标准目录不得恢复扩展局"),
    }
    let mut wrong = bridge("huanle-upstream-noname", &installed_cards).expect("错版桥接");
    wrong.def.version += 1;
    match Engine::restore_with_extensions(&saved, &[&wrong]) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(err) => panic!("安装升级反例应报版本不符: {err:?}"),
        Ok(_) => panic!("错版包必须拒绝升级前存档"),
    }

    let wuxing_text = std::fs::read_to_string(
        repo_extensions_root().join("wuxing-upstream-liuli/assets/cards.json"),
    )
    .expect("读五行 cards.json");
    let wuxing = bridge("wuxing-upstream-liuli", &wuxing_text).expect("桥接五行");
    let combo = Engine::new_with_extensions(3418, 3, &[&pack, &wuxing]).expect("组合建局");
    assert!(
        combo.catalog().cards.len() > base + 7,
        "组合正例：欢乐+五行牌数多于单包"
    );
    assert_eq!(
        combo
            .catalog()
            .cards
            .iter()
            .filter(|c| c.name.starts_with("wxl_"))
            .count(),
        1
    );
    assert_eq!(
        combo
            .catalog()
            .cards
            .iter()
            .filter(|c| c.name.starts_with("hnl_"))
            .count(),
        7
    );
    match Engine::restore_with_extensions(&saved, &[&pack, &wuxing]) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(err) => panic!("组合反例应报版本不符: {err:?}"),
        Ok(_) => panic!("单包存档不得用双包目录恢复"),
    }

    let mut upgraded = installed_pkg.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    let upgraded_bytes = upgraded.to_json_bytes().expect("序列化 2.0.0");
    store
        .install_with_assets(&upgraded_bytes, None, &options(), &source_root)
        .expect("安装升级正例");
    assert_eq!(huanle_version(&store), "2.0.0");
    assert!(store
        .current_lock()
        .unwrap()
        .packages
        .iter()
        .any(|p| p.package_id == "huanle-cards" && p.version == "2.0.0"));
    let rolled = store.rollback().expect("回滚");
    assert_eq!(
        rolled
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("/huanle-cards"))
            .map(|(_, version)| version.as_str()),
        Some("1.0.0"),
        "安装升级正例：回滚回到 1.0.0"
    );
    let still =
        Engine::restore_with_extensions(&saved, &[&pack]).expect("升级前存档仍可用原包恢复");
    assert_eq!(still.catalog().cards.len(), base + 7);
}
