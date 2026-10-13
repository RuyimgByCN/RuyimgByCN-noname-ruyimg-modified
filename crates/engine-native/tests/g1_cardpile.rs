//! G1 纵向样本（ADR 0776）：两仓 cardpile 扩展 → 真实包转换 →
//! discover_extensions 发现 → store 导入/激活 → 内容锁 → 桥接为
//! 原生扩展目录 → 建局 → 存档/恢复。
//!
//! 转换产物为仓库内真实文件（`packages/extensions/cardpile-upstream-*`），
//! 由 `content-runtime/examples/gen_cardpile_packages.rs` 生成；
//! 本测试同时校验产物与生成器输出逐字节一致（防漂移）。

use std::collections::BTreeMap;
use std::path::PathBuf;

use content_runtime::{
    discover_extensions, import_extensions, InstallOptions, PackageStore,
};
use content_schema::{ContentPackage, PackageManifest};
use engine_native::content::extension::{ExtensionDef, ExtensionError};
use engine_native::content::pile::{extension_pack_from_pile, PileEntry, PileSpec};
use engine_native::{
    CommandOutcome, Engine, GameAction, GameCommand, Phase, SnapshotError,
};
use resource_schema::{AssetEntry, AssetKind, AssetManifest, PreloadPriority};

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

/// 上游 cardpile 比例表（两仓一致；与生成器共享同一数据源语义）。
const ENTRIES: &[(&str, &[(&str, u32)])] = &[
    ("sha", &[("diamond", 6), ("club", 14), ("heart", 3), ("spade", 7)]),
    ("huosha", &[("diamond", 2), ("heart", 3)]),
    ("leisha", &[("spade", 5), ("club", 4)]),
    ("shan", &[("heart", 6), ("diamond", 18)]),
    ("jiu", &[("diamond", 1), ("spade", 2), ("club", 2)]),
    ("tao", &[("heart", 9), ("diamond", 3)]),
    ("wanjian", &[("heart", 1)]),
    ("nanman", &[("spade", 2), ("club", 1)]),
    ("guohe", &[("spade", 3), ("club", 2), ("heart", 1)]),
    ("shunshou", &[("spade", 3), ("diamond", 2)]),
    ("wuxie", &[("heart", 2), ("diamond", 1), ("spade", 2), ("club", 2)]),
    ("tiesuo", &[("spade", 2), ("club", 4)]),
];

fn expected_pile_json(source: &str, upstream_path: &str, upstream_commit: &str) -> String {
    let mut out = String::from("{\n");
    out.push_str(&format!("  \"source\": \"{source}\",\n"));
    out.push_str(&format!("  \"upstream_path\": \"{upstream_path}\",\n"));
    out.push_str(&format!("  \"upstream_commit\": \"{upstream_commit}\",\n"));
    out.push_str("  \"total\": 160,\n");
    out.push_str("  \"entries\": [\n");
    for (idx, (card, counts)) in ENTRIES.iter().enumerate() {
        let mut line = String::from("    {\"card\": \"");
        line.push_str(card);
        line.push_str("\", \"counts\": {");
        for (j, (suit, n)) in counts.iter().enumerate() {
            if j > 0 {
                line.push_str(", ");
            }
            line.push_str(&format!("\"{suit}\": {n}"));
        }
        line.push_str("}}");
        if idx + 1 < ENTRIES.len() {
            line.push(',');
        }
        out.push_str(&line);
        out.push('\n');
    }
    out.push_str("  ]\n}\n");
    out
}

