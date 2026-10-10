# 2026-09-23 第 22 轮：玩点论杀批量转换（规则版本 34）

本轮按已定型管线批量转换 **玩点论杀**（noname 来源，U07）卡牌切片，并增补
望梅止渴宿主原语（条件增益）。`STANDARD_RULESET_VERSION` **33 → 34**
（`CardKind::WangmeiZhike`；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **294 passed / 0 failed**（第 21 轮 291 + 玩点论杀纵向 3 项）。全工作区 4,860 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v34 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 转换产物（packages/extensions/wudian-upstream-noname/）

manifest.json + package.npack.json + assets/cards.json（逐张核对
`apps/core/extension/玩点论杀/card/list.js @ 5c02a745`）；生成器可复跑。

- **已转换 10/21 实体牌**：杀×2 / 闪×3 / 桃×2（标准牌种）+ 望梅止渴×2
  （新 `CardKind::WangmeiZhike`，锦囊走无懈窗口，效果为宿主原语
  `WangmeiZhikeCard`）+ 铁索×1（既有 Chain）。
- **暂缓（包数据 `deferred` 登记 reason）**：调兵遣将×2（多人多步方向选择）、
  诱敌深入×3（杀响应转化原语）、三尖×1（武器技原语）。
- **角色武将（5 名）另批转换**（character/skill.js 814 行定制技能，需逐技能增补原语）。

## 望梅止渴宿主原语（WangmeiZhikeCard）

上游 content：目标为最小体力且已受伤 → 回复 1；目标为最小手牌数 → 摸
（已回复 ? 1 : 2）张。原生以 `HostMachine` 有界原语实现（遍历存活者取
最小体力/手牌数比较），锦囊经无懈窗口包装；卡牌派发机器的 def_key
（`wd:wangmeizhike`）绑定校验扩展为按牌种在场判定（恢复期）。
回归：`wangmeizhike_recovers_min_hp_damaged_target`、
`wangmeizhike_draws_min_hand_target`（两分支 + 无懈窗口 Pass）、
`g1_wudian_content_lock_and_byte_restore`。

## 边界（如实登记）

- 玩点论杀为**部分转换**（10/21 实体牌；6 张暂缓有因；5 名武将未转）——
  管线批量能力已实证（第五个包复用同一生成器/发现/导入/桥接/恢复路径零改动），
  但每个新牌种/技能仍需对应原语或逐个重写。
- 整包验收仍 **0/37**（G1 纵向样本 ×5）。
