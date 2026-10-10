# U05 ow 纵切批十一：路霸双技+v126 存量清零修复（规则 v128，2026-10-08）

ow 纵切批十一：**路霸**（ow_luba，shu 4 男）双技闭环（全既有先例、无
新基建面）——链钩（liangou，判定挂钩距离/伤害双钩）+ 吸阳（xiyang，
急冻同型翻面回 2 无冰封）。**并修复 v126 遗留缺陷**（TurnState 换回合
原地改字段，乐动三角从未清零）。上游 25 将已闭环第 10 将。前置：
[ow 纵切批十](../2026-10-08-ow-mei-batch/README.md)。

## 上游语义核验（源码为准，…/character/ow/skill.js
liangou/liangou2/xiyang）

- `liangou`：phaseUse usable:1；弃 1 张 he（filterCard 任意）+目标
  （target≠自己）→ judge（suit≠heart 成功）→ 成功则
  target.addTempSkill("liangou2")（至回合结束）+storage.liangou2=路霸。
- `liangou2`：mod globalTo（from=路霸 → to=被钩者距离 -Infinity，经
  get.distance 下限钳 1）+ trigger damageBegin usable:1 forced
  （被钩者受伤害 num++，无 source 条件）。效果存活期=路霸自己回合内
  （使用时机为出牌阶段、addTempSkill 至回合结束）。
- `xiyang`：player phaseEnd（无 forced 有 check=运行时确认询问）；filter
  受伤 && 未翻面；content：turnOver+recover(2)。

## 改动（分开陈述）

- **代码已实现（链钩）**：`TurnState.liangou_hook: Option<PlayerId>` +
  `liangou_bonus_used: bool`（回合级存量）+ `WudianLiangou`（tag 124，
  limits.once_per_turn 走命令层 once_used——发动即消耗不按判定结果）
  两段询问机（弃牌必选→目标必选→弃置+判定（龙帆 v99 形态）→非红桃
  挂钩）；**双钩位**：computed_distance 开头短路（a=当前回合玩家 &&
  b=被钩者 → 距离恒 1，上游 -Infinity 钳距同口径）+ commit_damage
  增幅段（被钩者本回合首次受伤 +1，来源不限=上游无 source 条件；
  liangou_bonus_used 消耗位）。换回合随 TurnState 清零解除。
- **代码已实现（吸阳）**：`WudianXiyang`（tag 125，PhaseEnd+End/begin:
  false 触发，optional:true 确认门——上游无 forced 有 check 同口径；
  filter hp<max && 未翻面）——确认后翻面（内联，急冻同口径登记差异）+
  回复 2。
- **缺陷修复（v126 遗留）**：TurnState 换回合为**原地改字段**而非重建
  （commit_turn_ended 两分支）——乐动三角（yuedong_num/eff/recover）
  自 v126 起从未清零，扩音存量跨回合残留=「每回合重新积攒」语义破坏。
  两分支补显式复位；定向回归 `luxiao_yuedong_storage_resets_next_turn`
  锁定。golden 标准场景不触及（纯复位语义）。
- **登记差异**：吸阳/急冻翻面内联无窗口（v127 口径）；**批十补记：急
  冻触发为直发（optional:false），上游无 forced 会弹确认——已闭环不
  回改，直发对玩家有利且效果确定，登记于此**；链钩判定牌入弃堆（龙帆
  同款）；伤害 +1 为数值增幅不另发伤害事件。
- **转换包**：cards.json 增 路霸（shu 4 男，双技 mapping）+ npack 资产
  哈希重算。存档 **v8→v9**（TurnState 两字段追加）。

## 验收口径

- **回归已加入**：g1_wudian 4 例——`luba_liangou_hook_distance_and_
  damage`（无钩距离 2 被拒/黑桃判定挂钩/杀距离恒 1 达 2 号+首伤 +1
  4→2/决斗无增幅 2→1/usable:1 SkillAlreadyUsed/换回合解除）、
  `luba_liangou_heart_fail_and_xiyang_flip_recover`（红桃判定不挂钩+
  发动已消耗/吸阳确认翻面回 2/跳过回合翻回）、
  `luxiao_yuedong_storage_resets_next_turn`（v126 缺陷回归：跨回合清
  零+新回合重新积攒）。
- **全量通过**：engine-native **448 passed / 0 failed / 1 ignored**
  （26 测试二进制逐项实测；v127 基线 445 +4 含缺陷回归）；依赖
  **63/0**（content-runtime 33 + engine-ai 29 + content-native-loader 1）。
- **golden**：版本 127→128 + TurnState 两字段入 canonical_hash——14
  场景全 mismatch（标准场景无链钩/吸阳，纯版本/零值字段位移+复位语
  义），重录后 replay_scenarios 2/2。
- **测试坑（登记）**：判定牌确定性用 deck 顶构造（rig_next_judge 助
  手：save→移动指定花色到堆顶→restore，王梅 v? 先例）；链钩两段询问
  机的挂起 stage 约定=前移到应答臂（SelectCard→TargetChoice 问牌、
  TargetChoice→Act 问目标）。

## 剩余

- ow 其余 15 武将逐武将纵切；xianjian 36 武将；U07 duoqi/youzhan 暂缓；
  zhenfa 五种阵法。

## 改动文件

见 [source-hashes.txt](source-hashes.txt)（17 文件，相对 noname-next 根）。
