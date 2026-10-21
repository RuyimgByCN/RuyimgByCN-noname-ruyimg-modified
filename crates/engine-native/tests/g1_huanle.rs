//! G1 规则宿主纵向样本（ADR 0776）：欢乐卡牌 → 真实包 → 导入 →
//! 宿主行为技能装载 → 嵌套选择与恢复。
//!
//! 行为子集（数据声明，宿主原语执行）：
//! - 苦无（武器）：杀伤害后目标弃一张手牌（强制触发；登记差异：
//!   上游为目标自选+he 区+notLink，原生为确定性弃首张手牌）
//! - 酒尾（宝物）：结束阶段可选——已受伤回血否则摸牌（可选触发，
//!   确认门由框架 OptionalGate 承担；2026-10-07 载荷对照补齐实体牌）
//! - 手里剑（基础牌 ×2）：目标弃 1 张装备或受 1 点伤害（嵌套选择）
//! - 猴子（宝物）：他人出桃使用前取消+夺桃（命令层挂起机器）
//! - 写轮眼（宝物）：回合开始窃取他人技能至回合结束（多步机器）
//! - 面具（防具）：翻面前置取消（锁定技，TurnOverBefore 前置窗口）

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::CardKind;
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceKind, ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase,
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

/// 桥接统一走 C0 装载器（ADR 0777/0778）：映射来自 cards.json 的
/// 数据键与 pack 段，测试只校验包元数据与测试期望一致。
fn bridge(pack_name: &'static str, json: &str) -> Result<ExtensionPack, String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    Ok(out.pack)
}

fn options() -> InstallOptions {
    InstallOptions {
        activate: true,
        allow_unsigned: true,
        verify_key: None,
        host: content_runtime::tooling_host_versions(),
    }
}

fn report_installed_meta(store: &PackageStore) -> content_runtime::InstalledMeta {
    let lock = store.current_lock().expect("lock");
    let huanle = lock
        .packages
        .iter()
        .find(|p| p.package_id == "huanle-cards")
        .expect("欢乐卡牌在内容锁");
    content_runtime::InstalledMeta {
        namespace: huanle.namespace.clone(),
        package_id: huanle.package_id.clone(),
        version: huanle.version.clone(),
        logic_hash: huanle.logic_hash.clone(),
        content_hash: huanle.content_hash.clone(),
        signed: false,
        key_id: None,
        extension_root: Some(
            repo_extensions_root()
                .join("huanle-upstream-noname")
                .display()
                .to_string(),
        ),
    }
}

fn advance_to_play(e: &mut Engine, player: u8) {
    for _ in 0..96 {
        let t = e.turn();
        if t.current == player && t.phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        // 门感知：途中可能挂起可选触发/弃牌门（v127 跳回合基建后循环
        // 变长）——按 min 应答首选项后继续
        if let Some(g) = e.open_choice() {
            let sel: Vec<ChoiceOption> = if g.spec.min > 0 {
                g.spec.options.iter().take(g.spec.min).copied().collect()
            } else {
                vec![g.spec.options[0]]
            };
            let (id, obs) = (g.id, g.observer);
            match submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: id,
                    selection: sel,
                },
            ) {
                CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
                other => panic!("answer: {other:?}"),
            }
            continue;
        }
        let actor = e.turn().current;
        match submit(e, actor, GameAction::NextPhase) {
            CommandOutcome::Accepted { .. } | CommandOutcome::Pending { .. } => {}
            other => panic!("advance: {other:?}"),
        }
    }
    panic!("no play phase");
}

/// 从 repo drop-in 根桥接欢乐包（子进程/父进程同源同锁）。
fn huanle_pack_from_repo() -> ExtensionPack {
    let root = repo_extensions_root();
    let dir = tempfile::tempdir().unwrap();
    let store = PackageStore::open(dir.path().join("store")).unwrap();
    let _ = import_extensions(&store, &root, &options()).expect("import");
    let meta = report_installed_meta(&store);
    let json = std::fs::read_to_string(
        PathBuf::from(meta.extension_root.as_ref().unwrap()).join("assets/cards.json"),
    )
    .unwrap();
    bridge("huanle-upstream-noname", &json).expect("桥接")
}

#[path = "g1_huanle/caoren.rs"]
mod caoren;
#[path = "g1_huanle/equip.rs"]
mod equip;
#[path = "g1_huanle/flip.rs"]
mod flip;
#[path = "g1_huanle/g1.rs"]
mod g1;
#[path = "g1_huanle/host.rs"]
mod host;
#[path = "g1_huanle/huanle.rs"]
mod huanle;
#[path = "g1_huanle/u06.rs"]
mod u06;
#[path = "g1_huanle/xly.rs"]
mod xly;
