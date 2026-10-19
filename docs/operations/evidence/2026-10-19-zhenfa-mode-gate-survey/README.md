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
