# 受理面普查（小包，2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 动机：样本 126（火山）「**完全不可用**」比「效果为空」更严重，且**只测效果的样本会整类漏掉**
- 方法：对 4 个小包（hearth 12 / huanle 6 / yunchou 11 / zhenfa 6 = **55 张**）逐卡构造宽松局面
  （出牌阶段、四人各 1 张手牌、1 号 1 件装备），用**两种目标形态**（`vec![]`、`vec![0]`）尝试发动
- **判定口径**：两种形态**均被拒** = **候选**（**被拒 ≠ 缺陷**：可能是阶段/条件不合，如
  `zhiliaobo` 需受伤目标、`zhaomingdan` 需目标 `hej` 有牌）⇒ 须**逐卡核对上游 `filterTarget`/`enable`**

## 结果：共试 55 张（含重复副本），**两种形态均被拒 16 张（去重后；原始记录 29 条）**

| 包 | 卡 |
|---|---|
| `shahai-hearth-upstream-noname` | `zhaomingdan` |
| `shahai-hearth-upstream-noname` | `shandianjian` |
| `shahai-hearth-upstream-noname` | `zhiliaobo` |
| `huanle-upstream-noname` | `shoulijian` |
| `shahai-yunchou-upstream-noname` | `huoshan` |
| `shahai-yunchou-upstream-noname` | `guohe` |
| `shahai-yunchou-upstream-noname` | `fudichouxin` |
| `shahai-yunchou-upstream-noname` | `chenhuodajie` |
| `shahai-yunchou-upstream-noname` | `toulianghuanzhu` |
| `shahai-yunchou-upstream-noname` | `geanguanhuo` |
| `shahai-yunchou-upstream-noname` | `caochuanjiejian` |
| `shahai-zhenfa-upstream-noname` | `swd_tianfuzhen1` |
| `shahai-zhenfa-upstream-noname` | `swd_qixingzhen1` |
| `shahai-zhenfa-upstream-noname` | `swd_niaoxiangzhen1` |
| `shahai-zhenfa-upstream-noname` | `swd_changshezhen1` |
| `shahai-zhenfa-upstream-noname` | `swd_changshezhen2` |

## 已知交叉印证

- **`huoshan`（火山）**：本普查**再次命中**（与样本 126 / 修复清单第 47 项一致）；
- **`shandianjian`（闪电箭）**：本普查命中；但样本 125 显示它在**其它局面**下会被**通用杀路径接管**
  （受理）⇒ 说明「被拒」与局面强相关，**不能单凭本普查定性**。

## 边界与后续

1. 本普查**只判「是否至少一种形态被受理」**，不判语义等价；
2. 被拒项**须逐卡**对照上游条件后分类（合理拒绝 / 实现缺口）；
3. 目录名匹配用 `contains`（**若用相等匹配会静默跳过带前缀的卡**——首轮即因此漏掉全部 yunchou 卡）；
4. 建议后续：把「受伤目标 / `hej` 有牌 / 判定区可加」等条件纳入更宽松的构造，以降低误报。

## 候选逐卡定性（**已闭合**，2026-10-19）

修正后的 7 项候选（三形态均被拒）经**完整读取上游 `enable`/`filterTarget`/`filterAddedTarget`** 后：

| 卡 | 上游条件（读全后） | 定性 |
|---|---|---|
| `zhiliaobo`（治疗波） | `filterTarget: target.hp < target.maxHp`（**须受伤目标**） | **合理**（普查局面全员满血；样本 115 已验证该卡行为面） |
| `geanguanhuo`（隔岸观火） | `filterTarget` 要求目标手牌非空且**存在可拼点者**；**`filterAddedTarget` 要求第二目标**（**双目标**）；`enable: countPlayer() > 2` | **合理**（普查只试了 0/1 个目标） |
| `swd_changshezhen1/2`（长蛇阵） | `enable: player.inline()`（**队列**关系） | **合理** |
| `swd_qixingzhen1`（七星阵） | `enable: player.siege() \|\| player.sieged()`（**围攻/被围攻**） | **合理** |
| `swd_tianfuzhen1`（天覆阵） | `enable`: 存在 `isMajor()`；`filterTarget: target.isMajor() && he > 0`（**大势力**） | **合理** |
| `swd_niaoxiangzhen1`（鸟翔阵） | `filterTarget` **读全后**要求「**目标身份与其相邻座位相同**」（国战同势力相邻） | **合理** |

⇒ **7/7 全部为合理拒绝** ✓ —— 本普查**未发现新的「完全不可用」卡**；
唯一确认为真的门槛问题是 **`dizaizhen`（地载阵）的模式门槛缺失**（修复清单第 49 项，已单独登记）。

**普查结论**：小包（55 张）中**未发现「发动即被拒」的实现缺口**；
**「完全不可用」类问题在本批不成立**（火山一案亦经三形态核实为「**目标形态与上游相反**」而非不可用，第 47 项）。
