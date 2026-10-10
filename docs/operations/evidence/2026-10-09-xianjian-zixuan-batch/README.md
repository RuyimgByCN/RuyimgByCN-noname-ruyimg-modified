# U05 xianjian 第二十一切片（规则 v162，2026-10-09）

**离场区基建批**：武将牌上置牌区（off_court）落地 + 紫萱（pal_zixuan，
shu 3 女）水云（shuiyun）+水云·救（shuiyun5，濒死救援通道首例）+忘忧
（wangyou/wangyou2）闭环；changnian 登记暂缓。前置：
[xianjian 第二十切片](../2026-10-09-xianjian-yuntianhe-batch/README.md)。

## 立项依据（缺口族合并指认的兑现）

v161 登记的「离场区（武将牌上置牌区）缺口族」本批落地基建：猎空
（ow shanxian/shanhui）、雪见 xshuangren、紫萱 shuiyun 同属此缺口——
基建完成后猎空/雪见可后续逐批解锁。**慕容紫英移出该族**（勘误）：xuanning
实为计数标记且 skill.js 内无获得路径（上游缺口，单独登记）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js shuiyun/wangyou/changnian）

- `shuiyun`：player phaseEnd direct——从手中选一张与武将牌上牌类型
  （get.type 三分类：基本/锦囊/装备）不重复的牌置武将牌上（storage 列
  表 ≤3）；onunmark 弃置。
- `shuiyun5`（subSkill）：enable chooseToUse（dying）——弃武将牌上一
  张牌令濒死者 recover 1 + storage.shuiyun_count++（小蛮 shenwu 觉醒
  链依赖）。
- `wangyou`：global phaseEnd direct——他人受伤获 wangyou3 临时技
  （wangyou2 global damageEnd silent）；有标记者存续时可弃一张手牌令
  标记者们各摸 1。
- `changnian`：dieBegin direct——死亡时移交 hej 全部牌+授 changnian2
  （结束阶段摸 1）——**死亡拦截钩+跨将运行时授技复合缺口暂缓**。

## 改动（分开陈述）

- **离场区基建**：`PlayerState.off_court: Vec<u32>`（实体牌守恒——
  离开全部正常区、随所有者存放；snap validate_core check_cards 纳入）
  + machine.rs `hand_to_off_court`/`off_court_discard` 助手 + engine
  `debug_off_court_place` 布阵助手。存档 v27→**v28**。
- **状态**：`PlayerState.shuiyun_count: u8`（水云救人计数——小蛮
  shenwu 链）、`wangyou3_round: u32`（受伤回合戳，0=无）。清零点：忘
  忧戳=各玩家自己 Judge begin（**登记差异**：上游 addTempSkill 获标回
  合结束过期——原生架构下 End 帧 Commit 先于 AfterTriggers，收束清零
  会在触发前抹除；保留期延长至标记者下回合开始）。
- **宿主行为**：`XianjianShuiyun`（tag 175，PhaseEnd optional:false+
  机内取消：首臂发问（类型去重选项）→SelectCard 收应答置离场区）+
  `XianjianShuiyun5`（tag 176，global Dying 确认门=**濒死救援通道首
  例**（chengxin 同族后续接入）：确认后必选离场区一张牌弃置→濒死者
  recover 1→shuiyun_count++）+ `XianjianWangyou`（tag 177，global
  PhaseEnd direct 机内取消：弃一张手牌令标记者们各摸 1，逐目标
  Yield）+ `XianjianWangyou2`（tag 178，global Damage after 锁定
  silent：受伤者获回合戳）——六表+ALL 174→178。
- **登记暂缓**：changnian（死亡拦截+跨将授技复合缺口，mapping
  null+reason）。
- **cards.json**：紫萱行（shu 3 女；shuiyun5/wangyou2 子技常驻实例化
  登记）+ rehash 两步；**版本** 161→162；a0 追加 175-178。

## 验收口径

- **回归已加入**：g1_wudian 3 例——`zixuan_shuiyun_off_court_and_
  type_dedup`（预置锦囊占类→选项排除同类型/可置基本+装备→置牌入离场
  区→满 3 关闭）+ `zixuan_shuiyun5_dying_rescue_channel`（他人濒死
  确认→必选离场区牌→回复 1→计数=1）+
  `zixuan_wangyou_roundstamp_and_draw`（标记者自己回合内受伤获戳→其
  End 末询问→弃牌令标记者摸 1→取消路径）。
- **全量通过**：engine-native **490 passed / 0 failed / 1 ignored**
  （v161 基线 487 +3）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 161→162 + 三字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：①触发机首步无应答坑第 4/5 次（水云/忘忧初稿又
  犯——定式已两次登记仍需默诵）②忘忧戳测试的受伤必须发生在标记者自
  己回合内（Judge begin 后），否则其 Judge begin 清零先于 End 末触发
  ③测试导航：Judge 起**收束=4 次 NP**（5 次到下家 Play），从 Play 起
  收束=3 次——混用即错位。

## 剩余

- xianjian 其余 15/36 将；小蛮（lingquan/shenwu——shuiyun_count 链已
  通，lingquan 需 roundNumber 门控+useCard 全局计数+觉醒授 shuiyun=
  flag 门控常驻实例）下批候选；离场区解锁的猎空/雪见后续排；chengxin
  （救援通道已通，可接入）重估；changnian 死亡拦截钩、响应链、
  createCard、Recover 前置窗口等缺口族待排；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next
根）。
