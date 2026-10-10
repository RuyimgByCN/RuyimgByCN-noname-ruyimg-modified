# 2026-09-23 第 23 轮：玩点论杀武将批（规则版本 35）

本轮按定型管线完成 **玩点论杀武将批**：6 名武将元数据经角色包管线入目录
（真实包 `packages/extensions/wudian-upstream-noname` 升级 v2，assets/cards.json
增加 generals 数组），并增补**思继宿主原语**（弃牌阶段弃杀数×2 摸牌，可选触发）。
`STANDARD_RULESET_VERSION` **34 → 35**（`HostMachine.cards` 关联牌组入快照；
14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **295 passed / 0 failed**（第 22 轮 294 + 武将批/思继 1 项合并回归）。全工作区 4,861 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v35 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 武将批（6 名，元数据核对 @ 5c02a745）

费诗（蜀3）/ 韩龙（魏4）/ 刘馥（魏3）/ 刘焉（群3）/ 留赞（吴4）/ 田豫（魏4）。
`general_meta` 元数据与国战明置授予沿 3D精选管线（v33）。

- **已映射 1/11 技能**：思继（siji）——`HostBehavior::SijiDiscardDraw`：
  `CardsDiscarded` 强制收集 + OptionalGate 可选确认（框架承担），
  按本次弃置中【杀】类张数 ×2 摸牌（登记差异：火/雷杀计入，上游按牌名只计普通杀）。
- **暂缓 10/11**（包数据逐条登记 reason）：nsmod 类（moshou 目标免疫 mod 原语）、
  ciqiu（伤害增幅+自戕子技能）、zhucheng（多步筑城）、duoqi（全局弃牌遍历）、
  juedao（距离/决斗联动）、geju（割据标记）、kangyin（多步）、chezhen/youzhan（标记/多步）、
  nsshuaiyan（多目标联动）。

## 新增机制：HostMachine.cards 关联牌组

`HostMachine`/`MachineSnapshot::Host` 增加 `cards: Vec<CardId>`（入快照/哈希/字节，
恢复期界校验 + 牌归属核算计入）——支持「对一组牌计数/操作」类触发行为
（思继为首个使用者）。

## 回归

`g1_wudian_generals_and_siji_discard_draw`：6 将元数据 + 11 技能声明（1 映射/10 暂缓）
断言；韩龙明置路径（general_meta 技能=思继、体力 4）；弃牌阶段弃含杀 →
思继确认挂起 → **字节恢复** → 确认 → 摸 2×弃杀数。

## 边界（如实登记）

- 玩点论杀仍为**部分转换**（武将批元数据齐、技能 1/11 已映射；卡牌 10/21）。
- G1 纵向样本累计 **×5**（包粒度）；玩点论杀武将批为本批增量。整包验收仍 **0/37**。
