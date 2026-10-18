# T9 来源证据关系登记表（台账同源解析 · 机器可读字段 · 实测重建）

- 生成：`python3 -B docs/operations/evidence/2026-10-17-t9-source-samples/gen_registry.py`（可复现；表内计数全部由本脚本实测得出，不手写）。
- 证据关系：**来源功能/配置入口 → 上游证据 → 生产接口 → 测试案例 → 源码/规则/平台版本 → 结果 → 未覆盖项**。
- 同源绑定：来源行、状态标签、证据目录、包目录、测试名均解析自 [`docs/migration/upstream-extension-reconstruction.md`](../../../migration/upstream-extension-reconstruction.md)；整包状态取自 [`todo-register.json`](../2026-10-16-todo-register/todo-register.json) 中 tags 含「整包」条目的 `status` 机器字段，不另建状态表。
- **不自动认证完成**：脚本只报告「引用是否存在 + 是否登记未覆盖项」，不给出完成结论。匹配「八项整包」字符串、包目录存在或测试文件存在都不足以判定整包完成。

## 实测计数（2026-10-18，HEAD `5e141764`，255 项未提交改动）

| 口径 | 值 |
|---|---|
| 来源实例 | **37**（U01–U08 + L01–L29） |
| 台账标签 | 八项整包 **24** / 部分重构 9 / G0 完成 4 / 待重构 0 |
| 登记表整包条目（机器字段） | 已 resolved **24** / 仍 open **13**（共 37 条） |
| 行内引用证据目录 | 173 个引用，缺失 0 |
| 行内引用转换包 | 40 个引用，缺失 0（在盘包 **41** 个，全部含 `package.npack.json`） |
| 行内引用测试文件 | 1 个引用，缺失 0；另有 294 个测试名只在证据目录中出现（未写进台账行） |
| 装载器模块引用 | 2 个，缺失 0 |
| 源码版本（实测） | 规则 `STANDARD_RULESET_VERSION = 322`（crates/engine-native/src/content/standard.rs:126）、存档 `ARCHIVE_FORMAT_VERSION = 105`（crates/engine-native/src/archive.rs:113） |
| 平台证据线索 | 有 **31** / 无 **6**（关键词判据，非平台验收结论） |
| 证据缺失来源 | 缺上游证据 0 / 缺生产接口 5 / 缺测试案例 4 / 缺平台结果 6 |
| 整包标签存疑 | **19/24** 个标签行在行内仍登记未迁移功能/表现/平台面 |

## 逐来源证据关系

