# U05 ow 纵切批十九：奥丽莎强固（规则 v136，2026-10-09）

ow 纵切批十九：**奥丽莎**（ow_orisa，wu 4 女）——强固（qianggu）+
强固守卫（qianggu2）双技一体闭环；**woliu 暂缓**（回合结束选 1-2 目
标共挂 woliu2 组距离/杀限定+dieBegin/phaseBegin 清除组）。上游 25 将
已闭环第 18 将。前置：
[ow 纵切批十八](../2026-10-09-ow-banzang-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js qianggu/qianggu2）

- `qianggu`：phaseUse usable:1；弃 2 张 he（selectCard 2）→
  changeHujia(2)+addTempSkill("qianggu2")。
- `qianggu2`：target useCardToBefore forced——他人对持有者使用杀时使
  用者 chooseToDiscard 一张基本牌，否则杀对持有者无效；至 phaseBegin
  （自己下回合开始）到期。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.qianggu_active: bool`（存档 **v9→v10**）
  + `WudianQianggu`（tag 133，usable:1；弃 2 张 he 必选→+2 护甲
  （change_hujia 原语）+置位）+ `WudianQiangguGuard`（tag 134，CardUsed
  after 窗口触发=**筑城守卫同款 cancelled 写回**）——filter（他人杀指
  向持有者 && 生效中）→ Act 问使用者：选一张基本牌=弃置放行
  （Sha/Shan/Tao/Jiu 四种=上游 get.type==="basic" 同口径）；Pass/无基
  本牌=杀无效（Intrinsic 按 cancelled 跳过结算）。解除=自己回合判定
  阶段开始（commit_phase Judge begin 清位——上游 phaseBegin 到期同口
  径；翻面跳回合路径的 Judge begin 同样清位）。
- **登记差异**：守卫询问选项固定 Pass+基本牌（上游 chooseToDiscard
  可取消=空弃）；强固置位后对 AOE 类杀（无）不适用——仅指向性杀。
- **登记暂缓**：woliu（组距离/杀限定子技+清除组）。D.Va 全系（机甲独
  立血条+changeHp 拦截链）暂缓（立项核验，未入包）。
- **转换包**：cards.json 增 奥丽莎（wu 4 女，双技 mapping+woliu 暂缓
  reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`orisa_qianggu_armor_and_guard_
  nullify`（弃 2→甲 0→2+置位/1 号回合杀 0 号→守卫询问 observer=使用
  者/Pass→杀无效体力不变/自己判定阶段开始解除）。
- **全量通过**：engine-native **457 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v135 基线 456 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 135→136 + PlayerState.qianggu_active 零值字段入
  canonical_hash——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：守卫询问发生在攻击者回合（0 号收束需三次
  NextPhase 后 1 号才开局——NotYourTurn 两次修正）。

## 剩余

- ow 其余 6 武将（猎空/死神/D.Va/温斯顿/查莉娅/黑影）逐武将纵切；
  xianjian 36 武将；开局事件基建、虚拟 useCard 原语、U07 duoqi、
  zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
