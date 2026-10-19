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
