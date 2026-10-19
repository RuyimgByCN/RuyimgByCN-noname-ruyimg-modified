# 全包受理面普查（2026-10-19）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 方法：对**全部已转换包**的每张已交付卡，用**三种目标形态**（`[]`/`[0]`/`[1]`）尝试发动，并**应答所有中途询问**
- 判据：**三种形态均被拒** = **候选**（**被拒 ≠ 缺陷**，须完整读取上游 `enable`/`filterTarget`/`filterAddedTarget` 后定性）
- 目录名匹配用 `contains`（相等匹配会静默漏卡）

## 结果

```
DBG 全包受理普查：共试 231 条；**三形态均被拒 31(去重)**；目录未找到 0(去重)
DBG   REJECTED shahai-gujian-upstream-noname / dinvxuanshuang
DBG   REJECTED shahai-gujian-upstream-noname / jinlianzhu
DBG   REJECTED shahai-gujian-upstream-noname / longxugou
DBG   REJECTED shahai-hearth-upstream-noname / zhiliaobo
DBG   REJECTED shahai-swd-upstream-noname / swd_caoyao1
DBG   REJECTED shahai-swd-upstream-noname / swd_caoyao2
DBG   REJECTED shahai-swd-upstream-noname / swd_caoyao3
DBG   REJECTED shahai-swd-upstream-noname / swd_guilingzhitao
DBG   REJECTED shahai-swd-upstream-noname / swd_hufu
DBG   REJECTED shahai-swd-upstream-noname / swd_hufu2
DBG   REJECTED shahai-swd-upstream-noname / swd_jiguanshu1
DBG   REJECTED shahai-swd-upstream-noname / swd_jiguanshu2
DBG   REJECTED shahai-swd-upstream-noname / swd_jiguantong1
DBG   REJECTED shahai-swd-upstream-noname / swd_jiguantong2
DBG   REJECTED shahai-swd-upstream-noname / swd_jiguanyaoshu1
DBG   REJECTED shahai-swd-upstream-noname / swd_jiguanyaoshu2
DBG   REJECTED shahai-swd-upstream-noname / swd_shenmiguo
DBG   REJECTED shahai-swd-upstream-noname / swd_shenmiguo2
DBG   REJECTED shahai-swd-upstream-noname / swd_shijianhuisu1
DBG   REJECTED shahai-swd-upstream-noname / swd_yuchankan
DBG   REJECTED shahai-swd-upstream-noname / swd_yuchankun
DBG   REJECTED shahai-swd-upstream-noname / swd_yuchanxun
DBG   REJECTED shahai-swd-upstream-noname / swd_zhongxinghujia1
DBG   REJECTED shahai-yunchou-upstream-noname / geanguanhuo
DBG   REJECTED shahai-zhenfa-upstream-noname / swd_changshezhen1
DBG   REJECTED shahai-zhenfa-upstream-noname / swd_changshezhen2
DBG   REJECTED shahai-zhenfa-upstream-noname / swd_niaoxiangzhen1
DBG   REJECTED shahai-zhenfa-upstream-noname / swd_qixingzhen1
DBG   REJECTED shahai-zhenfa-upstream-noname / swd_tianfuzhen1
DBG   REJECTED wudian-upstream-noname / tao
DBG   REJECTED wudian-upstream-noname / youdishenru
```

## 分布（去重 31 项）

| 包 | 候选数 | 备注 |
|---|---:|---|
| `shahai-swd` | **19** | **待逐卡定性** |
| `shahai-zhenfa` | 5 | **已定性（全部合理）**：4 张依赖势力/围攻/队列/同势力相邻 + 1 副本 |
| `shahai-gujian` | 3 | **待逐卡定性** |
| `wudian` | 2 | **待逐卡定性** |
| `shahai-yunchou` | 1 | **已定性（合理）**：`geanguanhuo` 须双目标 |
| `shahai-hearth` | 1 | **已定性（合理）**：`zhiliaobo` 须受伤目标（样本 115 已验证） |

⇒ **已定性 7 项（全合理）**，**待定性 24 项**（swd 19 + gujian 3 + wudian 2）。

## 边界

