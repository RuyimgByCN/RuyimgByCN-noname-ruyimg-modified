# 阵法卡模式门槛普查（2026-10-19）

- 工作包：**NC08**；规则版本 v322（工作树）/ 存档格式 v105
- 背景：上游 `杀海拾遗/card/zhenfa/card.js` 的 **10 张阵法卡全部**带 **`mode: ["guozhan"]`**；
  样本 127（地载阵）已证原生**未校验**该门槛（修复清单第 49 项）⇒ 本普查核对**普遍性**

## 上游 `mode` 清单（10 张，全部 `mode: ["guozhan"]`）

`pozhenjue`（破阵决）/ `changshezhen`（长蛇阵）/ `tianfuzhen`（天覆阵）/ `dizaizhen`（地载阵）/
`fengyangzhen`（风扬阵）/ `yunchuizhen`（云垂阵）/ `qixingzhen`（七星阵）/ `shepanzhen`（蛇蟠阵）/
`longfeizhen`（龙飞阵）/ `niaoxiangzhen`（鸟翔阵）

## 已交付情况（与转换缺口核对**互相印证**）

`shahai-zhenfa` 包内 5 张：`zf_dizaizhen`、`swd_tianfuzhen1`、`swd_qixingzhen1`、`swd_niaoxiangzhen1`、
`swd_changshezhen1`（+ `swd_changshezhen2` 副本）；
**其余 5 张**（`pozhenjue`/`fengyangzhen`/`yunchuizhen`/`shepanzhen`/`longfeizhen`）
正是第 117 行转换缺口清单中的**5 项 zhenfa 缺口** ⇒ **两份独立核对一致**。

## 普查结果（身份局 4 人，三形态 `[]`/`[0]`/`[1]`）

| 卡 | 结果 |
|---|---|
| **`zf_dizaizhen`（地载阵）** | **`[]` → 受理** ⇒ **模式门槛缺失（已确认）** |
| `swd_tianfuzhen1`（天覆阵） | 三形态均被拒 |
| `swd_qixingzhen1`（七星阵） | 三形态均被拒 |
| `swd_niaoxiangzhen1`（鸟翔阵） | 三形态均被拒 |
| `swd_changshezhen1`（长蛇阵） | 三形态均被拒 |

**结论**：**1/5 已确认门槛缺失**；其余 4 张**被拒**，但**原因未定**（可能是模式门槛**已生效**，
也可能是**其它条件未满足**——如 `tianfuzhen` 上游 `enable` 要求「**存在大势力**」，
而 4 人身份局无势力概念）⇒ **须逐卡核对上游条件**后才能定性。

## 边界与后续

1. 本普查只判「**是否受理**」；**被拒 ≠ 门槛已实现**（须逐卡核对 `enable`/`filterTarget`/`mode`）；
2. 建议后续：为 `tianfuzhen`/`qixingzhen`/`niaoxiangzhen`/`changshezhen` 各建**国战模式用例**
   （或补模式开关），以区分「门槛已实现」与「条件未满足」。

## 逐卡定性（2026-10-19 补，读上游 `enable`/`filterTarget`）

| 卡 | 上游 `enable` | 上游 `filterTarget` | 身份局被拒是否合理 |
|---|---|---|---|
| `tianfuzhen`（天覆阵） | `game.hasPlayer(current => current.isMajor())`（**存在大势力**） | `target.isMajor() && target.countCards("he") > 0` | **合理**：身份局**无势力/大势力**概念 ⇒ 条件不满足 |
| `qixingzhen`（七星阵） | `player.siege() \|\| player.sieged()`（**围攻/被围攻**） | `target === player` | **合理**：无围攻关系 |
| `changshezhen`（长蛇阵） | `player.inline()` 或 `identity === "unknown"` 分支 | （无 `filterTarget`） | **合理**：无 `inline`（队列）关系 |
| **`niaoxiangzhen`（鸟翔阵）** | **`: true`（恒真）** | 见下（**完整读全后**） | **合理**（**同日更正**，见 §更正） |

⇒ **3 张为合理拒绝**（条件依赖**势力/围攻/队列**，身份局天然不满足）；
**`niaoxiangzhen` 待专项核对**（其 `enable` 恒真、`filterTarget` 仅要求身份不同）。

## 更正：`niaoxiangzhen` 的 `filterTarget` 须**读全**（2026-10-19，同日）

上一版只截取了 `filterTarget` 的**前两行**，误判为「**仅要求身份不同**」⇒ 记为「待核」。
**完整读全**后（上游 `zhenfa/card.js`）：

```js
filterTarget(card, player, target) {
    if (player.identity === target.identity) return false;          // ① 不能与自己同身份
    if (target.identity === "unknown" || target.identity === "ye") return false;   // ② 目标身份须明确
    return target.identity === target.next.identity
        || target.identity === target.previous.identity;            // ③ **目标身份须与其相邻座位相同**
},
```

⇒ 真实条件是「**目标的身份与其相邻座位相同**」（国战**同势力相邻**语义）——
**身份局下通常不满足** ⇒ 三形态被拒**合理**；且原生**已实现**该卡
（`content/host/intrinsic.rs:535` → `HostBehavior::SwdNiaoxiangzhenCard`；`content/host/step/swd_zhenfa_cards.rs:194`；
`engine/play_card.rs:903` 目标校验）。

**方法教训**：核对 `filterTarget` **必须读完整函数体**；**截断读取会把「多条件」误判为「单条件」**，
进而把**合理拒绝**误记为**门槛缺失/待核**。
