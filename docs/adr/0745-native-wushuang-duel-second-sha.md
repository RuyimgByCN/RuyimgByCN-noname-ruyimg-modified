# ADR 0745：N08 官方迁移补全——吕布无双的决斗部分

日期：2026-09-16
状态：已实施（engine-native）
关联：[0744](0744-native-lvbu-wushuang-huangyueying-jizhi.md)、进度文档 N08

## 背景与决定

[0744](0744-native-lvbu-wushuang-huangyueying-jizhi.md) 落地了无双的杀部分
（需连续两张闪），决斗部分（与你决斗的角色每次需打出两张杀）留待本批补全。

### 实现（DuelMachine.second_pending）

`DuelMachine` 增加 `second_pending` 标志（快照/哈希/存档六处同步）：

- 决斗轮转中，若**对手**持有无双，响应者打出第一张杀后**不轮转**，
  同一响应者再次询问第二张杀（second_pending 置位）；
- 第二张打出后轮转生效（second_pending 复位）；
- 响应者放弃（Pass/无杀）→ 受伤、决斗结束（无双不提供额外机会）；
- 空应答安全：响应读取改为 `answer().first()` 模式，兼容 allow_cancel 路径。

### 验收

`lvbu_wushuang_duel_needs_double_sha`（新增）：0 号（无双）与 2 号决斗——
2 连续打出两张杀（第一张后不轮转、second_pending 置位再询），0 无杀受伤；
两杀均弃置、决斗牌弃置恰一份、守恒。规则版本 14 → 15，14 golden 重录。

## 影响

- 测试 176 项（native_kernel 99），净增 1。
- 官方迁移武将十名全部技能语义完整（无双杀/决斗两部分齐备）。
