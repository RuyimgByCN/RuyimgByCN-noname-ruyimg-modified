# U05 mtg 第八批：冰河要塞（规则 v107，2026-10-08）

mtg 子包第八批转换：冰河要塞（♦5 镜像组 land 地）——换地后摸二弃二
（强制）；地图效果锁定全局：使用杀时若有牌须弃置一张。前置：
[mtg 第七批](../2026-10-08-u05-mtg-batch7/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:9`：`["diamond", 5, "mtg_bingheyaosai"]`（♦5，与海岛同点数
  不同组）；`translate.js`：冰河要塞——「摸两张牌，然后弃置两张牌。
  地图效果：锁定技，每当你使用一张杀，若你有牌，则需弃置一张牌」。
- `card.js`：content 换地 + `draw(2)` + `chooseToDiscard("he", 2, true)`
  （true=强制不可取消）；`skill.js mtg_bingheyaosai_skill`：trigger
  useCard forced + filter card.name=="sha" && he 有牌 → content
  `chooseToDiscard("he")`（**无 true=可取消**——info「需弃置」与实现
  可取消有出入，以实现为准）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Bingheyaosai`（编码 128）+
  `MtgBingheyaosaiLand`（tag 93，HostBehavior::ALL 93 守卫过）两段机
  （换地 → 摸 2 → 问弃两张 he=手牌∪装备 min2 max2 强制——摸 2 保证手
  牌恒 ≥2 → 弃所选项，手牌直弃/装备离装撤销技）+ kernel intrinsic 臂
  + PlayCard 地臂并入 + snap 派生臂。**地图效果**落 commit_card_used
  Sha 臂（current_land 直读）：使用者手牌∪装备非空 → RNG 代抽弃一张
  （手牌/装备各半随机；装备离装撤销技）。
- **登记差异**：上游 chooseToDiscard 玩家自选+可取消——原生 RNG 代抽
  不可取消（青龙令珠 gainPlayerCard 代抽先例同口径；info「需弃置」与
  实现可取消的出入以实现为准后仍偏保守）。
- **转换包**：cards.json 增 ♦5 一张（现 8 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_bingheyaosai_land_effect`——摸二弃二（强制
  min2 max2、存档往返、弃后无挂起）→ 用杀随机弃 1（杀离手 1+代抽 1，
  目标正常掉血 4→3）→ 换地覆盖（出海岛后出杀仅杀本身离手，不额外弃）。
- **全量通过**：engine-native **423 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 106→107 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（冰河要塞不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 4 张地（幽林/藏宝海湾/邻地溪流/毒沼；后三者依赖
  模式衍生牌或重流程另评估）逐批；zhenfa 14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
