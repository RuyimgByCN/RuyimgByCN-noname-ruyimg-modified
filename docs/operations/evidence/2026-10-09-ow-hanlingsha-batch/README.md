# U05 xianjian 第七切片：韩菱纱探囊（规则 v148，2026-10-09）

xianjian 第七武将切片：**韩菱纱**（pal_hanlingsha，qun 3 女）——探囊
（tannang，出牌阶段限一次）单技闭环；**tuoqiao 暂缓**（黑牌当石会粉
——石会粉为古剑卡片，跨包牌种依赖）。xianjian 包 7/36 将入包（7 闭
环）。前置：
[xianjian 第六切片](../2026-10-09-ow-linyueru-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js tannang）

- enable chooseToUse usable:1；filterCard suit==="club"（手牌）；viewAs
  shunshou；mod targetInRange（shunshou 恒可达）。

## 改动（分开陈述）

- **代码已实现**：`XianjianTannang`（tag 149 目录标记技）+ **PlayCard
  view-as 链新分支**（奇袭同级：持技者梅花手牌 kind 路由为 Snatch）+
  **Snatch 距离豁免**（实体牌非顺牵 && 持技→距离豁免=上游 targetInRan-
  ge 同口径；实体顺牵不受影响）。
- **登记差异**：上游 chooseToUse 双形态（杀/顺牵二选）原生无拆分入口
  ——梅花杀路由为顺牵（登记：杀形态不可选）。
- **登记暂缓**：tuoqiao（石会粉跨包牌种）。
- **转换包**：cards.json 增 韩菱纱（qun 3 女，tannang mapping+tuoqiao
  暂缓 reason）+ npack 资产哈希重算。无新状态字段——存档 v16 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`hanlingsha_tannang_snatch_unlimited_
  range`（梅花借刀当顺牵打距离 2 的 2 号=豁免+获得一张牌）。
- **全量通过**：engine-native **469 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v147 基线 468 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 147→148 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：梅花杀被武圣/杀形态占用（view-as 链只对非杀类
  开放）——测试改用梅花借刀；借刀本体目标校验要求目标有武器（给 2
  号装连弩满足）。

## 剩余

- xianjian 其余 29 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
