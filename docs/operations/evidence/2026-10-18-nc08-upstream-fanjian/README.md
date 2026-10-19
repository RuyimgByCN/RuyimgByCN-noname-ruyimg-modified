# NC08 第⑦层样本 55：`fanjian`（反间，**标准面** / 周瑜）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 项通过，未发现差异**（流程三步与上游一致）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-fanjian.js.txt](upstream-fanjian.js.txt)）

```js
fanjian: {
    enable: "phaseUse", usable: 1,
    filter(event, player) { return player.hasCards("h"); },
    filterTarget(card, player, target) { return player !== target; },
    async content(event, trigger, player) {
        const { control } = await target.chooseControl({
            controls: ["heart2", "diamond2", "club2", "spade2"] }).forResult();      // ① 选花色
        const { bool, cards } = await target
            .gainPlayerCard({ target: player, position: "h", forced: true }).forResult();  // ② 强制取手牌
        if (bool && cards?.length && get.suit(cards[0], player) + "2" !== event.choice) {
            await target.damage({ nocard: true });                                   // ③ 不符 → 1 点伤害
        }
    },
},
```

## 2 验收结果（`nc08_upstream_fanjian.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3 | `chooseControl` → `gainPlayerCard` → 条件 `damage` | 询问序列实测 **`["fanjian_target","fanjian_suit","fanjian_pick"]`**（与上游三步一一对应）；目标**获得一张手牌**（手牌 `[30]`）；**花色相符时 hp 不变**（4 → 4） | 通过 |
| 1 | `usable: 1` + `hasCards("h")` + `player !== target` | 原生注释与实现一致（限一次、须有手牌、目标为其他角色）——源码事实 | 通过 |

**原生实现引用**：`content/standard/skills/fan_jian.rs`（注释「v203 对齐上游 `chooseControl`：♥/♦/♣/♠」
「花色与所选不符时受到周瑜造成的 1 点伤害（上游条件 damage，花色相符无事）」；`activate` 要求存活 + 手牌非空）。

## 3 结论（**未发现差异**）

流程**三步**（选花色 → 强制取手牌 → 不符受伤）与上游**一一对应**；限次/手牌/目标门槛一致。
**保留项**：上游 `ai()` 的花色随机分布（`Math.random()` 六分支）与 `chat`/`game.log` 表现面。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 花色**不符**分支的行为面（本样本只验证相符分支；不符分支需定向选另一花色）
- [ ] 表现面（`chat`/`log`）与 AI 花色分布
- [ ] 样本 56+：其余来源/技能（标准面技能 **34 个样本**；来源覆盖 **23/37**）
