# NC08 第⑦层样本 14：`tianfuzhen`（天覆阵，U05 zhenfa 卡牌子包）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 zhenfa（阵法）子包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项验收通过**（1 项为差异登记）

## 1 上游源码（归档快照，逐字引用；见 [upstream-tianfuzhen.js.txt](upstream-tianfuzhen.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/card/zhenfa/card.js`：

```js
tianfuzhen: {
    type: "zhenfa",
    recastable: true,
    enable() { return game.hasPlayer(current => current.isMajor()); },   // 存在大势力角色
    filterTarget(card, player, target) {
        return target.isMajor() && target.countCards("he") > 0;           // 大势力且有牌
    },
    selectTarget: -1,
    content() { target.chooseToDiscard("he", true).delay = false; },      // 强制弃 1 张
    mode: ["guozhan"],
    ai: { … },
},
```

## 2 验收结果（`nc08_upstream_tianfuzhen.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `filterTarget` + `selectTarget: -1` | 目标集合必须**恰为**「大势力且有手牌/装备者」：空表被拒、期望集合被接受 | 通过 |
| 3 | `chooseToDiscard("he", true)` | 每名目标一次询问、**不可取消**、恰 1 张（手牌或装备） | 通过 |
| 1 | `enable: hasPlayer(isMajor)` | 原生**额外要求存活 ≥4** → **差异登记**（源码事实固定 + 3 人局行为断言被拒） | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:903`（存活 ≥4 门 + 期望目标集合）、
`content/host/step/swd_card_effects.rs:1146`（逐目标强制弃牌询问，`allow_cancel: false`）。

## 3 工程发现（写入证据）

- **大势力需要真实装配武将**：本包只有卡牌、无武将 → 测试须用 `assign_general_named` 给两名座位
  装配**同势力**武将（本样本自动从 `general_candidates()` 中挑同势力成对者），否则
  `is_major_faction` 恒假、阵法卡无法使用（这与上游「国战 `isMajor`」前提一致）；
- **阵法卡走无懈窗口**：断言弃牌询问时须先跳过 `nullify_window` 等前置询问。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 样本 15+：其余来源（当前 **13/37 来源**）
- [ ] zhenfa 其余已转卡（长蛇/七星/鸟翔/地载）的上游样本（本样本为 1/5 已转）
- [ ] `mode: ["guozhan"]` 模式门与 `recastable`（重铸）面的验收
- [ ] 「存活 ≥4 vs 存在大势力」差异的最终口径判定
- [ ] 逐命令对照工具；`ai` 策略面不在规则对照范围
