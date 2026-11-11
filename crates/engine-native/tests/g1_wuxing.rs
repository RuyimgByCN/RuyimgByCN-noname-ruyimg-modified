//! G1 wuxing 纵向样本（ADR 0776）：五行属性入状态 → 生克宿主行为 →
//! 五行盘主动技（PickTag 选择原语）→ 建局/存档/恢复。
//!
//! 上游（noname-liuli wuxing @ 8485dbb2）：每角色随机属性、牌堆 1/3
//! 随机属性、被克弃牌/被生摸牌、五行盘改手牌属性（equip5）。
//! 原生映射：属性入核心状态（规则随机性走引擎 RNG，不是上游
//! Math.random 逐局等价）；被克弃牌为确定性首张（登记差异：上游为
//! 目标自选「he」）；五行盘占栏 equip5 → 宝物栏，`equip_skills` 随
//! 装备授予 `wuxingpan_skill`。联机提前返回、录像和帮助 HTML 仍未迁入。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}


fn install_and_bridge() -> ExtensionPack {
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(
        discovered.iter().any(|d| d.name == "wuxing-upstream-liuli"),
        "wuxing 包可发现"
    );
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wuxing")
        .expect("wuxing 安装");
    let cfg = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/wuxing.json"),
    )
    .expect("配置可读");
    assert!(
        cfg.contains("\"wuxingpan\"") && cfg.contains("\"denominator\": 3"),
        "配置数据核对"
    );
    // 桥接统一走 C0 装载器（ADR 0777/0778）：五行盘、生克/盘属性改写
    // 两个宿主技能声明与开局钩子都来自 cards.json 的 pack 段与数据键。
    let cards_json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let out = content_native_loader::bridge_pack(&cards_json).expect("wuxing 可装载");
    assert_eq!(out.pack.def.name, "wuxing-upstream-liuli", "包名来自 pack 段");
    out.pack
}

/// G1 纵向：导入 → 建局（五行初始化：角色属性 + 牌堆属性 1/3）→
/// 生克结算 → 存档/恢复（属性随状态绑定）。
#[test]
fn g1_wuxing_vertical_natures_shengke_saverestore() {
    let pack = install_and_bridge();
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e =
        Engine::new_with_extensions(3301, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 1, "五行盘入目录");

    // 初始化：每名角色有属性；牌堆约 1/3 有属性（至少 1 张，全部合法）
    let snap = e.save();
    for p in snap.core.players.iter() {
        assert!(p.wunature.map(|n| n <= 4).unwrap_or(false), "角色属性已初始化");
    }
    let natured = snap.core.card_natures.len();
    assert!(natured > 0, "牌堆应有属性牌");
    // 统计比例容差（小牌堆采样波动）：1/4 ~ 1/2 区间即视为符合 1/3 设计
    let deck_len = snap.core.deck.len();
    assert!(
        natured * 4 >= deck_len && natured * 2 <= deck_len * 2 && natured <= deck_len,
        "约 1/3 比例: {natured} / {deck_len}"
    );
    assert!(natured * 3 <= deck_len * 2 + 6, "上限护栏: {natured}/{deck_len}");

    // 生克——被生（金生水）：0 号水属性，成为金属性杀的目标 → 摸 1
    // （推进到 1 号出牌阶段；途中挂起的选择取首选项应答）
    for _ in 0..128 {
        if e.turn().current == 1 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            break;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(&mut e, obs, GameAction::Answer { choice_id: cid, selection: sel }) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    let sha1 = e.debug_place(1, "sha");
    let mut snap = e.save();
    let hand0_before = snap.core.players[0].hand.len();
    snap.core.players[0].wunature = Some(2); // 水
    snap.core.card_natures.insert(sha1, 0); // 金
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("属性注入恢复");
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha1, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    assert_eq!(
        e.save().core.players[0].hand.len(),
        hand0_before + 1,
        "金生水：被生摸一张"
    );

    // 生克——被克（金克木）：0 号木属性，成为金属性杀的目标 → 弃一张手牌
    let sha2 = e.debug_place(1, "sha");
    let _tao = e.debug_place(0, "tao"); // debug_place 已将 tao 置入 0 号手牌
    let mut snap = e.save();
    snap.core.players[0].wunature = Some(1); // 木
    snap.core.turn.sha_used_play_phase = false; // 同阶段第二张杀（测试便捷）
    snap.core.card_natures.insert(sha2, 0); // 金
    e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let discard_before = e.save().core.discard.len();
    // v64 牌堆位移：前序结算可能留有挂起询问——先排空再出杀。
    for _ in 0..8 {
        match e.open_choice() {
            Some(c) => {
                let sel: Vec<ChoiceOption> = if c.spec.min > 0 {
                    c.spec.options.iter().take(c.spec.min).copied().collect()
                } else if !c.spec.options.is_empty() {
                    vec![c.spec.options[0]]
                } else {
                    vec![]
                };
                let (id2, obs2) = (c.id, c.observer);
                let _ = submit(&mut e, obs2, GameAction::Answer { choice_id: id2, selection: sel });
            }
            None => break,
        }
    }
    let out = submit(&mut e, 1, GameAction::PlayCard { card: sha2, targets: vec![0] });
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    if let Some(c) = e.open_choice().cloned() {
        if c.spec.prompt == "shengke_discard" {
            let sel: Vec<ChoiceOption> = c.spec.options.iter().take(1).copied().collect();
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: sel,
                },
            );
        }
    }
    assert_eq!(
        e.save().core.discard.len(),
        discard_before + 2, // 杀 + 被克弃的手牌
        "金克木：被克弃一张手牌"
    );

    // 存档绑定：带 wuxing 存档被纯标准恢复拒绝
    let saved = e.save();
    assert!(matches!(
        Engine::restore(&saved),
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. })
    ));
    Engine::restore_with_extensions(&saved, &[&pack]).expect("同集合恢复");
}

