# U05 mtg 第六批：沙漠绿洲（规则 v105，2026-10-08）

mtg 子包第六批转换：沙漠绿洲（♥6 land 地）——换地后从牌堆基本牌发现
一张（亮 ≤3 三选一）；地图效果锁定：基本牌不计入手牌上限（需弃数按
非基本手牌数计，弃牌选项排除基本牌）。前置：
[mtg 第五批](../2026-10-08-u05-mtg-batch5/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:7`：`["heart", 6, "mtg_shamolvzhou"]`（♥6 一张）；
  `translate.js:26-27`：沙漠绿洲——「发现一张基本牌。地图效果：你的
  基本牌不计入手牌上限」。
- `card.js`：content 换地 + `player.discoverCard(get.inpile("basic"))`
  （三选一）；`skill.js mtg_shamolvzhou_skill`：mod ignoredHandcard
  （basic）——基本牌不计入手牌数。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Shamolvzhou`（编码 126）+
  `HostBehavior::MtgShamolvzhouLand`（tag 90，HostBehavior::ALL 90 守
  卫过）两段机（换地 → 牌堆基本牌 Fisher-Yates 亮 ≤3 张（留牌堆）→
  三选一入手；无基本牌空发）+ kernel intrinsic 臂 + PlayCard 地臂并入
  + snap 派生臂（选项读快照 cards）。**地图效果两处**：①弃牌阶段
  limit 计算处——current_land=沙漠绿洲时需弃数按非基本手牌数计（上游
  ignoredHandcard basic 同口径）；②DiscardToLimitMachine 选项过滤排除
  基本牌（不计入即不被要求弃）。
- **登记差异**：类口径以 `yangpijuan_class`（本批转 pub）为准——杀/
  闪/桃/酒/swd 基本牌均计基本类；上游弃牌选择集是否排除基本牌归档
  不可完全核验——原生排除（语义等价：不计入即不要求弃）。
- **转换包**：cards.json 增 ♥6 一张（现 6 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_shamolvzhou_land_effect`——发现三选一（选项全
  为基本类、亮出态在牌堆、存档往返选项一致、选定入手离牌堆）→ 地图
  效果（体力压 1：上限 1，1 基本+2 锦囊 → 非基本 2>1 触发弃 1 且选项
  不含基本牌）。
- **全量通过**：engine-native **421 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 104→105 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（沙漠绿洲不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 6 张地（幽林 + 五镜像）逐批；zhenfa 14 张；
  ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
