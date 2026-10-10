# U05 ow 纵切批十：美双技+翻面跳回合基建（规则 v127，2026-10-08）

ow 纵切批十：**美**（ow_mei，wei 3 女）——急冻（jidong）+ 暴雪
（baoxue）双技闭环；**bingqiang 冰墙登记暂缓**（座位相邻 changeTarget
+ bingqiang2-5 距离修饰子技族）。同批落地**翻面跳回合基建**（E03 翻面
原语自 v9x 以来首次获得回合语义）——上游 25 将已闭环第 9 将。前置：
[ow 纵切批九](../2026-10-08-ow-luxiao-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js baoxue/jidong/jidong2
+ library/skill.js _turnover + library/element/player.js turnOver）

- `turnOver(bool)`：bool=true 翻至背面/false 翻至正面（幂等——已处于
  目标朝向则事件直接 resolve）；无参=翻转。翻面为独立 game event
  （turnOverBefore/turnOverAfter 触发）。
- `_turnover`（回合跳过，library/skill.js）：player phaseBefore 强制技
  ——翻面者回合开始翻回+整回合取消（phaseSkipped）。
- `baoxue`：phaseUse limited（每局一次）；filter 黑色 he>0；
  selectTarget [1, 黑色数]（target≠自己）；contentBefore：storage 置位
  +showHandcards+弃全部黑色 he；content：逐目标 he 区 randomGet 弃 1
  +turnOver(true)；contentAfter：自己 turnOver(true)。
- `jidong`：global phaseEnd；filter hp===1 && 未翻面；content：turnOver
  （翻转）+recover(2)；step1 仍翻面则 addTempSkill jidong2 至
  turnOverAfter。
- `jidong2`：forced damageBefore cancel（受所有伤害无效）+ mod
  targetEnabled（不能被其他角色的牌指定为目标）。

## 改动（分开陈述）

- **代码已实现（基建）**：翻面角色回合跳过——`commit_phase` 对
  Judge begin 且 face_down 的玩家改为「发 TurnOverBefore（前置取消
  窗口）→ 直达 TurnEnded 收束」，整回合无阶段帧（不摸/不出/不弃），
  回合槽与轮次计数照常（座位推进不变）。**仅限回合开始**：回合中翻
  面（如急冻在自己弃牌阶段结束触发）不影响剩余阶段（上游 phaseBefore
  仅回合开始判定同口径）。
- **代码已实现（暴雪）**：`PlayerState.baoxue_used`（limited 跨回合持
  久）+ `WudianBaoxue`（tag 123）——activate 门控（已用/无黑牌拒绝）
  + SelectPlayers min1 max=黑色数（空选择=放弃，limited 不消耗）→ 弃
  全部黑色 he→逐目标 he 区 RNG 随机弃 1+翻至背面（幂等）→自己翻至
  背面。
- **代码已实现（急冻）**：`PlayerState.jidong_active` + `WudianJidong`
  （tag 122，PhaseEnd global 触发机）——翻面（内联）+回复 2+冰封置位
  ；**冰封双门控**：commit_damage 入口（目标冰封→DamagePrevented，置
  于各伤害增幅钩前=上游 damageBefore cancel 同口径；铁索传导独立事件
  同门控）+ PlayCard 目标校验（其他角色不可指定冰封者为牌目标——麻茶
  同型；AOE 自动指向不受影响=上游 targetEnabled mod 同口径）。
  **解除点=commit_turn_over_before 翻回正面**（上游 turnOverAfter 到期
  同口径；含跳回合的翻回）。
- **登记差异**：暴雪/急冻翻面内联（风雪刃 v95 先例，不经
  turnOverBefore 窗口——漩涡面具仅在 huanle 包，引入窗口化留待需要时）
  ；showHandcards 表现层不建模；AOE 无响应牌者不询问直接受伤（既有挂
  账）照常。
- **既有测试基线更新**：`caoren_jushou_flips_and_mianju_cancels_flip`
  ——跳回合基建后据守第二次确认由「翻回正面」改为「回合开始已被跳过
  翻回→再翻至背面」（断言与注释同步；`advance_to_play` 助手升级为门
  感知驱动）。
- **转换包**：cards.json 增 美（wei 3 女，jidong/baoxue mapping +
  bingqiang null+reason）+ npack 资产哈希重算（rehash_cards_asset +
  rehash_npack）。存档 **v7→v8**（PlayerState 两 bool 追加）。

## 验收口径

- **回归已加入**：g1_wudian 3 例——`mei_baoxue_limited_discard_and_
  flips`（放弃不消耗/黑牌数=目标上限/弃全部黑牌+随机弃+三方翻面/
  limited 拒绝/存档往返 v8）、`mei_flip_face_down_turn_skipped_and_
  flipped_back`（跳过回合翻回+无摸牌+回合槽消耗）、
  `mei_jidong_freeze_damage_immune_and_untargetable`（弃牌阶段结束触
  发翻面回 2/回合中翻面不跳剩余阶段/不可被指定 IllegalTargets/万箭照
  常问闪伤害无效/跳回合翻回即解除）。
- **全量通过**：engine-native **445 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v126 基线 442 +3）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 126→127 + PlayerState 两零值字段入 canonical_hash
  ——14 场景全 mismatch（标准场景无翻面面，纯版本/零值字段位移），
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：暴雪目标数受黑色牌数上限约束（种子 9911 黑牌
  数=1，选 2 目标即超 max 被拒）；急冻在自己回合内触发时解除点若挂
  在 TurnEnded 会被 3 帧后的回合收束立即清位——必须在 commit_turn_
  over_before 翻回时清（turnOverAfter 口径）。

## 剩余

- ow 其余 16 武将（含美 bingqiang 冰墙暂缓登记）逐武将纵切；xianjian
  36 武将；U07 duoqi/youzhan 暂缓；zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
