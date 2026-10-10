# U05 xianjian 第二十武将切片（规则 v161，2026-10-09）

云天河（pal_yuntianhe，wu 4 男）逐月（zhuyue）+贯日（guanri）双技闭
环；longxi（龙息）响应链缺口登记暂缓。前置：
[xianjian 第十九切片](../2026-10-09-xianjian-mingxiu-batch/README.md)。

## 候选选择（字节表评估）

紫萱 4668（shuiyun 置武将牌上=离场区缺口，小蛮 lingquan/shenwu 觉醒链
同被此卡）、月今朝 6338（体量最大）暂缓；已登记缺口族复核：雪见
（xshuangren 同属离场区）、柳梦璃（tanhua 需 Recover 前置窗口——Recover
帧无取消语义，新增缺口登记）、罗招嫣（fenglue 虚拟用牌）——取
**pal_yuntianhe 3981**（guanri 全现成原语；zhuyue 本体+杀白名单钩可
做；longxi 单技暂缓）。

## 上游语义核验（源码为准，…/character/xianjian/skill.js zhuyue/guanri/longxi）

- `zhuyue`：enable phaseUse usable:1——弃一张非基本牌（filterCard
  type≠basic）→ 指定至多两名（selectTarget [1,2]）有牌其他角色各随机
  弃一张（storage.zhuyue 记录）；mod：本回合杀 targetInRange 无距离+
  selectTarget -1+playerEnabled 只能指定逐月目标；zhuyue2 phaseUseEnd
  silent 清 storage。
- `guanri`：limited 一次性 enable phaseUse——弃两张红手牌（selectCard
  2 filterCard 红）→ loseHp() → 指定体力≥自己的其他角色 damage(2,
  "fire") → 存活则弃其装备区所有牌。
- `longxi`：chooseToRespondBegin/chooseToUseBegin forced——响应/使用询
  问前置从牌堆顶 2 张获得可用牌（响应询问链改写=缺口）。

## 改动（分开陈述）

- **状态**：`TurnState.zhuyue_targets: Vec<u8>`（本出牌阶段杀白名单，
  出牌阶段结束清——上游 zhuyue2 phaseUseEnd 同口径）+
  `PlayerState.guanri_used: bool`（limited 一次性持久标志）。存档
  v26→**v27**（TurnState 段 Vec 追加+PlayerState 段 bool 追加）。
- **杀校验钩（engine.rs PlayCard 校验块）**：白名单非空时所有目标须在
  名单（IllegalTargets 拒绝）；距离豁免 `zhuyue_free`（杀向逐月目标无
  视距离——贴房绅视/堡垒哨卫/飞刃同区追加）。清钩：commit_phase Play
  !begin 清白名单（月行清钩同区）。
- **宿主行为**：`XianjianZhuyue`（tag 173，active 三段：弃非基本牌→
  至多两名有牌目标（min1 max2）→逐目标随机弃一张（手牌优先退随机装备
  =蓝格怪衣同款登记差异；Yield 重查存活）+白名单装填）+
  `XianjianGuanri`（tag 174，active 三段：弃两红手牌→LoseHp 1+对体力
  ≥自己者 2 火伤→Yield 后存活则弃其装备区所有牌（revoke_equip_skills
  逐张）；guanri_used 置位）——六表+ALL 172→174；双技 usable:1 走命
  令层 once_per_turn。
- **登记暂缓**：longxi（响应询问链缺口，mapping null+reason）。
- **新增缺口登记**：Recover 前置窗口（罗招嫣 tanhua——Recover 帧无取
  消语义）；离场区（武将牌上置牌区）缺口族合并指认：猎空/雪见双刃/紫
  萱水云/慕容紫英 xuanning——做此基建可解锁四将+小蛮链。
- **cards.json**：云天河行（wu 4 男）+ rehash 两步；**版本** 160→161；
  a0 追加 173/174。

## 验收口径

- **回归已加入**：g1_wudian 2 例——`yuntianhe_zhuyue_whitelist_and_
  clear`（弃非基本→两名目标各随机弃→白名单装填→杀非白名单拒绝/白名
  单距离豁免放行（座位距 2 命中）→出牌阶段结束白名单清空）+
  `yuntianhe_guanri_once_fire_and_equip_strip`（弃两红→失 1 体力→2
  火伤→装备区全弃→一次性置位→再发动拒绝）。
- **全量通过**：engine-native **487 passed / 0 failed / 1 ignored**
  （v160 基线 485 +2）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 160→161 + zhuyue_targets/guanri_used 入
  canonical_hash——14 场景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 16/36 将逐武将纵切；**离场区基建**成为最大解锁点（猎
  空/雪见/紫萱/小蛮链/慕容紫英），建议独立立项；响应链（longxi/张目/
  龙息族）、Recover 前置窗口（tanhua）、createCard（xuanmo/dujiang/机
  关牌）等缺口族待排；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
