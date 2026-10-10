# 2026-09-23 第 20 轮：G1 wuxing 纵向样本（规则版本 32）

本轮为 **G1 实现批次（第三批）**：wuxing（noname-liuli 来源，L06）按 cardpile/欢乐卡牌
同模式转换为真实包并导入——五行属性入核心状态、生克宿主行为、五行盘主动技
（PickTag 选择原语）与宝物栏（Treasure）装备槽贯通。
`STANDARD_RULESET_VERSION` **31 → 32**（PlayerState.wunature / core.card_natures 入状态、
ChoiceKind::PickTag / ChoiceOption::Tag、CardKind::WuxingPan；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **286 passed / 0 failed**（第 19 轮 283 + wuxing 纵向 3 项）。全工作区 4,852 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v32 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持——wuxing 改动未破坏既有验收。 |

## 转换产物（packages/extensions/wuxing-upstream-liuli/）

manifest.json + package.npack.json + assets/wuxing.json（属性表、1/3 比例、五行盘 黑桃5）；
数据核对自 `noname/extension/wuxing/extension.js @ 8485dbb2`；生成器可复跑（防漂移由
g1 测试的安装-资产核对承担）。

## 纵向验收点（g1_wuxing 3 项）

1. **五行初始化**：`InitHook::WuxingNatures` 经 `Engine::new_with_extensions` 生效——
   每名存活角色经引擎 RNG 获得随机属性（金/木/水/火/土），牌堆约 1/3 牌获得属性
   （统计区间断言），属性入 `PlayerState.wunature` / `core.card_natures`
   （快照/哈希/字节全覆盖，越界值恢复拒绝）。
2. **生克宿主行为**（`HostBehavior::ShengkeCheck`，强制触发，开局授予全体角色——
   对齐上游 `_shengke` 全局技能）：成为相生属性牌目标 → 摸 1（金生水用例）；
   成为相克属性牌目标 → 弃一张手牌（金克木用例；登记差异：上游为目标自选
   「he」弃牌，原生取首张手牌确定性弃置）。
3. **五行盘**：`CardKind::WuxingPan` 占栏装备经新增宝物栏 `EquipSlot::Treasure` 装备；
   `wuxingpan_skill`（出牌阶段限一次，改变一张手牌属性）经宿主主动技
   `ChangeCardNature` 两段选择（选牌 SelectCards → 选属性 **PickTag**）实现，
   两段挂起点均字节恢复，属性写入 `card_natures` 永久生效。
4. **内容锁**：带 wuxing 存档被纯标准恢复拒绝；同集合恢复成立。

## 新增选择原语

`ChoiceKind::PickTag` + `ChoiceOption::Tag(u8)`（0..=4，界外恢复拒绝）——定界标签
选择，供扩展声明「从有限命名集合中选一」类交互；选择种类字节编码扩展至 4。

## 边界（如实登记）

- 上游属性分配用 `Math.random`，原生用引擎 RNG（可重放）——逐局属性分布与上游
  不逐张等价（比例与机制等价，随机流不同源）。
- 被克弃牌为确定性首张（上游为目标自选含装备区的牌）——登记差异。
- 五行盘技能以授予验证；装备技能挂载生命周期（装备时授予/离场撤销）仍未做。
- 上游 `_status.connectMode`（联机模式停用 wuxing）分支不适用原生单机样本。
- wuxing 为**部分转换**（机制核心已贯通；联机/视频/帮助文本等表现层不适用原生）。
  **G1 纵向样本 ×3 已交付（cardpile 双来源 / 欢乐卡牌 / wuxing）；整包验收仍 0/37。**
