# U05 ow 纵切批十八：半藏音波（规则 v135，2026-10-09）

ow 纵切批十八：**半藏**（ow_banzang，qun 4 男）——音波（yinbo，出牌
阶段限一次）单技闭环；**bfengshi 暂缓**（shaBegin Math.random 概率直
击+countUsed 攻距+组技 bfengshi2/3/4）。上游 25 将已闭环第 17 将。前
置：[ow 纵切批十七](../2026-10-09-ow-laiyinhate-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js yinbo）

- enable phaseUse usable:1；filterCard suit spade（he 区）；content：
  targets=enemies 中有牌者 randomGets(3)（随机取 3）→ sort seat → 逐
  目标 he.randomGet 弃 1（addExpose 为 AI 面不迁移）。

## 改动（分开陈述）

- **代码已实现**：`WudianYinbo`（tag 132，usable:1 走命令层 once_used）
  ——activate 门控（he 黑桃≥1）→ SelectCard 黑桃必选 → Act：弃置后
  候选（其他存活且 he 非空）swap_remove 抽取式随机取 ≤3（龙帆同款），
  逐个 he 池随机弃 1（暴雪同口径）。
- **登记差异**：原型无阵营——上游 enemies=其他存活玩家（多场合阵营
  引入时需重审）。
- **登记暂缓**：bfengshi（随机直击+组技）。
- **转换包**：cards.json 增 半藏（qun 4 男，yinbo mapping+bfengshi 暂
  缓 reason）+ npack 资产哈希重算。无新状态字段——存档 v9 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`banzang_yinbo_random_discard_
  three`（黑桃弃牌发动/4 人全有牌时随机 3 名各弃 1/usable:1
  SkillAlreadyUsed）。
- **全量通过**：engine-native **456 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v134 基线 455 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 134→135 入 canonical_hash——14 场景全 mismatch
  （纯版本位移），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：无（链钩/暴雪/龙帆三先例组合即绿）。

## 剩余

- ow 其余 7 武将逐武将纵切；xianjian 36 武将；开局事件基建、虚拟
  useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（14 文件，相对 noname-next 根）。
