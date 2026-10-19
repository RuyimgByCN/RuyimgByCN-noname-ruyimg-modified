# NC08 第⑦层样本 27：`biyue`（闭月）/ `yingzi`（英姿），**标准面**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 1 项差异登记）；**首个使用共享夹具助手 `nc08_common` 的新样本**

## 1 上游源码（归档快照，逐字引用；见 [upstream-biyue-yingzi.js.txt](upstream-biyue-yingzi.js.txt)）

```js
biyue: { trigger: { player: "phaseJieshuBegin" }, frequent: true,
         async content(event, trigger, player) { player.draw(); } },          // 结束阶段开始摸 1
yingzi: { trigger: { player: "phaseDrawBegin2" }, frequent: true,
          filter(event, player) { return !event.numFixed; },                    // 摸牌数未被固定
          async content(event, trigger, player) { trigger.num++; } },           // 摸牌数 +1
```

## 2 验收结果（`nc08_upstream_biyue_yingzi.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `phaseJieshuBegin` + `draw()` | 确认发动后结束阶段开始 → 手牌 **+1** | 通过 |
| 2 | `phaseDrawBegin2` + `trigger.num++` | 同种子对照：**持英姿比基线多摸 1**（[baseline-compare.txt](baseline-compare.txt)） | 通过 |
| 2 | `!event.numFixed` | **登记差异**：原生 filter **只判阶段**，未建模 `numFixed` 分支（源码事实固定） | 通过（**差异登记**） |
| 1 | 技能门槛 | **未持闭月** → 结束阶段不额外摸牌 | 通过 |

**原生实现引用**：`content/standard/skills/bi_yue.rs`（`Phase::End` + `begin` → `draw_cards(1)`）、
`content/standard/skills/ying_zi.rs`（`Phase::Draw` + `begin` → 摸牌数 +1）。

## 3 夹具说明（本轮起改用共享助手）

本样本 `mod nc08_common;` 使用共享助手（`advance_to_phase`/`drain`/`next_dedup`），
其中 `drain` 对 `trigger_confirm` **一律确认**（修罗/自愈样本的教训）——本样本的闭月确认门即由此处理。
**未**使用花色断言，故不涉及「按 id 布阵」问题。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] 差异（`numFixed`）口径判定与修复
- [ ] 样本 28+：其余来源（当前 **21/37 来源**；标准面技能 **9 个样本**）
