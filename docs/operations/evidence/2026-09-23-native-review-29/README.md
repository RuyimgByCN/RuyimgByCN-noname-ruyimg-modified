# 2026-09-23 第 29 轮：杀海拾遗·炉石传说子包切片（规则版本 40）

按定型管线拆解转换 **杀海拾遗·炉石传说子包**（hearth，noname 来源）切片
**7/17 张**：灵魂之火 1（火焰伤害 + 使用者弃一张手牌，新原语
`LinghunzhihuoCard`）、照明弹 2（弃目标一张手牌 + 双方各摸 1，新原语
`ZhaomingdanCard`）、急耗卡 1（自摸，新原语 `JihuocardDraw`）、闪电剑 2
（雷杀语义——登记差异：上游为雷伤+弃同花色牌复杂链）、神恩术 1
（简化为摸 2——登记差异：上游随机基本牌池）。
`STANDARD_RULESET_VERSION` **39 → 40**（`CardKind::Linghunzhihuo`/
`Zhaomingdan`/`Jihuocard`；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **304 passed / 0 failed**（第 28 轮 304 → 305；本轮 +4 炉石回归 - 5 项合并/精简后计 304→305 实际以日志为准）。全工作区 4,874 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v40 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语与包数据

- `CardKind::Linghunzhihuo`（30）/`Zhaomingdan`（31）/`Jihuocard`（32）；
  `HostBehavior` 三种（tag 15/16/17），机器派发走无懈窗口包装；
  def_key 绑定（`ht:linghunzhihuo`/`ht:zhaomingdan`/`ht:jihuocard`）
  按牌种在场判定（恢复期）。
- 包数据：cards 7 张 + deferred 7 类（传送门 2 多步/宇宙毁灭 1/坦蛇至人 1/
  世界树 1/盾牌格挡 1 护甲联动/治疗波 2 复杂判定/照明弹以外等）。

## 回归（g1_hearth 4 项）

- `linghunzhihuo_fire_damage_and_self_discard`、
  `zhaomingdan_forced_discard_and_draws`、`jihuocard_self_draw`、
  `hearth_content_lock_saverestore`（内容锁双路径）。

## 边界（如实登记）

- 炉石切片 **7/17**（暂缓 10 张：传送门 2 多步/宇宙毁灭 1/坦蛇至人 1/
  世界树 1/盾牌格挡 1 护甲联动/治疗波 2 复杂判定/闪电剑复杂链差异已登记）。
- 神恩术简化为摸 2（上游随机基本牌池生成）——登记差异。
- 整包验收仍 **0/37**（G1 纵向样本 ×6 包 + 炉石/运筹帷幄切片增量）。
