# 2026-09-23 第 32 轮：草船借箭分支选择原语（规则版本 43）

按运筹帷幄包数据登记的 reason 增补**草船借箭**原语
（`CaochuanCard`，tag 19）：目标分支选择（PickTag 2 分支）——
分支 0：交出手牌中所有【杀】给使用者并受 1 点反噬杀（登记差异：
上游走完整杀结算，原生为直接 1 点伤害）；分支 1：展示手牌并被
弃一张（确定性首张）。目标无手牌杀时仅剩分支 1。
`STANDARD_RULESET_VERSION` **42 → 43**（`CardKind::Caochuanjiejian`；
14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **314 passed / 0 failed**（第 31 轮 311 + 草船借箭 3 项）。全工作区 4,879 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v43 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- `CardKind::Caochuanjiejian`（tag 35）；`HostBehavior::CaochuanCard`
  （tag 19），无懈窗口包装；`def_key` 绑定（`wd:caochuanjiejian`）
  按牌种在场判定。
- 分支 0：交出所有手牌【杀】+ 使用者受 1 点反噬杀（登记差异：
  上游走完整杀结算，原生为直接 1 点伤害）。
- 分支 1：展示手牌 + 被弃一张（确定性首张；登记差异：上游为目标
  自选，原生为确定性首张）。
- 无手牌杀时仅剩分支 1（PickTag 选项收敛）。

## 回归（g1_shahai_yunchou 新增 2 项 + 切片计数更新）

- `caochuan_branch_zero_gives_shas_and_reprisal`：分支 0 交出所有杀 +
  反噬伤害。
- `caochuan_no_sha_branch_shows_and_discards`：无手牌杀 → 仅剩分支 1
  （展示+被弃）。
- 切片计数：21/23 张（+草船借箭 2），deferred 对账 2 张（舌战群儒 2）。

## 边界（如实登记）

- 运筹帷幄切片 **21/23**（暂缓 2 张：舌战群儒 2[舌战体系]）。
- 草船借箭登记差异：分支 0 的"视为对使用者出杀"简化为直接 1 点伤害；
  分支 1 的"弃一张"为确定性首张（上游为目标自选）。
- 整包验收仍 **0/37**。
