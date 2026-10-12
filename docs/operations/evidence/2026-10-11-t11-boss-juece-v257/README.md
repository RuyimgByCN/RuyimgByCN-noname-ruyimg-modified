# 2026-10-11 T11 阴间系旧绝策片：绝策（规则 v257）＋马面条目补全

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `3a2ea8d5`）。上游依据：`juece`（一将包李儒旧绝策，yijiang/skill.js:9977：global lose/equip/gain/addJudge/loseAsync/addToExpansion 事件族——自己回合内（`_status.currentPhase === player`）其他角色失去牌后手牌清空（countCards("h")===0 且 evt.hs>0）→ chooseBool → damage(1)）；马面表含 juece。

## 建模

- **绝策**（查询型 Noop，`BOSS_JUECE`）单臂（commit_cards_discarded 追加段，魔炎臂后）：
  - 条件：持有者=当前回合角色（自己回合内）+ 失牌者≠持有者 + 失牌者**弃后空手**（失去最后一手牌）。
  - 效果：对失牌者造成 1 点伤害（chooseBool AI→恒发动确定性近似登记）。
- **马面条目补全**：`[鬼计, 反馈, 炼狱]` → `+绝策`（v257 顺延）。

## 登记差异

- 上游 lose/equip/gain/addJudge/loseAsync/addToExpansion 全事件族 → native CardsDiscarded 面（胆术/魔炎同口径登记）。
- chooseBool AI 判断 → 恒发动确定性近似（先例族）。
- 上游 getIndex 逐目标各触发 → native 每次弃牌事件逐空手者各触发（等价覆盖）。

## 坑位实录

- 回归④段驱动过头：手写 next_phase 链从 1 号 Discard 推进跨过了 0 号整回合到 2 号 Play（cur=2），导致 play(0 的拆桥) 被以 2 号身份提交 → Rejected(CardNotInHand)。修正=advance_to_play(0) 定点布阵（**驱动循环必须锚定目标座位+阶段，禁止裸链式推进**）。

## 验证

| 项 | 结果 |
|---|---|
| 新回归 `boss_mamian_juece_last_hand_damage` | 五段：①非最后一手牌失去不触发（拆 1 剩 1）②失去最后一手牌→1 伤害③自己回合马面自身牌不触发（自身 exempt）④非持有者回合拆 2 号最后手牌无绝策⑤绝策实例存档往返+守恒 |
| 既有回归 | boss 全滤 54/54；机械审计：无多重注册/零属性清零 |
| 版本递增探针 | 256→257 全 14 位移；回退 256 复跑 **全绿**（位移唯一来源=版本字段） |
| 14 golden 重录 | 257 全量重录 2/2 绿（行级程序化） |
| 全量 | **604 通过 / 0 失败 / 1 ignore**（2026-10-11 本机） |
| Gate 1b | cargo 真实 exit 0 |
| 版本 | 规则 256→**257**；存档不变 v59（无新字段/快照变体） |

## 遗留登记（不变）

- boss_stage 局 first=0 与虚失翻面臂自动推进交互；炎狱跨回合计数器精确口径。
- 下批候选：bianshen3 阴间变身系收尾（黑白无常/牛头马面 bianshen3→bianshen4 链）或 baonu 暴怒复杂面勘验。
