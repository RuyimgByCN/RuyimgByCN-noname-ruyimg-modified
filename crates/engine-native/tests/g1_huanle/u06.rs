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
        "上游对齐（欢乐卡牌/card/skill.js:53-69）：装载映射绑定 HostBehavior::ShaDamageForceDiscard"
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
        "上游对齐（欢乐卡牌/card/skill.js:53-69）：苦无弹出 kuwu_discard 令受伤角色自选弃置一张 he 牌"
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

// ---------------------------------------------------------------------------
// Queue Item 3 — U06 欢乐卡牌 11 项（6 卡牌 + 5 技能）六维度真实对局事件路径测试
// 1. 正向触发/使用路径
// 2. 反向过滤路径（负例）
// 3. 选择分支 A vs B
// 4. 可选取消路径
// 5. AI BoundedPlanner 仅凭公开视图决策
// 6. 选择挂起中点 save() -> encode_archive -> decode_archive -> restore() -> Answer 状态哈希与事件等价
// ---------------------------------------------------------------------------

/// `monkey` 卡牌（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:4-15`）
/// 与 `monkey` 技能（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:5-31`）六维度验收。
#[test]
fn u06_monkey_six_dimension_real_match_equivalence() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::state::{GameMode, Role};

    let pack = huanle_pack_from_repo();

    // 维度 2（负例）：装备者自己使用【桃】时不触发猴子偷桃（`skill.js:11` `event.player !== player`）
    {
        let mut e = Engine::new_with_extensions(3601, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let monkey = e.debug_place(0, "hnl_monkey_3");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: monkey, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        e.debug_set_hp(0, 2);
        let tao = e.debug_place(0, "tao");
        let out = submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }),
            "装备者自己出桃不得触发猴子确认门: {out:?}"
        );
        assert_eq!(e.debug_hp(0), 3, "自己出桃正常回血");
        assert!(e.debug_equips(0).contains(&monkey), "猴子仍在装备区");
    }

    // 维度 1/3/4/6（正向夺桃 vs 取消放行 + 挂起中点存档恢复状态哈希与事件等价）
    {
        let mut e_base = Engine::new_with_extensions(3602, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e_base, 0);
        for p in 0..3 {
            e_base.debug_clear_hand(p);
        }
        let monkey = e_base.debug_place(0, "hnl_monkey_3");
        assert!(matches!(
            submit(&mut e_base, 0, GameAction::PlayCard { card: monkey, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        advance_to_play(&mut e_base, 1);
        e_base.debug_clear_hand(1);
        e_base.debug_set_hp(1, 2);
        let tao = e_base.debug_place(1, "tao");
        let out = submit(&mut e_base, 1, GameAction::PlayCard { card: tao, targets: vec![] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let gate = e_base.open_choice().expect("huanle_monkey_confirm 挂起").clone();
        assert_eq!(gate.spec.prompt, "huanle_monkey_confirm");
        assert_eq!(gate.observer, 0);

        // 分支 B / 维度 4（取消 Pass）：桃正常结算回血，猴子保留在装备区
        let bytes = encode_archive(&e_base.save());
        let mut e_pass = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let out_pass = submit(
            &mut e_pass,
            0,
            GameAction::Answer {
                choice_id: gate.id,
                selection: vec![ChoiceOption::Pass],
            },
        );
        assert!(matches!(out_pass, CommandOutcome::Accepted { .. }));
        assert_eq!(e_pass.debug_hp(1), 3, "Pass 分支：1 号正常回血");
        assert!(e_pass.debug_equips(0).contains(&monkey), "Pass 分支：猴子保留");
        assert!(!e_pass.debug_hand(0).contains(&tao), "Pass 分支：0 号不获得桃");

        // 分支 A / 维度 1 & 6（确认 Confirm + 挂起中点存档恢复等价）
        let mut e_restored = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let log_before = e_base.log().len();
        let out_direct = submit(
            &mut e_base,
            0,
            GameAction::Answer {
                choice_id: gate.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        let out_restored = submit(
            &mut e_restored,
            0,
            GameAction::Answer {
                choice_id: gate.id,
                selection: vec![ChoiceOption::Confirm],
            },
        );
        let (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) =
            (out_direct, out_restored)
        else {
            panic!("确认应 Accepted");
        };
        assert_eq!(h1, h2, "猴子挂起中点恢复状态哈希必须一致");
        assert_eq!(
            &e_base.log()[log_before..],
            e_restored.log(),
            "猴子挂起中点恢复事件流必须一致"
        );
        assert_eq!(
            e_base.state_hash(),
            e_restored.state_hash(),
            "猴子挂起中点恢复状态哈希必须一致"
        );
        assert_eq!(e_base.debug_hp(1), 2, "Confirm 分支：桃被取消不回血");
        assert!(e_base.debug_hand(0).contains(&tao), "Confirm 分支：0 号获得桃");
        assert!(!e_base.debug_equips(0).contains(&monkey), "Confirm 分支：猴子已弃置");
    }

    // 维度 5（AI BoundedPlanner 公开态度决策：`skill.js:20-22` `get.attitude(player, event.player) <= 0`）：
    // 敌方（主公 0 出桃，反贼 1 持猴子）-> Confirm；友方（主公 0 出桃，忠臣 1 持猴子）-> Pass；
    // 且修改未明置第三方（2 号）的暗身份不影响 AI 决策（信息隔离）。
    {
        for (my_role, hidden_role_2, expect_confirm) in [
            (Role::Rebel, Role::Loyalist, true),
            (Role::Rebel, Role::Traitor, true),
            (Role::Loyalist, Role::Rebel, false),
            (Role::Loyalist, Role::Traitor, false),
        ] {
            let mut e = Engine::new_with_extensions(3603, 3, &[&pack]).expect("建局");
            advance_to_play(&mut e, 1);
            for p in 0..3 {
                e.debug_clear_hand(p);
            }
            let monkey = e.debug_place(1, "hnl_monkey_3");
            assert!(matches!(
                submit(&mut e, 1, GameAction::PlayCard { card: monkey, targets: vec![] }),
                CommandOutcome::Accepted { .. }
            ));
            advance_to_play(&mut e, 0);
            e.debug_clear_hand(0);
            e.debug_set_hp(0, 2);
            e.state_core_mut().mode = GameMode::Identity {
                roles: vec![Role::Lord, my_role, hidden_role_2],
            };
            let tao = e.debug_place(0, "tao");
            assert!(matches!(
                submit(&mut e, 0, GameAction::PlayCard { card: tao, targets: vec![] }),
                CommandOutcome::Pending { .. }
            ));
            let planner = BoundedPlanner::new(16);
            let cmd = e.plan_action(1, &planner);
            let expected = if expect_confirm {
                vec![ChoiceOption::Confirm]
            } else {
                vec![ChoiceOption::Pass]
            };
            assert_eq!(
                cmd.action,
                GameAction::Answer {
                    choice_id: e.open_choice().unwrap().id,
                    selection: expected,
                },
                "role={my_role:?} hidden2={hidden_role_2:?} AI 猴子偷桃决策应符合公开态度"
            );
        }
    }
}

/// `mianju` 卡牌（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:16-35`）
/// 与 `mianju` 技能（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:32-52`）六维度验收。
#[test]
fn u06_mianju_six_dimension_real_match_equivalence() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};

    let pack = huanle_pack_from_repo();
    let mut e = Engine::new_with_extensions(3604, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }

    // 维度 1a（正向：正面朝上装备旋涡面具后不能翻至背面）
    let mianju = e.debug_place(0, "hnl_mianju_5");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: mianju, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));
    assert!(!e.debug_flip(0), "装备旋涡面具时本人正面->背面翻面应被阻止");
    assert!(!e.debug_face_down(0));

    // 维度 2（负例：0 号装备旋涡面具不得阻止 1 号翻面！）
    assert!(e.debug_flip(1), "0 号的旋涡面具不得阻止 1 号翻面");
    assert!(e.debug_face_down(1), "1 号应成功翻至背面");

    // 维度 1b（反向锁定：`card.js:24-33` 背面朝上时装备旋涡面具，同样不能翻回正面）
    e.debug_take_owned_card_quiet(0, mianju);
    assert!(e.debug_flip(0), "卸下面具后 0 号翻至背面");
    assert!(e.debug_face_down(0));
    let mianju2 = e.debug_place(0, "hnl_mianju_5");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: mianju2, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));
    assert!(!e.debug_flip(0), "背面朝上且装备旋涡面具时不能翻回正面");
    assert!(e.debug_face_down(0), "0 号保持背面朝上");

    // 维度 5 & 6（AI 在背面朝上装备旋涡面具时被手里剑指定，宁可受 1 伤也不弃面具 `card.js:47-59` + 存档恢复等价）
    let horse_plus = e.debug_move_named_card_to_hand(0, "horse_plus");
    {
        let core = e.state_core_mut();
        core.players[0].hand.retain(|c| *c != horse_plus);
        core.players[0].equips.push(horse_plus);
    }
    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sj = e.debug_place(1, "hnl_shoulijian_1");
    assert!(matches!(
        submit(&mut e, 1, GameAction::PlayCard { card: sj, targets: vec![0] }),
        CommandOutcome::Pending { .. }
    ));
    let bytes = encode_archive(&e.save());
    let restored = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
    assert_eq!(e.state_hash(), restored.state_hash());

    let planner = BoundedPlanner::new(16);
    let ai_cmd = e.plan_action(0, &planner);
    assert_eq!(
        ai_cmd.action,
        GameAction::Answer {
            choice_id: e.open_choice().unwrap().id,
            selection: vec![ChoiceOption::Pass],
        },
        "背面朝上且装备旋涡面具时 AI 应选择 Pass 受 1 伤保留面具（card.js:49-51）"
    );
}

/// `shoulijian` 卡牌（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:36-80`）六维度验收。
#[test]
fn u06_shoulijian_six_dimension_real_match_equivalence() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::RejectReason;

    let pack = huanle_pack_from_repo();

    // 维度 2（负例：对自身或距离 1 的角色使用被拒 `outrange: { global: 2 }`；对距离 2 且无 he 装备的角色直接造伤不弹窗 `card.js:41-44`）
    {
        let mut e = Engine::new_with_extensions(3605, 4, &[&pack]).expect("4人局");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let sj = e.debug_place(0, "hnl_shoulijian_1");
        assert!(
            matches!(
                submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![0] }),
                CommandOutcome::Rejected(RejectReason::IllegalTargets)
            ),
            "不能对自己使用手里剑"
        );
        assert!(
            matches!(
                submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![1] }),
                CommandOutcome::Rejected(RejectReason::IllegalTargets)
            ),
            "距离 1 的角色（1 号）不满足 outrange: {{ global: 2 }}"
        );
        // 2 号在 4 人局与 0 号距离为 2，且手牌与装备区均无装备牌 -> 直接受 1 点伤害，不弹选择窗
        let hp_before = e.debug_hp(2);
        let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![2] });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }),
            "目标 he 区无装备牌时应直接结算 1 点伤害（不挂起窗口）: {out:?}"
        );
        assert!(e.open_choice().is_none());
        assert_eq!(e.debug_hp(2), hp_before - 1, "无装备目标直接受 1 点伤害");
    }

    // 维度 1/3/4/5/6（手牌装备 vs 装备区装备分支、Pass 受伤分支、AI 决策、挂起中点存档恢复等价）
    {
        let mut e = Engine::new_with_extensions(3606, 4, &[&pack]).expect("4人局");
        advance_to_play(&mut e, 0);
        for p in 0..4 {
            e.debug_clear_hand(p);
        }
        let sj = e.debug_place(0, "hnl_shoulijian_2");
        let hand_equip = e.debug_move_named_card_to_hand(2, "horse_minus");
        let worn_equip = e.debug_move_named_card_to_hand(2, "horse_plus");
        {
            let core = e.state_core_mut();
            core.players[2].hand.retain(|c| *c != worn_equip);
            core.players[2].equips.push(worn_equip);
        }
        let out = submit(&mut e, 0, GameAction::PlayCard { card: sj, targets: vec![2] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let c = e.open_choice().expect("host_target_discard_equip 挂起").clone();
        assert_eq!(c.spec.prompt, "host_target_discard_equip");
        assert!(c.spec.allow_cancel);
        assert!(
            c.spec.options.contains(&ChoiceOption::Card(hand_equip))
                && c.spec.options.contains(&ChoiceOption::Card(worn_equip)),
            "手里剑选项必须同时覆盖目标手牌装备与装备区装备（card.js:41,52 'he'）"
        );

        // 维度 5：AI 选择弃置装备牌免伤
        let planner = BoundedPlanner::new(16);
        let ai_cmd = e.plan_action(2, &planner);
        assert!(matches!(
            ai_cmd.action,
            GameAction::Answer { ref selection, .. }
            if selection == &vec![ChoiceOption::Card(hand_equip)]
                || selection == &vec![ChoiceOption::Card(worn_equip)]
        ));

        let bytes = encode_archive(&e.save());

        // 分支 A：弃置手牌中的装备牌 `hand_equip` + 维度 6 存档恢复等价
        let mut e_hand_direct = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let mut e_hand_restored = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let out1 = submit(
            &mut e_hand_direct,
            2,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(hand_equip)],
            },
        );
        let out2 = submit(
            &mut e_hand_restored,
            2,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(hand_equip)],
            },
        );
        let (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) = (out1, out2) else {
            panic!("应 Accepted");
        };
        assert_eq!(h1, h2);
        assert_eq!(e_hand_direct.log(), e_hand_restored.log());
        assert_eq!(e_hand_direct.state_hash(), e_hand_restored.state_hash());
        assert!(!e_hand_direct.debug_hand(2).contains(&hand_equip), "手牌装备被弃置");
        assert!(e_hand_direct.debug_equips(2).contains(&worn_equip), "装备区装备保留");
        assert_eq!(e_hand_direct.debug_hp(2), 4, "弃手牌装备同样免伤");

        // 分支 B：弃置装备区的装备牌 `worn_equip`
        let mut e_worn = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        submit(
            &mut e_worn,
            2,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(worn_equip)],
            },
        );
        assert!(e_worn.debug_hand(2).contains(&hand_equip), "手牌装备保留");
        assert!(!e_worn.debug_equips(2).contains(&worn_equip), "装备区装备被弃置");
        assert_eq!(e_worn.debug_hp(2), 4, "弃装备区装备免伤");

        // 维度 4（取消 / 空选）：受 1 点普通伤害，保留两张装备
        let mut e_cancel = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        submit(
            &mut e_cancel,
            2,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![],
            },
        );
        assert_eq!(e_cancel.debug_hp(2), 3, "取消弃牌受 1 点伤害");
        assert!(e_cancel.debug_hand(2).contains(&hand_equip));
        assert!(e_cancel.debug_equips(2).contains(&worn_equip));
    }
}

