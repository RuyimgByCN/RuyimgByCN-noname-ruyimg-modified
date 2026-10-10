# ADR 0644：jsrg 议事（debate）系统设计立项

- 状态：**P1+P2 已落地（2026-08-22）：P1=ABI 653/654 旗舰原语（三分支 Golden 5/5）；P2=ABI 655/656 发起原语 + DebateResolved 事件（EventKind tag 45）+ 三过滤器（initiator/participated/differs）+ jsrgfayi 反应技迁移（事件快照 Golden 3/3）；P3 待排期**
- 背景：jsrg 占位头重记批（delta 2026-08-21 第四批）曾以「议事系统多技耦合需
  整体设计」登记全包 153 块。本批逐条核实**更正**：真正议事耦合技仅 **11 个**
  （发起型 5 / 反应型 6），其余 142 块为普通机制技（可按既定挖矿工艺逐批消化），
  不应被「系统级」标签整体阻塞。

## 1. 上游机制（content.ts `chooseToDebate`，实测摘录）

1. **发起**：发起者（player）携参与者名单调用；每个有手牌的参与者**强制**选择
  一张手牌作为「意见」展示（`chooseCardOL`，AI 随机；`fixedResult` 可改写他人
  意见——jsrgbazheng「受你伤害的角色意见视为与你相同」即用此口）。
2. **分组**：意见按颜色分组——红 / 黑 / others（无手牌或自定义颜色；红黑之外的
  颜色仅在特定扩展出现，P1 可忽略）。
3. **结果**：红组人数 vs 黑组人数，多数者为结果，**平票或全空为「无结果」**。
4. **事件**：分组完成先触发 `debateShowOpinion`（zhongxiong「额外展示一张手牌」
  类插在此时机），结算完成触发 `debateEnd`（fayi/fumou 的「议事结算结束后」）。
5. **分支**：各技能按结果红/黑执行不同效果（回复/失去体力/摸牌/伤害/弃牌/获得
  意见牌/摸至体力上限/变更势力/【影】交互等），或按「意见与发起者不同/相同」
  选择目标。
6. **意见牌去向**：默认展示后**收回原处**（不进弃牌堆）；「获得所有意见牌」
  （shanzheng 黑色分支）需要意见牌快照——事件须携带或可查。

## 2. 引擎侧设计（分阶段）

### P1：旗舰技权威结算原语（1 技 1 号，dchaoshi/resolve_* 模式）

先行落地 2-3 个旗舰，验证玩家感知与 Golden 可测性：

- `resolve_jsrgchaozheng(player, skill)`（准备阶段令所有其他角色议事：红色→
  意见为红的角色各回复 1；黑色→意见为红的角色各失去 1；全同意见→发起者摸
  参与者数张牌）。引擎内部完成「参与者各亮首张手牌（P1 确定性近似，AI 随机
  留 P2）→ 分组 → 多数决 → 分支结算」全流程，一次领号（ABI 顺延，tag/Cont
  同批顺延）。
- `resolve_jsrgsheju(player, target, skill)`（用杀指定唯一目标后/成为唯一目标后
  与对方议事，黑色→双方各减 1 点体力上限）——展示「响应式议事」形态。

P1 规避了通用事件模型的前置成本；每技注释近似点（AI 随机 vs 首张、动画时序）。

### P2：通用事件模型（反应型技解锁）

- `DomainEvent::DebateResolved { initiator, participants, red, black, result,
  opinion_cards? }`（result: Red/Black/None；opinion_cards 供「获得意见牌」类）。
- 触发过滤器：`owner_is_initiator`、`owner_participated`、
  `owner_opinion_differs_initiator`（fayi/fumou/yaoyan 反应型 6 技的接入口）。
- 表达式：`debate_result_red` / `debate_result_black`（0/1）、
  `debate_red_count` / `debate_black_count`、`debate_opinion_is_red(player)`。
- 发起原语泛化：`start_debate_all_others` / `start_debate_targets`（选择续体
  选参与者），分支效果仍在 DSL body 以表达式 if 表达。

### P3：交互深水区（暂不承诺）

- `debateShowOpinion` 时机插入（xiongbao 额外展示/随机化他人意见）；
  fixedResult 改写（bazheng）；「未议事角色」追踪（yaoyan 红色分支）；
  【影】标记联动与变更势力复合。

## 3. ABI 与治理预算

- P1：2-3 个号（旗舰技）；P2：发起原语 2 + 事件/过滤器/表达式（过滤器与表达式
  零 ABI，事件零 ABI）；P3 视需求。每号先在 ABI-COORDINATION.md 领取表占号。
- 序列化：DebateResolved 仅作为事件（不进持久状态），无回放风险；P2 若引入
  意见牌快照须进 hash 评估。

## 4. 风险与开放问题

1. P1「亮首张手牌」的确定性近似会系统性偏离上游博弈语义（上游 AI 随机出牌）；
   Golden 以固定牌序断言分组与分支，近似点逐技注明。
2. 「各回复/各失去」类分组 AOE 结算顺序按座次（与上游 showCards 顺序一致性
   待核对）。
3. 142 个非议事 jsrg 块按普通挖矿工艺推进，勿等本系统。

## 5. 验收

- P1 落地标准：≥2 旗舰技真实 DSL + 行为 Golden（红/黑/无结果三分支）+ 全仓绿。
- P2 落地标准：DebateResolved 事件 + ≥2 反应型技迁移 + 事件快照测试。
