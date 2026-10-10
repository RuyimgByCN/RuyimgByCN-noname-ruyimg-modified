# ADR 0744：官方武将迁移续批——吕布（无双）与黄月英（集智）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0736](0736-native-simayi-guicai-fankui.md)、[0740](0740-native-wusheng-longdan-viewas.md)、进度文档 N08

## 背景与决定

官方迁移链路模板继续复制：**修改型响应需求**（无双）与**锦囊使用触发**（集智）。
官方迁移武将达十名。

### 1. 吕布「无双」（WU_SHUANG，修改型）

你的杀需**连续两张闪**才能抵消；与你决斗的角色每次需打出两张杀：

- `ShaResolveMachine` 增加 `dodges_used` 计数（快照/哈希/存档同步）：
  来源持有无双时需求闪数为 2——第一张闪后仍需第二张，两张齐出才免除伤害；
- 集成于既有的出闪询问循环（需求满足 → 杀弃置结算完成；闪不足 → 伤害）；
- 决斗部分（对手每次需两张杀）涉及 DuelMachine 状态扩展，留待下一批。

### 2. 黄月英「集智」（JI_ZHI，锦囊使用触发）

使用普通锦囊（南蛮/万箭/决斗/铁索/无懈）时摸一张牌——首次启用
`EventKind::CardUsed` 触发元数据；过滤含 view-as 语义（转化后的牌种参与判定）。
快照变体 `MachineSnapshot::JiZhi { owner }`（六处同步）。

### 3. 版本 14 与验收

- 编码变化（Sha.dodges_used / JiZhi 变体）→ `STANDARD_RULESET_VERSION`
  13 → 14，14 golden 重录并通过；runner 复跑退出 0。
- `lvbu_wushuang_sha_needs_two_dodges`：无双下第一张闪后仍需第二张，
  两张齐出完全抵消（hp 不变）、双闪均入弃牌堆、守恒；
- `huangyueying_jizhi_draws_on_trick`：南蛮入侵使用后集智摸一（手牌 =1）、守恒。

## 影响

- 测试 175 项（native_kernel 98），净增 2。规则版本 13 → 14。
- 官方迁移武将十名：张角/司马懿/曹操/郭嘉/貂蝉/甄姬/关羽/赵云/吕布/黄月英。
- N08 剩余：无双决斗部分、更多武将；N12 旧线驱动器闭环（环境边界已实证）。
