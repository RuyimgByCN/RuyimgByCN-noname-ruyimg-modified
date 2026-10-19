# NC08 第⑦层样本 26：`qixi`（奇袭，**标准面** / 甘宁）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 105
- 结论：**3 项通过**（含 1 项差异登记）；**1 项未验收（如实登记，含诊断）**
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-qixi.js.txt](upstream-qixi.js.txt)）

```js
qixi: {
    enable: "chooseToUse",              // 仅「使用」（无 chooseToRespond）
    filterCard(card) { return get.color(card) === "black"; },
    position: "hes",
    viewAs: { name: "guohe" },          // 视为【过河拆桥】
    prompt: "将一张黑色牌当过河拆桥使用",
},
```

## 2 验收结果（`nc08_upstream_qixi.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `get.color === "black"`（反例） | **红色**牌不可当【过河拆桥】（被拒） | 通过 |
| 1 | 技能门槛 | **未持奇袭** → 不产生【过河拆桥】效果（目标手牌不被拆） | 通过 |
| 3 | `viewAs` 候选选择 | **登记差异①**：原生为**固定优先级**（杀类优先，`play_card.rs:156` 注释），上游由玩家选择 → 黑色【杀】持奇袭时只作【杀】 | 通过（**差异登记**） |
| 1 | 黑色牌 → 【过河拆桥】 | **未验收（登记）**：以黑色【无懈可击】为载体时 `Rejected(WrongPhase)`（转化未生效） | — |

**原生实现引用**：`engine/play_card.rs:143/157`（`qixi && black` → `kind = CardKind::Dismantle`，
位于杀类转化之后）。

## 3 未验收项：**同一逻辑布阵在两种子/推进方式下结果不同**（未判定，2026-10-19 同轮）

| 场景 | 不持奇袭 | 持奇袭 |
|---|---|---|
| 探针（seed 20265601，`to_play` **不**处理选择） | `Rejected(WrongPhase)` | **`Accepted`** |
| 用例（seed 20265501，`advance_to_play` **会** drain 选择） | `Rejected(WrongPhase)` | **`Rejected(WrongPhase)`** |

→ **状态依赖** → **不能**据此断言「转化生效」或「不生效」。

**进一步矩阵探针（同轮，2 种子 × 2 推进方式 × 2 授予时机 = 8 组）**：

```
seed=20265501 drain=false/true grant_before=true/false → 全部 Rejected(WrongPhase)
seed=20265601 drain=false/true grant_before=true/false → 全部 Accepted
```

→ **唯一决定因素是「种子」**（推进方式与授予时机**均无影响**）；机制**仍未查明**（已排除
「选择 drain」与「授予时机」两个假设）。此前两种猜测（**牌种限制** / **可玩性检查顺序**）**均未证实，已撤回**。
`#[ignore]` 保留对照；后续需按**固定种子**逐步定位（建议：打印该黑色无懈的 **card id/suit** 与
转化前后的 `kind`）。

## 4 夹具经验（写入证据）

- 【闪】**全为红色** → 无法作「黑色载体」（武圣样本已踩过，本样本再踩一次）；
- 判定转化应优先用**效果**（如「目标手牌被拆」）而非「是否被接受」——因为**接受**可能来自
  原牌种本身可玩（本样本初版即因此把「黑色无懈被接受」误读为转化生效）。

## 5 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 6 未完成 / 待办

- [ ] 未验收项的两种可能区分（黑色非锦囊载体 / 可玩性检查顺序探针）
- [ ] 差异①（`viewAs` 候选选择 vs 固定优先级）口径判定
- [ ] `position: "hes"` 的装备/判定区转化（与制衡同族）
- [ ] 样本 27+：其余来源（当前 **21/37 来源**；标准面技能 8 个样本）
