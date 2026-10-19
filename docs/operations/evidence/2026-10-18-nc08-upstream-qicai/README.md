# NC08 第⑦层样本 33：`qicai`（奇才，**标准面** / 黄月英）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 2 项差异登记；**附带发现一处更基础的规则更宽松**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-qicai.js.txt](upstream-qicai.js.txt)）

```js
qicai: {
    mod: {
        targetInRange(card, player, target, now) {
            if (["trick", "delay"].includes(get.type(card))) return true;   // 任意锦囊/延时锦囊无视距离
        },
    },
},
```

## 2 验收结果（`nc08_upstream_qicai.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `targetInRange` 豁免 | 距离 1 的顺手牵羊可用（基线） | 通过 |
| 1 | 同上 | **持奇才**：距离 >1 的顺手牵羊**仍可用**（[distance-diagnostic.txt](distance-diagnostic.txt)） | 通过 |
| 1 | 技能门槛 | **未持奇才** → 距离 >1 被拒 | 通过 |
| 1+2 | 覆盖面 / 实现位置 | **差异①②**：原生仅在**顺手牵羊**分支读取奇才（`qicai` 全文件仅 2 处）；实现方式为**校验分支豁免**而非 `mod` 范围层 | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:698`（`qicai` 判定）、`:737`（`!qicai && … &&
computed_distance > 1 → IllegalTargets`，位于 `CardKind::Snatch` 分支内）。

## 3 **附带新发现**：兵粮寸断**没有距离限制**（比上游更宽松）

【兵粮寸断】在原生目录中为 `CardKind::Bing`（`catalog.rs:939`）——牌种**已实现**；
但其目标校验分支（`play_card.rs:764–769`）**只查目标数 / 非自己 / 存活**，**完全没有距离检查**
→ 即**对所有人**（不只奇才持有者）都可对任意距离使用 → 上游要求**距离 1**
→ **规则更宽松**（比「奇才覆盖面」更基础）→ **登记**（修法与验收见待实施修复清单）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **兵粮寸断距离限制**（对所有人）：补 `computed_distance > 1 → IllegalTargets`（奇才/神恩等豁免位同步）
- [ ] 差异①（奇才覆盖面）在补齐兵粮距离后**自动收敛**（同一豁免位）
- [ ] 样本 34+：其余来源（当前 **21/37 来源**；标准面技能 **15 个样本**）
