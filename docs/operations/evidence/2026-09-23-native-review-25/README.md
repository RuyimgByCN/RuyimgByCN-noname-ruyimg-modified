# 2026-09-23 第 25 轮：杀海拾遗·运筹帷幄子包切片（G1 首个子包）

按定型管线拆解转换 **杀海拾遗** 首个子包——运筹帷幄（yunchou，noname 来源）：
真实包 `packages/extensions/shahai-yunchou-upstream-noname/`（manifest + npack +
assets/cards.json），切片 4/22 张经**既有牌种直转**（火杀→FireSha、过河拆桥×3→
Dismantle），其余 18 张按 reason 逐条登记于包数据 `deferred`（拼点比较、特殊锦囊、
装备技、舌战群体系、范围锦囊等）。本批无新引擎原语、无规则版本变化（36 不变）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **298 passed / 0 failed**（第 24 轮 297 + 运筹帷幄纵向 1 项）。全工作区 4,864 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v36 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 纵向验收点（g1_yunchou_vertical_existing_kinds_and_lock）

1. 发现/导入：`discover_extensions` 发现子包；import 安装激活。
2. 桥接：cards.json → ExtensionPack（火杀→FireSha、拆桥→Dismantle）。
3. 建局与规则结算：火杀按杀结算（1 点伤害 + 占用出牌阶段杀次数）；
   拆桥弃目标手牌——均走既有引擎通道，扩展卡与基础卡行为一致。
4. 内容锁：带扩展存档被纯标准恢复拒绝；同集合恢复成立。
5. 暂缓登记：包数据 deferred 合计 18 张，与 22-4 对账。

## 边界（如实登记）

- 运筹帷幄为**切片转换**（4/22；17+1 张暂缓 reason 登记于包数据：拼点比较原语、
  多目标弃装备原语、展示/获得联动、换牌、获得武器、舌战群体系、范围锦囊、装备技）。
- 杀海拾遗其余 7 个子包（gwent/hearth/mtg/swd/xianjian/ow/gujian）与 wuxing play
  未转换；整包验收仍 **0/37**。
- 本批无引擎语义变化；Sanjian 牌种枚举值（v36 声明）在本轮修正为实际入目录
  （上轮断言遗漏的枚举编辑），仅扩展包可用，基础目录不变。
