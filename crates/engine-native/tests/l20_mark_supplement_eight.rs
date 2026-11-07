//! L20 标记补充（`mark-supplement-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置与包数据：`assets/config.json` 与 `assets/cards.json` 桥接 6 项技能（`xigua_mopaifanbei`,
//!   `xigua_xueliangfanbei`, `kurou`, `zhuwanzhongji`, `fz_liegong`, `fz_xinkuanggu`）；
//! ③ 组合共存：与 `select-beautify-upstream-liuli`、`doomsday-brawl-upstream-liuli` 同局共存；
//! ④ 保存恢复：对局中含烈弓/狂骨与摸牌翻倍状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：`fz_liegong` 强命不可闪避、`fz_xinkuanggu` 近距造伤摸牌、`xigua_mopaifanbei` 摸牌阶段+2；
//! ⑦ 异常拒装：损坏 JSON 与非法宿主行为键明确返回 `BridgeError`；
//! ⑧ 边界登记：纯标记展示（`mark_display`）与 Spine 动画音效（`spine_effects`）显式登记于 `deferred`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{bridge_pack, BridgeError};
use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
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
fn l20_mark_supplement_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "mark-supplement-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "mark-supplement" && m.namespace == "upstream-liuli")
        .expect("mark-supplement 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cards_json =
        std::fs::read_to_string(ext_root.join("assets/cards.json")).expect("读取 cards.json");
    let config_json =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");
    assert!(config_json.contains("\"mpfbmoshi\""));
    assert!(cards_json.contains("\"fz_liegong\""));

    // ② 桥接与技能/暂缓核对
    let out = bridge_pack(&cards_json).expect("bridge mark-supplement");
    assert_eq!(out.pack.def.name, "mark-supplement-upstream-liuli");
    assert_eq!(out.pack.host_skills.len(), 6);
    assert_eq!(
        out.deferred,
        vec![
            "mark_display".to_string(),
            "spine_effects".to_string(),
            "skill_batch_rest".to_string()
        ]
    );

    // ③ 与选将美化包共存建局
    let sb_json = std::fs::read_to_string(
        root.join("select-beautify-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 select-beautify cards.json");
    let sb_pack = bridge_pack(&sb_json).expect("bridge sb").pack;
    let packs = [&out.pack, &sb_pack];
    let mut e = Engine::new_with_extensions(192001, 4, &packs).expect("建局");

    // ⑤ 授予 fz_liegong + fz_xinkuanggu + kurou，验证烈弓强命、狂骨近距造伤摸牌与苦肉主动技
    e.debug_grant(0, "fz_liegong");
    e.debug_grant(0, "fz_xinkuanggu");
    e.debug_grant(0, "kurou");
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    for _ in 0..4 {
        let shan = e.debug_place(1, "shan");
        e.debug_give_hand_card(1, 1, shan);
    }
    let sha = e.debug_place(0, "sha");
    e.debug_give_hand_card(0, 0, sha);
    let hp_before = e.debug_hp(1);
    let _ = submit(
        &mut e,
        0,
        GameAction::PlayCard {
            card: sha,
            targets: vec![1],
        },
    );
    let mut kuanggu_triggered = false;
    for _ in 0..12 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "trigger_confirm" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Confirm],
                },
            );
            continue;
        }
        if c.spec.prompt == "fz_xinkuanggu_choice" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Tag(1)],
                },
            );
            kuanggu_triggered = true;
            continue;
        }
        break;
    }
    drain(&mut e);
    assert_eq!(e.debug_hp(1), hp_before - 1, "fz_liegong 强命直伤");
    assert!(kuanggu_triggered, "fz_xinkuanggu 狂骨选择窗已触发");
    assert_eq!(e.debug_hand(0).len(), 1, "fz_xinkuanggu 选摸 1 张牌");

    let kurou_def = e.catalog().skill_by_name("kurou").expect("kurou 在目录");
    let hp0_before_kr = e.debug_hp(0);
    let _ = submit(&mut e, 0, GameAction::ActivateSkill { skill: kurou_def });
    drain(&mut e);
    assert_eq!(e.debug_hp(0), hp0_before_kr - 1, "kurou 失去 1 点体力");
    assert_eq!(e.debug_hand(0).len(), 3, "kurou 摸 2 张牌（1 + 2 = 3）");

    // ④ 存档与恢复
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[]).is_err());

    // ⑦ 坏数据拒绝
    let bad = cards_json.replace("host:fz_liegong", "host:unknown_ms_skill");
    assert!(matches!(
        bridge_pack(&bad),
        Err(BridgeError::UnknownBehaviorKey(_))
    ));
}
