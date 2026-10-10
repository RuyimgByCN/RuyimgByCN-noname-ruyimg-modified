# 2026-09-23 第 38 轮：护甲引擎原语 + 古剑食物第三批 3 张（规则版本 49）

古剑子包续切片 **14/40 → 17/40**，本轮核心为**护甲引擎原语**（此前
登记的引擎缺口， unlocks 鲤鱼汤及后续 zi­yangdan/shatang/heilonglinpian
等 4+ 张跨簇卡牌）：

- **护甲字段**：`PlayerState.hujia: u8`（入 canonical 哈希与字节存档
  ——状态编码变化，版本递增）；`commit_damage` 中 1:1 先行抵扣伤害
  （护甲归零后剩余伤害入体力；全部抵消则不扣体力、无濒死）；
  `LogEvent::HujiaAbsorbed`（表现层投影为 Recovered）；
  `ctx.change_hujia(player, ±delta)` 有界辅助（下限 0）。
- **鲤鱼汤**（`GujianFoodLiyutang`，tag 33，×1，2 回合）：阶段开始时
  若体力值为全场最少或之一，获得 1 点护甲。
- **麻茶**（`GujianFoodMolicha`，tag 34，×1，3 回合）：不能成为其他
  角色**黑色牌**的目标——**出牌目标校验钩子**（PlayCard 路径按物理
  花色过滤；登记差异：不含响应路径）。
- **菱角**（`GujianFoodXiajiao`，tag 35，×1，4 回合）：摸牌阶段额外
  摸一张牌然后弃置一张牌——触发机 draw + `WaitChoice` 选择机
  （SelectCard 必选弃 1），挂起态入快照派生（字节恢复贯通）。

`STANDARD_RULESET_VERSION` **48 → 49**（`CardKind::Liyutang`/`Molicha`/
`Xiajiao` tag 48–50；PlayerState 编码变化；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **322 passed / 0 failed**（第 37 轮 321 + 护甲/食物 1 项）。全工作区 4,888 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v49 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 回归（g1_shahai_gujian 增补 1 项 + 切片计数更新）

- `hujia_primitive_and_food_batch_three`：麻茶——黑杀指定持有者被拒、
  红杀不受限（走完整杀结算）；鲤鱼汤——体力并列最少结束阶段 +1 护
  甲、1 号回合黑杀命中 → 护甲 -1 体力不变（抵扣生效）+ 护甲归零；
  菱角——新回合摸牌阶段额外摸 1 后弃 1 挂起、**挂起点字节恢复**、
  净 +1；全区牌归属守恒。
- 切片计数：17/40（+食物 3），deferred 对账 23。

## 边界（如实登记）

- 食物簇余 **2 张**暂缓：蜜链藕（红桃当桃 view-as——需濒死响应路径
  的状态驱动 view-as）、卤玉羹（发现牌——引擎缺口，可选确定性简化
  待定）。
- 护甲登记差异：原生 `commit_damage` 统一抵扣（含属性伤害/链传导
  前的本体伤害）；上游 `changeHujia` 语义一致，但护甲与濒死交互
  （护甲不为负、全部抵消不进濒死）按原生管线实现。
- 麻茶登记差异：仅出牌使用路径过滤（响应路径如决斗杀不校验）。
- 整包验收仍 **0/37**。
