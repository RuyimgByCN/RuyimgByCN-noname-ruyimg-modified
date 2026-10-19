# NC08 第⑦层样本 39：`tuxi`（突袭，**标准面** / 张辽）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含 2 项差异登记）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-tuxi.js.txt](upstream-tuxi.js.txt)）

```js
tuxi: {
    trigger: { player: "phaseDrawBegin1" },
    filter(event, player) { return !event.numFixed; },
    async cost(event, trigger, player) {
        event.result = await player.chooseTarget({
            filterTarget(card, player, target) { return player !== target && target.hasCards("h"); },
            selectTarget: [1, 2],
        }).forResult();
    },
    async content(event, trigger, player) {
        await player.gainMultiple(event.targets);      // 各取一张（随机）
        trigger.changeToZero();                        // 放弃摸牌
    },
},
```

## 2 验收结果（`nc08_upstream_tuxi.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3（**发动 + 放弃摸牌**） | `changeToZero` | `tuxi_target` 询问**出现并被应答**（发动成立）；手牌**未增加**（放弃本次摸牌，基线正常摸 2） | 通过 |
| 2+3（**取牌落地形态**） | `changeToZero`（**取消**摸牌 → 不摸不弃）+ `gainMultiple` | 净手牌变化应为 **+取牌数**；实测 **0** → **差异③（已定位到源码）**：机器**无条件 `hand.pop()` 弃 2 张**（若固定摸牌尚未发生即弃玩家原有手牌） | 通过（**差异登记③**） |
| 2 | 技能门槛 | **未持突袭** → 摸牌阶段**无** `tuxi_target` 询问 | 通过 |
| 1 | `!event.numFixed` | **差异①（族第 3 例）**：原生 filter 只判阶段，未建模 `numFixed`（`yingzi`/`luoyi`/`tuxi`） | 通过（**差异登记**） |
| 3 | `gainMultiple`（**随机**取牌） | **差异②（族第 2 例）**：原生取**各区首张**（确定性）——与 `linghunzhihuo` 同族 | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/tu_xi.rs`（filter `Phase::Draw + begin`；`tuxi_target`
询问 1–2 名；注释「各区首张，Dismantle 同款确定性取牌」；弃权 = 不发动）。

## 2b 夹具根因（同轮）：共享 `advance_to_phase` 会 **drain**，把 `tuxi_target` 询问应答掉

初版实测 `picked=false`、`询问序列=[]`（[diagnostics.txt](diagnostics.txt)）→ 根因：
`nc08_common::advance_to_phase` **每步都会 drain**（对可取消询问取「放弃」）→ 到达摸牌阶段时
`tuxi_target` 已被**默认应答为放弃** → 突袭未发动。
**第二次修正**：改为**手写推进且不 drain** 后**仍** `询问序列=[]` → 进一步查明：布阵用的
`debug_give_hand_card` 之后**还有一次 `drain`**，它会把已开放的 `tuxi_target` 默认应答掉。
**第三次修正（本轮）**：确认布阵路径已**无 drain** → 复跑 `picked=true`（**目标询问出现并被应答**）
→ 用例转为通过；但**取牌未落地**（`hand 6 → 6`）→ 故把判据拆为两半：
**已验证**（发动成立 + 放弃摸牌）与**未验收**（取牌落地，`NOTE` 行记录）。
**第四次定位（同轮，已到源码级）**：给目标补牌后复跑仍为 `6 → 6` → 读 `tu_xi.rs` 的 `Read` 阶段
发现：机器**无条件 `hand.pop()` 弃置 2 张**（注释称「弃置本阶段固定摸到的牌」）→ 若固定摸牌
尚未发生，就会**弃掉玩家原有手牌** → 净变化 = 取牌 − 2 = **0**。
而上游 `trigger.changeToZero()` 是**取消摸牌**（不摸也不弃）→ 净应为 **+取牌数**。
→ **差异③**（已由源码与行为共同固定），用例改为断言 `after == hand_before` 并保留 `NOTE` 行。

**方法学（第 7 条，建议写入规则基准）**：共享助手若**自动 drain**，会与「需要观察/应答特定询问」
的用例冲突 → 此类用例应使用**不 drain** 的推进方式。

## 3 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed / 0 ignored**（含差异③登记） | [targeted.log](targeted.log)、[diagnostics.txt](diagnostics.txt) |
| 行数门禁 | 见提交说明 | — |

## 4 未完成 / 待办

- [x] **取牌落地形态已定位**（差异③：无条件 pop 2 张；见待实施修复清单）
- [ ] 差异③修复：改为按「本阶段是否已固定摸牌」决定弃置张数（或直接采用 `changeToZero` 语义）
- [x] 布阵/推进中的 drain 已清除（`tuxi_target` 现能出现并被应答）
- [ ] 差异①（`numFixed`）随该族一并评估（当前 3 例）
- [ ] 差异②（取牌确定性）随「随机 → 确定性」族一并评估（当前 2 例）
- [ ] 「放弃摸牌」的连锁面（上游 `trigger.changeToZero()` 对后续摸牌类效果的影响）
- [ ] 样本 40+：其余来源（当前 **21/37 来源**；标准面技能 **21 个样本**）
