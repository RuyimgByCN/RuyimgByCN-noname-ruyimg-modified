# U05 xianjian 第十四切片：阿奴蛊舞（规则 v155，2026-10-09）

xianjian 第十四武将切片：**阿奴**（pal_anu，wu 3 女）——蛊舞（anwugu，
伤害后确认）+蛊舞·缚（anwugu2，三钩）双技闭环；**lingdi 暂缓**（距离
递增目标+花色去重存储——多钩存储系）。xianjian 包 14/36 将入包（14
闭环）。前置：
[xianjian 第十三切片](../2026-10-09-ow-muchanglan-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js anwugu/anwugu2）

- `anwugu`：source damageEnd check（询问式）——target != player && isIn
  && !hasSkill(anwugu2) → addSkill anwugu2（init storage=3）。
- `anwugu2`：mod cardEnabled（currentPhase==player && countUsed >=
  storage → 不能用牌）+ mod maxHandcard（-1）+ player phaseAfter silent
  （storage--，归 0 移除）。

## 改动（分开陈述）

- **代码已实现**：`PlayerState.anwugu_count: u8`（存档 **v20→v21**）+
  `XianjianAnwugu`（tag 162，Damage after 确认门）——受害者获蛊 3 阶段
  + `XianjianAnwugu2`（tag 163，三钩无机器）——①PlayCard WrongPhase
  后出牌受限（anwugu_count>0 && cards_used_play_phase>=3 → CardLimit；
  **登记差异**：countUsed 以出牌阶段计数近似——上游跨阶段计 countUsed）
  ②弃牌阶段 limit -1 ③自己每阶段结束 -1（归 0 解除）。
- **登记差异**：countUsed 出牌阶段近似（上游跨阶段）；蛊舞对同一目标唯
  一（filter anwugu_count==0=上游 !hasSkill 同口径）。
- **登记暂缓**：lingdi（距离递增+花色去重）。
- **转换包**：cards.json 增 阿奴（wu 3 女，双技 mapping+lingdi 暂缓
  reason）+ npack 资产哈希重算。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`anu_anwugu_afflict_and_limits`（伤
  害确认门→获蛊 3 阶段/蛊舞生效中 3 张可用第 4 张 CardLimit/阶段结束递
  减 3→2→0 归零解除）。
- **全量通过**：engine-native **476 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v154 基线 475 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 154→155 + anwugu_count 零值字段入 canonical_hash
  ——14 场景全 mismatch，重录后 replay_scenarios 2/2。

## 剩余

- xianjian 其余 22 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（16 文件，相对 noname-next 根）。
