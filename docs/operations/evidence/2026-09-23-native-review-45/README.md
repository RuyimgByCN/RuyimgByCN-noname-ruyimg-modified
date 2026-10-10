# 2026-09-23 第 45 轮：世界树装备穿戴机——炉石 17/17 收官（规则版本 56）

炉石子包 **16/17 → 17/17 全卡转换收官**（deferred 清零）：

- **世界树**（`HearthShijieshuCard`，`HostBehavior` tag 59，
  `CardKind::Shijieshu` 73）：**装备搜索+穿戴机原语**——牌堆自顶
  向底扫描，取前两张**不同槽位**的装备；目标依次穿戴：同槽旧装备
  撤销装备技后置入弃牌堆，新装备入装备栏并 `grant_equip_skills`
  授予技能实例，逐件记录 `LogEvent::Equipped`（含 replaced）。
  无懈窗口包装。

`STANDARD_RULESET_VERSION` **55 → 56**（`CardKind::Shijieshu` 73；
14 golden 因目录变化重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **330 passed / 0 failed**（第 44 轮 329 + 世界树 1 项）。全工作区 4,896 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v56 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 回归（g1_hearth 增补 1 项 + 切片计数更新）

- `shijieshu_equips_two_different_slots`：与机器同语义的自顶扫描
  预测两张不同槽位装备 → 使用后断言目标装备栏恰两张、槽位不同、
  与预测一致；全区牌归属守恒。
- 切片计数：**17/17**，deferred 清零——**炉石子包全卡转换收官**。

## 边界（如实登记）

- 世界树登记差异：上游按装备子类型（subtype）去重搜索，原生按
  equip_slot（武器/防具/坐骑/宝物四槽）去重——语义近似；上游两件
  装备经 ui.special 暂存展示，原生直接结算。
- 装备穿戴复用 `commit_equip` 同款逻辑（同槽替换 + grant_equip_
  skills），但为机器内直穿（不经 Equipped 事件帧），装备技触发类
  原语（如精彀）不会因本次穿戴触发——登记差异。
- 整包验收仍 **0/37**。

## 杀海拾遗子包进度汇总

| 子包 | 进度 |
|---|---|
| 运筹帷幄 yunchou | 23/23 收官（v44） |
| 古剑 gujian | 40/40 收官（v54，暂缓登记差异：帝女玄霜濒死骑乘/玉衡升级/黑龙鳞片距离技） |
| 炉石 hearth | **17/17 收官（本轮 v56）** |
| gwent / mtg / swd / xianjian / ow / zhenfa / wuxing play | 未转换 |
