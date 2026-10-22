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

// ------------------------------------------------ G20 `phaseEnd` 偏早一步

/// G20 守卫：上游 `trigger.player == "phaseEnd"`（**全部阶段跑完之后**，见
/// `game.js:15999-16003`），原生写成 **`Phase::End, begin: true`（结束阶段开始）**
/// ⇒ 早一步。修正后该 arm 应匹配 **`End, begin: false`**。
macro_rules! g20_guard {
    ($fn_name:ident, $variant:literal,) => {
        #[test]
        #[ignore = "登记缺陷 G20：上游 phaseEnd（全部阶段后），原生 End,begin:true（结束阶段开始）早一步；\
                    修复需改未跟踪的 content/host/filtering/common.rs"]
        fn $fn_name() {
            let src = read("filtering/common.rs");
            let seg = arm_until_next(&src, concat!("HostBehavior::", $variant, " =>"));
            assert!(
                seg.contains("Phase::End, begin: false"),
                concat!(
                    $variant,
                    " 上游为 phaseEnd（全部阶段跑完之后），应匹配 End, begin: false；",
                    "当前 arm：\n{seg}",
                ),
                seg = seg,
            );
        }
    };
}

g20_guard!(g20_jiuwei_turn_end, "PhaseEndRecoverOrDraw",);
g20_guard!(g20_liyutang_turn_end, "GujianFoodLiyutang",);
g20_guard!(g20_zhucheng_turn_end, "WudianZhucheng",);
g20_guard!(g20_paotai2_turn_end, "WudianPaotaiFire",);
g20_guard!(g20_ziyu_turn_end, "WudianZiyu",);

/// 上游 `ziyangdan` 是 **`trigger: { player: "phaseBegin" }`（任意阶段开始）**，
/// 原生却钉死 `Phase::End` ⇒ 归入「钉死单一阶段」家族（G14 的钉 `End` 变体）。
/// 修正后应写成 `Phase { player, begin: true, .. }`，即该 arm 不再出现 `Phase::End`。
#[test]
#[ignore = "登记缺陷（钉死错向）：ziyangdan 上游 phaseBegin（任意阶段），原生钉死 End；\
            修复需改未跟踪的 content/host/filtering/common.rs"]
fn ziyangdan_must_not_pin_the_end_phase() {
    let src = read("filtering/common.rs");
    let seg = arm_until_next(&src, "HostBehavior::GujianFoodZiyangdan =>");
    assert!(
        !seg.contains("Phase::End"),
        "ziyangdan 上游为任意阶段开始（phaseBegin），不应钉死结束阶段；当前 arm：\n{seg}"
    );
}

// ------------------------------------- G23 `PhaseEnd` 未钉阶段 ⇒ 每阶段触发

/// G23 守卫：上游 `player: "phaseEnd"` 是**每回合一次**（`game.js:15999-16003`），
/// 而原生 meta 用 `EventKind::PhaseEnd`（**每个阶段结束都派发**）。
/// 修正后该 arm 必须**钉住结束阶段**（`phase: crate::event::Phase::End`），
/// 否则一回合会触发 5 次。
macro_rules! g23_guard {
    ($fn_name:ident, $variant:literal,) => {
        #[test]
        #[ignore = "登记缺陷 G23：上游 phaseEnd 每回合一次，原生未钉阶段 ⇒ 每阶段结束都触发（5×）；\
                    修复需改未跟踪的 content/host/filtering/{common,swd}.rs"]
        fn $fn_name() {
            let src = {
                let a = read("filtering/common.rs");
                if a.contains(concat!("HostBehavior::", $variant, " =>")) {
                    a
                } else {
                    read("filtering/swd.rs")
                }
            };
            let seg = arm_until_next(&src, concat!("HostBehavior::", $variant, " =>"));
            assert!(
                seg.contains("phase: crate::event::Phase::End")
                    || seg.contains("*phase == crate::event::Phase::End"),
                concat!(
                    $variant,
                    " 上游 phaseEnd 每回合一次，filter 应钉住结束阶段；当前未钉 ⇒ 每阶段结束都触发：\n{seg}",
                ),
                seg = seg,
            );
        }
    };
}

