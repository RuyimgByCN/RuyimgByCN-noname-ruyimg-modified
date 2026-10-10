# P2/N08 曹仁装配授予路径核验（装配授予统一，2026-10-07）

主稿 P2/N08 行的「曹仁条目装配授予路径」切片：补齐 `debug_set_general`
（仅装槽）之外的装配授予机制，使据守技随武将装配真实授予。前置批次：
[帧生命周期核验](../2026-10-07-e03-frame-lifecycle/README.md)。

## 背景核对

- 上游 `archive/upstream/noname/apps/core/character/standard/character.js`
  实测 33 个武将条目，**无曹仁/据守**（`caoren`/`jushou` 零命中）——原生
  `STANDARD_GENERALS` 的曹仁为新增条目（26 非样例 = 上游对齐 25 + 原生新增 1），
  规则基线对齐表已更正旧计数（25→26，8 个缺失来源将清单不变）。
- 既有装配面：国战 `RevealGeneral`（ADR 0769）明置实际授予技能集+体力上限；
  `debug_set_general` 仅装槽（护驾等势力/性别资格测试依赖不授予语义）；
  `debug_grant_general` 按名授予完整技能集（无槽位/体力联动）。

## 实现

| 改动 | 内容 |
|---|---|
| `apply_general_assembly`（engine.rs，新私有函数） | 按武将表配置体力上限（主将槽、高体力夹到上限）并授予技能集 |
| `RevealGeneral` 重构 | 原地授予块替换为调用 `apply_general_assembly`（行为等价：golden 重放全绿，明置测试不变） |
| `debug_assemble_general`（新原语） | 配置主将槽 + 调用同一授予函数（授予语义不分叉）；原型模式无命令层选将入口（登记差异），装配类测试经此驱动 |
| `debug_remove_general`（新原语） | 撤销**当前槽**武将授予的技能实例（装备离场撤销语义）并清槽；空槽幂等；体力上限不回退（登记差异） |

语义登记：替换装配为**并集**授予（同明置授予语义，不自动撤销旧将技能），
摘除只撤销当前槽武将的技能集。

## 测试（分开陈述）

- **代码已实现**：装配授予统一（共用 `apply_general_assembly`）+ 装配/摘除原语。
- **回归已加入**：`general_assembly_grants_and_remove_revokes_slot_skills`
  （native_kernel）——set_general 不授予/不改体力；装配曹仁即授予据守+上限 4；
  替换装配（司马懿）授予新集、上限改 3、体力夹到上限、旧将据守保留；
  摘除撤销司马懿技能集、据守不受影响、清槽、空槽幂等。
- **据守测试改走装配驱动**：`caoren_jushou_flips_and_mianju_cancels_flip`
  由 `debug_set_general + debug_grant` 两步改为 `debug_assemble_general`
  单步（据守随装配真实授予），测试通过。
- **定向通过**（两测试）+ **全量通过**：engine-native **357/0/1**
  （唯一 ignore = 写轮眼规模诊断探针），replay_scenarios（golden）
  全绿——明置重构行为等价，哈希不变；lib `RUSTFLAGS=-D warnings` 通过。
- **规则/格式版本不变**（v61/v4）：无状态面/编码变化（技能实例表与
  general_slots 均已编码）。

## 未做（照旧登记）

标准 8 个缺失来源将、牌表逐张对照、KOF 会话一致性、逐技能语义对齐、
原型模式命令层选将入口（需协议层新命令，另批评估）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（6 文件，相对 noname-next 根）。
