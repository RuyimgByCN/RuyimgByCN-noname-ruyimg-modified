# U05 mtg 第四批：废墟（规则 v103，2026-10-08）

mtg 子包第四批转换：废墟（♣8 land 地）——换地后从弃牌堆发现一张牌
（亮 ≤3 三选一，选定才离弃堆）；地图效果锁定：回合开始（准备阶段）
随机将弃牌堆一张置于牌堆顶。前置：
[mtg 第三批](../2026-10-08-u05-mtg-batch3/README.md)。

## 上游语义核验（源码为准，archive/qita/sanguosha/noname/extension/杀海拾遗/card/mtg/）

- `list.js:6`：`["club", 8, "mtg_feixu"]`（♣8 一张）；
  `translate.js:11-14`：废墟——「从弃牌堆中发现一张牌。地图效果：准
  备阶段，随机将弃牌堆的一张牌置于牌堆顶」。
- `card.js`：content 换地 + `player.discoverCard(ui.discardPile)`（三
  选一）；`skill.js mtg_feixu_skill`：trigger phaseBegin silent——随机
  弃堆一张移到 cardPile 顶（经 addGlobalSkill 全局生效，任何玩家回合
  开始触发）。

## 改动（分开陈述）

- **代码已实现**：`CardKind::Feixu`（编码 124）+
  `HostBehavior::MtgFeixuLand`（tag 87，HostBehavior::ALL 87 守卫过）
  两段机：Act 换地 → 弃牌堆部分 Fisher-Yates 亮 ≤3 张（牌留弃堆，羊
  皮卷同口径）→ 问三选一 → SelectCard 选定从弃堆入手（CardsDrawn 日
  志）；弃堆空空发。地图效果落 `commit_phase` Judge begin 臂
  （kernel.rs）：current_land=废墟且弃堆非空 → RNG 随机一张弃堆→牌堆
  顶（deck.push=顶侧惯例；上游 phaseBegin silent 同口径）。kernel
  intrinsic 臂 + PlayCard 地臂并入 + snap 派生臂（选项读快照 cards）。
- **登记差异**：上游 discoverCard 展示数量以库内实现为准（原生 ≤3 同
  羊皮卷口径）；地图效果 RNG 随机（真消除口径）；其余无。
- **转换包**：cards.json 增 ♣8 一张（现 4 张）+ 三哈希重算。

## 验收口径

- **回归已加入**：`mtg_feixu_land_effect`——首出发现三选一（选项 ⊆
  弃堆、存档往返选项一致、选定入手离弃堆）→ 地图效果观测（跨回合
  Judge 开始弃堆 -1/牌堆 +1）→ 空弃堆空发（清弃堆后注回再出，无询问）。
- **全量通过**：engine-native **419 passed / 0 failed / 1 ignored**；
  依赖包 **63/0**（content-runtime 33 + engine-ai 29 +
  content-native-loader 1）。
- **golden**：版本 102→103 入 canonical_hash——14 场景全 mismatch，重录
  后 replay_scenarios 2/2（废墟不在标准场景，纯版本位移）。
- **剩余**：mtg 其余 8 张地（依夏兰/幽林/沙漠绿洲 + 五镜像）逐批；
  zhenfa 14 张；ow/xianjian 武将管线。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（18 文件，相对 noname-next 根）。
