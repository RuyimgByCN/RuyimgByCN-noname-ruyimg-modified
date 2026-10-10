# ADR 0457：YINGBIAN 贾充闭环（奸回，ABI 437）

本批完成应变包贾充（`jin_jiachong`，2266272721，男/晋/3 血）闭环：继「凶竖」
（ABI 436）后补齐「奸回」（`jianhui`，ABI 437），作为应变包首个完整武将进入
选将池。旧 JavaScript 仅作语义来源。

## 决策

- **ABI 437 `resolve_jianhui(player, source, target, skill)`**：伤害结算后
  （DSL `trigger damage owner_is_source_or_target`）：
  - 拥有者为伤害来源且受伤者与上次相同（连续对同一角色造成伤害）→ 摸一张
    （旧 JS `jianhui_draw`）；
  - 拥有者受伤且来源与上次相同（连续同一来源伤害）→ 来源强制弃一张手牌或装备
    （`ChooseDiscardOneHandOrEquip`，旧 JS 主技能弃牌分支）；
  - blob "JH1" + last_source + last_target 记录最近伤害来源/受伤者，
    每次结算后更新。

## 验证

- `original_yingbian_xiongshu.rs` 新增第 3 个测试：首次无奖励 → 连续同来源开
  强制弃牌 choice（解决后来源弃牌）→ 受伤者不同时来源视角不摸牌。
- 打包：12 技能 / 1 武将（贾充），`rule_vm_abi_version` 437。
- conformance 3/3 通过；engine-core 42 套件全绿。
- 顺带协作补位：Hermes `OwnerSkillPileCount` 迁移（tuple→unit，modifier 以
  skill_def 隐式取牌堆）的 registry/解析/求值三处同步。
