# 2026-10-16 临时技能存续期勘验（三方语义对照）

核验日期：2026-10-16。**只读核对**（未改行为；规则 v290 / 存档 v77）。

覆盖清单行为差异表「临时技能存续期」行。三方语义：

| 线 | 存续期语义 | 依据 |
|---|---|---|
| 原生 | 至**拥有者**回合结束（`expires_end_of_turn`；5 处授技站点：妄尊印/移伤/乐学/幻形/石会粉禁用） | `machine.rs:499-552`、`kernel.rs:3706`；`cursor` 结构扫描见 [temp-skill-expiry-audit.txt](temp-skill-expiry-audit.txt) |
| 旧线 | 至**授技时当前回合玩家**的回合结束 | `engine-core/src/effect/primitives/pack_tw_1.rs:388-434`、`command/turn.rs:110-118` |
| 上游 | 逐站点 `addTempSkill(skill, expire)` 规格：默认 `{global:['phaseAfter','phaseBeforeStart']}`（`phaseAfter` 无触发点、`phaseBeforeStart` 每阶段开始触发 ⇒ 默认≈至当前阶段结束）；乐学=`addSkill`+拥有者阶段边界重掷；幻形=`addAdditionalSkill`+受伤移除 | `player.js:11952-11970`、`gameEvent.ts:585-619`、`content.ts:4742`、`swd/skill.js:5950-6010/3039-3129`、`character/standard/skill.js:141-176` |

**处置**：已并入[规则基准「语义选择：窗口/顺序口径（2026-10-16 定案）」](../../../architecture/native-rules-baseline.md)——
保留原生「至拥有者回合结束」（上界近似：不会提前失效、可能延长可用窗口）；逐站点粒度差异在各批次台账登记；
要改「阶段边界结束」需新 expire 原语＋逐站点核对，另立项。

**未核对（不得据此对齐）**：妄尊 `addMark("rewangzun2",1)` 与技能实例配合下的实际手牌上限窗口未复算；
上游其余 `addTempSkill` 调用点未全量清点。

完整源码摘录见 [temp-skill-expiry-audit.txt](temp-skill-expiry-audit.txt)。
