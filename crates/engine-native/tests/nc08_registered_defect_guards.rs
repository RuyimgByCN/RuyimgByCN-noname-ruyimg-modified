//! `#[ignore]` acceptance guards for confirmed upstream differences (2026-10-19).
//!
//! Each guard is written to **pass after the fix** and is `#[ignore]`d while the
//! defect is live, following the convention established in
//! `tests/nc08_upstream_rangjie.rs` (see also `docs/operations/evidence/
//! 2026-10-19-registered-defects/README.md`).
//!
//! These are **source-fact** guards: they read the host dispatch source and
//! assert the corrected shape, so they need no scenario fixture and cannot rot
//! into a "green because never executed" test. When the owning agent fixes a
//! defect it removes the matching `#[ignore]` and the guard becomes the
//! acceptance check.
//!
//! Defects covered here (all are A-class: no enum extension needed):
//! - **G19** `swd_zhuhai` / `swd_tiebi` / `swd_rexue`: upstream triggers are
//!   `global`, but the native filters exclude the skill owner (`t != owner`),
//!   so the effects can never involve the owner itself.
//! - **5b** `reguhuo`: upstream keeps a per-turn `reguhuo_phase` marker; the
//!   native step has no marker, so the skill can be used unlimited times per
//!   turn.

use std::path::{Path, PathBuf};

fn host_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src/content/host")
}

fn read(rel: &str) -> String {
    let path = host_dir().join(rel);
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("读 {} 失败: {e}", path.display()))
}

/// Slice the arm body that starts at `anchor` for `len` bytes.
fn arm<'a>(src: &'a str, anchor: &str, len: usize) -> &'a str {
    let start = src
        .find(anchor)
        .unwrap_or_else(|| panic!("找不到 `{anchor}`（宿主分发表可能已重构，请更新本守卫）"));
    let rest = &src[start..];
    &rest[..rest.len().min(len)]
}

// ---------------------------------------------------------------- G19 竹海

/// 上游 `swd_zhuhai` 的 `trigger: { global: 'phaseEnd' }` 覆盖**全场**。
/// 原生过滤条件里带了 `t != owner`，把技能主排除在外 ⇒ 修正后该条件应消失。
#[test]
#[ignore = "登记缺陷 G19：swd_zhuhai 上游 global phaseEnd，原生过滤含 t != owner；\
            修复需改未跟踪的 content/host/filtering/swd.rs"]
fn g19_zhuhai_filter_must_not_exclude_owner() {
    let src = read("filtering/swd.rs");
    let seg = arm(&src, "HostBehavior::SwdZhuhai => match event", 1200);
    assert!(
        !seg.contains("t != owner"),
        "swd_zhuhai 应覆盖全场（上游 global），过滤条件不应排除技能主；\
         当前仍含 `t != owner`：\n{seg}"
    );
}

// ---------------------------------------------------------------- G19 铁壁

/// 上游 `swd_tiebi` 的 `trigger: { global: 'useCardToBegin' }` 覆盖**全场**。
#[test]
#[ignore = "登记缺陷 G19：swd_tiebi 上游 global，原生目标筛选含 t != owner；\
            修复需改未跟踪的 content/host/filtering/swd.rs"]
fn g19_tiebi_filter_must_not_exclude_owner() {
    let src = read("filtering/swd.rs");
    let seg = arm(&src, "HostBehavior::SwdTiebi => {", 1200);
    assert!(
        !seg.contains("t != owner"),
        "swd_tiebi 应覆盖全场（上游 global），目标筛选不应排除技能主；\
         当前仍含 `t != owner`：\n{seg}"
    );
}

// ---------------------------------------------------------------- G19 热血

/// 上游 `swd_rexue` 的 `trigger: { global: 'phaseEnd' }` 覆盖**全场**。
#[test]
#[ignore = "登记缺陷 G19：swd_rexue 上游 global，原生过滤含 actor != owner；\
            修复需改未跟踪的 content/host/filtering/swd.rs"]
fn g19_rexue_filter_must_not_exclude_owner() {
    let src = read("filtering/swd.rs");
    let seg = arm(&src, "HostBehavior::SwdRexue => {", 900);
    assert!(
        !seg.contains("actor != owner"),
        "swd_rexue 应覆盖全场（上游 global），过滤条件不应排除技能主；\
         当前仍含 `actor != owner`：\n{seg}"
    );
}

// ------------------------------------------------------------- 5b 蛊惑次数

/// 上游 `reguhuo` 用 `reguhuo_phase` 标记做到**每回合一次**；
/// 原生 `step` 收尾既不设标记也不计数 ⇒ 可无限次使用。
/// 修正后 `reguhuo` 的效果步应出现回合标记（`reguhuo_phase` 常量或等价计数）。
#[test]
#[ignore = "登记缺陷 5b：reguhuo 每回合一次未实现（EPIC_REGUHUO_KEY 定义未接线）；\
            修复需改未跟踪的 content/host/step/skills_cards.rs 与 constants.rs"]
fn reguhuo_must_record_a_per_turn_use_marker() {
    let src = read("step/skills_cards.rs");
    let seg = arm(&src, "HostBehavior::Reguhuo => match self.stage", 4000);
    assert!(
        seg.contains("reguhuo_phase") || seg.contains("EPIC_REGUHUO_KEY"),
        "reguhuo 应记录「本回合已用」标记以限制每回合一次；当前效果步无任何标记痕迹：\n{seg}"
    );
}
