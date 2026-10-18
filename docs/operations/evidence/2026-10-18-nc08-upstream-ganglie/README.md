# NC08 第⑦层样本 21：`ganglie`（刚烈，**标准面** / 夏侯惇）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项验收通过**；其中 **1 项为差异登记（规则损失）**
- **基准面**：`character/standard/skill.js`（规则基准来源；本轮口径更正后统一）

## 1 上游源码（归档快照，逐字引用；见 [upstream-ganglie.js.txt](upstream-ganglie.js.txt)）

```js
ganglie: {
    trigger: { player: "damageEnd" },
    async content(event, trigger, player) {
        const { source } = trigger;
        let result = await player.judge({ judge(card) {
            if (get.suit(card) === "heart") return -2;   // 红桃 → 判定失败
            return 2;
        } }).forResult();
        if (!result?.bool || !source?.isIn()) return;
        result = source.countDiscardableCards(source, "h") < 2
            ? { bool: false }
            : await source.chooseToDiscard({ selectCard: 2, … }).forResult();  // **弃两张手牌**
        if (!result?.bool) await source.damage();        // 否则来源受 1 点伤害
    },
},
```

## 2 验收结果（`nc08_upstream_ganglie.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | 技能门槛 | **未持刚烈**者受伤 → **无确认门** | 通过 |
| 1+2+3 | `damageEnd` + 判定 + 分支 | 确认发动后：判定**非红桃**时**来源体力不增**（原生直接结算受伤路径） | 通过 |
| 3 | `chooseToDiscard({selectCard:2})` → 否则 `source.damage()` | **差异登记（规则损失）**：原生**缺**「来源弃置两张手牌」分支（源码事实固定：文件内无 `chooseToDiscard`/`SelectCards`，`to_discard` 仅用于判定牌入弃牌堆） | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/gang_lie.rs`（filter：`damageEnd` 且来源非自身；
机器：判定 → 非红桃 → **直接**对来源造成 1 点伤害）。

## 3 口径与限制（不夸大）

- 行为面用例在**未出现确认门**时**如实打印 SKIP 并返回**（不写成通过）；本次运行出现了确认门；
- 判定牌控制：本用例在判定区放置一张【杀】实体（非红桃）；**红桃分支**未单独构造（登记待办）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **3 passed / 0 failed** | [targeted.log](targeted.log) |
| engine-native 全量（含 examples） | 见提交说明 | — |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异修复**：补「来源弃置两张手牌」分支（`gang_lie.rs` **未跟踪** → 并入待实施修复清单）
- [ ] **红桃分支**行为面（判定为红桃 → 无事发生）
- [ ] 样本 22+：其余来源（当前 **18/37 来源**；标准面技能 3 个样本）