1. 只判「是否至少一种形态被受理」，**不判语义等价**；
2. 被拒须**完整读取上游条件**后定性（**截断读取会把多条件误判为单条件**——台账第 128 行教训）；
3. 更宽松的构造（受伤目标 / 双目标 / 势力与围攻关系）可进一步降低误报。

## 逐卡定性（第一批：gujian 3 + wudian 2，2026-10-19）

| 卡 | 上游条件 / 性质 | 定性 |
|---|---|---|
| `dinvxuanshuang`（帝女玄霜） | 需**使用者已受伤**（样本 96 已验：满血时 `Rejected(HpFull)`） | **合理**（普查局面 0 号满血） |
| `jinlianzhu`（金莲珠） | `global` **打断牌**（仅在**触发窗口**可打出；样本 87 已验） | **合理**（普查无触发窗口） |
| `longxugou`（龙须钩） | `filterTarget` 要求目标**装备区有牌**（样本 86 已验门槛） | **合理**（**全包普查未给目标装备**——见下「构造缺口」） |
| `tao`（桃） | 需**受伤目标**（回血牌） | **合理** |
| `youdishenru`（诱敌深入） | **杀响应转化**（打出后杀无效、来源受 1 点反噬；基准登记） | **合理**（响应牌，非主动使用） |

⇒ **5/5 合理**；**待定性剩 19 项**（全部在 `shahai-swd`）。

### 构造缺口（本批暴露）

**全包普查的宽松局面未给目标装备**（小包版给了 1 件）⇒ 依赖「目标装备区有牌」的卡
（如 `longxugou`）会被误判为被拒 ⇒ **后续应在构造中加入装备**，以降低误报。

### 重点待核（swd 19 中的异常项）

`swd_zhongxinghujia1`（重型护甲）为**装备牌**——装备牌通常按「无显式目标」受理
（样本 103 木头面具即如此）⇒ 其被拒**不合常理**，**须专项核对**（可能该卡为 `hslingjian_*` 子件、
或需特定条件）。其余 18 项多为**响应/回复/机关**类，条件受限的可能性较大。

## 逐卡定性（第二批：swd，2026-10-19）

按「**完整读取上游 `filterTarget`/`enable`**」口径逐卡核对（`swd/card.js`）：

| 卡 | 上游条件（**逐字要点**） | 定性 |
|---|---|---|
| `caoyao`（草药） | `type: basic`；`filterTarget: target.hp < target.maxHp`（**须受伤**）；`enable: !hasSkill("tianxianjiu")` | **合理**（普查全员满血） |
| `shijianhuisu1`（时间回溯） | `filterTarget: target !== player && countCards("e") > 0`（**须有装备**） | **合理** |
| `jiguanshu`（机关术） | `filterTarget: target === player`；`enable`: **装备区须含指定牌** | **合理**（普查未给 0 号装备） |
| **`guilingzhitao`（龟龄芝桃）** | `type: equip`；**无 `filterTarget`** | **候选**（装备牌按「无显式目标」应受理，同样本 103） |
| **`hufu`（虎符）** | `type: basic`；**无 `filterTarget`** | **候选** |
| **`jiguantong`（机关筒）** | `filterTarget: target !== player` | **候选**（`[1]` 形态本应受理） |
| **`jiguanyaoshu`（机关妖术）** | `filterTarget`: 目标**有空装备位**且 `!hasSkill("jiguany…")` | **候选**（普查局面两条件均满足） |
| **`shenmiguo`（神秘果）** | `filterTarget: target === player`（**自己**） | **候选**（`[0]` 形态本应受理） |
| **`yuchankan`/`yuchankun`/`yuchanxun`（渔产三件）** | `type: jiqi`；**无 `filterTarget`** | **候选** |

⇒ 本批 **2 合理 + 8 候选**；**候选累计 8 项**（全部在 swd）。

### 候选的共同点与下一步

8 项候选**均为「无 `filterTarget`」或「条件明显满足」**的卡（装备/基本/机器类）⇒
**大概率存在实现侧问题**（与「重型护甲口径自创」同类，或**未接入 `CardKind` 分派**）。

**下一步**：对这 8 项**逐卡**核对原生 `CardKind` 分派与目标校验
（用 `contains` 匹配行为枚举名，**不按卡名 grep**——见样本 97/99 的命名教训）。
