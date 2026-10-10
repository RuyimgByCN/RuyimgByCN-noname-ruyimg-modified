# 2026-09-23 第 31 轮：隔岸观火双目标拼点链（规则版本 42）

按运筹帷幄包数据登记的 reason 增补**隔岸观火**原语
（`GeanguanhuoCompare`，tag 18）：双目标拼点链——目标一/目标二各选拼点牌，
点数大者赢，赢方获得输方一张手牌，平局目标一弃牌交由使用者。
`STANDARD_RULESET_VERSION` **41 → 42**（`CardKind::Geanguanhuo`；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **311 passed / 0 failed**（第 30 轮 308 + 隔岸观火 3 项）。全工作区 4,877 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v42 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- `CardKind::Geanguanhuo`（tag 34）：双目标锦囊（恰 2 名其他存活且有手牌的目标）。
- `HostBehavior::GeanguanhuoCompare` 三段：CompareTargetPick（目标一选牌挂起）→
  ComparePlayerPick（消费一牌、目标二选牌挂起）→ Act（结算+弃锦囊）。
- 恢复派生：两段挂起点的 ask（asker/prompt）按 stage 标签推导（含 Act 修正——
  机器在 WaitChoice 前推进 stage 的模式）。
- def_key 绑定：`yc:geanguanhuo` 按牌种在场判定（恢复期）。

## 回归（g1_shahai_yunchou 新增 2 项 + 切片计数更新）

- `geanguanhuo_two_target_compare_chain`：双目标拼点 → 目标二选牌挂起 →
  **字节恢复** → 应答 → 赢方获得输方手牌。
- `geanguanhuo_rejects_wrong_target_count`：1/3 目标均 IllegalTargets。
- 切片计数：19/23 张（+隔岸观火 2），deferred 对账 4 张（草船借箭 2、舌战群儒 2）。

## 边界（如实登记）

- 运筹帷幄切片 **19/23**（暂缓 4 张：草船借箭 2[多步+代用杀]、舌战群儒 2[舌战体系]）。
- 隔岸观火登记差异：平局结算为"目标一弃牌（弃置）"，上游为"弃牌交由使用者"；
  多对目标链（multicheck/addedTarget 多轮）简化为单对——已登记。
- 整包验收仍 **0/37**。
