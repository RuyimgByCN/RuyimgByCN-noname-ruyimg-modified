# U05 xianjian 第十二切片：龙葵蝶舞+灵愈（规则 v153，2026-10-09）

xianjian 第十二武将切片：**龙葵**（pal_longkui，wu 3 女）——蝶舞（di-
ewu，出牌阶段主动）+灵愈（lingyu，回合结束确认）双技闭环；**fenxing
暂缓**（双形态变身——Math.random 切换 avatar+衍生技组，结构性依赖）。
xianjian 包 12/36 将入包（12 闭环）。前置：
[xianjian 第十一切片](../2026-10-09-ow-xuanxiao-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js diewu/diewu2/-
lingyu）

- `diewu`：phaseUse——filterCard sha、filterTarget 其他角色、discard:
  false prepare give（不弃置直接转移）→首次（无 diewu2）摸 1+addTempSk-
  ill diewu2（本回合一次性）。
- `lingyu`：player phaseEnd direct——filter 存在其他已受伤角色 → choos-
  eTarget 令其回复 1（可放弃）。

## 改动（分开陈述）

- **代码已实现**：`TurnState.diewu_draw_used: bool`（蝶舞首次摸牌标志
  ——存档 **v18→v19**；两分支显式复位）+ `XianjianDiewu`（tag 159，us-
  able 无限制——首次摸牌由 DrawUsed 收窄）两段询问机（杀必选→目标必
  选→give_hand_card 直接转移（discard:false prepare give 同口径）→首
  次摸 1）+ `XianjianLingyu`（tag 160，PhaseEnd owner 触发）——filter
  存在其他已受伤者 → Act 目标询问（可放弃）→回复 1。
- **登记差异**：diewu2 上游 addTempSkill（至回合结束）原生 TurnState
  bool 等价。
- **登记暂缓**：fenxing（变身结构性依赖）。
- **转换包**：cards.json 增 龙葵（wu 3 女，双技 mapping+fenxing 暂缓
  reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`longkui_diewu_give_and_lingyu_rec-
  over`（杀转移至 1 号手牌+首次摸 1 净 0/灵愈令 1 号（2/3）回复 1）。
- **全量通过**：engine-native **474 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v152 基线 473 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 152→153 + diewu_draw_used 零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：owner phaseEnd 任意阶段结束均触发（灵愈在 0 号
  Discard→End 即挂起，非 End→收束）；两段询问机 Act 臂收目标应答（温
  慧同款重构第三次应用）。

## 剩余

- xianjian 其余 24 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
