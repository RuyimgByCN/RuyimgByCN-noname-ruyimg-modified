# U05 mtg 第七批：林中尖塔（规则 v106，2026-10-08）

mtg 子包第七批转换：林中尖塔（♥6 镜像组 land 地）——换地后从牌堆武
器发现并**直接装备**一张；地图效果=第二例地附主动技「塔击」——装备
区有武器时，出牌阶段限一次将一张基本牌当杀使用（丈八蛇矛 declared-
Sha 同构）。前置：
[mtg 第六批](../2026-10-08-u05-mtg-batch6/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:12`：`["heart", 6, "mtg_linzhongjianta"]`（♥6，与沙漠绿洲
  同点数不同组）；`translate.js:32-35`：林中尖塔——「发现一张武器牌
  并装备之。地图效果：若你装备区内有武器牌，你可以将一张基本牌当作
  杀使用」。
- `card.js`：content 换地 + `discoverCard(equip1 过滤, "nogain", ...)`
  + step1 `player.equip(game.createCard(result.choice), true)`（直接装
  备）；`skill.js mtg_linzhongjianta_skill`：enable chooseToUse/
  viewAs sha/usable 1/viewAsFilter getEquip(1)+手牌有基本牌。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Linzhongjianta`（编码 127）+
  `MtgLinzhongjiantaLand`（tag 91）两段机（换地 → 牌堆武器亮 ≤3 三选
  一 → 选定**先入手再 emit Equipped**（commit_equip 从手牌取牌的原生
  适配；完整装配含同 kind 替换/装备技授予）→ 无武器空发）+
  `MtgLinzhongjiantaStrike`（tag 92）塔击三段机（手牌基本牌 → 其他存
  活目标 → `declared_kinds.insert(Sha)` + `card_use_as`——丈八蛇矛
  TargetRead 同构，素材牌留在手牌由杀结算消费；出闪/贯石斧/麒麟弓全
  流程重走）。**sync_land_skill 泛化**：映射表化（依夏兰→重铸、林中
  尖塔→塔击），第二例地附技零结构改动接入。activate 守卫（地+武器+
  手牌基本牌）、once_per_turn、双 decl、snap 派生三臂全套同步。
- **登记差异**：上游 discoverCard 的 "nogain"（不入手直接装备）原生
  以「入手→Equipped」两步适配（内部机制等价）；塔击上游
  chooseToUse（响应+出牌双入口）——原生仅出牌入口（同丈八登记）。
- **转换包**：cards.json 增 ♥6 一张（现 7 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_linzhongjianta_land_and_strike`——武器发现（选
  项全武器、亮出态在牌堆、存档往返）→ 选定直接装备（装备区在场/离牌
  堆）→ 塔击杀结算（1 号空手掉血 4→3、素材消耗）→ 限一次
  （SkillAlreadyUsed）→ 换地撤销（出海岛后 NoSuchSkill）。
- **全量通过**：engine-native **422 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 105→106 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（林中尖塔不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 5 张地（幽林/藏宝海湾/邻地溪流/冰河要塞/毒沼）
  逐批；zhenfa 14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
