# 2026-09-23 第 26 轮：运筹帷幄原语增补（规则版本 37）

按第 25 轮包数据登记的 reason 增补两个宿主原语并扩展运筹帷幄切片：
**水攻**（`ShuigongCard`：多目标强制弃装备）与**釜底抽薪**
（`FudichouxinCompare`：拼点两段选择）。`STANDARD_RULESET_VERSION` **36 → 37**
（`CardKind::Shuigong`/`Fudichouxin`、`HostMachine.targets` 多目标入快照；
14 golden 重录）。运筹帷幄切片升至 **10/22**（暂缓降至 13 张）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **300 passed / 0 failed**（第 25 轮 298 + 水攻/拼点回归 2 项）。全工作区 4,866 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v37 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- **水攻**（`ShuigongCard`，tag 11）：多目标锦囊（所有有装备的存活角色），
  无懈窗口包装，效果为每目标强制弃一张装备（登记差异：上游为目标自选，
  原生取首张确定性弃置；撤销走装备技能生命周期同路径）。`HostMachine`
  新增 `targets: Vec<PlayerId>` 多目标字段（入快照/哈希/字节）。
- **釜底抽薪**（`FudichouxinCompare`，tag 12）：拼点两段选择——目标选牌
  （`CompareTargetPick`）→ 己方选牌（`ComparePlayerPick`）→ 点数大者赢、
  赢方获得两张拼点牌（平局目标胜，对齐上游 chooseToCompare）。
  `HostStage` 新增 `CompareTargetPick`/`ComparePlayerPick`（tag 4/5）。
- 恢复派生：两段挂起点 + 三尖目标选择（修正 Act 守卫）+ 拼点第二段
  （修正 Act 守卫）均与运行时同源。

## 回归（g1_shahai_yunchou 新增 2 项 + 切片扩展 1 项更新）

- `shuigong_multi_target_forced_equip_discard`：双目标装备均被强制弃置、
  不造成伤害、字节恢复。
- `fudichouxin_compare_two_stage_with_restore`：目标选牌 → 己方选牌挂起 →
  **字节恢复** → 己方赢获得两张拼点牌。
- `g1_yunchou_vertical_existing_kinds_and_lock` 更新：切片 4→10 张、
  暂缓对账 13 张。

## 边界（如实登记）

- 运筹帷幄切片 **10/22**（暂缓 12 张：范围锦囊 1/换牌 2/展示联动 3/装备技 1/
  获得武器 2/摸牌交换 2/舌战 2——共 13？与 deferred 合计 13 对账，含 hongshui 1）。
- 拼点平局目标胜（对齐上游 chooseToCompare 平局无收益）；
  火杀计入生克类判定的差异延续第 23 轮登记。
- 整包验收仍 **0/37**（G1 纵向样本 ×6，运筹帷幄切片 +6 张）。