| 来源 | 功能/配置入口 | 上游证据 | 生产接口 | 测试案例 | 版本 | 结果 | 未覆盖项 |
|---|---|---|---|---|---|---|---|
| L01 AI优化 | 技能威胁/武将权重、禁将、身份策略、规则钩子与设置 | `../../../archive/upstream/noname-liuli/noname/extension/AI优化/extension.js` | `ai-tuning-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-l01-ai-tuning-eight` | — |
| L02 SJ Settings | Android 外部打开扩展、配置导入、下载入口和调试台 | `../../../archive/upstream/noname-liuli/noname/extension/SJ Settings/extension.js` | `sj-settings-upstream-liuli` | —；证据目录：`l02_sj_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l02-eight` | `登记差异` |
| L03 boss | mode_extension_boss/jiange 注册、旧数组式角色与缓存 | `../../../archive/upstream/noname-liuli/noname/extension/boss/extension.js` | `boss-play-upstream-liuli` | —；证据目录：`boss_play_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-u02-l03-boss-play-eight` | `登记差异`；**标签存疑**：行内仍见 不画、界面 |
| L04 cardpile | 旧 play 牌堆比例/随机生成；来源同含 `jiu`、`leisha` | `../../../archive/upstream/noname-liuli/noname/extension/cardpile/extension.js` | `cardpile-upstream-liuli` | —；证据目录：`g1_cardpile.rs`、`l04_cardpile_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-08-l04-cardpile-liuli-lifecycle`、`2026-10-17-t12-l04-eight` | `登记差异` |
| L05 coin | 金币商店/效果、模式门控 | `../../../archive/upstream/noname-liuli/noname/extension/coin/extension.js` | `coin-upstream-liuli` | —；证据目录：`l05_coin_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l05-eight` | `登记差异`；**标签存疑**：行内仍见 录像 |
| L06 wuxing | 五行属性、五行盘卡牌/技能、开局随机插牌 | `../../../archive/upstream/noname-liuli/noname/extension/wuxing/extension.js` | `wuxing-upstream-liuli` | —；证据目录：`g1_wuxing.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-08-u05-wuxing-lifecycle`、`2026-10-17-t12-l06-eight` | `登记差异`；**标签存疑**：行内仍见 帮助、录像、联机 |
| L07 全能搜索 | 武将/技能/卡牌检索、详情/语音和脚本执行入口 | `../../../archive/upstream/noname-liuli/noname/extension/全能搜索/extension.js` | `omnisearch-upstream-liuli` | —；证据目录：`l07_omnisearch.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l07-eight` | `登记差异`；**标签存疑**：行内仍见 界面、语音 |
| L08 十周年UI | 牌桌/选择部件、动画、Worker、卡牌/玩家函数覆盖和技能钩子 | `../../../archive/upstream/noname-liuli/noname/extension/十周年UI/extension.js` | — | — | HEAD `5e141764` / 规则 v322 / 存档 v105 / **缺平台结果** | 登记表 open；证据目录 `2026-10-17-remaining-sources-g0` | `不迁移` |
| L09 千幻聆音 | 皮肤/语音、角色详情、共享映射、主题及插件系统 | `../../../archive/upstream/noname-liuli/noname/extension/千幻聆音/extension.js` | `qhly-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-l09-qhly-eight` | —；**标签存疑**：行内仍见 插件系统、语音 |
| L10 史诗卡牌 | 卡面/对话框特效，reguhuo/rangjie 等规则覆盖 | `../../../archive/upstream/noname-liuli/noname/extension/史诗卡牌/extension.js` | `epic-cards-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-l10-epic-cards-eight` | `不迁移`、`阻塞`；**标签存疑**：行内仍见 阻塞 |
| L11 太虚幻境 | 框架、技能/卡牌/角色、侍从/收集/祝福、赛季/关卡与 UI | `../../../archive/upstream/noname-liuli/noname/extension/太虚幻境/extension.js` | — | — | HEAD `5e141764` / 规则 v322 / 存档 v105 / **缺平台结果** | 登记表 open；证据目录 `2026-10-17-remaining-sources-g0` | — |
| L12 如真似幻 | 大厅/启动页、排位与跨局数据、主题和旧流程补丁 | `../../../archive/upstream/noname-liuli/noname/extension/如真似幻/extension.js` | — | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-17-l12-rzsh-slice1` | — |
| L13 手杀MVP | 事件评分与大段 game.over 替换、结算/战绩 | `../../../archive/upstream/noname-liuli/noname/extension/手杀MVP/extension.js` | `shousha-mvp-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`e02_c3_lifecycle.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs`、`g1_shahai_zhenfa.rs`、`g1_wudian.rs`、`l13_shousha_mvp_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l13-eight` | —；**标签存疑**：行内仍见 不画、录像、界面 |
| L14 手杀ui | 主题/菜单/聊天、动态加载、选择及规则/调试钩子 | `../../../archive/upstream/noname-liuli/noname/extension/手杀ui/extension.js` | — | — | HEAD `5e141764` / 规则 v322 / 存档 v105 / **缺平台结果** | 登记表 open；证据目录 `2026-10-17-remaining-sources-g0` | `不迁移` |
| L15 手杀全扩 | 名称/标签/显示、包分类与禁将选项 | `../../../archive/upstream/noname-liuli/noname/extension/手杀全扩/extension.js` | `shousha-full-upstream-liuli` | —；证据目录：`l15_shousha_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l15-eight` | `登记差异`；**标签存疑**：行内仍见 不画、界面、音频 |
| L16 拖拽读取 | Electron/Node 文件拖放、压缩包导入、JS 探测扩展名 | `../../../archive/upstream/noname-liuli/noname/extension/拖拽读取/extension.js` | `drag-read-upstream-liuli` | —；证据目录：`l16_drag_read_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l16-eight` | `登记差异` |
| L17 斗转星移 | 禁将方案导入导出、选将流程和动态说明覆盖 | `../../../archive/upstream/noname-liuli/noname/extension/斗转星移/extension.js` | `douzhuan-xingyi-upstream-liuli` | —；证据目录：`l17_douzhuan_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l17-eight` | `登记差异`；**标签存疑**：行内仍见 音频 |
| L18 更新保护 | 目录发现、下载/更新源、断点/错误处理及 fs 写入 | `../../../archive/upstream/noname-liuli/noname/extension/更新保护/extension.js` | `update-guard-upstream-liuli` | —；证据目录：`l18_update_guard_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l18-eight` | `登记差异`；**标签存疑**：行内仍见 DOM |
| L19 末日乱斗 | 僵尸等乱斗模式、感染技能、身份/配置与玩家显示补丁 | `../../../archive/upstream/noname-liuli/noname/extension/末日乱斗/extension.js` | `doomsday-brawl-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-17-l19-brawl-batch1`、`2026-10-17-l19-brawl-batch2`、`2026-10-17-remaining-sources-g0` | `deferred` |
| L20 标记补充 | 标记/势力/技能状态显示、动画触发及规则钩子 | `../../../archive/upstream/noname-liuli/noname/extension/标记补充/extension.js` | `mark-supplement-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs`、`host.rs`、`l20_liegong_batch3.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-17-l20-marks-batch1`、`2026-10-17-l20-marks-batch2`、`2026-10-17-l20-marks-batch3`、`2026-10-17-remaining-sources-g0` | `不迁移` |
| L21 武皇争霸 | martial 模式、选池/上阵/隐藏将、双将、回放 | `../../../archive/upstream/noname-liuli/noname/extension/武皇争霸/extension.js` | `martial-upstream-liuli` | —；证据目录：`l21_martial_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l21-eight` | —；**标签存疑**：行内仍见 不画、录像、界面、联机 |
| L22 段位制裁 | 摸牌/牌堆标记、难度、判定/拆顺调整与排位关联 | `../../../archive/upstream/noname-liuli/noname/extension/段位制裁/extension.js` | `elo-sanction-upstream-liuli` | —；证据目录：`l22_elo_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l22-eight` | `登记差异`；**标签存疑**：行内仍见 DOM |
| L23 电脑适配 | 桌面配置预设、扩展开关、导出和临时修复 | `../../../archive/upstream/noname-liuli/noname/extension/电脑适配/extension.js` | `desktop-adapt-upstream-liuli` | —；证据目录：`l23_desktop_adapt.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l23-eight` | `登记差异`；**标签存疑**：行内仍见 DOM、窗口 |
| L24 皮肤切换 | Spine/Worker、出框/攻击/背景动画、参数保存与十周年/千幻互调 | `../../../archive/upstream/noname-liuli/noname/extension/皮肤切换/extension.js` | `skin-switch-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-l24-skin-switch-eight` | —；**标签存疑**：行内仍见 Spine、动画、渲染 |
| L25 祖安武将 | 大量规则增强/技能、元气玩法、对话/特效、编辑器 | `../../../archive/upstream/noname-liuli/noname/extension/祖安武将/extension.js` | — | — | HEAD `5e141764` / 规则 v322 / 存档 v105 / **缺平台结果** | 登记表 open；证据目录 `2026-10-17-remaining-sources-g0` | — |
| L26 笨蛋插件 | 表情、设备/交互辅助及 chooseCard 相关改写 | `../../../archive/upstream/noname-liuli/noname/extension/笨蛋插件/extension.js` | `fool-plugin-upstream-liuli` | —；证据目录：`l26_fool_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l26-eight` | —；**标签存疑**：行内仍见 不画、界面、音频 |
| L27 选将美化 | 选将动画/对话框及 chooseCharacterTwo 等流程覆盖 | `../../../archive/upstream/noname-liuli/noname/extension/选将美化/extension.js` | `select-beautify-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-17-l27-select-beautify-batch1`、`2026-10-17-l27-select-beautify-batch2`、`2026-10-17-l27-select-beautify-batch3`、`2026-10-17-l27-select-beautify-batch4`、`2026-10-17-l27-select-beautify-batch5`、`2026-10-17-l27-select-beautify-g0` | — |
| L28 雷霆万钧 | 新角色/技能、thunder_card、小游戏选择和 UI 补丁 | `../../../archive/upstream/noname-liuli/noname/extension/雷霆万钧/extension.js` | `thunder-upstream-liuli` | —；证据目录：`a0_tag_registry.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs`、`thunder_loader.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-17-l28-thunder-slice1` | `deferred` |
| L29 龙血玄黄 | 深度清单见 G0/L29（原行未单列本栏，2026-10-18 NC11 仅补列结构、未新增结论） | `../../../archive/upstream/noname-liuli/noname/extension/龙血玄黄/extension.js` | `longxue-upstream-liuli`、`loader::longxue` | `l29_longxue_eight.rs`；证据目录：`a0_tag_registry.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-l29-eight` | `登记差异`；**标签存疑**：行内仍见 动画、外部依赖、显示面 |
| U01 3D精选 | 角色/技能、翻译、资源；precontent 导入角色包 | `../../../archive/upstream/noname/apps/core/extension/3D精选/extension.js` | `d3-upstream-noname` | —；证据目录：`a0_tag_registry.rs`、`g1_d3.rs`、`t12_g2g4_channels.rs`、`u01_d3_moyan.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-08-u01-d3-lifecycle`、`2026-10-17-t12-u01-moyan` | `未实现` |
| U02 boss | boss/versus 剑阁角色池、AI禁用与本地缓存 | `../../../archive/upstream/noname/apps/core/extension/boss/extension.js` | `boss-play-upstream-noname`、`loader::boss_playpack` | —；证据目录：`boss_play_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-u02-l03-boss-play-eight` | `登记差异`；**标签存疑**：行内仍见 不画、界面 |
| U03 cardpile | 牌堆比例、花色和随机点数；来源含 `jiu`、`leisha` | `../../../archive/upstream/noname/apps/core/extension/cardpile/extension.js` | `cardpile-upstream-noname` | —；证据目录：`g1_cardpile.rs`、`u03_cardpile_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-08-u03-cardpile-lifecycle`、`2026-10-17-t12-u03-eight` | `登记差异` |
| U04 coin | 本地金币、商店、天气/粒子效果与 chess 分支 | `../../../archive/upstream/noname/apps/core/extension/coin/extension.js` | `coin-upstream-noname` | —；证据目录：`u04_coin_eight.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-17-t12-u04-eight` | `登记差异`；**标签存疑**：行内仍见 录像 |
| U05 杀海拾遗 | gwent/gujian/yunchou/swd/hearth/mtg、zhenfa、wuxing 卡牌子包 + character/ow… | `../../../archive/upstream/noname/apps/core/extension/杀海拾遗/main/precontent.js` | `shaai-ow-upstream-noname`、`shaai-xianjian-upstream-noname`、`shahai-gujian-upstream-noname`、`shahai-gwent-upstream-noname`、`shahai-hearth-upstream-noname`、`shahai-mtg-upstream-noname`、`shahai-swd-upstream-noname`、`shahai-yunchou-upstream-noname`、`shahai-zhenfa-upstream-noname` | —；证据目录：`a0_tag_registry.rs`、`ai_planner.rs`、`archive.rs`、`auth_archive.rs`、`boundary_snapshot.rs`、`catalog.rs`、`e02_c3_lifecycle.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs`、`g1_shahai_zhenfa.rs`、`g1_wudian.rs`、`g1_wuxing.rs`、`host.rs`、`replay_scenarios.rs`、`standard.rs`、`t12_g5_relays.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / **缺平台结果** | 登记表 open；证据目录 `2026-10-08-ow-baolei-batch`、`2026-10-08-ow-duwen-batch`、`2026-10-08-ow-guangshu-batch`、`2026-10-08-ow-luba-batch`、`2026-10-08-ow-luxiao-batch`、`2026-10-08-ow-mei-batch`、`2026-10-08-ow-mode-effects`、`2026-10-08-ow-paotai-batch`、`2026-10-08-ow-shibing-slice`、`2026-10-08-ow-shouhu-batch`、`2026-10-08-ow-ziyu-batch`、`2026-10-08-u05-gujian-lifecycle`、`2026-10-08-u05-hearth-lifecycle`、`2026-10-08-u05-mtg-batch10`、`2026-10-08-u05-mtg-batch2`、`2026-10-08-u05-mtg-batch3`、`2026-10-08-u05-mtg-batch4`、`2026-10-08-u05-mtg-batch5`、`2026-10-08-u05-mtg-batch6`、`2026-10-08-u05-mtg-batch7`、`2026-10-08-u05-mtg-batch8`、`2026-10-08-u05-mtg-batch9`、`2026-10-08-u05-mtg-slice`、`2026-10-08-u05-swd-batch12`、`2026-10-08-u05-swd-batch13`、`2026-10-08-u05-swd-batch14`、`2026-10-08-u05-swd-batch15`、`2026-10-08-u05-swd-slice`、`2026-10-08-u05-wuxing-lifecycle`、`2026-10-08-u05-yunchou-lifecycle`、`2026-10-08-u05-zhenfa-slice`、`2026-10-09-ow-ana-batch`、`2026-10-09-ow-anu-batch`、`2026-10-09-ow-banzang-batch`、`2026-10-09-ow-closeout-xianjian-slice`、`2026-10-09-ow-dva-batch`、`2026-10-09-ow-falaozhiying-batch`、`2026-10-09-ow-hanlingsha-batch`、`2026-10-09-ow-heiying-batch`、`2026-10-09-ow-kuangshu-batch`、`2026-10-09-ow-laiyinhate-batch`、`2026-10-09-ow-leiyuange-batch`、`2026-10-09-ow-linyueru-batch`、`2026-10-09-ow-lixiaoyao-batch`、`2026-10-09-ow-longkui-batch`、`2026-10-09-ow-maikelei-batch`、`2026-10-09-ow-muchanglan-batch`、`2026-10-09-ow-orisa-batch`、`2026-10-09-ow-shenqishuang-batch`、`2026-10-09-ow-sishen-batch`、`2026-10-09-ow-tangyurou-batch`、`2026-10-09-ow-wangpengxu-batch`、`2026-10-09-ow-wenhui-batch`、`2026-10-09-ow-wensidun-batch`、`2026-10-09-ow-xia-batch`、`2026-10-09-ow-xuanxiao-batch`、`2026-10-09-ow-yuanshi-batch`、`2026-10-09-ow-zhaliya-batch`、`2026-10-09-ow-zhaoliner-batch`、`2026-10-09-xianjian-changnian-batch`、`2026-10-09-xianjian-chengxin-batch`、`2026-10-09-xianjian-fenglue-batch`、`2026-10-09-xianjian-husha-batch`、`2026-10-09-xianjian-jiangcheng-batch`、`2026-10-09-xianjian-jingtian-batch`、`2026-10-09-xianjian-liefeng-batch`、`2026-10-09-xianjian-liguang-batch`、`2026-10-09-xianjian-liumengli-batch`、`2026-10-09-xianjian-longyou-batch`、`2026-10-09-xianjian-mingxiu-batch`、`2026-10-09-xianjian-poyun-batch`、`2026-10-09-xianjian-sheling-batch`、`2026-10-09-xianjian-shiying-batch`、`2026-10-09-xianjian-sumei-batch`、`2026-10-09-xianjian-xiaoman-batch`、`2026-10-09-xianjian-xshuangren-batch`、`2026-10-09-xianjian-xtanxi-batch`、`2026-10-09-xianjian-xuanmo-batch`、`2026-10-09-xianjian-xuejian-batch`、`2026-10-09-xianjian-yuntianhe-batch`、`2026-10-09-xianjian-zhaoyao-batch`、`2026-10-09-xianjian-zhimeng-batch`、`2026-10-09-xianjian-zixuan-batch`、`2026-10-10-replay-golden-v192-recovery`、`2026-10-10-xianjian-ctianfu-batch`、`2026-10-10-xianjian-luanjian-batch`、`2026-10-10-xianjian-qianfang-batch`、`2026-10-10-xianjian-tianxian-batch`、`2026-10-10-xianjian-xiepan-batch`、`2026-10-10-xianjian-xunying-batch`、`2026-10-10-xianjian-yingfeng-batch`、`2026-10-10-xianjian-zhangmu-batch`、`2026-10-11-t12-jiguanfeng-v195`、`2026-10-11-t12-jiguanshu-v198`、`2026-10-11-t12-jiguanyaoshu-v197`、`2026-10-11-t12-jiguanyuan-v196`、`2026-10-11-t18-jiguanshu-primitive-v199`、`2026-10-14-t12-g5-batch2`、`2026-10-14-t12-yunchou-generals-b1`、`2026-10-14-t12-yunchou-generals-b18`、`2026-10-14-t12-yunchou-generals-b2`、`2026-10-14-t12-yunchou-generals-b3`、`2026-10-14-t12-yunchou-generals-b4`、`2026-10-14-t12-yunchou-generals-b5`、`2026-10-14-t12-yunchou-generals-b6`、`2026-10-14-t12-yunchou-generals-b7`、`2026-10-14-t12-yunchou-generals-b8`、`2026-10-14-t12-yunchou-generals-b9`、`2026-10-17-t12-mtg-lands-deferred` | `deferred`、`不迁移`、`暂缓`、`未实现`、`登记差异` |
| U06 欢乐卡牌 | 自定义装备/技能与资源 | `../../../archive/upstream/noname/apps/core/extension/欢乐卡牌/extension.js` | `huanle-upstream-noname` | —；证据目录：`a0_tag_registry.rs`、`ai_planner.rs`、`auth_archive.rs`、`boundary_snapshot.rs`、`e02_c3_lifecycle.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs`、`g1_shahai_zhenfa.rs`、`g1_wudian.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 resolved；证据目录 `2026-10-08-e03-package-acceptance`、`2026-10-14-t12-huanle-g5`、`2026-10-14-t12-u06-g2g4`、`2026-10-17-t12-u06-eight` | `登记差异`；**标签存疑**：行内仍见 界面、窗口 |
| U07 玩点论杀 | 角色、技能、卡牌及模块化资源 | `../../../archive/upstream/noname/apps/core/extension/玩点论杀/extension.js` | `wudian-upstream-noname` | —；证据目录：`a0_tag_registry.rs`、`archive.rs`、`g1_wudian.rs`、`host.rs`、`replay_scenarios.rs`、`standard.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / **缺平台结果** | 登记表 open；证据目录 `2026-10-08-u07-deferral-closeout`、`2026-10-08-u07-skill-batch1`、`2026-10-08-u07-skill-batch2`、`2026-10-08-u07-skill-batch3`、`2026-10-08-u07-skill-batch4`、`2026-10-08-u07-skill-batch5`、`2026-10-08-u07-skill-batch6`、`2026-10-08-u07-skill-batch7`、`2026-10-08-u07-wudian-lifecycle`、`2026-10-17-t12-deferred-registration` | `deferred`、`暂缓`、`登记差异` |
| U08 英雄杀 | 角色/技能与翻译、资源 | `../../../archive/upstream/noname/apps/core/extension/英雄杀/extension.js` | `hero-kill-upstream-noname` | —；证据目录：`a0_tag_registry.rs`、`ai_content_tuning.rs`、`ai_planner.rs`、`auth_archive.rs`、`boss_play_eight.rs`、`boundary_snapshot.rs`、`d5_longxue_mbqianlong.rs`、`e02_c3_lifecycle.rs`、`epic_rangjie_after_damage.rs`、`epic_reguhuo_viewas.rs`、`g1_cardpile.rs`、`g1_d3.rs`、`g1_hearth.rs`、`g1_huanle.rs`、`g1_shahai_gujian.rs`、`g1_shahai_gwent.rs`、`g1_shahai_mtg.rs`、`g1_shahai_swd.rs`、`g1_shahai_yunchou.rs`、`u08_hero_kill_batch1.rs` | HEAD `5e141764` / 规则 v322 / 存档 v105 / 有平台线索 | 登记表 open；证据目录 `2026-10-17-remaining-sources-g0`、`2026-10-17-u08-hero-kill-batch1`、`2026-10-17-u08-hero-kill-batch2` | `deferred` |

## 证据缺失清单（脚本实测，逐来源列出）

### 缺上游证据（0 个来源）

- 无。

### 缺生产接口（5 个来源）

- **L08 十周年UI**：行内未给出转换包目录；另：行内未给出装载器模块名（数据型包可不要求）
- **L11 太虚幻境**：行内未给出转换包目录；另：行内未给出装载器模块名（数据型包可不要求）
- **L12 如真似幻**：行内未给出转换包目录；另：行内未给出装载器模块名（数据型包可不要求）
- **L14 手杀ui**：行内未给出转换包目录；另：行内未给出装载器模块名（数据型包可不要求）
- **L25 祖安武将**：行内未给出转换包目录；另：行内未给出装载器模块名（数据型包可不要求）

### 缺测试案例（4 个来源）

- **L08 十周年UI**：行内未给出测试文件名，且引用证据目录中未出现测试名
- **L11 太虚幻境**：行内未给出测试文件名，且引用证据目录中未出现测试名
- **L14 手杀ui**：行内未给出测试文件名，且引用证据目录中未出现测试名
- **L25 祖安武将**：行内未给出测试文件名，且引用证据目录中未出现测试名

### 缺平台结果（6 个来源）

- **U05 杀海拾遗**：行内与证据目录均无平台/真实客户端关键词线索
- **U07 玩点论杀**：行内与证据目录均无平台/真实客户端关键词线索
- **L08 十周年UI**：行内与证据目录均无平台/真实客户端关键词线索
- **L11 太虚幻境**：行内与证据目录均无平台/真实客户端关键词线索
- **L14 手杀ui**：行内与证据目录均无平台/真实客户端关键词线索
- **L25 祖安武将**：行内与证据目录均无平台/真实客户端关键词线索

## 整包标签存疑清单（标签 ≠ 验收）

以下来源在台账中带「八项整包」标签，但同一行/同一证据内**仍登记未迁移的功能、表现或平台面**（方案阶段 5 与 AGENTS.md 要求这类面完成验收后才可上调整包）：

- **U02 boss**（登记表 resolved）：行内仍见 不画、界面；未覆盖标记 登记差异
- **U04 coin**（登记表 resolved）：行内仍见 录像；未覆盖标记 登记差异
- **U06 欢乐卡牌**（登记表 resolved）：行内仍见 界面、窗口；未覆盖标记 登记差异
- **L03 boss**（登记表 resolved）：行内仍见 不画、界面；未覆盖标记 登记差异
- **L05 coin**（登记表 resolved）：行内仍见 录像；未覆盖标记 登记差异
- **L06 wuxing**（登记表 resolved）：行内仍见 帮助、录像、联机；未覆盖标记 登记差异
- **L07 全能搜索**（登记表 resolved）：行内仍见 界面、语音；未覆盖标记 登记差异
- **L09 千幻聆音**（登记表 resolved）：行内仍见 插件系统、语音；未覆盖标记 —
- **L10 史诗卡牌**（登记表 resolved）：行内仍见 阻塞；未覆盖标记 不迁移、阻塞
- **L13 手杀MVP**（登记表 resolved）：行内仍见 不画、录像、界面；未覆盖标记 —
- **L15 手杀全扩**（登记表 resolved）：行内仍见 不画、界面、音频；未覆盖标记 登记差异
- **L17 斗转星移**（登记表 resolved）：行内仍见 音频；未覆盖标记 登记差异
- **L18 更新保护**（登记表 resolved）：行内仍见 DOM；未覆盖标记 登记差异
- **L21 武皇争霸**（登记表 resolved）：行内仍见 不画、录像、界面、联机；未覆盖标记 —
- **L22 段位制裁**（登记表 resolved）：行内仍见 DOM；未覆盖标记 登记差异
- **L23 电脑适配**（登记表 resolved）：行内仍见 DOM、窗口；未覆盖标记 登记差异
- **L24 皮肤切换**（登记表 resolved）：行内仍见 Spine、动画、渲染；未覆盖标记 —
- **L26 笨蛋插件**（登记表 resolved）：行内仍见 不画、界面、音频；未覆盖标记 —
- **L29 龙血玄黄**（登记表 resolved）：行内仍见 动画、外部依赖、显示面；未覆盖标记 登记差异

## 判据与禁止事项

- 本表**不产出完成结论**：`结果` 列只写登记表状态与证据目录是否在盘；「八项整包」字符串、包目录或测试文件存在均不构成完成判定（NC11 明文禁止）。
- 「平台结果」为关键词判据（wasm/浏览器/冒烟/真实客户端/平台/设备等），命中只说明有平台线索，**不等于**平台验收通过。
- 「未覆盖项」只提取行内已登记的标记词（登记差异、未迁移、不迁移、deferred、暂缓、未实现、阻塞、未验收、递延 等），不代表未覆盖面的完整清单；完整清单以各行证据与 G0 文档为准。
- 标签语义：`G0 完成` 表示**该来源**的原生重构只做到 G0 深度清单（不是「37 来源 G0 清点」这件事）；`八项整包` 表示台账/登记表把它登记为整包范围，**不表示八项验收已通过**。
- 台账状态或登记表机器字段变化后重跑本脚本，不要手改计数。