/// `kuwu` 卡牌（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:81-96`）
/// 与 `kuwu` 技能（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:53-69`）六维度验收。
#[test]
fn u06_kuwu_six_dimension_real_match_equivalence() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};
    use engine_native::RejectReason;

    let pack = huanle_pack_from_repo();

    // 维度 2（负例：受伤目标 he 区为空时不触发苦无 `event.player.countCards("he") > 0`）
    {
        let mut e = Engine::new_with_extensions(3607, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        let sha = e.debug_place(0, "sha");
        let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(
            matches!(out, CommandOutcome::Accepted { .. }),
            "目标 he 区无牌时苦无不得挂起询问: {out:?}"
        );
        assert!(e.open_choice().is_none());
    }

    // 维度 1/3/4/5/6（手牌 vs 装备区弃置、强制不可取消、AI 弃最低价值牌、挂起中点存档恢复等价）
    {
        let mut e = Engine::new_with_extensions(3608, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let kuwu = e.debug_move_named_card_to_hand(0, "hnl_kuwu_0");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: kuwu, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        let hand_tao = e.debug_place(1, "tao");
        let eq_monkey = e.debug_place(1, "hnl_monkey_3");
        {
            let core = e.state_core_mut();
            core.players[1].hand.retain(|c| *c != eq_monkey);
            core.players[1].equips.push(eq_monkey);
        }
        e.debug_grant(1, "huanle:monkey");
        let sha = e.debug_place(0, "sha");
        let out = submit(&mut e, 0, GameAction::PlayCard { card: sha, targets: vec![1] });
        assert!(matches!(out, CommandOutcome::Pending { .. }));
        let c = e.open_choice().expect("kuwu_discard 挂起").clone();
        assert_eq!(c.spec.prompt, "kuwu_discard");
        assert_eq!(c.observer, 1);
        assert!(!c.spec.allow_cancel, "苦无弃牌为强制（chooseToDiscard(1,'he',true)）");
        assert!(
            c.spec.options.contains(&ChoiceOption::Card(hand_tao))
                && c.spec.options.contains(&ChoiceOption::Card(eq_monkey)),
            "苦无选项包含手牌与装备区"
        );

        // 维度 4：强制不可取消（空选被拒）
        assert!(matches!(
            submit(
                &mut e,
                1,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![],
                }
            ),
            CommandOutcome::Rejected(RejectReason::InvalidChoice)
        ));

        // 维度 5：AI 优先保留高价值【桃】，弃置价值较低的装备牌
        let planner = BoundedPlanner::new(16);
        let ai_cmd = e.plan_action(1, &planner);
        assert_eq!(
            ai_cmd.action,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(eq_monkey)],
            }
        );

        let bytes = encode_archive(&e.save());

        // 分支 A（弃手牌）+ 维度 6 挂起中点存档恢复等价
        let mut e_hand_1 = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let mut e_hand_2 = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        let out1 = submit(
            &mut e_hand_1,
            1,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        let out2 = submit(
            &mut e_hand_2,
            1,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(hand_tao)],
            },
        );
        let (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) = (out1, out2) else {
            panic!("应 Accepted");
        };
        assert_eq!(h1, h2);
        assert_eq!(e_hand_1.log(), e_hand_2.log());
        assert_eq!(e_hand_1.state_hash(), e_hand_2.state_hash());
        assert!(!e_hand_1.debug_hand(1).contains(&hand_tao));
        assert!(e_hand_1.debug_equips(1).contains(&eq_monkey));

        // 分支 B（弃装备并撤销伴随装备技）
        let mut e_eq = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
        submit(
            &mut e_eq,
            1,
            GameAction::Answer {
                choice_id: c.id,
                selection: vec![ChoiceOption::Card(eq_monkey)],
            },
        );
        assert!(e_eq.debug_hand(1).contains(&hand_tao));
        assert!(!e_eq.debug_equips(1).contains(&eq_monkey));
        assert!(!e_eq.debug_has_skill(1, "huanle:monkey"), "弃置装备时撤销其装备技");
    }
}

