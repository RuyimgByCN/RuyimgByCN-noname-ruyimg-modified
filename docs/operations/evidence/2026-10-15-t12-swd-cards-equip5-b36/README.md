# T12 swd 卡牌面补切批⑥：空洞印——死亡取消型宝物（2026-10-15）

## 承载

**空洞印 kongdongyin**（新 kind tag 149，equip5 宝物栏，heart13 入包；
守卫技 tag 296）：Death 前置窗口（plan_triggers before_kind=Death 原生通
道，Timing::First forced）锁定取消自己的死亡——`death_cancelled` 置位
（v171 死亡取消型基建，DeathMachine 开头拦截）+ hp=1 + 摸 1 + 弃全部宝
物（含自身，revoke_equip_skills 同步撤销守卫技=一次性；上游 dieBefore
trigger.cancel 同口径）。

## 基建缺口修复（本批最有价值发现）

**死亡取消路径不清 `dying` 旗标**：v171 基建 DeathMachine cancelled 分
支只消费 death_cancelled 就返回——hp 由前置技重置但 dying=true 残留，后
续致死伤害被「已濒死」挡住**永不再进濒死/死亡流程**（空洞印一次性后玩
家永久卡 -8 体力存活）。修复：cancelled 分支同步 `dying=false`（脱离濒
死）。该缺口对全部死亡取消型用户（空洞印是首个）生效。

golden 不变、无版本 bump（行为门控）。

## 测试（swd_cards_equip5_b36_kongdongyin_death_cancel）

| 面 | 断言 |
|---|---|
| 正例 | hp1 吃 4 伤濒死无救 → 死亡取消：存活+hp=1+摸 1+两只宝物全弃入弃堆 |
| 反例 | 一次性——再次致死（9 伤）正常死亡 |

测试侧勘误三处：equip_slot 调用路径（kind 方法）、UnknownEquipSkill=
kind_host_skills 漏注册臂、冻结键数 295→296（全量抓漏）。

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **45/45** |
| engine-native 全量 | **764/0/0**（golden 不变）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 60/92 行；equip5 余 6 kind（书锤/散灯金环/昊天塔/玉如意/龟灵之桃/
  四非珍面）+锦囊 8+器 13+神秘果 2+多余 13 处置。
- 批⑦ 候选：锦囊族 8 张或 equip5 中复杂组。
