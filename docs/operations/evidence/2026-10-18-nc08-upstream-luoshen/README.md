# NC08 第⑦层样本 28：`luoshen`（洛神，**标准面** / 甄姬）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 **2 项差异登记**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-luoshen.js.txt](upstream-luoshen.js.txt)）

```js
luoshen: {
    trigger: { player: "phaseZhunbeiBegin" },        // 准备阶段开始
    async content(event, trigger, player) {
        while (true) {
            const judgeEvent = player.judge({ judge(card) {
                if (get.color(card) == "black") return 1.5; }, … });
            judgeEvent.set("callback", … gain(event.card) …);   // 黑色判定牌归自己
            const result = await judgeEvent.forResult();
            if (!result?.bool || !result.card) break;           // 红色 → 结束
            const result2 = await player.chooseBool({ prompt: "是否再次发动【洛神】？" });
            if (!result2?.bool) break;                           // 每轮单独询问
        }
    },
},
```

## 2 验收结果（`nc08_upstream_luoshen.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | 循环：黑→获得，红→结束 | 牌堆顶先黑后红：确认发动后黑色判定牌**离开牌堆**（被获得或入弃牌堆） | 通过 |
| 1 | `phaseZhunbeiBegin` | **差异①**：原生锚点在**判定阶段**（`Phase::Judge` + begin）——原生无准备阶段（**已登记**） | 通过（**差异登记①**） |
| 3 | `chooseBool` 每轮询问 | **差异②**：原生为「**整链一次确认，不分次放弃**」（源码注释自述） | 通过（**差异登记②**） |
| 1 | 技能门槛 | **未持洛神** → 判定阶段**无确认门** | 通过 |

**原生实现引用**：`content/standard/skills/luo_shen.rs`（filter `Phase::Judge` + begin；
注释「整链一次确认，不分次放弃」）。

## 3 口径与限制（不夸大）

- 行为面用例在**未出现确认门**时**如实打印 SKIP 并返回**（不写成通过）；本次运行出现了确认门；
- 判定牌由 `debug_set_deck_top_by`（按花色谓词置顶）布阵——**按谓词而非名字**，避免
  `debug_place` 的实例问题（见[夹具审计](../2026-10-18-nc08-fixture-audit/README.md)）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异②（每轮询问 vs 整链一次确认）口径判定与修复
- [ ] 国战模式 `rejudge` 分支（上游存在，原生未建模）
- [ ] 样本 29+：其余来源（当前 **21/37 来源**；标准面技能 **10 个样本**）