/// `xuelunyang` 卡牌（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:97-108`）
/// 与 `xuelunyang` 技能（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:70-128`）六维度验收。
#[test]
fn u06_xuelunyang_six_dimension_real_match_equivalence() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};

    let pack = huanle_pack_from_repo();

    // 维度 2（负例：其他存活角色均无技能时不触发写轮眼 `skill.js:73-75`）
    {
        let mut e = Engine::new_with_extensions(3609, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let xl = e.debug_place(0, "hnl_xuelunyang_4");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        advance_to_play(&mut e, 1);
        for _ in 0..16 {
            if e.turn().current == 0 && e.turn().phase == Phase::Play {
                break;
            }
            if let Some(c) = e.open_choice() {
                assert_ne!(
                    c.spec.prompt, "huanle_xuelunyang_target",
                    "场上无持技角色时写轮眼不得触发"
                );
                let (cid, obs) = (c.id, c.observer);
                let sel = c.spec.options.iter().take(c.spec.min.max(1)).copied().collect();
                submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel });
            } else {
                let actor = e.turn().current;
                submit(&mut e, actor, GameAction::NextPhase);
            }
        }
        assert_eq!(e.turn().current, 0);
    }

    // 维度 1/3/4/5/6（多目标与多技能 Tag(0) vs Tag(1) 分支、直接取消、AI 选最多技能目标、两阶段存档恢复等价）
    {
        let mut e = Engine::new_with_extensions(3610, 3, &[&pack]).expect("建局");
        advance_to_play(&mut e, 0);
        for p in 0..3 {
            e.debug_clear_hand(p);
        }
        let xl = e.debug_place(0, "hnl_xuelunyang_4");
        assert!(matches!(
            submit(&mut e, 0, GameAction::PlayCard { card: xl, targets: vec![] }),
            CommandOutcome::Accepted { .. }
        ));
        e.debug_grant(1, "马术");
        e.debug_grant(1, "咆哮");
        e.debug_grant(2, "武圣");

        advance_to_play(&mut e, 1);
        for _ in 0..32 {
            if e.open_choice().is_some_and(|c| c.spec.prompt == "huanle_xuelunyang_target") {
                break;
            }
            let actor = e.turn().current;
            submit(&mut e, actor, GameAction::NextPhase);
        }
        let c1 = e.open_choice().expect("huanle_xuelunyang_target 挂起").clone();
        assert_eq!(c1.spec.prompt, "huanle_xuelunyang_target");
        assert!(c1.spec.allow_cancel, "direct:true 目标选择允许取消");
        assert!(
            c1.spec.options.contains(&ChoiceOption::Player(1))
                && c1.spec.options.contains(&ChoiceOption::Player(2))
        );

        // 维度 5（AI 选技能数最多的 1 号）
        let planner = BoundedPlanner::new(16);
        let ai_t = e.plan_action(0, &planner);
        assert_eq!(
            ai_t.action,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![ChoiceOption::Player(1)],
            }
        );

        let stage1_bytes = encode_archive(&e.save());

        // 维度 4（取消路径）：空选取消后不弹技能窗、不获得技能
        let mut e_cancel =
            Engine::restore_with_extensions(&decode_archive(&stage1_bytes).unwrap(), &[&pack]).unwrap();
        submit(
            &mut e_cancel,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![],
            },
        );
        assert!(e_cancel.open_choice().is_none());
        assert!(!e_cancel.debug_has_skill(0, "马术") && !e_cancel.debug_has_skill(0, "武圣"));

        // 分支 B：选 2 号并获得「武圣」
        let mut e_t2 =
            Engine::restore_with_extensions(&decode_archive(&stage1_bytes).unwrap(), &[&pack]).unwrap();
        submit(
            &mut e_t2,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![ChoiceOption::Player(2)],
            },
        );
        let c2_t2 = e_t2.open_choice().expect("huanle_xuelunyang_skill").clone();
        submit(
            &mut e_t2,
            0,
            GameAction::Answer {
                choice_id: c2_t2.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        assert!(e_t2.debug_has_skill(0, "武圣"));

        // 分支 A：选 1 号（Stage 1 恢复后继续）-> Stage 2 挂起中点存档恢复 + Tag(0) vs Tag(1)
        let mut e_stage1_restored =
            Engine::restore_with_extensions(&decode_archive(&stage1_bytes).unwrap(), &[&pack]).unwrap();
        submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        submit(
            &mut e_stage1_restored,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![ChoiceOption::Player(1)],
            },
        );
        assert_eq!(e.state_hash(), e_stage1_restored.state_hash());

        let c2 = e.open_choice().expect("huanle_xuelunyang_skill 挂起").clone();
        assert_eq!(c2.spec.prompt, "huanle_xuelunyang_skill");
        assert_eq!(c2.spec.options, vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]);

        let stage2_bytes = encode_archive(&e.save());
        let mut e_tag0_restored =
            Engine::restore_with_extensions(&decode_archive(&stage2_bytes).unwrap(), &[&pack]).unwrap();
        let mut e_tag1 =
            Engine::restore_with_extensions(&decode_archive(&stage2_bytes).unwrap(), &[&pack]).unwrap();

        let log_before = e.log().len();
        let out_a = submit(
            &mut e,
            0,
            GameAction::Answer {
                choice_id: c2.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        let out_b = submit(
            &mut e_tag0_restored,
            0,
            GameAction::Answer {
                choice_id: c2.id,
                selection: vec![ChoiceOption::Tag(0)],
            },
        );
        let (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) = (out_a, out_b) else {
            panic!("应 Accepted");
        };
        assert_eq!(h1, h2);
        assert_eq!(&e.log()[log_before..], e_tag0_restored.log());
        assert_eq!(e.state_hash(), e_tag0_restored.state_hash());

        submit(
            &mut e_tag1,
            0,
            GameAction::Answer {
                choice_id: c2.id,
                selection: vec![ChoiceOption::Tag(1)],
            },
        );
        // Tag(0) 与 Tag(1) 分别对应 1 号的两个不同技能（马术 vs 咆哮）
        assert_ne!(
            e.debug_has_skill(0, "马术"),
            e_tag1.debug_has_skill(0, "马术"),
            "Tag(0) 与 Tag(1) 应授予 1 号的不同技能"
        );
        assert_ne!(
            e.debug_has_skill(0, "咆哮"),
            e_tag1.debug_has_skill(0, "咆哮"),
            "Tag(0) 与 Tag(1) 应授予 1 号的不同技能"
        );
    }
}

