# NC08 第⑦层样本 18：`ziyu`（自愈，U05 ow 武将包 / 禅雅塔）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项验收通过**（效果面「摸/回复」二选一与满血分支全部实测）

## 1 上游源码（归档快照，逐字引用；见 [upstream-ziyu.js.txt](upstream-ziyu.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`：

```js
ziyu: {
    trigger: { global: "phaseEnd" },          // 任意角色的阶段结束
    direct: true,
    filter(event, player) {
        return game.phaseNumber % 4 === 0;    // 每 4 个阶段一次
    },
    content() {
        player.chooseDrawRecover(get.prompt("ziyu")).logSkill = "ziyu";
    },
},
```

## 2 验收结果（`nc08_upstream_ziyu.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `global: phaseEnd` + `phaseNumber % 4 === 0` | 逐步推进至第 4 的倍数个阶段 → 出现 `wudian_ziyu_choice` 询问，含「摸一张」且**可取消** | 通过 |
| 3+4 | `chooseDrawRecover` | **已受伤** → 含「回复 1 点体力」；选回复 → 体力 +1 | 通过 |
| 4 | 同上 | **满血** → **不提供**回复选项，仍提供「摸一张」 | 通过 |
| 3 | 同上 | 选「摸」→ 手牌 +1 | 通过 |

**原生实现引用**：`content/host/filtering/common.rs:591`（`Phase` 事件 + `phase_number % 4 == 0`）、
`content/host/step/wudian_recovery.rs:15`（`Tag(0)`=摸 / `Tag(1)`=回复；满血不提供 `Tag(1)`；可取消）。

## 3 工程发现（写入证据）

- **可选触发的 `trigger_confirm` 必须确认**：本样本初版与修罗样本同因失败——夹具把确认门
  一并取消，导致技能**永不发动**（`phase_number` 已到 160 却零询问）。**已在两处样本登记该教训**。
- 诊断埋点位置要放在**早退分支之前**，否则永远不执行（本样本初版即踩）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 样本 19+：其余来源（当前 **16/37 来源**；ow 内 3/24）
- [ ] 阶段计数起点差异核对（上游 `game.phaseNumber` 为全局阶段计数，原生为 `turn.phase_number`）
- [ ] `direct`/`logSkill` 表现面不在规则对照范围
