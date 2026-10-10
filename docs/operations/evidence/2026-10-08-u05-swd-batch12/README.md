# U05 swd 第十二批：御夫录（规则 v96，2026-10-08）

swd 子包第十二批转换：御夫录（equip5 宝物）主动技「投掷」——出牌阶段
限一次，取一张武器牌（手牌/装备区，不弃置）交给一名其他角色：其受
1 点伤害并获得此武器。前置：[swd 第十一批](../2026-10-08-u05-swd-batch11/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/swd/）

- `list.js:31`：`["spade", 6, "yufulu"]`（♠6，一张）；
  `translate.js:314`：`yufulu: "御夫录"`。**源码校正**：此前工作记录
  误作「伏符录 ♠5」，以源码为准改正（代码注释已全部修正，键名
  yufulu 本就正确）。
- `card.js:1936`：`yufulu` type equip / subtype equip5 / skills ["touzhi"]。
- `skill.js:2580` `touzhi`：equipSkill、`enable: "phaseUse"`、
  `usable: 1`、`filterCard` subtype equip1（武器）、`filter` 手牌+装备区
  有武器（"he"）、`discard: false` + `prepare: "give"`（给出不弃置）、
  `filterTarget` target≠player、content `target.damage();
  target.gain(cards, player)`（先伤害后给牌）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Yufulu`（编码 116：archive enc/dec、a0
  variant/ALL 81 同步；equip5 宝物占栏）+ `HostBehavior::SwdYufuluEquipSkill`
  （tag 81，HostBehavior::ALL index==tag 守卫过）三段询问机：Act 问武器
  （手牌/装备区）→ SelectCard 记录并问目标（其他存活角色）→ TargetChoice
  对目标 1 点普通伤害 + 武器交入其手牌（damage→gain 同序）。武器在
  TargetChoice 段才离开原区域（上游 prepare:"give" 同口径，保证存档
  归属核算各挂起态区域完整）。快照派生臂两段（snap.rs）、存档 enc/dec、
  PlayCard 穷尽臂（装配占位拒绝）同步。
- **潜偷 usable:1 补执行**（v92 批缺口，本批一并修正）：`host_skill_def`
  对 `SwdShentouEquipSkill | SwdYufuluEquipSkill` 设 `once_per_turn`
  （上游两者均 `usable: 1`；once_used 于拥有者回合开始重置既有路径）。
- **登记差异**：上游 check() 为 AI 决策（10 - get.value），原生不迁移
  AI 权重（与各批同口径）；御夫录选择自装备区的路径与手牌路径同臂实现，
  回归覆盖手牌路径。
- **转换包**：cards.json 增 swd_yufulu ♠6（现 20 张）+ source_hash/
  logic_hash/asset_hash 三哈希重算（rehash_npack + blake3 补注）。

## 验收口径

- **回归已加入**：`swd_yufulu_give_weapon_and_damage`——无武器拒绝
  （InvalidChoice）→ 武器询问（存档往返①恢复同一询问）→ 目标询问
  （存档往返②）→ 目标受 1 伤 + 武器到手（不弃置）→ 再发动
  SkillAlreadyUsed（usable:1）。
- **全量通过**：engine-native **411 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1），DEPS_EXIT=0。
- **golden**：版本 95→96 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（御夫录/潜偷均不在标准场景，纯版本位移）。
- **发牌敏感修复**：牌堆构成 +1 张后天仙酒/潜偷注入测试
  card_in_multiple_zones——注入改全区域 purge（既有陷阱清单同口径）。
- **剩余**：swd 其余卡种（羊皮卷/兰阁怪衣/岱火风轮/夏云彩绫等多步型）
  逐批转换；ow/xianjian/mtg/zhenfa 同前置。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