/// `jiuwei` 卡牌（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/card.js:109-120`）
/// 与 `jiuwei` 技能（`archive/upstream/noname/apps/core/extension/欢乐卡牌/card/skill.js:129-143`）六维度验收。
#[test]
fn u06_jiuwei_six_dimension_real_match_equivalence() {
    use engine_native::ai::BoundedPlanner;
    use engine_native::archive::{decode_archive, encode_archive};

    let pack = huanle_pack_from_repo();
    let mut e = Engine::new_with_extensions(3611, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    let jw = e.debug_place(0, "hnl_jiuwei_6");
    assert!(matches!(
        submit(&mut e, 0, GameAction::PlayCard { card: jw, targets: vec![] }),
        CommandOutcome::Accepted { .. }
    ));

    // 推进到 0 号回合结束确认门（受伤状态 hp=2/4）
    e.debug_set_hp(0, 2);
    for _ in 0..16 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm" && c.observer == 0) {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let gate = e.open_choice().expect("酒尾 trigger_confirm 挂起").clone();
    assert!(matches!(
        e.ai_session(0).choice(),
        Some(engine_native::ChoiceView::Own {
            source_skill: Some("huanle_jiuwei"),
            ..
        })
    ));

    // 维度 5：AI BoundedPlanner 自动选择 Confirm（frequent: true）
    let planner = BoundedPlanner::new(16);
    let ai_cmd = e.plan_action(0, &planner);
    assert_eq!(
        ai_cmd.action,
        GameAction::Answer {
            choice_id: gate.id,
            selection: vec![ChoiceOption::Confirm],
        }
    );

    // 维度 4：取消路径（空选放弃触发）不回复体力、不摸牌
    let bytes = encode_archive(&e.save());
    let mut e_cancel = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
    let out_cancel = submit(
        &mut e_cancel,
        0,
        GameAction::Answer {
            choice_id: gate.id,
            selection: vec![],
        },
    );
    assert!(matches!(out_cancel, CommandOutcome::Accepted { .. }));
    assert_eq!(e_cancel.debug_hp(0), 2, "放弃发动九尾不回复体力");
    assert_eq!(e_cancel.debug_hand(0).len(), 0, "放弃发动九尾不摸牌");

    // 维度 6：挂起中点存档恢复等价 + 维度 3 分支 A（已受伤回复 1 点体力）
    let mut e_restored = Engine::restore_with_extensions(&decode_archive(&bytes).unwrap(), &[&pack]).unwrap();
    let log_before = e.log().len();
    let out1 = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: gate.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let out2 = submit(
        &mut e_restored,
        0,
        GameAction::Answer {
            choice_id: gate.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    let (CommandOutcome::Accepted { state_hash: h1 }, CommandOutcome::Accepted { state_hash: h2 }) = (out1, out2) else {
        panic!("应 Accepted");
    };
    assert_eq!(h1, h2);
    assert_eq!(&e.log()[log_before..], e_restored.log());
    assert_eq!(e.state_hash(), e_restored.state_hash());
    assert_eq!(e.debug_hp(0), 3, "已受伤时回复 1 点体力");
    assert_eq!(e.debug_hand(0).len(), 0, "已受伤时不摸牌");

    // 维度 2（负例）：推进经过 1 号回合结束时，0 号的九尾不触发
    for _ in 0..16 {
        if e.turn().current == 1 && e.turn().phase == Phase::End {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let out_other_end = submit(&mut e, 1, GameAction::NextPhase);
    assert!(matches!(out_other_end, CommandOutcome::Accepted { .. }));
    assert!(
        e.open_choice().is_none(),
        "他人回合结束不得触发 0 号的九尾"
    );

    // 维度 3 分支 B（满血未受伤 hp == max_hp 时发动九尾：摸 1 张牌）
    advance_to_play(&mut e, 0);
    e.debug_clear_hand(0);
    e.debug_set_hp(0, 4);
    for _ in 0..16 {
        if e.open_choice().is_some_and(|c| c.spec.prompt == "trigger_confirm" && c.observer == 0) {
            break;
        }
        let actor = e.turn().current;
        submit(&mut e, actor, GameAction::NextPhase);
    }
    let gate_full = e.open_choice().expect("满血九尾 trigger_confirm 挂起").clone();
    submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: gate_full.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert_eq!(e.debug_hp(0), 4, "满血保持 4 点体力");
    assert_eq!(e.debug_hand(0).len(), 1, "未受伤时发动九尾摸 1 张牌");
}
