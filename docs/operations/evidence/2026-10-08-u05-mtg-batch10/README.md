# U05 mtg 第十批：邻地溪流 + 两张暂缓评估（规则 v109，2026-10-08）

mtg 子包第十批：邻地溪流（♠11 镜像组 land 地）转换 + 藏宝海湾/毒沼
**暂缓评估定案**——mtg 12 张上游清单至此全覆盖（10 转换 + 2 暂缓）。
前置：[mtg 第九批](../2026-10-08-u05-mtg-batch9/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:10`：`["spade", 11, "mtg_lindixiliu"]`（♠11）；
  translate.js：邻地溪流——「获得一张♥牌。地图效果：锁定技……」+
  content：牌堆♥入手 + chooseToDiscard("he",[1,∞),{suit:♥}) 可取消 →
  每张随机敌方 `addJudge(createCard(inpile("delay") 中未持有者))`。
- **藏宝海湾（♣12）暂缓**：`enable` 守卫 `lib.cardPack.mode_derivation`
  为空即 return false——原生无模式衍生牌概念，上游自身禁用；效果完全
  围绕衍生包/武将包选牌。**口径：登记暂缓（上游禁用态），不编码。**
- **毒沼（♣8）暂缓**：效果 `createCard("du")`——毒为 mode_derivation
  衍生牌实体，原生无对应实体与中毒结算。**口径：登记暂缓（依赖衍生
  牌实体），不编码。**

## 改动（分开陈述）

- **代码已实现**：`CardKind::Lindixiliu`（编码 130）+
  `MtglindixiliuLand`（tag 96，HostBehavior::ALL 96 守卫过）两段机：
  Act 换地 → 牌堆♥首张入手（get.cardPile2 顺序同义）→ 手牌∪装备♥非
  空问弃任意张（可取消）→ SelectCard 逐张弃置，每张随机敌方判定区置
  入其未持有的延时锦囊（牌堆 Le→Bing→ShanDian 顺序=上游 inpile
  ("delay") 同序；判定区直接 push，JudgePhaseMachine 阶段取队；目标
  死亡则入弃堆）。kernel intrinsic 臂 + PlayCard 地臂并入 + snap 派生
  臂 + a0 守卫全套。
- **登记差异**：延时置入无专用日志（状态可观察，登记）；♥花色以目录
  suit 为准；取消语义=空选择（既有口径）。
- **转换包**：cards.json 增 ♠11 一张（现 10 张）+ 三哈希重算。
- **暂缓登记**：藏宝海湾/毒沼不入转换包（deferred 口径见上），台账
  U05 行同步。

## 验收口径

- **回归已加入**：`mtg_lindixiliu_land_effect`——出地♥入手 + 弃♥询问
  （可取消、存档往返选项数一致）→ 弃全部♥ → 敌方判定区置入延时锦囊
  （种类/归属断言）→ 取消路径（空选择判定区不变）。
- **全量通过**：engine-native **425 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 108→109 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（邻地溪流不在标准场景，纯版本位移）。
- **剩余**：mtg 12 张清单全覆盖（10 转换 + 2 暂缓）；zhenfa 14 张；
  ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
