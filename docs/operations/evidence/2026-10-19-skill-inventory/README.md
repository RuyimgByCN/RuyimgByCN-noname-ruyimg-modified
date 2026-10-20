# 全库**技能映射清单**：94 技能 / 89 已映射 / **5 未映射**（2026-10-19）

- 工作包：**NC08 / 八项第 7 项（规则差异）的声明面基线**
- 用例：`crates/engine-native/tests/nc08_skill_mapping_inventory.rs`（**回归护栏**）
- 原始输出：[run.txt](run.txt)

## 1 数据（跨全部包，来自各包 `assets/cards.json` 的 `skills[]`）

| 包 | 技能 | 已映射 | **未映射** |
|---|---:|---:|---:|
| `shahai-swd` | 16 | 16 | 0 |
| `shaai-xianjian` | 22 | 22 | 0 |
| `shaai-ow` | 11 | 11 | 0 |
| `wudian` | 13 | 11 | **2** |
| `longxue` | 4 | 1 | **3** |
| `select-beautify` | 7 | 7 | 0 |
| `mark-supplement` | 6 | 6 | 0 |
| `hero-kill` | 5 | 5 | 0 |
| `doomsday-brawl` | 3 | 3 | 0 |
| `epic-cards` | 2 | 2 | 0 |
| `huanle` | 2 | 2 | 0 |
| `wuxing` | 2 | 2 | 0 |
| `boss-zhugeliang` | 1 | 1 | 0 |
| **合计** | **94** | **89** | **5** |

## 2 未映射的 5 条（**每条都有 `reason`** ✓）

```
longxue-upstream-liuli/mbcmjiushi
longxue-upstream-liuli/mbcmfangzhu
longxue-upstream-liuli/mbjuejin
wudian-upstream-noname/duoqi
wudian-upstream-noname/youzhan
```

用例断言：**未映射条目必须带非空 `reason`**（**不允许静默缺口** ✅）。

## 3 这条数据的用途

| 用途 | 说明 |
|---|---|
| **回归护栏** | 任何人新增未映射技能（或删掉 `reason`）都会让用例失败 ✅ |
| **规则差异的**「**声明面**」基线 | **89 条声称已映射** ⇒ 它们是**待逐条比对等价性**的对象 |
| **不替代**等价性验收 | ❗ 本表**只证明「有映射」**，**不证明「与上游等价」** —— U08 已证「有实现 ≠ 可达 ≠ 等价」 |

## 4 边界

1. **只扫 `skills[]` 声明面** —— 未扫 `generals[].skills` 与 `cards[].skills` 中的映射（**另一层**）；
2. 未判定 89 条已映射者的**行为等价**（需逐条样本，属各来源的规则差异工作）；
3. 计数为**声明条目数**，不等于「不同技能数」（同名可重复）。