g23_guard!(g23_huanling_turn_end, "SwdHuanling",);
g23_guard!(g23_lingxin_turn_end, "SwdLingxin",);
g23_guard!(g23_lqingcheng_turn_end, "SwdLqingcheng",);
g23_guard!(g23_donghuangzhong_turn_end, "SwdDonghuangzhongCard",);
g23_guard!(g23_guangshu_spade_turn_end, "WudianGuangshuSpade",);
g23_guard!(g23_jidong_turn_end, "WudianJidong",);
g23_guard!(g23_binxin_turn_end, "XianjianBinxin",);
g23_guard!(g23_lingyu_turn_end, "XianjianLingyu",);
g23_guard!(g23_xuanyuan_turn_end, "Xuanyuan",);
g23_guard!(g23_menshen_turn_end, "YxsMenshen",);

/// `SwdJiguanyuanGain` 的问题不是「未钉阶段」，而是**事件种类完全错**：
/// 上游是 `player: "damageEnd"` + `source: "damageSource"`（伤害事件），
/// 原生却把它挂在 `EventKind::PhaseEnd` 上 ⇒ 永不按伤害触发。
/// 修正后 meta 应为伤害事件。
#[test]
#[ignore = "登记缺陷 G23b：SwdJiguanyuanGain 上游 damageEnd+damageSource，原生挂 PhaseEnd；\
            修复需改未跟踪的 content/host/metadata/swd.rs 与 filtering/swd.rs"]
fn jiguanyuan_gain_must_trigger_on_damage() {
    let meta = read("metadata/swd.rs");
    let seg = arm_until_next(&meta, "HostBehavior::SwdJiguanyuanGain =>");
    assert!(
        seg.contains("EventKind::Damage"),
        "SwdJiguanyuanGain 上游是伤害事件（damageEnd + damageSource），meta 不应用 PhaseEnd：\n{seg}"
    );
}

// ------------------------------------------- G14 补充：相等比较写法与卡片 global 技能


// ------------------------------------- 能力守卫：缺失的事件/阶段槽位是否已补

/// 3 条 `需档` 缺陷都卡在「引擎缺某个事件槽位」（第 795 轮复核：`useCardTo*` 11 + `sha*` 6 + `dieBegin` 4）。
/// 这类缺陷的验收条件不是某个行为，
/// 而是**枚举里出现该槽位** —— 所以守卫直接断言 `event.rs` / `phase` 枚举的内容。
/// 每个守卫的文档注明它解锁哪些缺陷族。
macro_rules! capability_guard {
    ($fn_name:ident, $needle:literal, $why:literal) => {
        #[test]
        #[ignore = "缺失能力：补上枚举槽位后本守卫转绿；解锁的缺陷族见函数文档"]
        fn $fn_name() {
            let src = std::fs::read_to_string(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/src/event.rs"
            ))
            .expect("读 event.rs");
            assert!(
                src.contains($needle),
                concat!("引擎缺少槽位 `", $needle, "`：", $why),
            );
        }
    };
}

capability_guard!(cap_card_targeted, "CardTargeted", "「用牌前 / 指定目标时」——当前台账里仍有需档行依赖此槽位需档（useCardToBefore/ToBegin/ToTargeted/ToPlayered）");
capability_guard!(cap_sha_begin, "ShaBegin", "【杀】开始幕——条数随台账变动，不在此写死（lianwu/lingshi/shoulie/yueren；lingyue 见 sha 幕合计 6 条）");
capability_guard!(cap_sha_after, "ShaAfter", "【杀】结束幕——解锁 2 条（SwdLianda、XianjianXunying）");
capability_guard!(cap_sha_before, "ShaBefore", "【杀】之前幕——解锁 2 条（WudianZhuchengGuard、SwdTiebi）");
capability_guard!(cap_death_before, "DeathBefore", "「死亡前」——解锁 4+ 条（dieBegin 家族）");
capability_guard!(cap_game_start, "GameStart", "「游戏开始」——解锁 L20b（xigua_xueliangfanbei）");
capability_guard!(cap_judge_before, "JudgeBefore", "「判定前」——解锁 L03c（gaiming）");
capability_guard!(cap_damage_begin_second, "DamageBegin2", "「第 2/3/4 次」——解锁 Tianxiang、L20a");

// ------------------------------------- 「完全没有挂触发」与「事件种类错」的行为守卫


