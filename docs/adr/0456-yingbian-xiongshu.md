# ADR 0456：YINGBIAN 凶竖（ABI 436）

本批为应变包（`yingbian`）首个权威原语「凶竖」（`xiongshu`，ABI 436）；
贾充（`jin_jiachong`）武将待第二技能「奸回」完成后发布，技能先行入包。
旧 JavaScript 仅作语义来源。

## 决策

- **ABI 436 `resolve_xiongshu(player, target, card, action, stage, skill)`**：
  其他角色出牌阶段开始时（新过滤器
  `owner_alive_other_play_phase_discardable_gte_mark`：他人出牌阶段开始、有手牌、
  拥有者可弃牌数 ≥ 凶竖标记数）四段状态机：
  - stage 0：弃置 X 张牌（X = 凶竖标记数，首次 0 张直接推进；发动后标记 +1，
    标记键 = 技能 def id）；X > 0 时开强制弃牌 choice；
  - stage 1：选择目标一张手牌展示（`RevealCards`）；
  - stage 2：预测 choice（0 会使用 / 1 不会使用，可取消），预测状态写入
    state_blob "XS1" + target + card + action；
  - stage 3（DSL `phase_play_end owner_alive_not_target` 触发）：结算——以
    "牌是否离开目标区域"近似"是否使用"（旧 JS 以 useCard 历史判定，文档化近似）；
    预测对 → 目标受 1 点伤害（无来源）；预测错 → 获得该牌（目标区域或弃牌堆）。
- 技能与既有 yingbian 纯 DSL 草稿（追击/宣穆/强手/仪法 2 等）一同入包。

## 验证

- 新增 `original_yingbian_xiongshu.rs` 2 个行为测试：预测对（牌离开目标区域）
  目标受 1 点伤害且预测状态清空；预测错（牌仍在目标手牌）拥有者获得该牌。
- 打包：11 技能 / 0 武将，`rule_vm_abi_version` 436。
- conformance 3/3 通过，无 golden 漂移；全部 shiji 套件回归绿。
