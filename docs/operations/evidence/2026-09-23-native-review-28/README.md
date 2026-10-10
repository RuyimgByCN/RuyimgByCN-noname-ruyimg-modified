# 2026-09-23 第 28 轮：洪水 + 偷梁换柱原语（规则版本 39）

按运筹帷幄包数据登记的 reason 增补两个宿主原语并扩展切片至 **17/23**：
**洪水**（`CardKind::HongShui` 自-target 延时锦囊——判定梅花 2–9 → 3 点雷伤，
登记差异：上游未标注伤害属性，原生按雷属性处理）与**偷梁换柱**
（`TouliangSwap` 三段选择：己方选牌 → 目标选牌 → 交换，同色摸 1；
登记差异：目标临时技暂缓）。
`STANDARD_RULESET_VERSION` **38 → 39**（`CardKind::HongShui`/`Toulianghuanzhu`；
14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **304 passed / 0 failed**（第 27 轮 302 + 洪水/偷梁换柱 2 项）。全工作区 4,870 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v39 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- **洪水**：`CardKind::HongShui` 延时锦囊——使用时无懈窗口包装 +
  `PlaceDelayedMachine` 置入自己判定区（自-target，对齐上游 toself）；
  判定分发新增 HongShui 臂：梅花 2–9 → 3 点雷伤（`DamageNature::Thunder`）。
- **偷梁换柱**：`HostMachine` 三段选择（复用 `SelectCard`/`SelectNature`/`Act`
  阶段标签）——己方选牌 → 目标选牌 → 交换，同色摸 1。恢复派生：
  SelectCard 态 = 己方选牌挂起；SelectNature 态 = 目标选牌挂起（Act 为瞬态）。
- 运筹帷幄包数据同步：cards 增至 17 张（+洪水 1、偷梁换柱 2），deferred
  降至 6 张（草船借箭 2、隔岸观火 2、舌战群儒 2）。

## 回归（g1_shahai_yunchou 新增 2 项）

- `hongshui_self_delayed_trick_thunder_damage`：置入自己判定区 + 判定
  梅花命中 → 3 点雷伤（`debug_set_deck_top_by` 确定性判定牌，每步保持顶牌）。
- `touliang_swap_two_stage_with_restore`：己方选牌 → 目标选牌挂起 →
  **字节恢复** → 交换断言（己方得目标牌、目标得己方牌）。

## 边界（如实登记）

- 运筹帷幄切片 **17/23**（暂缓 6 张：草船借箭 2[多步+代用杀]、隔岸观火 2[多目标
  拼点链]、舌战群儒 2[舌战体系]）；整包验收仍 **0/37**。
- 偷梁换柱的目标临时技（`addTempSkill`）暂缓；洪水伤害属性按雷属性登记。
