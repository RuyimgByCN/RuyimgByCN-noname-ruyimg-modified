# U05 xianjian 第二十七批切片（规则 v168，2026-10-09）

南宫煌（pal_nangonghuang，wei 3 男）摄灵（sheling）单技闭环；
zhaoyao/zhangmu 登记暂缓。**并交付：已通基建批量解锁优先级排序写入台
账**（用户点名的解锁面总览）。前置：
[织梦批](../2026-10-09-xianjian-zhimeng-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js sheling/zhaoyao/zhangmu）

- `sheling`：global ["useCardAfter","respondAfter","discardAfter"]
  frequent:"check" usable:3——自己为当前回合玩家且事件者非自己、事件
  牌 position "d"（入弃堆）→ 获得之（check 排除毒——毒牌种不在目录，
  登记不转）。
- `zhaoyao`：global phaseDrawBegin 拼点+置顶——**时序偏差暂缓**（原生
  PhaseBegin 触发晚于固定摸牌=摸后拼点功能性偏离）。
- `zhangmu`：chooseToRespondBegin/chooseToUseBegin 询问前置——响应链
  缺口暂缓。

## 改动（分开陈述）

- **状态**：`TurnState.sheling_used: u8`（每回合 3 次=上游 usable:3
  同口径；Judge begin 清）。存档 v30→**v31**。
- **宿主行为**：`XianjianSheling`（tag 187，TriggerMeta **三事件首
  例**：event=CardUsed+extra=[CardResponded, CardsDiscarded]）——filter
  （自己回合+他人事件+未满 3+事件牌在弃堆）→ Act 回收事件牌中在弃堆
  者（羊皮卷同款离堆入手+CardsDrawn 日志）+次数+1——六表+ALL
  186→187。**登记差异**：仅三事件面（使用/打出/弃置），其他入弃堆路
  径（装备替换等）不触发；毒排除不迁移。
- **cards.json**：南宫煌行（wei 3 男）+ rehash 两步；**版本** 167→
  168；a0 追加 187。

## 已通基建批量解锁优先级排序（写入台账）

1. **离场区（off_court，v162 已通）**：已解锁紫萱水云系/织梦暗牌；待
   消费=猎空 shanxian/shanhui（ow 来源）、雪见 xshuangren（余项=武器
   技能转移非放牌本身）。
2. **濒死救援通道（Dying 确认门模式，v162-v165 三例）**：shuiyun5/
   chengxin/shenmu 已接；后续任何「濒死时可选发动」技能照抄三段模式。
3. **runtime 授技（skills.insert，v163 首例）**：灵泉/织梦已用；任意
   addSkill/allowance 族可接。
4. **待建基建按解锁面排序**：
   ①虚拟 useCard 原语（解锁 5 将：姜云凡 liefeng/tianjian 授技、仙卿
   xiaoyue、王小虎 husha、罗招嫣 fenglue 部分、慕容紫英 qianfang）
   ②死亡拦截钩（Death 帧 BeforeTriggers 起步+cancelled 写回——
   TurnOverBefore 同款；解锁 changnian/月琦 shiying 部分）
   ③createCard（解锁夏侯瑾轩 xuanmo/苏媚 dujiang/绝世方机关牌族）
   ④moveCard 装备/判定区移动（月琦 liguang）
   ⑤Recover 前置窗口（罗招嫣 tanhua）
   ⑥响应询问链改写（云天河 longxi/南宫煌 zhangmu/姜云凡 xunying）
   ⑦mod 系（柳梦璃 tianxian/姜云凡 liefeng 白名单部分已通）。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`nangonghuang_sheling_three_event_
  reclaim`（CardResponded 面：杀→闪响应自动回收+次数；CardsDiscarded
  面：过河拆桥弃牌自动回收+次数）。
- **全量通过**：engine-native **498 passed / 0 failed / 1 ignored**
  （v167 基线 497 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 167→168 + sheling_used 入 canonical_hash——14 场
  景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 12/36 将（按上述待建基建排序逐批解锁）；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next
根）。
