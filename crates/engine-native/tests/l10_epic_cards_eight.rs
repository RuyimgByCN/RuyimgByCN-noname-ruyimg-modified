//! L10 史诗卡牌八项整包（2026-10-17）。
//!
//! 从真实 `import_extensions` 安装 `upstream-liuli/epic-cards`。功能面为两个
//! 规则技能（`reguhuo` 蛊惑 / `rangjie` 让节，宿主行为 tag 515/516）；配置面
//! 走 `content-native-loader::epic_cards`（展示面开关 + 十常侍面四态依赖判定）；
//! 十常侍面因依赖 `k_danggu`/`k_mowang`（本仓未定义）为**结构性阻塞**，登记差异。

use std::path::PathBuf;

use content_native_loader::{
    parse_epic_config, ShichangshiPlan, EXTERNAL_SKILLS, OVERRIDE_CHARACTER, REQUIRED_EXTENSION,
};
use content_runtime::{import_extensions, InstallOptions, PackageStore};
use engine_native::event::DamageNature;
use engine_native::{ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase};
use std::sync::atomic::{AtomicU64, Ordering};

static DEDUP: AtomicU64 = AtomicU64::new(1);

fn next_dedup() -> u64 {
    DEDUP.fetch_add(1, Ordering::Relaxed)
}

fn repo_extensions_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packages/extensions")
}

fn upstream_extension() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js");
    std::fs::read_to_string(&path).unwrap_or_else(|_| panic!("读上游 {}", path.display()))
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn epic_pack() -> engine_native::content::extension::ExtensionPack {
    let json = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/cards.json"),
    )
    .expect("读史诗卡牌包");
    content_native_loader::bridge_pack(&json).expect("桥接").pack
}

fn engine(seed: u64) -> Engine {
    Engine::new_with_extensions(seed, 4, &[&epic_pack()]).expect("建局")
}

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel: Vec<ChoiceOption> = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: sel },
        });
    }
}

