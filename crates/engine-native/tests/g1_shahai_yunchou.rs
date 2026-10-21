//! G1 杀海拾遗·运筹帷幄子包切片（ADR 0776）：既有牌种直转
//! （火杀 + 过河拆桥×3）→ 真实包 → 导入 → 建局/规则结算/内容锁。
//!
//! 切片 4/22 张（其余 17 张按 reason 登记于包数据 deferred）。
//! 上游核对：`杀海拾遗/card/yunchou/{list,card}.js @ 5c02a745`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::event::DamageNature;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, Phase, RejectReason,
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

/// 桥接统一走 C0 装载器（ADR 0777/0778）：映射来自 cards.json 的
/// 数据键与 pack 段，测试只校验包元数据一致。
fn bridge(pack_name: &'static str, json: &str) -> Result<(ExtensionPack, Vec<String>), String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    assert_eq!(out.pack.def.name, pack_name, "包名来自 pack 段");
    Ok((out.pack, out.deferred))
}

fn assert_zone_conservation_yunchou(e: &Engine) {
    let snap = e.save();
    let mut ids: Vec<u32> = Vec::new();
    for p in &snap.core.players {
        ids.extend(p.hand.iter().copied());
        ids.extend(p.equips.iter().copied());
        ids.extend(p.judge.iter().copied());
    }
    ids.extend(snap.core.deck.iter().copied());
    ids.extend(snap.core.discard.iter().copied());
    ids.extend(snap.core.table.iter().copied());
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "实体牌重复归属");
}

fn hand_len(e: &Engine, p: u8) -> usize {
    e.state_core().player(p).hand.len()
}

/// 推进到指定玩家的指定阶段（空/首选应答排空）。
fn advance_to(e: &mut Engine, player: u8, phase: Phase) {
    for _ in 0..200 {
        if e.turn().current == player && e.turn().phase == phase {
            return;
        }
        assert!(e.outcome().is_none(), "对局提前结束");
        let actor = e.turn().current;
        submit(e, actor, GameAction::NextPhase);
        if let Some(c) = e.open_choice() {
            let sel: Vec<ChoiceOption> = c
                .spec
                .options
                .iter()
                .filter(|o| matches!(o, ChoiceOption::Card(_)))
                .copied()
                .take(c.spec.min.max(1))
                .collect();
            let (cid, obs) = (c.id, c.observer);
            if !sel.is_empty() {
                submit(
                    e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: sel,
                    },
                );
            } else if !c.spec.options.is_empty() {
                submit(
                    e,
                    obs,
                    GameAction::Answer {
                        choice_id: cid,
                        selection: vec![c.spec.options[0].clone()],
                    },
                );
            }
        }
    }
    panic!("200 步未到 p{player} 目标阶段");
}

/// 可选触发确认门排空（Confirm/Pass 取首选项；无门则空转）。
fn settle_optional_confirm(e: &mut Engine) {
    for _ in 0..8 {
        let Some(c) = e.open_choice() else { return };
        let (cid, obs) = (c.id, c.observer);
        let sel = if c.spec.options.is_empty() {
            return;
        } else {
            vec![c.spec.options[0].clone()]
        };
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: sel,
            },
        );
    }
}

#[path = "g1_shahai_yunchou/caochuan.rs"]
mod caochuan;
#[path = "g1_shahai_yunchou/chenhuodajie.rs"]
mod chenhuodajie;
#[path = "g1_shahai_yunchou/fudichouxin.rs"]
mod fudichouxin;
#[path = "g1_shahai_yunchou/g1.rs"]
mod g1;
#[path = "g1_shahai_yunchou/geanguanhuo.rs"]
mod geanguanhuo;
#[path = "g1_shahai_yunchou/hongshui.rs"]
mod hongshui;
#[path = "g1_shahai_yunchou/shezhanqunru.rs"]
mod shezhanqunru;
#[path = "g1_shahai_yunchou/shuigong.rs"]
mod shuigong;
#[path = "g1_shahai_yunchou/suolianjia.rs"]
mod suolianjia;
#[path = "g1_shahai_yunchou/touliang.rs"]
mod touliang;
#[path = "g1_shahai_yunchou/yunchou.rs"]
mod yunchou;
