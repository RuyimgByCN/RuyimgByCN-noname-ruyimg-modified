# U05 swd 第十五批：蓝格怪衣「龙帆」（规则 v99，2026-10-08）

swd 子包第十五批转换：蓝格怪衣（equip5 宝物）主动判定技「龙帆」——
出牌阶段限一次，判定按花色：红桃回 1 体力/方片摸 1/梅花随机敌人随机
弃一张/黑桃无事。前置：
[swd 第十四批](../2026-10-08-u05-swd-batch14/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/swd/）

- `list.js:12`：`["diamond", 1, "langeguaiyi"]`（♦1 一张）；
  `translate.js:305`：`langeguaiyi: "蓝格怪衣"`。**源码校正**：技能名
  为「**龙帆**」（translate.js:308 `longfan: "龙帆"`），此前工作记录
  误作「乱番」；键名 longfan 本就正确。
- `card.js`：langeguaiyi equip5、skills ["longfan"]、onLose 清
  getStat.skill.longfan；`skill.js` `longfan`：equipSkill、
  enable phaseUse、usable 1、content 两步——judge(suit 提示函数)+
  switch 花色：heart `recover()`、diamond `draw()`、club
  候选=其他角色 he 非空→randomGet→`randomDiscard()`、spade 无。
- **onLose 核验**：清 getStat 为上游按统计计数的 usable 簿记，原生
  usable:1 由 once_per_turn（回合开始重置）等价承担，无需额外逻辑。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Langeguaiyi`（编码 120：archive enc/dec、
  a0 variant/ALL 121 同步；equip5 宝物占栏）+
  `HostBehavior::SwdLangeguaiyiEquipSkill`（tag 83，HostBehavior::ALL
  83 守卫过）单段判定机（无询问，无派生需求）：判定牌入弃堆（潜偷
  同款）后按花色分支——Heart `ctx.recover`（满血饱和为既有路径，上游
  recover 同钳制）、Diamond `ctx.draw_cards`、Club 随机敌人随机弃一张
  （候选=其他存活且手牌∪装备区非空，上游 he filter 同口径；种子 RNG）、
  Spade 无。activate 守卫无条件（上游无 filter）；usable:1 并入
  host_skill_def once_per_turn 臂（潜偷/御夫录同款）。
- **登记差异**：上游 judge 的 suit 提示函数为 AI 排序（不迁移）；
  `randomDiscard()` 默认弃牌区不可从归档源码核验（全库现有调用点均
  显式传 "h"）——原生取手牌优先、无手牌退随机装备（离装撤销装备技）。
- **转换包**：cards.json 增 ♦1 一张（现 25 张）+ source_hash/
  logic_hash/asset_hash 三哈希重算。

## 验收口径

- **回归已加入**：`swd_langeguaiyi_longfan_suit_branches`——六子局：
  红桃回 1（2→3）+ 限一次（SkillAlreadyUsed）、红桃满血无回复、方片
  摸 1、梅花敌人弃 1（3→2）、梅花无候选不弃、黑桃三不变。
  判定牌经 debug_set_deck_top_by 按花色可控。
- **全量通过**：engine-native **414 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 98→99 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（蓝格怪衣不在标准场景，纯版本位移）。
- **发牌敏感**：+1 张后护符/草药注入 card_in_multiple_zones——补全
  区域 purge（既有陷阱清单同口径）。
- **剩余**：swd 常规卡种已完（护符/流星火雨/草药/赤龙牙/天仙酒/吸血
  鬼指环/归有界/潜偷/星君眼+青龙令珠/贴房绅视/风雪刃/御夫录/羊皮卷/
  距离对/蓝格怪衣 15 批）；剩余卡种与 ow/xianjian/mtg/zhenfa 子包
  逐批转换。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（15 文件，相对 noname-next 根）。
