# 2026-09-23 第 24 轮：三尖武器技 + 莫守目标免疫原语（规则版本 36）

本轮按第 23 轮包数据登记的 reason 增补两个宿主原语，直接消化玩点论杀的
部分暂缓余量：**三尖武器技**（`SanjianWeaponDamage`：两段选择——弃一张手牌 →
对距受损者距离 ≤1 的另一名角色造成 1 点伤害）与**莫守目标免疫**
（`MoshouImmunity`：延时锦囊「乐/兵」不能指定持有者为目标，命令边界过滤）。
`STANDARD_RULESET_VERSION` **35 → 36**（`CardKind::Sanjian`；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **297 passed / 0 failed**（第 23 轮 295 + 三尖/莫守回归合并净增 2 项）。全工作区 4,863 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v36 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- **三尖**（`SanjianWeaponDamage`，tag 9）：`Damage` 强制触发（来源=持有者、
  杀类、未防止）；`HostMachine` 初始阶段 `SelectCard` → 选弃手牌（SelectCards）
  → `SelectNature` 复用为「等待选目标」→ `Act` 消费应答：弃牌 + 对距受损者
  距离 ≤1 的另一角色造成 1 点伤害。恢复派生按 `Act` 守卫（与精彀同模式：
  机器在 WaitChoice 前推进阶段）；目标选项与运行时同源（距离计算）。
- **莫守**（`MoshouImmunity`，tag 10）：`SkillBehavior` 新增目标过滤原语钩子
  `blocks_delayed_trick_target`；`Catalog::blocks_delayed_trick(core, p)` 供
  PlayCard 命令边界在乐/兵目标校验时拒绝（`IllegalTargets`），对其他目标无影响。
- wudian 包数据同步：cards 增加 `sanjian`（club 5，武器射程 3——上游
  `attackFrom -2`）；`deferred` 移除 sanjian（原语已交付）；skills 增加
  `wd_sanjian`（卡牌装备技能，经 `with_equip_skills` 挂载生命周期）与
  `moshou`（费诗武将技能，映射 `host:moshou_immunity`）。

## 回归（g1_wudian 新增 2 项）

- `sanjian_weapon_skill_two_stage_choice_with_restore`：装备三尖（equip_skills
  生命周期授予）→ 杀伤害 → 弃牌选择 → **字节恢复** → 目标选择 → 弃牌+1 点伤害。
- `moshou_blocks_delayed_trick_target`：乐→无免疫目标正常置入；乐/兵→莫守
  持有者被拒（IllegalTargets）。

## 边界（如实登记）

- 玩点论杀余量更新：卡牌 **11/21**（+三尖）；暂缓 5 张（调兵遣将×2、
  诱敌深入×3）；武将技能映射 **2/11**（思继、莫守）。整包验收仍 **0/37**。
- 莫守免疫范围=乐/兵（上游 mod 仅这两个牌名）；新增延时锦囊牌种时须同步。
- 三尖目标须「距受损者 ≤1 且非受损者」——上游 `get.distance` 同语义；
  多段伤害/属性传导（铁索）下三尖是否逐段触发未验证（登记待办）。
