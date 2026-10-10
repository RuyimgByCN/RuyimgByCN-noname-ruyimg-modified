# U05 xianjian 第二切片：沈欺霜冰心（规则 v143，2026-10-09）

xianjian 第二武将切片：**沈欺霜**（pal_shenqishuang，wei 3 女）——冰
心（binxin，global 阶段结束确认）单技闭环；**qixia 暂缓**（花色集集齐
4 色 insertPhase 额外回合——引擎缺口 ADR0646 同源）+ **jianzhen 暂缓**
（shaAfter 逐人 chooseToUse 对同一目标用杀询问链）。xianjian 包 2/36
将入包（2 闭环）。前置：
[ow 收官 + xianjian 首切片](../2026-10-09-ow-closeout-xianjian-slice/README.md)。

## 上游语义核验（源码为准，…/character/xianjian/skill.js binxin）

- trigger global phaseEnd（check 询问式）：filter event.player.hp===1 →
  trigger.player.changeHujia()（+1 护甲；logTarget 该角色）。

## 改动（分开陈述）

- **代码已实现**：`XianjianBinxin`（tag 142，PhaseEnd global 窗口触发、
  optional:true=上游 check 询问式同口径）——filter（阶段结束者存活且
  hp==1，含自己）→ 确认后 Act：change_hujia(阶段结束者, 1)（事件者经
  machine() 分流带入 target）。
- **登记暂缓**：qixia（额外回合缺口）、jianzhen（chooseToUse 链）。
- **转换包**：cards.json 增 沈欺霜（wei 3 女，binxin mapping+双暂缓
  reason）+ npack 资产哈希重算。无新状态字段——存档 v15 不变。

## 验收口径

- **回归已加入**：g1_wudian 1 例——`shenqishuang_binxin_grants_hujia_
  at_low_hp`（1 号压至 1 血后弃牌阶段结束→确认门 observer=0→Confirm→
  +1 甲；满血阶段不触发）。
- **全量通过**：engine-native **464 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v142 基线 463 +1）；依赖 **63/0**
  （content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 142→143 入 canonical_hash——14 场景全 mismatch，
  重录后 replay_scenarios 2/2。
- **测试坑（登记）**：弃牌门在 Play→Discard 转换即挂起（抽 2>上限 1——
  先应答弃牌门）；弃牌应答后阶段收束自动链至 Discard→End 的冰心确认
  门（无需再 NextPhase）；测试探测补丁残留重复应答块——断言「确认门」
  expect 空暴露后去重。

## 剩余

- xianjian 其余 34 武将逐武将纵切；ow 猎空 deferred（离场区基建立项）；
  开局事件基建、虚拟 useCard 原语、U07 duoqi、zhenfa 五种阵法长期暂缓。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（13 文件，相对 noname-next 根）。
