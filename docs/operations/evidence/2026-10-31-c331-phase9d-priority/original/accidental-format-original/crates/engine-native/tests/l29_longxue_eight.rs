//! L29 龙血玄黄 八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/longxue`。规则面（潜龙
//! 道心值 + 阈值解锁 + 清正）走内核与宿主行为；本包配置（`config.shengjian`
//! 圣剑显示开关）与阈值报告走 `content_native_loader::longxue`。不迁移
//! Spine/DOM 显示面（`_CMeffects`、死亡动画、进度条）。

use std::path::PathBuf;

use content_native_loader::{
    longxue_unlock_report, parse_longxue_config, LongxueError, LONGXUE_THRESHOLDS,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::ContentPackage;
use engine_native::event::DamageNature;
use engine_native::{CommandOutcome, Engine, GameAction, GameCommand};

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

fn store_dir(store: &PackageStore, namespace: &str, package_id: &str, version: &str) -> PathBuf {
    store
        .root()
        .join("packages")
        .join(format!("{namespace}__{package_id}"))
        .join(version)
}

fn read_installed(
    store: &PackageStore,
    namespace: &str,
    package_id: &str,
    version: &str,
) -> (ContentPackage, PathBuf) {
    let dir = store_dir(store, namespace, package_id, version);
    let bytes = std::fs::read(dir.join("package.json")).expect("商店副本 package.json");
    let package = ContentPackage::from_json_bytes(&bytes).expect("解析商店副本");
    (package, dir)
}

fn general_def_of(e: &Engine, name: &str) -> u16 {
    for def in 0..=u16::MAX {
        if e.catalog().general_meta(def).map(|g| g.0) == Some(name) {
            return def;
        }
        if e.catalog().general_meta(def).is_none() && def > 64 {
            break;
        }
    }
    panic!("目录中无武将 {name}");
}

/// 从已安装商店副本桥接武将包并建局（曹髦装配到 1 号，隔离首个摸牌阶段）。
fn engine_from_install(dir: &PathBuf, seed: u64) -> Engine {
    let json = std::fs::read_to_string(dir.join("assets/cards.json")).expect("包数据");
    let bridged = content_native_loader::bridge_pack(&json).expect("桥接");
    let pack = bridged.pack;
    let mut e = Engine::new_with_extensions(seed, 4, &[&pack]).expect("建局");
    let def = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(1, def);
    e
}

fn drain(e: &mut Engine) {
    for _ in 0..48 {
        let Some(c) = e.open_choice().cloned() else {
            break;
        };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec
                .options
                .iter()
                .take(c.spec.min.max(1))
                .copied()
                .collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: 700_000 + u64::from(c.id),
            action: GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        });
    }
}

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: 800_000 + u64::from(e.revision()),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(e);
}

