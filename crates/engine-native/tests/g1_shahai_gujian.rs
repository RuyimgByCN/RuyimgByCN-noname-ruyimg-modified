//! G1 杀海拾遗·古剑子包切片（ADR 0776）：舒筋散（目标弃任意张手牌
//! 摸等量）/面里金针（目标摸 1 后失去 1 点体力，规则 v77）/
//! 飞镖（目标弃一张黑色手牌或失去 1 点体力；登记差异
//! 同前）→ 真实包 → 导入 → 建局/规则结算/内容锁。
//!
//! 切片 40/40 张全卡转换（deferred 清零）——食物簇 12/12、机关簇
//! 6/8、余簇全部收尾；帝女玄霜/玉衡按登记差异简化转换。
//! 上游核对：`杀海拾遗/card/gujian/{list,card}.js @ 5c02a745`。

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use content_runtime::{discover_extensions, import_extensions, InstallOptions, PackageStore};
use engine_native::catalog::{CardKind, Suit};
use engine_native::content::extension::ExtensionPack;
use engine_native::{
    ChoiceOption, CommandOutcome, Engine, GameAction, GameCommand, LogEvent, Phase, RejectReason,
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

/// 装载（ADR 0777/0778）：包元数据写在 cards.json，食物/装备伴随技能由
/// 引擎牌种表登记，桥接统一在 content-native-loader，测试不含逐包映射。
fn bridge(json: &str) -> Result<(ExtensionPack, Vec<String>), String> {
    let out = content_native_loader::bridge_pack(json).map_err(|e| e.to_string())?;
    Ok((out.pack, out.deferred))
}

/// 前进到 0 号玩家出牌阶段（消化开局摸牌与挂起选择）。
fn advance_to_play(e: &mut Engine) {
    for _ in 0..64 {
        if e.turn().current == 0 && e.turn().phase == Phase::Play && e.open_choice().is_none() {
            return;
        }
        if let Some(c) = e.open_choice().cloned() {
            let cid = c.id;
            let sel = if c.spec.min > 0 {
                c.spec.options.iter().take(c.spec.min).copied().collect()
            } else {
                vec![c.spec.options[0]]
            };
            let obs = c.observer;
            match submit(
                e,
                obs,
                GameAction::Answer {
                    choice_id: cid,
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
    panic!("64 步内未到达 0 号出牌阶段");
}

/// 全区清除若干实体牌（hand/equips/judge/deck/discard/table）——
/// 注入前置：归属总账按目录全集断言，缺一区都会被恢复校验拒绝。
fn purge_all_zones(snap: &mut engine_native::StateSnapshot, ids: &[u32]) {
    for c in ids {
        snap.core.deck.retain(|x| x != c);
        snap.core.discard.retain(|x| x != c);
        for p in snap.core.players.iter_mut() {
            p.hand.retain(|x| x != c);
            p.equips.retain(|x| x != c);
            p.judge.retain(|x| x != c);
        }
        snap.core.table.retain(|x| x != c);
    }
}

/// 无懈窗口：无人持无懈时自动跳过（不挂起）；出现则直过。
fn pass_nullify_window(e: &mut Engine) {
    let is_window = e
        .open_choice()
        .map(|c| c.spec.prompt == "nullify_window")
        .unwrap_or(false);
    if is_window {
        let c = e.open_choice().unwrap();
        let cid = c.id;
        let obs = c.observer;
        submit(
            e,
            obs,
            GameAction::Answer {
                choice_id: cid,
                selection: vec![ChoiceOption::Pass],
            },
        );
    }
}

/// 清理剩余询问直至静止。空选择合法的询问空应答（=取消/不发动）；
/// min>0 的确认类询问（如金莲烛确认）空应答非法，会原样重开——改选
/// 中性 Pass（放弃响应），无 Pass 时取首个合法选项。步数上限防挂。
fn drain_choices(e: &mut Engine) {
    for _ in 0..64 {
        let Some(c) = e.open_choice() else { return };
        let cid = c.id;
        let obs = c.observer;
        let sel = if c.spec.min > 0 {
            let pass = c
                .spec
                .options
                .iter()
                .find(|o| **o == ChoiceOption::Pass)
                .copied();
            match pass {
                Some(p) => vec![p],
                None => vec![c.spec.options[0]],
            }
        } else {
            vec![]
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
    panic!("64 步内询问未消化");
}

fn assert_zone_conservation(e: &Engine) {
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

fn parsed_deferred_count(json: &str) -> usize {
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    parsed["deferred"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| d["count"].as_u64().unwrap_or(0) as usize)
        .sum()
}

fn parsed_deferred_len(json: &str) -> usize {
    let parsed: serde_json::Value = serde_json::from_str(json).unwrap();
    parsed["deferred"].as_array().unwrap().len()
}

#[path = "g1_shahai_gujian/dinvxuanshuang.rs"]
mod dinvxuanshuang;
#[path = "g1_shahai_gujian/feibiao.rs"]
mod feibiao;
#[path = "g1_shahai_gujian/food.rs"]
mod food;
#[path = "g1_shahai_gujian/g1.rs"]
mod g1;
#[path = "g1_shahai_gujian/gujian.rs"]
mod gujian;
#[path = "g1_shahai_gujian/hujia.rs"]
mod hujia;
#[path = "g1_shahai_gujian/jiguan.rs"]
mod jiguan;
#[path = "g1_shahai_gujian/jinlianzhu.rs"]
mod jinlianzhu;
#[path = "g1_shahai_gujian/lifecycle.rs"]
mod lifecycle;
#[path = "g1_shahai_gujian/mizhilianou.rs"]
mod mizhilianou;
#[path = "g1_shahai_gujian/remaining.rs"]
mod remaining;
#[path = "g1_shahai_gujian/shujinsan.rs"]
mod shujinsan;
