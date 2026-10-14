# T12 swd 卡牌面补切批⑦：锦囊族 4 kind 7 张（2026-10-15）

## 承载

4 个新 trick kind（tag 150-153；包 60→**65 张**）：

- **惊雷山 jingleishan**（club7 thunder）：AOE 全体其他需出杀，不出雷
  1 伤——雷版南蛮。AoeResolveMachine/AoeTargetMachine 加 `nature` 字段
  （DamageNature，MachineSnapshot::Aoe/AoeTarget 变体扩展：snap 枚举+
  archive enc/dec+restore+canonical hash 五处），damage_data_with 直写
  雷属性（铁索传导天然支持）。**字段批**：存档 v71→**v72**、规则
  v284→**v285**、golden 14 场重录（284+字段探针逐字节一致零扰动）。
- **炽羽袭 chiyuxi**（diamond7 fire）：火版万箭（需出闪，火 1 伤）。
- **鬼蛇旗 guisheqi**（spade9+club7）：单目标 +1 护甲（卡牌机
  GuisheqiCard，change_hujia 原语）。
- **仙露回 xianluhui**（spade13）：AOE 全体受伤角色（含自己）+1 护甲
  （卡牌机 XianluhuiCard；reverseOrder 逆座位序近似登记）。
- 神草结（damageBefore ±1 交互锦囊）留批⑧——g_shencaojie 依赖伤害事件
  改写方向选择面。

## 发牌敏感回归一处

指路小虎测试 `position(kind==Jiguan)` 首实体在批①对齐后可能已在发牌手
牌——仅 deck.retain 造成双区（restore 守卫拦）——改全域摘除。

## 测试（swd_cards_trick_b37_jingleishan_chiyuxi_guisheqi_xianluhui）

| 面 | 断言 |
|---|---|
| 鬼蛇旗 | p2 +1 护甲 |
| 惊雷山 | p1/p3 雷 1 伤；p2 有护甲抵消（hp 不动护甲清零） |
| 炽羽袭 | p1 火 1 伤；p2 无护甲 4→3；p3 火 1 伤 |
| 仙露回 | 全体受伤者（p1/p2/p3）各 +1 护甲；满血 p0 不受益 |

## 实测记录

| 项 | 结果 |
|---|---|
| 定向 g1_shahai_swd | **46/46** |
| engine-native 全量 | **765/0/0**（golden v285+存档 v72 往返）——[日志](engine-native-tests.log) |
| Gate 1b | exit 0（a0 探针夹具+冻结键数 298 由全量/Gate 抓漏补齐）——[日志](gate1b-all-targets.log) |

## 登记边界与下一步

- 包 65/92 行；余 27 行（equip5 余 6+锦囊神草结 3+器 13+神秘果 2+多余
  13 处置）。
- 批⑧ 候选：器族 13（jiqi 四象五件+玉蝉八卦 8）或 equip5 余 6。
