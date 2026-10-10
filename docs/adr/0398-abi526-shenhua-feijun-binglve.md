# ADR 0398：shenhua 王平飞军/兵略（ABI 526）

本批完整移植 SHENHUA 王平（`wangping`，飞军 + 兵略），旧 JavaScript 仅作为语义来源，
不进入运行时。SHENHUA npack 的 `rule_vm_abi_version` 提升至 526。

## ABI 526：`resolve_nzry_feijun` + 过滤器 `owner_is_target_feijun_new_target`

- 飞军：出牌阶段限一次，先由 active 选牌合同弃一张手牌/装备牌，再由
  `resolve_nzry_feijun` 权威状态机结算：
  - 分支可用性：存在手牌数大于拥有者的其他角色（A）或装备区牌数大于拥有者的其他
    角色（B）时提供对应分支，两者均可用时二选一（复用 `choose` 模式）。
  - 目标选择：分支 A 目标须手牌数大于拥有者，分支 B 目标须装备数大于拥有者，屏障
    重校验。
  - 结算：A 由目标自选手牌/装备交予拥有者（通用赠予管线）；B 由目标自选一张装备牌
    弃置（弃置保护查询）。
  - 兵略旗标：目标此前从未被飞军选中（以目标身上的稳定标记 `FEIJUN_TARGET_MARK`
    记录，永久）时，置拥有者本回合 `feijun_new_target` 回合旗标并加标记。
- 兵略：`phase_play_end` 触发 + 新过滤器（拥有者本回合旗标非零），摸两张。旧 JS 的
  自定义 `feijunAfter` 紧随飞军结算，DSL 阶段末结算为文档化近似；重复目标不再触发。
- 回合旗标按玩家位图族实现，随 `clear_per_player_card_counts` 在回合边界清零。

## 打包与验证

- `nzry_feijun.skill` 加入 SHENHUA 包；`wangping` 加入 `characters.json`。重建 npack
  并同步 `extensions/shenhua/`（ABI 526、99 技能、26 武将），重建可复现；同步更新
  包级契约断言。
- Golden 覆盖：手牌分支赠予+阶段末兵略摸二、装备分支弃置、重复目标不再触发兵略、
  取消不动状态；`original_shenhua_nzry_feijun.rs` 全绿，37 个 shenhua 测试二进制与
  conformance 8/8 通过。
