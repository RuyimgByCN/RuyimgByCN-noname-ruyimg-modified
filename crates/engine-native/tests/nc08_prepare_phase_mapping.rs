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

//! R05（A2 审计，规则 333）：准备阶段作为独立相位真实存在（Phase::Prepare，
//! 回合开始窗→准备阶段→判定……）。原「准备阶段并入判定起点」的登记差异
//! 反转；修罗等 phaseZhunbeiBegin 技能按审计分批迁移到 Phase{Prepare,begin}。

/// 锁面：原生相位集合现在**包含** Prepare（声明序首位；VM 索引表尾追加 5；
/// 存档编码位 5），且每回合阶段序为 准备→判定→摸牌→出牌→弃牌→结束。
#[test]
fn prepare_phase_exists_as_real_phase() {
    let phases: Vec<String> = engine_native::Phase::ALL
        .iter()
        .map(|p| format!("{p:?}"))
        .collect();
    assert!(
        phases.iter().any(|p| p == "Prepare"),
        "原生相位集合含 Prepare：{phases:?}"
    );
    assert_eq!(
        engine_native::Phase::ALL[..5],
        [
            engine_native::Phase::Judge,
            engine_native::Phase::Draw,
            engine_native::Phase::Play,
            engine_native::Phase::Discard,
            engine_native::Phase::End
        ],
        "VM 阶段索引 0–4 不移位（Prepare 为表尾 5）"
    );
    // 行为键名稳定（防止重命名导致映射失效）
    let keys = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-kernel/src/vocabulary/host/keys.rs"
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
    let src = std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../engine-native-content/src/content/host/filtering/common.rs"
    ))
    .expect("读过滤源码");
    let idx = src.find("HostBehavior::Xiuluo =>").expect("修罗过滤分支");
    let seg: String = src[idx..].chars().take(600).collect();
    assert!(
        seg.contains("hand.is_empty()"),
        "修罗过滤仍要求手牌非空（登记差异锁面）：{seg}"
    );
}
