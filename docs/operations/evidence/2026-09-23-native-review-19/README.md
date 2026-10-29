# 2026-09-23 第 19 轮：G1 规则宿主原型（欢乐卡牌）+ AI 候选合法回退

本轮为 **G1 实现批次（第二批）**：规则宿主原型（扩展机器载荷的独立装载与续行编码）
落地，欢乐卡牌按 cardpile 同模式转换为真实包并导入，嵌套选择恢复贯通；
同时关闭最后一个 AI 探针反例 OPEN_AI_CANDIDATES。
`STANDARD_RULESET_VERSION` **30 → 31**（MachineSnapshot::Host、手里剑牌种、宿主技能
目录注册；14 golden 因目录/快照变化重录）。全部探针反例至此清零：**22 FIXED / 0 OPEN**。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **283 passed / 0 failed**（第 18 轮 277 + 规则宿主/欢乐卡牌 G1 4 项 + AI 候选回退 2 项）。全工作区 4,849 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v31 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN**——OPEN_AI_CANDIDATES 关闭（单选候选逐项枚举后规划选择出闪）。 |

## 规则宿主原型（crates/engine-native-content/src/content/host.rs）

- **数据声明行为**：扩展包声明 `HostBehavior`（版本化标签枚举），宿主以有界原语执行，
  不暴露 Rust 内存布局或任意状态写权限。本批原语：苦无（杀伤害后目标弃一张手牌，
  强制触发）、酒尾（结束阶段已受伤回血否则摸牌，可选触发——确认门由框架 OptionalGate
  承担）、手里剑（基础牌：目标弃 1 张装备或受 1 点伤害，嵌套选择）。
- **显式续行状态**：`MachineSnapshot::Host { def_key, behavior, stage, owner, target, card }`
  入快照/哈希/字节存档（tag 42，定义键有界字符串 ≤64 字节）；
  `active_ask` 为手里剑目标选择提供恢复派生（与机器同形状）。
- **装载绑定**：恢复期校验 def_key——技能型须解析为目录宿主技能且行为标签一致；
  卡牌型（手里剑）要求目录含该牌种。扩展集合不含该包 → `UnknownMachine` 拒绝。
- **目录注册**：`standard_catalog_with_extensions` 将 `ExtensionPack.host_skills` 声明
  合并为 SkillDef（名称跨包/与基础目录唯一，冲突返回 `DuplicateSkill`）。

## 欢乐卡牌转换（packages/extensions/huanle-upstream-noname/）

- manifest.json + package.npack.json + assets/cards.json；数据核对自
  `apps/core/extension/欢乐卡牌/card/{list,card,skill}.js @ 5c02a745`。
- 转换切片：苦无（武器，射程 2 对应上游 attackFrom -1）+ 手里剑×2 + 苦无/酒尾宿主技能。
- **如实登记的暂缓项**：monkey（事件取消+弃装备换牌）、xuelunyang（技能窃取）、
  mianju（翻面取消——原生无翻面机制）需更多宿主原语；装备技能的挂载生命周期
  （装备时授予/离场撤销）未实现，本批以技能授予验证行为；无对立angena。

## AI 候选合法回退（原 OPEN_AI_CANDIDATES）

`plan_action` 的候选生成从 `options.take(max)` 改为：每个单选项（上限 32）∪
min≥2 时按序取 min 的合法集合 ∪ min=0 且允许放弃时的空应答。
回归：tests/ai_planner.rs `plan_enumerates_single_choice_fallbacks`（持闪应打出闪）、
`plan_enumerates_confirm_and_cancel_fallbacks`（确认两支+放弃回退）。

## 边界（如实登记）

- 规则宿主为**原语组合**原型，非通用脚本执行后端；执行后端选型（WASM/DSL 等）
  仍按 v3.3 门槛「实测后定案」，本批交付的是可恢复/可绑定/有界扩展机器契约的最小实证。
- 欢乐卡牌为**部分转换**（4/7 实体牌、2/5 装备技能）；整包验收仍 **0/37**。
- 宿主行为数不向上扩展为任意代码：新增行为 = 宿主 ABI 递增 + 工程回归。
