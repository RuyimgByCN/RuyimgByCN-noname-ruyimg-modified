# NC08 第⑦层样本 9：`xicai`（戏才，U05 运筹帷幄 / 曹操·DIY 面）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 U05 运筹帷幄武将面样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**1 项源码事实固定通过（登记差异）**；**行为面（取回伤害牌）本轮未验收，如实登记**

## 1 上游源码（归档快照，逐字引用；见 [upstream-xicai.js.txt](upstream-xicai.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`：

```js
xicai: {
    preHidden: true,
    trigger: { player: "damageEnd" },
    filter(event, player) {
        return get.itemtype(event.cards) === "cards"
            && get.position(event.cards[0], true) === "o";   // 牌在**处理区**
    },
    async content(event, trigger, player) {
        await player.gain(trigger.cards, "gain2");
    },
},
```

## 2 验收结果

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2 | `position === "o"`（处理区） | 原生不校验位置，改为 **table 优先、弃牌堆兜底** → **差异登记**（源码事实固定断言） | **通过（差异登记）** |
| 1+3 | `damageEnd` + `gain(trigger.cards)` | 受到带牌伤害后取回伤害牌 | **未验收（登记）** |

**行为面为何未验收（如实）**：夹具（`debug_deal_damage_with_card` + 触发确认）未能稳定驱动该
**可选触发**（未观察到 `trigger_confirm`、手牌未增长）；已由**源码核对**记录实现位置
（`filtering/common.rs:521` 过滤臂、`step/yunchou.rs:413` 效果：table 优先/弃堆兜底 → 入手 +
`CardsGiven` 日志），但**不写成通过**；用例以 `#[ignore]` 守护，修复夹具后移除即可。

## 3 包结构发现（写入证据，供后续样本复用）

运筹帷幄的**武将面在 `assets/character.json`**（10 名武将、技能名带 `ycn` 前缀，如 `ycnxicai`），
而 `assets/cards.json` **只有卡牌段**（无 generals）；装载须 **`bridge_pack` + `bridge_generals`
双产物**（后者取 `.generals` 赋回 pack）。本样本是第⑦层首次覆盖该桥型。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **1 passed / 0 failed / 1 ignored** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1188 passed / 0 failed / 6 ignored** | — |
| 行数门禁 | 8287 文件 ≤ 2000 行 | — |

## 5 未完成 / 待办

- [ ] 戏才**行为面**夹具修复后补验收（可选触发驱动）
- [ ] 差异（处理区位置校验）是否对齐：需判定「已入弃牌堆的伤害牌是否仍可取回」
- [ ] 样本 10+：其余来源（当前 **8/37 来源**：L20 / U07 / 标准面 / L19 / L06 / swd / L10 / U05）

## 6 行为面**已补验收通过**（2026-10-19 同轮）

按「真实路径」重写后通过（不再需要 `#[ignore]`）：

```
advance_to_play(0) → 0 号对 1 号（**戏才持有者**）出【杀】→ 触发确认门（可选技，已确认）
→ 断言：1 号手牌 +1 且**包含该【杀】**；该牌**不在弃牌堆**
```

`upstream_xicai_gains_damage_card` → **通过**（2 passed / 0 failed / 0 ignored）。

**此前失败的原因（已查明，写入证据供后续样本复用）**：夹具用的
`debug_deal_damage_with_card` 只把 `card` 写入 `DamageData.card`，**不把牌放入处理区**
（原生 `table`）；而戏才效果按「**table 优先、弃堆兜底**」取牌 → 取不到 → 手牌不变。
→ 结论：凡「伤害牌/使用牌」类效果，夹具应走**真实出牌路径**（牌会自然进入处理区），
或先用可把牌置入处理区的手段布阵。

**登记（助手限制）**：`debug_deal_damage_with_card` 的语义是「带牌伤害事件（牌**不入**处理区）」，
与 `debug_recover`（只发事件不施加体力）同属「事件面助手」——两者都易被误当作完整效果路径，
建议后续统一在文档/命名上标注（测试侧改动，待 host/kernel 拆分落定后一并处理）。

## 7 验证结果（更新）

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed / 0 ignored** | [targeted-2.log](targeted-2.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
