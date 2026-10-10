# U05 xianjian 第四十六批切片（规则 v188，2026-10-10）

长卿天符系（ctianfu+ctianfu2+ctianfu3）闭环——**长卿升全闭环**（乱
剑系 v187+天符系 v188）。挂附基建首例（跨玩家离场区承载+双实例同
授/同清+伤害后窗）。前置：
[乱剑批](../2026-10-10-xianjian-luanjian-batch/README.md)。

## 选型依据

- **天符**：四件均有先例——挂附实体=离场区（双刃 v180）、runtime
  授技=liefeng 同款 skills.insert、伤害后窗=Damage 帧 AfterTriggers
  （射影同款事件）、死亡清理=Death 窗（v170）——本批选天符。
- **机关牌族**（dujiang/御甲/偃师）：需新卡种设计（牌表/张数/花色
  从上游机关牌子包挖）——留后续基建批。

## 上游语义核验（源码为准，…/character/xianjian/skill.js ctianfu/ctianfu2/ctianfu3）

- `ctianfu`：phaseUse usable:1——filterCard 闪、discard:false（不
  弃置=置武将牌上）、prepare:"give"、filterTarget=其他角色且
  !hasSkill("ctianfu2")；content=storage.ctianfu2=闪、ctianfu3=来
  源、addSkill ctianfu2。
- `ctianfu2`：source damageAfter forced——来源存活则来源对目标 1
  雷伤否则无来源雷伤；随机弃 he 一张；闪 discard；清 storage+移除
  技。
- `ctianfu3`：dieBegin forced（group 于 ctianfu2）——死亡清挂附。

## 改动（分开陈述）

- **宿主行为 1**：`XianjianCtianfu`（tag 213，phaseUse once_per_turn
  主动）——Act 选闪（可取消）→ SelectCard 选目标（其他角色且无挂
  附，无合法目标=闪留手不消耗）→ 跨玩家挂附（take_hand_card 来源+
  目标 off_court.push=hand_to_off_court 为同人同区不适用）+同授
  ctianfu2/ctianfu3 双实例+目标 ctianfu_source/ctianfu_card 记录。
- **宿主行为 2**：`XianjianCtianfu2`（tag 214，Damage 帧
  AfterTriggers forced=上游 source damageAfter 同口径）——filter（挂
  附者造成伤害+挂附在）→ 来源存活=来源对挂附者 1 雷伤（
  damage_data_with Thunder），否则无来源雷伤+随机弃 he 一张（手牌/
  装备池 RNG）+挂附闪入弃堆+清两实例两标记。触发即清=双向挂附递归
  链天然终止。
- **宿主行为 3**：`XianjianCtianfu3`（tag 215，Death 窗 forced）——
  挂附者死亡清挂附。**登记差异**：挂附闪入弃堆（上游凭空消失不弃
  置）。
- **状态**：`PlayerState.ctianfu_source: Option<u8>` +
  `ctianfu_card: Option<u32>`（入 canonical_hash+存档 tag 式 Option
  编码）。存档 v38→**v39**。
- **cards.json**：长卿 ctianfu null→host+追加 ctianfu2/ctianfu3
  host 条目（reason_ref=天剑 def 源同款）+ rehash 两步；**版本**
  187→188；a0 追加 213/214/215。

## 调试修正记录（工程面）

- 初稿挂附用 `hand_to_off_court(self.owner, c)`——语义为**同人**手
  牌→离场区，闪进了长卿自己离场区（回归离场区断言暴露）；改跨玩家
  两步（take_hand_card+目标 off_court.push）。
- 初稿只授 ctianfu2——死亡清理 ctianfu3 无实例不触发（场景 B 暴
  露）；上游 group 同授=两实例同授同清。

## 验收口径

- **回归已加入**：g1_wudian 1 例双场景——`changqing_ctianfu_attach_
  thunder_and_death_clear`（A：挂附三断言[闪离手入目标离场区+双实
  例+来源]→挂附者杀长卿→长卿 -1+挂附者反噬雷伤 -1+全清[弃堆含闪]
  ；B：挂附后目标被杀→死亡清挂闪入弃堆+标记清）。
- **全量通过**：engine-native **521 passed / 0 failed / 1 ignored**
  （v187 基线 520 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 187→188 + 挂附字段入 canonical_hash——14 场景
  全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- **长卿升全闭环**。xianjian 半闭环将余：居十方（御甲/偃师=机关牌
  种）、越今朝（雾魂=额外回合+全局复原）、及各登记差异技（dujiang/
  xiaoyue/tanhua/yanzhan viewAs/danqing/xuanning 获得路径/炎杀授
  技）；长期项不变（zhenfa 五阵法、整包验收 0/37 等）。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
