# NC08 第⑦层样本 54：`liuli`（流离，**标准面** / 大乔）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过，未发现差异**（对齐良好）；仅表现面未覆盖
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-liuli.js.txt](upstream-liuli.js.txt)）

```js
liuli: {
    trigger: { target: "useCardToTarget" },          // 成为【杀】目标时
    filter(event, player) {
        if (event.card.name !== "sha") return false;
        if (!player.hasCards("he")) return false;
        return game.hasPlayer(current =>
            player.inRange(current) && current !== event.player && current !== player
            && !!lib.filter.targetEnabled(event.card, event.player, current));
    },
    async cost(event, trigger, player) { event.result = await player.chooseCardTarget({ … }); },
},
```

## 2 验收结果（`nc08_upstream_liuli.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2+3 | `chooseCardTarget`（弃牌 + 选目标） | 询问序列实测 **`["liuli_discard_choose","liuli_transfer"]`**（**先选牌后选目标**，与上游 `cost` 顺序一致）；费用牌**被弃置**；**转移生效**（新目标 2 号 hp 4 → 3） | 通过 |
| 3 | `targetEnabled(...)` | **跨技能交互**：2 号持**空城**且无手牌 → 流离的**目标询问未出现**、且**未把空城者列为候选**（与原生注释「**空城不可**」一致） | 通过 |

**原生实现引用**：`kernel/sha/step.rs:630`（注释自述「**对齐上游 chooseCardTarget**」；弃牌候选 =
手牌 ∪ 装备；「整体可取消」；「**空城不可**」）。

## 3 结论（**未发现差异**）

原生实现与上游四条判据**逐条对应**：时机（成为【杀】目标）✓、费用（`he`）✓、
目标（范围内 / 非使用者 / 非自己 / **为该杀合法目标**）✓、可取消 ✓——
其中 `targetEnabled` 复查使**空城目标被正确排除**（**跨技能交互亦对齐**）。

**保留项**：`preHidden`/`audioname` 表现面；`cost` 中「先选牌后选目标」的**逐帧顺序**（本样本按询问
序列已间接验证）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed**（NOTE 行数 0） | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 表现面（`preHidden`/`audioname`）
- [ ] 样本 55+：其余来源/技能（标准面技能 **33 个样本**；来源覆盖 **23/37**）
