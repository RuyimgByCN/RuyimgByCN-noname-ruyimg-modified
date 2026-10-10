# 2026-09-23 第 35 轮：古剑续切片——金莲烛 damageBefore 事件取消原语（规则版本 46）

古剑子包续切片 **5/40 → 7/40**：增补**金莲烛**（`JinlianzhuCancel`，
`HostBehavior` tag 24）——**伤害前置事件取消原语**：当任意角色即将
受到伤害时（`EventKind::DamageBefore` 触发窗口），持烛者被逐个询问
（座位序），确认则弃置一张金莲烛（确定性首张）、**防止此伤害**
（`prevented = true`，与碎盾同一前置窗口机制）、伤害来源摸 1。
监听技授予全体角色（`new_with_extensions` 按 host_skills 声明授予，
过滤器按「手牌持有金莲烛」收敛询问面）。
`STANDARD_RULESET_VERSION` **45 → 46**（`CardKind::Jinlianzhu`
tag 40；14 golden 重录）。

| 验证 | 结果及边界 |
|---|---|
| [engine-native 全量测试](native-tests.log) | **319 passed / 0 failed**（第 34 轮 318 + 金莲烛 1 项）。全工作区 4,885 项通过。测试全绿不代表与上游规则等价。 |
| [探针（v46 重跑）](probes.log) | **22 项 FIXED / 0 项 OPEN** 维持。 |

## 原语细节

- `CardKind::Jinlianzhu`（tag 40，响应牌：主动使用被
  `RejectReason::IllegalTargets` 拒绝）；archive nums 至 40；
  `KNOWN_PROMPTS` + `gj_jinlianzhu_confirm`。
- 监听技定义键 `gj_jinlianzhu_listener`——机器 `def_key` 与目录技能
  名一致，恢复校验走 skill_ok 路径（目录含该技能 ⇔ 声明包在场）。
- 触发元数据：`TriggerMeta { event: DamageBefore, timing: Normal,
  optional: false }`；过滤器：`!d.prevented && d.amount > 0 && owner
  手牌含金莲烛`——已被防止的伤害不再询问（运行时重查生效）。
- 挂起询问派生（snap）：`TargetChoice` 态 = 持有者确认挂起
  （Confirm/Pass 二选）。

## 回归（g1_shahai_gujian 增补 1 项 + 切片计数更新）

- `jinlianzhu_cancels_damage_with_holder_confirm`：确认分支——弃烛 +
  伤害防止 + 来源摸 1，**确认挂起点字节恢复**（Confirm 选择一致性过
  N04-e 校验）；拒绝分支——伤害照常、金莲烛保留、来源不摸；主动
  使用拒绝。
- 切片计数：7/40（+金莲烛 2），deferred 对账 33。

## 边界（如实登记）

- 登记差异：上游经 `chooseToUse` 选牌使用（持有者可有多张时选其一），
  原生为确认后确定性弃置首张。
- 多持有者询问顺序 = 座位序（触发队列 seat_rel 排序），与上游一致；
  任一持有者确认后伤害即防止、后续持有者不再询问（过滤器重查
  `!d.prevented`）。
- 古剑暂缓 33/40：食物 12（计时状态原语）、机关 8、锦囊 6（响应
  拒绝/生成虚拟牌/护甲）、基本 4、装备 2。
- 整包验收仍 **0/37**。