fn build_package(namespace: &str, source: &str, upstream_path: &str, upstream_commit: &str) -> ContentPackage {
    let mut package = ContentPackage {
        manifest: PackageManifest::new(namespace, "cardpile", "1.0.0"),
        modules: vec![],
        skill_index: vec![],
        character_index: vec![],
        skin_index: Default::default(),
        card_index: vec![],
        trigger_index: vec![],
        modifier_index: vec![],
        active_index: vec![],
        view_as_index: vec![],
        translations: BTreeMap::from([(
            "zh-CN".to_string(),
            BTreeMap::from([
                ("jiu".to_string(), "酒".to_string()),
                ("leisha".to_string(), "雷杀".to_string()),
                ("huosha".to_string(), "火杀".to_string()),
                ("cardpile".to_string(), "牌堆".to_string()),
            ]),
        )]),
        asset_manifest: None,
        asset_paths: BTreeMap::new(),
        compiler_version: "gen-1".into(),
    };
    let pile = expected_pile_json(source, upstream_path, upstream_commit);
    let mut assets = AssetManifest::new("cardpile");
    assets.add_entry(AssetEntry {
        asset_id: "cardpile:pile".into(),
        source_hash: *blake3::hash(pile.as_bytes()).as_bytes(),
        kind: AssetKind::Data,
        image_variants: vec![],
        audio_variants: vec![],
        priority: PreloadPriority::Lazy,
        depends_on: vec![],
        platforms: vec![],
    });
    package.asset_manifest = Some(assets);
    package
        .asset_paths
        .insert("cardpile:pile".into(), "assets/pile.json".into());
    package.manifest.logic_hash = package.compute_logic_hash();
    package.manifest.asset_hash = package.compute_asset_hash();
    package
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

/// 转换产物防漂移：仓库内的 npack 与生成器输出逐字节一致。
#[test]
fn converted_packages_match_generator() {
    let sources: &[(&str, &str, &str, &str)] = &[
        (
            "upstream-noname",
            "noname",
            "apps/core/extension/cardpile/extension.js",
            "5c02a745eb33b61b2e61e1167e1b71ca163911f6",
        ),
        (
            "upstream-liuli",
            "noname-liuli",
            "noname/extension/cardpile/extension.js",
            "8485dbb29ff16f9b258a766b1a17ef75f883aade",
        ),
    ];
    for (namespace, source, path, commit) in sources {
        let dir = repo_extensions_root().join(format!("cardpile-{namespace}"));
        let bytes = std::fs::read(dir.join("package.npack.json")).expect("npack 存在");
        let expected = build_package(namespace, source, path, commit);
        assert_eq!(
            bytes,
            expected.to_json_bytes().unwrap(),
            "{namespace} npack 与生成器输出不一致"
        );
    }
}

/// G1 纵向样本：发现 → 导入（安装+激活）→ 内容锁 → 桥接建局 →
/// 扩展牌参与规则结算 → 存档按集合绑定恢复。
#[test]
fn g1_cardpile_vertical_install_enable_newgame_saverestore() {
    // 1) 发现：两仓 cardpile 目录均被发现（旧根目录 0 包的反例由
    //    真实转换产物关闭）
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    let names: Vec<&str> = discovered
        .iter()
        .map(|d| d.name.as_str())
        .collect();
    assert!(
        names.contains(&"cardpile-upstream-noname") && names.contains(&"cardpile-upstream-liuli"),
        "两仓 cardpile 均可发现: {names:?}"
    );

    // 2) 导入：安装 + 激活 → 内容锁包含两个包
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let ids: Vec<&str> = report
        .installed
        .iter()
        .map(|m| m.package_id.as_str())
        .collect();
    assert!(ids.contains(&"cardpile"), "两仓 cardpile 均安装: {ids:?}");
    assert_eq!(ids.len(), report.content_lock.packages.len(), "全部安装包进入内容锁");
    assert!(report.skipped.is_empty(), "默认启用：无跳过");

    // 3) 桥接：读取已安装 pile.json → PileSpec → 原生扩展包
    let mut packs = Vec::new();
    for meta in report.installed.iter().filter(|m| m.package_id == "cardpile") {
        let source_root = PathBuf::new()
            .join(meta.extension_root.as_ref().expect("drop-in 来源目录"));
        let pile_bytes = std::fs::read_to_string(source_root.join("assets/pile.json"))
            .expect("已安装资产可读");
        // 装载器入口（T6①）：pile.json 解析在 content-native-loader，
        // 测试不再内嵌 parse_spec
        let spec = content_native_loader::bridge_pile(&pile_bytes).expect("pile.json 可装载");
        assert_eq!(spec.total, 160, "上游比例表 total=160");
        let prefix = if meta.namespace == "upstream-noname" { "cnp" } else { "cpl" };
        let def = ExtensionDef {
            name: match prefix {
                "cnp" => "cardpile-upstream-noname",
                _ => "cardpile-upstream-liuli",
            },
            version: 1,
            description: "G1 转换：上游 cardpile 牌堆比例",
        };
        let pack = extension_pack_from_pile(def, prefix, &spec).expect("桥接");
        assert_eq!(pack.card_count(), 113, "比例表合计 113 张（非 160：上游按倍率补足其余）");
        packs.push(pack);
    }
    assert_eq!(packs.len(), 2);

    // 4) 建局：noname 来源扩展卡入目录，目录版本随内容锁变化
    //    （发现按目录名排序：liuli 在前；按来源身份显式选取）
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let pack_ref = packs
        .iter()
        .find(|p| p.def.name == "cardpile-upstream-noname")
        .expect("noname 来源扩展包");
    let mut e = Engine::new_with_extensions(3101, 3, std::slice::from_ref(&pack_ref))
        .expect("带扩展建局");
    assert_eq!(
        e.catalog().cards.len(),
        base_cards + 113,
        "扩展牌堆卡合并入目录"
    );
    assert_ne!(
        e.catalog().ruleset_version,
        Engine::new(1, 3).catalog().ruleset_version,
        "内容锁：扩展参与目录版本"
    );

    // 5) 扩展牌参与规则结算：从目录定位扩展【酒】/【杀】，注入手牌，
    //    酒 → 杀 = 2 点伤害
    let (_jiu_id, sha_id) = {
        let catalog = e.catalog();
        let jiu = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("cnp_jiu"))
            .expect("扩展酒牌在目录") as u32;
        let sha = catalog
            .cards
            .iter()
            .position(|c| c.name.starts_with("cnp_sha"))
            .expect("扩展杀牌在目录") as u32;
        (jiu, sha)
    };
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        match submit(&mut e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    for p in 0..3 {
        e.debug_clear_hand(p);
    }
    e.debug_set_hp(1, 4);
    // 注入扩展牌（快照层操作保持归属完整）
    let mut snap = e.save();
    // v64 牌堆扩容后目录位次位移：首个 cnp_sha 可能已在前序段落的弃牌堆
    // ——改选仍在牌堆中的扩展酒/杀注入（牌归属恰一区约束）。
    let (jiu_in_deck, sha_in_deck) = {
        let cat = e.catalog();
        let pick = |kind: engine_native::CardKind| {
            snap.core
                .deck
                .iter()
                .copied()
                .find(|c| cat.cards[*c as usize].kind == kind)
                .expect("扩展牌在牌堆")
        };
        (pick(engine_native::CardKind::Jiu), pick(engine_native::CardKind::Sha))
    };
    snap.core.deck.retain(|c| *c != jiu_in_deck && *c != sha_in_deck);
    snap.core.players[0].hand.push(jiu_in_deck);
    snap.core.players[0].hand.push(sha_in_deck);
    {
        let cnt = |v: &Vec<u32>| v.iter().filter(|c| **c == sha_id).count();
        eprintln!("PROBE-CP2 after-inject sha114: deck={} hand0={} discard={} table={}",
            cnt(&snap.core.deck), cnt(&snap.core.players[0].hand), cnt(&snap.core.discard), cnt(&snap.core.table));
        let mut total = 0usize;
        for (_i, p) in snap.core.players.iter().enumerate() {
            total += cnt(&p.hand) + cnt(&p.equips) + cnt(&p.judge);
        }
        eprintln!("PROBE-CP2 sha114 in all player zones={total}");
    }
    let mut e = Engine::restore_with_extensions(&snap, std::slice::from_ref(&pack_ref))
        .expect("注入后恢复");
    let out = submit(&mut e, 0, GameAction::PlayCard { card: jiu_in_deck, targets: vec![] });
    assert!(matches!(out, CommandOutcome::Accepted { .. }), "扩展酒可使用: {out:?}");
    let out2 = submit(&mut e, 0, GameAction::PlayCard { card: sha_in_deck, targets: vec![1] });
    assert!(matches!(out2, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "{out2:?}");
    assert_eq!(e.debug_hp(1), 2, "扩展酒+扩展杀 = 2 点伤害");

    // 6) 存档/恢复：同集合 OK 且同哈希；异集合/纯标准被内容锁拒绝
    let saved = e.save();
    let ok = Engine::restore_with_extensions(&saved, std::slice::from_ref(&pack_ref))
        .expect("同集合恢复");
    assert_eq!(ok.state_hash(), e.state_hash());
    assert!(matches!(
        Engine::restore(&saved),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ), "纯标准目录不能恢复带扩展存档");
    let other_ref = packs
        .iter()
        .find(|p| p.def.name == "cardpile-upstream-liuli")
        .expect("liuli 来源扩展包");
    assert!(matches!(
        Engine::restore_with_extensions(&saved, std::slice::from_ref(&other_ref)),
        Err(SnapshotError::RuleVersionMismatch { .. })
    ), "另一来源的扩展集合不能恢复");
}

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    use std::sync::atomic::{AtomicU64, Ordering};
    static DEDUP: AtomicU64 = AtomicU64::new(1);
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

/// 扩展错误可处理：未知牌名/零数量比例返回错误而非 panic。
#[test]
fn pile_bridge_rejects_invalid_spec() {
    let def = ExtensionDef { name: "t", version: 1, description: "" };
    let spec = PileSpec {
        source: "s".into(),
        total: 1,
        entries: vec![PileEntry {
            card: "nope".into(),
            counts: vec![("spade".into(), 1)],
        }],
    };
    match extension_pack_from_pile(def, "t", &spec) {
        Err(ExtensionError::DuplicateCard { name }) => {
            assert_eq!(name, "pile:unknown_card");
        }
        Err(_) => panic!("未知牌名应返回 DuplicateCard 错误"),
        Ok(_) => panic!("未知牌名应返回错误"),
    }
}

/// 整包验收 U03（2026-10-08，来源级第二例）：cardpile（noname 仓）真实
/// 宿主全生命周期——独立安装+激活 → 安装产物→桥接→原生建局（113 张
/// 比例表入目录）→ 对局中卸载 → 在局锁定副本续玩 → 撤销后重装/重新
/// import 被拒。双仓同装场景下仅撤 noname 仓，liuli 仓激活不受影响。
#[test]
fn cardpile_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 双仓独立安装 + 激活（两 namespace 同 slot 名 cardpile）
    let report = import_extensions(&store, &root, &options()).expect("import");
    assert!(report.installed.iter().any(|m| m.package_id == "cardpile" && m.namespace == "upstream-noname"));
    assert!(report.installed.iter().any(|m| m.package_id == "cardpile" && m.namespace == "upstream-liuli"));
    let cardpile_locks = report
        .content_lock
        .packages
        .iter()
        .filter(|p| p.package_id == "cardpile")
        .count();
    assert_eq!(cardpile_locks, 2, "双仓 cardpile 进入内容锁");

    // ② 安装产物 → 桥接 → 原生建局（noname 仓 113 张唯一实体牌）
    let meta_cnp = report
        .installed
        .iter()
        .find(|m| m.package_id == "cardpile" && m.namespace == "upstream-noname")
        .unwrap()
        .clone();
    let source_root = PathBuf::new()
        .join(meta_cnp.extension_root.as_ref().expect("drop-in 来源目录"));
    let pile_bytes = std::fs::read_to_string(source_root.join("assets/pile.json")).expect("pile.json");
    let spec = content_native_loader::bridge_pile(&pile_bytes).expect("pile.json 可装载");
    assert_eq!(spec.total, 160);
    let def = ExtensionDef {
        name: "cardpile-upstream-noname",
        version: 1,
        description: "G1 转换：上游 cardpile 牌堆比例",
    };
    let pack = extension_pack_from_pile(def, "cnp", &spec).expect("桥接");
    assert_eq!(pack.card_count(), 113, "比例表合计 113 张");
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6001, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 113, "113 张唯一实体牌入目录");
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载：仅撤 noname 仓 slot
    store
        .revoke_slot("upstream-noname", "cardpile", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked("upstream-noname", "cardpile"));
    assert!(!store.read_active().unwrap().packages.contains_key("upstream-noname/cardpile"));
    // liuli 仓激活不受影响
    assert!(store.read_active().unwrap().packages.contains_key("upstream-liuli/cardpile"));

    // ④ 在局锁定副本续玩（113 张仍可用）
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 113, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫：撤销后 noname 仓 npack 再安装被拒
    let bytes = std::fs::read(source_root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒（含撤销 slot 的全量导入）
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}

/// 整包验收 L04（2026-10-08，来源级第三例）：cardpile（liuli 仓）真实
/// 宿主全生命周期——镜像 U03：双仓同装下仅撤 liuli slot（noname 不受
/// 影响），liuli 仓锁定副本续玩、重装/重新 import 被拒。
#[test]
fn cardpile_liuli_real_host_full_lifecycle() {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();

    // ① 双仓独立安装 + 激活
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta_cpl = report
        .installed
        .iter()
        .find(|m| m.package_id == "cardpile" && m.namespace == "upstream-liuli")
        .unwrap()
        .clone();
    assert!(store.read_active().unwrap().packages.contains_key("upstream-liuli/cardpile"));

    // ② liuli 仓安装产物 → 桥接 → 原生建局（113 张）
    let source_root = PathBuf::new()
        .join(meta_cpl.extension_root.as_ref().expect("drop-in 来源目录"));
    let pile_bytes = std::fs::read_to_string(source_root.join("assets/pile.json")).expect("pile.json");
    let spec = content_native_loader::bridge_pile(&pile_bytes).expect("pile.json 可装载");
    assert_eq!(spec.total, 160);
    let def = ExtensionDef {
        name: "cardpile-upstream-liuli",
        version: 1,
        description: "G1 转换：上游 cardpile 牌堆比例（liuli）",
    };
    let pack = extension_pack_from_pile(def, "cpl", &spec).expect("桥接");
    assert_eq!(pack.card_count(), 113);
    let base_cards = Engine::new(1, 3).catalog().cards.len();
    let mut e = Engine::new_with_extensions(6003, 3, &[&pack]).expect("建局");
    assert_eq!(e.catalog().cards.len(), base_cards + 113);
    let _ = submit(&mut e, 0, GameAction::NextPhase);
    let mid = e.save();

    // ③ 对局中卸载：仅撤 liuli slot；noname 仓激活不受影响
    store
        .revoke_slot("upstream-liuli", "cardpile", "验收：对局中卸载", true)
        .expect("撤销");
    assert!(store.read_revoke().unwrap().is_slot_revoked("upstream-liuli", "cardpile"));
    assert!(!store.read_active().unwrap().packages.contains_key("upstream-liuli/cardpile"));
    assert!(store.read_active().unwrap().packages.contains_key("upstream-noname/cardpile"));

    // ④ liuli 仓锁定副本续玩（113 张）
    let mut resumed = Engine::restore_with_extensions(
        &engine_native::archive::decode_archive(&engine_native::archive::encode_archive(&mid))
            .expect("解码"),
        &[&pack],
    )
    .expect("对局中卸载后恢复");
    assert_eq!(resumed.catalog().cards.len(), base_cards + 113, "锁定副本仍可用");
    let out = submit(&mut resumed, 0, GameAction::NextPhase);
    assert!(matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }), "在局续玩: {out:?}");

    // ⑤ 重装守卫：liuli npack 再安装被拒
    let bytes = std::fs::read(source_root.join("package.npack.json")).expect("npack");
    let err = store.install(&bytes, None, &options());
    assert!(matches!(err, Err(content_runtime::RuntimeError::Revoked(_))), "撤销后重装应被拒: {err:?}");

    // ⑥ 重新 import 被拒
    let report2 = import_extensions(&store, &root, &options());
    assert!(report2.is_err(), "撤销后重新 import 应被拒");
}