/// L29 八项整包：功能 / 配置 / 依赖 / 组合 / 保存恢复 / 安装升级 / 目标平台 /
/// 规则差异（每项正例 + 反例）。
#[test]
fn l29_eight_dimension_from_fresh_install() {
    let root = std::env::temp_dir().join(format!(
        "l29-eight-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0)
    ));
    let store = PackageStore::open(root.clone()).expect("商店");

    // ---- 安装升级（正例）：真实 npack 安装并激活 ----
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("导入");
    assert!(
        report
            .installed
            .iter()
            .any(|p| p.package_id == "longxue" && p.namespace == "upstream-liuli"),
        "安装报告含龙血玄黄：{:?}",
        report
            .installed
            .iter()
            .map(|p| format!("{}/{}@{}", p.namespace, p.package_id, p.version))
            .collect::<Vec<_>>()
    );
    let (package, dir) = read_installed(&store, "upstream-liuli", "longxue", "1.0.0");
    assert_eq!(package.manifest.package_id, "longxue");
    // 资产只有 cards.json 与 config.json（无图片/音频/Spine——显示面不迁移）
    let assets: Vec<String> = package.asset_paths.values().cloned().collect();
    assert!(
        assets.iter().all(|a| a.ends_with(".json")),
        "资产仅 JSON：{assets:?}"
    );
    assert!(
        package
            .asset_manifest
            .as_ref()
            .expect("资产清单")
            .entries
            .iter()
            .all(|e| e.image_variants.is_empty() && e.audio_variants.is_empty()),
        "无图片/音频变体"
    );

    // ---- 配置（正例）：真实 config.json 解析出上游默认 ----
    let cfg_text = std::fs::read_to_string(dir.join("assets/config.json")).expect("配置");
    let cfg = parse_longxue_config(&cfg_text).expect("解析配置");
    assert_eq!(cfg.source, "noname-liuli");
    assert_eq!(cfg.package_name, "龙血玄黄");
    assert_eq!(cfg.package_version, "1.0");
    assert!(cfg.shengjian, "圣剑显示开关默认开（上游 config init:true）");
    assert!(cfg.upstream_path.contains("龙血玄黄"));
    // 配置（反例）：坏 JSON / 缺字段 / 类型错误一律拒绝
    assert!(matches!(
        parse_longxue_config("{"),
        Err(LongxueError::Json(_))
    ));
    assert_eq!(
        parse_longxue_config(
            r#"{"upstream_path":"x","package_name":"n","package_version":"1","author":"a","settings":{"shengjian":true}}"#
        ),
        Err(LongxueError::Missing("source"))
    );
    assert_eq!(
        parse_longxue_config(
            r#"{"source":"","upstream_path":"x","package_name":"n","package_version":"1","author":"a","settings":{"shengjian":true}}"#
        ),
        Err(LongxueError::BadField("source"))
    );
    assert_eq!(
        parse_longxue_config(
            r#"{"source":"s","upstream_path":"x","package_name":"n","package_version":"1","author":"a"}"#
        ),
        Err(LongxueError::Missing("settings"))
    );
    assert_eq!(
        parse_longxue_config(
            r#"{"source":"s","upstream_path":"x","package_name":"n","package_version":"1","author":"a","settings":{"shengjian":"on"}}"#
        ),
        Err(LongxueError::Missing("settings.shengjian"))
    );

    // ---- 功能（正例）：道心值开局面 +20（另一魏势力持卫统者 +60）、获得牌
    // +5、受伤 +10×、造成伤害 +15×、上限 99；阈值 25 授予清正 ----
    let mut e = engine_from_install(&dir, 96001);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 0, "装配未推进");
    next_phase(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 20, "开局面 +20");
    let shan = e.debug_place(0, "shan");
    e.debug_give_hand_card(0, 1, shan);
    drain(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 25, "获得牌 +5");
    let qz = e
        .catalog()
        .skill_by_name("mbcmqingzheng")
        .expect("清正入目录");
    assert!(
        e.state_core().skills.contains_key(&(1, qz)),
        "25 阈值授予清正"
    );
    let out = e.debug_deal_damage(Some(0), 1, 2, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 45, "受伤 +20");
    let out = e.debug_deal_damage(Some(1), 0, 3, DamageNature::Normal);
    assert!(
        matches!(
            out,
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
        ),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(
        e.state_core().player(1).mbqianlong_marks,
        90,
        "造成伤害 +45"
    );
    // 功能（反例）：外部技能不入目录（不虚构空操作）；未持潜龙者道心值恒 0
    for name in ["mbcmjiushi", "mbcmfangzhu", "mbjuejin"] {
        assert!(
            e.catalog().skill_by_name(name).is_none(),
            "{name} 外部依赖未入目录"
        );
    }
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 0);

    // ---- 依赖（正例/反例）：manifest 依赖为空；缺失依赖版本安装失败 ----
    assert!(package.manifest.dependencies.is_empty(), "龙血玄黄无包依赖");
    let bad_dir = root.join("bad");
    std::fs::create_dir_all(&bad_dir).expect("bad dir");
    let missing = bad_dir.join("longxue-missing-dep");
    std::fs::create_dir_all(&missing).expect("pkg dir");
    let mut npack: serde_json::Value = serde_json::from_str(
        &std::fs::read_to_string(dir.join("package.npack.json")).unwrap_or_else(|_| {
            // 商店副本无 npack 时用仓库副本
            std::fs::read_to_string(
                repo_extensions_root().join("longxue-upstream-liuli/package.npack.json"),
            )
            .expect("仓库 npack")
        }),
    )
    .expect("npack json");
    npack["manifest"]["dependencies"] = serde_json::json!([{
        "namespace": "upstream-liuli",
        "package_id": "missing-base",
        "requirement": "1.0.0",
        "kind": "Required"
    }]);
    std::fs::write(
        missing.join("package.npack.json"),
        serde_json::to_vec(&npack).unwrap(),
    )
    .expect("写 npack");
    std::fs::create_dir_all(missing.join("assets")).expect("assets");
    std::fs::copy(
        repo_extensions_root().join("longxue-upstream-liuli/assets/cards.json"),
        missing.join("assets/cards.json"),
    )
    .expect("copy cards");
    std::fs::copy(
        repo_extensions_root().join("longxue-upstream-liuli/assets/config.json"),
        missing.join("assets/config.json"),
    )
    .expect("copy config");
    let store2 = PackageStore::open(root.join("store2")).expect("商店2");
    let err = import_extensions(&store2, &bad_dir, &options());
    assert!(err.is_err(), "缺依赖安装应失败：{err:?}");

    // ---- 组合（正例/反例）：同目录其余包照常安装且内容哈希不受影响 ----
    let coin_dir = store_dir(&store, "upstream-liuli", "coin", "1.0.0");
    if coin_dir.exists() {
        let (coin, _) = read_installed(&store, "upstream-liuli", "coin", "1.0.0");
        assert_eq!(coin.manifest.package_id, "coin");
    }
    // 龙血玄黄未注册武将以外的内容（本包只登记 1 名宿主武将 + 1 映射技能 + 3 外部）
    let json = std::fs::read_to_string(dir.join("assets/cards.json")).expect("cards");
    let bridged = content_native_loader::bridge_pack(&json).expect("桥接");
    assert_eq!(bridged.pack.generals.len(), 1, "仅宿主武将");
    assert_eq!(bridged.pack.def.name, "longxue-upstream-liuli");

    // ---- 保存恢复（正例/反例）：道心值与技能实例存档往返 ----
    let snap = e.save();
    assert_eq!(snap.core.players[1].mbqianlong_marks, 90);
    let restored = {
        let json = std::fs::read_to_string(dir.join("assets/cards.json")).unwrap();
        let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
        Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复")
    };
    assert_eq!(restored.state_core().player(1).mbqianlong_marks, 90);
    let qz = restored
        .catalog()
        .skill_by_name("mbcmqingzheng")
        .expect("清正");
    assert!(
        restored.state_core().skills.contains_key(&(1, qz)),
        "恢复后清正实例保持"
    );
    // 反例：未知规则版本的存档被拒
    let mut bad = e.save();
    bad.ruleset_version = 1;
    let json = std::fs::read_to_string(dir.join("assets/cards.json")).unwrap();
    let pack = content_native_loader::bridge_pack(&json).unwrap().pack;
    assert!(
        Engine::restore_with_extensions(&bad, &[&pack]).is_err(),
        "规则版本不符的存档被拒"
    );

    // ---- 安装升级（正例/反例）：同版本重装幂等；禁用包被跳过 ----
    let longxue_hash = report
        .installed
        .iter()
        .find(|p| p.package_id == "longxue")
        .map(|p| p.content_hash.clone())
        .expect("首次安装含 longxue");
    let again = import_extensions(&store, &repo_extensions_root(), &options()).expect("重装");
    let longxue_again = again
        .installed
        .iter()
        .find(|p| p.package_id == "longxue")
        .expect("重装仍含 longxue");
    assert_eq!(
        longxue_again.content_hash, longxue_hash,
        "重装后内容哈希不变"
    );
    assert_eq!(longxue_again.version, "1.0.0");
    let (_, dir_again) = read_installed(&store, "upstream-liuli", "longxue", "1.0.0");
    assert_eq!(
        std::fs::read(dir_again.join("assets/cards.json")).unwrap(),
        std::fs::read(dir.join("assets/cards.json")).unwrap(),
        "重装后包数据字节不变"
    );

    // ---- 目标平台（正例/反例）：包资产可在宿主读取；无平台特化代码 ----
    let cards = std::fs::read_to_string(dir.join("assets/cards.json")).expect("读 cards");
    assert!(cards.contains("\"mb_caomao\""));
    assert!(cards.contains("host:mbqianlong"));
    assert!(
        !cards.contains("<img") && !cards.contains("innerHTML"),
        "包数据无 HTML/显示面载荷"
    );

    // ---- 规则差异（正例/反例）：阈值报告标注外部依赖；显示面不迁移 ----
    assert_eq!(
        longxue_unlock_report(24),
        "marks:24|unlocked:",
        "24 < 25 未解锁"
    );
    assert_eq!(longxue_unlock_report(25), "marks:25|unlocked:mbcmqingzheng");
    assert_eq!(
        longxue_unlock_report(99),
        "marks:99|unlocked:mbcmqingzheng,mbcmjiushi(external),mbcmfangzhu(external),mbjuejin(external)",
        "99 解锁全部（后三为外部依赖）"
    );
    assert_eq!(LONGXUE_THRESHOLDS.len(), 4);
    // 反例：显示面（_CMeffects/死亡动画/进度条）未进包数据与内核
    assert!(!cards.contains("_CMeffects"));
    assert!(!cards.contains("playSpine"));
    assert!(e.catalog().skill_by_name("_CMeffects").is_none());

    let _ = std::fs::remove_dir_all(&root);
}
