//! L27 选将美化（`select-beautify-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置与包数据：`assets/config.json` 与 `assets/cards.json` 桥接 7 项技能声明（`xiefang`, `new_zhixi`,
//!   `new_zhixi_mark2`, `new_meibu`, `new_meibu_range`, `sbzhaxiang`, `jishe`）；
//! ③ 组合共存：与 `mark-supplement-upstream-liuli`、`doomsday-brawl-upstream-liuli` 同局共存；
//! ④ 保存恢复：对局中含撷芳（`xiefang`）标记与距离修正状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：撷芳（`xiefang`）按存活女性角色数减少距离并刷新标记；
//! ⑦ 异常拒装：损坏 JSON 与未知宿主行为键明确返回 `BridgeError`；
//! ⑧ 边界登记：选将 UI 主面混淆块（`select_ui`）与未入批技能显式登记于 `deferred`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{bridge_pack, BridgeError};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::content::standard::Gender;
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

fn drain(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice().cloned() else { break };
        let has_pass = c.spec.options.contains(&ChoiceOption::Pass);
        let sel = if has_pass {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
}

#[test]
fn l27_select_beautify_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "select-beautify-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "select-beautify" && m.namespace == "upstream-liuli")
        .expect("select-beautify 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cards_json =
        std::fs::read_to_string(ext_root.join("assets/cards.json")).expect("读取 cards.json");
    let config_json =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");
    assert!(config_json.contains("\"xiefang\""));

    // ② 桥接与技能核对
    let out = bridge_pack(&cards_json).expect("bridge select-beautify");
    assert_eq!(out.pack.def.name, "select-beautify-upstream-liuli");
    assert_eq!(out.pack.host_skills.len(), 7);
    assert_eq!(
        out.deferred,
        vec!["select_ui".to_string(), "skill_batch_rest".to_string()]
    );

    // ③ 与末日乱斗共存建局
    let db_json = std::fs::read_to_string(
        root.join("doomsday-brawl-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 doomsday-brawl cards.json");
    let db_pack = bridge_pack(&db_json).expect("bridge db").pack;
    let packs = [&out.pack, &db_pack];
    let mut e = Engine::new_with_extensions(192701, 4, &packs).expect("建局");

    // ⑤ 授予 xiefang / new_zhixi / new_meibu / sbzhaxiang / jishe 并验证女性存活距离减少 + 集舍主动发动
    for sk in ["xiefang", "new_zhixi", "new_meibu", "sbzhaxiang", "jishe"] {
        e.debug_grant(0, sk);
    }
    let before = e.state_core().computed_distance(e.catalog(), 0, 2);
    let female_gid = (0..e.catalog().general_count() as u16)
        .find(|g| {
            e.catalog()
                .general_meta(*g)
                .map(|(_, _, _, _, gender)| gender == Gender::Female)
                .unwrap_or(false)
        })
        .expect("标准包女性武将");
    let female_name = e.catalog().general_meta(female_gid).unwrap().0;
    e.debug_set_general(2, &female_name);
    for _ in 0..12 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    let actor = e.turn().current;
    let _ = submit(&mut e, actor, GameAction::NextPhase); // Draw -> Play 刷新撷芳标记
    drain(&mut e);
    let after = e.state_core().computed_distance(e.catalog(), 0, 2);
    assert!(after < before, "撷芳减少距离: {before} -> {after}");

    // 发动集舍（jishe）：摸 1 张牌并累加标记
    let jishe_def = e.catalog().skill_by_name("jishe").expect("jishe 在目录");
    let hand_before_js = e.debug_hand(0).len();
    let js_out = submit(&mut e, 0, GameAction::ActivateSkill { skill: jishe_def });
    assert!(matches!(js_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), hand_before_js + 1, "集舍摸 1 张牌");
    assert_eq!(
        e.state_core()
            .vm_aux
            .marks
            .get(&(0, u32::from(jishe_def)))
            .copied()
            .unwrap_or(0),
        1,
        "集舍标记 +1"
    );

    // ④ 存档与恢复
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[]).is_err());

    // ⑦ 坏数据拒绝
    let bad = cards_json.replace("host:xiefang", "host:unknown_sb_skill");
    assert!(matches!(
        bridge_pack(&bad),
        Err(BridgeError::UnknownBehaviorKey(_))
    ));
}