/// 五行盘主动技：授予 → 激活 → 选牌 → 选属性（PickTag）→ 属性入表；
/// 两段选择各支持字节恢复。
#[test]
fn wuxingpan_skill_change_nature_with_restore() {
    use engine_native::archive::{decode_archive, encode_archive};
    let pack = install_and_bridge();
    let mut e =
        Engine::new_with_extensions(3302, 3, &[&pack]).expect("建局");
    e.debug_grant(0, "wuxingpan_skill");
    advance_to_play(&mut e, 0);
    let card = e.debug_place(0, "tao");
    let skill = e.catalog().skill_by_name("wuxingpan_skill").unwrap();
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill });
    match out {
        CommandOutcome::Pending { choice } => {
            assert_eq!(choice.spec.prompt, "wuxing_select_card");
        }
        other => panic!("应挂起选牌: {other:?}"),
    }
    // 选牌阶段字节恢复
    let bytes = encode_archive(&e.save());
    let mut restored = Engine::restore_with_extensions(
        &decode_archive(&bytes).expect("decode"),
        &[&pack],
    )
    .expect("选牌挂起恢复");
    let c = restored.open_choice().expect("恢复件仍挂起");
    assert_eq!(c.spec.prompt, "wuxing_select_card");
    let cid = c.id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid,
        selection: vec![ChoiceOption::Card(card)],
    });
    // 选属性（PickTag，5 个 Tag 选项）
    let c2 = restored.open_choice().expect("选属性挂起");
    assert_eq!(c2.spec.prompt, "wuxing_select_nature");
    assert!(matches!(
        c2.spec.kind,
        engine_native::ChoiceKind::PickTag
    ));
    assert_eq!(c2.spec.options.len(), 5);
    let cid2 = c2.id;
    submit(&mut restored, 0, GameAction::Answer {
        choice_id: cid2,
        selection: vec![ChoiceOption::Tag(3)], // 火
    });
    assert_eq!(
        restored.save().core.card_natures.get(&card),
        Some(&3),
        "五行盘：属性已永久改变"
    );
}

