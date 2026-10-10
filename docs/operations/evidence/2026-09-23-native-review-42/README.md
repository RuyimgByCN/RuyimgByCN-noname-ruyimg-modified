# 2026-09-23 第 42 轮：古剑最后一批 5 张（规则版本 53）

古剑子包续切片 **33/40 → 38/40**：

- **砂糖**（`JiguanShatangCard`，tag 47，×1）：目标受 1 点火焰伤害，
  然后获得 1 点护甲——**emit/Yield 两段式**：伤害帧提交后再给护甲
  （否则新护甲会吸收本次伤害，实测暴露后修正）。
- **黑龙鳞片**（`JiguanHeilongCard`，tag 48，×1）：获得 1 点护甲。
  登记差异：上游附带「防御距离 +1」临时技——距离修改原语暂缓。
- **硫头阁**（`JiguanLiutougeCard`，tag 49，×2）：对自己使用摸一张
  牌；对其他角色使用其弃置首张手牌。登记差异：上游 getBuff/getDebuff
  为随机效果池（且需敌我判定），原生简化为确定性效果。
- **木头面具**（`CardKind::Mutoumianju`，tag 64 装备；装备技
  `GujianMutouEquipSkill` tag 50）：出牌阶段限一次，任意手牌当
  【杀】——**view-as 装备技**：装备经 `equip_skills` 生命周期授予
  实例，PlayCard 转化链读取（enabled && !once_used），杀结算侧置
  once_used（随回合重置）。

`STANDARD_RULESET_VERSION` **52 → 53**（`CardKind` 61–64；
`Mutoumianju` 注册 `EquipSlot::Armor`；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **326 passed / 0 failed**（第 41 轮 325 + 收官批 1 项）。全工作区 4,892 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v53 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 回归（g1_shahai_gujian 增补 1 项 + 切片计数更新）

- `final_batch_shatang_heilong_liutouge_mutou`：砂糖——1 火伤 + 1
  护甲（两段式时序验证：护甲不吸收本次伤害）；黑龙鳞片——+1 护甲；
  硫头阁——自己摸 1、他人弃首张；木头面具——装备授予实例、手牌当
  杀放行并命中、once_used 置位；全区牌归属守恒。
- 切片计数：38/40（+收官批 5），deferred 对账 2。

## 调试记录（本轮修复的缺陷）

- 砂糖初版在同一机器步骤内「先 emit 伤害再 change_hujia」——emit
  为排队帧，护甲在伤害提交前生效并吸收了本次伤害（探针
  `HujiaAbsorbed` 定位）→ 改为 emit 后 `Flow::Yield` 两段式。
- `Mutoumianju` 未注册 `equip_slot` → `is_equip()` 为 false → 装备
  路径未走、装备技未授予（探针 grant_equip_skills 未被调用定位）
  → 补 `EquipSlot::Armor` 注册。
- 本轮两个生成器编辑脚本因锚点漂移未落盘（deferred 条目文本与
  推断不符），按实际文件内容重锚后完成。

## 边界（如实登记）

- 古剑暂缓 **2/40**：帝女玄霜（濒死特殊牌结算路径——savable 基本牌
  的弃摸骑乘需濒死机器扩展）、玉衡（升级机制+随机敌目标——复合；
  黑龙鳞片距离技同属距离修改原语缺口）。
- 硫头阁登记差异：随机效果池 → 确定性效果（无敌我判定需求——目标
  区分用「自己/其他」表达）。
- 整包验收仍 **0/37**。
