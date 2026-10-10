# U05 xianjian 第十九武将切片（规则 v160，2026-10-09）

明绣（pal_mingxiu，shu 3 女）灵火（linghuo）+归烬（guijin）双技闭环；
chengxin（诚信）濒死技能型救援通道缺口登记暂缓。前置：
[xianjian 第十八切片](../2026-10-09-xianjian-longyou-batch/README.md)。

## 候选选择（字节表评估）

雪见 3364（xshuangren 置武将牌+装备技能转移结构性；duci 需「失去装备
区牌」事件面）、绝世方 3586（三技全依赖机关牌类型/createCard）、南宫
煌 3613（sheling 触发面缺 useCardAfter 入弃堆事件；zhangmu 询问前置改
写）、王小虎 3825（husha 虚拟用牌族）、罗招嫣 3848（fenglue 虚拟用牌
族）——取 **pal_mingxiu 3851**（linghuo=round 冷却+回合伤害统计可建；
guijin=亮牌交付全现成原语；chengxin 单技暂缓）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js linghuo/guijin/chengxin）

- `linghuo`：trigger global phaseEnd round:2 check 询问——filter
  event.player.getStat("damage") 且 ≠自己；content 对 trigger.player
  damage("fire")。
- `guijin`：round:3 enable phaseUse——get.cards(4) 亮牌堆顶 4 张 →
  chooseCardButton [1,N] 选若干 → chooseTarget 交一名角色（获得者
  skip("phaseDraw") 跳下次摸牌阶段 + guijin2 临时技）→ goto 1 循环
  （可取消→剩余弃置）。
- `chengxin`：round:4 enable chooseToUse（dying）——濒死时对濒死者
  recover 至 1 + chengxin2（damageBefore cancel 至其阶段后）——技能型
  救援（非牌），原生 DyingSaveMachine 仅询问桃类牌 → 通道缺口暂缓。

## 改动（分开陈述）

- **状态四字段（PlayerState）**：`turn_damage: u8`（本回合已造成伤害
  量——上游 getStat("damage") 同口径）、`linghuo_ready_round: u32`、
  `guijin_ready_round: u32`（round:N 冷却=触发后 N 轮，**登记差异**：
  以轮数冷却近似上游 round 标签）、`skip_next_draw: bool`（归烬牌获
  得者跳下次摸牌阶段）。存档 v25→**v26**。
- **伤害统计钩**：commit_damage 实扣后来源 `turn_damage += 实扣量`
  （护甲/机甲吸收后；来源≠目标）；**清零点=各玩家自己 Judge begin
  （turn_opened 块）**——非 commit_turn_ended 遍历清：Phase{begin:
  false} 帧的 Commit（含收束+下一回合开始）先于其 AfterTriggers 跑，
  收束清零会让灵火 filter 读到已清零的统计（本轮实测踩坑，机制登记）。
- **跳摸钩**：commit_phase Draw begin——`skip_next_draw` 消费跳过固定
  摸 2（兵粮寸断 TurnState 版同区；跨玩家版=PlayerState 字段）。
- **宿主行为**：`XianjianLinghuo`（tag 171，global PhaseEnd 确认门：
  filter 他人 End 末+其 turn_damage>0+自己冷却就绪；Act 置
  ready=round+2+对阶段归属者 1 火伤）+ `XianjianGuijin`（tag 172，主
  动三段循环：Act 亮 4 张（空堆弃堆洗回兜底）+置 ready=round+3→
  SelectCard 收牌（可取消=弃置剩余收束）→TargetChoice 交付任意角色
  （hand.push+skip_next_draw+CardsGiven）→Yield 回 Act 循环；**登记差
  异**：逐张交付，上游 chooseCardButton 可批量）。六表+ALL 170→172。
- **登记暂缓**：chengxin（濒死技能型救援通道缺口，mapping null+reason）。
- **cards.json**：明绣行（shu 3 女）+ rehash 两步；**版本** 159→160；
  a0 追加 171/172。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`mingxiu_linghuo_round_cooldown_
  and_fire`（r1 触发+冷却置位 round+2；r2 各阶段末不触发；r3 首个满
  冷却伤害者 End 末再触发+续置；终态血量核对）+
  `mingxiu_guijin_deal_and_skip_draw`（亮 4→逐张交付→获得者 skip 标
  记→取消弃余→获得者下回合跳摸消费）。
- **全量通过**：engine-native **485 passed / 0 failed / 1 ignored**
  （v159 基线 483 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 159→160 + 四字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **机制坑（登记）**：①Phase{begin:false} 帧 Commit 先于 AfterTriggers
  ——阶段末触发技读到的是收束+下一回合开始后的世界（回合级统计清零
  必须放各玩家自己回合开始，不能放收束）②阶段末触发的可见询问点=该
  End→收束 NP 期间（commit 已推进、门挂起在其 AfterTriggers）③全局
  PhaseEnd 触发的 filter 若依赖「阶段归属者回合内统计」，统计须按玩
  元素各自清零（本人 Judge begin）。

## 剩余

- xianjian 其余 17/36 将逐武将纵切；chengxin 通道缺口（濒死技能型救
  援）与其他缺口族一并待排；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
