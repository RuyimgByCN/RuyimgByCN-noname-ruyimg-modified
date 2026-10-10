# ADR 0645：迁移剩余深水区机制章程

- 状态：**章程（2026-08-22 立项）**——第三十三至五十九批（约 102 块）后，
  全包散技/短段/中长段/误标池/TODO 池浅面已系统性挖尽；本 ADR 固化
  剩余阻塞项的清单、阻断原因与原语设计草案，作为后续切片批次的章程。
- 关联：ADR 0644（议事系统 P3）、`docs/migration/upstream-1.11.5-delta.md`
  各批「剔除/缺口登记」。

## 1. jsrg【影】族剩余（切片 4+）

| 技能 | 阻断点 | 设计草案 |
|---|---|---|
| jsrgfuni 伏匿 | ②段「共计X张分发给1..X名角色」的逐目标数量分配（chooseButton 分配盘） | `distribute_shadow_cards(player, skill)`：Cont 选择 1..N 目标 + 均分/逐目标数量（首版可均分近似）；①段攻击范围0=`modifier attack_range override 0`；③段弃牌堆影联动需弃牌堆 kind 查询 |
| jsrgjinfa 矜伐 | 展示牌颜色 vs 议事结果比色分支 + 变更势力 | 比色：DebateResolved 上下文加 `event_card_color` 或以 storage 暂存展示牌色；变更势力：`change_group(player, group)` 原语（延祚/邀虎等同需） |

## 2. 延迟机制族

| 技能 | 阻断点 | 设计草案 |
|---|---|---|
| jsrgyaoyan 邀宴 | 每角色登记「回合末议事」+ 到点触发 + 红/黑分支 | `register_turn_end_debates(player, skill)`：Cont 逐角色 chooseBool 登记 → GameState.pending_turn_end_debates → 各自 turn_ended 触发 start_debate |
| jsrgchushi 出师 | 红分支「手牌和<7 则重复议事」循环 | 循环上限展开（≤3 次）或 while 语义（DSL 无循环——按上游实测循环罕见可近似单轮） |

## 3. ADR 0644 P3 剩余

- **意见牌快照**：DebateResolved 携带 opinion_cards（进 hash 评估）——
  当前无直接消费者（「获得意见牌」类技未入候选），挂起至消费者出现。
- 【影】联动变更势力：同 jinfa change_group。

## 4. tw 深水区（95 noop 文件抽样核验）

主体为：置于武将牌上（expansion/星舞/表/经略）、逐目标分配盘
（武魂/延祸/行殇）、势力/任务系统（延祚/令戮/密备）、观牌调序
（攻心/羽化/看破）、拼点/辩论变体（双刃/舌论——可复用 0644 管线）。
均为 2-4 原语级组合，按机制族切片推进，不做散扫。

## 5. 过滤器/选择器缺口台账（未达 3/3 维持登记）

装备失牌基础变体 2/3（yinzong/duci）｜sha_dodged 被闪侧 1（lxianglong）｜
fire-trick 目标取消选择器 1（huoshaolianying）｜弃牌阶段弃置≥1 任意牌 1｜
choose-other-change-max-hp 1（oldfushi）｜手牌空条件 modifier 1｜
per-color 杀选择器 1（poyue）｜锦囊伤害 taken-cancel trick 选择器 1
（qinglianxindeng）。

## 5b. jsrg 蓄谋族立项详案（2026-08-26 更新）

- 消费者：jsrgeqian（结束阶段蓄谋任意次+杀/蓄谋牌联动）、jsrgdaimou
  （杀指定时牌堆顶蓄谋/弃蓄谋牌）、jsrgmanjuan（无手牌时如手牌般使用
  弃牌堆牌）、jsrgjixiang（弃牌令他人用基本牌）等 8+ 技。
- 基建：`GameState.xumou_cards: BTreeMap<PlayerId, Vec<CardId>>` +
  `place_xumou_card(player, card)` / `consume_xumou_card(player)` +
  蓄谋牌可视为手牌使用（query 层把 xumou 区并入 hand 读取）。
- 切片 1：置牌+消耗原语（ABI 665 领号）；切片 2：蓄谋牌视为手牌
  （view_as 或 query 合并）；切片 3：逐消费者迁移。



### 状态（2026-08-26 切片 3b 后闭环，见 ADR 0646 §0）：实际实现走判定区+XUMOU_TAG 路线（非本节 ExpansionMap 草案）；eqian①/daimou 已迁移；cangxiong/temp 不达门槛。eqian② xumou-origin 交互留债务。

- 单技能半解锁不铸原语；缺口 3/3 干净消费者才立项。
- 伤害/回合热路径加状态必须 opt-in 门控（批 55 教训）。
- 牌种/view_as 零 ABI；效果原语全链 15 处 + 领号。
