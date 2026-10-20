# 41 个扩展包**逐包清单**（2026-10-19，数据驱动）

- 工作包：**NC08 / 扩展台账**；规则版本 v322（工作树）/ 存档格式 v105
- 生成方式：枚举 `packages/extensions/*/`，读 `package.npack.json` 与 `assets/{cards,character,pile,config}.json`
 （原始数据：[packages.tsv](packages.tsv)）

- 合计 **41** 个包目录（**40** 对应 37 个来源实例；`boss-zhugeliang-upstream-liuli` 属 T11 Boss 内容）

| # | 包目录 | `package_id` | namespace | 资产形态 | 主数量 | 已有验收证据 |
|---:|---|---|---|---|---:|---|
| 1 | `ai-tuning-upstream-liuli` | `ai-tuning` | upstream-liuli | config(14 项) | 0 | **仅台账行** |
| 2 | `boss-play-upstream-liuli` | `boss-play` | upstream-liuli | config(12 项) | 0 | **仅台账行** |
| 3 | `boss-play-upstream-noname` | `boss-play` | upstream-noname | config(12 项) | 0 | **仅台账行** |
| 4 | `boss-zhugeliang-upstream-liuli` | `boss-zhugeliang` | upstream-liuli | cards(0) + skills(1) | 0 | **仅台账行** |
| 5 | `cardpile-upstream-liuli` | `cardpile` | upstream-liuli | pile(160) | 160 | 保存恢复/升级、效能 |
| 6 | `cardpile-upstream-noname` | `cardpile` | upstream-noname | pile(160) | 160 | 保存恢复/升级、效能 |
| 7 | `coin-upstream-liuli` | `coin` | upstream-liuli | config(8 项) | 0 | **仅台账行** |
| 8 | `coin-upstream-noname` | `coin` | upstream-noname | config(5 项) | 0 | **仅台账行** |
| 9 | `d3-upstream-noname` | `d3-select` | upstream-noname | character(1) | 1 | 保存恢复/升级、效能 |
| 10 | `desktop-adapt-upstream-liuli` | `desktop-adapt` | upstream-liuli | config(5 项) | 0 | **仅台账行** |
| 11 | `doomsday-brawl-upstream-liuli` | `doomsday-brawl` | upstream-liuli | cards(0) + skills(3) + config(5 项) | 0 | **仅台账行** |
| 12 | `douzhuan-xingyi-upstream-liuli` | `douzhuan-xingyi` | upstream-liuli | config(18 项) | 0 | **仅台账行** |
| 13 | `drag-read-upstream-liuli` | `drag-read` | upstream-liuli | config(13 项) | 0 | **仅台账行** |
| 14 | `elo-sanction-upstream-liuli` | `elo-sanction` | upstream-liuli | config(6 项) | 0 | **仅台账行** |
| 15 | `epic-cards-upstream-liuli` | `epic-cards` | upstream-liuli | cards(0) + skills(2) + config(13 项) | 0 | **仅台账行** |
| 16 | `fool-plugin-upstream-liuli` | `fool-plugin` | upstream-liuli | config(10 项) | 0 | **仅台账行** |
| 17 | `hero-kill-upstream-noname` | `hero-kill` | upstream-noname | cards(0) + skills(5) + config(5 项) | 0 | **仅台账行** |
| 18 | `huanle-upstream-noname` | `huanle-cards` | upstream-noname | cards(7) + skills(2) | 7 | 配置/组合、效能 |
| 19 | `longxue-upstream-liuli` | `longxue` | upstream-liuli | cards(0) + skills(4) + config(8 项) | 0 | **仅台账行** |
| 20 | `mark-supplement-upstream-liuli` | `mark-supplement` | upstream-liuli | cards(0) + skills(6) + config(7 项) | 0 | **仅台账行** |
| 21 | `martial-upstream-liuli` | `martial` | upstream-liuli | config(12 项) | 0 | **仅台账行** |
| 22 | `omnisearch-upstream-liuli` | `omnisearch` | upstream-liuli | config(2 项) | 0 | **仅台账行** |
| 23 | `qhly-upstream-liuli` | `qhly` | upstream-liuli | config(10 项) | 0 | **仅台账行** |
| 24 | `select-beautify-upstream-liuli` | `select-beautify` | upstream-liuli | cards(0) + skills(7) + config(5 项) | 0 | **仅台账行** |
| 25 | `shaai-ow-upstream-noname` | `shaai-ow` | upstream-noname | cards(0) + skills(11) | 0 | **仅台账行** |
| 26 | `shaai-xianjian-upstream-noname` | `shaai-xianjian` | upstream-noname | cards(0) + skills(22) | 0 | **仅台账行** |
| 27 | `shahai-gujian-upstream-noname` | `shahai-gujian` | upstream-noname | cards(40) | 40 | 配置/组合、效能 |
| 28 | `shahai-gwent-upstream-noname` | `shahai-gwent` | upstream-noname | cards(21) | 21 | 配置/组合、效能 |
| 29 | `shahai-hearth-upstream-noname` | `shahai-hearth` | upstream-noname | cards(17) | 17 | 配置/组合、效能 |
| 30 | `shahai-mtg-upstream-noname` | `shahai-mtg` | upstream-noname | cards(10) | 10 | 配置/组合、效能 |
| 31 | `shahai-swd-upstream-noname` | `shahai-swd` | upstream-noname | cards(90) + skills(16) | 90 | 配置/组合、保存恢复/升级、效能 |
| 32 | `shahai-yunchou-upstream-noname` | `shahai-yunchou` | upstream-noname | cards(23) + character(10) | 23 | 配置/组合、效能 |
| 33 | `shahai-zhenfa-upstream-noname` | `shahai-zhenfa` | upstream-noname | cards(8) | 8 | 配置/组合、效能 |
| 34 | `shousha-full-upstream-liuli` | `shousha-full` | upstream-liuli | config(15 项) | 0 | **仅台账行** |
| 35 | `shousha-mvp-upstream-liuli` | `shousha-mvp` | upstream-liuli | config(8 项) | 0 | **仅台账行** |
| 36 | `sj-settings-upstream-liuli` | `sj-settings` | upstream-liuli | config(5 项) | 0 | **仅台账行** |
| 37 | `skin-switch-upstream-liuli` | `skin-switch` | upstream-liuli | config(13 项) | 0 | **仅台账行** |
| 38 | `thunder-upstream-liuli` | `thunder` | upstream-liuli | config(14 项) | 0 | **仅台账行** |
| 39 | `update-guard-upstream-liuli` | `update-guard` | upstream-liuli | config(8 项) | 0 | **仅台账行** |
| 40 | `wudian-upstream-noname` | `wudian` | upstream-noname | cards(14) + skills(13) | 14 | 配置/组合、保存恢复/升级、效能 |
| 41 | `wuxing-upstream-liuli` | `wuxing` | upstream-liuli | cards(1) + skills(2) | 1 | 配置/组合、效能 |

## 说明（**口径**）

1. **「已有验收证据」只列本项目实际跑过的验收**：配置/组合（`nc08_config_acceptance`、`nc08_composition_pair_matrix`）、
   保存恢复/升级（`nc08_save_restore_*`、`nc08_upgrade_five_sources`）、效能（`nc12_bench` 逐来源档）；
2. **未做**：八项中的**目标平台真机运行**、**产品级长程**、**逐来源规则差异签署**（仅 U01/U03/U04/U07 有差异族）；
3. **`package_id` 重名**：`cardpile` 同时用于 `cardpile-upstream-{noname,liuli}`、`boss-play` 同时用于两个 namespace
   ⇒ bench 输出的 `package_ids` **不能区分**；
4. **无卡牌资产的包**：`ai-tuning`/`boss-play`（`config.json` 14/12 项）、`boss-zhugeliang`（`cards(0)+skills(1)`）
   ⇒ 这些包走**非卡牌管线**，不进入卡牌组合/保存恢复验收；
5. 本表**不判定 G0–G5 等级**（无逐来源机器可读等级字段），只列**可核实事实**。
