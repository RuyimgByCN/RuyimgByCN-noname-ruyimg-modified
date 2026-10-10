# U05 xianjian 第十切片：李逍遥御风（规则 v151，2026-10-09）

xianjian 第十武将切片：**李逍遥**（pal_lixiaoyao，qun 4 男）——御风
（xjyufeng，弃牌触发锁定每回合限两次）单技闭环；**tianjian 暂缓**（vi-
ewAs 万箭齐发——杀牌当 AOE+tianjian_discard 伤害归属，AOE 虚拟用牌+归
属双缺口）。xianjian 包 10/36 将入包（10 闭环）。前置：
[xianjian 第九切片](../2026-10-09-ow-leiyuange-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js xjyufeng）

- trigger loseEnd forced usable:2——filter cards 含 original=="h"（手牌
  区失去）&& 手牌<2 → draw(2-countCards("h"))。

## 改动（分开陈述）

- **代码已实现**：`TurnState.xjyufeng_used: u8`（每回合限两次——存档
  **v17→v18**；两分支显式复位）+ `XianjianXjyufeng`（tag 154，CardsDi-
  scarded 帧触发 forced=隐匿同位）——filter（弃置含任意牌 && 手牌<2 &&
  本回合<2 次）→ Act：置次数+摸（2-手牌数）张。
- **登记差异**：上游 loseEnd original=="h"，原生按弃置路径 CardsDiscar-
  ded 覆盖（隐匿同款登记）。
- **登记暂缓**：tianjian（viewAs 万箭 AOE+伤害归属双缺口）。
- **转换包**：cards.json 增 李逍遥（qun 4 男，xjyufeng mapping+tianjian
  暂缓 reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`lixiaoyao_xjyufeng_draw_to_two_
  twice_per_turn`（弃 1 剩 1→补至 2/新回合计数复位断言）。
- **全量通过**：engine-native **472 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v150 基线 471 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 150→151 + xjyufeng_used 零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：首脚本 host.rs 锚点失败（尾变体已是 Song 非 Tan-
  nang——枚举尾随批次增长）——重探尾锚后六表全落。

## 剩余

- xianjian 其余 26 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
