# NC08 第⑦层样本 29：`qingnang`（青囊，**标准面** / 华佗）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**4 项通过**（含 1 项差异登记，属**规则损失**）
- **基准面**：`character/standard/skill.js`

## 1 上游源码（归档快照，逐字引用；见 [upstream-qingnang.js.txt](upstream-qingnang.js.txt)）

```js
qingnang: {
    enable: "phaseUse",
    filterCard: true,                 // 弃任意一张手牌
    usable: 1,                        // 出牌阶段限一次
    filterTarget(card, player, target) {
        if (target.hp >= target.maxHp) return false;   // 仅已受伤者
        return true;                                    // ← 未排除自己
    },
    async content(event, trigger, player) { await event.target.recover(); },
},
```

## 2 验收结果（`nc08_upstream_qingnang.rs`，4 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 2+3+4 | `filterCard` + 已受伤 + `recover()` | 弃 1 张手牌 → 已受伤的**其他**角色**回复 1 点** | 通过 |
| 1 | `usable: 1` | 同回合**第二次发动被拒** | 通过 |
| 3 | `filterTarget`（满血排除） | **满血**目标**不出现在候选中** | 通过 |
| 3 | `filterTarget` **未排除自己** | **差异登记（规则损失）**：原生 `activate` 要求 `i != owner` → **仅自己受伤时不可发动**（[self-target-diagnostic.txt](self-target-diagnostic.txt)） | 通过（**差异登记**） |

**原生实现引用**：`content/standard/skills/qing_nang.rs`（`activate`：手牌非空 + 存在
**`i as PlayerId != owner`** 的已受伤角色；`TargetAsk` → `Discard` 两阶段；回复 1 点）。

## 3 差异说明（不夸大）

上游 `filterTarget` **没有**「其他角色」限制 → **华佗可以青囊自己**（只要自己已受伤）；
原生 `activate` 明确排除自己（源码注释亦写作「其他角色」，**与上游不符**）→ 属**规则损失**，
修法：`activate` 的受伤者判定去掉 `i != owner`（`TargetAsk` 的候选生成需同步放开）。

## 4 验证结果

| 检查 | 结果 | 证据 |
|---|---|---|
| 定向 | **4 passed / 0 failed** | [targeted.log](targeted.log) |
| 行数门禁 | 见提交说明 | — |

## 5 未完成 / 待办

- [ ] **差异修复**：允许自我青囊（`qing_nang.rs` **未跟踪** → 并入待实施修复清单）
- [ ] `check`/`ai` 策略面
- [ ] 样本 30+：其余来源（当前 **21/37 来源**；标准面技能 **11 个样本**）
