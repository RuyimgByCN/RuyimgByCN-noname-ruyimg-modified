# NC08 第⑦层样本 77：**来源 U05（杀海拾遗·运筹）** 武将技 `luweiyan`（厉焰）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）第⑦层
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**3 项通过**（含行为面正反例）+ **1 项差异登记**（**牌区族第 7 例**）

## 1 上游源码（归档快照，逐字引用）

`archive/upstream/noname/apps/core/extension/杀海拾遗/character/yunchou/skill.js`（`luweiyan`）：

```js
luweiyan: {
    enable: "phaseUse", usable: 1,
    filterCard(card) { return get.type(card) !== "basic"; },   // 非基本牌
    position: "hse",                                  // 手牌 + 装备区 + 判定区
    filter(event, player) { return player.hasCard(c => get.type(c) !== "basic", "hes"); },
    viewAs: { name: "shuigong" },                     // 当【水攻】使用
    prompt: "将一张非基本牌当【水攻】使用",
    group: "luweiyan2",                               // 链杀子技
},
```

## 2 验收结果（`nc08_upstream_yunchou_luweiyan.rs`，3 项）

| # | 上游依据 | 判据 | 结果 |
|---|---|---|---|
| 1 | `filterCard: 非基本` + `viewAs: shuigong` | **行为面（正例）**：0 号持【过河拆桥】（非基本）→ 以【水攻】形态对**有装备的 1 号**使用 → **Accepted** | 通过 |
| — | 水攻目标门槛（原生 `play_card.rs:1253`：目标须有装备） | **行为面（反例）**：1 号**无装备** → **`Rejected(IllegalTargets)`** | 通过 |
| 2 | `position: "hse"` | **差异登记（牌区族第 7 例）**：原生转化在**出牌命令面**（牌须为**手牌**）→ **装备区/判定区的非基本牌不能当【水攻】** | 通过（**差异登记**） |

**原生实现引用**：`engine/play_card.rs:189`（持 `ycnluweiyan` 且非基本 → `CardKind::Shuigong`）；
`play_card.rs:1253`（水攻目标须有装备）；`content/host/factory.rs:420`（`YunchouLuweiyan2` 链杀机器）。

## 3 未完成 / 待办

- [ ] **链杀**（`luweiyan2`）行为面：**未验收**（实测虚拟杀未产生，`chain-kill-diagnostic.txt`）——
      已排除「targets 为空」与「use_as 未声明」两假设；剩余候选=触发时机（原生 `CardUsed+Normal`
      ↔ 上游 `useCardAfter`）与嵌套结算；详见修复清单第 31 项
- [ ] **牌区族第 7 例**随族统一修（清单第 11 项）
- [ ] `check`/`ai` 策略面；水攻自身的伤害结算细节
- [ ] 运筹其余武将技（纵火/烧营/归汉/精械/巧思/奸雄变体/蛊惑变体/悲歌变体）
