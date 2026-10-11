# 2026-10-11 T11 反间子项：目标选花色+条件伤害（规则 v203，存档 v47）

任务来源：[进度主稿 T11](../../../architecture/native-progress.md)（领取锁 `8605fe5d`）。上游依据：`standard/skill.js` fanjian:1731——目标 `chooseControl`（♥/♦/♣/♠）→ `gainPlayerCard`（从周瑜手牌强制获得一张）→ 获得牌花色与所选**不符**才 `damage`（花色相符无事）。

## 语义修正

- 原生改前：目标选周瑜一张手牌获得 → **无条件**受 1 点伤害（登记差异：以看牌选牌承载花色选择）。
- 改后：TargetAsk → **SuitAsk**（目标 PickTag 四花色选项 0=♥/1=♦/2=♣/3=♠，上游 controls 序，强制）→ PickAsk（目标选周瑜手牌一张）→ Consume（交牌；`fanjian_suit_tag(获得牌花色) != chosen_suit` 才 emit 1 点伤害）。

## 接线（存档 v47）

- `MachineSnapshot::FanJian` 追加 `chosen_suit: Option<u8>`；`FanJianStageSnap` 追加 `SuitAsk`（stage 名=消费者语义：SuitAsk 消费 target 应答、PickAsk 消费 suit、Consume 消费 pick；snap 恢复臂三询问映射同步）。
- `archive.rs` FanJian（tag 40）编解码追加 chosen_suit（0=None/1+tag）与 stage 四臂；**存档格式 46→47**（v46 字节被版本门禁拒绝，无生产迁移数据）；engine.rs 哈希混合臂加 chosen_suit 字节。

## 验证

| 项 | 结果 |
|---|---|
| 回归 `n08_fanjian_give_and_damage` | (1) 不符受伤（选♠得♥→3 血+四 Tag 选项断言）；(2) 相符不伤（选♥得♥→4 血）；(3) fanjian_pick 挂起中存档往返→恢复重挂→应答继续结算（验证 v47 编解码+恢复臂） |
| 版本回退探针（203→202 复跑 golden） | 2/2 全绿（位移唯一来源=版本字段；反间不在标准场景可达路径） |
| 14 golden 重录 | 全部更新后 2/2 绿（重录依据注释入文件头） |
| 全量 `cargo test --offline -p engine-native --no-fail-fast` | **549/0/1**（2026-10-11 本机） |
| Gate 1b `RUSTFLAGS="-D warnings" cargo check -p engine-native --lib` | exit 0 |
| 规则版本 | 202→**203** |

## 剩余登记（基准差异节更新后）

「内容差异与后续验收」块仅剩：护驾代出闪为弃置+计数而非完整 respond 通道。
