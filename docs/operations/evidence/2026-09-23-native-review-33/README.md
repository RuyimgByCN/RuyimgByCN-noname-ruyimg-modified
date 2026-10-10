# 2026-09-23 第 33 轮：舌战群儒全体拼点原语——运筹帷幄 23/23 收官（规则版本 44）

按运筹帷幄包数据最后一张暂缓登记增补**舌战群儒**原语
（`ShezhanqunruCompare`，tag 20）：自-target 锦囊，使用者与**所有
有手牌的其他角色**逐一拼点（登记差异：上游为多轮 Bool 问答链，
原生为确定性首轮拼点——各取手牌首张，响应者点数大者摸 1，否则
使用者摸 1）。无懈窗口包装；`def_key`（`wd:shezhanqunru`）按牌种
在场判定。
`STANDARD_RULESET_VERSION` **43 → 44**（`CardKind::Shezhanqunru`
tag 36；14 golden 重录）。**运筹帷幄切片 23/23 完整切片**（deferred
清空）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **314 passed / 0 failed**（第 32 轮 313 + 舌战群儒 1；第 32 轮 README 曾记 314，按其日志实为 313）。全工作区 4,880 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v44 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- `CardKind::Shezhanqunru`（tag 36）；`HostBehavior::ShezhanqunruCompare`
  （tag 20），无懈窗口包装；快照 `MachineSnapshot::Host`（tag 42）
  序列化；archive nums 至 36；`KNOWN_PROMPTS` 含全部宿主提示词。
- 引擎校验：自-target（`targets` 必须为空）；kernel 派发臂将目标
  扩展为**按座位序的存活且有手牌的其他角色**（本类 AOE 语义在
  kernel 侧展开，不复用 `u.targets`）。
- 拼点为确定性首轮：使用者手牌首张 vs 各响应者手牌首张；
  响应者胜 → 响应者摸 1；平或使用者胜 → 使用者摸 1。
- 使用者无手牌时机器直接完成（`Flow::Done`）。

## 回归（g1_shahai_yunchou 新增 1 项 + 切片计数更新）

- `shezhanqunru_compares_all_responders`：3 人局注入确定手牌
  （使用者高点数首张、两名响应者低点数首张），过无懈窗口后断言
  使用者摸 2 + 全区牌归属守恒。
  - 注入修正：`sz` 自身必须先全区清除再放回手牌（先前只清
    手牌+牌库，`sz` 落弃牌堆时恢复被 `card_missing_from_all_zones`
    正确拒绝——归属总账校验按目录全集断言）。
- 切片计数：**23/23 张**（+舌战群儒 2），deferred 对账 **0 张**。

## 边界（如实登记）

- 舌战群儒登记差异：上游为逐人多轮 Bool 问答（答错者出局式
  连锁），原生为**确定性首轮拼点**（每响应者恰比一次）。
- 运筹帷幄切片 **23/23**（暂缓 0 张）——该子包完整切片收官。
- 装备牌历史遗留：`fudichouxin` 测试内高点数牌取自 `.rev().position()`
  （反向索引），现行断言恰可过；属测试局部瑕疵，非引擎问题，
  待后续轮次顺手修正。
- 整包验收仍 **0/37**。