/// 需要「多幕」或「另一事件」的行为：断言目标事件的字面名出现在该行为的 metadata arm 附近。
/// 这里退一步只断言**该行为确实出现在 metadata 里**，把「幕是否正确」留给台账的 `fix` 列，
/// 以免守卫比缺陷本身更脆弱。
#[test]
#[ignore = "补 dieBegin 幕 / 改事件种类后转绿"]
fn guard_yxs_menshen3_extra_phase() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/metadata.rs"
    ))
    .expect("读 metadata.rs");
    let idx = src.find("HostBehavior::YxsMenshen3 =>").expect("YxsMenshen3 无 arm");
    let around = &src[idx..(idx + 600).min(src.len())];
    assert!(
        around.contains("DeathBefore") || around.contains("dieBegin"),
        "yxs_menshen3 上游还有 dieBegin 幕，原生只有 phaseBegin",
    );
}

/// `MbQianlong` / `MbWeitong` 同上：需要 `phaseBefore` 与 `enterGame` 两幕。
#[test]
#[ignore = "补 phaseBefore / enterGame 幕或改用 Dedicated 事件后转绿"]
fn guard_mb_phase_before_enter_game() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(
        src.contains("EnterGame") || src.contains("PhaseBefore"),
        "mbqianlong / mbweitong 需要「入场」与「阶段前」两幕",
    );
}

// ------------------------------------- 事件种类错 / 缺第二幕 的行为守卫

/// 断言某行为的 `TriggerMeta` 事件等于期望值（当前是错的，故忽略；改对后转绿）。
macro_rules! meta_event_guard {
    ($fn_name:ident, $variant:literal, $expected:literal, $why:literal) => {
        #[test]
        #[ignore = "把 TriggerMeta 事件改对后本守卫转绿"]
        fn $fn_name() {
            use std::path::Path;
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/content/host");
            let mut all = String::new();
            let mut stack = vec![dir];
            while let Some(next) = stack.pop() {
                let Ok(entries) = std::fs::read_dir(&next) else { continue };
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().is_some_and(|ext| ext == "rs") {
                        all.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
                    }
                }
            }
            let needle = format!("HostBehavior::{} =>", $variant);
            let idx = all.find(&needle).expect("该行为在 metadata 里没有 arm");
            let around = &all[idx..(idx + 400).min(all.len())];
            assert!(
                around.contains($expected),
                concat!("期望 ", $expected, "：", $why),
            );
        }
    };
}

meta_event_guard!(guard_niepan_dying, "Niepan", "Dying", "涅槃是濒死自救主动技，不是 Death");
meta_event_guard!(guard_shouyin_dying, "SwdShouyin", "Dying", "收印 filter 要求 event.type=='dying'");
meta_event_guard!(guard_chengxin_dying, "XianjianChengxin", "Dying", "濒死主动技");
meta_event_guard!(guard_shuiyun5_dying, "XianjianShuiyun5", "Dying", "濒死主动技");
meta_event_guard!(guard_cigu_pre_draw, "GwentCiguState", "DrawBefore", "上游 phaseDrawBegin：摸牌数确定前");
meta_event_guard!(guard_fuchou2_before, "WudianFuchou2", "PhaseBefore", "上游 phaseUseBefore");
meta_event_guard!(guard_fenglue_before, "XianjianFenglue", "PhaseBefore", "上游 phaseUseBefore");

/// 准备阶段幕（G18）：`HlssFengqi` 与 `Xiuluo` 需要 `Phase::Prepare`。
#[test]
#[ignore = "补 `Phase::Prepare` 后转绿"]
fn guard_prepare_phase_exists() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(src.contains("Prepare"), "上游 phaseZhunbeiBegin 需要准备阶段幕");
}

/// `HlssXiangxing` 少了「失去体力后」幕。
#[test]
#[ignore = "补 LoseHpEnd 幕并挂上后转绿"]
fn guard_xiangxing_lose_hp_end() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(src.contains("LoseHpEnd"), "上游 xiangxing 有 loseHpEnd 幕");
}

/// `HlssYueyin` 上游是「第 4 次伤害 + 伤害后」双幕。
#[test]
#[ignore = "补「第 4 次」档后转绿"]
fn guard_yueyin_fourth_damage() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(src.contains("DamageBegin4") || src.contains("Damage4"), "上游 damageBegin4");
}

