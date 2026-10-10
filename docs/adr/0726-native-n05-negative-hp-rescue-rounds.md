# ADR 0726：N05 濒死询问轮次语义与负体力多桃连续救援

日期：2026-09-16
状态：已实施（engine-native）
关联：[0720](0720-native-judgement-order-equip-qualification-v6.md)、[0722](0722-native-n05-active-judgement-matrix-reshuffle.md)、进度文档 N05、[phase19](../operations/phase19-native-baseline.md)

## 背景与决定

进度文档 N05 登记：负体力连续救援逐点恢复缺失。排查发现这不是测试缺口而是**内核
语义缺陷**：`DyingSaveMachine` 的响应者应答一次即永久出队——1 血吃雷（-2 负体力）
时，唯一持桃者用一张桃后即被移出队列，剩余桃无法再使用，负体力实际上不可自救，
与标准濒死询问规则（每轮每人至多一桃、有人用桃且仍濒死则开新一轮）不符。

### 1. 轮次语义修复

`DyingSaveMachine` 增加 `round_used` 标志，濒死询问改为轮次制：

- 每轮自濒死者起按座位序询问，每人至多被询问一次（出桃或放弃）；
- 队列耗尽（整轮问完）时：本轮有人用过桃且场上仍有桃 → 重建新一轮询问队列、
  清零 `round_used`；本轮无人用桃或场上无桃 → 落至死亡结算；
- 用桃即置 `round_used`，随机器状态入快照与哈希（`DyingSave.round_used`，
  五处同步：结构体/构造/snapshot/restore/`push_machine`）。

单桃救援路径行为不变（救回即 Done，不触发轮转）；放弃不再造成无限询问
（整轮无人用桃即死亡）。负体力可由同一持桃者跨轮连续多张救回。

### 2. 回归（净增 2）

- `negative_hp_rescued_by_three_taos`（seed 241）：1 血吃雷 -2 负体力，
  同一持桃者跨三轮询问连用三桃（-2→-1→0→1）；断言每轮询问观察者、
  三次 `DyingSaved`、无 `PlayerDied`、闪电离场与逐牌区守恒。
- `pause_roundtrip_negative_hp_multi_tao_rescue`：同一布阵进恢复矩阵——
  三个濒死询问点各挂起 save→restore，基准线/快照线每点哈希、事件增量、
  终态与守恒一致；死亡分支由既有 `pause_roundtrip_dying_save`（Pass→死亡）
  继续覆盖。

### 3. 规则版本 7

语义变化（濒死询问轮次）+ 状态编码变化（`DyingSave.round_used`）→
`STANDARD_RULESET_VERSION` 6 → 7，14 golden 全部重录并通过。
release runner 复跑（20 局 seed 9000）退出 0，负载聚合与版本 6 一致
（负载脚本不含多桃负体力路径），记录见
[evidence/2026-09-16-native-n09/runner-v7.log](../operations/evidence/2026-09-16-native-n09/runner-v7.log)。

## 影响

- 测试 142 项（native_kernel 81、boundary_snapshot 56、场景 2、源码 2、调试 1），净增 2。
- N05 剩余：无懈×洗牌组合并入通用矩阵、主动改判/天妒放弃（Decline）分支的
  恢复矩阵用例；其余批次项按进度文档优先级推进。
