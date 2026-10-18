# T9 来源行为样本与接口登记表（2026-10-17，实测重建）

- 口径：**37 来源**逐行登记「台账状态 / 动态样本证据目录 / 测试入口」。
- 本表由台账 + 证据目录 + 测试文件**实测生成**（脚本核对：每个非「待重构」来源均有证据目录与测试入口）。
- 计数：**整包 22 / 部分重构 4 / 待重构 11**（合计 37）。

| 来源 | 名称 | 台账状态 | 动态样本证据（目录） | 测试入口 |
|---|---|---|---|---|
| L01 | AI优化 | 整包 | `2026-10-17-l01-ai-tuning-eight`, `2026-10-17-l01-ai-tuning-slice1`, `2026-10-17-l01-ai-tuning-slice2` | `ai_tuning_loader.rs`, `l01_ai_tuning_eight.rs` |
| L02 | SJ Settings | 整包 | `2026-10-17-t12-l02-eight` | `l02_sj_eight.rs` |
| L03 | boss | 整包 | `2026-10-17-t12-u02-l03-boss-play-eight` | `boss_play_eight.rs` |
| L04 | cardpile | 整包 | `2026-10-08-l04-cardpile-liuli-lifecycle`, `2026-10-17-t12-l04-eight` | `g1_cardpile.rs`, `l04_cardpile_eight.rs`, `u03_cardpile_eight.rs` |
| L05 | coin | 整包 | `2026-10-17-t12-l05-eight` | `l05_coin_eight.rs`, `u04_coin_eight.rs` |
| L06 | wuxing | 整包 | `2026-10-17-t12-l06-eight` | `g1_wuxing.rs` |
| L07 | 全能搜索 | 整包 | `2026-10-17-t12-l07-eight` | `l07_omnisearch.rs` |
| L08 | 十周年UI | 待重构 | —（待补） | —（待补） |
| L09 | 千幻聆音 | 待重构 | —（待补） | —（待补） |
| L10 | 史诗卡牌 | 整包 | `2026-10-17-l10-epic-cards-slice1`, `2026-10-17-l10-epic-cards-slice2`, `2026-10-17-l10-epic-cards-slice3` | `epic_cards_loader.rs`, `epic_rangjie_after_damage.rs`, `epic_reguhuo_viewas.rs`, `l10_epic_cards_eight.rs` |
| L11 | 太虚幻境 | 待重构 | —（待补） | —（待补） |
| L12 | 如真似幻 | 待重构 | —（待补） | —（待补） |
| L13 | 手杀MVP | 整包 | `2026-10-17-t12-l13-eight` | `l13_shousha_mvp_eight.rs` |
| L14 | 手杀ui | 待重构 | —（待补） | —（待补） |
| L15 | 手杀全扩 | 整包 | `2026-10-17-t12-l15-eight` | `l15_shousha_eight.rs` |
| L16 | 拖拽读取 | 整包 | `2026-10-17-t12-l16-eight` | `l16_drag_read_eight.rs` |
| L17 | 斗转星移 | 整包 | `2026-10-17-t12-l17-eight` | `l17_douzhuan_eight.rs` |
| L18 | 更新保护 | 整包 | `2026-10-17-t12-l18-eight` | `l18_update_guard_eight.rs` |
| L19 | 末日乱斗 | 待重构 | —（待补） | —（待补） |
| L20 | 标记补充 | 待重构 | —（待补） | —（待补） |
| L21 | 武皇争霸 | 整包 | `2026-10-17-t12-l21-eight` | `l21_martial_eight.rs` |
| L22 | 段位制裁 | 整包 | `2026-10-17-t12-l22-eight` | `l22_elo_eight.rs` |
| L23 | 电脑适配 | 整包 | `2026-10-17-t12-l23-eight` | `l23_desktop_adapt.rs` |
| L24 | 皮肤切换 | 待重构 | —（待补） | —（待补） |
| L25 | 祖安武将 | 待重构 | —（待补） | —（待补） |
| L26 | 笨蛋插件 | 整包 | `2026-10-17-t12-l26-eight` | `l26_fool_eight.rs` |
| L27 | 选将美化 | 待重构 | —（待补） | —（待补） |
| L28 | 雷霆万钧 | 部分 | `2026-10-17-l28-thunder-slice1` | `thunder_loader.rs` |
| L29 | 龙血玄黄 | 整包 | `2026-10-17-t12-l29-eight` | `d5_longxue_mbqianlong.rs`, `l29_longxue_eight.rs` |
| U01 | 3D精选 | 部分 | `2026-10-08-u01-d3-lifecycle`, `2026-10-17-t12-u01-moyan` | `u01_d3_moyan.rs` |
| U02 | boss | 整包 | `2026-10-17-t12-u02-l03-boss-play-eight` | `boss_play_eight.rs` |
| U03 | cardpile | 整包 | `2026-10-08-u03-cardpile-lifecycle`, `2026-10-17-t12-u03-eight` | `g1_cardpile.rs`, `l04_cardpile_eight.rs`, `u03_cardpile_eight.rs` |
| U04 | coin | 整包 | `2026-10-17-t12-u04-eight` | `l05_coin_eight.rs`, `u04_coin_eight.rs` |
| U05 | 杀海拾遗 | 部分 | `2026-10-08-u05-wuxing-lifecycle`, `2026-10-08-u05-yunchou-lifecycle`, `2026-10-08-u05-zhenfa-slice` | `g1_wudian.rs`, `swd_bingfeng_lock.rs`, `swd_cyqiaoxie_jiguan.rs`, `swd_gaizao_slot.rs` |
| U06 | 欢乐卡牌 | 整包 | `2026-10-07-u06-combo-matrix`, `2026-10-14-t12-u06-g2g4`, `2026-10-17-t12-u06-eight` | `g1_huanle.rs` |
| U07 | 玩点论杀 | 部分 | `2026-10-08-u07-skill-batch6`, `2026-10-08-u07-skill-batch7`, `2026-10-08-u07-wudian-lifecycle` | `g1_wudian.rs` |
| U08 | 英雄杀 | 待重构 | —（待补） | —（待补） |

## 说明

- 「动态样本」= 该来源在原生侧的可复跑证据目录（含定向日志/八项用例日志）。
- 「测试入口」= 覆盖该来源的测试文件（八项用例或装载器/定向用例）。
- 「待重构」来源尚无原生实现，故样本与测试入口为空——G0 清点结论见台账各行。
- 本表替代 2026-10-17 早前**空文件**占位（原 README 为 0 字节、其中的「23/37」无据）；现值以实测为准。
