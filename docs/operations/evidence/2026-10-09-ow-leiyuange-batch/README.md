# U05 xianjian 第九切片：雷元戈风雅颂（规则 v150，2026-10-09）

xianjian 第九武将切片：**雷元戈**（pal_leiyuange，qun 4 男）——风/雅/
颂三计数器技全闭环（无暂缓）。xianjian 包 9/36 将入包（9 闭环）。前
置：[xianjian 第八切片](../2026-10-09-ow-wangpengxu-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js feng/ya/song）

- 三技同构：init 计数=0；累计 2 次后第 3 次触发强化并清零。风=drawBe-
  gin num++（每三次摸牌多摸 1）；雅=damageBegin num--（每三次受伤-1，
  num>0 才减）；颂=source damageBegin num++（每三次造成伤害+1）。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.feng_count/ya_count/song_count: u8`（存
  档 **v16→v17**）+ `XianjianFeng`（tag 151，CardsDrawn after 触发）—
  —计 2 后本次**额外摸 1** 并清零（**登记差异**：上游 drawBegin 前置
  改数量，原生 after 补摸——净效果同为每三次摸牌多摸一张）；
  `XianjianYa`（tag 152，DamageBefore forced）——计 2 后本次受伤 -1
  （saturating_sub，前置窗口直写）；`XianjianSong`（tag 153，DamageBe-
  fore forced source scope）——计 2 后本次伤害 +1（前置窗口直写）。
- **登记差异**：风为 after 补摸等价（其余两技一一对应）。
- **转换包**：cards.json 增 雷元戈（qun 4 男，三技 mapping，无暂缓）
  + npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`leiyuange_feng_ya_song_counters`
  （颂：三轮杀 6→5→4→2（第 3 次 +1）/雅：三轮火杀 6→5→4→4（第 3 次
  -1→0）/计数清零断言）。
- **全量通过**：engine-native **471 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v149 基线 470 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 149→150 + 三零值字段入 canonical_hash——14 场景全
  mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：风=CardsDrawn after 补摸（drawBegin 原生无帧，
  登记差异）；龙翔/雅测试驱动循环需在每轮结束离开出牌阶段（下一轮
  drive 立即 break 同回合重玩→ShaLimit）。

## 剩余

- xianjian 其余 27 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
