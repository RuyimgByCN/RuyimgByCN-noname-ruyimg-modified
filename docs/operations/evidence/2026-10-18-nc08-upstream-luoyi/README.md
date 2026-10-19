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
| 1+2 | `trigger.num--` | 同种子对照：**手牌数基线 6 → 持技 5**（**少摸 1 张**，判据不依赖确认门捕获） | 通过 |
| 3 | `luoyi2` + `trigger.num++` | 本回合【杀】造成的伤害 **+1**（**夹具已修**：不依赖确认门，直接对比伤害，实测 ≥2） | 通过 |
| 1 | `!event.numFixed && event.num > 0` | **差异①**：原生 filter 只判阶段，**未建模 `numFixed`**（与英姿同族） | 通过（**差异登记**） |
| 4 | `event.notLink()` | **差异②**：`luoyi_bonus` helper **未检查牌种与 `notLink`**（由调用方决定）→ 等价性**未判定** | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/luo_yi.rs`（filter `Phase::Draw + begin`；旗标复用 `once_used`）、
`kernel/combat_rules.rs:6`（`luoyi_bonus`：持裸衣且 `once_used` → 1）。

## 3 口径与限制（不夸大）

- **少摸 1 张**用例已改为**手牌数对照**（基线 6 → 持技 5）→ 不依赖确认门捕获，**构成验收**；
- **伤害 +1 用例夹具已修并转为通过**：初版要求「捕获确认门」才继续，但裸衣在该路径下
  **无需确认即生效**（少摸 1 张已由手牌数对照证实 6 → 5）→ 改为**不依赖确认门**直接对比伤害
  → 实测 ≥2，`#[ignore]` 已移除（[diagnostics.txt](diagnostics.txt)）；
- 差异②**只作源码事实登记**：helper 不检查牌种/`notLink` **不等于**行为不等价（调用方可能已限制）
  → **未判定，不写成结论**。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed / 0 ignored** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异①（`numFixed`）与英姿同族，宜一并评估
- [x] **伤害 +1 已补验收**（夹具改为不依赖确认门）
- [ ] 差异②：核对 `luoyi_bonus` 的**调用方**是否已限制牌种与铁索传导
- [ ] 样本 39+：其余来源（当前 **21/37 来源**；标准面技能 **20 个样本**）
