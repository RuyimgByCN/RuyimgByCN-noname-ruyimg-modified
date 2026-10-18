# NC08 基准面复核（21 个上游样本）（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**21 个样本的基准来源全部核对完毕**；发现并已更正的误用 **2 例**（集智/奸雄）；**伪差异清零**

## 1 动机

上一轮（提交 `15e7c6f5`）发现：上游**同名技能可能在多个面**存在且语义不同
（`character/standard/`、`character/sb/`、`mobile/`、`newjiang/` …）。初版把 `sb` 面当作基准，
导致 **2 项伪差异**被登记。为防同类错误，对全部样本做一次基准面复核。

## 2 方法

逐样本提取其文档注释中引用的**上游源码路径**，判定是否属于「该来源的基准面」：

```bash
for f in crates/engine-native/tests/nc08_upstream_*.rs; do
  grep -oE "archive/upstream/[^ )\"]+" "$f" | sort -u
done
```

判据：
- **扩展来源**（U05/L06/L10/L19/L20/U07 等）：基准面 = **扩展自身文件**（`extension/...`）→ 唯一，无歧义；
- **基础仓库面**（标准武将/卡牌、模式规则）：基准面 = **`character/standard/`、`card/standard.js`、`mode/*.js`**
  （见[规则基准](../../../architecture/native-rules-baseline.md)「本地上游对齐的新增核查」），
  `character/sb/`、`mobile/` 等**不是**基准。

## 3 结果

| 类别 | 样本数 | 引用路径 | 判定 |
|---|---:|---|---|
| 扩展来源（唯一面） | 17 | `extension/...`（杀海拾遗各子包、玩点论杀、标记补充、欢乐卡牌、史诗卡牌、wuxing、gwent/gujian/hearth/mtg/zhenfa/yunchou/ow/xianjian/swd） | **正确**（无歧义） |
| 基础仓库·标准技能 | 3 | `character/standard/skill.js`（ganglie；jizhi/jianxiong 已于 `15e7c6f5` 更正） | **正确** |
| 基础仓库·标准卡牌 | 2 | `card/standard.js`（standard_tricks、kuwu 引用） | **正确** |
| 基础仓库·模式 | 1 | `mode/identity.js`（身份局开局判定） | **正确** |
| **误用（已更正）** | 2 | 初版引用 `character/sb/skill.js` | **已改为 standard 面并撤销伪差异** |

**风险边界（写入证据）**：基准面歧义**只存在于基础仓库面**（本批 6 个样本），
扩展来源不存在多面问题 → 后续样本只需在**基础仓库面**样本中标注基准面。

## 4 验证

| 检查 | 结果 |
|---|---|
| 21 个样本引用路径提取 | 全部可提取且指向存在的文件 |
| 伪差异 | **0**（2 项已于 `15e7c6f5` 撤销） |
| 定向回归 | 各样本用例通过（见各自 targeted.log） |
| 行数门禁 | 8305 文件 ≤ 2000 行 |

## 5 未完成 / 待办

- [ ] 新样本模板强制写明「基准面 + 文件路径」（本次复核后已成为惯例，建议写入 AGENTS/模板）
- [ ] 上游 harness 接入后，基准面判定应与 harness 读取的文件一致（[登记](../2026-10-16-todo-register/todo-register.json) `NC08-UPSTREAM-HARNESS`）
