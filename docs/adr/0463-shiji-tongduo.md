# ADR 0463：SHIJI 通夺（ABI 442）

本批发布刘巴（`liuba`）首技能「通夺」（`tongduo`，ABI 442）；武将待「断笔」
（限定技多目标强制弃牌 + 弃牌堆随机获牌原语）落地后发布，技能先行入包。
旧 JavaScript 仅作语义来源。

## 决策

- **ABI 442 `resolve_tongduo(player, target, card, stage, skill)`**：成为其他
  角色单目标牌目标时（新过滤器
  `owner_is_target_single_target_other_source_once_per_turn`：CardTargeted 事件、
  拥有者为目标、来源为他人、单目标、`turn_fire_count == 0` 每回合一次）三段
  状态机：stage 0 选一名有牌角色（手/装/判三区任一有牌）→ stage 1 该角色强制
  选一张牌 → stage 2 提交 `recast_cards`（复用引擎重铸管线，旧 JS
  `cardRecastable` 以 hej 三区牌近似）。

## 验证

- `original_shiji_zhengsu.rs` 新增第 11 个测试：选角色 → 推进选牌 → 重铸牌进入
  弃牌堆。
- 打包：88 技能 / 27 武将，`rule_vm_abi_version` 502（max 语义）。
- conformance 3/3 通过；我的 5 套件 26 测试全绿。
- 顺带协作补位：Codex 势曹爽 ABI 569 常量重复定义清理、`turn_fire_count` 过滤
  器读取修正。
