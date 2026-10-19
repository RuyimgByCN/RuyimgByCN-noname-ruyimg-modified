# NC08 第⑦层样本 49：`paoxiao`（咆哮，**标准面** / 张飞）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-paoxiao.js.txt](upstream-paoxiao.js.txt)）

```js
paoxiao: {
    audio: 2, firstDo: true,
    trigger: { player: "useCard1" },          // 第 2 张起的语音触发
    forced: true,
    filter(event, player) {
        return !event.audioed && event.card.name === "sha"
            && player.countUsed("sha", true) > 1 && event.getParent()?.type === "phase";
    },
    mod: {
        cardUsable(card, player, num) { if (card.name == "sha") return Infinity; },
    },
    ai: { unequip: true, … },
},
```

## 2 验收结果（`nc08_upstream_paoxiao.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `mod.cardUsable → Infinity` | 持咆哮者出牌阶段**连续两张【杀】均被接受** | 通过 |
| 1 | 同上（**反例**） | **无咆哮**者同回合第二张【杀】→ **`Rejected(ShaLimit)`** | 通过 |
| 1 | `mod.cardUsable` | **差异登记**：上游**统一把上限设为 `Infinity`**；原生在**限次校验处 bypass**（`!paoxiao && …`）→ 语义等价、**维护性风险族第 3 例** | 通过（**差异登记**） |

**原生实现引用**：`engine/play_sha.rs:32`（读 `PAO_XIAO`）、`:118`（`if !paoxiao && … && sha_used_play_phase`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] `firstDo`/`trigger`（语音面）与 `ai.unequip` 策略面
- [ ] 维护性风险族（`mod` ↔ 显式 bypass/校验）当前 **3 例**：kongcheng、qianxun、paoxiao
- [ ] 样本 50+：其余来源/技能（标准面技能 **28 个样本**；来源覆盖 **23/37**）
