# 2026-09-23 第 36 轮：古剑食物计时状态原语——摊黄冻/有更（规则版本 47）

古剑子包续切片 **7/40 → 9/40**：落地**食物计时状态原语**（同簇 12 张
的前 2 张，共享同一框架）——

- **框架**：`SkillInstance` 新增 `rounds`（剩余回合，0=非计时）字段，
  入 canonical 哈希与字节存档（v47 状态编码变化）；食物牌使用 →
  `GujianFoodGrant` 授予机（`HostBehavior` tag 27）给目标安装监听技
  实例（rounds=上游 markcount）并弃置食物牌；监听技在**效果触发点
  扣减**倒计时，归零移除实例（登记差异：上游部分食物在另一阶段
  扣减，原生统一合并到效果触发点）。
- **摊黄冻**（`GujianFoodTanhuadong`，tag 25，×1）：出牌阶段结束时
  摸一张牌，3 回合。
- **有更**（`GujianFoodYougeng`，tag 26，×1）：阶段开始时若已受伤且
  体力值为全场最少或之一，回复 1 点体力，3 回合（登记差异：原生
  无准备阶段，挂判定阶段开始）。

`STANDARD_RULESET_VERSION` **46 → 47**（`CardKind::Tanhuadong`/
`Yougeng`，tag 41/42；SkillInstance 编码变化；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **320 passed / 0 failed**（第 35 轮 319 + 食物状态 1 项）。全工作区 4,886 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v47 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- `CardKind::Tanhuadong`（41）/`Yougeng`（42）；监听技定义键
  `gj_food_tanhuadong`/`gj_food_yougeng`（host_skills 声明 → 目录
  SkillDef → 授予机按牌种映射安装）；授予/卡牌派发机 def_key
  `gj:tanhuadong`/`gj:yougeng` 走 card_ok 绑定。
- 触发元数据：摊黄冻 = `PhaseEnd`（Play，begin=false）；有更 =
  `PhaseBegin`（Judge，begin=true）——过滤器按 owner 收敛。
- PlayCard 校验：恰 1 名存活目标（可含自己），且目标尚未持有同名
  状态（重复喂食拒绝，对齐上游 filterTarget）；食物无 wuxieable，
  无懈窗口不包。
- `rounds` 无限时长语义：字段仅食物监听使用，其余技能构造点为 0
  （非计时）。

## 回归（g1_shahai_gujian 增补 1 项 + 切片计数更新）

- `food_state_grant_tick_and_expire`：摊黄冻自喂——授予即 rounds=3、
  出牌阶段结束摸 1 扣减、**rounds 入字节存档**（encode/decode 后
  一致）；注入 rounds=1 → 最后一回合仍摸 1 → **归零移除**。有更——
  对他人授予、已受伤且体力最少时判定阶段开始回复 1、扣减；全区
  牌归属守恒。
- 切片计数：9/40（+食物 2），deferred 对账 31。

## 边界（如实登记）

- 食物簇余 **10 张**暂缓（春饼[手牌上限 mod]/咕咚羹[杀伤修改]/
  鲤鱼汤[护甲]/蜜链藕[view-as]/菱角[摸牌阶段+1]/青团[首杀摸牌]/
  卤玉羹[发现原语]/麻婆豆腐[随机弃]/元宝肉[额外杀]/麻茶[黑色目标
  限制]）——计时框架已就绪，逐张补效果面即可。
- 有更登记差异：原生无准备阶段，挂判定阶段开始；倒计时统一在
  效果触发点扣减（上游部分为独立阶段扣减）。
- 食物登记差异保留：鲤鱼汤护甲、卤玉羹发现牌为引擎缺口。
- 整包验收仍 **0/37**。
