# U05 xianjian 第六切片：林月如归元（规则 v147，2026-10-09）

xianjian 第六武将切片：**林月如**（pal_linyueru，wei 3 女）——归元
（guiyuan，出牌阶段限一次主动）单技闭环；**qijian 暂缓**（phaseDiscard-
End 弃牌→虚拟 useCard 杀——虚拟用牌原语缺口第五项同源）。xianjian 包
6/36 将入包（6 闭环）。前置：
[xianjian 第五切片](../2026-10-09-ow-wenhui-batch/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js guiyuan）

- enable phaseUse usable:1；filterCard name==="sha"；filter hp<maxHp →
  recover()+draw()。

## 改动（分开陈述）

- **代码已实现**：`XianjianGuiyuan`（tag 148，usable:1 走命令层
  once_used）——activate 门控（he 无杀/满血拒绝）→ SelectCard 杀必选
  → Act：弃置+回复 1（满血饱和）+摸 1。
- **登记差异**：无新增（上游 content recover+draw 一一对应）。
- **登记暂缓**：qijian（虚拟用牌）。
- **转换包**：cards.json 增 林月如（wei 3 女，guiyuan mapping+qijian
  暂缓 reason）+ npack 资产哈希重算。无新状态字段——存档 v16 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`linyueru_guiyuan_discard_sha_
  recover_draw`（弃杀→回复 1（3→4）+摸 1/满血或限一次拒绝）。
- **全量通过**：engine-native **468 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v146 基线 467 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 146→147 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：once_used 命中先于 activate 门控（满血+限一次
  双拒绝路径——断言放宽为两者皆可）；链迭代器 deref 层级按上下文定
  （&u32 vs &&u32——`*c`/`**c` 以编译器指示为准，勿跨臂复制）。

## 剩余

- xianjian 其余 30 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
