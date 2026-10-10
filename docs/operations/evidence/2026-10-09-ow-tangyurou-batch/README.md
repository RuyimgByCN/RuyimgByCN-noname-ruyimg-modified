# U05 xianjian 第十五切片：唐雨柔甜香去（规则 v156，2026-10-09）

xianjian 第十五武将切片：**唐雨柔**（pal_tangyurou，wu 3 女）——甜香去
（txianqu，伤害前置窗口确认每回合限一次）单技闭环；**qiongguang 登记
不转**（纯表现技——content 仅 line/expose/delay，无机制本体，表现层
noop）。xianjian 包 16/36 将入包（16 闭环）。前置：
[xianjian 第十四切片](../2026-10-09-ow-anu-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js txianqu）

- source damageBefore（自己 phaseUse 内）check 询问式——filter !hasSk-
  ill(txianqu2) → trigger.cancel()+draw(2)+recover()+addTempSkill(txi-
  anqu2)（每回合限一次）。

## 改动（分开陈述）

- **代码已实现**：`TurnState.txianqu_used: bool`（每回合限一次——存档
  **v21→v22**；两分支显式复位）+ `XianjianTxianqu`（tag 164，DamageBe-
  fore 前置窗口确认门 optional=true=上游 check 同口径）——filter（来源
  =持有者 && 未防止有量 && 自己出牌阶段内 && 本回合未发动）→ Act：pre-
  vented 直写（安娜支援同款）+摸 2+自己回复 1（上游 player.recover() 同
  口径；满血饱和）。
- **登记差异**：无（安娜支援同族：cancel+回复，加摸 2）。
- **登记不转**：qiongguang（纯表现层 noop）。
- **转换包**：cards.json 增 唐雨柔（wu 3 女，txianqu mapping+qionggua-
  ng 登记不转 reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`tangyurou_txianqu_prevent_and_repl-
  enish`（确认门→防止+自己回复 1（2→3）+摸 2 净+1/第二次伤害照常）。
- **全量通过**：engine-native **477 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v155 基线 476 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 155→156 + txianqu_used 零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。
- **测试坑（登记）**：甜香去回复的是**自己**（上游 player.recover() 非
  受伤者——初版断言误写受伤者不变）；版本号检出 153（v154/v155 批的
  bump 因锚点失败未落——重设 156 并复核全链）。

## 剩余

- xianjian 其余 20 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
