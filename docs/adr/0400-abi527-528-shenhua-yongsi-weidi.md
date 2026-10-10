# ADR 0400：shenhua 袁术庸肆/伪帝（ABI 527-528）

本批完整移植 SHENHUA 袁术（`yl_yuanshu`，庸肆 + 伪帝），旧 JavaScript 仅作为语义
来源，不进入运行时。SHENHUA npack 的 `rule_vm_abi_version` 提升至 528。

## ABI 527：`resolve_drlt_yongsi(player, stage)` — 庸肆

- 摸牌阶段（过滤器 `owner_is_target_draw_not_replaced`）：把待执行摸牌数固定为存活
  势力数（`alive_group_count`，复用 `draw_phase_fixed_count` 状态机；旧 JS
  `trigger.num = countGroup()` 同义）。
- 出牌阶段结束：本回合未造成伤害时摸至体力值（`turn.damage_dealt_count` 近似旧 JS
  的出牌阶段伤害历史；判定/摸牌阶段无法造成伤害，文档化近似）；已造成伤害时授予
  既有 `drlt_yongsi1`（3870932875，庸肆弃牌阶段状态），以 `add_skill_until_turn_end`
  表达（旧 JS 于弃牌阶段后移除，回合末移除为文档化近似）。

## ABI 528：`resolve_drlt_weidi` + 过滤器 `owner_is_target_discard_phase_owe_limit_other_qun` — 伪帝

- 主公技（`lord_only`）：弃牌阶段开始（`phase_discard`）触发，新过滤器只在拥有者
  手牌数大于手牌上限且存在其他存活群势力角色时成立，避免空询问。
- 结算三段：选至多 `min(需弃置数, 其他群势力角色数)` 张可弃置手牌（按弃牌保护查询）→
  选等量其他群势力角色（按序配对）→ 逐对赠予。给出的牌离开手牌后，弃牌需求随
  `hand_count - hand_limit` 自然下降，无需额外修正器。
- 弃牌阶段的弃牌数在阶段触发器结算后计算，故赠予先于弃牌提示生效。

## 打包与验证

- `drlt_yongsi.skill` 加入 SHENHUA 包；`yl_yuanshu` 加入 `characters.json`。重建 npack
  并同步 `extensions/shenhua/`（ABI 528、101 技能、27 武将），重建可复现；同步更新
  包级契约断言。
- Golden 覆盖：摸牌数替换为势力数、未造成伤害摸至体力、造成伤害授予弃牌状态、伪帝
  选牌/选目标配对赠予；`original_shenhua_drlt_yongsi.rs` 全绿，38 个 shenhua 测试
  二进制与 conformance 8/8 通过。
