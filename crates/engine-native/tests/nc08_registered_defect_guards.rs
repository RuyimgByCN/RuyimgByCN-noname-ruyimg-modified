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

// ------------------------------------------------------------ G14 阶段错位

/// Slice one dispatch arm: from `anchor` up to the next `HostBehavior::` arm.
fn arm_until_next<'a>(src: &'a str, anchor: &str) -> &'a str {
    let start = src
        .find(anchor)
        .unwrap_or_else(|| panic!("找不到 `{anchor}`（宿主分发表可能已重构，请更新本守卫）"));
    let body_start = start + anchor.len();
    let rest = &src[body_start..];
    match rest.find("HostBehavior::") {
        Some(end) => &src[start..body_start + end],
        None => &src[start..],
    }
}

/// G14 守卫：上游 `trigger.player == "phaseBegin"`（**任意阶段开始**），
/// 原生把 filter 钉死在**判定阶段**（`Phase::Judge, begin: true`）。
/// 修正后应照 `SwdJinlin2` 的先例写成 `Phase { player, begin: true, .. }`，
/// 即该 arm 里**不再出现 `Phase::Judge`**。
macro_rules! g14_guard {
    ($fn_name:ident, $variant:literal, $file:literal) => {
        #[test]
        #[ignore = "登记缺陷 G14：上游 player phaseBegin，原生钉死 Phase::Judge；\
                    修复需改未跟踪的 content/host/filtering*"]
        fn $fn_name() {
            let src = read($file);
            let seg = arm_until_next(&src, concat!("HostBehavior::", $variant, " =>"));
            assert!(
                !seg.contains("Phase::Judge"),
                concat!(
                    $variant,
                    " 上游为「任意阶段开始」(phaseBegin)，filter 不应钉死判定阶段；",
                    "当前 arm 仍含 `Phase::Judge`：\n{seg}",
                ),
                seg = seg,
            );
        }
    };
}

g14_guard!(g14_diyguhuo_any_phase, "YunchouDiyguhuo", "filtering/common.rs");
g14_guard!(g14_yougeng_any_phase, "GujianFoodYougeng", "filtering/common.rs");
g14_guard!(g14_xuelunyang_any_phase, "HuanleXuelunyangCard", "filtering/common.rs");
g14_guard!(g14_geju_any_phase, "WudianGeju", "filtering/common.rs");
g14_guard!(g14_hyunshen2_any_phase, "SwdHyunshen2", "filtering/swd.rs");
g14_guard!(g14_daixing2_any_phase, "SwdDaixing2", "filtering/swd.rs");
g14_guard!(g14_zhenjiu2_any_phase, "SwdZhenjiu2", "filtering/swd.rs");
g14_guard!(g14_huanxing_any_phase, "SwdHuanxing", "filtering/swd.rs");
g14_guard!(g14_qianfang_any_phase, "XianjianQianfang", "filtering/xianjian.rs");

// ------------------------------------------------- 凭空新增 / 自加分支（A 类）

/// 上游 `baiyi` 是**主动技**（`enable: 'phaseUse'`, `usable: 1`, `limited: true`），
/// **没有 trigger**。原生却给它挂了一个「第 5 轮判定阶段开始」的 boss 模式触发。
/// 修正后该 arm 不应再匹配 `EventData::Phase`。
#[test]
#[ignore = "登记缺陷（凭空新增触发）：baiyi 上游为 phaseUse 主动技、无 trigger；\
            修复需改未跟踪的 content/host/filtering/common.rs"]
fn baiyi_must_not_invent_a_phase_trigger() {
    let src = read("filtering/common.rs");
    let seg = arm_until_next(&src, "HostBehavior::BossBaiyi =>");
    assert!(
        !seg.contains("EventData::Phase"),
        "baiyi 上游无 trigger（phaseUse 主动技），不应凭空挂阶段触发；当前 arm：\n{seg}"
    );
}

/// 上游 `huajin2` 只有 `trigger: { player: "damageBegin" }`。
/// 原生在 `Damage` 之外**又 OR 了一条 `Phase::Judge, begin: true` 分支**（自加分支）。
/// 修正后该 arm 不应含阶段分支。
#[test]
#[ignore = "登记缺陷（自加分支）：huajin2 上游仅 damageBegin，原生 OR 了判定阶段分支；\
            修复需改未跟踪的 content/host/filtering/swd.rs"]
fn huajin2_must_not_add_a_phase_branch() {
    let src = read("filtering/swd.rs");
    let seg = arm_until_next(&src, "HostBehavior::SwdHuajin2 =>");
    assert!(
        !seg.contains("EventData::Phase"),
        "huajin2 上游只有 damageBegin，不应额外挂阶段分支；当前 arm：\n{seg}"
    );
}

/// 上游 `lexue` 是复合技，真实触发面在子技能 `lexue1`：`player: "phaseBegin"`
/// （**任意阶段开始**）。原生钉死 `Phase::Judge, begin: true` 且**未校验 `player == owner`**
/// ⇒ 会在**其他角色**的判定阶段也触发。修正后应写成 `Phase { player, begin: true, .. }`
/// 并带 `*player == owner`。
#[test]
#[ignore = "登记缺陷 G14：lexue1 上游 player phaseBegin（任意阶段、自己），\
            原生钉死 Judge 且缺 owner 校验；修复需改未跟踪的 content/host/filtering/swd.rs"]
fn lexue_any_phase_of_owner_only() {
    let src = read("filtering/swd.rs");
    let seg = arm_until_next(&src, "HostBehavior::SwdLexue =>");
    assert!(
        !seg.contains("Phase::Judge"),
        "lexue 上游为任意阶段开始（phaseBegin），不应钉死判定阶段；当前 arm：\n{seg}"
    );
    assert!(
        seg.contains("player == owner") || seg.contains("*player == owner"),
        "lexue 上游是 `player: phaseBegin`（只在自己阶段触发），过滤条件应校验 player == owner；\
         当前 arm：\n{seg}"
    );
}