fn next_phase(e: &mut Engine) {
    let actor = e.turn().current;
    let out = e.submit(GameCommand {
        actor,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::NextPhase,
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
}

fn answer(e: &mut Engine, prompt: &str, sel: Vec<ChoiceOption>) -> bool {
    for _ in 0..32 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == prompt {
            let out = e.submit(GameCommand {
                actor: c.observer,
                expected_revision: e.revision(),
                dedup_id: next_dedup(),
                action: GameAction::Answer { choice_id: c.id, selection: sel },
            });
            assert!(
                matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
                "{out:?}"
            );
            return true;
        }
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let fallback: Vec<ChoiceOption> = if c.spec.prompt == "trigger_confirm" {
            vec![ChoiceOption::Confirm]
        } else if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = e.submit(GameCommand {
            actor: c.observer,
            expected_revision: e.revision(),
            dedup_id: next_dedup(),
            action: GameAction::Answer { choice_id: c.id, selection: fallback },
        });
    }
    false
}

/// L10 八项。
#[test]
fn l10_eight_dimension_from_fresh_install() {
    use content_runtime::RuntimeError;

    // ---- 上游回读（功能面存在性）----
    let upstream = upstream_extension();
    assert!(upstream.contains("lib.skill.reguhuo"));
    assert!(upstream.contains("lib.skill.rangjie"));
    assert!(upstream.contains("lib.config['extension_十周年UI_enable']"));
    assert!(upstream.contains("k_danggu"));
    assert!(upstream.contains("k_mowang"));

    // ---- 功能：蛊惑（声明牌种使用）----
    let mut e = engine(140001);
    e.debug_grant(0, "reguhuo");
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e);
        drain(&mut e);
    }
    e.debug_clear_hand(0);
    let card = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, card);
    e.debug_set_hp(0, 2);
    let hp0 = e.debug_hp(0);
    let def = e.catalog().skill_by_name("reguhuo").expect("蛊惑");
    let out = e.submit(GameCommand {
        actor: 0,
        expected_revision: e.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e, "reguhuo_card", vec![ChoiceOption::Card(card)]),
        "蛊惑选牌窗"
    );
    // 声明【桃】（下标 4）
    assert!(
        answer(&mut e, "reguhuo_name", vec![ChoiceOption::Tag(4)]),
        "蛊惑声明窗"
    );
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0 + 1, "蛊惑：声明桃回复 1");

    // ---- 功能：让节（受伤后获得指定类型牌）----
    let mut e2 = engine(140002);
    e2.debug_grant(0, "rangjie");
    for p in 0..4 {
        e2.debug_clear_hand(p);
    }
    let out = e2.debug_deal_damage(Some(1), 0, 1, DamageNature::Normal);
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e2, "trigger_confirm", vec![ChoiceOption::Confirm]),
        "让节确认门"
    );
    assert!(
        answer(&mut e2, "rangjie_choice", vec![ChoiceOption::Tag(0)]),
        "让节选择窗"
    );
    assert!(
        answer(&mut e2, "rangjie_type", vec![ChoiceOption::Tag(0)]),
        "让节类型窗"
    );
    drain(&mut e2);
    assert_eq!(e2.debug_hand(0).len(), 2, "让节：摸 1 + 得 1");

    // ---- 配置：展示面开关 + 十常侍面依赖判定 ----
    let stored_config = std::fs::read_to_string(
        repo_extensions_root().join("epic-cards-upstream-liuli/assets/config.json"),
    )
    .expect("读配置");
    let cfg = parse_epic_config(&stored_config).expect("解析配置");
    assert_eq!(cfg.upstream_commit, "8485dbb29ff16f9b258a766b1a17ef75f883aade");
    assert!(cfg.card_effects && cfg.skill_effects);
    assert!(!cfg.shichangshi, "默认关闭十常侍面");
    let pres = cfg.presentation();
    assert!(pres.card_effects && pres.skill_effects);
    assert!(!stored_config.contains("eval(") && !stored_config.contains("fetch("));
    let mut cfg_on = cfg.clone();
    cfg_on.shichangshi = true;
    assert_eq!(
        cfg_on.shichangshi_plan(false, |_| true),
        ShichangshiPlan::MissingCharacter(OVERRIDE_CHARACTER.to_string())
    );
    assert_eq!(
        cfg_on.shichangshi_plan(true, |_| false),
        ShichangshiPlan::MissingSkill(EXTERNAL_SKILLS[0].to_string())
    );
    match cfg_on.shichangshi_plan(true, |_| true) {
        ShichangshiPlan::Apply {
            required_extension, ..
        } => assert_eq!(required_extension, REQUIRED_EXTENSION),
        other => panic!("应为 Apply：{other:?}"),
    }

    // ---- 组合：与 swd 包同时装载（两包技能均可授予）----
    let swd_json = std::fs::read_to_string(
        repo_extensions_root().join("shahai-swd-upstream-noname/assets/cards.json"),
    )
    .expect("读 swd 包");
    let swd = content_native_loader::bridge_pack(&swd_json).expect("桥接 swd").pack;
    let both = Engine::new_with_extensions(140003, 4, &[&epic_pack(), &swd]).expect("双包建局");
    let mut both = both;
    both.debug_grant(0, "reguhuo");
    both.debug_grant(0, "rangjie");
    both.debug_grant(0, "jilve");
    for name in ["reguhuo", "rangjie", "jilve"] {
        let d = both.catalog().skill_by_name(name).expect(name);
        assert!(
            both.state_core().skills.contains_key(&(0, d)),
            "组合：{name} 在册"
        );
    }

    // ---- 保存恢复：带包存档往返 ----
    let mut e3 = engine(140004);
    e3.debug_grant(0, "rangjie");
    e3.debug_set_hp(0, 2);
    let snap = e3.save();
    let restored = Engine::restore_with_extensions(&snap, &[&epic_pack()]).expect("恢复");
    let d = restored.catalog().skill_by_name("rangjie").expect("rangjie");
    assert!(
        restored.state_core().skills.contains_key(&(0, d)),
        "保存恢复：技能实例保留"
    );
    assert_eq!(restored.state_core().player(0).hp, 2);

    // ---- 安装：真实 import ----
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let report = import_extensions(&store, &repo_extensions_root(), &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|item| item.package_id == "epic-cards" && item.namespace == "upstream-liuli")
        .expect("史诗卡牌已安装");
    assert_eq!(meta.version, "1.0.0");
    let npack_bytes =
        std::fs::read(repo_extensions_root().join("epic-cards-upstream-liuli/package.npack.json"))
            .unwrap();
    assert_eq!(
        meta.content_hash,
        content_runtime::content_hash_hex(&npack_bytes)
    );

    // ---- 依赖：本包无 manifest 依赖；十常侍面依赖为内容级（已登记）----
    // 商店目录布局：`<root>/packages/<namespace>__<package_id>/<version>`
    let installed_dir = store
        .root()
        .join("packages")
        .join("upstream-liuli__epic-cards")
        .join("1.0.0");
    let installed = content_schema::ContentPackage::from_json_bytes(
        &std::fs::read(installed_dir.join("package.json")).unwrap(),
    )
    .expect("读商店包");
    assert!(installed.manifest.dependencies.is_empty());
    assert_eq!(installed.skill_index.len(), 2, "两个技能条目");
    let names: Vec<&str> = installed
        .skill_index
        .iter()
        .map(|s| s.name.as_str())
        .collect();
    assert!(names.contains(&"reguhuo") && names.contains(&"rangjie"));
    assert!(installed.card_index.is_empty() && installed.character_index.is_empty());

    // ---- 保存恢复（商店）：重开商店仍 1.0.0 ----
    let lock_before = store.current_lock().expect("lock").lock_hash;
    let reopened = PackageStore::open(dir.path().join("store")).unwrap();
    assert_eq!(
        reopened.current_lock().expect("lock").lock_hash,
        lock_before
    );

    // ---- 安装升级：2.0.0 覆盖 + rollback ----
    let mut upgraded = installed.clone();
    upgraded.manifest.version = "2.0.0".into();
    upgraded.manifest.logic_hash = upgraded.compute_logic_hash();
    upgraded.manifest.asset_hash = upgraded.compute_asset_hash();
    store
        .install_with_assets(
            &upgraded.to_json_bytes().unwrap(),
            None,
            &options(),
            &installed_dir,
        )
        .expect("安装 2.0.0");
    assert_eq!(
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("epic-cards"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default(),
        "2.0.0"
    );
    store.rollback().expect("rollback");
    assert_eq!(
        store
            .read_active()
            .unwrap()
            .packages
            .iter()
            .find(|(slot, _)| slot.ends_with("epic-cards"))
            .map(|(_, v)| v.clone())
            .unwrap_or_default(),
        "1.0.0"
    );

    // ---- 依赖（错误路径）：未安装版本 NotFound ----
    match store.activate_version("upstream-liuli", "epic-cards", "9.9.9") {
        Err(RuntimeError::NotFound(_)) => {}
        other => panic!("未安装版本应 NotFound: {other:?}"),
    }

    // ---- 规则差异：蛊惑声明面（Tag 表长度）与让节「牌堆取首张」----
    let mut e4 = engine(140005);
    e4.debug_grant(0, "reguhuo");
    for _ in 0..12 {
        if e4.turn().current == 0 && e4.turn().phase == Phase::Play {
            break;
        }
        next_phase(&mut e4);
        drain(&mut e4);
    }
    let def = e4.catalog().skill_by_name("reguhuo").expect("reguhuo");
    let out = e4.submit(GameCommand {
        actor: 0,
        expected_revision: e4.revision(),
        dedup_id: next_dedup(),
        action: GameAction::ActivateSkill { skill: def },
    });
    assert!(
        matches!(out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }),
        "{out:?}"
    );
    assert!(
        answer(&mut e4, "reguhuo_card", Vec::new()),
        "蛊惑选牌窗（取消）"
    );
    drain(&mut e4);
}
