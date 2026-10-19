# NC08 第⑦层样本 32：`xiaoji`（枭姬，**标准面** / 孙尚香）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记，属**规则损失**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-xiaoji.js.txt](upstream-xiaoji.js.txt)）

```js
xiaoji: {
    trigger: {
        player: "loseAfter",
        global: ["equipAfter", "addJudgeAfter", "gainAfter", "loseAsyncAfter", "addToExpansionAfter"],
    },
    getIndex(event, player) {
        const evt = event.getl(player);
        if (evt?.player === player && evt.es) return evt.es.length;   // 失去的**装备区牌数**
        return false;
    },
    async content(event, trigger, player) { await player.draw(2); },
},
```

## 2 验收结果（`nc08_upstream_xiaoji.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `loseAfter` + `es` + `draw(2)`（**替换**方式） | 同栏位换装 → 确认发动后手牌**净 +1**（打出 -1、枭姬 +2） | 通过 |
| 1 | 同上（**弃置/被拆**方式） | **差异登记（规则损失）**：装备被【过河拆桥】拆走 → **不触发**（[discard-loss-diagnostic.txt](discard-loss-diagnostic.txt)） | 通过（**差异登记**） |
| 1 | 技能门槛 | **未持枭姬** → 替换装备**无确认门**且不摸牌 | 通过 |

**原生实现引用**：`content/standard/skills/xiao_ji.rs`（filter **仅**匹配
`EventData::Equipped { replaced: Some(_) }`）。

## 3 差异说明（不夸大）

上游 `loseAfter` + `es` 计数覆盖**所有**失去装备的方式（替换 / 被弃置 / 被获得 / 被拆 …）；
原生只覆盖**替换**（同栏位换装）——`XIAO_JI` 在 crate 内除定义与行为外**无其他引用**，
且各 `equips.retain(...)` 的弃置路径**不发** `Equipped{replaced}` → **规则损失**。

**修法**：为「装备区失去牌」引入统一事件（或在各 `equips.retain` 路径补发），
`xiao_ji.rs` 的 filter 改为按「失去的装备区牌数 > 0」判定，并按数量 ×2 摸牌。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异修复**：统一「失去装备区牌」事件并放开 filter（`xiao_ji.rs` **未跟踪** → 并入待实施修复清单）
- [ ] 失去**多张**装备时的摸牌张数（上游按 `es.length` 计次）
- [ ] 样本 33+：其余来源（当前 **21/37 来源**；标准面技能 **14 个样本**）
