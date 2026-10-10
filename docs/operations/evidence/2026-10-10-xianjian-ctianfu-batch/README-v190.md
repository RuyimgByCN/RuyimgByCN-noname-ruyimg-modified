# U05 swd 第十七批切片（规则 v190，2026-10-10）

木甲人（mujiaren）机关牌第二切片——同 kind 异行为按目录名分流首例
（Jiguan kind 下指路小虎/木甲人双行为）。机关牌池 2/4（余勾魂锣/机
关桶待切片）。前置：[指路小虎批](README-v189.md)。

## 上游语义核验（源码为准，…/card/swd/card.js mujiaren）

- `mujiaren`：type:"jiguan"、usable:1+updateUsable:"phaseUse"+
  forceUsable+wuxieable、selectTarget -1+filterTarget target===player
  （自目标）；content=弃目标（自己）全部非基本手牌（lose，
  `_triggered=null`=不触发弃牌事件）→num=1+弃掉数→机关牌池（排除
  derivation 衍生牌）randomGet createCard num 张入手（凭空造牌）。

## 改动（分开陈述）

- **宿主行为**：`SwdMujiarenCard`（tag 217，卡牌机器=无懈窗包装）
  ——kernel Jiguan 分派臂按目录名含 "mujiaren" 分流（**同 kind 异行
  为首例**）；step=mujiaren_used 置位→弃全部非基本手牌（静默移除，
  不 emit CardsDiscarded=上游 lose 不触发同口径）→实体发现 Jiguan
  池随机 1+N 张入手（排除本体，不足给余量）→本体弃置。
- **限次**：`TurnState.mujiaren_used: bool`（Judge begin 清；入
  canonical_hash+存档字节）。存档 v39→**v40**；PlayCard
  CardLimit=酒口径（上游 usable:1+updateUsable phaseUse 同口径）。
- **cards.json**：swd_mujiaren ♥4（花色手补）+rehash 两步；**版本**
  189→190；a0 追加 217。
- **swd 存量**：计数 26→27（三处断言）——16/16 全绿。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`swd_mujiaren_discard_nonbasic_
  discover_jiguan`（手牌[sha,wuzhong]+木甲人→自目标使用→无懈
  Pass→弃 wuzhong 留 sha→获小虎（池仅 1 张=不足给余量口径）→本体
  入弃堆→同回合再使用拒 CardLimit）。
- **全量通过**：engine-native **523 passed / 0 failed / 1 ignored**
  （v189 基线 522 +1；g1_shahai_swd 16/16）；依赖包 **63/0**。
- **golden**：版本 189→190+mujiaren_used 入 canonical_hash——14 场
  景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- 机关牌池 2/4（勾魂锣/机关桶待切片——勾魂锣=目标标记类，机关桶
  =?待核）；御甲/偃师解锁仍需计数基建（storage.yujia）；dujiang 毒
  牌种独立评估；长期项不变。

## 改动文件

见 [source-hashes-v190.txt](source-hashes-v190.txt)（16 文件）+
[test-logs-v190.txt](test-logs-v190.txt)。
