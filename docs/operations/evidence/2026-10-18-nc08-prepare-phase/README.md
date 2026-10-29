# NC08：准备阶段（上游 `phaseZhunbei`）的原生落点（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）——上一批「原生缺 Prepare 阶段」登记的**语义影响核查**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**准备阶段不独立存在，其触发被并入判定阶段起点**（原生源码显式注明）；由此产生 **2 处需上游判定的边界**，本批**固定为可执行断言**（不静默）

## 1 上游依据

- 阶段顺序（归档文档 `docs/game-startup-flow.md` + `apps/core/mode/identity.js`）：
  `phaseZhunbei / phaseJudge / phaseDraw / phaseUse / phaseDiscard / phaseJieshu`；
- 具体技能（`apps/core/extension/杀海拾遗/character/swd/skill.js`，`swd_xiuluo`）：
  ```js
  swd_xiuluo: {
      trigger: { player: "phaseZhunbeiBegin" },
      filter(event, player) { return player.countCards("j") > 0; },
      direct: true,
      content() { /* 弃一张手牌弃置一张同花色判定牌 */ },
  }
  ```

## 2 原生落点（源码引用）

| 面 | 原生实现 | 引用 |
|---|---|---|
| 相位集合 | `Judge/Draw/Play/Discard/End`（**无 Prepare**） | `crates/engine-native-kernel/src/event.rs`（`Phase::ALL`） |
| 准备阶段类触发 | `TriggerMeta.event = PhaseBegin` + 过滤条件限定 `Phase::Judge, begin: true`；源码注释「**Judge 阶段 = 上游 phaseZhunbei**」 | `vocabulary/host/metadata/common.rs:100`（起风）、`:272`（修罗）、`:533`（青翼/赤翼） |
| 修罗过滤 | `judge` 非空 **且** `hand` 非空 | `content/host/filtering/common.rs:217` |

统计：`metadata/common.rs` 中显式标注「准备阶段」的映射 **3 处**（起风 / 修罗 / 青翼·赤翼锚点）。

## 3 由该映射产生的边界（登记，逐条固定）

| # | 边界 | 影响 | 处置 |
|---|---|---|---|
| 1 | **准备与判定的相对次序**不再由相位顺序保证，改由 `Timing`/`priority` 决定 | 同一回合内「准备阶段触发」与「判定阶段触发」的执行先后可能与上游不同 | 登记待办：按技能逐条核对（本批未做全量核对） |
| 2 | **修罗过滤差异**：上游仅要求判定区有牌；原生**额外要求手牌非空** | 判定区有牌但手牌为空时，上游可发动（随后弃牌代价失败/取消），原生直接不触发 | 登记待办：按上游判定是否放宽 |

## 4 回归（新增 `crates/engine-native/tests/nc08_prepare_phase_mapping.rs`，2 项）

| 判据 | 结果 |
|---|---|
| 原生相位集合不含 `Prepare`；行为键 `host:swd_xiuluo` 保持稳定（防重命名） | 通过 |
| 修罗过滤在源码中**同时**要求 `judge` 与 `hand` 非空（差异固定，防静默放宽） | 通过 |

> 两例均为**事实固定型**断言：它们不声称行为正确，而是让「映射」与「过滤差异」在源码变化时**必然失败**，从而强制复核。

## 5 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1166 passed / 0 failed / 2 ignored**（1164 → +2） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8266 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 6 未完成 / 待办

- [ ] 准备阶段类触发的**逐技能核对**（原生共 3 处显式映射 + 其它以 `PhaseBegin` 表达的行为）
- [ ] 修罗过滤差异的上游判定（是否放宽为「仅判定区非空」）
- [ ] 若确认需要独立 Prepare 相位 → 属规则变化（版本递增 + 14 golden 重录 + 回退探针）
- [ ] 逐命令上游对照工具与更多来源样本
