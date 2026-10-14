# T12 swd 卡牌面补切批⑧：龟灵之桃——逆攻三面（2026-10-15）

## 承载

**龟灵之桃 guilingzhitao**（新 kind tag 154，equip5 宝物栏，spade7 入包）
+ 两装备技（tags 299/300；同批将批⑦卡牌机 GuisheqiCard/XianluhuiCard
tag 重编号 301/302 避撞）：

- **逆攻积攒 nigong_accumulate**（tag 299，Damage forced）：受伤标记
  +amount 上限 4（`PlayerState.nigong_lv` u8）。
- **逆攻发动 nigong_strike**（tag 300，出牌主动技）：lv=0 拒；lv=1 直发
  摸 1 清零；lv≥2 TargetChoice 选其他角色 → 弃全部标记+奇数摸 1+
  floor(lv/2) 伤。
- 字段批：`nigong_lv` u8——存档 v72→**v73**；规则 v285→**v286**；golden
  14 场重录（**285+字段探针与旧 golden 逐字节一致**——位移唯一来源版本
  字段，行为面零扰动）。
- tag 撞号勘误：批⑦卡牌机占 297/298，龟灵之桃 299/300、卡牌机重编
  301/302（tags_unique 往返守护抓漏）。

## 测试（swd_cards_equip5_b38_guilingzhitao_nigong）

| 面 | 断言 |
|---|---|
| 积攒 | 受伤 2 → 标记 2（forced 无窗） |
| 发动偶数 | 弃全部标记+floor(2/2)=1 伤、不摸 |
| 反例 | 标记 0 发动拒 |
| 发动奇数 | 受伤 3 → 摸 1+floor(3/2)=1 伤 |
| 上限 | 受伤 9 → 标记钳 4 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **47/47** |
| engine-native 全量 | **766/0/0**（golden v286+存档 v73 往返）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 66/92 行；equip5 余 5 kind（书锤/散灯金环/昊天塔/玉如意/四非珍面）
  +锦囊神草结 3+器 13+神秘果 2+多余 13 处置。
- 批⑨ 候选：器族或余下 equip5 中复杂组。
