# U05 xianjian 第三切片：赵灵儿回梦+蛇蜕（规则 v144，2026-10-09）

xianjian 第三武将切片：**赵灵儿**（pal_zhaoliner，wei 3 女）——回梦
（huimeng）+蛇蜕（tianshe/tianshe2）三机**全闭环**（无暂缓——上游最
小组合 696 字符）。xianjian 包 3/36 将入包（3 闭环）。前置：
[xianjian 第二切片](../2026-10-09-ow-shenqishuang-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js
huimeng/tianshe/tianshe2）

- `huimeng`：recoverAfter frequent → draw(2)（登记差异：frequent 询问
  式，原生直发——摸牌无选择面，观察等价）。
- `tianshe`：damageBefore forced cancel（hasNature=属性伤害免疫）+group
  tianshe2。
- `tianshe2`：source damageAfter forced——hasNature && hp<maxHp →
  recover()。

## 改动（分开陈述）

- **代码已实现**：`XianjianHuimeng`（tag 143，Recover after 触发=灵雪
  同位直发）→ Act 摸 2；`XianjianTianshe`（tag 144，DamageBefore 前置
  窗口 forced）→ Act：prevented 直写（属性伤害无效——碎盾/急冻先例；
  传导事件同免疫=上游 hasNature 同口径）；`XianjianTianshe2`（tag 145，
  Damage after 触发 forced）→ Act：回复 1（filter 收窄已受伤）。
- **登记差异**：huimeng frequent 直发（无选择面观察等价）。
- **转换包**：cards.json 增 赵灵儿（wei 3 女，三技 mapping——tianshe2
  子技独立 mapping 同 ow 子技先例）+ npack 资产哈希重算。无新状态字段
  ——存档 v15 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`zhaoliner_huimeng_tianshe_family`
  （吃桃回复+回梦摸 2/蛇蜕火杀免疫体力不变/蛇蜕·息：火杀命中后回复 1）。
- **全量通过**：engine-native **465 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v143 基线 464 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 143→144 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：无（三机均为既有先例形态直用，首跑即绿）。

## 剩余

- xianjian 其余 33 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
