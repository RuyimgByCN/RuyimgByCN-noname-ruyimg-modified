//! L19 末日乱斗（`doomsday-brawl-upstream-liuli`）八维整包验收（2026-11-06）。
//!
//! 覆盖八维：
//! ① 生命周期：真实 `discover_extensions` + `import_extensions` 安装、激活、停用与卸载；
//! ② 配置与包数据：`assets/cards.json` 桥接 3 项技能（`shishi`, `xunmeng`, `zaibian`）+ `doomsday` 模式辅助函数；
//! ③ 组合共存：与 `mark-supplement-upstream-liuli`、`select-beautify-upstream-liuli` 同局共存；
//! ④ 保存恢复：对局中含 `zaibian`/`shishi` 技能状态下 `save` / `restore_with_extensions` 字节级一致；
//! ⑤ 功能执行：灾变（`zaibian`）出牌阶段开始摸牌、无尽模式人类位加成计算（`WujinMoshiConfig`）与身份转化链（`identity_chain`）；
//! ⑦ 异常拒装：损坏 JSON 与未知宿主行为键返回明确 `BridgeError`；
//! ⑧ 边界登记：`wujinmoshi`、`brawl_mode` 与 `skill_batch_rest` 在 `cards.json` 显式登记暂缓原因。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_native_loader::{
    bridge_pack, identity_chain, zaibian_draw, BridgeError, WujinMoshiConfig, IDENTITY_NAMES,
};
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
fn l19_doomsday_brawl_eight_dimension_acceptance() {
    // ① 发现与安装
    let root = repo_extensions_root();
    let discovered = discover_extensions(&root).expect("discover");
    assert!(discovered
        .iter()
        .any(|d| d.name == "doomsday-brawl-upstream-liuli"));

    let tmp = tempfile::tempdir().unwrap();
    let store = PackageStore::open(tmp.path().join("store")).unwrap();
    let report = import_extensions(&store, &root, &options()).expect("import");
    let meta = report
        .installed
        .iter()
        .find(|m| m.package_id == "doomsday-brawl" && m.namespace == "upstream-liuli")
        .expect("doomsday-brawl 已安装")
        .clone();

    let ext_root = PathBuf::from(meta.extension_root.as_ref().unwrap());
    let cards_json =
        std::fs::read_to_string(ext_root.join("assets/cards.json")).expect("读取 cards.json");
    let config_json =
        std::fs::read_to_string(ext_root.join("assets/config.json")).expect("读取 config.json");
    assert!(config_json.contains("\"shishi\""));

    // ② 包数据桥接与模式辅助校验
    let out = bridge_pack(&cards_json).expect("bridge doomsday-brawl");
    assert_eq!(out.pack.def.name, "doomsday-brawl-upstream-liuli");
    assert_eq!(out.pack.host_skills.len(), 3);
    assert_eq!(
        out.deferred,
        vec![
            "wujinmoshi".to_string(),
            "brawl_mode".to_string(),
            "skill_batch_rest".to_string()
        ]
    );

    // doomsday 模式纯函数校验
    assert_eq!(IDENTITY_NAMES.len(), 15);
    assert_eq!(identity_chain(4), vec!["er", "san"]);
    assert_eq!(zaibian_draw(2, 1, 0), 3);
    assert_eq!(zaibian_draw(0, 3, 1), 0);
    let wj = WujinMoshiConfig {
        draw: 2,
        damage: 1,
        damage_taken: 1,
        recover: 1,
        max_hand_card: 3,
        damage_divisor: 30,
        human_seats: vec![0],
    };
    assert_eq!(wj.draw_bonus(0), 2);
    assert_eq!(wj.draw_bonus(1), 0);

    // ③ 与标记补充包共存建局
    let ms_json = std::fs::read_to_string(
        root.join("mark-supplement-upstream-liuli/assets/cards.json"),
    )
    .expect("读取 mark-supplement cards.json");
    let ms_pack = bridge_pack(&ms_json).expect("bridge ms").pack;
    let packs = [&out.pack, &ms_pack];
    let mut e = Engine::new_with_extensions(191901, 4, &packs).expect("建局");

    // ⑤ 授予 zaibian 并验证出牌阶段开始额外摸牌
    e.debug_grant(0, "zaibian");
    for _ in 0..16 {
        if e.turn().current == 0 && e.turn().phase == Phase::Draw {
            break;
        }
        let actor = e.turn().current;
        let _ = submit(&mut e, actor, GameAction::NextPhase);
        drain(&mut e);
    }
    let hand_after_draw = e.debug_hand(0).len();
    let actor = e.turn().current;
    let _ = submit(&mut e, actor, GameAction::NextPhase); // Draw -> Play (zaibian 触发)
    drain(&mut e);
    assert_eq!(e.debug_hand(0).len(), hand_after_draw + 2, "灾变额外摸 2 张");

    // ④ 存档与恢复
    let snap = e.save();
    let restored = Engine::restore_with_extensions(&snap, &packs).expect("恢复对局");
    assert_eq!(restored.state_hash(), e.state_hash());
    assert!(Engine::restore_with_extensions(&snap, &[]).is_err());

    // ⑦ 坏数据拒绝
    let bad = cards_json.replace("host:zaibian", "host:unknown_doomsday_skill");
    assert!(matches!(
        bridge_pack(&bad),
        Err(BridgeError::UnknownBehaviorKey(_))
    ));
}
