# NC08 第⑦层样本 53：`guanxing`（观星，**标准面** / 诸葛亮）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过**（含 **3 项差异登记**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-guanxing.js.txt](upstream-guanxing.js.txt)）

```js
guanxing: {
    trigger: { player: "phaseZhunbeiBegin" },        // 准备阶段开始
    frequent: true,
    async content(event, trigger, player) {
        const num = player.hasSkill("yizhi") && player.hasSkill("guanxing")
            ? 5 : Math.min(5, game.countPlayer());   // X = 存活角色数，至多 5；遗志组合 → 5
        const result = await player.chooseToGuanxing(num).set("prompt", "观星：…").forResult();
        if (!result.bool || !result.moved[0].length) player.addTempSkill("guanxing_fail");
    },
},
guanxing_fail: {},
```

## 2 验收结果（`nc08_upstream_guanxing.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `chooseToGuanxing(num)` | 观星发动后出现 **`guanxing_arrange`** 询问（询问序列实测 `["discard_to_limit","trigger_confirm","guanxing_arrange"]`） | 通过 |
| 1 | `phaseZhunbeiBegin` | **差异①（族）**：原生锚在**判定阶段**（注释自述「判定阶段占位」）——原生无准备阶段，与 `luoshen` **同族** | 通过（**差异登记**） |
| 1 | `hasSkill("yizhi") && hasSkill("guanxing") → 5` | **差异②**：原生**未见 `yizhi`（遗志）组合判定** | 通过（**差异登记**） |
| 3 | `addTempSkill("guanxing_fail")` | **差异③**：原生**未见 `guanxing_fail` 标记** | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/guan_xing.rs`（filter `Phase::Judge + begin`；`guanxing_arrange` 询问）。

## 3 夹具经验（写入证据，**复用天妒教训**）

**引擎初态即 0 号判定阶段**（该 `begin` 已发出）→ 若在 `Engine::new` 之后授予观星，
**本轮判定阶段不会再触发** → 夹具须**先推进到本轮 Draw，再授予技能，然后驱动一整轮**等下一轮判定阶段
（初版 `询问序列=[]` 即此原因；修正后 `guanxing_arrange` 正常出现）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed**（NOTE 行数 0） | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（时机锚点）随该族评估（当前：luoshen、guanxing）
- [ ] 差异②（`yizhi` 组合 X=5）与差异③（`guanxing_fail`）口径与修复
- [ ] X 的具体取值行为面（需构造不同存活人数）
- [ ] 样本 54+：其余来源/技能（标准面技能 **32 个样本**；来源覆盖 **23/37**）
