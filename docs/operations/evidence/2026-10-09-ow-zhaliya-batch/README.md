# U05 ow 纵切批二十一：查莉娅屏障+应用（规则 v138，2026-10-09）

ow 纵切批二十一：**查莉娅**（ow_zhaliya，wei 4 女）——屏障（pingzhang，
global 伤害前置窗口 direct）+应用（owliyong，摸牌阶段开始触发锁定）
双技全闭环（无暂缓——空壳 pingzhang2/3=两 bool 标志等价）。上游 25 将
已闭环第 20 将。前置：
[ow 纵切批二十](../2026-10-09-ow-wensidun-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js
pingzhang/pingzhang_count/pingzhang2/pingzhang3/owliyong）

- `pingzhang`：global damageBegin direct——自己受伤（未用过自己项+有红
  桃 he）弃 1 张红桃 / 他人受伤（未用过他人项+有黑桃）弃 1 张黑桃 →
  trigger.num--（伤害 -1）+置对应标志（空壳 pingzhang2/3，各每回合一
  次）。
- `pingzhang_count`：phaseBegin silent——storage 清零后按两存在标志各
  ++ 并清标志（标志→计数转换）。
- `owliyong`：phaseDrawBegin forced——摸牌数 += storage.pingzhang（上
  回合屏障次数；storage 保持至下回合 phaseBegin）。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.pingzhang_self_used/pingzhang_other_used:
  bool`+`pingzhang_count: u8`（存档 **v11→v12**）+
  `WudianPingzhang`（tag 136，DamageBefore global 窗口 direct）——filter
  （量>0+分支花色/标志）→ Act 建对应花色弃牌选项（可取消=不发动）→
  SelectCard 弃置+前置窗口直写 `amount = amount.saturating_sub(1)`（安
  娜/猛击同机制）+置对应标志；`WudianOwliyong`（tag 137，PhaseBegin
  窗口+Draw/begin:true 过滤）——Act：摸 count 张（count 保持至下回合
  判定阶段=上游 storage 同口径）；**标志转计数**：commit_phase 自己
  Judge begin——count=两标志和并清（qianggu 清除同位扩展）。
- **登记差异**：上游 damageBegin num-- 可至负值？原生 saturating_sub(1)
  （下限 0——num>0 filter 下等价）；屏障对 AOE 伤害（global 每目标事
  件）逐事件判定=上游同口径。
- **转换包**：cards.json 增 查莉娅（wei 4 女，双技 mapping，无暂缓）+
  npack 资产哈希重算。
- **过程勘误**：三字段初版误加 TurnState（跨回合语义属 PlayerState）—
  —撤销改挂 PlayerState 后编译对齐。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`zhaliya_pingzhang_barrier_and_
  owliyong_draw`（自己受伤弃红桃→伤害 0 体力不变+自己项置位/自己判
  定阶段标志转计数+清除/摸牌阶段=基础 2+应用 1/计数保留至下回合）。
- **全量通过**：engine-native **459 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v137 基线 458 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 137→138 + PlayerState 三零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：owliyong 只读不清 storage（保持至下回合判定阶段
  ——断言初版写成消费即清）；三字段归属次勘误（TurnState→PlayerState）。

## 剩余

- ow 其余 4 武将（猎空/死神/D.Va/黑影）逐武将纵切；xianjian 36 武将；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
