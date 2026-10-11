# 2026-10-11 T9：legacy-analyzer 新增壳扩展 `const card = {}` 段统计

任务来源：[进度主稿 T9](../../../architecture/native-progress.md)（领取锁 `ac915389`）。

## 工具能力

- `legacy_analyzer::count_card_segment(source) -> CardSegmentStats`（`crates/legacy-analyzer/src/cardstats.rs`）：定位**顶层** `const card = {` 段（词边界判定，函数体内局部 `const card = ...` 与 `const cards` 复数声明不误判），枚举卡牌键名、`type`/`fullskin` 字段、行为函数键有无（`filter/content/precontent/mod/ai/...`，`enable: true` 等数据值不算行为）。
- CLI：`cargo run -p legacy-analyzer -- card-stats <card.js> [--json out.json]`。
- 单测 4 支（键/类型/行为统计、无段、函数内局部声明负例、复数声明负例）+既有 18 支全绿（lib 13/0、integration 7/0、1/0）。

## 五来源真实统计（上游 `apps/core/extension/`，2026-10-11 实测）

| 来源 | 文件 | 卡牌数 | 纯数据/含行为 | type 分布 | 备注 |
|---|---|---|---|---|---|
| 玩点论杀 | card/card.js | 3 | 0/3 | trick×3 | 调兵遣将/望梅止渴/三尖——与已转 14 张口径吻合（基础牌在数据表不在本段） |
| 欢乐卡牌 | card/card.js | 6 | 0/6 | basic×1+equip×5 | monkey 等——与 g1_huanle 转换面吻合 |
| 3D精选 | character/card.js | 0 | — | — | `const cards = {};` 空段（卡定义在别处/无） |
| 英雄杀 | character/skill.js | 0 | — | — | 无卡段（纯技能文件；负例实证：函数内局部 `const card` 不再误判） |
| 杀海拾遗·wuxing | play/wuxing/card.js | 1 | 1/0 | equip×1 | 五行盘（equip5，数据直转已落地） |

## 边界

- 浅层统计：行为面只判定「有无」，不解析语义；`type` 分布只取字符串字面量。
- 启发式按上游扩展 card.js 单 tab 缩进惯例；非惯例缩进的文件统计为 0（可后续按需扩展，不阻塞评估用途）。
