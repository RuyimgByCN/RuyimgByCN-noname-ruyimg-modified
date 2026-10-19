# NC08 第⑦层样本 61：`qingguo`（倾国，**标准面** / 甄姬）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过（含行为面）** + **1 项规则损失**（**牌区族第 6 例**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-qingguo.js.txt](upstream-qingguo.js.txt)）

```js
qingguo: {
    viewAs: { name: "shan" },
    viewAsFilter(player) {
        if (!player.hasCards("hs", { color: "black" })) return false;   // 黑色牌（hs）
    },
    position: "hs",                                  // 手牌 + 装备区
    prompt: "将一张黑色手牌当闪使用或打出",
},
```

## 2 验收结果（`nc08_upstream_qingguo.rs`，2 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1+3 | `viewAs: shan` + 与龙胆并列 | 原生【闪】响应候选含 **`qingguo && black`** 与 **`longdan && kind.is_sha_like()`**（源码事实） | 通过 |
| 2 | `position: "hs"` | **差异（牌区族第 6 例，规则损失）**：原生候选**只遍历 `.hand`**，**未含 `.equips`** → **装备区的黑色牌不能当【闪】** | 通过（**差异登记**） |
| 1 | **行为面（手牌）** | 黑色**手牌**（非【闪】）出现在 `respond_shan` 候选中 → 打出后【杀】被抵消（**hp 4 → 4**） | 通过 |
| 2 | **行为面（装备区）** | 1 号装备区放黑色武器 → `respond_shan` 出现，但**装备区黑色牌不在候选中**（**差异的行为面确认**） | 通过 |

**原生实现引用**：`kernel/sha/step.rs:374`/`:789`/`:1195`（倾国并入【闪】响应选项；
`:1192` 注释「倾国（甄姬，**v202 对齐上游 `position: "hs"`**）：黑色手牌可当闪打出」）。

**同族对照**：出牌命令面亦限定手牌（`play_card.rs` 的 `hand.contains(&card)`）→ 与本项同源。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log)、[behavior-diagnostic.txt](behavior-diagnostic.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [x] **行为面已验收**（黑色手牌当闪成功抵消；装备区黑色牌不入候选——差异的行为面确认）
- [ ] **牌区族第 6 例**随族统一修（清单第 11 项：放宽牌区校验或新增命令）
- [ ] `ai.order`/`respondShan` 策略面
- [ ] 样本 62+：其余来源/技能（标准面技能 **39 个样本**；来源覆盖 **23/37**）
