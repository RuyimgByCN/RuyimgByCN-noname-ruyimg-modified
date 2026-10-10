# U05 xianjian 第十六武将切片（规则 v157，2026-10-09）

姜城（pal_jiangcheng，qun 4 男）焚逝（fenshi）+龙火（longhuo）双技闭环；
yanzhan 系登记暂缓。前置：
[xianjian 第十五切片](../2026-10-09-ow-tangyurou-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js fenshi/longhuo/yanzhan）

- `fenshi`：trigger player dyingAfter forced juexingji——content
  changeHujia(2)+draw(2)+addSkill("longhuo")；derivation longhuo。
- `longhuo`：trigger player phaseEnd（无 forced=check 询问式）——
  event.targets=get.players(lib.sort.seat) 逐目标 redo：isIn() 则
  damage("fire")（全场含自己各 1 火伤，逐目标结算后重查存活）。
- `yanzhan` 系：enable phaseUse viewAs fire sha usable:1 红牌 filterCard
  +onuse 目标 addTempSkill yanzhan3（响应花色限制 mod）+group yanzhan2
  （damageEnd→yanzhan4 杀限次 +1 mod）——viewAs/mod 系暂缓。

## 关键实现决策：新事件种 EventKind::DyingSaved

上游 dyingAfter 在固有救援结算**之后**触发。原生若挂既有
`EventKind::Dying`（濒死入口、Intrinsic 救援机之前），焚逝摸到的 2 张
牌会出现在随后的求桃询问里参与自救——功能性偏离上游。因此新增
`EventKind::DyingSaved`（救援机在体力>0 且脱离濒死的出口发射，触发技
在其帧 AfterTriggers 窗口消费），时序与上游一致。九点接线（同
EventData-variant 接线清单）：EventKind 枚举/EventData 变体/kind()
臂、snap 载荷校验臂、commit 空 arm（catch-all 兜住）、intrinsic
None 臂、event_kind 编解码 tag 19、EventData 编解码 tag 17
（player+opt source）、canonical_hash tag 23。

龙火多目标用 `Flow::Yield` 逐目标结算：每 emit 1 伤害即让栈回自身深
度后重查存活——铁索传导致死者不再向后结算（上游 redo isIn 同口径），
区别于一次 step 内连发全部伤害帧（存活检查前移、死者仍收后续帧）。

## 改动（分开陈述）

- **状态**：`PlayerState.jiangcheng_longhuo: bool`（常驻标志——兼作已
  觉醒一次性门控与龙火持有标记）。**编码中途纠偏**：初稿误置
  TurnState（换回合复位）且 PlayerState::new 混入两行 TurnState 字段
  init（build E0560/E0609 七错）——按上游 addSkill 永久技语义迁移至
  PlayerState 三段（state 声明+init、archive player 段 encode/decode/
  literal、engine player 哈希 push），TurnState 三处字面量与 archive
  TurnState 段拆除。
- **存档 v22→v23**（ARCHIVE_FORMAT_VERSION=23，PlayerState 段追加
  boolean）。
- **宿主行为**：`XianjianFenshi`（tag 165，DyingSaved 强制；filter 门
  控 flag==false）+ `XianjianLonghuo`（tag 166，PhaseEnd 确认门
  optional=true；filter 门控 flag==true；Act 惰性装填全场座位序
  （target 字段兼装填标记），逐目标 Yield 发射 fire 伤）——六表+
  ALL 计数 164→166。
- **cards.json**：姜城行（qun 4 男）：yanzhan mapping null+reason 暂
  缓、fenshi→host:xj_fenshi、longhuo→host:xj_longhuo（reason_ref 登
  记常驻实例建模）；rehash_cards_asset+rehash_npack 两步重算。
- **登记差异**：衍生技常驻实例+标志门控建模上游 addSkill 动态获得
  （未觉醒时实例在而不触发，语义等价）。
- **登记暂缓**：yanzhan 系（红牌转火杀 usable:1 viewAs+目标响应花色
  mod+杀限次 mod）。
- **版本**：STANDARD_RULESET_VERSION 156→157；a0_tag_registry 穷尽表
  追加 165/166。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`jiangcheng_fenshi_awaken_once_
  on_rescue`（直伤濒死→桃救回→+2 甲/摸 2/flag 置位；二次濒死伤 3（甲
  吸 2）不重触发）+ `jiangcheng_longhuo_fires_at_own_phase_end`（觉醒
  后 0 号 End 收束→确认→全场座位序各 1 火伤：0 号甲 2→1 吸 1 体力不
  变、1/2 号 4→3）。
- **全量通过**：engine-native **479 passed / 0 failed / 1 ignored**
  （v156 基线 477 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 156→157 + PlayerState 哈希 push + DyingSaved tag 入
  canonical_hash——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试要点（登记）**：二次濒死用伤 3 绕甲（debug_deal_damage 走完
  整伤害管线，护甲 1:1 吸收——甲 2 时伤 1 会被吸干不进濒死）。

## 剩余

- xianjian 其余 19/36 将逐武将纵切；yanzhan 系（viewAs/mod 系）与
  ow 猎空（离场区基建）、开局事件基建、虚拟 useCard 原语、U07 duoqi、
  zhenfa 五阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next
根）。
