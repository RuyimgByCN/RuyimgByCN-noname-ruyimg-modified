# NC08 第⑦层样本 35：`lijian`（离间，**标准面** / 貂蝉）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 1 项差异登记，属**牌区族第 4 例**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-lijian.js.txt](upstream-lijian.js.txt)）

```js
lijian: {
    enable: "phaseUse",
    usable: 1,
    filter(event, player) { return game.countPlayer(c => c !== player && c.hasSex("male")) > 1; },
    filterCard: true,
    position: "he",                                  // 手牌 + 装备区
    filterTarget(card, player, target) {
        if (player === target) return false;
        if (!target.hasSex("male")) return false;    // 目标须为男性
        if (ui.selected.targets.length === 1) return target.canUse({ name: "juedou" }, ui.selected.targets[0]);
        return true;
    },
    selectTarget: 2, multitarget: true,
    async content(event, trigger, player) { … },     // 后选者向先选者使用【决斗】（无无懈）
},
```

## 2 验收结果（`nc08_upstream_lijian.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+4 | `hasSex("male")` ×2 + 决斗 | 两名男性被选中 → **决斗发生**（一方体力下降，[duel-diagnostic.txt](duel-diagnostic.txt)） | 通过 |
| 2 | `!target.hasSex("male") → false` | **女性**角色**不出现在候选中** | 通过 |
| 3 | `position: "he"` | **差异登记**：原生 `activate` 要求**手牌非空**（上游 `he` 含**装备区**）——与 `zhiheng`/`qixi`/`guose` **同族**（清单第 11 项） | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/li_jian.rs`（`activate` 手牌非空；目标过滤 `seat_is_male`；
机器 `DiscardAsk` → `Launch` 内嵌决斗直结算）。

## 2b 观察（**登记，未判定**）：目标选择询问未出现

行为面运行中 `picked=[]`（夹具的「选两名男性」分支**未被触发**），而离间**仍完成了结算**
（`hp1=3 hp2=3`，双方各降 1）。两种可能（**未判定，不写成结论**）：
1. 原生机器的目标选择**不是**「候选 = 男性玩家列表」（选项形态/时机不同，夹具分支未命中）；
2. 原生**自动选择**目标（上游为玩家 `selectTarget: 2` 手动选择）→ 若成立则为**差异**。

→ 需按固定种子打印该步骤的 `ChoiceSpec`（prompt/kind/options）来区分；已登记待办。

## 3 夹具经验（写入证据）

- **性别**由**装配武将**推导（`seat_is_male` → `general_slots` → `general_gender`），
  **没有** `debug_set_gender`；且**非国战局无将槽时门控默认 true**（`slots_is_male` 的空槽分支）
  → 夹具须用 `debug_assemble_general` 装配**男/女**武将才能构造性别差异。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 牌区族（第 11 项）统一修复后，本样本的差异项同步收敛
- [ ] **目标选择询问未出现**的两种可能区分（打印该步 ChoiceSpec 的 prompt/kind/options）
- [ ] 「后选者向先选者决斗」的**方向性**（本样本只验证决斗发生与体力变化）
- [ ] 样本 36+：其余来源（当前 **21/37 来源**；标准面技能 **17 个样本**）
