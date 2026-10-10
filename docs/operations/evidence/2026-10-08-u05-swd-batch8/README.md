# U05 swd 第八批：潜偷面具（规则 v92，2026-10-08）

swd 子包第八批转换：潜偷面具（♠6，equip5 宝物，主动装备技潜偷）——
出牌阶段限一次，弃一张手牌为代价，判定非梅花则获得手牌数比你多的
其他角色一张随机手牌。前置：
[swd 第七批](../2026-10-08-u05-swd-batch7/README.md)。

## 上游语义核验（源码为准，card/swd/skill.js）

`shentou`：equipSkill、`enable: "phaseUse"`、`usable: 1`、
`filterCard: true`（代价任一手牌）、filter——存在手牌数比你多的其他
角色；content——judge 梅花则失败无事，否则 `target.getCards("h").
randomGet()` 由使用者获得。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Shentoumianju`（编码 111：archive enc/dec、
  a0 variant/ALL 同步；`EquipSlot::Treasure` 占栏 + equip_skills 声明
  `swd_shentou_skill`）+ `HostBehavior::SwdShentouEquipSkill`（tag 80、
  `ALL` 79→80、a0 index 79）——三段：Act（问代价 SelectCards min=1
  `swd_shentou_cost`）→ SelectCard（弃代价 → 判定：梅花无事、非梅花
  随机偷手牌更多目标一张）；ActivateSkill 路径 filter=有代价可弃且
  存在手牌更多者（Yuheng 同款 activate 臂）；快照派生 SelectCard
  询问重 derivation。
- **装备注入同步**：快照注入装备须同步授予技能实例（首跑
  `EquipSkillMismatch` 校验捕获）。
- **登记**：判定牌为真判定（非梅花成功），无 rewriters 通道（本卡无
  改判交互）。

## 验收口径

- **回归已加入**：`swd_shentou_active_steal_with_club_judge`——发动后
  代价弃置、判定结果二选一断言（偷牌则目标减 1 / 梅花失败则手牌不变）
  + 守恒粗检。
- **全量通过**：engine-native **407 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**，DEPS_EXIT=0。
- **golden**：版本 91→92 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（纯版本位移）。
- **剩余**：swd 其余 8+ 卡种逐批转换（青龙令珠/星君眼/伏符录/风雪刃/
  兰阁怪衣/岱火风轮/羊皮卷/贴房绅视等）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
