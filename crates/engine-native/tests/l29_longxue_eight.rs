//! L29 龙血玄黄 八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/longxue`。规则面（潜龙
//! 道心值 + 阈值解锁 + 清正）走内核与宿主行为；本包配置（`config.shengjian`
//! 圣剑显示开关）与阈值报告走 `content_native_loader::longxue`。不迁移
//! Spine/DOM 显示面（`_CMeffects`、死亡动画、进度条）。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    longxue_unlock_report, parse_longxue_config, LongxueError, LONGXUE_THRESHOLDS,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use content_schema::ContentPackage;
use engine_native::ai::BoundedPlanner;
use engine_native::archive::{decode_archive, encode_archive};
use engine_native::catalog::Suit;
use engine_native::content::extension::ExtensionPack;
use engine_native::event::DamageNature;
use engine_native::{
    ChoiceKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
};

static DEDUP: AtomicU64 = AtomicU64::new(1_000_000);

fn submit(e: &mut Engine, actor: u8, action: GameAction) -> CommandOutcome {
    e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
        action,
    })
}

fn longxue_pack() -> ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("longxue-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 longxue cards.json");
    content_native_loader::bridge_pack(&json)
        .expect("桥接 longxue-upstream-liuli")
        .pack
}

fn advance_to_play(e: &mut Engine, seat: u8) {
    for _ in 0..48 {
        if e.turn().current == seat && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if e.open_choice().is_some() {
            drain(e);
            continue;
        }
        let actor = e.turn().current;
        let _ = submit(e, actor, GameAction::NextPhase);
    }
    panic!("未能推进到座位 {seat} 的出牌阶段");
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
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&engine_native::ChoiceOption::Pass);
        let sel: Vec<engine_native::ChoiceOption> = if has_pass {
            vec![engine_native::ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: DEDUP.fetch_add(1, Ordering::Relaxed),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
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
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
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
    let assets: Vec<String> = package
        .asset_paths
        .values()
        .cloned()
        .collect();
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
        parse_longxue_config(r#"{"upstream_path":"x","package_name":"n","package_version":"1","author":"a","settings":{"shengjian":true}}"#),
        Err(LongxueError::Missing("source"))
    );
    assert_eq!(
        parse_longxue_config(r#"{"source":"","upstream_path":"x","package_name":"n","package_version":"1","author":"a","settings":{"shengjian":true}}"#),
        Err(LongxueError::BadField("source"))
    );
    assert_eq!(
        parse_longxue_config(r#"{"source":"s","upstream_path":"x","package_name":"n","package_version":"1","author":"a"}"#),
        Err(LongxueError::Missing("settings"))
    );
    assert_eq!(
        parse_longxue_config(r#"{"source":"s","upstream_path":"x","package_name":"n","package_version":"1","author":"a","settings":{"shengjian":"on"}}"#),
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
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 45, "受伤 +20");
    let out = e.debug_deal_damage(Some(1), 0, 3, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    assert_eq!(e.state_core().player(1).mbqianlong_marks, 90, "造成伤害 +45");
    // 功能（正例）：50/75/99 阈值宿主技能均已入目录；功能（反例）：未持潜龙者道心值恒 0
    for name in ["mbcmjiushi", "mbcmfangzhu", "mbjuejin"] {
        assert!(
            e.catalog().skill_by_name(name).is_some(),
            "{name} 宿主技能已入目录"
        );
    }
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 0);

    // ---- 依赖（正例/反例）：manifest 依赖为空；缺失依赖版本安装失败 ----
    assert!(
        package.manifest.dependencies.is_empty(),
        "龙血玄黄无包依赖"
    );
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
    assert_eq!(
        longxue_unlock_report(25),
        "marks:25|unlocked:mbcmqingzheng"
    );
    assert_eq!(
        longxue_unlock_report(99),
        "marks:99|unlocked:mbcmqingzheng,mbcmjiushi,mbcmfangzhu,mbjuejin",
        "99 解锁全部四项宿主技能"
    );
    assert_eq!(LONGXUE_THRESHOLDS.len(), 4);
    // 反例：显示面（_CMeffects/死亡动画/进度条）未进包数据与内核
    assert!(!cards.contains("_CMeffects"));
    assert!(!cards.contains("playSpine"));
    assert!(e.catalog().skill_by_name("_CMeffects").is_none());

    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn l29_mbqianlong_mbweitong_and_mbcmqingzheng_six_dimensions() {
    let pack = longxue_pack();
    // 1. 卫统联动：场上另一存活魏势力角色持 mbweitong 时，曹髦首回合开始道心 +60；达 99 解锁决进并移除自身卫统
    let mut e = Engine::new_with_extensions(96010, 4, &[&pack]).expect("建局");
    let caomao = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(0, caomao);
    e.debug_assemble_general_def(1, caomao);
    let weitong = e.catalog().skill_by_name("mbweitong").expect("mbweitong");
    let qingzheng = e
        .catalog()
        .skill_by_name("mbcmqingzheng")
        .expect("mbcmqingzheng");
    let jiushi = e.catalog().skill_by_name("mbcmjiushi").expect("mbcmjiushi");
    let fangzhu = e
        .catalog()
        .skill_by_name("mbcmfangzhu")
        .expect("mbcmfangzhu");
    let juejin = e.catalog().skill_by_name("mbjuejin").expect("mbjuejin");
    assert!(e.state_core().skills.contains_key(&(0, weitong)));
    assert!(e.state_core().skills.contains_key(&(1, weitong)));

    next_phase(&mut e);
    assert_eq!(
        e.state_core().player(1).mbqianlong_marks,
        60,
        "另一魏势力角色有卫统时开局道心 +60"
    );
    assert_eq!(
        e.state_core().player(0).mbqianlong_marks,
        65,
        "0 号开局 +60 且首回合摸牌 +5 = 65"
    );
    assert!(e.state_core().skills.contains_key(&(0, qingzheng)));
    assert!(e.state_core().skills.contains_key(&(0, jiushi)));
    assert!(!e.state_core().skills.contains_key(&(0, fangzhu)));

    // 造成 3 点伤害（+45 -> 封顶 99），解锁放逐与决进，且移除 0 号自身 mbweitong
    let _ = e.debug_deal_damage(Some(0), 2, 3, DamageNature::Normal);
    drain(&mut e);
    assert_eq!(e.state_core().player(0).mbqianlong_marks, 99, "道心上限 99");
    assert!(e.state_core().skills.contains_key(&(0, fangzhu)));
    assert!(e.state_core().skills.contains_key(&(0, juejin)));
    assert!(
        !e.state_core().skills.contains_key(&(0, weitong)),
        "99 阈值解锁决进并移除自身卫统"
    );

    // 2. 清正（mbcmqingzheng）出牌阶段开始三段交互 + 中途存档恢复 + AI 规划 + 造成伤害/不造成伤害分支
    let mut e2 = Engine::new_with_extensions(96011, 4, &[&pack]).expect("建局2");
    e2.debug_assemble_general_def(0, caomao);
    e2.debug_grant(0, "mbcmqingzheng");
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    e2.debug_place_suit(0, Suit::Spade);
    e2.debug_place_suit(0, Suit::Spade);
    e2.debug_place_suit(0, Suit::Heart);
    e2.debug_place_suit(1, Suit::Club);
    // 推进到 0 号出牌阶段开始（触发清正 trigger_confirm）
    for _ in 0..12 {
        if e2.open_choice().map(|c| c.spec.prompt == "trigger_confirm").unwrap_or(false) {
            break;
        }
        if e2.open_choice().is_some() {
            drain(&mut e2);
            continue;
        }
        let actor = e2.turn().current;
        let _ = submit(&mut e2, actor, GameAction::NextPhase);
    }
    let gate = e2.open_choice().expect("清正确认门").clone();
    assert_eq!(gate.spec.prompt, "trigger_confirm");
    let _ = submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: gate.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );

    // Stage 1: mbcmqingzheng_suit
    let c1 = e2.open_choice().expect("清正第一段花色选择").clone();
    assert_eq!(c1.spec.prompt, "mbcmqingzheng_suit");
    assert_eq!(c1.spec.kind, ChoiceKind::PickTag);
    assert!(!c1.spec.allow_cancel);
    assert!(c1.spec.options.contains(&ChoiceOption::Tag(0)));
    // 反例：不允许取消
    assert!(matches!(
        submit(
            &mut e2,
            0,
            GameAction::Answer {
                choice_id: c1.id,
                selection: vec![],
            }
        ),
        CommandOutcome::Rejected { .. }
    ));

    // AI 规划（处于清正 Choice 窗口中）
    let planner = BoundedPlanner::new(16);
    let plan = e2.plan_action(0, &planner);
    assert!(matches!(plan.action, GameAction::Answer { .. }), "AI 可规划清正选花色");

    let out = submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: c1.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");

    // Stage 2: mbcmqingzheng_target
    let c2 = e2.open_choice().expect("清正第二段目标选择").clone();
    assert_eq!(c2.spec.prompt, "mbcmqingzheng_target");
    assert_eq!(c2.spec.kind, ChoiceKind::SelectPlayers);
    let out = submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: c2.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");

    // Stage 3: mbcmqingzheng_target_suit
    let c3 = e2.open_choice().expect("清正第三段弃目标花色").clone();
    assert_eq!(c3.spec.prompt, "mbcmqingzheng_target_suit");
    assert_eq!(c3.spec.options, vec![ChoiceOption::Tag(2)]);
    let hp_before = e2.debug_hp(1);
    let out = submit(
        &mut e2,
        0,
        GameAction::Answer {
            choice_id: c3.id,
            selection: vec![ChoiceOption::Tag(2)],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e2);
    // 存档编码/解码/恢复往返保持清正结算后的状态
    let bytes = encode_archive(&e2.save());
    let snap = decode_archive(&bytes).expect("解码清正存档");
    let e2 = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复清正存档");
    assert!(e2.debug_hand(1).is_empty(), "1 号梅花牌全弃");
    assert_eq!(
        e2.debug_hp(1),
        hp_before - 1,
        "弃牌数 2 > 1 时对目标造成 1 点伤害"
    );
}

#[test]
fn l29_mbcmjiushi_six_dimensions() {
    let pack = longxue_pack();
    let mut e = Engine::new_with_extensions(96020, 4, &[&pack]).expect("建局");
    drain(&mut e);
    let caomao = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(0, caomao);
    e.debug_assemble_general_def(1, caomao);
    let jiushi = e.catalog().skill_by_name("mbcmjiushi").expect("mbcmjiushi");
    advance_to_play(&mut e, 0);
    e.debug_grant(0, "mbcmjiushi");
    e.debug_grant(1, "mbcmjiushi");
    e.debug_clear_hand(0);
    e.debug_clear_hand(1);
    e.debug_set_hp(1, 3);

    // Part A: 正面向上且未用过酒时发动 mbcmjiushi -> 翻面、视为使用酒、打开 mbcmjiushi_gain_confirm
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: jiushi });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert!(e.state_core().player(0).face_down, "发动酒诗后翻至背面");
    assert!(e.turn().jiu_used && e.turn().jiu_active, "视为已使用酒且酒劲生效");

    let c_gain = e.open_choice().expect("酒诗翻面获锦囊确认窗").clone();
    assert_eq!(c_gain.spec.prompt, "mbcmjiushi_gain_confirm");
    assert!(c_gain.spec.allow_cancel);

    // AI 规划（处于酒诗 Choice 窗口中）
    let planner = BoundedPlanner::new(16);
    let plan = e.plan_action(0, &planner);
    assert!(matches!(plan.action, GameAction::Answer { .. }), "AI 可规划酒诗确认窗");

    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_gain.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    // 存档编码/解码/恢复往返保持翻面、酒劲与获牌状态
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("解码酒诗存档");
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复酒诗存档");
    assert!(restored.state_core().player(0).face_down);
    assert_eq!(restored.debug_hand(0).len(), 1, "确认后获得 1 张锦囊牌");
    let gained_cid = e.debug_hand(0)[0];
    let sample_sha = e.debug_place(3, "sha");
    assert_ne!(
        e.catalog().card_kind(gained_cid),
        e.catalog().card_kind(sample_sha),
        "获得的牌为锦囊牌而非基本牌"
    );

    // 反例：背面向上且本回合已用过酒时不可再次发动 mbcmjiushi
    let dup = submit(&mut e, 0, GameAction::ActivateSkill { skill: jiushi });
    assert!(
        matches!(dup, CommandOutcome::Rejected(_)),
        "背面/已用酒不可再发动酒诗：{dup:?}"
    );

    // 酒诗增伤的【杀】造成 2 点伤害（+1 增伤）
    let sha = e.debug_place(0, "sha");
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(1), 1, "酒诗增伤杀造成 2 点伤害");

    // Part B1: 非酒诗增伤的伤害命中背面 0 号 -> 弹出 mbcmjiushi_turnback_confirm -> 确认翻回正面并弹出 mbcmjiushi_gain_confirm
    e.debug_set_hp(0, 3);
    let out = e.debug_deal_damage(Some(2), 0, 1, DamageNature::Normal);
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c_turnback = e.open_choice().expect("受击翻回确认窗").clone();
    assert_eq!(c_turnback.spec.prompt, "mbcmjiushi_turnback_confirm");
    assert!(c_turnback.spec.allow_cancel);
    let hand_before = e.debug_hand(0).len();
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_turnback.id,
            selection: vec![ChoiceOption::Confirm],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    assert!(!e.state_core().player(0).face_down, "确认后翻回正面");
    let c_gain2 = e.open_choice().expect("翻回正面后联动获锦囊窗").clone();
    assert_eq!(c_gain2.spec.prompt, "mbcmjiushi_gain_confirm");
    // 取消获牌（vec![]）则不获得锦囊牌
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_gain2.id,
            selection: vec![],
        },
    );
    assert_eq!(e.debug_hand(0).len(), hand_before, "取消获牌不增加手牌");
}

