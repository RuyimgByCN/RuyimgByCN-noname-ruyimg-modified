# 2026-10-07 E01/G0 功能面清点（37 来源基线）

E01/G0 的功能级清点基线：对两仓 37 个来源逐个提取功能面特征（只读，不执行
扩展代码）。来源目录与字节基线沿用 [10-06 复核](../2026-10-06-native-progress-audit/README.md)
（8+29=37 项，未变化）。

## 产物

- [feature_inventory.py](feature_inventory.py)：清点工具（正则/括号计数近似，
  不依赖 JS 运行时；永不 import 扩展）。
- [feature-inventory.json](feature-inventory.json)：37 来源逐项特征（入口规模、
  `game.import`、skill/card/character/translate/config 段键数、配置与设置引用、
  外部扩展引用、混淆标记、子插件、资源类别与体量）。
- [feature-summary.json](feature-summary.json)：逐来源摘要表。

## 关键发现

| 发现 | 来源 | 说明 |
|---|---|---|
| 混淆入口确认 | 龙血玄黄 | 入口 1 行 + packed 标记——功能面无法静态提取，行为样本须动态观测（与主稿 E01 行的「混淆入口」登记一致） |
| 混淆标记第二例 | 选将美化 | packed 标记 |
| 大体量来源 | 祖安武将（51,474 行）、千幻聆音（15,970）、十周年UI（12,724）、AI优化（10,967）、选将美化（9,094）、雷霆万钧（9,187） | 功能清单需按来源规模分批做行为样本 |
| 壳扩展形态 | 欢乐卡牌、杀海拾遗、玩点论杀、3D精选、英雄杀 | 入口 ~22 行壳 + `card/card.js`、`card/skill.js` 承载数据（原生 G1 已按此形态转换）——本工具 v1 的 shell 段计数未覆盖 `const card = {}` 形态，列为工具改进项 |
| 配置/设置类 | SJ Settings、全能搜索、更新保护、拖拽读取、手杀MVP、电脑适配 | settings/config 引用密集（G2 公共配置类的对照面） |
| eval 使用 | AI优化(6)、更新保护(6)、斗转星移(3)、手杀MVP(3)、手杀ui(4)、千幻聆音(4) 等 | 独立执行后端（C1）评估时的受限执行风险清单 |

## 结论边界

- 已交付：E01 功能面**清点基线**（37/37 全覆盖、覆盖与 10-06 来源清单对齐）。
- 仍属 E01 待做（基线之后的逐来源深化）：每来源的旧行为样本（动态观测，
  龙血玄黄必须动态）、菜单/规则分支的语义清单、「目标接口与验证入口」的
  逐项登记；本工具 shell 段计数改进。

## 改动文件

- feature_inventory.py / feature-inventory.json / feature-summary.json /
  inventory-stderr.log（本目录）。
- 主稿 E01 行与本 README。
