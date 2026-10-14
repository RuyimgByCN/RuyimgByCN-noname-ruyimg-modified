# T12 swd 卡牌面补切批①：对账勘误+纯数据补张（45→50 张）（2026-10-15）

## 承载（纯数据，无新 CardKind、无行为接线）

按对账批勘误清单将 7 kind 与上游 list.js 精确对齐：

| 卡 | 变更 |
|---|---|
| hufu 护符 | swd_hufu2 club1 → **heart1**（上游 heart1+spade1） |
| zhiluxiaohu 逐鹿小虎 | club4 → **heart3**，新增 **diamond4**（2 张，jiguan 分流 contains 兼容尾数字） |
| mujiaren 木甲人 | heart4 → **club7**，新增 **heart6 / diamond11**（3 张） |
| shentoumianju 潜偷面具 | spade6 → **club6** |
| xingjunyan | spade8 → **spade7** |
| zhufangshenshi 逐方绅视 | club7 → **spade4**，新增 **spade8** |
| yangpijuan 羊皮卷 | 保留 spade9，新增 **club7**（上游 2 张） |

包 45→**50 张**；目录三处 `base_cards + 45` 断言 → +50。

## 对账守护测试（swd_cards_alignment_guard_b29）

13 张 (名, 花色, 点数) 三元组逐一断言在包（def_name 以 `swd_<名>` 子串+
花色/点数判定——loader 加前缀序号名不可精确等值）+ 总张数 50 冻结（批②+
随补切递增并同步）。

## 发牌敏感回归两处（对账批预告的坑全中）

- 旋风之刃随机弃牌与 RNG 流耦合：`swd_lingjian_xuanfeng` 原依赖随机掷中
  装备面（时间回溯校验装备区非空）——改为缺装备时确定性重注。
- 机关妖术补偿耦合随机掷中装备面（hand 弃置静默不发 CardsDiscarded=登记
  近似）——改为 ys 后清空 p1 手牌，xf 随机池仅剩装备区 1 件，弃装备路径
  必发事件补偿确定性触发。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **40/40**（新增守护测试） |
| engine-native 全量 | **759/0/0**（golden 不变——扩展卡牌不入标准卡堆）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 50/92 行对齐；余 42 行缺失（basic 5/锦囊 8/器 13/equip5 13/equip2 3/
  equip1 3——均需新 CardKind 或技能，随批②-⑤ 承接）+ 多余 13 处置待决。
- 批② 候选：basic 族（雪霁冰雹×2/神秘果×2/封印之丹——3 新 kind basic 规则面）。
