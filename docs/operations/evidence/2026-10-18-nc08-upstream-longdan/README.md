# NC08 第⑦层样本 24：`longdan`（龙胆，**标准面** / 赵云）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 1 项差异登记）；**首次验证「闪响应面」的转化确实存在**（与武圣样本 §7 互补）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-longdan.js.txt](upstream-longdan.js.txt)）

```js
longdan: {
    group: ["longdan_sha", "longdan_shan", "longdan_draw"],
    subSkill: {
        sha:  { enable: ["chooseToUse","chooseToRespond"], filterCard: { name: "shan" },
                viewAs: { name: "sha" }, position: "hs", prompt: "将一张闪当杀使用或打出" },
        shan: { enable: ["chooseToRespond","chooseToUse"], filterCard: { name: "sha" },
                viewAs: { name: "shan" }, position: "hs", prompt: "将一张杀当闪使用或打出" },
        draw: { trigger: { player: ["useCard","respond"] }, forced: true,
                filter(event, player) { return get.zhu(player, "shouyue") && … },
                async content(event, trigger, player) { await player.draw(); … } },
    },
},
```

## 2 验收结果（`nc08_upstream_longdan.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+2 | `longdan_sha`（【闪】当【杀】，`chooseToUse`） | 持龙胆时【闪】可当【杀】**使用**（被接受且结算伤害） | 通过 |
| 1 | 同上（反例） | **未持龙胆** → 【闪】不可当【杀】 | 通过 |
| 1+2 | `longdan_shan`（【杀】当【闪】，`chooseToRespond`） | 被杀指定时，`respond_shan` 的候选**包含实体【杀】**（转化可用） | 通过（**响应面转化确证**） |
| 4 | `longdan_draw` + `get.zhu(player,"shouyue")` | **登记差异②**：守约下用转化牌摸 1 的**摸牌面未建模**（全 crate 无 `shouyue`） | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:154`（使用面 `longdan && kind == Shan → Sha`）、
`kernel/sha/step.rs:672–725`、`kernel/sha/resume.rs:322–345`（闪响应面 `kind == Shan || (longdan && kind == Sha)`）、
无双双闪臂 `:369–392`（同口径）。

## 3 与武圣样本 §7 的**互补关系**（方向不对称的正面证据）

- **「闪」方向**（【杀】→【闪】）：**已实现** —— 本样本第 3 项实测候选含实体【杀】；
- **「杀」方向**（【闪】→【杀】）：**使用面已实现**（本样本第 1 项）、**打出面缺失**
  （决斗 `kernel/duel.rs:70` 只取实体【杀】，见武圣样本 §7 与待实施修复清单第 7 项）。

→ 因此龙胆与武圣**同族**：**杀方向响应转化**是共同缺口，修法可一次覆盖两者（按 `respond_shan` 模式）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| 响应选项实测 | 见 [response-options.txt](response-options.txt) | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] `position: "hs"` 的**装备区**转化路径未单独验证
- [ ] 守约（`longdan_draw` / 武圣 `shouyue`）口径与实现
- [ ] 样本 25+：其余来源（当前 **21/37 来源**；标准面技能 6 个样本）
