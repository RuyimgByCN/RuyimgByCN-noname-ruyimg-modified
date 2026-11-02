//! 改命取消：候选全部归还牌堆后必须走正常判定初始化（规则 332，裁决 24）。
//!
//! 修复前红（2026-10-30 复现、2026-11-03 在 331 主树再证）：取消把 7 张候选
//! 回堆，但 `current.judge_card` 仍指向首候选并由机器持有——同一实体既在
//! 牌堆又被当作判定牌结算弃置，物理多重集 124→125、牌 65 重复，恢复拒绝
//! `IdOutOfBounds { what: "card_in_multiple_zones", got: 65 }`。
//! 修复：取消分支归还候选后直接复用正常判定初始化（抽判定牌／Judged／
//! 昊天塔／改判队列／天妒资格），不再重复询问改命。

#[path = "support/nc09_machine_windows.rs"]
mod windows;

use engine_native::LogEvent;
use windows::{answer, fixture, Window};

fn sorted_visible(engine: &engine_native::Engine) -> Vec<u32> {
    let mut ids = engine.view_referee().visible_card_ids();
    ids.sort_unstable();
    ids
}

/// 取消必须保持「每个物理实体恰好一处」，且取消后的状态可恢复。
#[test]
fn cancel_preserves_physical_multiset_and_restores() {
    let mut f = fixture(Window::GaiMing);
    let pending = f.engine.save();
    let restored = f.restore(&pending).expect("挂起态可恢复");
    assert_eq!(restored.state_hash(), f.engine.state_hash());
    let before = sorted_visible(&f.engine);
    assert_eq!(before, f.physical_ids, "挂起态物理多重集");

    answer(&mut f.engine, vec![]); // 合法空选择＝取消

    let after = sorted_visible(&f.engine);
    let duplicates: Vec<_> = after
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .map(|pair| pair[0])
        .collect();
    assert!(duplicates.is_empty(), "取消后出现重复实体：{duplicates:?}");
    assert_eq!(after.len(), before.len(), "取消不得增减物理牌数量");
    let snapshot = f.engine.save();
    f.restore(&snapshot).expect("取消后状态必须可恢复");
}

/// 取消后必须真正进入正常判定：Judged 日志取走恰好一张堆顶候选、
/// 不再重复询问改命；选牌分支（正控制）保持原规则。
#[test]
fn cancel_then_normal_judgement_and_pick_control() {
    // 取消分支
    let mut f = fixture(Window::GaiMing);
    let judged_before = f
        .engine
        .log()
        .iter()
        .filter(|x| matches!(x, LogEvent::Judged { .. }))
        .count();
    // 挂起时 7 张候选已被亮出离堆；held[0] 即揭示前的堆顶（原首候选）。
    let first_candidate = f.held[0];
    answer(&mut f.engine, vec![]);
    assert_ne!(
        f.engine.open_choice().map(|c| c.spec.prompt.clone()),
        Some("gaiming_pick"),
        "取消后不得重复询问改命"
    );
    let judged: Vec<_> = f.engine.log()[judged_before..]
        .iter()
        .filter_map(|x| match x {
            LogEvent::Judged { card, .. } => Some(*card),
            _ => None,
        })
        .collect();
    assert_eq!(judged.len(), 1, "取消后恰有一次正常判定：{judged:?}");
    assert_eq!(
        judged[0], first_candidate,
        "正常判定抽走归还后的堆顶（原首候选）"
    );
    // 选牌分支正控制：选中最后一张候选成为判定结果。
    let mut f2 = fixture(Window::GaiMing);
    let held = f2.held.clone();
    answer(&mut f2.engine, vec![engine_native::ChoiceOption::Card(held[6])]);
    let judged2: Vec<_> = f2
        .engine
        .log()
        .iter()
        .filter_map(|x| match x {
            LogEvent::Judged { card, .. } => Some(*card),
            _ => None,
        })
        .collect();
    assert!(judged2.contains(&held[6]), "选中候选成为判定结果");
}

/// 旧规则档（331 真实挂起夹具）在新版本下按版本门禁拒绝，原件不动。
#[test]
fn genuine_331_pending_archive_is_version_rejected() {
    let bytes = include_bytes!("fixtures/gaiming-window-331.ngsa");
    let snap = engine_native::archive::decode_archive(bytes).expect("106 可解码");
    let f = fixture(Window::GaiMing);
    let packs: Vec<_> = f.packs.iter().collect();
    assert!(matches!(
        engine_native::Engine::restore_with_extensions(&snap, &packs),
        Err(engine_native::snap::SnapshotError::RuleVersionMismatch { .. })
    ));
}