/// `HlssGaiming` 上游 `judgeBefore`，原生完全没有 arm。
#[test]
#[ignore = "补 `judgeBefore` 幕并挂上后转绿"]
fn guard_gaiming_judge_before() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(src.contains("JudgeBefore"), "上游 gaiming 是 judgeBefore");
}

/// `HuanleKuwu`（5a）与 `ShengkeCheck`（5c）都卡在「用牌前 / 指定目标时」幕。
#[test]
#[ignore = "补 `CardTargeted` 后转绿（与 cap_card_targeted 同一能力，按行为分别记录）"]
fn guard_kuwu_and_shengke_card_targeted() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(src.contains("CardTargeted"), "kuwu / shengke_check 需要「使用到目标」幕");
}

/// `XiguaMopaiFanbei`（L20a）需要「摸牌阶段第 2 次」与「摸牌前」两者之一。
#[test]
#[ignore = "补「第 2 次」档或摸牌前幕后转绿"]
fn guard_mopaifanbei_second_draw() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/event.rs"
    ))
    .expect("读 event.rs");
    assert!(
        src.contains("DrawBegin2") || src.contains("DrawBefore"),
        "上游 mopaifanbei 是 phaseDrawBegin2",
    );
}

/// `Rangjie`（`6`）缺少分支 1 的摸牌动作。
#[test]
#[ignore = "补 `draw_cards` 后转绿（分支 1 应摸牌）"]
fn guard_rangjie_branch1_draws() {
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/step/skills_cards.rs"
    ))
    .expect("读 step/skills_cards.rs");
    let idx = src.find("Rangjie").expect("找不到 rangjie 的效果步");
    let body = &src[idx..(idx + 1200).min(src.len())];
    assert!(
        body.contains("draw") || body.contains("Draw"),
        "分支 1（移动牌）应同时摸牌",
    );
}

// ------------------------------------- G23：`PhaseEnd` 未钉阶段（应钉 End）

/// 断言某行为的过滤臂**钉住了指定阶段**（当前未钉 ⇒ 每个阶段结束都触发）。
macro_rules! pin_phase_guard {
    ($fn_name:ident, $variant:literal, $phase:literal, $why:literal) => {
        #[test]
        #[ignore = "把过滤臂钉到该阶段后本守卫转绿"]
        fn $fn_name() {
            use std::path::Path;
            let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/content/host");
            let mut all = String::new();
            let mut stack = vec![dir];
            while let Some(next) = stack.pop() {
                let Ok(entries) = std::fs::read_dir(&next) else { continue };
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        stack.push(path);
                    } else if path.extension().is_some_and(|ext| ext == "rs")
                        && (path
                            .file_name()
                            .and_then(|name| name.to_str())
                            .is_some_and(|name| name.starts_with("filtering"))
                            // `filtering/` 子目录里的实现（如 `filtering/common.rs`）文件名不以
                            // `filtering` 开头，必须按路径判断，否则只扫到委派臂（第 903 轮发现）。
                            || path
                                .components()
                                .any(|part| part.as_os_str().to_string_lossy() == "filtering"))
                    {
                        all.push_str(&std::fs::read_to_string(&path).unwrap_or_default());
                    }
                }
            }
            let needle = format!("HostBehavior::{} =>", $variant);
            // 跳过 `filtering.rs` 里的委派臂（`=> self.filter_common(...)`）——真正的过滤条件在
            // `filtering/common.rs` 等同名实现里，只查委派臂永远为红（第 903 轮发现）。
            let mut cursor = 0usize;
            let mut checked = false;
            while let Some(rel) = all[cursor..].find(&needle) {
                let start = cursor + rel;
                let end = all[start + 10..]
                    .find("HostBehavior::")
                    .map(|offset| start + 10 + offset)
                    .unwrap_or(all.len());
                let arm = &all[start..end];
                cursor = end;
                if arm.contains("filter_common(") {
                    continue;
                }
                checked = true;
                assert!(
                    arm.contains(concat!("Phase::", $phase)),
                    concat!("应钉 ", $phase, "：", $why),
                );
            }
            assert!(checked, "过滤臂里没有该行为的实现（只有委派臂）");
        }
    };
}

pin_phase_guard!(guard_hutian_pin_end, "Hutian", "End", "上游 phaseEnd（回合末一次）");
pin_phase_guard!(guard_mufeng_pin_end, "SwdMufengHeld", "End", "上游 phaseEnd（有每回合标记，但仍应钉 End）");