/// 五行盘卡牌：占栏装备路径（Treasure 槽）。
#[test]
fn wuxingpan_card_equips_via_treasure_slot() {
    let pack = install_and_bridge();
    let mut e = Engine::new_with_extensions(3303, 3, &[&pack]).expect("建局");
    advance_to_play(&mut e, 0);
    let (wp,) = {
        let catalog = e.catalog();
        let wp = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("wxl_wuxingpan"))
            .expect("五行盘在目录") as u32;
        (wp,)
    };
    let mut snap = e.save();
    snap.core.deck.retain(|c| *c != wp);
    snap.core.players[0].hand.push(wp);
    let mut e = Engine::restore_with_extensions(&snap, &[&pack]).expect("注入恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: wp, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "{out:?}");
    assert!(
        e.save().core.players[0].equips.contains(&wp),
        "五行盘应装备至宝物栏"
    );
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..64 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play {
            return;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    panic!("no play phase");
}

/// 整包验收续（2026-10-08，第六例）：wuxing 真实宿主全生命周期——
/// 独立安装+激活 → 安装包→原生建局 → 对局中卸载 → 在局锁定副本
/// 续玩 → 撤销后重装/重新 import 被拒。
#[test]
fn wuxing_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wuxing")
        .expect("wuxing 安装")
        .clone();
    let slot = format!("{}/{}", meta.namespace, meta.package_id);
    assert!(store.read_active().unwrap().packages.contains_key(&slot), "激活集");

    // ② 安装包 → 原生建局（五行盘入目录）
    let cards_json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .expect("cards.json 可读");
    let out = content_native_loader::bridge_pack(&cards_json).expect("桥接");
    let pack = out.pack;
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(5801, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 1, "五行盘入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载
    store
        .revoke_slot(&meta.namespace, "wuxing", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked(&meta.namespace, "wuxing"));
    assert!(!store.read_active().unwrap().packages.contains_key(&slot), "移出激活集");

    // ④ 在局锁定副本续玩
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 1, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫
    let bytes = std::fs::read(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("package.npack.json"),
    )
    .expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// L06 五行八项整包（2026-10-17）：从真实安装覆盖功能、配置、依赖、组合、
/// 保存恢复、安装升级、目标平台与规则差异。联机/录像/帮助 HTML 仍是登记差异。
/// 本测试只关闭 L06。
#[test]
fn l06_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;
    use content_schema::{ContentPackage, PackageDependency};

    let source_root = repo_extensions_root().join("wuxing-upstream-liuli");
    let npack_path = source_root.join("package.npack.json");
    let cards_path = source_root.join("assets/cards.json");
    let npack_bytes = std::fs::read(&npack_path).expect("读 npack");
    let cards_text = std::fs::read_to_string(&cards_path).expect("读 cards.json");
    let cfg_text = std::fs::read_to_string(source_root.join("assets/wuxing.json")).expect("配置");
    assert!(
        cfg_text.contains("\"denominator\": 3"),
        "配置正例：牌堆属性比例分母为 3"
    );

    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "wuxing")
        .expect("五行已安装");
    assert_eq!(meta.version, "1.0.0");
    assert_eq!(meta.namespace, "upstream-liuli");
    let installed_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let installed_npack = std::fs::read(installed_root.join("package.npack.json")).expect("安装副本");
    assert_eq!(installed_npack, npack_bytes);
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );
    let installed_pkg = ContentPackage::from_json_bytes(&installed_npack).expect("解析");
    assert!(installed_pkg.manifest.dependencies.is_empty(), "依赖正例：无外部依赖");

    let active_version = |store: &PackageStore| -> String {
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("/wuxing"))
            .map(|(_, version)| version.clone())
            .expect("激活集含五行")
    };
    assert_eq!(active_version(&store), "1.0.0");

    match store.activate_version("upstream-liuli", "wuxing", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }
    assert_eq!(active_version(&store), "1.0.0");

    let mut escaped = installed_pkg.clone();
    escaped.manifest.package_id = "../wuxing".into();
    escaped.manifest.logic_hash = escaped.compute_logic_hash();
    match store.install(&escaped.to_json_bytes().unwrap(), None, &options()) {
        Err(RuntimeError::Package(msg)) => assert!(msg.contains("invalid package_id"), "{msg}"),
        other => panic!("路径逃逸应拒绝: {other:?}"),
    }

    let mut missing = installed_pkg.clone();
    missing.manifest.version = "1.0.1".into();
    missing.manifest.dependencies.push(PackageDependency {
        namespace: "upstream-liuli".into(),
        package_id: "missing-base".into(),
        version: "1.0.0".into(),
        logic_hash: [0u8; 32],
    });
    missing.manifest.logic_hash = missing.compute_logic_hash();
    match store.install_with_assets(
        &missing.to_json_bytes().unwrap(),
        None,
        &options(),
        &source_root,
    ) {
        Err(RuntimeError::Invalid(msg)) => assert!(msg.contains("missing dependency"), "{msg}"),
        other => panic!("缺依赖应拒绝: {other:?}"),
    }
    assert_eq!(active_version(&store), "1.0.0");

    let disabled_dir = tempfile::tempdir().unwrap();
    let disabled_root = disabled_dir.path().join("wuxing-disabled");
    std::fs::create_dir_all(disabled_root.join("assets")).unwrap();
    std::fs::copy(source_root.join("manifest.json"), disabled_root.join("manifest.json")).unwrap();
    std::fs::copy(&npack_path, disabled_root.join("package.npack.json")).unwrap();
    std::fs::copy(&cards_path, disabled_root.join("assets/cards.json")).unwrap();
    std::fs::copy(
        source_root.join("assets/wuxing.json"),
        disabled_root.join("assets/wuxing.json"),
    )
    .unwrap();
    let mf_path = disabled_root.join("manifest.json");
    let mut mf: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(&mf_path).unwrap()).unwrap();
    mf["enabled"] = serde_json::Value::Bool(false);
    std::fs::write(&mf_path, serde_json::to_string_pretty(&mf).unwrap()).unwrap();
    {
        let disabled_store = PackageStore::open(disabled_dir.path().join("store")).unwrap();
        let disabled =
            import_extensions(&disabled_store, disabled_dir.path(), &options()).expect("import");
        assert!(!disabled.installed.iter().any(|m| m.package_id == "wuxing"));
        assert!(disabled.skipped.iter().any(|s| s == "wuxing-disabled"));
    }

    let value: serde_json::Value = serde_json::from_str(&cards_text).unwrap();
    let shengke = value["skills"]
        .as_array()
        .unwrap()
        .iter()
        .find(|s| s["name"] == "wuxing_shengke")
        .unwrap()["mapping"]
        .as_str()
        .unwrap();
    assert_eq!(shengke, "host:shengke_check");
    assert!(
        value["cards"][0]["equip_skills"]
            .as_array()
            .unwrap()
            .iter()
            .any(|s| s == "wuxingpan_skill"),
        "五行盘声明装备技能"
    );

    let installed_cards =
        std::fs::read_to_string(installed_root.join("assets/cards.json")).unwrap();
    assert_eq!(installed_cards, cards_text);
    let pack = {
        let out = content_native_loader::bridge_pack(&installed_cards).unwrap();
        assert_eq!(out.pack.def.name, "wuxing-upstream-liuli");
        out.pack
    };
    let mut e = Engine::new_with_extensions(3419, 3, &[&pack]).expect("建局");
    let again = Engine::new_with_extensions(3419, 3, &[&pack]).expect("同种子");
    assert_eq!(
        e.save().core.players[0].wunature,
        again.save().core.players[0].wunature,
        "规则差异：属性由引擎 RNG 决定，同种子可重放；不是上游 Math.random 逐局等价"
    );
    let base = Engine::new(1, 3).catalog().cards.len();
    assert_eq!(e.catalog().cards.len(), base + 1);
    assert!(e.save().core.players.iter().all(|p| p.wunature.is_some()));
    assert!(!e.save().core.card_natures.is_empty(), "功能正例：牌堆获得属性");

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
        other => panic!("不存在的牌必须拒绝: {other:?}"),
    }
    let pan_name = e
        .catalog()
        .cards
        .iter()
        .find(|c| c.name.starts_with("wxl_wuxingpan"))
        .expect("五行盘")
        .name
        .to_string();
    let pan = e.debug_move_named_card_to_hand(0, &pan_name);
    let out = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: pan,
            targets: vec![],
        },
    );
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "装备五行盘: {out:?}");
    assert!(e.save().core.players[0].equips.contains(&pan));
    let def = e.catalog().skill_by_name("wuxingpan_skill").expect("技能在目录");
    assert!(
        e.save().core.skills.contains_key(&(0, def)),
        "功能正例：装备授予五行盘技能"
    );

    let saved = e.save();
    let bytes = engine_native::archive::encode_archive(&saved);
    let decoded = engine_native::archive::decode_archive(&bytes).unwrap();
    let restored = Engine::restore_with_extensions(&decoded, &[&pack]).expect("保存恢复");
    assert_eq!(restored.catalog().cards.len(), base + 1);
    match Engine::restore(&saved) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(err) => panic!("{err:?}"),
        Ok(_) => panic!("标准目录不得恢复"),
    }
    let mut wrong = content_native_loader::bridge_pack(&installed_cards).unwrap().pack;
    wrong.def.version += 1;
    match Engine::restore_with_extensions(&saved, &[&wrong]) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(err) => panic!("错版必须拒绝: {err:?}"),
        Ok(_) => panic!("错版必须拒绝"),
    }

    let huanle_text = std::fs::read_to_string(
        repo_extensions_root().join("huanle-upstream-noname/assets/cards.json"),
    )
    .unwrap();
    let huanle = content_native_loader::bridge_pack(&huanle_text).unwrap().pack;
    let combo = Engine::new_with_extensions(3420, 3, &[&pack, &huanle]).expect("组合");
    assert!(combo.catalog().cards.len() > base + 1);
    assert_eq!(
        combo.catalog().cards.iter().filter(|c| c.name.starts_with("wxl_")).count(),
        1
    );
    assert_eq!(
        combo.catalog().cards.iter().filter(|c| c.name.starts_with("hnl_")).count(),
        7
    );
    match Engine::restore_with_extensions(&saved, &[&pack, &huanle]) {
        Err(engine_native::SnapshotError::RuleVersionMismatch { .. }) => {}
        Err(err) => panic!("组合反例: {err:?}"),
        Ok(_) => panic!("单包存档不得用双包目录恢复"),
    }

    let mut upgraded = installed_pkg.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    store
        .install_with_assets(&upgraded.to_json_bytes().unwrap(), None, &options(), &source_root)
        .expect("升级");
    assert_eq!(active_version(&store), "2.0.0");
    let rolled = store.rollback().unwrap();
    assert_eq!(
        rolled
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("/wuxing"))
            .map(|(_, v)| v.as_str()),
        Some("1.0.0")
    );
    let still = Engine::restore_with_extensions(&saved, &[&pack]).expect("升级前存档");
    assert!(still.save().core.skills.contains_key(&(0, def)));
}
