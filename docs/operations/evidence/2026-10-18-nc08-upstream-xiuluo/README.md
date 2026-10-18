# NC08 第⑦层样本 7：`swd_xiuluo`（修罗，swd 转换包）（2026-10-18）

- 日期：2026-10-18
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层——**首个 swd 转换包样本**
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**2 条规则点验收通过（均为「差异登记 + 事实固定」）；效果面（三段询问机）本轮未验收，如实登记**

## 1 上游源码（归档快照，逐字引用；见 [upstream-xiuluo.js.txt](upstream-xiuluo.js.txt)）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/swd/skill.js`：

```js
swd_xiuluo: {
    trigger: { player: "phaseZhunbeiBegin" },
    filter(event, player) { return player.countCards("j") > 0; },
    direct: true,
    content() {
        var next = player.discardPlayerCard(player, 2, "hj",
            "是否一张手牌来弃置一张花色相同的判定牌？");
        next.filterButton = function (button) { /* 先手牌后判定牌、花色须相同 */ };
    },
},
```

## 2 验收结果（`nc08_upstream_xiuluo.rs`）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `trigger: phaseZhunbeiBegin` | 原生挂在**判定阶段起点**（`Phase::Judge, begin: true`）——**登记差异**，事实固定型断言 + 行为面确认时机可用 | **通过（差异登记）** |
| 2 | `filter: countCards("j") > 0` | 原生**额外要求手牌非空**（判定区有牌但手牌为空时上游可发动、原生不触发）——**登记差异**，含行为断言（无询问）+ 源码事实固定 | **通过（差异登记）** |
| 3+4 | `discardPlayerCard(player, 2, "hj", …)` | 三段询问机「手牌 → **同花色**判定牌 → 双方离区」 | **未验收（登记）** |

**效果面为何未验收（如实）**：测试夹具的 `advance_to_judge_of` 在**同一次提交内**既推进到判定
阶段又消费掉阶段开始事件产生的询问（其取消循环），未能稳定停在 `xiuluo_hand` 断点；需要改写为
「按阶段逐步驱动 + 遇 `xiuluo*` 询问即停」的夹具。已由**源码核对**记录实现位置与口径
（`content/host/step/skills_cards.rs:674-745`：手牌询问 → 同花色过滤 → 判定牌询问 → 双方离区，
花色过滤与上游 `filterButton` 同口径），但**不写成通过**。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **2 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | **1184 passed / 0 failed / 2 ignored**（1182 → +2） | [engine-native-tests.log](engine-native-tests.log) |
| 行数门禁 | 8280 文件 ≤ 2000 行 | [source-lines.log](source-lines.log) |

## 4 未完成 / 待办

- [ ] 修罗**效果面**夹具改写后补验收（三段询问机 + 花色匹配）
- [ ] 差异 1/2（准备阶段落点、手牌门槛）的最终口径待上游判定（与
      [Prepare 落点证据](../2026-10-18-nc08-prepare-phase/README.md) 同一族）
- [ ] 样本 8+：其余来源（当前 **6/37 来源**）

## 5 效果面**已补验收通过**（2026-10-19 同轮）

按「逐阶段驱动 + 确认可选触发 + 动态选牌」重写后通过（3 passed / 0 failed / 0 ignored）：

| 步骤 | 处理 |
|---|---|
| 驱动 | 逐步 `NextPhase`；**遇到 `trigger_confirm` 一律确认**（修罗是可选技——此前夹具把它**取消**了，这正是长期失败的首要根因）；遇 `xiuluo*` 询问即停 |
| 布阵 | 判定牌在**0 号本轮判定阶段结束（进入 Draw）后**补放 → 可存活到**下一轮**判定阶段起点（首轮判定阶段在授予技能前已开始，且判定牌会被本阶段消耗） |
| 选牌 | 从**当前手牌**中挑一张与判定区某张**同花色**者（布阵期的牌可能已被首轮结算消耗） |
| 断言 | 第一段 `xiuluo_hand` → 第二段 `xiuluo_judge` 且选项**恰为同花色判定牌** → 手牌 −1 → **所选判定牌已离开**判定区 |

**登记（口径）**：判定区**数量**受「判定阶段自身结算」影响（判定牌会被本阶段消耗），
故只断言「所选判定牌已离开、数量不增加」，不断言「恰好 −1」。

**教训（写入证据，供后续夹具复用）**：①可选技的 `trigger_confirm` **不得**在夹具里统一取消；
②判定/准备阶段类触发要注意「引擎初态已在该相位」与「判定牌会被本阶段消耗」两件事。
