# NC08 第⑦层首例：上游行为对照（`kurou`，2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层「上游行为」的**首个对照样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项由上游 JS 推导的规则点全部对齐**；**登记 1 处顺序差异**（先摸后失去 vs 上游书写顺序先失去后摸）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname-liuli/noname/extension/标记补充/extension.js`，`config.zhijiechifa` 分支
（原文片段见 [upstream-kurou.js.txt](upstream-kurou.js.txt)）：

```js
lib.skill.kurou = {
    audio: 2,
    enable: "phaseUse",       // 出牌阶段可用
    direct: true,             // 无目标直接生效
    content: function () {
        player.logSkill('kurou');
        player.loseHp(1);     // 失去 1 点体力（非伤害）
        player.draw(2);       // 摸 2 张
    },
    ai: { ... },              // AI 估值（非规则）
};
```

## 2 由 JS 推导的规则点 → 原生验收（新增 `crates/engine-native/tests/nc08_upstream_kurou.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | **无 `usable`/`round`/`turn` 限制** | 同一出牌阶段可**多次**发动：两次发动 → 体力 −2、手牌 +4 | 通过 |
| 2 | `loseHp`（失去体力，非 `damage`） | 产生 `HpLost` 且**无任何伤害事件**（`DamageDealt`/`Damage`） | 通过 |
| 3 | **无 `filter`** | 体力为 1 时仍可发动 → 进入濒死/阵亡（不被拒绝） | 通过 |
| 4 | `draw(2)` | 手牌 +2（与 #1 合并验收） | 通过 |
| — | — | **顺序差异登记**：原生 `CardsDrawn → HpLost`，上游书写顺序为 `loseHp(1); draw(2);` | **已登记** |

> 顺序差异的**影响分析**：在「失去体力后进入濒死」的场景下，摸到的【桃】是否已入手参与
> 求援会影响结果；是否对齐需按采用的上游语义（含上游 `loseHp` 的异步濒死流程）判定 →
> **NC08 待办**，本批只做**可复跑记录**（用例 `upstream_kurou_effect_order_is_recorded`
> 把观测顺序固定下来，防止静默改变）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 上游对照定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1157 passed / 0 failed / 2 ignored**（1153 → +4） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8262 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办（NC08 第⑦层）

- [ ] **顺序差异的上游判定**（`loseHp` 与 `draw` 的先后；含上游濒死异步流程）
- [ ] **逐命令对照工具**：固定上游版本/初始局面/随机输入与玩家选择，比较状态、事件、选择、
      随机流与续行（需上游 JS 侧可复跑 harness——本轮只做「单技能语义对照」样本）
- [ ] 更多来源的同类样本（本轮 1 例；37 来源待铺开）
- [ ] `audio`/`ai` 等表现与策略面不在规则对照范围（登记边界）
