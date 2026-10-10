# ADR 0751：官方武将迁移——大乔（国色）与吕蒙（克己）

日期：2026-09-16
状态：已实施（engine-native）
关联：[0740](0740-native-wusheng-longdan-viewas.md)、[0750](0750-native-lvmeng-keji.md)、进度文档 N08

## 背景与决定

官方迁移链路模板继续：**大乔「国色」（GUO_SE）**——view-as 延时锦囊扩展
（方块牌当乐不思蜀），以及 **吕蒙「克己」（KE_JI）**——条件型手牌上限。

### 1. 国色：view-as 延时锦囊 + 声明牌种传播

此前 view-as 仅覆盖即时牌（杀）。延时锦囊转化需要**声明牌种跨阶段持久化**：

- `GameStateCore` 新增 `declared_kinds: BTreeMap<CardId, CardKind>`；
- `commit_card_used`：`use_as` 为 Some 时写入声明表；
- `JudgePhase.apply_judgement`：效果按声明牌种优先（目录牌种回退），
  消费后移除条目（防洗回残留误判）；
- 声明表入状态哈希（防篡改）与 NGSA 存档（确定性遍历）；
- 出牌校验：国色（方块→乐）优先于武圣/龙胆。

### 2. 吕蒙「克己」（KE_JI）

本回合未使用杀 → 结束阶段无需弃牌：弃牌阶段固有机器构造前检查
克己 + `!sha_used_play_phase`，未出杀则不产生弃牌机器。

### 3. 版本 18 与验收

- 语义 + 编码变化 → `STANDARD_RULESET_VERSION` 17 → 18，14 golden 重录；
- `daqiao_guose_diamond_as_le`（本批配套测试在国色布阵下验证：方块手牌
  经国色当乐使用，置入判定区并在判定阶段按乐生效跳过出牌阶段，守恒）；
- `lvmeng_keji_skips_discard_without_sha`：克己下未出杀回合无弃牌询问、
  手牌保留。

## 影响

- 测试 186 项（native_kernel 107），净增 2。规则版本 17 → 18。
- 官方迁移武将十六名（新增大乔/吕蒙）。
