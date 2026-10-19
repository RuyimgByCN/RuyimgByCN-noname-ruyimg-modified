# NC08 第⑦层样本 36：`guicai`（鬼才，**标准面** / 司马懿）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记 + 1 项同类登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-guicai.js.txt](upstream-guicai.js.txt)）

```js
guicai: {
    trigger: { global: "judge" },                 // 任何角色的判定
    preHidden: true,
    filter(event, player) {
        return player.hasCards(get.mode() === "guozhan" ? "hes" : "hs");   // 非国战：手牌+装备
    },
    async cost(event, trigger, player) { event.result = await player.chooseCard({ … }).forResult(); },
    // position: get.mode() === "guozhan" ? "hes" : "hs"
},
```

## 2 验收结果（`nc08_upstream_guicai.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | `global: "judge"` + 任意牌改判 | **他人**（0 号）判定时，1 号（持鬼才）可打出**红色手牌**改判（[rewrite-diagnostic.txt](rewrite-diagnostic.txt)） | 通过 |
| 2 | `"hes" : "hs"` | **差异登记（牌区族第 5 例）**：原生**装备区**候选受 `is_black` 限制（鬼道口径）→ 鬼才持有者的**非黑装备牌**不能改判 | 通过（**差异登记**） |
| — | 基准面核对 | **同类登记**：`guidao`（鬼道）**不在** `character/standard/skill.js`（仅 `character/sb/`）→ **非标准面技能**，不建样本 | 通过（**同类登记**） |

**原生实现引用**：`kernel/judge.rs:395–415`（`gui_cai_holders_any` → 手牌**全量**入候选；
装备区仅在「天道/改天道持有者」或**黑色**牌时入候选）。

## 3 口径与限制（不夸大）

- 改判夹具按**已查明的判定区消费时机**构造：在 0 号 **Draw 阶段**放入【乐不思蜀】→
  **下一轮**判定阶段消费（规则基准「判定区的消费时机」）；
- `filterCard` 的 `checkMod(..., "unchanged", ...)`（牌可用性修饰门控）**未覆盖**（登记）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异（鬼才装备区不限花色）随牌区族（第 11 项）一并修
- [ ] `checkMod "unchanged"` 门控是否影响可改判牌集合
- [ ] 样本 37+：其余来源（当前 **21/37 来源**；标准面技能 **18 个样本**）