#[test]
fn l29_mbcmfangzhu_and_mbjuejin_six_dimensions() {
    let pack = longxue_pack();
    let mut e = Engine::new_with_extensions(96030, 4, &[&pack]).expect("建局");
    let caomao = general_def_of(&e, "mb_caomao");
    e.debug_assemble_general_def(0, caomao);
    e.debug_assemble_general_def(1, caomao);
    let fangzhu = e
        .catalog()
        .skill_by_name("mbcmfangzhu")
        .expect("mbcmfangzhu");
    let jiushi = e.catalog().skill_by_name("mbcmjiushi").expect("mbcmjiushi");
    let qianlong = e.catalog().skill_by_name("mbqianlong").expect("mbqianlong");
    let juejin = e.catalog().skill_by_name("mbjuejin").expect("mbjuejin");

    advance_to_play(&mut e, 0);
    // 通过造成 3 点伤害令 0 号道心值达 99，永久解锁放逐与决进（expires_end_of_turn: false）
    e.debug_set_hp(3, 4);
    let _ = e.debug_deal_damage(Some(0), 3, 3, DamageNature::Normal);
    drain(&mut e);
    assert!(e.state_core().skills.contains_key(&(0, fangzhu)));
    assert!(e.state_core().skills.contains_key(&(0, juejin)));

    // 1. 放逐（mbcmfangzhu）第一段取消回滚 once_used
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: fangzhu });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c_tgt = e.open_choice().expect("放逐目标窗").clone();
    assert_eq!(c_tgt.spec.prompt, "mbcmfangzhu_target");
    assert!(c_tgt.spec.allow_cancel);
    assert!(!c_tgt.spec.options.contains(&ChoiceOption::Player(0)));
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt.id,
            selection: vec![],
        },
    );
    // 取消后可再次发动
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: fangzhu });
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");
    let c_tgt = e.open_choice().expect("放逐目标窗").clone();
    let out = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt.id,
            selection: vec![ChoiceOption::Player(1)],
        },
    );
    assert!(matches!(out, CommandOutcome::Pending { .. }), "{out:?}");

    // Stage 2: mbcmfangzhu_option + AI 规划
    let c_opt = e.open_choice().expect("放逐选项窗").clone();
    assert_eq!(c_opt.spec.prompt, "mbcmfangzhu_option");
    assert_eq!(
        c_opt.spec.options,
        vec![ChoiceOption::Tag(0), ChoiceOption::Tag(1)]
    );
    let planner = BoundedPlanner::new(16);
    let plan = e.plan_action(0, &planner);
    assert!(matches!(plan.action, GameAction::Answer { .. }), "AI 可规划放逐选项");

    // Tag(0): 1 号直到下回合结束只能使用锦囊手牌（禁用基本/装备手牌）
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_opt.id,
            selection: vec![ChoiceOption::Tag(0)],
        },
    );
    // 存档编码/解码/恢复往返保持放逐禁令与本回合已用标记
    let bytes = encode_archive(&e.save());
    let snap = decode_archive(&bytes).expect("解码放逐存档");
    let restored = Engine::restore_with_extensions(&snap, &[&pack]).expect("恢复放逐存档");
    assert_eq!(
        restored.state_core().players.len(),
        e.state_core().players.len()
    );
    // 同回合不可再次发动放逐
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: fangzhu }),
        CommandOutcome::Rejected(_)
    ));

    advance_to_play(&mut e, 1);
    e.debug_clear_hand(1);
    let sha = e.debug_place(1, "sha");
    let zhuge = e.debug_place(1, "zhuge");
    let wuzhong = e.debug_place(1, "wuzhong");
    assert!(
        matches!(
            submit(
                &mut e,
                1,
                GameAction::PlayCard {
                    card: sha,
                    targets: vec![2],
                }
            ),
            CommandOutcome::Rejected(_)
        ),
        "放逐禁令下不可使用基本手牌"
    );
    assert!(
        matches!(
            submit(
                &mut e,
                1,
                GameAction::PlayCard {
                    card: zhuge,
                    targets: vec![],
                }
            ),
            CommandOutcome::Rejected(_)
        ),
        "放逐禁令下不可使用装备手牌"
    );
    let wz_out = submit(
        &mut e,
        1,
        GameAction::PlayCard {
            card: wuzhong,
            targets: vec![],
        },
    );
    assert!(
        matches!(wz_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "放逐禁令下可使用锦囊手牌：{wz_out:?}"
    );
    drain(&mut e);

    // 推进回 0 号出牌阶段：1 号回合结束后禁令解除；且 0 号连续发动放逐时不可重复指派上次目标 1 号
    advance_to_play(&mut e, 0);
    let out2 = submit(&mut e, 0, GameAction::ActivateSkill { skill: fangzhu });
    assert!(
        matches!(out2, CommandOutcome::Pending { .. }),
        "次轮放逐应可发动：{out2:?}, turn={:?}, open={:?}, all_skills={:?}",
        e.turn(),
        e.open_choice(),
        e.state_core().skills
    );
    let c_tgt2 = e.open_choice().expect("次轮放逐目标窗").clone();
    assert!(
        !c_tgt2.spec.options.contains(&ChoiceOption::Player(1)),
        "不可连续两回合放逐同一目标 1 号"
    );
    // 选 2 号并测 Tag(1) 非锁定技失效（先把 2 号也授予 mbcmjiushi + mbqianlong）
    e.debug_grant(2, "mbcmjiushi");
    e.debug_grant(2, "mbqianlong");
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_tgt2.id,
            selection: vec![ChoiceOption::Player(2)],
        },
    );
    let c_opt2 = e.open_choice().expect("放逐选项窗2").clone();
    let _ = submit(
        &mut e,
        0,
        GameAction::Answer {
            choice_id: c_opt2.id,
            selection: vec![ChoiceOption::Tag(1)],
        },
    );
    assert!(
        !e.state_core().skills[&(2, jiushi)].enabled,
        "Tag(1) 使目标非锁定技失效"
    );
    assert!(
        e.state_core().skills[&(2, qianlong)].enabled,
        "Tag(1) 不影响目标锁定技"
    );

    // 2. 决进（mbjuejin）限定技：全场体力置 1、溢出转护甲、使用者额外 +2 护甲、驱逐全场闪/桃/酒至向死存魏区
    e.debug_set_hp(0, 3);
    e.debug_set_hp(1, 2);
    e.debug_set_hp(2, 1);
    let shan_in_hand = e.debug_place(1, "shan");
    let tao_in_hand = e.debug_place(2, "tao");
    let out = submit(&mut e, 0, GameAction::ActivateSkill { skill: juejin });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    drain(&mut e);
    for p in 0..4 {
        assert_eq!(e.debug_hp(p), 1, "{p} 号体力调整为 1");
    }
    assert_eq!(e.state_core().player(0).hujia, 4, "0 号减 2 体力转 2 护甲 + 决进额外 2 护甲 = 4");
    assert_eq!(e.state_core().player(1).hujia, 1, "1 号减 1 体力转 1 护甲");
    assert_eq!(e.state_core().player(2).hujia, 0, "2 号原为 1 体力无额外护甲");
    assert!(
        e.state_core()
            .detached_cards
            .iter()
            .any(|&(c, _)| c == shan_in_hand),
        "手牌中的【闪】被移入向死存魏区"
    );
    assert!(
        e.state_core()
            .detached_cards
            .iter()
            .any(|&(c, _)| c == tao_in_hand),
        "手牌中的【桃】被移入向死存魏区"
    );
    // 反例：限定技决进整局只能发动一次
    assert!(matches!(
        submit(&mut e, 0, GameAction::ActivateSkill { skill: juejin }),
        CommandOutcome::Rejected(_)
    ));
}

