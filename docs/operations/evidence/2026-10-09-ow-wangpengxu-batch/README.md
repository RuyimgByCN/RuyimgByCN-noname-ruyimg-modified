# U05 xianjian 第八切片：王蓬絮废墟（规则 v149，2026-10-09）

xianjian 第八武将切片：**王蓬絮**（pal_wangpengxu，wei 3 女）——废墟
（feixu，他人使用/打出闪后确认）单技闭环；**duxinshu 暂缓**（读心：
看目标手牌 chooseCardButton 选一张用自己的手牌替换——按钮选择+双向
换牌）。xianjian 包 8/36 将入包（8 闭环）。前置：
[xianjian 第七切片](../2026-10-09-ow-hanlingsha-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js feixu）

- trigger global [useCard, respond]（check 询问式）：filter card 为闪 →
  trigger.player.draw()。

## 改动（分开陈述）

- **代码已实现**：`XianjianFeixu`（tag 150，CardUsed 主事件+
  extra_events=[CardResponded] 双事件同技——废墟同时覆盖「使用闪」与
  「打出闪」（响应路径）；optional:true=上游 check 询问式同口径）——
  filter（牌为闪）→ 确认后 Act：使用者/打出者（machine() 分流带入）
  摸 1。
- **登记差异**：无新增（useCard/respond 双事件一一对应）。
- **登记暂缓**：duxinshu（按钮选择+双向换牌）。
- **转换包**：cards.json 增 王蓬絮（wei 3 女，feixu mapping+duxinshu
  暂缓 reason）+ npack 资产哈希重算。无新状态字段——存档 v16 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`wangpengxu_feixu_shan_draw`（0 号
  杀 1 号→1 号出闪→两位持有者（0/1）各触发一次确认→出闪者 1 号净
  +1 牌）。
- **全量通过**：engine-native **470 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v148 基线 469 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 148→149 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：global 双持有者各触发一次（确认门×2 依次应答）；
  距离 1 才可杀（目标选 1 号非 2 号）。

## 剩余

- xianjian 其余 28 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
