# U05 mtg 第九批：幽林（规则 v108，2026-10-08）

mtg 子包第九批转换：幽林（♣10 land 地）——换地后随机使用一张普通锦
囊；地图效果=第三例地附主动技「弃锦囊发现」——出牌阶段限一次弃一张
锦囊牌并从牌堆发现一张牌。前置：
[mtg 第八批](../2026-10-08-u05-mtg-batch8/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:4`：`["club", 10, "mtg_youlin"]`（♣10 一张）；
  `translate.js:19-21`：幽林——「随机使用一张普通锦囊牌，随机指定一
  个具有正收益的目标。地图效果：出牌阶段限一次，你可以弃置一张锦囊
  牌并发现一张牌」。
- `card.js`：content 换地 + `get.inpile("trick")` 随机排除 multitarget
  + `player.useCard({name}, target)`（虚拟牌，正收益 AI 判定）；
  `skill.js mtg_youlin_skill`：enable phaseUse/usable 1/filterCard
  trick → content `discoverCard()`（无参=全牌堆）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Youlin`（编码 129）+
  `MtgYoulinLand`（tag 94）两段机（换地 → 随机池内一张 → 随机其他存
  活目标 → **实体牌入手后 emit CardUsed**（commit_card_used 从手牌取
  牌的适配；内核各锦囊结算/无懈窗口自动复用））+
  `MtgYoulinDiscard`（tag 95）地附技三段机（手牌锦囊 → 弃 → 牌堆随机
  亮 ≤3 三选一入手）。sync_land_skill 第三映射；once_per_turn；双
  decl；snap 派生两臂；a0 守卫全套。
- **登记差异（最小闭环口径）**：随机池收窄为**决斗/无中生有**两种
  （内核结算已确证者；上游全普通锦囊池随各自内核结算覆盖逐批扩）；正
  收益目标判定为 AI 逻辑不迁移（随机目标，无懈窗口照常给对手反制）；
  上游虚拟牌使用 → 原生牌堆实体（守恒口径）。
- **转换包**：cards.json 增 ♣10 一张（现 9 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_youlin_land_and_discover`——无地无实例 → 出地
  随机使用（池内一张离牌堆进入结算、地生效）→ 地附技（锦囊弃 → 发现
  三选一×存档往返选项一致 → 入手）→ 限一次 → 换地撤销（出海岛后
  NoSuchSkill）。
- **全量通过**：engine-native **424 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 107→108 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（幽林不在标准场景，纯版本位移）。
- **剩余**：mtg 3 张地（藏宝海湾/邻地溪流/毒沼——依赖模式衍生牌或重
  流程，另评估）逐批；zhenfa 14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
