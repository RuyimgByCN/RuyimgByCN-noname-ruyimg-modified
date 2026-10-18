# NC08 第⑦层样本 15：`zhiyuan`（支援，U05 ow 武将包 / 安娜）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 ow 武将包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**由上游 JS 推导的 4 条规则点全部对齐**（无新差异登记）

## 1 上游源码（归档快照，逐字引用；见 [upstream-zhiyuan.js.txt](upstream-zhiyuan.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/ow/skill.js`：

```js
zhiyuan: {
    trigger: { source: "damageBefore" },       // 自己即将造成伤害
    check(event, player) { … },                // AI 是否发动（策略面）
    logTarget: "player",
    filter(event, player) { return event.num > 0; },
    content() {
        trigger.cancel();                      // 防止此伤害
        trigger.player.recover(trigger.num);   // 目标回复等量体力
    },
},
```

## 2 验收结果（`nc08_upstream_zhiyuan.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 3+4 | `content: cancel() + recover(num)` | 确认发动 → **伤害被防止**且目标**回复等量**（2 点伤害 → 目标体力 +2） | 通过 |
| 3 | 可选技（需确认门） | **取消**确认 → 伤害照常结算（体力 −1，不回复） | 通过 |
| 1+2 | `trigger: {source: "damageBefore"}` + `filter: num > 0` | **非自己造成**的伤害不触发（无确认门） | 通过 |

**原生实现引用**：`content/host/filtering/common.rs:762`（`Damage` 且 `source == owner &&
!prevented && amount > 0`）、`content/host/step/wudian_recovery.rs:271`（`d.prevented = true` +
`recover(hurt, num)`）。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [ ] 样本 16+：其余来源（当前 **14/37 来源**）
- [ ] ow 其余 23 名武将的上游样本（本样本为 1/24）
- [ ] `check`/`ai`（策略面）与 `logTarget`（表现面）不在规则对照范围
- [ ] 逐命令对照工具