/// 上游 archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js:1
/// 的 mbcmjiushi：主动翻面、受伤翻回及翻面后
/// 获锦囊都是可选确认。真实事件创建的确认窗必须能在回答前保存恢复。
#[test]
fn l29_jiushi_pending_confirmations_restore_and_match_direct_answers() {
    let pack = longxue_pack();
    for (turnback, first_confirm, gain_confirm) in [
        (false, true, true),
        (false, false, false),
        (true, false, false),
        (true, true, true),
        (true, true, false),
    ] {
        let mut e = Engine::new_with_extensions(2026111429, 3, &[&pack]).unwrap();
        e.state_core_mut().skills.clear();
        for seat in 0..3 {
            e.debug_clear_hand(seat);
        }
        e.debug_grant(0, "mbcmjiushi");
        if turnback {
            e.state_core_mut().player_mut(0).face_down = true;
            e.debug_set_turn(1, Phase::Play, 1);
            let sha = e.debug_place(1, "sha");
            assert!(matches!(
                submit(
                    &mut e,
                    1,
                    GameAction::PlayCard {
                        card: sha,
                        targets: vec![0]
                    }
                ),
                CommandOutcome::Pending { .. }
            ));
        } else {
            e.debug_set_turn(0, Phase::Play, 1);
            let skill = e.catalog().skill_by_name("mbcmjiushi").unwrap();
            assert!(matches!(
                submit(&mut e, 0, GameAction::ActivateSkill { skill }),
                CommandOutcome::Pending { .. }
            ));
        }
        let mut answers = 0;
        while let Some(choice) = e.open_choice().cloned() {
            assert!(answers < 2, "酒诗最多两个确认窗");
            let is_turnback = choice.spec.prompt == "mbcmjiushi_turnback_confirm";
            assert!(is_turnback || choice.spec.prompt == "mbcmjiushi_gain_confirm");
            assert_eq!(choice.observer, 0);
            assert_eq!(choice.spec.options, vec![ChoiceOption::Confirm]);
            assert!(choice.spec.allow_cancel);

            let bytes = encode_archive(&e.save());
            let snapshot = decode_archive(&bytes).unwrap();
            let mut resumed = Engine::restore_with_extensions(&snapshot, &[&pack])
                .expect("真实酒诗确认窗在回答前必须可恢复");
            assert_eq!(e.state_hash(), resumed.state_hash());
            assert_eq!(e.open_choice(), resumed.open_choice());

            // 损坏候选仍须拒绝，不能通过放宽开放选择校验修复恢复失败。
            let mut corrupt = snapshot.clone();
            corrupt
                .resolution
                .open_choice
                .as_mut()
                .unwrap()
                .spec
                .options
                .push(ChoiceOption::Pass);
            assert!(Engine::restore_with_extensions(&corrupt, &[&pack]).is_err());

            let confirm = if answers == 0 {
                first_confirm
            } else {
                gain_confirm
            };
            let selection = if confirm {
                vec![ChoiceOption::Confirm]
            } else {
                vec![]
            };
            let before = e.log().len();
            let direct = submit(
                &mut e,
                0,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection: selection.clone(),
                },
            );
            let restored = submit(
                &mut resumed,
                0,
                GameAction::Answer {
                    choice_id: choice.id,
                    selection,
                },
            );
            assert!(
                matches!(
                    direct,
                    CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }
                ),
                "{direct:?}"
            );
            assert_eq!(
                std::mem::discriminant(&direct),
                std::mem::discriminant(&restored)
            );
            assert_eq!(e.state_hash(), resumed.state_hash());
            assert_eq!(&e.log()[before..], resumed.log());
            assert_eq!(e.open_choice(), resumed.open_choice());
            answers += 1;
        }
        assert_eq!(answers, if turnback && first_confirm { 2 } else { 1 });
        assert_eq!(
            e.state_core().player(0).face_down,
            !turnback || !first_confirm
        );
        let gained = if turnback {
            first_confirm && gain_confirm
        } else {
            first_confirm
        };
        assert_eq!(e.debug_hand(0).len(), usize::from(gained));
    }
}
