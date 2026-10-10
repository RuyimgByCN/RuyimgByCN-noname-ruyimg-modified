# U05 swd 第十三批：羊皮卷（规则 v97，2026-10-08）

swd 子包第十三批转换：羊皮卷（trick，对自己使用）——选一种类别
（基本/锦囊/装备），发现该类别一张牌入手。前置：
[swd 第十二批](../2026-10-08-u05-swd-batch12/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/swd/）

- `list.js:40/53`：`["spade", 9, "yangpijuan"]`、`["club", 7,
  "yangpijuan"]`（♠9/♣7 两张）；`translate.js:77`：`yangpijuan:
  "羊皮卷"`，info「出牌阶段对自己使用，选择一种卡牌类别，发现一张该
  类别的卡牌」。
- `card.js` `yangpijuan`：type trick、enable、toself、filterTarget
  target==player、modTarget；content 三步——①chooseControl
  "basic"/"trick"/"equip"（AI 推荐为默认项）；②`get.inpile(类别,
  "trick")` + `randomGets(3)` 随机亮 3 张；③chooseButton(true) 必选
  一张，`game.createCard(...)` 凭空造牌 gain 入手。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Yangpijuan`（编码 117：archive enc/dec、
  a0 variant/ALL 118 同步）+ `HostBehavior::SwdYangpijuanCard`（tag 82，
  HostBehavior::ALL 82 守卫过）三段询问机：Act 问类别（PickTag
  0=基本/1=锦囊/2=装备，上游 chooseControl 顺序）→ SelectCard 牌堆
  部分 Fisher-Yates 随机亮该类别 ≤3 张（种子 RNG 真消除口径）→
  TargetChoice 选一张从牌堆入手（CardsDrawn 日志，五谷同款）。PlayCard
  无目标门控 + kernel intrinsic 臂无懈窗口包装（流星火雨同款）。
  snap 派生两段（类别静态 Tag；发现选项读快照 `cards`——
  `active_ask_host` 增 `cards` 参数）。
- **登记差异（上游 createCard vs 原生牌堆发现）**：上游凭空造牌（不占
  牌堆实体），原生无自由牌 id——发现牌从牌堆实体取出；亮出的 3 张在
  择定前留在牌堆（挂起态区域完整，御夫录同口径）。类别池按目录牌种
  枚举：基本（Sha/FireSha/LeiSha/Shan/Tao/Jiu/Hufu/Caoyao/Tianxianjiu）、
  锦囊（26 种通用锦囊含 swd 三种）、装备（equip_slot 非空）——gwent/
  炉石/古剑机关/食物/欢乐等专用牌种不入池（边界登记）；牌堆无该类别
  牌则空发。
- **AI 不迁移**：上游 content step0 的 AI 推荐与 chooseButton ai 为决策
  逻辑，原生以玩家询问代替（各批同口径）。
- **转换包**：cards.json 增 ♠9/♣7 两张羊皮卷（现 22 张）+
  source_hash/logic_hash/asset_hash 三哈希重算。

## 验收口径

- **回归已加入**：`swd_yangpijuan_discover_from_deck`——非空目标拒绝
  （IllegalTargets）→ 无懈窗口过 → 类别询问（存档往返①）→ 发现
  1–3 张（选项 ⊆ 牌堆且全为所选类别；存档往返②选项一致）→ 择定牌
  入手且离牌堆 → 守恒。
- **全量通过**：engine-native **412 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 96→97 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（羊皮卷不在标准场景，纯版本位移）。
- **发牌敏感**：牌堆构成 +2 张，swd 计数断言 20→22 三处（既有注入
  已全区域 purge，未再现多区域/缺失报错）。
- **剩余**：swd 其余卡种（蓝格怪衣/带火风轮/霞云彩绫等）逐批转换；
  ow/xianjian/mtg/zhenfa 同前置。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
