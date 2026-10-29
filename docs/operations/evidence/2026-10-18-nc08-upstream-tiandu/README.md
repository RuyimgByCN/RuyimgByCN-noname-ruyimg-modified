# NC08 第⑦层样本 34：`tiandu`（天妒，**标准面** / 郭嘉）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 2 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-tiandu.js.txt](upstream-tiandu.js.txt)）

```js
tiandu: {
    trigger: { player: "judgeEnd" },
    frequent(event) { return event.result.card?.name !== "du"; },   // 毒牌除外
    check(event) { return event.result.card?.name !== "du"; },
    filter(event, player) { return get.position(event.result.card, true) === "o"; },
    async content(event, trigger, player) { … },                    // 获得该判定牌
},
```

## 2 验收结果（`nc08_upstream_tiandu.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | `judgeEnd` + 获得判定牌 | **根因查明后转为通过**：在 0 号 **Draw 阶段**放入【乐不思蜀】→ 下一轮判定阶段消费 → 出现 `tian_du_gain` 且确认后手牌非空 | 通过 |
| 2 | `name !== "du"` | **差异①**：原生构建 `tian_du` 列表**只判持有者**（`vec![self.owner]`），**无牌名判断** → 【毒】未除外 | 通过（**差异登记**） |
| 2 | `position === "o"` | **差异②（实现方式）**：原生以机器字段 `judge_held` 等价表达「牌仍在处理区」 | 通过（**差异登记**） |
| 1 | 技能门槛 | **未持天妒** → **无** `tian_du_gain` 询问 | 通过 |

**原生实现引用**：`kernel/judge.rs:626–670`（`tian_du_self` → `tian_du: vec![self.owner]`）、
`:486–505`（询问 `tian_du_gain`，置 `gained`/`judge_held`）。

## 3 口径与限制（不夸大）

- **根因已查明（同轮）**：判定区在 **`Phase{Judge, begin:true}` 事件处被 `std::mem::take` 消费**
  （`kernel/intrinsic.rs:294`）；而引擎**初态即「0 号判定阶段」**（begin 已发出）→ 在 `Engine::new`
  之后放入判定区**永远不会被结算**（初版夹具即因此失败：`判定区仍为 [74]、零询问`）。
  **修法**：改在**本轮判定阶段之后**（0 号 Draw 阶段）放入 → 由**下一轮**判定阶段开始消费
  → 用例**转为通过**（[targeted.log](targeted.log)；`skip-note.txt` 现为空文件）；
- 差异①是否**可观测**取决于原生目录是否存在【毒】牌（`name == "du"`）→ **未核对**（登记待办），
  故本条**只作为源码事实差异**，不声称可复现的行为差异。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed / 0 ignored** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [x] **行为面夹具已修**（判定区在 Judge 阶段 begin 处被消费；改为 Draw 阶段放入、下一轮消费）
- [ ] 核对原生目录是否存在【毒】（`du`）：若存在则差异①**可观测**并应修
- [ ] 差异②（`judge_held` ↔ `position === 'o'`）是否在所有路径等价（含改判/天道等分支）
- [ ] 样本 35+：其余来源（当前 **21/37 来源**；标准面技能 **16 个样本**）
