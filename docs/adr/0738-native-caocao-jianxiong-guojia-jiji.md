# ADR 0738：官方武将迁移续批——曹操（奸雄）与郭嘉（遗计）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0736](0736-native-simayi-guicai-fankui.md)、[0737](0737-native-damage-card-field-oldline-feasibility.md)、进度文档 N08

## 背景与决定

基于 [0737](0737-native-damage-card-field-oldline-feasibility.md) 就位的
`DamageData.card` 前置，迁移两名带新技能机器语义的官方武将。

### 1. 曹操「奸雄」（JIAN_XIONG）

受到伤害后，可获得造成此伤害的牌（`DamageData.card` 前置的直接消费者）：

- 触发：`EventKind::Damage / optional: true`；过滤：目标命中、伤害未防止且 >0、
  **伤害牌已在弃牌堆**（奸雄取弃牌堆中的牌）；
- `JianXiongMachine { owner, card, asked }`：闸门确认后从弃牌堆收入手牌
  （竞态防御：牌不在弃牌堆则跳过）；
- 快照变体 `MachineSnapshot::JianXiong`（六处同步）。

### 2. 郭嘉「遗计」（JI_JI）

受到 1 点伤害后，摸两张牌，然后可将至多两张手牌交给一至两名其他玩家：

- `JiJiMachine` 四阶段：Draw（摸 2）→ Target（询问交给谁，空应答弃权）→
  GiveCard（读目标、询问交哪张牌）→ GiveChooseCard（执行给出并计入，
  至多两张后或手牌空则结束；空应答 = 放弃本张并结束）；
- 快照变体 `MachineSnapshot::JiJi { owner, stage: JiJiStageSnap, target, given }`
  （五值阶段枚举，六处同步）。
- 迁移期间发现并修复：杀的无闪分支未把杀牌弃置（牌滞留桌面的泄漏）——
  该修复使杀牌正常循环（弃牌堆→洗回），长对局的牌序与终局随之确定性改变。

### 3. 规则版本 10 → 11 与验收

- `STANDARD_RULESET_VERSION` 10 → 11，14 golden 全部重录（临时插桩批量
  提取 actual 后回填，并移除插桩）。
- `caocao_jianxiong_takes_damaging_card`：确认后获得造成伤害的杀
  （离开弃牌堆、入手牌、守恒）。
- `guojia_jiji_draws_and_gives_one_card`：确认 → 摸 2 → 给 0 号一张 →
  第二轮空应答弃权 → 终态守恒（CardsGiven 断言）。
- 关联更新：`death_clears_judge_zone` / `dying_rescue_and_death` 的弃牌计数
  修正（杀牌现正常入弃牌堆）；`identity_pure_command_traitor_victory`
  因杀牌循环修复，seed 9 终局由 Traitor 确定性变为 Rebels——断言更新为
  「主公死亡的阵营终局」并注明原因（TRAITOR 终局锚点待重新选种）。
- `STANDARD_GENERALS` 新增曹操、郭嘉条目。

## 影响

- 测试 169 项（native_kernel 92），净增 2（新增 2 项 + 两处计数修正 +
  1 处断言更新）。规则版本 10 → 11；14 golden 重录。
- 官方迁移链路（目录注册 + 触发元数据 + 机器 + 存档 + 验收）已两次复制成功，
  后续武将按同模式扩展。
