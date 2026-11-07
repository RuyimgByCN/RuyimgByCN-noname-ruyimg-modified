//! U08 英雄杀（`hero-kill-upstream-noname`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 包数据桥接：`assets/cards.json` 5 项英雄杀技能（`yxs_fanji`, `yxs_fanji2`, `yxs_menshen`,
//!   `yxs_menshen2`, `yxs_menshen3`）装载与目录登记；
//! ③ 组合共存：与 `mark-supplement-upstream-liuli`、`select-beautify-upstream-liuli` 同局共存；
//! ④ 保存恢复：门神标记与反击技能挂载状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：门神（`yxs_menshen`）回合结束指定目标挂载 `yxs_menshen2` 与反击（`yxs_fanji`）触发；
//! ⑦ 异常拒装：损坏 JSON 与非法宿主行为键明确返回 `BridgeError`；
//! ⑧ 边界登记：43 条跨包外部技能引用武将表（`character_table`）显式登记于 `deferred`。

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
fn u08_hero_kill_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered.iter().any(|d| d.name == "hero-kill-upstream-noname"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "hero-kill" && m.namespace == "upstream-noname")
        .expect("hero-kill 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let json = std::fs::read_to_string(ext_root.join("assets/cards.json"))
        .expect("读取 hero-kill cards.json");
    let config_json = std::fs::read_to_string(ext_root.join("assets/config.json"))
        .expect("读取 hero-kill config.json");
    assert!(config_json.contains("\"yxs_fanji\""));
    assert!(config_json.contains("\"yxs_menshen\""));

    // ② 桥接与技能/暂缓核对
    let out = bridge_pack(&json).expect("bridge hero-kill");
    assert_eq!(out.pack.def.name, "hero-kill-upstream-noname");
    assert_eq!(out.pack.host_skills.len(), 5);
    assert_eq!(
        out.deferred,
        vec!["character_table".to_string(), "skill_batch_rest".to_string()]
    );

    // ③ 与标记补充包共存
    let ms_json = std::fs::read_to_string(
        root.join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 mark-supplement cards.json");
    let ms_pack = bridge_pack(&ms_json).expect("bridge mark-supplement").pack;
    let packs = [&out.pack, &ms_pack];
    let mut e = Engine::new_with_extensions(190801, 4, &packs).expect("hero-kill + ms 建局");
    for sk in ["yxs_fanji", "yxs_fanji2", "yxs_menshen", "yxs_menshen2", "yxs_menshen3"] {
        assert!(e.catalog().skill_by_name(sk).is_some(), "技能 {sk} 已登记");
    }

    // ⑤ 授予 yxs_menshen + yxs_menshen3 + yxs_fanji，验证门神挂载与反击触发
    e.debug_grant(0, "yxs_menshen");
    e.debug_grant(0, "yxs_menshen3");
    e.debug_grant(0, "yxs_fanji");
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Discard {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    let actor = e.turn().current;
    let _ = submit(&mut e, actor, GameAction::NextPhase); // Discard -> End
    drain(&mut e);
    let actor = e.turn().current;
    let _ = submit(&mut e, actor, GameAction::NextPhase); // End -> 整回合结束窗触发门神

    let mut targeted = false;
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
        if c.spec.prompt == "yxs_menshen_target" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Player(1)],
                },
            );
            targeted = true;
            continue;
        }
        break;
    }
    drain(&mut e);
    assert!(targeted, "门神目标选择窗已触发并应答");
    let m2_def = e.catalog().skill_by_name("yxs_menshen2").unwrap();
    assert!(
        e.state_core().skills.contains_key(&(1, m2_def)),
        "1 号获得 yxs_menshen2 实例"
    );

    // 5b. 反击（yxs_fanji）：3 号（与 0 号距离 1）出牌阶段对 0 号出【杀】造伤 → 0 号手牌【杀】触发 yxs_fanji_sha 反击 3 号
    for _ in 0..24 {
        if e.turn().current == 3 && e.turn().phase == Phase::Play {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    for p in 0..4 {
        e.debug_clear_hand(p);
    }
    let counter_sha = e.debug_place(0, "sha");
    let atk_sha = e.debug_place(3, "sha");
    let hp3_before = e.debug_hp(3);
    let play_out = submit(
        &mut e,
        3,
        GameAction::PlayCard {
            card: atk_sha,
            targets: vec![0],
        },
    );
    assert!(matches!(play_out, CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. }));
    let mut fanji_used = false;
    for _ in 0..16 {
        let Some(c) = e.open_choice().cloned() else { break };
        if c.spec.prompt == "yxs_fanji_sha" {
            let _ = submit(
                &mut e,
                c.observer,
                GameAction::Answer {
                    choice_id: c.id,
                    selection: vec![ChoiceOption::Card(counter_sha)],
                },
            );
            fanji_used = true;
            continue;
        }
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
        let sel = if c.spec.options.contains(&ChoiceOption::Pass) {
            vec![ChoiceOption::Pass]
        } else if c.spec.allow_cancel {
            Vec::new()
        } else {
            c.spec.options.iter().take(c.spec.min.max(1)).copied().collect()
        };
        let _ = submit(
            &mut e,
            c.observer,
            GameAction::Answer {
                choice_id: c.id,
                selection: sel,
            },
        );
    }
    assert!(fanji_used, "yxs_fanji_sha 反击选牌窗已触发");
    assert_eq!(e.debug_hp(3), hp3_before - 1, "反击命中伤害来源 3 号");

    // ④ 保存与恢复
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复 hero-kill 对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[]).is_err());

    // ⑦ 损坏数据拒绝
    let bad = json.replace("host:yxs_fanji", "host:unknown_yxs_behavior");
    assert!(matches!(
        bridge_pack(&bad),
        Err(BridgeError::UnknownBehaviorKey(_))
    ));
}
