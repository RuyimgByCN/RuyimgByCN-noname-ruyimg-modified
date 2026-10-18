//! NC08：**准备阶段（上游 `phaseZhunbei`）在原生被映射到判定阶段起点**——把映射固定为
//! 可执行断言，避免静默改变。
//!
//! 上游依据（归档快照）：
//! - `apps/core/mode/identity.js` + `docs/game-startup-flow.md`：阶段顺序
//!   `phaseZhunbei / phaseJudge / phaseDraw / phaseUse / phaseDiscard / phaseJieshu`；
//! - `apps/core/extension/杀海拾遗/character/swd/skill.js`（`swd_xiuluo`）：
//!   `trigger: { player: "phaseZhunbeiBegin" }`，`filter: player.countCards("j") > 0`。
//!
//! 原生实现（源码引用）：
//! - `content/host/metadata/common.rs`：`Xiuluo` 的 `TriggerMeta.event = PhaseBegin`，
//!   注释「修罗：自己准备阶段开始（可选）」；
//! - `content/host/filtering/common.rs`：`Xiuluo` 过滤条件为
//!   `Phase { phase: Judge, begin: true }` 且判定区非空、手牌非空。
//!
//! 结论：**准备阶段不作为一个独立相位存在**，其触发被**并入判定阶段起点**（原生注释
//! 「Judge 阶段 = 上游 phaseZhunbei」）。由此产生的语义边界：准备与判定两类触发
//! **相对次序**由 `Timing`/`priority` 决定，不再由相位顺序保证——需按技能逐条核对
//! （登记为 NC08 待办）。

/// 映射固定：修罗的触发事件为 `PhaseBegin`（相位由过滤条件限定为 Judge 起点）。
#[test]
fn prepare_phase_triggers_are_folded_into_judge_begin() {
    // 该断言通过源码引用固定事实：原生没有独立 Prepare 相位（见 Phase::ALL），
    // 准备阶段类行为以 `PhaseBegin` + Judge 过滤表达。
    let phases: Vec<String> = engine_native::Phase::ALL
        .iter()
        .map(|p| format!("{p:?}"))
        .collect();
    assert!(
        !phases.iter().any(|p| p == "Prepare"),
        "原生相位集合不含 Prepare（上游 phaseZhunbei 已并入 Judge 起点）：{phases:?}"
    );
    // 行为键名稳定（防止重命名导致映射失效）：以源码引用固定
    let keys = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/keys.rs"
    ))
    .expect("读键表源码");
    assert!(
        keys.contains("\"host:swd_xiuluo\""),
        "修罗行为键 host:swd_xiuluo 应保持稳定"
    );
}

/// 上游 `swd_xiuluo` 的 filter 是 `countCards("j") > 0`；原生额外要求手牌非空
/// （弃一张手牌作为代价，无手牌则不可发动）——**登记为过滤差异**。
#[test]
fn xiuluo_filter_requires_hand_which_upstream_does_not() {
    // 上游：`filter(event, player) { return player.countCards("j") > 0; }`
    // 原生（filtering/common.rs）：`!judge.is_empty() && !hand.is_empty()`
    // 差异影响：判定区有牌但**手牌为空**时，上游可发动（随后 discardPlayerCard 失败/取消），
    // 原生直接不触发。此处只固定「原生要求手牌」这一事实，供上游判定。
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/src/content/host/filtering/common.rs"
    ))
    .expect("读过滤源码");
    let idx = src.find("HostBehavior::Xiuluo =>").expect("修罗过滤分支");
    // 按字符边界截取（源码含中文注释）
    let seg: String = src[idx..].chars().take(600).collect();
    let seg = seg.as_str();
    assert!(
        seg.contains("judge.is_empty()") && seg.contains("hand.is_empty()"),
        "原生修罗过滤应同时要求判定区与手牌非空（差异登记）：{seg}"
    );
}
