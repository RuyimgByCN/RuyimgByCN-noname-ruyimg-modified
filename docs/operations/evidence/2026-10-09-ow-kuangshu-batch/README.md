# U05 ow 纵切批十六：狂鼠拾火（规则 v133，2026-10-09）

ow 纵切批十六：**狂鼠**（ow_kuangshu，shu 3 男）——拾火（shihuo，
global 伤害后触发锁定）单技闭环（上游 160 字符最简技）；**liudan 暂缓**
（杀使用时 50% 概率随机追加合法目标——RNG 目标追加+targetEnabled 复
用）+ **shoujia 暂缓**（给牌+shoujia2 挂牌子技）。上游 25 将已闭环第
15 将。前置：
[ow 纵切批十五](../2026-10-09-ow-maikelei-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js shihuo）

- trigger global damageEnd forced：filter event.hasNature("fire") →
  player.draw()。任何人受到火属性伤害后持有者摸 1；每次火伤各触发
  （传导的独立事件照常；damageEnd 仅实际伤害后事件——被防止/零量不触
  发）。

## 改动（分开陈述）

- **代码已实现**：`WudianShihuo`（tag 130，Damage after 窗口触发、
  optional:false=上游 forced 同口径）——filter（火属性 && 未防止 &&
  有量——prevented/零量不触发=上游 damageEnd 事件口径登记）→ Act：
  摸 1（率言 global 触发先例+飞弹/神枪 Damage after 同位）。
- **登记差异**：无新增（火性判定=DamageNature::Fire 一一对应；传导的
  独立火伤事件照常触发=上游每事件 damageEnd）。
- **登记暂缓**：liudan（RNG 目标追加）、shoujia（挂牌子技）。
- **转换包**：cards.json 增 狂鼠（shu 3 男，shihuo mapping+双暂缓
  reason）+ npack 资产哈希重算。无新状态字段——存档 v9 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`kuangshu_shihuo_fire_damage_draw`
  （火杀命中后摸 1/普通杀不触发）。
- **全量通过**：engine-native **454 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v132 基线 453 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 132→133 入 canonical_hash——14 场景全 mismatch
  （纯版本位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：连出火杀+普通杀需先装诸葛连弩（杀限次第三次命
  中此坑）；手牌断言用增量口径（火杀放置尝试会在手牌残留非火杀）。

## 剩余

- ow 其余 10 武将逐武将纵切；xianjian 36 武将；虚拟 useCard 原语
  （huoyu/youzhan/dianji 同源）、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
