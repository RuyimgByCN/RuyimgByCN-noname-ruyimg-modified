# **「待迁移技能量」收窄到 5**（2026-10-19）

## 1 方法（可复现）

在分类脚本的**规则疑似**集合上，加两层过滤：

* **正**：块内含**效果原语调用** —— 正则 `\.(draw|damage|recover|loseHp|discardCards|gain|equip|judge)\s*\(`；
* **负**：排除 UI/标记/动画/聊天/配置 —— `linexy`、`player.line(`、`game.line`、`audio`、`as_`、`ui.`、
  `document.`、`mark`、`storage.`、`lib.config`、`game.log`、`setTimeout`、`animation`、`style`、`frame`。

原始输出：[filter-run.txt](filter-run.txt)。

## 2 结果

| 来源 | 疑似规则 | **调用效果原语** | 结论 |
|---|---:|---:|---|
| **L08 十周年UI** | 14 | **0** | 全是 UI/标记 |
| **L11 太虚幻境** | 104 | **5** | **唯一有真规则的来源** |
| **L12 如真似幻** | 0 | 0 | 无 `lib.skill` |
| **L25 祖安武将** | 63 | **0** | 见 §3 |
| **合计** | **181** | **5** | — |

### 2.1 这 5 个（**全部在 L11**）

| 技能 | 大小 | 语义（读块） |
|---|---:|---|
| `dz_gs_yinfu` | 300 B | `phaseBegin`：若 **已损失体力 ≥ 当前轮数** ⇒ `player.recover()`；累计 **3 次**后 `removeSkill` |
| `boss_danshu` | 368 B | `loseEnd`：`player.judge()` 为**红** ⇒ `recover()` |
| `boss_xixing` | 396 B | `phaseZhunbeiBegin`：`chooseTarget` 链上角色 ⇒ **雷属性伤害** |
| `polu_damage` | 440 B | `damageEnd`：`draw()` + 从牌堆找 `equip1` |
| `boss_tianyun` | 728 B | `phaseEnd`：`chooseTarget` 敌人 ⇒ 伤害；`forceDie` |

## 3 L25 的 63 个「疑似规则」是什么（**抽样 4 个**）

```
_useskill   { usable:3, trigger:{player:['useCard']}, content:function(){ game.zuanExp(player, 2); } }
_killskill  { trigger:{source:['dieBegin']},          content:function(){ game.zuanExp(player, 25); } }
_exp2skill  { trigger:{player:['recoverEnd']},        content:function(){ … game.zuanExp(player, add); } }
_ssrnohuahensskill { trigger:{player:['huashen2Before']}, content:function(){ trigger.cancel(); } }
```

⇒ **`game.zuanExp(...)` 是「祖安武将」**自建的**元游戏经验系统**（成长/成就）**，不是牌局规则；
其余是 `trigger.cancel()` 这类**钩子开关**。**均不属引擎迁移范围**。

## 4 判读（**如实**）

1. **「待迁移技能量」的可靠上界 = 5**（此前写 179 → 181 → 现在 **5**）；
2. 这 5 个**全在 L11**，且都是 `content:` 型（**无 `mod:`**）⇒ **理论上是 DSL/宿主 effect 层的候选**；
3. **但 DSL 覆盖度仍未验证** ❗ —— 例如 `dz_gs_yinfu` 需要「**使用次数计数 + 移除技能**」，
   `boss_xixing` 需要「**选择目标 + 属性伤害**」，DSL 是否有对应原语**未验证**；
4. **本轮不宣称任何技能已迁移**。

## 5 边界

- 正则**可能漏判**：若某技能通过**间接调用**（如 `someFunc()` 内部再 damage）产生效果，本过滤会**漏掉**；
- **未读** L11 的 99 个非效果块（只读了 5 个效果块 + 抽样 4 个 L25 块）；
- `boss_*` 前缀的技能可能是 **Boss 模式专用**，是否属 37 来源的「规则面」**需另行判定**。
