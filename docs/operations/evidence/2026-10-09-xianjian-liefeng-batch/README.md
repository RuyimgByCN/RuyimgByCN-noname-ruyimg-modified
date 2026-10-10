# U05 xianjian 第三十六批切片（规则 v178，2026-10-09）

姜云凡（pal_jiangyunfan，wei 4 男）疾风（liefeng）+天剑（tianjian）闭
环——**runtime 授技模式第二次消费**；xunying 响应链暂缓。前置：
[探息批](../2026-10-09-xianjian-xtanxi-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js liefeng/tianjian/xunying）

- `liefeng`：useCard forced——出牌回合内 countUsed 2/3/4 分别授
  yanzhan/tianjian/xjyufeng（xjyufeng 授予时补牌由御风自身 filter 兜
  底）。
- `tianjian`：enable phaseUse viewAs wanjian filterCard sha usable:1——
  弃一张手牌杀视为万箭齐发；子技 discard：source damageEnd forced 目
  标随机弃 he 一张。
- `xunying`：shaAfter chooseToUse 再出杀——响应链缺口暂缓。

## 改动（分开陈述）

- **宿主行为**：`XianjianLiefeng`（tag 200，CardUsed forced：出牌回合
  内 cards_used_play_phase=3/4 时 runtime 授天剑/御风实例（灵泉同
  款）；**第 2 张授炎杀 yanzhan viewAs 暂缓不授=登记差异**；次数单调
  递增=天然一次性）+ `XianjianTianjian`（tag 201，phaseUse usable:1
  主动：SelectCard 弃杀（必选）→**虚拟 AOE 原语**（use_virtual_card
  ArcheryAttack 全其他存活）→伤害目标随机弃 he 一张（discard 子技直
  连=**登记差异**：上游为 damageEnd 子技））——六表+ALL 199→201。
- **登记暂缓**：xunying（响应链）、liefeng 第 2 张授炎杀（yanzhan
  viewAs 暂缓联动）。
- **cards.json**：姜云凡行（wei 4 男；tianjian 常驻实例登记=授技 def
  源）+ rehash 两步；**版本** 177→178；a0 追加 200/201。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`jiangyunfan_liefeng_grant_and_
  tianjian_aoe`（0 号用 3 张无中→第 3 张授天剑实例→天剑弃杀→万箭
  AOE 1/2 号各-1→各随机弃 he 一张）。
- **全量通过**：engine-native **511 passed / 0 failed / 1 ignored**
  （v177 基线 510 +1）；依赖包 **63/0**（content-runtime 33 +
  engine-ai 29 + content-native-loader 1）。
- **golden**：版本 177→178 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。

## 剩余

- 姜云凡半闭环（liefeng/tianjian 闭环、xunying/炎杀授技暂缓）。
  xianjian 其余 5/36 将按缺口族；长期项不变。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（10 文件，相对 noname-next
根）。
