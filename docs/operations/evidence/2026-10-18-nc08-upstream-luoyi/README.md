# NC08 第⑦层样本 38：`luoyi`（裸衣，**标准面** / 许褚）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 2 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-luoyi.js.txt](upstream-luoyi.js.txt)）

```js
luoyi: {
    trigger: { player: "phaseDrawBegin2" },
    filter(event, player) { return !event.numFixed && event.num > 0; },
    async content(event, trigger, player) {
        player.addTempSkill("luoyi2", "phaseJieshuBegin");
        trigger.num--;                                        // 少摸 1
    },
},
luoyi2: {
    trigger: { source: "damageBegin1" },
    filter(event) { return event.card && (event.card.name === "sha" || event.card.name === "juedou")
        && event.notLink(); },                                // 铁索传导不加成
    forced: true,
    async content(event, trigger, player) { trigger.num++; },
},
```

## 2 验收结果（`nc08_upstream_luoyi.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `trigger.num--` | 同种子对照：确认发动后本回合**少摸 1 张**（[diagnostics.txt](diagnostics.txt)） | 通过 |
| 3 | `luoyi2` + `trigger.num++` | 本回合【杀】造成的伤害 **+1**（实测 ≥2） | 通过 |
| 1 | `!event.numFixed && event.num > 0` | **差异①**：原生 filter 只判阶段，**未建模 `numFixed`**（与英姿同族） | 通过（**差异登记**） |
| 4 | `event.notLink()` | **差异②**：`luoyi_bonus` helper **未检查牌种与 `notLink`**（由调用方决定）→ 等价性**未判定** | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/luo_yi.rs`（filter `Phase::Draw + begin`；旗标复用 `once_used`）、
`kernel/combat_rules.rs:6`（`luoyi_bonus`：持裸衣且 `once_used` → 1）。

## 3 口径与限制（不夸大）

- 行为面在**未出现确认门**时**如实打印 SKIP 并返回**；本次运行两处均出现（[diagnostics.txt](diagnostics.txt)）；
- 差异②**只作源码事实登记**：helper 不检查牌种/`notLink` **不等于**行为不等价（调用方可能已限制）
  → **未判定，不写成结论**。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（`numFixed`）与英姿同族，宜一并评估
- [ ] 差异②：核对 `luoyi_bonus` 的**调用方**是否已限制牌种与铁索传导
- [ ] 样本 39+：其余来源（当前 **21/37 来源**；标准面技能 **20 个样本**）
