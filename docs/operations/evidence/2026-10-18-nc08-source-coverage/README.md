# NC08 第⑦层来源覆盖：**21/37 的天花板来自「未转换」而非「未采样」**（2026-10-19）

- 日期：2026-10-19
- 工作包：**NC08**（[方案 §NC08](../../../architecture/native-convergence-plan.md#nc08)）
- 规则版本 v322（工作树）/ 存档格式 v105
- 结论：**已转换的 14 个包全部有第⑦层样本**；**其余来源尚未转换（G1 未做）** → 第⑦层**无法**为其建样本

## 1 方法

枚举 `packages/extensions/*/assets/cards.json` 的 `cards + generals` 数量，按「有无可执行内容」分类
（清单见 [package-inventory.txt](package-inventory.txt)）：

| 类别 | 包数 | 说明 |
|---|---:|---|
| **有内容（cards+generals > 0）** | **14** | 可做第⑦层行为对照（原生已有对应行为） |
| **无内容（= 0）** | **5** | **包数据为空**（上游**有**内容，只是**未转换进包**）→ 原生无对应行为可对照 |

**有内容的 14 个包**：`shaai-ow`(24) / `shaai-xianjian`(36) / `shahai-gujian`(40) / `shahai-gwent`(21) /
`shahai-hearth`(17) / `shahai-mtg`(10) / `shahai-swd`(169) / `shahai-yunchou`(23) / `shahai-zhenfa`(8) /
`wudian`(23) / `wuxing`(1) / `huanle`(7) / `longxue`(1) / `boss-zhugeliang`(1)。

**无内容的 5 个包**：`doomsday-brawl`(L19) / `epic-cards`(L10) / `hero-kill`(U08) /
`mark-supplement`(L20) / `select-beautify`(L27)。

## 1b 上游**确有内容**（更正上表措辞，2026-10-19 同轮）

核对 5 个空包对应的上游扩展（[upstream-content.txt](upstream-content.txt)）：

**统计口径更正（第二次）**：`extension.js` 往往只是**加载器**（如 U08 英雄杀 23 行的
`extension.js` 只 `import` 了 `./main/content.js` 与 `../character/index.js`，真正内容在子目录）
→ 改为**整目录统计**（该扩展下全部 `.js`）：

| 来源 | 整目录 .js 行数 | `lib.character` | `lib.skill` | `lib.card` |
|---|---:|---:|---:|---:|
| L19 末日乱斗 | 3,410 | 32 | 75 | 2 |
| L10 史诗卡牌 | **10,710** | 4 | 32 | 5 |
| U08 英雄杀 | **3,855** | 2 | 5 | 1 |
| L20 标记补充 | 6,518 | 13 | 84 | 4 |
| L27 选将美化 | **9,093** | **64** | **121** | 6 |

→ **两次更正**：①这些来源**不是「没有内容」**（上游确有内容，只是**未转换进包**，G1 未做）
→ 第⑦层**无法**为其建样本（**无可对照的原生行为**）；②**U08 英雄杀不是 23 行存根**——
其 `character/skill.js` 等子目录合计 **3,855 行**（`yxs_*` 武将 44 处引用），
**`extension.js` 只是加载器**。
规模上 **L10 史诗卡牌（10,710 行）** 与 **L27 选将美化（64 将 + 121 技）** 最大。

## 1c `jsjiami` 混淆来源全库盘清（2026-10-19）

`grep -rl "jsjiami" archive/upstream/` → **6 个文件**（[obfuscation-scan.txt](obfuscation-scan.txt)）：

| 文件 | 归类 |
|---|---|
| `noname-liuli/noname/extension/龙血玄黄/extension.js` | **来源 `longxue`** → 已建**弱对照**样本 41 |
| `noname-liuli/noname/extension/如真似幻/extension.js` | **来源 L12 如真似幻** → 未建样本，**预期同为弱对照** |
| `noname-liuli/noname/game/game.js` | 核心（非来源） |
| `noname/apps/core/noname/init/compatible.js` | 核心（疑为混淆**检测器**） |
| `noname/apps/core/noname/init/import.ts` | 核心（同上） |
| `noname/apps/core/noname/library/init/index.js` | 核心（同上） |

→ **受影响来源恰为 2 个**（`longxue`、L12 如真似幻）；**扩展目录内命中为 0**，
故其余来源的上游源码**可读**，可做**源码级**对照。

## 2 判读（不夸大）

- 第⑦层样本已覆盖 **21 个来源**（见台账 §3.3）——其中包含**同一包的多来源**（如 U05 杀海拾遗的
  各子包）与**标准面**；
- 剩余来源（L08/L11/L12/L14/L19/L20/L25/L27/L28/U01/U05 余项/U07/U08 等）中，
  **多数在 `packages/extensions/` 下只有包元数据（0 卡 0 将）**，而**上游扩展确有内容**
  （见 §1b）→ 属**未转换（G1 未做）**，**不是**「第⑦层漏采样」、**也不是**「上游无内容」
  → 因此**不能**通过继续加样本来提升 37 来源覆盖；
- 要推进这些来源，需先做**内容转换（G1）**：把上游 JS 的功能面转成包数据 + 宿主行为键
  （属 T12/T9 面，见[进度主稿](../../../architecture/native-progress.md)与[台账](../../../migration/upstream-extension-reconstruction.md)）。

## 3 验证

| 检查 | 结果 | 证据 |
|---|---|---|
| 枚举脚本可复跑 | 是（清单文件随附） | [package-inventory.txt](package-inventory.txt) |
| 与台账 §3.3 的 21 来源一致 | 是（含标准面与同包多来源） | 台账 §3.3 |
| 行数门禁 | 8324 文件 ≤ 2000 行 | — |

## 4 未完成 / 待办

- [ ] **未转换来源的 G1 内容转换**（5 个空包 + 其余来源的余项）——第⑦层样本的**前置条件**
- [ ] `longxue`(L16?) / `boss-zhugeliang` 等仅 1 条目的包，是否值得建样本（评估）
- [ ] 台账「覆盖面」一栏补充本口径（避免把「未转换」误读为「未采样」）
