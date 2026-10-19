# 标准牌组构成核对：`CardKind` 实现 vs 牌组实例（2026-10-19）

- 工作包：**NC08 / 修复清单第 54 项附带发现**；规则版本 v322（工作树）/ 存档格式 v105
- 用例：`crates/engine-native/tests/nc08_standard_deck_coverage.rs`

## 结果（**量化**）

```
标准牌组：目录 124 张实例、覆盖 34 个不同 kind
有实现但牌组无实例的 kind：27 个
```

## 27 个差集（分两类）

**① 核心标准牌（9 个）——值得关注**：

| kind | 中文 | 说明 |
|---|---|---|
| `Guohe` | 过河拆桥 | 标准锦囊 |
| `Shunshou` | 顺手牵羊 | 标准锦囊 |
| `Juedou` | 决斗 | 标准锦囊 |
| `Nanman` | 南蛮入侵 | 标准 AOE |
| `Wanjian` | 万箭齐发 | 标准 AOE |
| `Wuxie` | 无懈可击 | 标准响应 |
| `LeBu` | 乐不思蜀 | 标准延时 |
| `Shandian` | 闪电 | 标准延时 |
| `Bingliang` | 兵粮寸断 | 标准延时 |

**② 扩展包牌（18 个）——预期内**：`Hufu`/`Tianxianjiu`/`Caoyao`/`Liuxinghuoyu`/`Chilongya`/
`Xixueguizhihuan`/`Guiyoujie`/`Shentoumianju`/`Xingjunyan`/`Qinglonglingzhu`/`Zhufangshenshi`/
`Fengxueren`/`Yufulu`/`Yangpijuan`/`Daihuofenglun`/`Xiayuncailing`/`Langeguaiyi`/`Jiguan`
（这些 kind 的实例**只由扩展包提供**，标准建局无实例是**正确**的）。

## 判读（**不预设缺陷**）

1. **扩展包 18 项差集是正常的**（kind 由包提供实例）；
2. **核心 9 项**说明：**标准牌组 124 张不含这 9 种牌** ⇒
   - 若**设计如此**（牌组构成属既定范围），则应在**规则基准/覆盖清单**登记，并说明这些牌的**行为测试依赖夹具构造**；
   - 若**非设计**，则这 9 种牌的**行为面在当前牌组下不可达**（与第 54 项同类）。
3. **本用例不设硬断言**：差集是**事实**，是否为缺陷需与**牌组构成口径**对照后判定。

## 待办

- [ ] 对照**规则基准/覆盖清单**确认「标准牌组 124 张」是否为**既定构成**
- [ ] 若非既定：补 9 种核心牌实例（会影响 golden 与存档，需按 AGENTS.md「golden 变化须先查原因」处理）
- [ ] 若既定：把「9 种牌行为测试依赖夹具」写入覆盖清单
