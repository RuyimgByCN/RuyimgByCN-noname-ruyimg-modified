# ADR 0736：官方武将迁移——司马懿（鬼才 + 反馈）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0712](0712-native-first-official-skills.md)、[0729](0729-native-n08-armor-bagua-generals-n09-counter-compare.md)、进度文档 N08/N11

## 背景与决定

官方武将迁移第二批：**司马懿（鬼才 + 反馈）**——首个带**新技能机器语义**的
官方武将（张角复用既有判定机器；本批引入伤害触发取牌机器并扩展判定改判资格）。

### 1. 鬼才（GUI_CAI，权限型）

一名角色的判定牌生效前，可打出一张**任意花色手牌**代替之——区别于鬼道的
「黑色手牌/装备牌」限定。判定阶段机器扩展：

- 改判资格队列：鬼道 **或** 鬼才持有者入队（同座位双技能只入队一次）；
- 询问时资格：黑牌（鬼道路径）**或** 鬼才持有者的任意手牌；
- 选项构建：鬼才持有者全部手牌可选；装备区黑色牌仅鬼道路径可用（鬼才限手牌，
  对齐官方）；`active_ask` 恢复推导同步（可见性/恢复一致性由 N04-e 守护）。

### 2. 反馈（FAN_KUI，伤害触发 + OptionalGate）

受到 1 点伤害后，可获得伤害来源的一张牌（手牌优先、其次装备区）：

- `SkillDef` 触发：`EventKind::Damage / Timing::Normal / optional: true`；
- 过滤：目标是持有者、伤害未防止且 >0、来源存活且持有至少一张牌
  （来源死亡/无牌/无来源不触发——避免空询问）；
- 确认经 `OptionalGate`（trigger_confirm），确认后 `FanKuiMachine` 直接执行
  `give_hand_card`（CardsMoved 事件 + CardsGiven 日志）；放弃 = 空应答；
- 新机器快照变体 `MachineSnapshot::FanKui { owner, source, asked }`
  ——六处同步（枚举/restore/validate/哈希/archive 编码/解码）。

### 3. 版本 9 与验收

- 语义 + 编码变化 → `STANDARD_RULESET_VERSION` 8 → 9，14 golden 重录并通过；
  runner 复跑退出 0（判定专项 rewrites=6 finalized=6 gained=2 不变）。
- 回归（native_kernel 3 项）：
  - `simayi_guicai_rewrites_with_any_suit_card`：红桃手牌（鬼道不可用）进入
    鬼才选项并完成改判（Rewritten 归属、旧判定牌归改判者、Finalized 恰一次、守恒）；
  - `simayi_fankui_steals_from_damage_source`：来源无牌不触发；确认后取得
    来源手牌（CardsGiven 断言、守恒）；
  - `simayi_fankui_decline_takes_nothing`：空应答放弃、来源手牌不动。
- `STANDARD_GENERALS` 新增司马懿条目（general_table 测试自动覆盖表完整性）。

## 影响

- 测试 165 项（native_kernel 88），净增 3。规则版本 8 → 9。
- N08 官方迁移路径已打通：目录注册 + 触发元数据 + 机器 + 存档 + 验收的
  全链路可复制到后续武将（郭嘉遗计的「交给其他玩家」、曹操奸雄的「获得造成
  伤害的牌」需 DamageData 携带牌信息，留待后续批次）。
